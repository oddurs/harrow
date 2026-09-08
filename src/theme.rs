//! Colour themes.
//!
//! A theme is a set of *roles* — what a colour is for, not what colour it is —
//! so a palette can be swapped without touching a line of drawing code. Themes
//! come from four places, all through the same parser:
//!
//!   1. `auto`, which maps every role onto the terminal's own ANSI palette and
//!      is the default: harrow should look like the terminal it runs in, not
//!      like somebody else's screenshot;
//!   2. the built-in files below, compiled in and parsed like any other, so a
//!      built-in cannot drift from the format users write;
//!   3. `.toml` files in `~/.config/harrow/themes/`;
//!   4. **Ghostty theme files**, read directly. If you have already picked a
//!      theme for your terminal, harrow can wear it rather than making you
//!      transcribe it.
//!
//! One role a terminal dashboard does not have: the project's own colours.
//! `cairn.toml` may say a bug is red and `doing` is yellow, and where it does,
//! that wins — the theme is what fills in everything the project did not say.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

use crate::schema::{Category, ItemType, Status};

/// Theme files shipped with harrow.
const BUILTIN: &[(&str, &str)] = &[
    ("gotham", include_str!("../themes/gotham.toml")),
    ("night", include_str!("../themes/night.toml")),
    ("paper", include_str!("../themes/paper.toml")),
];

pub const DEFAULT: &str = "auto";

/// The names that resolve without touching the filesystem.
pub const SPECIAL: &[&str] = &["auto", "mono"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Auto,
    Builtin,
    User,
    Ghostty,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Auto => "terminal",
            Source::Builtin => "built-in",
            Source::User => "user",
            Source::Ghostty => "ghostty",
        }
    }
}

/// Every colour the interface can use, named by purpose.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub source: Source,
    pub dark: bool,

    // Structure.
    pub background: Color,
    pub surface: Color,
    pub overlay: Color,
    pub border: Color,
    pub border_focus: Color,
    pub selection: Color,
    /// Draw the selected row in reverse video instead of on `selection`. The
    /// only way to mark a row legibly when the ground colour is unknown.
    pub selection_reverse: bool,

    // Text.
    pub text: Color,
    pub muted: Color,
    pub faint: Color,
    pub heading: Color,

    // Emphasis.
    pub accent: Color,
    pub secondary: Color,

    // What a status means. The four categories, which is what cairn guarantees
    // about a status table however the project has named its columns.
    pub open: Color,
    pub active: Color,
    pub done: Color,
    pub dropped: Color,

    // What the dependency graph says about an item.
    pub blocked: Color,
    pub ready: Color,

    // Messages.
    pub ok: Color,
    pub warn: Color,
    pub error: Color,

    // Furniture.
    pub milestone: Color,
    pub label: Color,
    pub person: Color,

    /// Emphasis by position in a declared enum: `p0` gets the first, `p3` the
    /// last. Generic on purpose — the field is usually `priority`, and a project
    /// that calls it `severity` gets the same treatment for free.
    pub ranks: [Color; 4],

    /// Per-name overrides from a theme file, for a project whose own colours you
    /// would rather not use.
    pub types: BTreeMap<String, Color>,
    pub statuses: BTreeMap<String, Color>,
}

impl Theme {
    /// The colour of a status: what the project asked for, then what the theme
    /// says about that name, then what its category means.
    pub fn status(&self, status: Option<&Status>) -> Color {
        let Some(status) = status else {
            return self.muted;
        };
        if let Some(c) = self.statuses.get(&status.name) {
            return *c;
        }
        if let Some(c) = status.color.as_deref().and_then(parse_color) {
            return c;
        }
        self.category(status.category)
    }

    pub fn category(&self, category: Category) -> Color {
        match category {
            Category::Open => self.open,
            Category::Active => self.active,
            Category::Done => self.done,
            Category::Dropped => self.dropped,
        }
    }

    pub fn item_type(&self, kind: Option<&ItemType>) -> Color {
        let Some(kind) = kind else {
            return self.muted;
        };
        if let Some(c) = self.types.get(&kind.name) {
            return *c;
        }
        kind.color
            .as_deref()
            .and_then(parse_color)
            .unwrap_or(self.secondary)
    }

    /// Colour for the nth of `len` declared values. The first is the one worth
    /// noticing; the last is the one worth ignoring.
    pub fn rank(&self, index: usize, len: usize) -> Color {
        if len == 0 {
            return self.muted;
        }
        let slot = (index * 4) / len.max(1);
        self.ranks[slot.min(3)]
    }

    /// The style for the selected row.
    pub fn selected(&self) -> Style {
        if self.selection_reverse {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
                .bg(self.selection)
                .add_modifier(Modifier::BOLD)
        }
    }

    /// The terminal's own palette.
    ///
    /// Every role is an ANSI slot or `Reset`, never an RGB value — that is the
    /// whole point. The terminal substitutes its configured colours, so harrow
    /// matches whatever theme is already on screen and follows it when it
    /// changes. The cost is that roles wanting a shade *between* two slots do
    /// not get one: `surface` and `background` are the same, and selection is
    /// reverse video. Correct in every terminal beats ideal in one.
    pub fn auto(dark: bool) -> Theme {
        Theme {
            name: "auto".into(),
            source: Source::Auto,
            dark,
            background: Color::Reset,
            surface: Color::Reset,
            overlay: Color::Reset,
            border: Color::DarkGray,
            border_focus: Color::Blue,
            selection: Color::Reset,
            selection_reverse: true,
            text: Color::Reset,
            muted: Color::Gray,
            faint: Color::DarkGray,
            heading: Color::Reset,
            accent: Color::Blue,
            secondary: Color::Cyan,
            open: Color::Gray,
            active: Color::Yellow,
            done: Color::Green,
            dropped: Color::DarkGray,
            blocked: Color::Red,
            ready: Color::Green,
            ok: Color::Green,
            warn: Color::Yellow,
            error: Color::Red,
            milestone: Color::Magenta,
            label: Color::Cyan,
            person: Color::Blue,
            ranks: [Color::Red, Color::Yellow, Color::Cyan, Color::DarkGray],
            types: BTreeMap::new(),
            statuses: BTreeMap::new(),
        }
    }

    /// No colour at all. Emphasis is carried by bold, dim and reverse, which is
    /// what `NO_COLOR`, `TERM=dumb` and a colour-blind reader all need.
    pub fn mono() -> Theme {
        let r = Color::Reset;
        Theme {
            name: "mono".into(),
            source: Source::Auto,
            dark: true,
            background: r,
            surface: r,
            overlay: r,
            border: r,
            border_focus: r,
            selection: r,
            selection_reverse: true,
            text: r,
            muted: r,
            faint: r,
            heading: r,
            accent: r,
            secondary: r,
            open: r,
            active: r,
            done: r,
            dropped: r,
            blocked: r,
            ready: r,
            ok: r,
            warn: r,
            error: r,
            milestone: r,
            label: r,
            person: r,
            ranks: [r; 4],
            types: BTreeMap::new(),
            statuses: BTreeMap::new(),
        }
    }

    /// Fill in from a parsed file, leaving unmentioned roles as they are. A
    /// theme file that only changes the accent is valid and useful.
    fn apply(mut self, file: ThemeFile, name: String, source: Source) -> Theme {
        macro_rules! set {
            ($($field:ident),* $(,)?) => {
                $(if let Some(v) = file.$field.as_deref().and_then(parse_color) {
                    self.$field = v;
                })*
            };
        }
        set!(
            background,
            surface,
            overlay,
            border,
            border_focus,
            selection,
            text,
            muted,
            faint,
            heading,
            accent,
            secondary,
            open,
            active,
            done,
            dropped,
            blocked,
            ready,
            ok,
            warn,
            error,
            milestone,
            label,
            person,
        );
        for (i, raw) in file.ranks.iter().enumerate().take(4) {
            if let Some(c) = parse_color(raw) {
                self.ranks[i] = c;
            }
        }
        if let Some(dark) = file.dark {
            self.dark = dark;
        }
        if let Some(rev) = file.selection_reverse {
            self.selection_reverse = rev;
        } else if file.selection.is_some() {
            // A file that names a selection colour means it to be used.
            self.selection_reverse = false;
        }
        for (name, raw) in &file.types {
            if let Some(c) = parse_color(raw) {
                self.types.insert(name.clone(), c);
            }
        }
        for (name, raw) in &file.statuses {
            if let Some(c) = parse_color(raw) {
                self.statuses.insert(name.clone(), c);
            }
        }
        self.name = file.name.unwrap_or(name);
        self.source = source;
        self
    }

    /// Parse a harrow theme file.
    pub fn from_toml(body: &str, name: &str, source: Source) -> Result<Theme, ThemeError> {
        let file: ThemeFile = toml::from_str(body).map_err(|e| ThemeError::Parse {
            name: name.to_string(),
            detail: e.to_string(),
        })?;
        // A theme starts from a readable base so an incomplete file cannot
        // produce an unreadable screen.
        let base = if file.dark == Some(false) {
            Theme::auto(false)
        } else {
            Theme::auto(true)
        };
        Ok(base.apply(file, name.to_string(), source))
    }

    /// Read a Ghostty theme file: `background`, `foreground`, `palette = N=#hex`.
    pub fn from_ghostty(body: &str, name: &str) -> Result<Theme, ThemeError> {
        let mut palette: BTreeMap<u8, String> = BTreeMap::new();
        let mut keys: BTreeMap<&str, String> = BTreeMap::new();
        for line in body.lines() {
            let line = line.trim();
            // A leading '#' is a comment; a '#' inside a value is a colour.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            if key == "palette" {
                if let Some((slot, colour)) = value.split_once('=')
                    && let Ok(slot) = slot.trim().parse::<u8>()
                {
                    palette.insert(slot, colour.trim().to_string());
                }
            } else {
                let mapped = match key {
                    "background" => "background",
                    "foreground" => "foreground",
                    "selection-background" => "selection",
                    _ => continue,
                };
                keys.insert(mapped, value.to_string());
            }
        }
        if !keys.contains_key("background") && palette.is_empty() {
            return Err(ThemeError::NotGhostty {
                name: name.to_string(),
            });
        }

        let slot = |n: u8| palette.get(&n).cloned();
        // Bright first: the official Gotham port fills its bright slots with
        // background shades, and a port that does that is exactly the one worth
        // honouring rather than second-guessing.
        let pick = |bright: u8, normal: u8| slot(bright).or_else(|| slot(normal));

        let background = keys.get("background").cloned().or_else(|| slot(0));
        let foreground = keys.get("foreground").cloned().or_else(|| slot(7));
        let dark = background
            .as_deref()
            .and_then(parse_color)
            .map(is_dark)
            .unwrap_or(true);

        let file = ThemeFile {
            name: Some(name.to_string()),
            dark: Some(dark),
            background: background.clone(),
            surface: slot(0).or_else(|| background.clone()),
            overlay: slot(8).or_else(|| slot(0)),
            border: slot(8),
            border_focus: pick(12, 4),
            selection: keys.get("selection").cloned().or_else(|| slot(8)),
            selection_reverse: Some(false),
            text: foreground.clone(),
            muted: pick(15, 7),
            faint: pick(8, 0),
            heading: foreground,
            accent: pick(11, 3),
            secondary: pick(14, 6),
            open: pick(15, 7),
            active: pick(11, 3),
            done: pick(10, 2),
            dropped: slot(8),
            blocked: pick(9, 1),
            ready: pick(10, 2),
            ok: pick(10, 2),
            warn: pick(11, 3),
            error: pick(9, 1),
            milestone: pick(13, 5),
            label: pick(14, 6),
            person: pick(12, 4),
            ranks: [pick(9, 1), pick(11, 3), pick(14, 6), slot(8)]
                .into_iter()
                .flatten()
                .collect(),
            types: BTreeMap::new(),
            statuses: BTreeMap::new(),
        };
        Ok(Theme::auto(dark).apply(file, name.to_string(), Source::Ghostty))
    }

    /// Resolve a theme spec: `auto`, `mono`, a built-in name, `ghostty:<name>`,
    /// a user theme name, or a path.
    pub fn resolve(spec: &str) -> Result<Theme, ThemeError> {
        let spec = spec.trim();
        if spec.is_empty() || spec == "auto" {
            return Ok(Theme::auto(true));
        }
        if spec == "mono" || spec == "none" {
            return Ok(Theme::mono());
        }

        if let Some(name) = spec.strip_prefix("ghostty:") {
            return load_ghostty(name);
        }

        if let Some((_, body)) = BUILTIN.iter().find(|(n, _)| *n == spec) {
            return Theme::from_toml(body, spec, Source::Builtin);
        }

        // A path, if it really is one. Not merely "has a dot in it": Ghostty
        // ships themes called `Hopscotch.256`, and treating those as filenames
        // made them unresolvable.
        let as_path = Path::new(spec);
        if spec.contains('/') || as_path.is_file() {
            let body = std::fs::read_to_string(as_path).map_err(|e| ThemeError::Io {
                name: spec.to_string(),
                detail: e.to_string(),
            })?;
            return parse_either(&body, spec, Source::User);
        }

        // A user theme.
        let user = user_theme_dir().join(format!("{spec}.toml"));
        if user.is_file() {
            let body = std::fs::read_to_string(&user).map_err(|e| ThemeError::Io {
                name: spec.to_string(),
                detail: e.to_string(),
            })?;
            return parse_either(&body, spec, Source::User);
        }

        // Whatever the terminal already has under that name.
        load_ghostty(spec)
    }

    /// Resolve, and fall back to `auto` rather than refusing to start. Returns
    /// the error so the caller can report it where the user will see it.
    pub fn resolve_or_default(spec: &str) -> (Theme, Option<ThemeError>) {
        match Theme::resolve(spec) {
            Ok(t) => (t, None),
            Err(e) => (Theme::auto(true), Some(e)),
        }
    }
}

fn parse_either(body: &str, name: &str, source: Source) -> Result<Theme, ThemeError> {
    // A Ghostty theme has no section headers and uses `palette =` lines.
    if body.contains("palette") && !body.contains('[') {
        return Theme::from_ghostty(body, name);
    }
    Theme::from_toml(body, name, source)
}

fn load_ghostty(name: &str) -> Result<Theme, ThemeError> {
    for dir in ghostty_dirs() {
        let path = dir.join(name);
        if path.is_file() {
            let body = std::fs::read_to_string(&path).map_err(|e| ThemeError::Io {
                name: name.to_string(),
                detail: e.to_string(),
            })?;
            return Theme::from_ghostty(&body, name);
        }
    }
    Err(ThemeError::NotFound {
        name: name.to_string(),
    })
}

/// Every theme that can be resolved right now, with where it came from.
pub fn available() -> Vec<(String, Source)> {
    let mut out: Vec<(String, Source)> = vec![
        ("auto".to_string(), Source::Auto),
        ("mono".to_string(), Source::Auto),
    ];
    for (name, _) in BUILTIN {
        out.push((name.to_string(), Source::Builtin));
    }
    collect_dir(&user_theme_dir(), Source::User, &mut out);
    for dir in ghostty_dirs() {
        collect_dir(&dir, Source::Ghostty, &mut out);
    }
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

fn collect_dir(dir: &Path, source: Source, out: &mut Vec<(String, Source)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if source == Source::User {
                name.strip_suffix(".toml").map(str::to_string)
            } else {
                Some(name)
            }
        })
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    for name in names {
        if !out.iter().any(|(n, _)| *n == name) {
            out.push((name, source));
        }
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(xdg).join("harrow");
    }
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join(".config/harrow"),
        None => PathBuf::from(".config/harrow"),
    }
}

pub fn user_theme_dir() -> PathBuf {
    config_dir().join("themes")
}

fn ghostty_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        dirs.push(PathBuf::from(xdg).join("ghostty/themes"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(&home).join(".config/ghostty/themes"));
    }
    dirs.push(PathBuf::from(
        "/Applications/Ghostty.app/Contents/Resources/ghostty/themes",
    ));
    dirs.push(PathBuf::from("/usr/share/ghostty/themes"));
    dirs
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeError {
    NotFound { name: String },
    NotGhostty { name: String },
    Parse { name: String, detail: String },
    Io { name: String, detail: String },
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::NotFound { name } => write!(f, "no theme called {name:?}"),
            ThemeError::NotGhostty { name } => {
                write!(f, "{name:?} is not a theme file harrow understands")
            }
            ThemeError::Parse { name, detail } => {
                write!(f, "theme {name:?}: {}", detail.lines().next().unwrap_or(""))
            }
            ThemeError::Io { name, detail } => write!(f, "theme {name:?}: {detail}"),
        }
    }
}

impl std::error::Error for ThemeError {}

/// The on-disk form. Every colour optional, so a file states only what it means
/// to change.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeFile {
    name: Option<String>,
    dark: Option<bool>,
    selection_reverse: Option<bool>,
    background: Option<String>,
    surface: Option<String>,
    overlay: Option<String>,
    border: Option<String>,
    border_focus: Option<String>,
    selection: Option<String>,
    text: Option<String>,
    muted: Option<String>,
    faint: Option<String>,
    heading: Option<String>,
    accent: Option<String>,
    secondary: Option<String>,
    open: Option<String>,
    active: Option<String>,
    done: Option<String>,
    dropped: Option<String>,
    blocked: Option<String>,
    ready: Option<String>,
    ok: Option<String>,
    warn: Option<String>,
    error: Option<String>,
    milestone: Option<String>,
    label: Option<String>,
    person: Option<String>,
    #[serde(default)]
    ranks: Vec<String>,
    #[serde(default)]
    types: BTreeMap<String, String>,
    #[serde(default)]
    statuses: BTreeMap<String, String>,
}

/// Accepts `#rrggbb`, `#rgb`, `rgb:RR/GG/BB` as xterm writes it, an ANSI index,
/// a colour name, and `reset` for "whatever the terminal already uses".
pub fn parse_color(raw: &str) -> Option<Color> {
    let s = raw.trim().trim_matches('"');
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();

    match lower.as_str() {
        "reset" | "default" | "terminal" | "none" => return Some(Color::Reset),
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" | "purple" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "white" => return Some(Color::Gray),
        "bright-black" | "gray" | "grey" => return Some(Color::DarkGray),
        "bright-red" => return Some(Color::LightRed),
        "bright-green" => return Some(Color::LightGreen),
        "bright-yellow" => return Some(Color::LightYellow),
        "bright-blue" => return Some(Color::LightBlue),
        "bright-magenta" => return Some(Color::LightMagenta),
        "bright-cyan" => return Some(Color::LightCyan),
        "bright-white" => return Some(Color::White),
        _ => {}
    }

    if let Some(rest) = lower.strip_prefix("ansi:") {
        return rest.trim().parse::<u8>().ok().map(Color::Indexed);
    }
    // A bare small number is an ANSI slot, which is how a theme asks to follow
    // the terminal for one particular role.
    if let Ok(n) = lower.parse::<u8>() {
        return Some(Color::Indexed(n));
    }

    if let Some(rest) = lower.strip_prefix("rgb:") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 3 {
            let c = |p: &str| u16::from_str_radix(p, 16).ok().map(|v| scale(v, p.len()));
            if let (Some(r), Some(g), Some(b)) = (c(parts[0]), c(parts[1]), c(parts[2])) {
                return Some(Color::Rgb(r, g, b));
            }
        }
        return None;
    }

    let hex = lower.strip_prefix('#').unwrap_or(&lower);
    match hex.len() {
        3 => {
            let d = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).ok().map(|v| v * 17);
            match (d(0), d(1), d(2)) {
                (Some(r), Some(g), Some(b)) => Some(Color::Rgb(r, g, b)),
                _ => None,
            }
        }
        6 => {
            let d = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
            match (d(0), d(2), d(4)) {
                (Some(r), Some(g), Some(b)) => Some(Color::Rgb(r, g, b)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// xterm writes 1, 2 or 4 hex digits per channel; normalise to 8 bits.
fn scale(value: u16, digits: usize) -> u8 {
    match digits {
        1 => (value as u8) * 17,
        2 => value as u8,
        _ => (value >> 8) as u8,
    }
}

/// Perceived lightness, for deciding whether a background is dark.
pub fn is_dark(color: Color) -> bool {
    match color {
        Color::Rgb(r, g, b) => {
            let l = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
            l < 128.0
        }
        Color::Black | Color::DarkGray => true,
        Color::White | Color::Gray => false,
        Color::Indexed(n) => n < 8 || (16..=231).contains(&n) && n < 100,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Schema;
    use crate::testkit;

    #[test]
    fn auto_never_uses_a_colour_the_terminal_did_not_choose() {
        // The entire point of `auto`: one RGB value here and the theme stops
        // following the terminal.
        let t = Theme::auto(true);
        let mut all = vec![
            t.background,
            t.surface,
            t.overlay,
            t.border,
            t.border_focus,
            t.selection,
            t.text,
            t.muted,
            t.faint,
            t.heading,
            t.accent,
            t.secondary,
            t.open,
            t.active,
            t.done,
            t.dropped,
            t.blocked,
            t.ready,
            t.ok,
            t.warn,
            t.error,
            t.milestone,
            t.label,
            t.person,
        ];
        all.extend_from_slice(&t.ranks);
        for c in all {
            assert!(
                !matches!(c, Color::Rgb(..)),
                "auto must not name an absolute colour, found {c:?}"
            );
        }
    }

    #[test]
    fn mono_emits_no_colour_at_all() {
        let t = Theme::mono();
        let mut all = vec![t.text, t.accent, t.done, t.blocked, t.milestone, t.border];
        all.extend_from_slice(&t.ranks);
        for c in all {
            assert_eq!(c, Color::Reset, "mono must be colourless");
        }
        assert!(t.selection_reverse, "mono has to mark selection somehow");
    }

    #[test]
    fn every_builtin_parses() {
        for (name, body) in BUILTIN {
            let t = Theme::from_toml(body, name, Source::Builtin)
                .unwrap_or_else(|e| panic!("built-in {name} does not parse: {e}"));
            assert!(t.name.to_lowercase().replace(' ', "-").contains(name));
            assert_ne!(t.done, t.blocked, "{name}: finished and stuck must differ");
        }
    }

    #[test]
    fn the_project_gets_the_colour_it_asked_for() {
        // A theme names roles; cairn.toml names statuses and types. Where the
        // project has an opinion, it wins.
        let schema = testkit::schema();
        let t = Theme::from_toml("done = \"#00ff00\"\n", "test", Source::User).expect("parses");
        assert_eq!(
            t.item_type(schema.item_type("bug")),
            Color::Red,
            "cairn.toml says a bug is red"
        );
        assert_eq!(
            t.status(schema.status("doing")),
            Color::Yellow,
            "and that doing is yellow"
        );
    }

    #[test]
    fn a_theme_file_can_overrule_the_project() {
        let schema = testkit::schema();
        let t =
            Theme::from_toml("[types]\nbug = \"#ff00ff\"\n", "test", Source::User).expect("parses");
        assert_eq!(
            t.item_type(schema.item_type("bug")),
            Color::Rgb(255, 0, 255)
        );
    }

    #[test]
    fn a_status_with_no_colour_falls_back_to_what_its_category_means() {
        let schema = Schema::parse(
            "[[status]]\nname = \"shipped\"\ncategory = \"done\"\n",
            PathBuf::from("/tmp"),
        )
        .expect("parses");
        let t = Theme::auto(true);
        assert_eq!(t.status(schema.status("shipped")), t.done);
    }

    #[test]
    fn ranks_run_from_notice_this_to_ignore_this() {
        let t = Theme::auto(true);
        assert_eq!(t.rank(0, 4), t.ranks[0]);
        assert_eq!(t.rank(3, 4), t.ranks[3]);
        assert_eq!(
            t.rank(0, 2),
            t.ranks[0],
            "a two-value enum still spans them"
        );
        assert_eq!(t.rank(9, 4), t.ranks[3], "out of range is the quiet end");
        assert_eq!(t.rank(0, 0), t.muted, "an enum with no values has no rank");
    }

    #[test]
    fn a_partial_theme_file_inherits_the_rest() {
        let t = Theme::from_toml("accent = \"#ff0000\"\n", "partial", Source::User)
            .expect("a one-line theme is valid");
        assert_eq!(t.accent, Color::Rgb(255, 0, 0));
        assert_eq!(t.text, Theme::auto(true).text, "unset roles inherit");
    }

    #[test]
    fn a_broken_theme_file_reports_rather_than_panicking() {
        let err = Theme::from_toml("accent = ", "broken", Source::User).unwrap_err();
        assert!(err.to_string().contains("broken"), "{err}");

        let t = Theme::from_toml("accent = \"not-a-colour\"\n", "odd", Source::User)
            .expect("a bad value is not a bad file");
        assert_eq!(t.accent, Theme::auto(true).accent);
    }

    #[test]
    fn unknown_keys_are_rejected_so_a_typo_is_visible() {
        let err = Theme::from_toml("acccent = \"#fff\"\n", "typo", Source::User).unwrap_err();
        assert!(err.to_string().contains("typo"), "{err}");
    }

    #[test]
    fn colours_parse_in_every_form_a_theme_might_use() {
        assert_eq!(parse_color("#ff8800"), Some(Color::Rgb(255, 136, 0)));
        assert_eq!(parse_color("#f80"), Some(Color::Rgb(255, 136, 0)));
        assert_eq!(parse_color("ff8800"), Some(Color::Rgb(255, 136, 0)));
        assert_eq!(parse_color("rgb:ff/88/00"), Some(Color::Rgb(255, 136, 0)));
        assert_eq!(
            parse_color("rgb:ffff/8888/0000"),
            Some(Color::Rgb(255, 136, 0))
        );
        assert_eq!(parse_color("4"), Some(Color::Indexed(4)));
        assert_eq!(parse_color("ansi:12"), Some(Color::Indexed(12)));
        assert_eq!(parse_color("reset"), Some(Color::Reset));
        assert_eq!(parse_color("bright-cyan"), Some(Color::LightCyan));
        for bad in ["", "   ", "#12", "#1234567", "zzz", "rgb:1/2"] {
            assert_eq!(parse_color(bad), None, "accepted {bad:?}");
        }
    }

    #[test]
    fn a_ghostty_theme_becomes_a_usable_theme() {
        let body = include_str!("../tests/fixtures/ghostty-gotham");
        let t = Theme::from_ghostty(body, "gotham").expect("gotham parses");
        assert_eq!(t.source, Source::Ghostty);
        assert!(t.dark, "gotham is a dark theme");
        assert_eq!(t.background, Color::Rgb(0x0a, 0x0f, 0x14));
        assert!(
            !t.selection_reverse,
            "a full palette can colour the selection"
        );
        assert_ne!(t.done, t.blocked, "finished and stuck must differ");
    }

    #[test]
    fn something_that_is_not_a_theme_is_rejected() {
        let err = Theme::from_ghostty("hello\nworld\n", "nope").unwrap_err();
        assert!(matches!(err, ThemeError::NotGhostty { .. }), "{err:?}");
    }

    #[test]
    fn resolve_finds_the_names_that_need_no_filesystem() {
        assert_eq!(Theme::resolve("auto").unwrap().source, Source::Auto);
        assert_eq!(Theme::resolve("").unwrap().name, "auto");
        assert_eq!(Theme::resolve("mono").unwrap().name, "mono");
        assert_eq!(Theme::resolve("night").unwrap().source, Source::Builtin);
        assert!(Theme::resolve("definitely-not-a-theme").is_err());
    }

    #[test]
    fn an_unresolvable_theme_falls_back_instead_of_failing() {
        let (theme, err) = Theme::resolve_or_default("definitely-not-a-theme");
        assert_eq!(theme.name, "auto");
        assert!(err.is_some(), "the failure must still be reportable");
    }
}
