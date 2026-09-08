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
    ToggleGroup,
    PrevGroup,
    NextGroup,
    ViewBoard,
    GroupBy,
    Read,
    Edit,
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
    Filter,
    Back,
    ToggleClosed,
    Refresh,
    Reload,
    Diagnostics,
    Help,
    ToggleMouse,
    Quit,
}

impl Command {
    pub const ALL: [Command; 33] = [
        Command::Down,
        Command::Up,
        Command::PageDown,
        Command::PageUp,
        Command::First,
        Command::Last,
        Command::ToggleGroup,
        Command::PrevGroup,
        Command::NextGroup,
        Command::ViewBoard,
        Command::GroupBy,
        Command::Read,
        Command::Edit,
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
        Command::Filter,
        Command::Back,
        Command::ToggleClosed,
        Command::Refresh,
        Command::Reload,
        Command::Diagnostics,
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
            Command::ToggleGroup => "toggle-group",
            Command::PrevGroup => "prev-group",
            Command::NextGroup => "next-group",
            Command::ViewBoard => "view-board",
            Command::GroupBy => "group-by",
            Command::Read => "read",
            Command::Edit => "edit",
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
            Command::Filter => "filter",
            Command::Back => "back",
            Command::ToggleClosed => "toggle-closed",
            Command::Refresh => "refresh",
            Command::Reload => "reload",
            Command::Diagnostics => "diagnostics",
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
            Command::ToggleGroup => "collapse or expand a group",
            Command::PrevGroup => "previous group — column, on the board",
            Command::NextGroup => "next group — column, on the board",
            Command::ViewBoard => "switch between the list and the board",
            Command::GroupBy => "group by something else",
            Command::Read => "read the item in full",
            Command::Edit => "open the item in your editor",
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
            Command::Filter => "filter, in cairn's own grammar",
            Command::Back => "back out — clear the filter, close an overlay",
            Command::ToggleClosed => "show finished and dropped items too",
            Command::Refresh => "re-read the backlog now",
            Command::Reload => "reload the config and theme",
            Command::Diagnostics => "diagnostics — what failed, and why",
            Command::Help => "this help",
            Command::ToggleMouse => "mouse capture — off restores text selection",
            Command::Quit => "quit",
        }
    }

    /// Rows shown in the help overlay, in the order they appear.
    pub fn help_order() -> [Command; 25] {
        [
            Command::Down,
            Command::First,
            Command::PrevGroup,
            Command::ToggleGroup,
            Command::ViewBoard,
            Command::GroupBy,
            Command::Read,
            Command::Edit,
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
            Command::Filter,
            Command::ToggleClosed,
            Command::Back,
            Command::Refresh,
            Command::Diagnostics,
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
                (K::Char(' '), n, C::ToggleGroup),
                (K::Left, n, C::PrevGroup),
                (K::Right, n, C::NextGroup),
                (K::Tab, n, C::ViewBoard),
                (K::Char('v'), n, C::GroupBy),
                (K::Enter, n, C::Read),
                (K::Char('o'), n, C::Read),
                (K::Char('e'), n, C::Edit),
                (K::Char('c'), n, C::Claim),
                (K::Char('C'), n, C::Release),
                (K::Char('x'), n, C::Close),
                (K::Char('u'), n, C::Reopen),
                (K::Char('n'), n, C::New),
                (K::Char('s'), n, C::Status),
                (K::Char('p'), n, C::Priority),
                (K::Char('M'), n, C::Milestone),
                (K::Char('l'), n, C::Advance),
                (K::Char('>'), n, C::Advance),
                (K::Char('h'), n, C::Retreat),
                (K::Char('<'), n, C::Retreat),
                (K::Char('y'), n, C::Copy),
                (K::Char('/'), n, C::Filter),
                (K::Esc, n, C::Back),
                (K::Char('a'), n, C::ToggleClosed),
                (K::Char('r'), n, C::Refresh),
                (K::Char('r'), ctrl, C::Reload),
                (K::Char('D'), n, C::Diagnostics),
                (K::Char('?'), n, C::Help),
                (K::Char('m'), n, C::ToggleMouse),
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
                Command::PrevGroup => pair(keys, self.keys_for(Command::NextGroup)),
                Command::Advance => pair(self.keys_for(Command::Retreat), keys),
                _ => keys.join("/"),
            };
            let describe = match command {
                Command::Down => "move between items",
                Command::First => "jump to the first or last",
                Command::PrevGroup => "previous or next group — a column, on the board",
                Command::Advance => "move it back or forward through the statuses",
                other => other.describe(),
            };
            rows.push((keys, describe));
        }
        rows.push(("click".to_string(), "select a row, or fold a group"));
        rows
    }

    /// The short hints along the bottom of the screen.
    pub fn footer_hints(&self) -> Vec<(String, &'static str)> {
        let wanted = [
            (Command::Down, "move"),
            (Command::Read, "read"),
            (Command::Claim, "claim"),
            (Command::Status, "status"),
            (Command::Close, "close"),
            (Command::Filter, "filter"),
            (Command::ViewBoard, "board"),
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
                key.map(|k| (k, label))
            })
            .collect()
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

    #[test]
    fn the_defaults_cover_every_command() {
        let map = Keymap::default();
        for command in Command::ALL {
            assert!(
                !map.keys_for(command).is_empty(),
                "{} has no default key",
                command.name()
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
