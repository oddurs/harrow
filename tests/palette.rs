//! Every command has a name before it has a key.
//!
//! Forty-eight commands over forty-one spent letters is why a mouse-reporting
//! toggle held one and `check` ended up on `ctrl-k`. The palette is what makes
//! the keymap stop being a zero-sum game: a new command lands here and earns a
//! key later, if it earns one at all.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App, Pane};
use harrow::keys::Command;
use harrow::{testkit, ui};

fn key(app: &mut App, c: char) -> Action {
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE)
}

fn open(app: &mut App, typed: &str) {
    key(app, ':');
    for c in typed.chars() {
        key(app, c);
    }
}

fn named(app: &App) -> Vec<&'static str> {
    app.palette
        .as_ref()
        .expect("the palette is open")
        .matches
        .iter()
        .map(|(c, _)| c.name())
        .collect()
}

/// The property that matters most: it is generated, so a command cannot exist
/// in harrow and be missing from it.
#[test]
fn every_command_there_is_can_be_reached_by_name() {
    let mut app = testkit::app();
    open(&mut app, "");
    assert_eq!(named(&app).len(), Command::ALL.len());
}

#[test]
fn typing_narrows_it_and_a_name_beats_a_description() {
    let mut app = testkit::app();
    open(&mut app, "cl");
    let found = named(&app);
    assert!(found.contains(&"claim"), "{found:?}");
    assert!(found.contains(&"close"), "{found:?}");
    // `claim` is ranked above `close` because the name begins with what was
    // typed in both, and `claim` is declared first — but neither may be
    // ranked below a command that only matched its sentence.
    let first_by_name = found
        .iter()
        .position(|n| n.starts_with("cl"))
        .expect("something matched by name");
    assert_eq!(
        first_by_name, 0,
        "a description outranked a name: {found:?}"
    );
}

/// The sentence is searchable too, because nobody remembers the stable name.
/// `close` is described as "close it, with a confirm" and its name says
/// nothing about confirming.
#[test]
fn a_command_is_findable_by_what_it_does() {
    let mut app = testkit::app();
    open(&mut app, "confirm");
    assert!(named(&app).contains(&"close"), "{:?}", named(&app));
}

#[test]
fn running_one_does_what_the_key_does() {
    let mut app = testkit::app();
    open(&mut app, "view-board");
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.palette.is_none(), "it stayed open");
    assert_eq!(app.pane, Pane::Board);
}

/// It returns an `Action` like any other key, so what is reachable here stays
/// assertable with no repository underneath.
#[test]
fn a_command_that_writes_returns_the_action_it_would_have() {
    let mut app = testkit::app();
    app.select_id(3);
    open(&mut app, "copy-view");
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(action, Action::Copy(_)), "{action:?}");
}

#[test]
fn escape_leaves_it_and_changes_nothing() {
    let mut app = testkit::app();
    let before = app.pane;
    open(&mut app, "view-board");
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.palette.is_none());
    assert_eq!(app.pane, before);
}

/// The key on the right is what makes this teach the keymap rather than
/// replace it — and a command with no key has to say so rather than be hidden.
#[test]
fn each_row_shows_the_key_that_does_the_same_thing() {
    let mut app = testkit::app();
    open(&mut app, "");
    let rows = &app.palette.as_ref().unwrap().matches;
    let claim = rows.iter().find(|(c, _)| *c == Command::Claim).unwrap();
    assert_eq!(claim.1.as_deref(), Some("c"));

    let screen = ui::render_to_string(&mut app, 100, 24, 0);
    assert!(screen.contains("the key on the right"), "{screen}");
}

#[test]
fn a_name_nothing_matches_says_so() {
    let mut app = testkit::app();
    open(&mut app, "zzzz");
    assert!(named(&app).is_empty());
    let screen = ui::render_to_string(&mut app, 100, 24, 0);
    assert!(screen.contains("nothing by that name"), "{screen}");
}

/// Typing into the palette must not reach the keymap underneath: `x` is close.
#[test]
fn typing_a_verb_into_it_does_not_run_the_verb() {
    let mut app = testkit::app();
    open(&mut app, "x");
    assert!(app.confirm.is_none(), "`x` closed something while typing");
    assert_eq!(app.palette.as_ref().unwrap().typed, "x");
}
