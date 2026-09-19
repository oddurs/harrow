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

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Pane};
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

/// A screen, and what it took to get there.
type State = (&'static str, Box<dyn Fn() -> App>);

/// Screens no recorded snapshot covers, because they need a key pressed
/// first — which is exactly where a new glyph is most likely to hide.
fn states() -> Vec<State> {
    let build = |f: fn(&mut App)| -> Box<dyn Fn() -> App> {
        Box::new(move || {
            let mut app = testkit::app();
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
                app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE);
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
            "with the filter panel open",
            build(|app| {
                app.run(Command::Facets);
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
        for c in body.chars() {
            assert!(
                allowed(c),
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
        if path.extension().is_some_and(|e| e == "txt") {
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
