//! Named actions, and the keys bound to them.
//!
//! The keymap is data rather than a `match`, for two reasons. A config file can
//! rebind it, and the help overlay can be generated from it — a help screen that
//! lists the defaults while the user runs something else is worse than no help
//! screen.
//!
//! Two things are deliberately not configurable. Quit is always reachable, and
//! closing an item always raises a confirmation: only the confirmation writes.
//! Everything else that changes an item is a single keystroke on purpose —
//! triage is the whole point, the change is recorded in the repository, and `<`
//! puts it back.

use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Command {
    Down,
    Up,
    PageDown,
    PageUp,
    First,
    Last,
    DetailDown,
    DetailUp,
    ToggleGroup,
    PrevGroup,
    NextGroup,
    ViewBoard,
    ViewBack,
    ViewLens(u8),
    GroupBy,
    CycleGroup,
    Frontier,
    SortBy,
    Read,
    Edit,
    Note,
    History,
    Accept,
    Propose,
    Tick,
    Claim,
    Release,
    Close,
    Reopen,
    New,
    Status,
    Priority,
    Milestone,
    Advance,
    Retreat,
    Copy,
    CopyView,
    Palette,
    Filter,
    Sort,
    Views,
    Facets,
    Back,
    ToggleAll,
    Refresh,
    Reload,
    Diagnostics,
    Check,
    Help,
    ToggleMouse,
    Quit,
}

impl Command {
    pub const ALL: [Command; 55] = [
        Command::Down,
        Command::Up,
        Command::PageDown,
        Command::PageUp,
        Command::First,
        Command::Last,
        Command::DetailDown,
        Command::DetailUp,
        Command::ToggleGroup,
        Command::PrevGroup,
        Command::NextGroup,
        Command::ViewBoard,
        Command::ViewBack,
        Command::ViewLens(1),
        Command::ViewLens(2),
        Command::ViewLens(3),
        Command::ViewLens(4),
        Command::ViewLens(5),
        Command::GroupBy,
        Command::CycleGroup,
        Command::Frontier,
        Command::SortBy,
        Command::Read,
        Command::Edit,
        Command::Note,
        Command::History,
        Command::Accept,
        Command::Propose,
        Command::Tick,
        Command::Claim,
        Command::Release,
        Command::Close,
        Command::Reopen,
        Command::New,
        Command::Status,
        Command::Priority,
        Command::Milestone,
        Command::Advance,
        Command::Retreat,
        Command::Copy,
        Command::CopyView,
        Command::Palette,
        Command::Filter,
        Command::Sort,
        Command::Views,
        Command::Facets,
        Command::Back,
        Command::ToggleAll,
        Command::Refresh,
        Command::Reload,
        Command::Diagnostics,
        Command::Check,
        Command::Help,
        Command::ToggleMouse,
        Command::Quit,
    ];

    /// The stable name a config file uses. Stable is the point: a binding has
    /// to survive a refactor of the code behind it.
    pub fn name(self) -> &'static str {
        match self {
            Command::Down => "down",
            Command::Up => "up",
            Command::PageDown => "page-down",
            Command::PageUp => "page-up",
            Command::First => "first",
            Command::Last => "last",
            Command::DetailDown => "detail-down",
            Command::DetailUp => "detail-up",
            Command::ToggleGroup => "toggle-group",
            Command::PrevGroup => "prev-group",
            Command::NextGroup => "next-group",
            Command::ViewBoard => "view-board",
            Command::ViewBack => "view-back",
            Command::ViewLens(1) => "lens-1",
            Command::ViewLens(2) => "lens-2",
            Command::ViewLens(3) => "lens-3",
            Command::ViewLens(4) => "lens-4",
            Command::ViewLens(_) => "lens-5",
            Command::GroupBy => "group-by",
            Command::CycleGroup => "cycle-group",
            Command::Frontier => "go-to-the-work",
            Command::SortBy => "sort-by",
            Command::Read => "read",
            Command::Edit => "edit",
            Command::Note => "note",
            Command::History => "history",
            Command::Accept => "accept",
            Command::Propose => "propose",
            Command::Tick => "tick",
            Command::Claim => "claim",
            Command::Release => "release",
            Command::Close => "close",
            Command::Reopen => "reopen",
            Command::New => "new",
            Command::Status => "status",
            Command::Priority => "priority",
            Command::Milestone => "milestone",
            Command::Advance => "advance",
            Command::Retreat => "retreat",
            Command::Copy => "copy",
            Command::CopyView => "copy-view",
            Command::Palette => "palette",
            Command::Filter => "filter",
            Command::Sort => "sort",
            Command::Views => "views",
            Command::Facets => "facets",
            Command::Back => "back",
            Command::ToggleAll => "toggle-all",
            Command::Refresh => "refresh",
            Command::Reload => "reload",
            Command::Diagnostics => "diagnostics",
            Command::Check => "check",
            Command::Help => "help",
            Command::ToggleMouse => "toggle-mouse",
            Command::Quit => "quit",
        }
    }

    pub fn from_name(name: &str) -> Option<Command> {
        let name = name.trim().to_lowercase();
        Command::ALL.into_iter().find(|c| c.name() == name)
    }

    pub fn describe(self) -> &'static str {
        match self {
            Command::Down => "move down",
            Command::Up => "move up",
            Command::PageDown => "down a page",
            Command::PageUp => "up a page",
            Command::First => "jump to the first item",
            Command::Last => "jump to the last item",
            Command::DetailDown => "scroll the detail pane down",
            Command::DetailUp => "scroll the detail pane up",
            Command::ToggleGroup => "mark it for the next change — a heading folds",
            Command::PrevGroup => "previous group — column, on the board",
            Command::NextGroup => "next group — column, on the board",
            Command::ViewBoard => "switch between the list, the board and the stats",
            Command::ViewBack => "the lens before this one",
            Command::ViewLens(_) => "go straight to a lens, in the order of the tabs",
            Command::GroupBy => "arrange it by something else",
            Command::CycleGroup => "step to the next arrangement",
            Command::Frontier => "go to where the work is",
            Command::SortBy => "type an order, in --sort's own syntax",
            Command::Read => "read it in the panel, and drive the panel",
            Command::Edit => "open the item in your editor",
            Command::Note => "add a line to the item's body — why, what you tried",
            Command::History => "how this item got the way it is",
            Command::Accept => "accept the change somebody proposed",
            Command::Propose => "in a picker: ask for the change rather than make it",
            Command::Tick => "tick an acceptance criterion that has come true",
            Command::Claim => "claim it — assign it to you and start it",
            Command::Release => "give it back",
            Command::Close => "close it, with a confirm",
            Command::Reopen => "reopen it",
            Command::New => "new item",
            Command::Status => "set the status",
            Command::Priority => "set the priority",
            Command::Milestone => "set the milestone",
            Command::Advance => "move it one status forward",
            Command::Retreat => "move it one status back",
            Command::Copy => "copy the item's reference",
            Command::CopyView => "copy this view as a command line",
            Command::Palette => "every command, by name",
            Command::Filter => "filter, in cairn's own grammar",
            Command::Sort => "order it by something else",
            Command::Views => "look at it the way the project does",
            Command::Facets => "open the filter panel",
            Command::Back => "back out — one press leaves whatever is open",
            Command::ToggleAll => "show everything — finished, dropped, and milestones",
            Command::Refresh => "re-read the backlog now",
            Command::Reload => "reload the config and theme",
            Command::Diagnostics => "diagnostics — what failed, and why",
            Command::Check => "validate the project against its own schema",
            Command::Help => "this help",
            Command::ToggleMouse => "mouse capture — off restores text selection",
            Command::Quit => "quit",
        }
    }

    /// Rows shown in the help overlay, in the order they appear.
    pub fn help_order() -> [Command; 42] {
        [
            Command::Down,
            Command::First,
            Command::PageDown,
            Command::DetailDown,
            Command::PrevGroup,
            Command::ToggleGroup,
            Command::ViewBoard,
            Command::ViewLens(1),
            Command::GroupBy,
            Command::CycleGroup,
            Command::Read,
            Command::Edit,
            Command::Note,
            Command::History,
            Command::Accept,
            Command::Propose,
            Command::Tick,
            Command::Claim,
            Command::Release,
            Command::Status,
            Command::Advance,
            Command::Priority,
            Command::Milestone,
            Command::Close,
            Command::Reopen,
            Command::New,
            Command::Copy,
            Command::Palette,
            Command::Facets,
            Command::Filter,
            Command::Sort,
            Command::SortBy,
            Command::Views,
            Command::CopyView,
            Command::ToggleAll,
            Command::Back,
            Command::Refresh,
            Command::Reload,
            Command::Diagnostics,
            Command::Check,
            Command::Help,
            Command::Quit,
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keymap {
    bindings: Vec<(KeyCode, KeyModifiers, Command)>,
}

impl Default for Keymap {
    fn default() -> Self {
        use Command as C;
        use KeyCode as K;
        let n = KeyModifiers::NONE;
        let ctrl = KeyModifiers::CONTROL;
        Keymap {
            bindings: vec![
                (K::Down, n, C::Down),
                (K::Char('j'), n, C::Down),
                (K::Up, n, C::Up),
                (K::Char('k'), n, C::Up),
                (K::PageDown, n, C::PageDown),
                (K::Char('d'), ctrl, C::PageDown),
                (K::PageUp, n, C::PageUp),
                (K::Char('u'), ctrl, C::PageUp),
                (K::Home, n, C::First),
                (K::Char('g'), n, C::First),
                (K::End, n, C::Last),
                (K::Char('G'), n, C::Last),
                (K::Char('f'), n, C::Facets),
                (K::Char('J'), n, C::DetailDown),
                (K::Char('K'), n, C::DetailUp),
                (K::Char(' '), n, C::ToggleGroup),
                (K::Left, n, C::PrevGroup),
                (K::Char('h'), n, C::PrevGroup),
                (K::Right, n, C::NextGroup),
                (K::Char('l'), n, C::NextGroup),
                (K::Tab, n, C::ViewBoard),
                (K::BackTab, KeyModifiers::SHIFT, C::ViewBack),
                // The digits, positional and in the order the tabs are in.
                // The alternative was initials, and with five lenses they
                // collide with the letters that act on an item — `l` is
                // advance, `s` is status. See 0069 for what this spends.
                (K::Char('1'), n, C::ViewLens(1)),
                (K::Char('2'), n, C::ViewLens(2)),
                (K::Char('3'), n, C::ViewLens(3)),
                (K::Char('4'), n, C::ViewLens(4)),
                (K::Char('5'), n, C::ViewLens(5)),
                (K::Char('v'), n, C::GroupBy),
                (K::Char('v'), ctrl, C::CycleGroup),
                (K::Char('V'), n, C::Views),
                (K::Enter, n, C::Read),
                (K::Char('o'), n, C::Read),
                (K::Char('e'), n, C::Edit),
                (K::Char('N'), n, C::Note),
                (K::Char('H'), n, C::History),
                (K::Char('A'), n, C::Accept),
                (K::Char('t'), n, C::Tick),
                (K::Char('p'), ctrl, C::Propose),
                (K::Char('c'), n, C::Claim),
                (K::Char('C'), n, C::Release),
                (K::Char('x'), n, C::Close),
                (K::Char('u'), n, C::Reopen),
                (K::Char('n'), n, C::New),
                (K::Char('s'), n, C::Status),
                (K::Char('S'), n, C::Sort),
                (K::Char('s'), ctrl, C::SortBy),
                (K::Char('p'), n, C::Priority),
                (K::Char('M'), n, C::Milestone),
                (K::Char('>'), n, C::Advance),
                (K::Char('<'), n, C::Retreat),
                (K::Char('y'), n, C::Copy),
                (K::Char('Y'), n, C::CopyView),
                (K::Char(':'), n, C::Palette),
                (K::Char('/'), n, C::Filter),
                (K::Esc, n, C::Back),
                (K::Char('a'), n, C::ToggleAll),
                (K::Char('r'), n, C::Refresh),
                (K::Char('r'), ctrl, C::Reload),
                (K::Char('?'), n, C::Help),
                (K::Char('q'), n, C::Quit),
                (K::Char('c'), ctrl, C::Quit),
            ],
        }
    }
}

impl Keymap {
    /// Apply the user's `[keys]` table over the defaults. Returns whatever could
    /// not be understood, so the caller can report it where it will be seen.
    pub fn from_config(keys: &BTreeMap<String, String>) -> (Keymap, Vec<String>) {
        let mut map = Keymap::default();
        let mut problems = Vec::new();

        for (spec, action) in keys {
            let Some((code, mods)) = parse_key(spec) else {
                problems.push(format!("[keys] {spec:?} is not a key harrow understands"));
                continue;
            };
            if action.trim().is_empty() || action == "none" {
                map.bindings.retain(|(c, m, _)| !(*c == code && *m == mods));
                continue;
            }
            let Some(command) = Command::from_name(action) else {
                problems.push(format!("[keys] {spec} = {action:?} is not an action"));
                continue;
            };
            // A rebind replaces whatever held that key.
            map.bindings.retain(|(c, m, _)| !(*c == code && *m == mods));
            map.bindings.push((code, mods, command));
        }

        // Quit must survive any config. A user who leaves no way out has made a
        // terminal they cannot leave, and that is not a preference we are
        // obliged to honour.
        if !map.bindings.iter().any(|(_, _, c)| *c == Command::Quit) {
            let restored = map.restore_quit();
            problems.push(format!(
                "[keys] quit was left unbound; restored to {restored}"
            ));
        }

        debug_assert!(
            map.no_key_is_bound_twice(),
            "a key ended up bound to two commands: {:?}",
            map.bindings
        );
        (map, problems)
    }

    /// Bind quit to the first key nothing else is using.
    ///
    /// Appending to a key that is already taken would be worse than useless:
    /// `lookup` finds the first match, so the restored binding would be dead
    /// while the help overlay cheerfully advertised it.
    fn restore_quit(&mut self) -> String {
        let candidates = [
            (KeyCode::Char('q'), KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
            (KeyCode::Char('Q'), KeyModifiers::NONE),
            (KeyCode::Char('q'), KeyModifiers::CONTROL),
            (KeyCode::Esc, KeyModifiers::NONE),
        ];
        for (code, mods) in candidates {
            if !self
                .bindings
                .iter()
                .any(|(c, m, _)| *c == code && *m == mods)
            {
                self.bindings.push((code, mods, Command::Quit));
                return key_name(code, mods);
            }
        }
        // Every escape hatch is taken. Ctrl-C is the one a user is least
        // entitled to reassign away from quitting, so it loses.
        let (code, mods) = (KeyCode::Char('c'), KeyModifiers::CONTROL);
        self.bindings
            .retain(|(c, m, _)| !(*c == code && *m == mods));
        self.bindings.push((code, mods, Command::Quit));
        key_name(code, mods)
    }

    /// No key may resolve to two commands: `lookup` takes the first, so a
    /// duplicate is a binding that silently does nothing.
    /// Every key this map binds, with its modifiers.
    ///
    /// Exists so that the randomised suite can ask the program what its keys
    /// are rather than keeping a list beside it — the list fell behind and
    /// nobody noticed, which is the failure a test cannot report.
    pub fn every_key(&self) -> Vec<(KeyCode, KeyModifiers)> {
        self.bindings.iter().map(|(c, m, _)| (*c, *m)).collect()
    }

    pub fn no_key_is_bound_twice(&self) -> bool {
        let mut seen: Vec<(KeyCode, KeyModifiers)> = Vec::new();
        for (code, mods, _) in &self.bindings {
            if seen.contains(&(*code, *mods)) {
                return false;
            }
            seen.push((*code, *mods));
        }
        true
    }

    pub fn lookup(&self, code: KeyCode, mods: KeyModifiers) -> Option<Command> {
        let (code, mods) = normalise(code, mods);
        self.bindings
            .iter()
            .find(|(c, m, _)| *c == code && *m == mods)
            .map(|(_, _, cmd)| *cmd)
    }

    /// Every key bound to a command, in binding order.
    pub fn keys_for(&self, command: Command) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|(_, _, c)| *c == command)
            .map(|(c, m, _)| key_name(*c, *m))
            .collect()
    }

    /// How much room the overlay gives a row's keys.
    ///
    /// It pads to this and does not truncate, because half a key name is no
    /// use to anybody — so a row wider than this does not get clipped, it
    /// pushes its own description right and drags the next column out of
    /// alignment. `a_help_row_fits_the_column_it_is_drawn_in` holds it.
    pub const HELP_KEY_COLUMN: usize = 14;

    /// `(keys, description)` for the help overlay, generated from the active
    /// bindings rather than from a hardcoded list.
    pub fn help_rows(&self) -> Vec<(String, &'static str)> {
        let mut rows = Vec::new();
        for command in Command::help_order() {
            let keys = self.keys_for(command);
            if keys.is_empty() {
                continue;
            }
            // Pairs read better as one row than as two, and the two separators
            // have to differ or `↑ / k / ↓ / j` looks like four alternatives to
            // one action.
            let keys = match command {
                Command::Down => pair(self.keys_for(Command::Up), keys),
                Command::First => pair(keys, self.keys_for(Command::Last)),
                Command::PageDown => {
                    let first = |c| self.keys_for(c).into_iter().next().unwrap_or_default();
                    pair(vec![first(Command::PageUp)], vec![first(Command::PageDown)])
                }
                Command::DetailDown => pair(self.keys_for(Command::DetailUp), keys),
                Command::PrevGroup => pair(keys, self.keys_for(Command::NextGroup)),
                Command::ViewBoard => pair(keys, self.keys_for(Command::ViewBack)),
                // Five bindings, one row: "1…5" says it and a list of them
                // would be the widest row in the overlay for no more meaning.
                Command::ViewLens(_) => {
                    let bound: Vec<String> = (1..=5)
                        .filter_map(|n| self.keys_for(Command::ViewLens(n)).into_iter().next())
                        .collect();
                    match (bound.first(), bound.last()) {
                        (Some(a), Some(b)) if bound.len() > 1 => format!("{a}…{b}"),
                        _ => bound.join("/"),
                    }
                }
                Command::Advance => pair(self.keys_for(Command::Retreat), keys),
                _ => keys.join("/"),
            };
            let describe = match command {
                Command::Down => "move between items",
                Command::First => "jump to the first or last",
                Command::PageDown => "a screenful up or down",
                Command::DetailDown => "scroll the detail pane",
                Command::PrevGroup => "previous or next group — a column, on the board",
                Command::ViewBoard => "the next lens, or the one before it",
                Command::ViewLens(_) => "go straight to the first, second, … lens",
                Command::Advance => "move it back or forward through the statuses",
                other => other.describe(),
            };
            rows.push((keys, describe));
        }
        // The pointer is not a second-class way to drive this, so the help
        // says what it does rather than leaving it to be discovered.
        rows.push((String::new(), ""));
        rows.push(("click".to_string(), "a tab, a status, a row, a footer hint"));
        rows.push(("double-click".to_string(), "read the item"));
        rows.push((
            "drag".to_string(),
            "a card to another column, which sets its status",
        ));
        rows.push(("scroll".to_string(), "move the pane under the pointer"));
        // Capture is on from the first frame, which means the terminal's own
        // drag-to-select is dead from the first frame. That is a fair trade
        // for dragging a card between columns, but only if the way back is
        // posted somewhere — and `help_rows` skips keyless commands, so the
        // one command that undoes it can never reach this list on its own.
        rows.push((
            format!(":{}", Command::ToggleMouse.name()),
            "give the pointer back to the terminal, to select text",
        ));
        rows
    }

    /// The short hints along the bottom of the screen, each with the command
    /// it advertises so it can be clicked as well as read.
    pub fn footer_hints(&self) -> Vec<(String, &'static str, Command)> {
        let wanted = [
            (Command::Down, "move"),
            (Command::Read, "read"),
            (Command::Claim, "claim"),
            (Command::Status, "status"),
            (Command::Close, "close"),
            (Command::Facets, "filter"),
            (Command::ViewBoard, "lenses"),
            (Command::Help, "help"),
        ];
        wanted
            .into_iter()
            .filter_map(|(command, label)| {
                let key = if command == Command::Down {
                    Some("↑↓".to_string())
                } else {
                    self.keys_for(command).into_iter().next()
                };
                key.map(|k| (k, label, command))
            })
            .collect()
    }

    /// `↑↓`, or `K/J` — the two keys that move a pane, for the hint a pane
    /// holding more than it shows puts on its own edge. Glyphs sit together
    /// the way the footer writes them; letters need the slash to read as two
    /// keys rather than one word.
    pub fn scroll_hint(&self, up: Command, down: Command) -> Option<String> {
        let up = self.keys_for(up).into_iter().next()?;
        let down = self.keys_for(down).into_iter().next()?;
        let glyphs = up.chars().chain(down.chars()).all(|c| !c.is_alphanumeric());
        Some(if glyphs {
            format!("{up}{down}")
        } else {
            format!("{up}/{down}")
        })
    }
}

/// `↑/k, ↓/j` — alternatives within a group, groups separated by a comma.
fn pair(first: Vec<String>, second: Vec<String>) -> String {
    match (first.is_empty(), second.is_empty()) {
        (true, _) => second.join("/"),
        (_, true) => first.join("/"),
        _ => format!("{}, {}", first.join("/"), second.join("/")),
    }
}

/// A shifted character arrives as the uppercase char *and* a SHIFT modifier in
/// some terminals and without it in others. The case carries the information, so
/// SHIFT is dropped for characters and kept for everything else.
fn normalise(code: KeyCode, mods: KeyModifiers) -> (KeyCode, KeyModifiers) {
    let mut mods = mods;
    if matches!(code, KeyCode::Char(_)) {
        mods.remove(KeyModifiers::SHIFT);
    }
    (code, mods)
}

/// `q`, `K`, `ctrl-r`, `alt-x`, `enter`, `space`, `pgdn`, `f5`.
pub fn parse_key(spec: &str) -> Option<(KeyCode, KeyModifiers)> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let mut mods = KeyModifiers::NONE;
    let mut rest = spec;

    loop {
        let lower = rest.to_lowercase();
        let taken = if let Some(r) = lower.strip_prefix("ctrl-").or(lower.strip_prefix("c-")) {
            mods |= KeyModifiers::CONTROL;
            rest.len() - r.len()
        } else if let Some(r) = lower.strip_prefix("alt-").or(lower.strip_prefix("m-")) {
            mods |= KeyModifiers::ALT;
            rest.len() - r.len()
        } else if let Some(r) = lower.strip_prefix("shift-").or(lower.strip_prefix("s-")) {
            mods |= KeyModifiers::SHIFT;
            rest.len() - r.len()
        } else {
            break;
        };
        rest = &rest[taken..];
        if rest.is_empty() {
            return None;
        }
    }

    let code = match rest.to_lowercase().as_str() {
        // The glyphs the help overlay prints parse back, so anything shown on
        // screen can be written into a config file verbatim.
        "↵" => KeyCode::Enter,
        "↑" => KeyCode::Up,
        "↓" => KeyCode::Down,
        "←" => KeyCode::Left,
        "→" => KeyCode::Right,
        "enter" | "return" | "cr" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "space" => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        // Terminals send this as its own key rather than as tab with a
        // modifier, and `normalise` keeps SHIFT for everything that is not a
        // character — so the modifier is part of the name.
        "backtab" | "shift-tab" => {
            mods |= KeyModifiers::SHIFT;
            KeyCode::BackTab
        }
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "insert" | "ins" => KeyCode::Insert,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pgup" | "pageup" => KeyCode::PageUp,
        "pgdn" | "pagedown" => KeyCode::PageDown,
        other => {
            if let Some(n) = other.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()) {
                KeyCode::F(n)
            } else {
                let mut chars = rest.chars();
                let c = chars.next()?;
                if chars.next().is_some() {
                    return None; // more than one character and not a known name
                }
                KeyCode::Char(c)
            }
        }
    };
    Some(normalise(code, mods))
}

/// The inverse, for the help overlay.
pub fn key_name(code: KeyCode, mods: KeyModifiers) -> String {
    let mut out = String::new();
    if mods.contains(KeyModifiers::CONTROL) {
        out.push_str("ctrl-");
    }
    if mods.contains(KeyModifiers::ALT) {
        out.push_str("alt-");
    }
    let base = match code {
        KeyCode::Char(' ') => "space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "↵".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "del".to_string(),
        KeyCode::Insert => "ins".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "pgup".to_string(),
        KeyCode::PageDown => "pgdn".to_string(),
        KeyCode::F(n) => format!("f{n}"),
        other => format!("{other:?}").to_lowercase(),
    };
    out.push_str(&base);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    /// What the palette changed. Every command used to need a key, which is
    /// the zero-sum rule written down as a test: forty-eight commands over
    /// forty-one letters is how a mouse-reporting toggle came to hold one.
    ///
    /// Now a command may live in the palette instead — but only on purpose.
    /// The list below is the whole of what does, so demoting anything else
    /// fails here until somebody adds it and says why.
    #[test]
    fn a_command_has_a_key_unless_it_is_one_of_these() {
        /// Occasional, and reachable by name. A mouse-reporting toggle and a
        /// project validation are not things a hand reaches for by reflex,
        /// and `ctrl-k` for `check` was already the sign that there was
        /// nothing left to spend.
        // In declaration order, which is the order the list below is
        // compared in and the order the palette offers them.
        const BY_NAME_ONLY: &[Command] = &[
            // Where the work is. Worth having and not worth a letter — it is
            // what opening the program already does, and this is for after
            // you have wandered.
            Command::Frontier,
            Command::Diagnostics,
            Command::Check,
            Command::ToggleMouse,
        ];

        let map = Keymap::default();
        let keyless: Vec<&str> = Command::ALL
            .into_iter()
            .filter(|c| map.keys_for(*c).is_empty())
            .map(|c| c.name())
            .collect();
        let expected: Vec<&str> = BY_NAME_ONLY.iter().map(|c| c.name()).collect();
        assert_eq!(keyless, expected, "the keymap gained or lost a demotion");
    }

    /// The overlay is the map, and a map that omits a road is worse than no
    /// map: the reader stops looking. Paging was bound to the four keys
    /// everyone expects and named in no row of it, and `sort-by` was named in
    /// two, and the suite was green through both.
    ///
    /// So the contract, stated once: a command a hand can press is in the
    /// overlay, or folded into the row of the command it pairs with, and
    /// nothing is in it twice. A command with no key is the palette's
    /// business and `a_command_has_a_key_unless_it_is_one_of_these` above is
    /// where that is held.
    #[test]
    fn every_key_a_hand_can_press_is_somewhere_in_the_overlay() {
        /// Two commands, one row, because `↑/k, ↓/j` reads as one idea and
        /// two rows of it read as two. The left of each pair is the one the
        /// row is filed under in `help_order`.
        const FOLDED: &[(Command, Command)] = &[
            (Command::Down, Command::Up),
            (Command::First, Command::Last),
            (Command::PageDown, Command::PageUp),
            (Command::DetailDown, Command::DetailUp),
            (Command::PrevGroup, Command::NextGroup),
            (Command::ViewBoard, Command::ViewBack),
            (Command::Advance, Command::Retreat),
        ];

        let order = Command::help_order();

        let mut seen = Vec::new();
        for command in order {
            assert!(
                !seen.contains(&command),
                "`{}` is in the overlay twice",
                command.name()
            );
            seen.push(command);
        }

        let map = Keymap::default();
        for command in Command::ALL {
            if map.keys_for(command).is_empty() {
                continue;
            }
            // `1…5` is one row standing for five bindings, and only the first
            // is filed; the others are reached through it.
            if matches!(command, Command::ViewLens(n) if n > 1) {
                continue;
            }
            let folded_into = FOLDED.iter().find(|(_, b)| *b == command).map(|(a, _)| *a);
            let reached =
                order.contains(&command) || folded_into.is_some_and(|a| order.contains(&a));
            assert!(
                reached,
                "`{}` has a key and no row: put it in `help_order`, or fold it \
                 into one there and say so in `FOLDED`",
                command.name()
            );
        }
    }

    /// The overlay pads the key column rather than clipping it, so this is
    /// not a cosmetic bound: `pgup/ctrl-u, pgdn/ctrl-d` is twenty-four
    /// characters and it shoved its own description sideways and took the
    /// right-hand column with it.
    #[test]
    fn a_help_row_fits_the_column_it_is_drawn_in() {
        for (keys, _) in Keymap::default().help_rows() {
            assert!(
                keys.chars().count() <= Keymap::HELP_KEY_COLUMN,
                "`{keys}` is {} wide and the column is {}; shorten the row \
                 rather than widening the column, which narrows every \
                 description in the overlay",
                keys.chars().count(),
                Keymap::HELP_KEY_COLUMN
            );
        }
    }

    #[test]
    fn every_command_name_round_trips() {
        for command in Command::ALL {
            assert_eq!(Command::from_name(command.name()), Some(command));
        }
        assert_eq!(Command::from_name("not-an-action"), None);
    }

    #[test]
    fn a_rebind_replaces_what_held_the_key() {
        let (map, problems) = Keymap::from_config(&config(&[("z", "close")]));
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(
            map.lookup(KeyCode::Char('z'), KeyModifiers::NONE),
            Some(Command::Close)
        );
        // The default binding is still there; a rebind adds a key rather than
        // moving one, which is what a user expects.
        assert_eq!(
            map.lookup(KeyCode::Char('x'), KeyModifiers::NONE),
            Some(Command::Close)
        );
    }

    #[test]
    fn a_binding_can_be_removed() {
        let (map, _) = Keymap::from_config(&config(&[("x", "")]));
        assert_eq!(map.lookup(KeyCode::Char('x'), KeyModifiers::NONE), None);
    }

    #[test]
    fn an_unknown_key_or_action_is_reported_and_skipped() {
        let (map, problems) = Keymap::from_config(&config(&[("ctrl-", "quit"), ("z", "explode")]));
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert_eq!(map.lookup(KeyCode::Char('z'), KeyModifiers::NONE), None);
        assert!(problems.iter().any(|p| p.contains("not a key")));
        assert!(problems.iter().any(|p| p.contains("not an action")));
    }

    #[test]
    fn quit_survives_a_config_that_unbinds_every_way_out() {
        let (map, problems) = Keymap::from_config(&config(&[("q", ""), ("ctrl-c", "")]));
        assert_eq!(
            map.lookup(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(Command::Quit),
            "a config must not be able to make harrow impossible to leave"
        );
        assert!(problems.iter().any(|p| p.contains("quit")));
    }

    #[test]
    fn quit_survives_a_config_that_reassigns_every_way_out() {
        let (map, _) = Keymap::from_config(&config(&[("q", "help"), ("ctrl-c", "read")]));
        assert_eq!(
            map.lookup(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(Command::Help)
        );
        let keys = map.keys_for(Command::Quit);
        assert_eq!(keys.len(), 1, "exactly one restored binding: {keys:?}");
        let (code, mods) = parse_key(&keys[0]).expect("the restored key is nameable");
        assert_eq!(
            map.lookup(code, mods),
            Some(Command::Quit),
            "the key the help screen advertises has to be the key that quits"
        );
    }

    #[test]
    fn no_default_or_configured_key_is_bound_twice() {
        assert!(Keymap::default().no_key_is_bound_twice());
        let (map, _) = Keymap::from_config(&config(&[
            ("q", "help"),
            ("ctrl-c", "read"),
            ("Q", "refresh"),
            ("ctrl-q", "copy"),
            ("esc", "filter"),
        ]));
        assert!(map.no_key_is_bound_twice());
        let keys = map.keys_for(Command::Quit);
        let (code, mods) = parse_key(&keys[0]).expect("nameable");
        assert_eq!(map.lookup(code, mods), Some(Command::Quit));
    }

    #[test]
    fn keys_parse_in_the_forms_a_config_would_write() {
        let n = KeyModifiers::NONE;
        let ctrl = KeyModifiers::CONTROL;
        assert_eq!(parse_key("q"), Some((KeyCode::Char('q'), n)));
        assert_eq!(parse_key("C"), Some((KeyCode::Char('C'), n)));
        assert_eq!(parse_key("ctrl-r"), Some((KeyCode::Char('r'), ctrl)));
        assert_eq!(parse_key("C-r"), Some((KeyCode::Char('r'), ctrl)));
        assert_eq!(parse_key("enter"), Some((KeyCode::Enter, n)));
        assert_eq!(parse_key("space"), Some((KeyCode::Char(' '), n)));
        assert_eq!(parse_key("pgdn"), Some((KeyCode::PageDown, n)));
        assert_eq!(parse_key("f5"), Some((KeyCode::F(5), n)));
        assert_eq!(parse_key("/"), Some((KeyCode::Char('/'), n)));
        for bad in ["", "  ", "ctrl-", "notakey", "ctrl-alt-"] {
            assert_eq!(parse_key(bad), None, "accepted {bad:?}");
        }
    }

    #[test]
    fn shift_is_carried_by_the_character_not_the_modifier() {
        let map = Keymap::default();
        assert_eq!(
            map.lookup(KeyCode::Char('C'), KeyModifiers::SHIFT),
            Some(Command::Release)
        );
        assert_eq!(
            map.lookup(KeyCode::Char('C'), KeyModifiers::NONE),
            Some(Command::Release)
        );
        assert_eq!(
            map.lookup(KeyCode::Char('c'), KeyModifiers::NONE),
            Some(Command::Claim),
            "and the lowercase one still means something else"
        );
    }

    #[test]
    fn help_rows_come_from_the_active_bindings() {
        let (map, _) = Keymap::from_config(&config(&[("z", "close"), ("x", "")]));
        let rows = map.help_rows();
        let close = rows
            .iter()
            .find(|(_, d)| d.contains("close it"))
            .expect("close is in the help");
        assert!(
            close.0.contains('z'),
            "help must show the bound key: {close:?}"
        );
        assert!(!close.0.contains('x'), "and not the unbound one: {close:?}");
    }

    #[test]
    fn key_names_round_trip_through_parsing() {
        for (code, mods) in Keymap::default().bindings.iter().map(|(c, m, _)| (*c, *m)) {
            let name = key_name(code, mods);
            assert_eq!(parse_key(&name), Some((code, mods)), "{name}");
        }
    }

    #[test]
    fn control_bindings_resolve() {
        let map = Keymap::default();
        assert_eq!(
            map.lookup(KeyCode::Char('r'), KeyModifiers::CONTROL),
            Some(Command::Reload)
        );
        assert_eq!(
            map.lookup(KeyCode::Char('r'), KeyModifiers::NONE),
            Some(Command::Refresh)
        );
        assert_eq!(
            map.lookup(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Some(Command::Quit)
        );
    }
}
