//! User configuration.
//!
//! Read from `$HARROW_CONFIG`, else `~/.config/harrow/config.toml`, else
//! defaults. This is *your* configuration — how you like to read a backlog —
//! and is deliberately separate from `cairn.toml`, which is the project's and
//! is shared with everyone else working on it.
//!
//! Nothing here is fatal. A missing file is the normal case, a malformed one is
//! reported and stepped over, and an unknown key is a warning rather than a
//! refusal — a config written for a newer harrow has to keep working on an
//! older one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::diag;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// `auto`, `mono`, a built-in name, `ghostty:<name>`, or a path.
    pub theme: String,
    /// What the rows are grouped under: a field name, or `none`.
    pub group_by: String,
    /// Sort keys within a group, in cairn's `--sort` spelling.
    pub sort: String,
    /// Start in one of the project's saved views.
    pub view: String,
    /// Show everything on startup, as `--all` does: finished, dropped, and the
    /// containers work belongs to.
    pub show_all: bool,
    /// Which pane to open on: `list`, `board` or `stats`.
    pub pane: String,
    /// Seconds between checks for a changed backlog.
    pub refresh_secs: u64,
    /// Watch the filesystem as well as polling it, so a change made elsewhere
    /// shows up at once.
    pub watch: bool,
    /// How long to give `cairn` to carry out a change before giving up on it.
    pub write_ms: u64,
    /// The `cairn` to run for writes. A path, if it is not on `PATH`.
    pub cairn: String,
    /// What `e` opens. Falls back to `$VISUAL`, `$EDITOR`, then `vi`.
    pub editor: String,
    /// Key → action name.
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: crate::theme::DEFAULT.to_string(),
            group_by: "milestone".to_string(),
            sort: String::new(),
            view: String::new(),
            show_all: false,
            pane: "list".to_string(),
            refresh_secs: 3,
            watch: true,
            write_ms: 8000,
            cairn: "cairn".to_string(),
            editor: String::new(),
            keys: BTreeMap::new(),
        }
    }
}

/// Top-level keys harrow knows. Anything else is warned about rather than
/// rejected, so a config written for a newer version still loads here.
const KNOWN: &[&str] = &[
    "theme",
    "group_by",
    "sort",
    "view",
    "show_all",
    "pane",
    "refresh_secs",
    "watch",
    "write_ms",
    "cairn",
    "editor",
    "keys",
];

impl Config {
    /// Where the config would be read from, if it exists.
    pub fn path() -> PathBuf {
        if let Some(explicit) = std::env::var_os("HARROW_CONFIG").filter(|v| !v.is_empty()) {
            return PathBuf::from(explicit);
        }
        crate::theme::config_dir().join("config.toml")
    }

    /// Load, reporting problems rather than failing. Returns the config and the
    /// path it came from, if any.
    pub fn load(explicit: Option<&Path>) -> (Config, Option<PathBuf>) {
        let path = match explicit {
            Some(p) => p.to_path_buf(),
            None => Self::path(),
        };
        if !path.exists() {
            if explicit.is_some() {
                diag::warn("config", format!("{} does not exist", path.display()));
            }
            return (Config::default(), None);
        }
        let body = match std::fs::read_to_string(&path) {
            Ok(b) => b,
            Err(e) => {
                diag::error("config", format!("{}: {e}", path.display()));
                return (Config::default(), None);
            }
        };
        let config = Self::parse(&body, &path.display().to_string());
        (config, Some(path))
    }

    /// Parse a config body. Never fails: a broken file yields defaults and a
    /// diagnostic, because refusing to start over a typo in an optional file is
    /// worse than starting without it.
    pub fn parse(body: &str, source: &str) -> Config {
        let table: toml::Table = match toml::from_str(body) {
            Ok(t) => t,
            Err(e) => {
                diag::error(
                    "config",
                    format!("{source}: {}", e.to_string().lines().next().unwrap_or("")),
                );
                return Config::default();
            }
        };
        for key in table.keys() {
            if !KNOWN.contains(&key.as_str()) {
                diag::warn("config", format!("{source}: ignoring unknown key {key:?}"));
            }
        }
        let mut config: Config = match table.try_into() {
            Ok(c) => c,
            Err(e) => {
                diag::error(
                    "config",
                    format!("{source}: {}", e.to_string().lines().next().unwrap_or("")),
                );
                return Config::default();
            }
        };
        config.clamp(source);
        config
    }

    /// Hold every setting inside a range where harrow still works. A refresh of
    /// zero re-reads the backlog as fast as the disk will answer.
    fn clamp(&mut self, source: &str) {
        let note = |what: &str, from: String, to: String| {
            diag::warn(
                "config",
                format!("{source}: {what} {from} out of range, using {to}"),
            );
        };
        if !(1..=3600).contains(&self.refresh_secs) {
            let was = self.refresh_secs;
            self.refresh_secs = self.refresh_secs.clamp(1, 3600);
            note(
                "refresh_secs",
                was.to_string(),
                self.refresh_secs.to_string(),
            );
        }
        if !(200..=120_000).contains(&self.write_ms) {
            let was = self.write_ms;
            self.write_ms = self.write_ms.clamp(200, 120_000);
            note("write_ms", was.to_string(), self.write_ms.to_string());
        }
    }

    pub fn refresh(&self) -> Duration {
        Duration::from_secs(self.refresh_secs)
    }

    pub fn write_timeout(&self) -> Duration {
        Duration::from_millis(self.write_ms)
    }

    /// What `e` should open, in the order everything else in a terminal looks.
    pub fn editor(&self) -> String {
        if !self.editor.trim().is_empty() {
            return self.editor.clone();
        }
        for var in ["VISUAL", "EDITOR"] {
            if let Ok(value) = std::env::var(var)
                && !value.trim().is_empty()
            {
                return value;
            }
        }
        "vi".to_string()
    }

    /// Render as TOML, for `harrow config`.
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|e| format!("# could not render: {e}\n"))
    }

    /// A fully commented default file, for `harrow config --write`. Kept here
    /// rather than in the docs so the two cannot drift.
    pub fn commented_default() -> String {
        let d = Config::default();
        format!(
            r##"# harrow configuration
#
# Every key is optional. Delete anything you do not want to change — harrow
# reads what is here and uses its own defaults for the rest.
#
# This file is yours. The project's schema — its types, statuses and fields —
# lives in the repository's own cairn.toml, and harrow never writes to it.
#
#   harrow config       show what is actually in effect
#   harrow themes       list every theme harrow can find

# A theme name, or "auto" to use your terminal's own colours.
# Try: auto, mono, gotham, night, paper, ghostty:<name>, or a path to a file.
theme = "{theme}"

# What the list is grouped under. Any field the project has — milestone,
# status, type, priority, area, assignee — or "none" for a flat list.
group_by = "{group_by}"

# Sort order within a group, in cairn's own spelling: "priority,-updated".
# Empty means the order harrow picks: what is ready first, then priority.
sort = "{sort}"

# Open in one of the project's saved views, by name. Empty means everything.
view = "{view}"

# Show everything on startup: finished, dropped, and the milestones work
# belongs to. This is cairn's `--all`, and means what it means there.
show_all = {show_all}

# Which pane to open on: list, board or stats.
pane = "{pane}"

# Watch the filesystem, so a change made in another window shows up at once
# rather than on the next poll. The poll stays either way: it is what notices a
# change on a network mount, where the operating system tells nobody anything.
watch = {watch}

# Seconds between those polls.
refresh_secs = {refresh_secs}

# How long to give `cairn` to carry out a change before giving up on it.
write_ms = {write_ms}

# The cairn to run for writes. A path, if it is not on PATH. harrow reads the
# item files itself and only shells out to change one.
cairn = "{cairn}"

# What `e` opens. Empty falls back to $VISUAL, then $EDITOR, then vi.
editor = "{editor}"

# Keys, bound to action names. See `harrow --help`, or press `?`, for the
# defaults — the help overlay is generated from your bindings, not from ours.
# Actions: down up page-down page-up first last detail-down detail-up
#          toggle-group next-group prev-group view-board group-by read edit
#          note history accept claim release close reopen new status priority
#          milestone advance retreat copy filter back toggle-all refresh
#          reload diagnostics help toggle-mouse quit
[keys]
# "ctrl-r" = "reload"
# "s"      = "status"
"##,
            theme = d.theme,
            group_by = d.group_by,
            sort = d.sort,
            view = d.view,
            show_all = d.show_all,
            pane = d.pane,
            watch = d.watch,
            refresh_secs = d.refresh_secs,
            write_ms = d.write_ms,
            cairn = d.cairn,
            editor = d.editor,
        )
    }

    /// Write the commented default. Refuses to clobber.
    pub fn write_default(path: &Path, force: bool) -> std::io::Result<()> {
        if path.exists() && !force {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "{} already exists; pass --force to replace it",
                    path.display()
                ),
            ));
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, Self::commented_default())
    }

    /// Which settings differ from the defaults. Used by `harrow config` so a
    /// value that came from the file is visible as such.
    pub fn overridden(&self) -> Vec<&'static str> {
        let d = Config::default();
        let mut out = Vec::new();
        if self.theme != d.theme {
            out.push("theme");
        }
        if self.group_by != d.group_by {
            out.push("group_by");
        }
        if self.sort != d.sort {
            out.push("sort");
        }
        if self.view != d.view {
            out.push("view");
        }
        if self.show_all != d.show_all {
            out.push("show_all");
        }
        if self.pane != d.pane {
            out.push("pane");
        }
        if self.refresh_secs != d.refresh_secs {
            out.push("refresh_secs");
        }
        if self.watch != d.watch {
            out.push("watch");
        }
        if self.write_ms != d.write_ms {
            out.push("write_ms");
        }
        if self.cairn != d.cairn {
            out.push("cairn");
        }
        if self.editor != d.editor {
            out.push("editor");
        }
        if !self.keys.is_empty() {
            out.push("keys");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_is_the_defaults() {
        assert_eq!(Config::parse("", "test"), Config::default());
    }

    #[test]
    fn a_partial_file_changes_only_what_it_names() {
        let c = Config::parse("theme = \"gotham\"\n", "test");
        assert_eq!(c.theme, "gotham");
        assert_eq!(c.group_by, Config::default().group_by);
        assert_eq!(c.overridden(), vec!["theme"]);
    }

    #[test]
    fn a_malformed_file_yields_defaults_rather_than_failing() {
        let _guard = diag::test_lock();
        diag::reset();
        let c = Config::parse("theme = \n", "broken.toml");
        assert_eq!(c, Config::default());
        assert!(
            diag::recent(20)
                .iter()
                .any(|e| e.message.contains("broken.toml")),
            "the failure has to be reportable"
        );
    }

    #[test]
    fn an_unknown_key_warns_and_is_ignored() {
        let _guard = diag::test_lock();
        diag::reset();
        let c = Config::parse("theme = \"night\"\nfuture_setting = 3\n", "test");
        assert_eq!(c.theme, "night", "the keys we know still apply");
        assert!(
            diag::recent(20)
                .iter()
                .any(|e| e.message.contains("future_setting")),
            "an unknown key must be reported"
        );
    }

    #[test]
    fn absurd_values_are_clamped_rather_than_obeyed() {
        let c = Config::parse("refresh_secs = 0\nwrite_ms = 1\n", "test");
        assert_eq!(c.refresh_secs, 1);
        assert_eq!(c.write_ms, 200, "a write needs longer than a millisecond");
    }

    #[test]
    fn the_written_default_parses_back_to_the_default() {
        let text = Config::commented_default();
        assert_eq!(
            Config::parse(&text, "commented_default"),
            Config::default(),
            "the file harrow writes must mean what harrow defaults to"
        );
    }

    #[test]
    fn the_rendered_config_round_trips() {
        let mut c = Config {
            theme: "gotham".into(),
            group_by: "status".into(),
            ..Default::default()
        };
        c.keys.insert("ctrl-r".into(), "reload".into());
        assert_eq!(
            Config::parse(&c.to_toml(), "roundtrip"),
            c,
            "`harrow config` output must load again"
        );
    }

    #[test]
    fn the_editor_falls_back_the_way_a_terminal_tool_should() {
        let explicit = Config {
            editor: "hx".into(),
            ..Default::default()
        };
        assert_eq!(explicit.editor(), "hx");
        // Whatever is in the environment here, something has to come back.
        assert!(!Config::default().editor().is_empty());
    }

    #[test]
    fn write_default_refuses_to_clobber() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("config.toml");
        Config::write_default(&path, false).expect("first write");
        let err = Config::write_default(&path, false).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        Config::write_default(&path, true).expect("--force replaces it");
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let (c, path) = Config::load(Some(Path::new("/nonexistent/harrow.toml")));
        assert_eq!(c, Config::default());
        assert_eq!(path, None);
    }
}
