//! What harrow is allowed to draw.
//!
//! A terminal draws a box for a character its font has not got, and nothing
//! on screen says which character it was. The grouping segment shipped with
//! `⊞` — U+229E SQUARED PLUS — and the first thing on the toolbar was a
//! character that was not anything.
//!
//! Counting the glyphs in every recorded screen said how that happened: every
//! other glyph harrow draws had been on screen for months, and the only two
//! that were new were the only two that broke. So the rule is that a glyph is
//! declared here before it reaches a screen, which means somebody is asked
//! whether the terminals this runs in have got it.
//!
//! 0048 is the general version — a terminal without the glyphs still getting
//! a usable screen. This is the cheap half that stops it getting worse.
//!
//! The Nerd Font set is held to the same rule by a different list: its own
//! table in `src/glyphs.rs`, which is where a Nerd glyph is declared. And to
//! one more, because its icons are drawn wider than a cell in most fonts
//! that carry them: every one has a blank cell after it to spill into.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Pane, ToastKind};
use harrow::glyphs::{Glyphs, NERD, UNICODE};
use harrow::keys::Command;
use harrow::{testkit, ui};

/// Everything harrow may put on a screen, by what it is for.
const ALLOWED: &[(&str, &str)] = &[
    ("box drawing", "─│╭╮╰╯"),
    ("what a thing is", "○◐✓×⊘☐"),
    ("how far along", "▰▱▇▁"),
    ("a list opens, or is open", "▾▴▸"),
    ("direction", "↑↓←→↵"),
    ("punctuation the width is worth", "·—…"),
    ("marked, quoted, changed", "▌▏•●"),
    ("something is wrong", "⚠"),
    ("it is working", "⠋⠙⠹⠸⠼⠴⠦⠧"),
];

fn allowed(c: char) -> bool {
    c.is_ascii() || ALLOWED.iter().any(|(_, set)| set.contains(c))
}

/// The same, on a screen drawn with the Nerd set: what every screen may
/// draw, and what that set declares.
fn allowed_in_nerd(c: char) -> bool {
    allowed(c) || NERD.every().iter().any(|g| g.contains(c))
}

/// A recorded screen of the Nerd set says so in its name.
fn recorded_in_nerd(path: &std::path::Path) -> bool {
    path.file_stem()
        .is_some_and(|s| s.to_string_lossy().ends_with("-nerd"))
}

/// A screen, and what it took to get there.
type State = (&'static str, Box<dyn Fn() -> App>);

/// Screens no recorded snapshot covers, because they need a key pressed
/// first — which is exactly where a new glyph is most likely to hide.
fn states() -> Vec<State> {
    states_in(&UNICODE)
}

fn states_in(glyphs: &'static Glyphs) -> Vec<State> {
    let build = move |f: fn(&mut App)| -> Box<dyn Fn() -> App> {
        Box::new(move || {
            let mut app = testkit::app();
            app.glyphs = glyphs;
            let _ = ui::render_frame(&mut app, 110, 30, 0);
            f(&mut app);
            app
        })
    };
    vec![
        ("at rest", build(|_| {})),
        (
            "with a list open",
            build(|app| {
                app.handle_key(KeyCode::Char('v'), KeyModifiers::NONE);
            }),
        ),
        (
            "with an order open",
            build(|app| {
                app.handle_key(KeyCode::Char('S'), KeyModifiers::NONE);
            }),
        ),
        (
            "with the palette open",
            build(|app| {
                app.handle_key(KeyCode::Char(':'), KeyModifiers::NONE);
            }),
        ),
        (
            "with a group collapsed",
            build(|app| {
                app.collapsed.insert("v0.1".into());
                app.rebuild();
            }),
        ),
        (
            // `space` on the first row folds or collapses now, so marking has
            // to be asked for on a row that is an item.
            "with something marked",
            build(|app| {
                app.select_id(3.into());
                app.run(Command::ToggleGroup);
            }),
        ),
        (
            "with the finished work unfolded",
            build(|app| {
                app.unfolded.insert("v0.1".into());
                app.rebuild();
            }),
        ),
        (
            "showing everything",
            build(|app| {
                app.show_all = true;
                app.rebuild();
            }),
        ),
        (
            // Nothing in the fixture is dropped, and dropped is the only
            // thing `×` is for.
            "with something dropped",
            build(|app| {
                app.items[1].status = "dropped".into();
                app.items[1].category = harrow::schema::Category::Dropped;
                app.show_all = true;
                app.rebuild();
            }),
        ),
        (
            // The grouping off, which is the one arrangement with a word of
            // its own rather than a field name.
            "arranged flat",
            build(|app| {
                app.set_grouping("none");
            }),
        ),
        (
            "on the log",
            build(|app| {
                app.pane = Pane::Log;
                app.rebuild();
            }),
        ),
        (
            "narrowed to nothing",
            build(|app| {
                app.filter = "priority=p9".into();
                app.ingest(testkit::report());
            }),
        ),
        (
            "on the board",
            build(|app| {
                app.pane = Pane::Board;
                app.rebuild();
            }),
        ),
        (
            "on the statistics",
            build(|app| {
                app.pane = Pane::Stats;
                app.rebuild();
            }),
        ),
        (
            "on what needs you",
            build(|app| {
                app.pane = Pane::Needs;
                app.rebuild();
            }),
        ),
        (
            "in the help",
            build(|app| {
                app.run(Command::Help);
            }),
        ),
        (
            "reading an item",
            build(|app| {
                app.run(Command::Read);
            }),
        ),
        (
            "asked to split an item",
            build(|app| {
                app.show_split(
                    3.into(),
                    Ok("  0010 Read the file\n  0011 Parse it  after 0010\n".into()),
                );
            }),
        ),
        (
            "reading an item's prompt",
            build(|app| {
                app.show_prompt(
                    3.into(),
                    Ok("# 0003 Draw the list\n\n## Done when\n\n1. Rows scroll\n- a list\n".into()),
                );
            }),
        ),
        (
            "on what a finished item concluded",
            build(|app| {
                app.ingest(testkit::concluded_report());
                app.show_all = true;
                app.rebuild();
                app.select_id(6.into());
            }),
        ),
        (
            "reading what a finished item concluded",
            build(|app| {
                app.ingest(testkit::concluded_report());
                app.show_all = true;
                app.rebuild();
                app.select_id(6.into());
                app.run(Command::Read);
            }),
        ),
        (
            "with the filter panel open",
            build(|app| {
                app.run(Command::Facets);
            }),
        ),
        (
            "grouped by status",
            build(|app| {
                app.set_grouping("status");
            }),
        ),
        (
            "with something said in the footer",
            build(|app| {
                app.toast("closed 0005".to_string(), ToastKind::Good);
            }),
        ),
        (
            "while it reads",
            build(|app| {
                app.loading = true;
            }),
        ),
        (
            "with an item filed on another branch",
            build(|app| {
                let mut report = testkit::report();
                let mut seven = testkit::item(7, "Write the changelog", "doing");
                seven.category = report.schema.category("doing");
                seven.filed_on = Some("feat/changelog".into());
                report.filed.push(seven);
                app.ingest(report);
                app.select_id(7.into());
            }),
        ),
    ]
}

#[test]
fn nothing_is_drawn_that_has_not_been_declared() {
    for (which, make) in states() {
        let mut app = make();
        let screen = ui::render_to_string(&mut app, 110, 30, 0);
        for c in screen.chars() {
            assert!(
                allowed(c),
                "{which}: U+{:04X} `{c}` is drawn and not declared in tests/glyphs.rs — \
                 is it in the fonts harrow runs in?",
                c as u32
            );
        }
    }
}

/// The recorded screens too, which is most of what harrow draws.
#[test]
fn nothing_recorded_uses_a_glyph_outside_the_set() {
    let mut checked = 0;
    for entry in std::fs::read_dir("tests/snapshots").expect("the snapshots are there") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_none_or(|e| e != "txt") {
            continue;
        }
        let body = std::fs::read_to_string(&path).expect("a snapshot reads");
        let nerd = recorded_in_nerd(&path);
        for c in body.chars() {
            assert!(
                if nerd { allowed_in_nerd(c) } else { allowed(c) },
                "{}: U+{:04X} `{c}` is recorded and not declared",
                path.display(),
                c as u32
            );
        }
        checked += 1;
    }
    assert!(checked > 20, "only {checked} snapshots were read");
}

/// The list is a list of what is *used*. One that grows and never shrinks
/// stops being a decision and becomes a place to put things.
#[test]
fn every_declared_glyph_is_actually_drawn() {
    let mut drawn = String::new();
    for (_, make) in states() {
        let mut app = make();
        drawn.push_str(&ui::render_to_string(&mut app, 110, 30, 0));
    }
    for entry in std::fs::read_dir("tests/snapshots").expect("the snapshots are there") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_some_and(|e| e == "txt") && !recorded_in_nerd(&path) {
            drawn.push_str(&std::fs::read_to_string(&path).expect("a snapshot reads"));
        }
    }
    // The spinner and the warning need a failing load or a slow one, which no
    // still frame has; everything else has to earn its place.
    const ONLY_WHEN_IT_GOES_WRONG: &str = "⚠⠋⠙⠹⠸⠼⠴⠦⠧";
    for (what, set) in ALLOWED {
        for c in set.chars() {
            assert!(
                drawn.contains(c) || ONLY_WHEN_IT_GOES_WRONG.contains(c),
                "U+{:04X} `{c}` is declared under {what:?} and nothing draws it",
                c as u32
            );
        }
    }
}

#[test]
fn a_nerd_screen_draws_only_what_the_nerd_set_declares() {
    for (which, make) in states_in(&NERD) {
        let mut app = make();
        let screen = ui::render_to_string(&mut app, 110, 30, 0);
        for c in screen.chars() {
            assert!(
                allowed_in_nerd(c),
                "{which}: U+{:04X} `{c}` is drawn and is in neither set",
                c as u32
            );
        }
    }
}

/// The other half: choosing Unicode keeps every icon off the screen, which
/// is what makes it the safe answer for a terminal nobody recognises.
#[test]
fn a_unicode_screen_draws_nothing_from_the_private_use_area() {
    for (which, make) in states() {
        let mut app = make();
        let screen = ui::render_to_string(&mut app, 110, 30, 0);
        assert!(
            !screen
                .chars()
                .any(|c| NERD.every().iter().any(|g| g.contains(c))),
            "{which}: a Nerd glyph on a Unicode screen"
        );
    }
}

/// A Nerd Font icon is drawn as wide as the font likes, which in every
/// variant but the `Mono` one is more than a cell — so the cell after it has
/// to be blank, or the icon is drawn over whatever is there. The bar and the
/// spinner are exempt: they are cut to the cell so that a bar joins up.
#[test]
fn every_icon_has_a_cell_to_spill_into() {
    for (which, make) in states_in(&NERD) {
        let mut app = make();
        for tick in [0, 2, 4, 6] {
            let buf = ui::render_frame(&mut app, 110, 30, tick);
            for y in 0..buf.area.height {
                for x in 0..buf.area.width {
                    let symbol = buf[(x, y)].symbol();
                    let Some(c) = symbol.chars().next() else {
                        continue;
                    };
                    if !NERD.every().iter().any(|g| g.contains(c)) || Glyphs::fits_its_cell(c) {
                        continue;
                    }
                    let after = (x + 1 < buf.area.width).then(|| buf[(x + 1, y)].symbol());
                    assert_eq!(
                        after,
                        Some(" "),
                        "{which}, tick {tick}: U+{:04X} at {x},{y} has no room after it",
                        c as u32
                    );
                }
            }
        }
    }
}
