//! Sorting: the third question a backlog answers.
//!
//! Which items, in what order, grouped how. `--sort` existed as a flag and
//! `App::sort` existed as state, and nothing but argv ever set it — so to look
//! at the same backlog oldest-first you quit and started again, losing the
//! filter you had typed, the marks you had set and the item you were reading.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Row};
use harrow::{testkit, ui};

fn key(app: &mut App, c: char) {
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
}

/// The text box. `S` opens the list of fields since 0090; typing a whole
/// order — `-priority,updated,id` — is what this key is for, because a list
/// of choices cannot express one.
fn open_box(app: &mut App) {
    app.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
}

/// `S` prefills with the order in force — you are editing it, not starting
/// over — so a test that types a whole spec has to clear it first.
fn type_sort(app: &mut App, spec: &str) {
    open_box(app);
    for _ in 0..40 {
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
    }
    for c in spec.chars() {
        key(app, c);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
}

/// That prefill is the point of the key: it is the order you are changing.
#[test]
fn it_opens_on_the_order_in_force() {
    let mut app = testkit::app();
    app.sort = "-priority".into();
    open_box(&mut app);
    assert_eq!(app.input, "-priority");
}

fn ids(app: &App) -> Vec<u32> {
    app.rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect()
}

#[test]
fn a_key_reorders_the_backlog_without_restarting() {
    let mut app = testkit::app();
    app.group_by = "none".into();
    app.rebuild();
    let before = ids(&app);

    type_sort(&mut app, "-id");
    let after = ids(&app);
    assert_ne!(before, after, "nothing moved");
    let mut descending = after.clone();
    descending.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(after, descending, "not in reverse id order: {after:?}");
}

/// `-` reverses, which is what `--sort` already accepts — so the segment is
/// the flag's own syntax and nothing has to be learned twice.
#[test]
fn a_leading_dash_reverses_it() {
    let mut app = testkit::app();
    app.group_by = "none".into();
    app.rebuild();

    type_sort(&mut app, "id");
    let up = ids(&app);
    type_sort(&mut app, "-id");
    let down = ids(&app);
    assert_eq!(down, up.iter().rev().copied().collect::<Vec<_>>());
}

/// Reordering is a rearrangement of what is already there, so the item under
/// the cursor should still be under it afterwards.
#[test]
fn sorting_does_not_lose_your_place() {
    let mut app = testkit::app();
    app.group_by = "none".into();
    app.rebuild();
    app.select_id(4);
    let before = app.selected_item().map(|i| i.id);
    type_sort(&mut app, "-id");
    assert_eq!(app.selected_item().map(|i| i.id), before);
}

/// The list reorders as you type, the way it narrows as you type a filter.
#[test]
fn it_reorders_as_you_type() {
    let mut app = testkit::app();
    app.group_by = "none".into();
    app.rebuild();
    let before = ids(&app);
    open_box(&mut app);
    for c in "-id".chars() {
        key(&mut app, c);
    }
    assert_ne!(ids(&app), before, "nothing moved until enter");
}

/// Backing out has to put it back: the preview changed the screen while you
/// typed, so `esc` that kept the change would be indistinguishable from `↵`.
#[test]
fn escape_puts_the_order_back() {
    let mut app = testkit::app();
    app.group_by = "none".into();
    app.rebuild();
    let before = ids(&app);
    open_box(&mut app);
    for c in "-id".chars() {
        key(&mut app, c);
    }
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(ids(&app), before);
}

/// The fields come from the project, not from a list here.
#[test]
fn what_it_offers_comes_from_the_schema() {
    let app = testkit::app();
    let fields = app.sort_fields();
    assert!(fields.contains(&"priority".to_string()), "{fields:?}");
    assert!(fields.contains(&"area".to_string()), "{fields:?}");
    assert!(fields.contains(&"status".to_string()), "{fields:?}");
    assert!(!fields.contains(&"severity".to_string()), "{fields:?}");
}

/// A field nothing declares orders by nothing, which is indistinguishable
/// from an order somebody asked for. The filter already says so; so does this.
#[test]
fn a_field_the_project_has_not_got_is_said_out_loud() {
    let mut app = testkit::app();
    open_box(&mut app);
    for c in "severity".chars() {
        key(&mut app, c);
    }
    assert_eq!(app.unknown_sort(), vec!["severity".to_string()]);
    let screen = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(screen.contains("no such field: severity"), "{screen}");
}

/// The order in force is on the line, and says whether it is a choice.
#[test]
fn the_order_is_stated_whether_or_not_anybody_chose_it() {
    let mut app = testkit::app();
    let default = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(
        default
            .lines()
            .nth(1)
            .unwrap_or_default()
            .contains("status"),
        "the default order is not stated"
    );
    type_sort(&mut app, "-priority");
    let chosen = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(
        chosen
            .lines()
            .nth(1)
            .unwrap_or_default()
            .contains("-priority"),
        "{chosen}"
    );
}
