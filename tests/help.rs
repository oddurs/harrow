//! The help overlay is the map of the program, and it is held to two things.
//!
//! It is arranged by what a reader has come to do, because forty-two rows in
//! one run is the very problem harrow exists to solve for a backlog.
//!
//! And it never lies about where it ends. It was taller than a short terminal
//! and simply stopped at the edge; the rows past it did not exist as far as the
//! reader could tell. It scrolls now, and says so on its edge.

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::App;
use harrow::keys::{Command, Keymap};
use harrow::{testkit, ui};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(code, KeyModifiers::NONE);
}

/// The hint on the overlay's bottom edge, spelled from the bindings the way
/// the overlay spells it. Not the word `scroll` on its own: the overlay has
/// rows about scrolling, so that word is on screen whether or not the hint is.
fn hint() -> String {
    let keys = Keymap::default()
        .scroll_hint(Command::Up, Command::Down)
        .expect("up and down are bound");
    format!(" {keys} scroll ")
}

fn open() -> App {
    let mut app = testkit::app();
    app.run(Command::Help);
    app
}

#[test]
fn every_section_is_named_once_and_holds_something() {
    let sections = Keymap::default().help_sections();
    let mut seen = HashSet::new();
    for section in &sections {
        assert!(seen.insert(section.title), "`{}` twice", section.title);
        assert!(
            !section.rows.is_empty(),
            "`{}` is a heading over nothing",
            section.title
        );
    }
}

/// Headings are for finding, so the question is whether a reader who knows
/// what they want to do lands in the right place. A handful of the ones people
/// actually go looking for, by the section a reader would try first.
#[test]
fn a_command_is_filed_where_a_reader_would_look_for_it() {
    let filed = |command: Command| {
        Command::HELP_SECTIONS
            .iter()
            .find(|(_, commands)| commands.contains(&command))
            .map(|(title, _)| *title)
    };
    assert_eq!(filed(Command::Down), Some("Move"));
    assert_eq!(filed(Command::ViewBoard), Some("Look"));
    assert_eq!(filed(Command::Filter), Some("Find"));
    assert_eq!(filed(Command::Status), Some("Change"));
    assert_eq!(filed(Command::Close), Some("Change"));
    assert_eq!(filed(Command::Quit), Some("Program"));
}

/// At a height the whole overlay does not fit, the rows past the edge are a
/// keypress away and the edge says so.
#[test]
fn a_short_terminal_scrolls_the_overlay_rather_than_cutting_it() {
    let mut app = open();
    let top = ui::render_to_string(&mut app, 110, 20, 0);
    assert!(top.contains(&hint()), "the edge says there is more:\n{top}");
    assert!(
        !top.contains(":toggle-mouse"),
        "the last row is below the edge"
    );

    press(&mut app, KeyCode::End);
    let bottom = ui::render_to_string(&mut app, 110, 20, 0);
    assert!(
        bottom.contains(":toggle-mouse"),
        "and `end` reaches it:\n{bottom}"
    );

    press(&mut app, KeyCode::Home);
    ui::render_to_string(&mut app, 110, 20, 0);
    assert_eq!(app.help_scroll, 0, "`home` goes back to the top");
}

/// The keys that move scroll it; anything else puts it away, as it always
/// did. Nothing typed at the overlay reaches the backlog underneath.
#[test]
fn moving_scrolls_and_anything_else_closes() {
    let mut app = open();
    ui::render_to_string(&mut app, 110, 20, 0);

    press(&mut app, KeyCode::Down);
    assert!(app.help, "`↓` scrolls, it does not close");
    assert_eq!(app.help_scroll, 1);
    press(&mut app, KeyCode::Char('k'));
    assert!(app.help);
    assert_eq!(app.help_scroll, 0);

    // `x` closes items. At the overlay it only closes the overlay.
    let open_items = app.items.iter().filter(|i| i.status != "done").count();
    press(&mut app, KeyCode::Char('x'));
    assert!(!app.help, "anything else puts it away");
    assert_eq!(
        app.items.iter().filter(|i| i.status != "done").count(),
        open_items,
        "and does nothing else"
    );
    assert!(app.confirm.is_none(), "not even ask");
}

/// Where it fits, it neither scrolls nor claims to.
#[test]
fn a_tall_enough_terminal_shows_all_of_it_and_says_nothing_about_scrolling() {
    let mut app = open();
    let screen = ui::render_to_string(&mut app, 110, 40, 0);
    assert!(screen.contains(":toggle-mouse"));
    assert!(
        !screen.contains(&hint()),
        "no scroll hint where there is nothing to scroll"
    );
}
