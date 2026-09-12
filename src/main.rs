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
        _ => {}
    }

    let known_flags = [
        "-a",
        "--all",
        "-b",
        "--board",
        "--stats",
        "-p",
        "--plain",
        "--doctor",
        "--fix-terminal",
        "--no-color",
        "-h",
        "--help",
        "-V",
        "--version",
    ];
    let valued = [
        "-C",
        "--directory",
        "--theme",
        "--config",
        "--screenshot",
        "--color",
        "--filter",
        "-f",
        "--view",
        "--group-by",
        "--sort",
    ];
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a.starts_with('-') {
            let base = a.split('=').next().unwrap_or(a);
            if valued.contains(&base) {
                if !a.contains('=') {
                    i += 1;
                }
            } else if !known_flags.contains(&a.as_str()) {
                eprintln!("harrow: unknown option {a}\n");
                print_usage();
                std::process::exit(2);
            }
        }
        i += 1;
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
    println!(
        "harrow — work a cairn backlog from the terminal\n\n\
         USAGE:\n  harrow [options]\n  harrow config [--write] [--force]\n  harrow themes\n\n\
         OPTIONS:\n\
         \x20 -C, --directory <DIR>  start looking for the project here\n\
         \x20 -a, --all              show everything: finished, dropped, milestones\n\
         \x20 -b, --board            open on the board\n\
         \x20     --stats            open on the statistics\n\
         \x20 -f, --filter <EXPR>    open filtered, in cairn's grammar\n\
         \x20     --view <NAME>      open in one of the project's saved views\n\
         \x20     --group-by <FIELD> milestone, status, type, or any field\n\
         \x20     --sort <KEYS>      sort keys, `-` for descending\n\
         \x20 -p, --plain            print one line per item and exit\n\
         \x20     --theme <NAME>     use a theme for this run (auto, mono, gotham, …)\n\
         \x20     --config <PATH>    read this config file instead of the usual one\n\
         \x20     --color <WHEN>     always, never, or auto\n\
         \x20     --no-color         same as --color never\n\
         \x20     --doctor           check everything harrow depends on\n\
         \x20     --screenshot WxH   render one frame as text and exit\n\
         \x20     --fix-terminal     undo a terminal left in mouse-reporting mode\n\
         \x20 -h, --help             show this help\n\
         \x20 -V, --version          show the version\n\n\
         ENVIRONMENT:\n\
         \x20 HARROW_CONFIG  config file to read\n\
         \x20 HARROW_LOG     append diagnostics to this file\n\
         \x20 NO_COLOR       render without colour\n"
    );
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
    app.pane = if args.iter().any(|a| a == "-b" || a == "--board") {
        harrow::app::Pane::Board
    } else if args.iter().any(|a| a == "--stats") {
        harrow::app::Pane::Stats
    } else {
        harrow::app::Pane::from_name(&startup.config.pane).unwrap_or_default()
    };
    app.group_by =
        flag_value(args, "--group-by").unwrap_or_else(|| startup.config.group_by.clone());
    app.sort = flag_value(args, "--sort").unwrap_or_else(|| startup.config.sort.clone());
    app.filter = flag_value(args, "--filter")
        .or_else(|| flag_value(args, "-f"))
        .unwrap_or_default();
    let view = flag_value(args, "--view").unwrap_or_else(|| startup.config.view.clone());
    app.view = (!view.trim().is_empty()).then_some(view);
    app.readonly = (!on_path(&startup.config.cairn)).then_some(harrow::app::ReadOnly::NoCairn);
    app
}

/// Whether `cairn` can be run at all. Read-only is a legitimate way to work — a
/// backlog is a directory of Markdown — so this decides what harrow says rather
/// than whether it starts.
fn on_path(cairn: &str) -> bool {
    harrow::exec::run(cairn, &["--version"], Duration::from_secs(5)).is_ok()
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
    Ok(())
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
    println!("{}", ui::render_to_string(&mut app, w, h, 0));
    Ok(())
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

fn run_tui(mut startup: Startup, args: &[String]) -> Result<()> {
    let project = open_project(&startup);
    let settings = Settings::from_config(&startup.config);

    let (mut guard, mut terminal) = Guard::new()?;

    // Asked once, in raw mode, before anything else reads stdin. `auto` is the
    // only theme whose choices depend on the answer.
    if startup.theme.source == harrow::theme::Source::Auto && startup.theme.name == "auto" {
        let palette = term::query_palette(Duration::from_millis(150));
        match Theme::from_palette(&palette, "auto") {
            Some(theme) => {
                diag::info(
                    "theme",
                    format!(
                        "terminal answered {} of 18 colour queries; palette read directly",
                        palette.known()
                    ),
                );
                startup.theme = theme;
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
                startup.theme = Theme::auto(dark);
            }
        }
    }

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
            let name = fresh.theme.name.clone();
            app.theme = fresh.theme;
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
