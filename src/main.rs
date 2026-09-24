//! The binary: argument handling, and the event loop that owns the terminal.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use harrow::app::{Action, App, Change, ToastKind};
use harrow::config::Config;
use harrow::engine::{Project, Source};
use harrow::keys::Keymap;
use harrow::runtime::{self, Msg, Settings};
use harrow::term::{self, Guard, Tui};
use harrow::theme::{self, Theme};
use harrow::{diag, doctor, ui};

const TICK: Duration = Duration::from_millis(100);

/// Everything the command line and the config file resolve to between them.
struct Startup {
    config: Config,
    config_path: Option<PathBuf>,
    theme: Theme,
    keymap: Keymap,
    /// Where to look for the project, which is the current directory unless
    /// `-C` said otherwise.
    start: PathBuf,
}

fn main() -> Result<()> {
    diag::init_from_env();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |names: &[&str]| args.iter().any(|a| names.contains(&a.as_str()));

    if has(&["-h", "--help"]) {
        print_usage();
        return Ok(());
    }
    if has(&["-V", "--version"]) {
        println!("harrow {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if has(&["--fix-terminal"]) {
        term::print_reset();
        println!("terminal reset");
        return Ok(());
    }

    // Subcommands come before flag validation so `harrow config --write` works.
    match args.first().map(String::as_str) {
        Some("config") => return cmd_config(&args),
        Some("themes") => return cmd_themes(&args),
        Some("completions") => return cmd_completions(&args),
        Some("man") => {
            print!("{}", harrow::cli::man(env!("CARGO_PKG_VERSION")));
            return Ok(());
        }
        _ => {}
    }

    // One list, in `cli`: the usage text, this check and the completions all
    // read it, so a flag cannot work while nothing says it exists.
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a.starts_with('-') {
            match harrow::cli::flag(a) {
                Some(flag) if flag.wants_value() && !a.contains('=') => i += 1,
                Some(_) => {}
                None => {
                    eprintln!("harrow: unknown option {a}\n");
                    print_usage();
                    std::process::exit(2);
                }
            }
        }
        i += 1;
    }

    // Before the terminal is touched. A name nothing answers to is a usage
    // error, and a usage error printed over an alternate screen that was set
    // up to show it is not one anybody reads.
    if let Some(name) = flag_value(&args, "--lens")
        && harrow::app::Pane::from_name(&name).is_none()
    {
        eprintln!(
            "harrow: no lens called {name:?}\n\nusage: harrow --lens <{}>",
            harrow::app::Pane::ALL
                .iter()
                .map(|p| p.name())
                .collect::<Vec<_>>()
                .join("|")
        );
        std::process::exit(2);
    }

    let startup = resolve(&args);

    if has(&["--doctor"]) {
        std::process::exit(doctor::report(&doctor::run(
            &startup.config,
            startup.config_path.as_deref(),
            &startup.theme,
            &startup.start,
        )));
    }
    if let Some(spec) = flag_value(&args, "--screenshot") {
        return screenshot(&spec, startup, &args);
    }
    if has(&["-p", "--plain"]) {
        return plain(startup, &args);
    }
    run_tui(startup, &args)
}

fn print_usage() {
    print!("{}", harrow::cli::usage());
}

/// Resolve the config, then let the command line override it. Flags win over
/// the file, and the file wins over the defaults.
fn resolve(args: &[String]) -> Startup {
    let explicit = flag_value(args, "--config").map(PathBuf::from);
    let (config, config_path) = Config::load(explicit.as_deref());

    let colour = colour_choice(args);
    let spec = flag_value(args, "--theme").unwrap_or_else(|| config.theme.clone());
    let theme = match colour {
        Colour::Never => Theme::mono(),
        _ => {
            let (theme, err) = Theme::resolve_or_default(&spec);
            if let Some(e) = err {
                diag::warn("theme", e.to_string());
            }
            theme
        }
    };

    let (keymap, key_problems) = Keymap::from_config(&config.keys);
    for p in key_problems {
        diag::warn("keys", p);
    }

    let start = flag_value(args, "-C")
        .or_else(|| flag_value(args, "--directory"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Startup {
        config,
        config_path,
        theme,
        keymap,
        start,
    }
}

/// Fill an `App` from the config and the command line, before anything is read.
fn prepare(startup: &Startup, args: &[String]) -> App {
    let mut app = App::new();
    app.theme = startup.theme.clone();
    app.keymap = startup.keymap.clone();
    app.show_all = startup.config.show_all || args.iter().any(|a| a == "-a" || a == "--all");
    // Every lens by name, and the two that had flags of their own keep them.
    // The needs queue and the log answer *what needs me* and *what changed*,
    // which are the questions somebody returning after a fortnight arrives
    // with — and until this they were the only two with no way in from
    // outside the program. The names come from `Pane` itself, so a sixth lens
    // gets its door by existing rather than by somebody remembering a list.
    app.pane = match flag_value(args, "--lens").and_then(|n| harrow::app::Pane::from_name(&n)) {
        // A name nothing answers to never reaches here: it is refused before
        // the terminal is touched.
        Some(pane) => pane,
        None if args.iter().any(|a| a == "-b" || a == "--board") => harrow::app::Pane::Board,
        None if args.iter().any(|a| a == "--stats") => harrow::app::Pane::Stats,
        None => harrow::app::Pane::from_name(&startup.config.pane).unwrap_or_default(),
    };
    // The axis of whatever is opening. The board keeps its own — a list by
    // milestone beside a board by status is the pair that sharing one would
    // cost — but asking for an axis on the command line while opening the
    // board and getting the list's is just a flag that does nothing.
    match flag_value(args, "--group-by") {
        Some(axis) if app.pane == harrow::app::Pane::Board => app.board_by = axis,
        Some(axis) => app.group_by = axis,
        None => app.group_by = startup.config.group_by.clone(),
    }
    app.sort = flag_value(args, "--sort").unwrap_or_else(|| startup.config.sort.clone());
    app.filter = flag_value(args, "--filter")
        .or_else(|| flag_value(args, "-f"))
        .unwrap_or_default();
    let view = flag_value(args, "--view").unwrap_or_else(|| startup.config.view.clone());
    app.view = (!view.trim().is_empty()).then_some(view);
    app.readonly = (!on_path(&startup.config.cairn)).then_some(harrow::app::ReadOnly::NoCairn);
    app.me = whoami();
    app.can_tick = app.writable() && cairn_can(&startup.config.cairn, "tick");
    app
}

/// Who this is, asked the way cairn asks it — `CAIRN_USER`, then git's
/// configured name — so that harrow and cairn agree about whose work is
/// whose. An agent's writes carry the agent's name, and telling them apart
/// from yours is the whole point of recording either.
fn whoami() -> String {
    if let Ok(v) = std::env::var("CAIRN_USER")
        && !v.trim().is_empty()
    {
        return v.trim().to_string();
    }
    std::process::Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_default()
}

/// Whether `cairn` can be run at all. Read-only is a legitimate way to work — a
/// backlog is a directory of Markdown — so this decides what harrow says rather
/// than whether it starts.
fn on_path(cairn: &str) -> bool {
    harrow::exec::run(cairn, &["--version"], Duration::from_secs(5)).is_ok()
}

/// Whether the cairn on this machine knows a command.
///
/// The first time harrow has had to care *which* cairn it is talking to, and
/// it will not be the last: the format version says what a project is, and
/// nothing until now said what the tool can do. Asked once, because a key
/// that offers something and then reports `unrecognized subcommand` is worse
/// than a key that is not offered.
fn cairn_can(cairn: &str, command: &str) -> bool {
    harrow::exec::run(cairn, &[command, "--help"], Duration::from_secs(5)).is_ok()
}

enum Colour {
    Auto,
    Never,
}

/// `--color`, `--no-color`, `NO_COLOR`, and a terminal that cannot show any.
fn colour_choice(args: &[String]) -> Colour {
    if let Some(when) = flag_value(args, "--color") {
        return match when.as_str() {
            "never" | "no" | "off" => Colour::Never,
            _ => Colour::Auto,
        };
    }
    if args.iter().any(|a| a == "--no-color") {
        return Colour::Never;
    }
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return Colour::Never;
    }
    match std::env::var("TERM").as_deref() {
        Ok("dumb") | Ok("") => Colour::Never,
        _ => Colour::Auto,
    }
}

fn open_project(startup: &Startup) -> Project {
    match Project::discover(&startup.start) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("harrow: {e}");
            eprintln!("        run `cairn init` to start one");
            std::process::exit(1);
        }
    }
}

fn cmd_config(args: &[String]) -> Result<()> {
    let path = flag_value(args, "--config")
        .map(PathBuf::from)
        .unwrap_or_else(Config::path);

    if args.iter().any(|a| a == "--write") {
        let force = args.iter().any(|a| a == "--force");
        match Config::write_default(&path, force) {
            Ok(()) => println!("wrote {}", path.display()),
            Err(e) => {
                eprintln!("harrow: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let startup = resolve(args);
    match &startup.config_path {
        Some(p) => println!("# from {}", p.display()),
        None => {
            println!("# no config file; showing defaults\n# write one with: harrow config --write")
        }
    }
    let overridden = startup.config.overridden();
    if !overridden.is_empty() {
        println!("# set by the file: {}", overridden.join(", "));
    }
    println!(
        "# theme resolves to {} ({})",
        startup.theme.name,
        startup.theme.source.label()
    );
    println!();
    print!("{}", startup.config.to_toml());
    Ok(())
}

/// `harrow completions <shell>`.
fn cmd_completions(args: &[String]) -> Result<()> {
    let shell = args.iter().skip(1).find(|a| !a.starts_with('-'));
    let Some(script) = shell.and_then(|s| harrow::cli::completions(s)) else {
        eprintln!(
            "usage: harrow completions <{}>",
            harrow::cli::SHELLS.join("|")
        );
        std::process::exit(2);
    };
    print!("{script}");
    Ok(())
}

/// `harrow themes [filter]`. There are several hundred Ghostty themes on a
/// typical machine, so a filter is not a luxury.
fn cmd_themes(args: &[String]) -> Result<()> {
    let filter = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.to_lowercase());
    let startup = resolve(args);
    let active = startup.theme.name.to_lowercase();

    let mut shown = 0;
    for (name, source) in theme::available() {
        if let Some(f) = &filter
            && !name.to_lowercase().contains(f)
        {
            continue;
        }
        let mark = if name.to_lowercase() == active {
            "*"
        } else {
            " "
        };
        let status = match Theme::resolve(&name) {
            Ok(_) => String::new(),
            Err(e) => format!("  ({e})"),
        };
        println!("{mark} {:<30} {}{}", name, source.label(), status);
        shown += 1;
    }
    if shown == 0 {
        println!("no theme matches {}", filter.unwrap_or_default());
    }
    Ok(())
}

/// A scriptable one-shot: read the backlog, print a table, exit.
/// A filter naming a field nothing declares matches nothing, and nothing is
/// indistinguishable from an empty backlog. On the command line that is an
/// error with a non-zero status, because a script cannot see an empty pane
/// and wonder about it.
///
/// Named rather than counted: the point is to say *which* field, since the
/// usual cause is a spelling cairn accepts and harrow did not.
fn refuse_a_filter_that_does_not_parse(app: &App) {
    if let Some(why) = app.filter_problem() {
        eprintln!("harrow: {why}");
        if let Some(view) = &app.view {
            eprintln!("harrow: it came from the saved view `{view}` in cairn.toml");
        }
        std::process::exit(2);
    }
}

fn plain(startup: Startup, args: &[String]) -> Result<()> {
    let mut project = open_project(&startup);
    let report = match project.load() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("harrow: {e}");
            std::process::exit(1);
        }
    };
    for w in &report.warnings {
        eprintln!("harrow: {w}");
    }

    let mut app = prepare(&startup, args);
    app.ingest(report);
    refuse_a_filter_that_does_not_parse(&app);

    // Each lens answers a different question, so each prints a different
    // shape. Tab-separated in every case, because the point of printing it at
    // all is that something else can read it.
    match app.pane {
        harrow::app::Pane::Needs => plain_needs(&app),
        harrow::app::Pane::Log => plain_log(&mut app, &startup),
        _ => plain_items(&app),
    }
    Ok(())
}

/// `id`, `status`, `type`, `milestone`, `title`.
fn plain_items(app: &App) {
    for row in &app.rows {
        let harrow::app::Row::Item(i) = row else {
            continue;
        };
        let item = &app.items[*i];
        println!(
            "{}\t{}\t{}\t{}\t{}",
            app.schema.format_id(item.id),
            item.status,
            item.kind,
            item.milestone().unwrap_or(""),
            item.title
        );
    }
}

/// `id`, what kind of question it is, who it is about, and its particulars.
///
/// Nothing needing attention prints nothing and exits 0: an empty queue is an
/// answer rather than a failure, and a caller that wants to branch on it
/// counts lines. That is what makes it composable.
fn plain_needs(app: &App) {
    for question in &app.questions {
        let (kind, who, detail) = match &question.asking {
            harrow::app::Asking::Proposal { field, to, by } => {
                ("proposal", by.as_str(), format!("{field}={to}"))
            }
            harrow::app::Asking::ColdClaim { who, days } => {
                ("cold-claim", who.as_str(), format!("{days} days"))
            }
            harrow::app::Asking::Finished => {
                let ticked = app
                    .items
                    .iter()
                    .find(|i| i.id == question.id)
                    .map(|i| i.criteria())
                    .unwrap_or((0, 0));
                (
                    "finished",
                    "",
                    format!("{} of {} ticked", ticked.0, ticked.1),
                )
            }
            harrow::app::Asking::NothingUnfinished => ("nothing-unfinished", "", String::new()),
            harrow::app::Asking::Unowned { by } => ("unowned", by.as_str(), String::new()),
        };
        println!(
            "{}\t{}\t{}\t{}",
            app.schema.format_id(question.id),
            kind,
            who,
            detail
        );
    }
}

/// `when`, `who`, `id`, `what` — the change, as the repository recorded it.
///
/// The log is the one lens whose answer is not in the item files, so it has to
/// ask Git. `plain` has no event loop, the same way a screenshot has none, so
/// it asks here rather than rendering the placeholder shown while waiting.
fn plain_log(app: &mut App, startup: &Startup) {
    if let Some(Action::Activity) = app.pending() {
        let result = activity(app, startup.config.write_timeout());
        app.show_activity(result);
    }
    if let Some(Err(why)) = &app.moments {
        eprintln!("harrow: {why}");
        std::process::exit(1);
    }
    for moment in app.moments() {
        println!(
            "{}\t{}\t{}\t{}",
            moment.when,
            moment.who,
            app.schema.format_id(moment.id),
            moment.what
        );
    }
}

/// `--screenshot 120x40`. Renders one frame with no terminal at all, which is
/// what makes a bug report reproducible.
fn screenshot(spec: &str, startup: Startup, args: &[String]) -> Result<()> {
    let (w, h) = spec
        .split_once(['x', 'X'])
        .and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))
        .unwrap_or((120u16, 40u16));

    let mut project = open_project(&startup);
    let report = project.load().unwrap_or_else(|e| {
        eprintln!("harrow: {e}");
        std::process::exit(1);
    });

    let mut app = prepare(&startup, args);
    app.ingest(report);
    app.loading = false;
    // A screenshot has no event loop, so what the interface still needs is
    // asked for here. Without it the log lens renders the placeholder it shows
    // before the repository has answered — a frame that is a screenshot of a
    // question rather than of the program.
    if let Some(Action::Activity) = app.pending() {
        let result = activity(&app, startup.config.write_timeout());
        app.show_activity(result);
    }
    println!("{}", ui::render_to_string(&mut app, w, h, 0));
    Ok(())
}

/// What the repository says happened, as the log lens reads it.
///
/// git, directly. Reading history is a read, and harrow already reads this
/// repository's files rather than asking for them — cairn does the same for
/// one item's history.
fn activity(app: &App, timeout: Duration) -> Result<String, String> {
    let dir = app.schema.items_dir();
    harrow::exec::git(
        &[
            "-C",
            &app.schema.root.display().to_string(),
            "log",
            "-n",
            "300",
            "--no-merges",
            "--name-only",
            "--pretty=format:%h\x1f%an\x1f%aI\x1f%s",
            "--",
            &dir.display().to_string(),
        ],
        timeout,
    )
    .map_err(|_| "not a git repository, so there is no history to read".to_string())
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
        if let Some(v) = a.strip_prefix(&format!("{name}=")) {
            return Some(v.to_string());
        }
    }
    None
}

/// Ask the terminal what colours it is actually using, and wear them.
///
/// Only `auto` cares: every other theme names its own colours. The query is
/// OSC 11, OSC 10 and OSC 4 for all sixteen slots, and the answers are what
/// let surfaces be mixed from the real page rather than approximated by
/// naming ANSI slots and hoping.
///
/// One function because both paths need it. Reload used to skip it, which
/// replaced eighteen measured colours with a flat sixteen-slot fallback and
/// made `ctrl-r` quietly downgrade the interface — and asking again is also
/// the only way a terminal theme *changed since harrow started* is noticed,
/// which is the reason somebody presses the key.
fn wear_the_terminal(theme: Theme) -> Theme {
    if theme.source != harrow::theme::Source::Auto || theme.name != "auto" {
        return theme;
    }
    let palette = term::query_palette(Duration::from_millis(150));
    match Theme::from_palette(&palette, "auto") {
        Some(fresh) => {
            diag::info(
                "theme",
                format!(
                    "terminal answered {} of 18 colour queries; palette read directly",
                    palette.known()
                ),
            );
            fresh
        }
        None => {
            // An older terminal that does not answer. Fall back to naming
            // ANSI slots and letting it substitute, which is what `auto`
            // always did.
            let dark = palette
                .background
                .map(harrow::theme::is_dark)
                .unwrap_or_else(|| term::background_is_dark(Duration::from_millis(0)));
            diag::info(
                "theme",
                format!(
                    "terminal did not report its palette; using ANSI slots on a {} background",
                    if dark { "dark" } else { "light" }
                ),
            );
            Theme::auto(dark)
        }
    }
}

fn run_tui(mut startup: Startup, args: &[String]) -> Result<()> {
    let project = open_project(&startup);
    let settings = Settings::from_config(&startup.config);

    let (mut guard, mut terminal) = Guard::new()?;

    startup.theme = wear_the_terminal(startup.theme);

    let mut app = prepare(&startup, args);
    app.theme = startup.theme.clone();

    let (handle, msgs) = runtime::spawn(Box::new(project), settings);
    let result = event_loop(
        &mut terminal,
        &mut app,
        &msgs,
        &handle,
        &mut guard,
        &startup,
        args,
    );
    drop(handle);
    guard.restore();
    result
}

#[allow(clippy::too_many_arguments)]
fn event_loop(
    terminal: &mut Tui,
    app: &mut App,
    msgs: &std::sync::mpsc::Receiver<Msg>,
    handle: &runtime::Handle,
    guard: &mut Guard,
    startup: &Startup,
    args: &[String],
) -> Result<()> {
    let mut tick = 0usize;
    let mut dirty = true;
    let mut last_draw = std::time::Instant::now();
    let mut shown_second = 0u64;

    loop {
        for msg in msgs.try_iter() {
            match msg {
                // Read, and nothing had changed. Deliberately not a redraw: a
                // screen that repainted every three seconds whether or not
                // anything happened is a screen you stop trusting.
                Msg::Idle => continue,
                Msg::Loading => app.loading = true,
                Msg::Loaded(report) => app.ingest(*report),
                Msg::LoadFailed { detail, transient } => app.load_failed(detail, transient),
            }
            dirty = true;
        }
        let had_toast = app.toast.is_some();
        app.tick_clock();
        app.expire_toast();
        // Anything the interface needs before it can draw something true. The
        // log throws away what the repository said whenever the backlog is
        // re-read, and this is what asks again.
        if let Some(action) = app.pending() {
            dirty = true;
            if dispatch(action, app, handle, guard, terminal, startup, args)? {
                return Ok(());
            }
        }
        // A row on its way out leaves when its moment is up, rather than
        // waiting for the next keystroke to notice.
        if app.settle() {
            dirty = true;
        }
        let alive = handle.is_alive();
        if alive != app.watcher_alive {
            app.watcher_alive = alive;
            dirty = true;
        }
        // The parts of the screen that move on their own: the spinner, the
        // "updated N ago" clock, and a toast that has just gone.
        if app.loading || app.now != shown_second || had_toast != app.toast.is_some() {
            shown_second = app.now;
            dirty = true;
        }

        // Both checks belong before the draw and the read. A closed terminal
        // makes crossterm's read spin forever, so the loop must never enter it
        // once the far end has gone.
        if term::terminating() || term::input_closed() {
            return Ok(());
        }

        // Idle, harrow redraws once a second rather than ten times. The
        // once-a-second floor bounds how stale the screen can get if something
        // changes without setting the flag.
        if dirty || last_draw.elapsed() >= Duration::from_secs(1) {
            terminal.draw(|f| ui::draw(f, app, tick))?;
            last_draw = std::time::Instant::now();
            dirty = false;
        }

        if event::poll(TICK)? {
            let action = match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key.code, key.modifiers)
                }
                Event::Mouse(m) => app.handle_mouse(m),
                _ => Action::None,
            };
            dirty = true;
            if dispatch(action, app, handle, guard, terminal, startup, args)? {
                return Ok(());
            }
        }
        tick = tick.wrapping_add(1);
    }
}

/// The only place in the program that touches the outside world on the user's
/// behalf. `App` decides *what* should happen; this decides *how*, and reports
/// the outcome back as a toast.
#[allow(clippy::too_many_arguments)]
fn dispatch(
    action: Action,
    app: &mut App,
    handle: &runtime::Handle,
    guard: &mut Guard,
    terminal: &mut Tui,
    startup: &Startup,
    args: &[String],
) -> Result<bool> {
    match action {
        Action::None => {}
        Action::Quit => return Ok(true),
        Action::Refresh => handle.refresh(),
        Action::Reload => {
            let fresh = resolve(args);
            // Ask the terminal again rather than reusing what it said at
            // startup: reload exists to pick up what changed, and the
            // terminal's own theme is one of the things that can have.
            let theme = wear_the_terminal(fresh.theme);
            let name = theme.name.clone();
            app.theme = theme;
            app.keymap = fresh.keymap;
            app.rebuild();
            // Deliberately no `terminal.clear()`. It issues a cursor-position
            // query and waits for a reply, which stalls for seconds on a
            // terminal that does not answer — and it is not needed: ratatui's
            // diff compares styles, so a changed palette repaints itself.
            app.toast(format!("reloaded — theme {name}"), ToastKind::Good);
        }
        Action::SetMouse(on) => guard.set_mouse(on)?,
        Action::Copy(text) => match copy_to_clipboard(&text) {
            Ok(()) => app.toast(format!("copied {text}"), ToastKind::Good),
            Err(e) => app.toast(format!("clipboard unavailable: {e}"), ToastKind::Bad),
        },
        Action::Open(url) => {
            if let Err(e) = open_in_browser(&url) {
                app.toast(format!("could not open it: {e}"), ToastKind::Bad);
            }
        }
        Action::Edit(path) => {
            // The editor gets the terminal, whole and to itself; harrow takes
            // it back afterwards and re-reads what changed.
            guard.restore();
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "{} {}",
                    startup.config.editor(),
                    shell_quote(&path.display().to_string())
                ))
                .status();
            let (again, again_terminal) = Guard::new()?;
            *guard = again;
            *terminal = again_terminal;
            terminal.clear().ok();
            match status {
                Ok(_) => handle.refresh(),
                Err(e) => app.toast(format!("could not open your editor: {e}"), ToastKind::Bad),
            }
        }
        Action::History(id) => {
            // cairn reads it out of the repository's own history, which is the
            // only place it is. harrow asking git directly would be a second
            // answer to a question cairn already answers.
            let result = harrow::exec::run(
                &startup.config.cairn,
                &["log", &id.to_string(), "--color", "never"],
                startup.config.write_timeout(),
            )
            .map_err(|e| e.to_string());
            app.show_history(id, result);
        }
        Action::Activity => {
            let result = activity(app, startup.config.write_timeout());
            app.show_activity(result);
        }
        Action::Check => {
            // The project's own validator, on the project's own rules. harrow
            // reports what it could not read; this reports what cairn will
            // not accept, and the two are different questions.
            let result = harrow::exec::run(
                &startup.config.cairn,
                &["check", "--color", "never"],
                startup.config.write_timeout(),
            )
            .map_err(|e| e.to_string());
            app.show_check(result);
        }
        Action::Write(change) => run_change(app, handle, &startup.config, change),
    }
    Ok(false)
}

/// Hand a change to cairn. Synchronous on purpose: it takes milliseconds, and a
/// change that had not landed before the next read would show up as the screen
/// silently reverting.
fn run_change(app: &mut App, handle: &runtime::Handle, config: &Config, change: Change) {
    let args: Vec<&str> = change.args.iter().map(String::as_str).collect();
    match harrow::exec::run(&config.cairn, &args, config.write_timeout()) {
        Ok(_) => {
            // So the re-read this causes is not announced back as somebody
            // else's news.
            app.wrote();
            let message = match &change.undo {
                Some(undo) => format!("{} · undo: {undo}", change.describe),
                None => change.describe.clone(),
            };
            app.toast(message, ToastKind::Good);
            handle.refresh();
        }
        Err(e) => {
            diag::error("write", format!("cairn {}: {e}", change.args.join(" ")));
            app.toast(format!("cairn refused: {e}"), ToastKind::Bad);
        }
    }
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Hand a URL to whatever the desktop opens URLs with.
///
/// Detached, with both streams closed: the opener on Linux prints to stderr
/// and can outlive the click, and either would land in the middle of the
/// interface. Not waited on for the same reason.
fn open_in_browser(url: &str) -> std::io::Result<()> {
    // A URL from a body is text somebody else wrote, and it reaches a process
    // as one argument, never a shell. `--` so a url starting with a dash is
    // an argument and not a flag.
    let candidates: [&[&str]; 3] = [&["open"], &["xdg-open"], &["wslview"]];
    let mut last = std::io::Error::other("no way to open a link found");
    for cmd in candidates {
        match Command::new(cmd[0])
            .args(&cmd[1..])
            .arg("--")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => return Ok(()),
            Err(e) => last = e,
        }
    }
    Err(last)
}

fn copy_to_clipboard(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let candidates: [&[&str]; 3] = [
        &["pbcopy"],
        &["wl-copy"],
        &["xclip", "-selection", "clipboard"],
    ];
    let mut last = std::io::Error::other("no clipboard tool found");
    for cmd in candidates {
        let mut child = match Command::new(cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                last = e;
                continue;
            }
        };
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }
    Err(last)
}
