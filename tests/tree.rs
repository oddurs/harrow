//! The list is a tree, and a tree you cannot fold from the keyboard is a list
//! that happens to have headings in it.
//!
//! Headings were landable only while shut, so nothing on the keyboard could
//! shut an open one: every letter, every `ctrl-` letter, space, the arrows,
//! `enter`, `home`, `<` and `>`, from every row, and no group folded. The only
//! way was the pointer — and `:toggle-mouse` exists to give the pointer back to
//! the terminal, at which point there was no way at all. 0095.
//!
//! What a reader is owed, all of it through keys:

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Row};
use harrow::testkit;

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(code, KeyModifiers::NONE);
}

fn on_heading(app: &App) -> Option<String> {
    match app.rows.get(app.selected)? {
        Row::Group(g) => Some(app.groups[*g].key.clone()),
        _ => None,
    }
}

/// The whole of the bug, stated as the thing it prevented.
#[test]
fn a_heading_folds_and_opens_again_without_a_pointer() {
    let mut app = testkit::app();
    assert!(app.collapsed.is_empty(), "the tree opens expanded");

    press(&mut app, KeyCode::Char('h'));
    let key = on_heading(&app).expect("`h` reaches a heading");

    press(&mut app, KeyCode::Char(' '));
    assert!(
        app.collapsed.contains(&key),
        "`space` on a heading folds it"
    );
    assert_eq!(
        on_heading(&app),
        Some(key.clone()),
        "and the cursor stays on it"
    );

    press(&mut app, KeyCode::Char(' '));
    assert!(!app.collapsed.contains(&key), "and `space` again opens it");
}

/// Row 0 is a heading, and a heading could not be stood on, so `home` went to
/// row 1.
#[test]
fn home_goes_to_the_top_of_the_list() {
    let mut app = testkit::app();
    press(&mut app, KeyCode::End);
    press(&mut app, KeyCode::Home);
    assert_eq!(app.selected, 0);
}

/// Back is the parent first — the heading of the group you are in — and then
/// the heading before that. One press from folding what you are in.
#[test]
fn back_from_an_item_is_its_own_heading_before_the_one_above() {
    let mut app = testkit::app();
    let last_heading = app
        .rows
        .iter()
        .rposition(|r| matches!(r, Row::Group(_)))
        .expect("more than one group");
    let child = last_heading + 1;
    assert!(matches!(app.rows.get(child), Some(Row::Item(_))));
    app.selected = child;

    press(&mut app, KeyCode::Char('h'));
    assert_eq!(app.selected, last_heading, "the parent");

    press(&mut app, KeyCode::Char('h'));
    assert!(app.selected < last_heading, "then the group above");
    assert!(on_heading(&app).is_some());
}

/// `space` marks and moves on. With headings on the path, moving on to the
/// next row would stop on a heading, and the next `space` would fold a group
/// away in the middle of marking a run.
#[test]
fn a_run_of_marks_steps_over_a_heading_rather_than_folding_it() {
    let mut app = testkit::app();
    let heading = app
        .rows
        .iter()
        .rposition(|r| matches!(r, Row::Group(_)))
        .expect("a heading");
    let before = heading - 1;
    let after = heading + 1;
    let id = |app: &App, row: usize| match app.rows[row] {
        Row::Item(i) => app.items[i].id,
        _ => panic!("row {row} is not an item"),
    };
    let (first, second) = (id(&app, before), id(&app, after));
    app.selected = before;

    press(&mut app, KeyCode::Char(' '));
    assert_eq!(app.selected, after, "over the heading, to the next item");
    press(&mut app, KeyCode::Char(' '));

    assert_eq!(app.marked, HashSet::from([first, second]));
    assert!(app.collapsed.is_empty(), "and nothing folded");
}

/// A heading with nothing behind it is still something under the cursor, and
/// the pane says what rather than dropping out and moving the list sideways.
#[test]
fn a_heading_with_no_item_behind_it_is_still_read() {
    let mut app = testkit::app();
    app.set_grouping("status");
    app.selected = app
        .rows
        .iter()
        .position(|r| matches!(r, Row::Group(_)))
        .expect("a heading");
    assert!(app.selected_item().is_none(), "a status is not an item");
    assert!(app.selected_group().is_some(), "but it is a group");
}
