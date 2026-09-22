//! What the keys mean, and what it costs to change them.
//!
//! 0044 promises that the action names a `[keys]` table binds to are fixed at
//! 1.0. Fixing them is the right promise; making it while `l` writes to the
//! backlog is not, and afterwards the correction costs somebody's config file.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Pane};
use harrow::keys::{Command, Keymap};
use harrow::testkit;

fn press(app: &mut App, c: char) {
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
}

/// Every tool that binds `hjkl` binds it to the arrows. `j` and `k` already
/// were `Down` and `Up`, so the half of the reflex that was right taught you
/// to trust the half that was a write.
#[test]
fn h_and_l_do_what_the_arrows_do() {
    let map = Keymap::default();
    for (key, arrow) in [('h', KeyCode::Left), ('l', KeyCode::Right)] {
        assert_eq!(
            map.lookup(KeyCode::Char(key), KeyModifiers::NONE),
            map.lookup(arrow, KeyModifiers::NONE),
            "`{key}` and the arrow beside it disagree"
        );
    }
}

/// And the one that mattered: pressing it must not move an item.
#[test]
fn l_no_longer_writes_to_the_backlog() {
    let mut app = testkit::app();
    app.select_id(5);
    // Item 5 itself, not whatever is under the cursor afterwards: `l` moves
    // the cursor, so comparing the selection compared two different things.
    let status_of = |app: &App| {
        app.items
            .iter()
            .find(|i| i.id == 5)
            .map(|i| i.status.clone())
    };
    let before = status_of(&app);
    press(&mut app, 'l');
    assert_eq!(status_of(&app), before, "`l` still moved it");
    assert!(app.toast.is_none(), "`l` still announced a change");
}

/// `<` and `>` were always bound to them, and are the better glyph for it.
#[test]
fn advance_and_retreat_keep_the_glyphs_that_mean_it() {
    let map = Keymap::default();
    assert_eq!(map.keys_for(Command::Advance), vec![">".to_string()]);
    assert_eq!(map.keys_for(Command::Retreat), vec!["<".to_string()]);
}

/// A mouse-reporting toggle should not hold a letter, and `ctrl-k` for
/// `check` was already the sign that there was nothing left to spend.
#[test]
fn the_occasional_commands_hold_no_key() {
    let map = Keymap::default();
    for command in [Command::ToggleMouse, Command::Diagnostics, Command::Check] {
        assert!(
            map.keys_for(command).is_empty(),
            "{} still holds a key",
            command.name()
        );
    }
}

/// Demoting is only safe because they are reachable by name. Taking a key
/// away without that would be taking the capability away.
#[test]
fn nothing_that_had_a_key_became_unreachable() {
    let mut app = testkit::app();
    press(&mut app, ':');
    let reachable: Vec<&str> = app
        .palette
        .as_ref()
        .expect("the palette opens")
        .matches
        .iter()
        .map(|(c, _)| c.name())
        .collect();
    for command in [Command::ToggleMouse, Command::Diagnostics, Command::Check] {
        assert!(
            reachable.contains(&command.name()),
            "{} is gone entirely",
            command.name()
        );
    }
}

/// The freed letters are the point: they are what sorting and the palette
/// needed, and what the next five years of features will need.
#[test]
fn the_letters_that_were_freed_are_free() {
    let map = Keymap::default();
    for freed in ['m', 'D'] {
        assert!(
            map.lookup(KeyCode::Char(freed), KeyModifiers::NONE)
                .is_none(),
            "`{freed}` is still spent"
        );
    }
    assert!(
        map.lookup(KeyCode::Char('k'), KeyModifiers::CONTROL)
            .is_none(),
        "ctrl-k is still spent"
    );
}

/// `h` and `l` have to move the group cursor, not merely not-write.
#[test]
fn h_and_l_step_between_groups() {
    let mut app = testkit::app();
    app.pane = Pane::Board;
    app.rebuild();
    let before = app.column;
    press(&mut app, 'l');
    assert_ne!(app.column, before, "`l` did not move between columns");
    press(&mut app, 'h');
    assert_eq!(app.column, before, "`h` did not come back");
}
