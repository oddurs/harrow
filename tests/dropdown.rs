//! A list under the word it changes.
//!
//! The toolbar had four controls and four ways of working: two text boxes, a
//! side panel, a blind cycle, and a picker over the middle of the screen. They
//! sit next to each other and answer the same kind of question — *how am I
//! looking at this* — and not one behaved like its neighbour.

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{App, Hit, Pane};
use harrow::keys::Command;
use harrow::{testkit, ui};

/// The anchor comes from where the segment was drawn, so a frame has to have
/// been drawn before a key can open anything under it.
fn drawn(app: &mut App) {
    let _ = ui::render_frame(app, 110, 26, 0);
}

fn press(app: &mut App, c: char) {
    drawn(app);
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
}

fn labels(app: &App) -> Vec<String> {
    app.dropdown
        .as_ref()
        .expect("a dropdown is open")
        .options
        .iter()
        .map(|(_, label, _)| label.clone())
        .collect()
}

/// The worst of the four: `v` stepped to the next axis with no way to see
/// what the axes were, so finding `assignee` in a project with six meant
/// pressing it six times and reading the line each time to find out where you
/// had landed.
#[test]
fn v_shows_the_arrangements_instead_of_cycling_blindly() {
    let mut app = testkit::app();
    press(&mut app, 'v');
    let axes = labels(&app);
    assert!(axes.contains(&"milestone".to_string()), "{axes:?}");
    assert!(axes.contains(&"status".to_string()), "{axes:?}");
    assert!(axes.contains(&"flat".to_string()), "{axes:?}");
    // And it opens on what is in force, so `↵` alone changes nothing.
    assert_eq!(app.dropdown.as_ref().unwrap().chosen(), Some("milestone"));
}

/// The cycle is still there for anybody who wants it — it just no longer has
/// the key that should show you the choices.
#[test]
fn the_blind_cycle_is_still_reachable() {
    let mut app = testkit::app();
    let before = app.group_by.clone();
    app.run(Command::CycleGroup);
    assert_ne!(app.group_by, before);
    assert!(app.dropdown.is_none(), "cycling should not open a list");
}

#[test]
fn choosing_an_arrangement_rearranges() {
    let mut app = testkit::app();
    press(&mut app, 'v');
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.group_by, "status");
    assert!(app.dropdown.is_none());
}

/// It is drawn under the word it is about, not over the middle of the screen.
#[test]
fn it_opens_under_the_segment_it_changes() {
    let mut app = testkit::app();
    drawn(&mut app);
    let segment = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, Hit::Run(Command::GroupBy)))
        .map(|(rect, _)| *rect)
        .expect("the toolbar drew a grouping segment");

    app.handle_key(KeyCode::Char('v'), KeyModifiers::NONE);
    let open = app.dropdown.as_ref().unwrap();
    assert_eq!(open.anchor.x, segment.x);
    assert_eq!(open.anchor.y, segment.y);

    // And the list itself lands under it rather than in the middle.
    drawn(&mut app);
    let screen = ui::render_to_string(&mut app, 110, 26, 0);
    let row = screen
        .lines()
        .nth(2)
        .expect("the row below the toolbar")
        .to_string();
    // By character, not by byte: a row of box-drawing is three bytes a cell.
    let at = row
        .chars()
        .position(|c| c == '╭')
        .expect("a box opens on that row");
    assert!(
        at.abs_diff(segment.x as usize) <= 2,
        "the list is {at} but the segment is at {}",
        segment.x
    );
}

#[test]
fn typing_narrows_it_and_escape_leaves_everything_alone() {
    let mut app = testkit::app();
    let before = app.group_by.clone();
    press(&mut app, 'v');
    for c in "stat".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert_eq!(labels(&app), vec!["status".to_string()]);
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.dropdown.is_none());
    assert_eq!(app.group_by, before);
}

/// An order has a direction, which a list of bare names cannot say.
#[test]
fn the_order_list_shows_and_turns_the_direction() {
    let mut app = testkit::app();
    press(&mut app, 'S');
    let notes: Vec<String> = app
        .dropdown
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|(_, _, note)| note.clone())
        .collect();
    assert!(
        notes.iter().any(|n| n == "↑"),
        "no direction shown: {notes:?}"
    );

    // Taking the key it is already ordered by turns it around, which is what
    // anybody wants of a heading they click twice.
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.sort, "-status");
    press(&mut app, 'S');
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.sort, "status");
}

/// A filter is compositional — clauses, ranges, negation — and a list of
/// choices cannot express one, so the text box stays.
#[test]
fn an_order_can_still_be_typed_in_full() {
    let mut app = testkit::app();
    app.run(Command::SortBy);
    assert!(app.dropdown.is_none(), "it opened a list instead of a box");
    for c in "-priority,updated".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.sort, "-priority,updated");
}

/// A view is the filter, and the toolbar shows it in the filter segment — so
/// that is the word its list belongs under.
#[test]
fn views_open_under_the_filter_they_replace() {
    let mut app = testkit::app();
    drawn(&mut app);
    let filter = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, Hit::Run(Command::Filter)))
        .map(|(rect, _)| *rect)
        .expect("a filter segment");
    app.handle_key(KeyCode::Char('V'), KeyModifiers::NONE);
    assert_eq!(app.dropdown.as_ref().unwrap().anchor.x, filter.x);
}

#[test]
fn clicking_a_segment_opens_it_and_clicking_a_row_takes_it() {
    let mut app = testkit::app();
    drawn(&mut app);
    let segment = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, Hit::Run(Command::GroupBy)))
        .map(|(rect, _)| *rect)
        .expect("a grouping segment");
    let click = |app: &mut App, x: u16, y: u16| {
        app.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    };
    click(&mut app, segment.x, segment.y);
    assert!(app.dropdown.is_some(), "the click did not open it");

    drawn(&mut app);
    let (row, _) = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, Hit::Choose(1)))
        .expect("the second row is clickable");
    let (x, y) = (row.x, row.y);
    click(&mut app, x, y);
    assert!(app.dropdown.is_none());
    assert_eq!(app.group_by, "status");
}

/// An anchor near the right edge must not push the list off the screen.
#[test]
fn it_stays_on_screen_at_the_right_edge() {
    let mut app = testkit::app();
    app.pane = Pane::List;
    drawn(&mut app);
    app.handle_key(KeyCode::Char('v'), KeyModifiers::NONE);
    if let Some(d) = app.dropdown.as_mut() {
        d.anchor.x = 108;
    }
    let screen = ui::render_to_string(&mut app, 110, 26, 0);
    for line in screen.lines() {
        assert!(
            line.chars().count() <= 110,
            "the list ran off the edge: {line}"
        );
    }
    assert!(screen.contains("arrange by"), "{screen}");
}
