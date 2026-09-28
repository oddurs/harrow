//! The characters harrow draws with.
//!
//! Two tables of the same roles, the way a theme is a table of colours: one
//! of the Unicode every font has, and one of Nerd Font icons. Drawing code
//! asks for a role and never names a character, so the choice is made once
//! and a screen cannot come out half in one set and half in the other.
//!
//! The Nerd table is the better-looking one, and it cannot be the default.
//! Its icons live in the Private Use Area, no terminal will say whether its
//! font has them, and one that has not draws a box — the failure
//! `tests/glyphs.rs` was written after. So it is chosen on evidence or on
//! request, and everything it adds is an empty string in the Unicode table:
//! the ordinary screen does not move by a cell.
//!
//! One family, so the icons read as one set rather than a collection: GitHub's
//! octicons for the things a backlog shares with an issue tracker, codicon
//! chevrons for the folds, Fira Code's progress glyphs for the bars and the
//! spinner. Each is written as an escape with its Nerd Fonts name beside it,
//! because the character itself is invisible in any editor without the font.

use crate::app::Pane;
use crate::item::Item;
use crate::schema::Category;

/// Every role harrow draws a glyph for.
#[derive(Debug, PartialEq, Eq)]
pub struct Glyphs {
    /// What the config calls it.
    pub name: &'static str,

    // What a thing is. One each, so the state column still reads with no
    // colour at all.
    pub open: &'static str,
    pub active: &'static str,
    pub done: &'static str,
    pub dropped: &'static str,
    pub blocked: &'static str,
    /// Work under way, a frame at a time. The first frame is `active`, which
    /// is the one a recorded screen is taken at.
    pub turning: &'static [&'static str],
    /// A read in progress.
    pub spinner: &'static [&'static str],

    /// A progress bar, a cell at a time.
    bar: Bar,

    // Folds and lists.
    pub folded: &'static str,
    pub unfolded: &'static str,
    /// A control that opens a list, and the same control while it is open.
    pub opens: &'static str,
    pub opened: &'static str,
    /// The row a list's cursor is on.
    pub pointer: &'static str,

    // Checkboxes: a criterion, the same unticked, and a ticked filter value.
    // Unicode has a tick and a ticked box, and the filter uses the box so a
    // value reads as a control rather than a result.
    pub ticked: &'static str,
    pub unticked: &'static str,
    pub chosen: &'static str,

    // Messages.
    pub warning: &'static str,
    pub good: &'static str,
    pub bad: &'static str,
    pub info: &'static str,

    /// Yours and somebody else's, which is the first-order question on a
    /// backlog shared with programs. Written as they are drawn: in Unicode a
    /// mark glued to the name, in the Nerd set an icon and the cell it spills
    /// into.
    pub you: &'static str,
    pub other: &'static str,
    /// Somebody is waiting on an answer. Drawn at the end of a row, so it
    /// carries its own spill cell rather than borrowing the border's.
    pub asked: &'static str,

    /// Whether a state glyph may also label a heading — a status column, a
    /// group of one status. A set with no icons keeps them to the rows,
    /// where they have always been.
    ornate: bool,
    /// What the Unicode set has no characters for.
    pub icons: Icons,
}

/// Icons that label a thing rather than being it. All empty in a set that has
/// none, so every place one is drawn costs nothing there.
#[derive(Debug, PartialEq, Eq)]
pub struct Icons {
    // The lenses, on their tabs and on the panes they open.
    pub needs: &'static str,
    pub list: &'static str,
    pub board: &'static str,
    pub stats: &'static str,
    pub log: &'static str,

    // The toolbar.
    pub filter: &'static str,
    pub sort: &'static str,
    pub group: &'static str,
    pub everything: &'static str,

    // Things an item has.
    pub milestone: &'static str,
    pub criteria: &'static str,
    pub branch: &'static str,
    pub said: &'static str,
    /// What a finished item concluded: its Result.
    pub concluded: &'static str,
    /// Finished work this item rests on.
    pub builds_on: &'static str,
    pub body: &'static str,
    pub fields: &'static str,
    /// The field a project ranks by, whatever it calls it.
    pub rank: &'static str,
    /// Something that holds other work: a rollup, a container, the types.
    pub holds: &'static str,

    // Figures.
    pub standing: &'static str,
    pub ready: &'static str,
    pub waiting: &'static str,

    // What needs you.
    pub attention: &'static str,
    pub cold: &'static str,
    pub unowned: &'static str,
}

/// How a bar is drawn: one glyph for a filled cell and one for an empty one,
/// and — where the font joins them into one rounded bar — the two ends.
#[derive(Debug, PartialEq, Eq)]
struct Bar {
    full: &'static str,
    empty: &'static str,
    ends: Option<Ends>,
}

#[derive(Debug, PartialEq, Eq)]
struct Ends {
    full_left: &'static str,
    empty_left: &'static str,
    full_right: &'static str,
    empty_right: &'static str,
}

const NO_ICONS: Icons = Icons {
    needs: "",
    list: "",
    board: "",
    stats: "",
    log: "",
    filter: "",
    sort: "",
    group: "",
    everything: "",
    milestone: "",
    criteria: "",
    branch: "",
    said: "",
    concluded: "",
    builds_on: "",
    body: "",
    fields: "",
    rank: "",
    holds: "",
    standing: "",
    ready: "",
    waiting: "",
    attention: "",
    cold: "",
    unowned: "",
};

/// What every font has, and what harrow has always drawn.
pub const UNICODE: Glyphs = Glyphs {
    name: "unicode",
    open: "○",
    active: "◐",
    done: "✓",
    dropped: "×",
    blocked: "⊘",
    turning: &["◐", "◓", "◑", "◒"],
    spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"],
    bar: Bar {
        full: "▰",
        empty: "▱",
        ends: None,
    },
    folded: "▸",
    unfolded: "▾",
    opens: "▾",
    opened: "▴",
    pointer: "▸",
    ticked: "✓",
    unticked: "☐",
    chosen: "☑",
    warning: "⚠",
    good: "●",
    bad: "●",
    info: "●",
    you: "@",
    other: "·",
    asked: "?",
    ornate: false,
    icons: NO_ICONS,
};

/// Nerd Fonts 3. Octicons and codicons have kept their codepoints since
/// version 2; the Material pie and boxes are the version-3 ones.
pub const NERD: Glyphs = Glyphs {
    name: "nerd",
    open: "\u{f41b}",    // oct-issue_opened
    active: "\u{f0aa1}", // md-circle_slice_4
    done: "\u{f41d}",    // oct-issue_closed
    dropped: "\u{f517}", // oct-skip — GitHub's "not planned"
    blocked: "\u{f479}", // oct-blocked
    // A pie filling, from half: it reads as work being done rather than as a
    // wheel going round, and the first frame is `active`.
    turning: &[
        "\u{f0aa1}", // md-circle_slice_4
        "\u{f0aa2}", // md-circle_slice_5
        "\u{f0aa3}", // md-circle_slice_6
        "\u{f0aa4}", // md-circle_slice_7
        "\u{f0aa5}", // md-circle_slice_8
        "\u{f0a9e}", // md-circle_slice_1
        "\u{f0a9f}", // md-circle_slice_2
        "\u{f0aa0}", // md-circle_slice_3
    ],
    spinner: &[
        "\u{ee06}", // extra-progress_spinner_1
        "\u{ee07}", // extra-progress_spinner_2
        "\u{ee08}", // extra-progress_spinner_3
        "\u{ee09}", // extra-progress_spinner_4
        "\u{ee0a}", // extra-progress_spinner_5
        "\u{ee0b}", // extra-progress_spinner_6
    ],
    bar: Bar {
        full: "\u{ee04}",  // extra-progress_full_mid
        empty: "\u{ee01}", // extra-progress_empty_mid
        ends: Some(Ends {
            full_left: "\u{ee03}",   // extra-progress_full_left
            empty_left: "\u{ee00}",  // extra-progress_empty_left
            full_right: "\u{ee05}",  // extra-progress_full_right
            empty_right: "\u{ee02}", // extra-progress_empty_right
        }),
    },
    folded: "\u{eab6}",    // cod-chevron_right
    unfolded: "\u{eab4}",  // cod-chevron_down
    opens: "\u{eab4}",     // cod-chevron_down
    opened: "\u{eab7}",    // cod-chevron_up
    pointer: "\u{eab6}",   // cod-chevron_right
    ticked: "\u{f0135}",   // md-checkbox_marked_outline
    unticked: "\u{f0131}", // md-checkbox_blank_outline
    chosen: "\u{f0135}",   // md-checkbox_marked_outline
    warning: "\u{f421}",   // oct-alert
    good: "\u{f49e}",      // oct-check_circle
    bad: "\u{f52f}",       // oct-x_circle
    info: "\u{f449}",      // oct-info
    you: "\u{f4ff} ",      // oct-person_fill
    other: "\u{f415} ",    // oct-person
    asked: "\u{f407} ",    // oct-git_pull_request — a change waiting on a yes
    ornate: true,
    icons: Icons {
        needs: "\u{f48d}",      // oct-inbox
        list: "\u{f451}",       // oct-list_unordered
        board: "\u{f502}",      // oct-project
        stats: "\u{f437}",      // oct-graph
        log: "\u{f464}",        // oct-history
        filter: "\u{f4d7}",     // oct-filter
        sort: "\u{f51a}",       // oct-sort_desc
        group: "\u{f50b}",      // oct-rows
        everything: "\u{f441}", // oct-eye
        milestone: "\u{f45d}",  // oct-milestone
        criteria: "\u{f45e}",   // oct-checklist
        branch: "\u{f418}",     // oct-git_branch
        said: "\u{f41f}",       // oct-comment
        concluded: "\u{f400}",  // oct-light_bulb — what it came to
        builds_on: "\u{f419}",  // oct-git_merge — finished work flowing in
        body: "\u{f4f6}",       // oct-note
        fields: "\u{f412}",     // oct-tag
        rank: "\u{f023b}",      // md-flag
        holds: "\u{f51e}",      // oct-stack
        standing: "\u{f469}",   // oct-pulse
        ready: "\u{f427}",      // oct-rocket
        waiting: "\u{f4e3}",    // oct-hourglass
        attention: "\u{f49a}",  // oct-bell
        cold: "\u{f0717}",      // md-snowflake — a claim gone cold
        unowned: "\u{f4fe}",    // oct-person_add — nobody answerable yet
    },
};

impl Glyphs {
    /// What a piece of work is, blocked before anything else, because a
    /// blocked item is not work anybody can pick up.
    pub fn state(&self, item: &Item) -> &'static str {
        if item.blocked && !item.category.is_closed() {
            return self.blocked;
        }
        self.category(item.category)
    }

    /// The same, turning where the work is under way.
    ///
    /// Work being done is the only thing on a backlog that is *happening*, and
    /// it should be the only thing moving. A frame per two ticks of the clock
    /// the spinner already runs on, so it animates in every terminal and
    /// still snapshots — a recorded screen is taken at tick 0, where this is
    /// `active`.
    ///
    /// Not the terminal's blink attribute: half the terminals harrow runs in
    /// ignore it, and the ones that honour it blink the whole cell.
    ///
    /// Work under way in another worktree turns too, because it is just as
    /// much under way — it is only the record here that has not heard yet.
    pub fn turning(&self, item: &Item, tick: usize) -> &'static str {
        if (item.category == Category::Active && !item.blocked) || item.active_elsewhere().is_some()
        {
            return self.turning[(tick / 2) % self.turning.len()];
        }
        self.state(item)
    }

    pub fn category(&self, category: Category) -> &'static str {
        match category {
            Category::Open => self.open,
            Category::Active => self.active,
            Category::Done => self.done,
            Category::Dropped => self.dropped,
        }
    }

    /// A glyph put to work as a label, where the set is one that labels
    /// things; nothing where it is not.
    pub fn label(&self, glyph: &'static str) -> &'static str {
        if self.ornate { glyph } else { "" }
    }

    pub fn spinner(&self, tick: usize) -> &'static str {
        self.spinner[tick % self.spinner.len()]
    }

    pub fn lens(&self, pane: Pane) -> &'static str {
        match pane {
            Pane::Needs => self.icons.needs,
            Pane::List => self.icons.list,
            Pane::Board => self.icons.board,
            Pane::Stats => self.icons.stats,
            Pane::Log => self.icons.log,
        }
    }

    /// How far along, in `width` cells. Anything above nothing shows
    /// something: rounding a real 4% down to an empty bar says "not started",
    /// which is a different claim.
    pub fn bar(&self, percent: u32, width: usize) -> String {
        let filled = (percent as usize * width).div_ceil(100).min(width);
        let mut out = String::with_capacity(width * 3);
        for i in 0..width {
            let full = i < filled;
            let cell = match &self.bar.ends {
                Some(ends) if i == 0 && width > 1 => {
                    if full {
                        ends.full_left
                    } else {
                        ends.empty_left
                    }
                }
                Some(ends) if i + 1 == width && width > 1 => {
                    if full {
                        ends.full_right
                    } else {
                        ends.empty_right
                    }
                }
                _ if full => self.bar.full,
                _ => self.bar.empty,
            };
            out.push_str(cell);
        }
        out
    }

    /// Every character this set can draw, for the test that holds a screen
    /// to what has been declared.
    pub fn every(&self) -> Vec<&'static str> {
        let i = &self.icons;
        let mut all = vec![
            self.open,
            self.active,
            self.done,
            self.dropped,
            self.blocked,
            self.bar.full,
            self.bar.empty,
            self.folded,
            self.unfolded,
            self.opens,
            self.opened,
            self.pointer,
            self.ticked,
            self.unticked,
            self.chosen,
            self.warning,
            self.good,
            self.bad,
            self.info,
            self.you,
            self.other,
            self.asked,
            i.needs,
            i.list,
            i.board,
            i.stats,
            i.log,
            i.filter,
            i.sort,
            i.group,
            i.everything,
            i.milestone,
            i.criteria,
            i.branch,
            i.said,
            i.concluded,
            i.builds_on,
            i.body,
            i.fields,
            i.rank,
            i.holds,
            i.standing,
            i.ready,
            i.waiting,
            i.attention,
            i.cold,
            i.unowned,
        ];
        all.extend(self.turning);
        all.extend(self.spinner);
        if let Some(e) = &self.bar.ends {
            all.extend([e.full_left, e.empty_left, e.full_right, e.empty_right]);
        }
        all.retain(|g| !g.is_empty());
        all.iter().map(|g| g.trim_end()).collect()
    }

    /// Whether a glyph fills its cell exactly, rather than being an icon that
    /// may be drawn wider. The bar and the spinner are Fira Code's, cut to
    /// the cell so that a bar joins up, and they need no room after them.
    pub fn fits_its_cell(c: char) -> bool {
        ('\u{ee00}'..='\u{ee0b}').contains(&c)
    }
}

/// An icon and the blank cell it may spill into, or nothing for a set that
/// has no icon here. A glyph that already carries its spill cell gets one,
/// not two.
pub fn adorn(icon: &str) -> String {
    let icon = icon.trim_end();
    if icon.is_empty() {
        String::new()
    } else {
        format!("{icon} ")
    }
}

/// What the config and `--glyphs` accept.
pub const CHOICES: &[&str] = &["auto", "nerd", "unicode"];

/// Which set to draw with, and why — the why is what `harrow config` and
/// `--doctor` print, because `auto` is a guess and a guess should say what it
/// was based on.
///
/// `var` reads the environment. It is a parameter so the guess can be tested
/// without setting variables on a process that runs tests in parallel.
pub fn resolve(
    spec: &str,
    var: impl Fn(&str) -> Option<String>,
) -> Result<(&'static Glyphs, String), String> {
    match spec.trim().to_lowercase().as_str() {
        "nerd" => Ok((&NERD, "asked for".to_string())),
        "unicode" => Ok((&UNICODE, "asked for".to_string())),
        "auto" | "" => Ok(detect(var)),
        other => Err(format!(
            "no glyph set called {other:?}; expected one of {}",
            CHOICES.join(", ")
        )),
    }
}

/// `auto`: the Nerd set on evidence, and Unicode on anything less.
///
/// The evidence is a terminal that ships the symbols with it, which is the
/// one case where the answer does not depend on a font somebody may or may
/// not have installed. Ghostty and WezTerm have always bundled them; kitty
/// has since 0.36. Each is recognised by `TERM_PROGRAM`, then `TERM` — which
/// survives `ssh` — then a variable of its own, which survives `tmux`, where
/// the other two are tmux's.
///
/// A font you installed yourself is invisible from here. That is what
/// `glyphs = "nerd"` is for, and the reason given says so.
fn detect(var: impl Fn(&str) -> Option<String>) -> (&'static Glyphs, String) {
    let get = |name: &str| var(name).filter(|v| !v.trim().is_empty());

    // The first of these that is set is the one that decides, which is the
    // order the C library reads them in.
    if let Some(locale) = ["LC_ALL", "LC_CTYPE", "LANG"].into_iter().find_map(get) {
        let lower = locale.to_lowercase();
        if !(lower.contains("utf-8") || lower.contains("utf8")) {
            return (&UNICODE, format!("the locale {locale} is not UTF-8"));
        }
    }

    let term = get("TERM").unwrap_or_default();
    let program = get("TERM_PROGRAM").unwrap_or_default().to_lowercase();
    let bundled = [
        (
            "Ghostty",
            program == "ghostty"
                || term == "xterm-ghostty"
                || get("GHOSTTY_RESOURCES_DIR").is_some(),
        ),
        (
            "WezTerm",
            program == "wezterm" || get("WEZTERM_EXECUTABLE").is_some(),
        ),
        (
            "kitty",
            term == "xterm-kitty" || get("KITTY_WINDOW_ID").is_some(),
        ),
    ];
    match bundled.into_iter().find(|(_, here)| *here) {
        Some((name, _)) => (&NERD, format!("{name} ships the Nerd Font symbols")),
        None => (
            &UNICODE,
            "no sign of Nerd Font symbols; set glyphs = \"nerd\" if your font has them".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    fn chosen(pairs: &[(&str, &str)]) -> &'static str {
        resolve("auto", env(pairs))
            .expect("auto always resolves")
            .0
            .name
    }

    #[test]
    fn a_terminal_that_ships_the_symbols_gets_them() {
        assert_eq!(chosen(&[("TERM_PROGRAM", "ghostty")]), "nerd");
        assert_eq!(chosen(&[("TERM", "xterm-ghostty")]), "nerd");
        assert_eq!(chosen(&[("TERM_PROGRAM", "WezTerm")]), "nerd");
        assert_eq!(chosen(&[("TERM", "xterm-kitty")]), "nerd");
    }

    #[test]
    fn inside_tmux_the_terminal_is_still_recognised_by_its_own_variable() {
        let tmux = [
            ("TERM", "tmux-256color"),
            ("TERM_PROGRAM", "tmux"),
            (
                "GHOSTTY_RESOURCES_DIR",
                "/Applications/Ghostty.app/Contents/Resources/ghostty",
            ),
        ];
        assert_eq!(chosen(&tmux), "nerd");
        assert_eq!(
            chosen(&[("TERM_PROGRAM", "tmux"), ("KITTY_WINDOW_ID", "1")]),
            "nerd"
        );
    }

    #[test]
    fn an_unknown_terminal_keeps_what_every_font_has() {
        assert_eq!(chosen(&[]), "unicode");
        assert_eq!(chosen(&[("TERM_PROGRAM", "Apple_Terminal")]), "unicode");
        assert_eq!(chosen(&[("TERM", "tmux-256color")]), "unicode");
    }

    #[test]
    fn a_locale_that_is_not_utf8_wins_over_the_terminal() {
        let why = resolve("auto", env(&[("TERM_PROGRAM", "ghostty"), ("LC_ALL", "C")]))
            .expect("resolves");
        assert_eq!(why.0.name, "unicode");
        assert!(why.1.contains("not UTF-8"), "{}", why.1);
        // LC_ALL decides before LANG does, whichever way it goes.
        assert_eq!(
            chosen(&[
                ("TERM", "xterm-kitty"),
                ("LC_ALL", "en_US.UTF-8"),
                ("LANG", "C")
            ]),
            "nerd"
        );
    }

    #[test]
    fn asking_for_a_set_is_not_second_guessed() {
        let (set, _) = resolve("nerd", env(&[("LC_ALL", "C")])).expect("resolves");
        assert_eq!(set.name, "nerd");
        let (set, _) = resolve("Unicode", env(&[("TERM_PROGRAM", "ghostty")])).expect("resolves");
        assert_eq!(set.name, "unicode");
    }

    #[test]
    fn a_set_nobody_has_heard_of_is_refused_by_name() {
        let err = resolve("emoji", env(&[])).unwrap_err();
        assert!(err.contains("emoji") && err.contains("nerd"), "{err}");
    }

    #[test]
    fn every_state_has_its_own_glyph_in_both_sets() {
        for set in [&UNICODE, &NERD] {
            let states = [set.open, set.active, set.done, set.dropped, set.blocked];
            for (n, a) in states.iter().enumerate() {
                for b in &states[n + 1..] {
                    assert_ne!(a, b, "{} draws two states the same", set.name);
                }
            }
            assert_eq!(
                set.turning[0], set.active,
                "{} turns from somewhere else",
                set.name
            );
            assert_ne!(set.you, set.other, "{} cannot tell you apart", set.name);
        }
    }

    #[test]
    fn the_unicode_set_adds_no_icons() {
        assert_eq!(UNICODE.icons, NO_ICONS);
        assert_eq!(adorn(UNICODE.icons.milestone), "");
        assert_eq!(
            adorn(NERD.icons.milestone),
            format!("{} ", NERD.icons.milestone)
        );
    }

    /// Nerd Fonts' own ranges: the BMP Private Use Area, where octicons,
    /// codicons and the progress glyphs are, and plane 15, where the version-3
    /// Material icons moved. A codepoint anywhere else is not a Nerd Font
    /// glyph, and is one a font without them may still draw as something else.
    #[test]
    fn every_nerd_glyph_is_a_nerd_font_codepoint() {
        for glyph in NERD.every() {
            let mut chars = glyph.chars();
            let c = chars.next().expect("not empty");
            assert!(
                chars.next().is_none(),
                "{glyph:?} is more than one character"
            );
            assert!(
                ('\u{e000}'..='\u{f8ff}').contains(&c) || ('\u{f0000}'..='\u{fffff}').contains(&c),
                "U+{:04X} is outside the Nerd Font ranges",
                c as u32
            );
        }
    }

    #[test]
    fn a_bar_joins_up_where_the_font_can_join_it() {
        assert_eq!(UNICODE.bar(0, 4), "▱▱▱▱");
        assert_eq!(UNICODE.bar(100, 4), "▰▰▰▰");
        assert_eq!(UNICODE.bar(50, 4), "▰▰▱▱");
        assert!(UNICODE.bar(4, 8).starts_with('▰'));

        assert_eq!(NERD.bar(50, 4), "\u{ee03}\u{ee04}\u{ee01}\u{ee02}");
        assert_eq!(NERD.bar(100, 3), "\u{ee03}\u{ee04}\u{ee05}");
        assert_eq!(NERD.bar(0, 3), "\u{ee00}\u{ee01}\u{ee02}");
        // One cell has no ends to draw.
        assert_eq!(NERD.bar(100, 1), "\u{ee04}");
    }
}
