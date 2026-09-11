//! Marking a set, and changing all of it at once.
//!
//! The point of the feature is that "everything in this milestone is now p2" is
//! one decision. These check it stays one decision and one write.

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{Action, App, Hit};
use harrow::keys::Command;
use harrow::testkit;
use harrow::ui;

fn app() -> App {
    let mut app = testkit::app();
    app.select_id(5);
    app
}

fn args(action: &Action) -> Vec<String> {
    match action {
        Action::Write(change) => change.args.clone(),
        other => panic!("expected a change, got {other:?}"),
    }
}

/// Marks are answered by the confirmation, so a bulk change is two steps.
fn confirm(app: &mut App) -> Action {
    assert!(app.confirm.is_some(), "a bulk change has to ask first");
    app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE)
}

#[test]
fn space_marks_and_moves_on() {
    let mut app = app();
    let first = app.selected_item().map(|i| i.id).expect("an item");
    app.run(Command::ToggleGroup);
    assert!(app.marked.contains(&first));
    assert_ne!(
        app.selected_item().map(|i| i.id),
        Some(first),
        "marking moves to the next, so a run of them is a run of keystrokes"
    );
}

#[test]
fn space_on_a_heading_still_folds_it() {
    let mut app = testkit::app();
    app.selected = 0;
    let before = app.rows.len();
    app.run(Command::ToggleGroup);
    assert!(app.rows.len() < before, "a heading folds");
    assert!(app.marked.is_empty(), "and marks nothing");
}

#[test]
fn one_decision_is_one_write() {
    let mut app = testkit::app();
    for id in [3, 5, 6] {
        app.marked.insert(id);
    }
    app.select_id(3);

    let action = app.run(Command::Priority);
    assert!(action == Action::None, "the picker opens first");
    let picker = app.picker.as_ref().expect("a priority picker");
    let at = picker
        .options
        .iter()
        .position(|(v, _, _)| v == "p0")
        .expect("p0");
    app.picker.as_mut().expect("picker").selected = at;
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    let action = confirm(&mut app);
    assert_eq!(
        args(&action),
        vec!["set", "3", "5", "6", "priority=p0"],
        "one invocation, every id in it"
    );
}

#[test]
fn the_prompt_says_how_many() {
    let mut app = testkit::app();
    for id in [3, 5, 6] {
        app.marked.insert(id);
    }
    app.run(Command::Claim);
    let prompt = &app.confirm.as_ref().expect("it asks").prompt;
    assert!(prompt.contains('3'), "{prompt}");
}

#[test]
fn a_marked_set_that_is_the_whole_filter_becomes_one_filtered_change() {
    let mut app = testkit::app();
    // Filter to one status, then mark everything it is showing — so every one
    // of them genuinely needs the change, and the marked set really is the
    // filter rather than a subset of it.
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    for c in "status=backlog".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    let showing: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            harrow::app::Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    assert!(showing.len() > 1, "the filter has to show more than one");
    app.marked = showing.iter().copied().collect();

    app.select_id(showing[0]);
    let action = app.run(Command::Advance);
    let action = if action == Action::None {
        confirm(&mut app)
    } else {
        action
    };
    let args = args(&action);
    assert_eq!(
        &args[..3],
        &["set", "--filter", "status=backlog"],
        "the same change, said the way somebody would have typed it: {args:?}"
    );
    assert!(args.contains(&"--yes".to_string()), "{args:?}");
}

#[test]
fn nothing_that_is_already_right_is_written_again() {
    let mut app = testkit::app();
    app.show_all = true;
    app.rebuild();
    // 2 is done; 3 is not.
    app.marked.insert(2);
    app.marked.insert(3);
    app.select_id(3);

    let action = app.run(Command::Reopen);
    assert_eq!(
        args(&action),
        vec!["reopen", "2"],
        "only the one that is closed"
    );
}

#[test]
fn esc_clears_the_marks_before_anything_else() {
    let mut app = testkit::app();
    app.filter = "p1".into();
    app.marked.insert(3);
    app.run(Command::Back);
    assert!(app.marked.is_empty());
    assert_eq!(app.filter, "p1", "the filter is still there");
    app.run(Command::Back);
    assert!(app.filter.is_empty(), "and goes on the next one");
}

#[test]
fn a_mark_is_visible_without_reading_a_count() {
    let mut app = testkit::app();
    let plain = ui::render_to_string(&mut app, 100, 24, 0);
    app.marked.insert(3);
    let marked = ui::render_to_string(&mut app, 100, 24, 0);
    assert_ne!(plain, marked, "marking has to change the screen");
    assert!(marked.contains('▌'), "the row carries a mark:\n{marked}");
    assert!(marked.contains("1 marked"), "and the header says how many");
}

#[test]
fn the_pointer_marks_one_or_a_range() {
    let mut app = testkit::app();
    let _ = ui::render_frame(&mut app, 100, 24, 0);

    let row_at = |app: &App, want: usize| {
        let (area, _) = app
            .hits
            .iter()
            .find(|(_, hit)| *hit == Hit::Row(want))
            .expect("a row on screen");
        (area.x, area.y)
    };
    let click = |app: &mut App, (x, y): (u16, u16), mods: KeyModifiers| {
        app.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: mods,
        })
    };

    // Rows 1..3 are items under the first heading.
    let first = row_at(&app, 1);
    click(&mut app, first, KeyModifiers::CONTROL);
    assert_eq!(app.marked.len(), 1, "ctrl-click marks one");

    let _ = ui::render_frame(&mut app, 100, 24, 0);
    let third = row_at(&app, 3);
    click(&mut app, third, KeyModifiers::SHIFT);
    assert_eq!(app.marked.len(), 3, "shift-click marks the range between");
}

#[test]
fn a_read_only_backlog_refuses_a_bulk_change() {
    let mut app = testkit::app();
    app.writable = false;
    app.marked.insert(3);
    app.marked.insert(5);
    assert_eq!(app.run(Command::Claim), Action::None);
    assert!(app.confirm.is_none(), "and does not ask about it either");
}
