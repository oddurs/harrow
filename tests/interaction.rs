//! Driving the interface with keys, and what comes back out of it.
//!
//! Every write is asserted as the `cairn` invocation it would produce. Nothing
//! here runs cairn: the point is that the decision and the doing are separable,
//! which is why a test can hold the whole write path to account.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App};
use harrow::keys::Command;
use harrow::testkit;

fn app() -> App {
    let mut app = testkit::app();
    app.select_id(3);
    app
}

fn press(app: &mut App, key: char) -> Action {
    app.handle_key(KeyCode::Char(key), KeyModifiers::NONE)
}

fn args(action: &Action) -> Vec<String> {
    match action {
        Action::Write(change) => change.args.clone(),
        other => panic!("expected a change, got {other:?}"),
    }
}

#[test]
fn claiming_and_releasing_are_the_commands_they_look_like() {
    let mut app = app();
    assert_eq!(args(&press(&mut app, 'c')), vec!["claim", "3"]);
    assert_eq!(args(&press(&mut app, 'C')), vec!["release", "3"]);
}

#[test]
fn closing_asks_first_and_only_the_answer_writes() {
    let mut app = app();
    assert_eq!(
        press(&mut app, 'x'),
        Action::None,
        "the key itself writes nothing"
    );
    assert!(app.confirm.is_some(), "it has to ask");

    // Anything but yes cancels.
    assert_eq!(
        app.handle_key(KeyCode::Char('n'), KeyModifiers::NONE),
        Action::None
    );
    assert!(app.confirm.is_none());

    press(&mut app, 'x');
    let action = app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE);
    assert_eq!(args(&action), vec!["close", "3"]);
}

#[test]
fn a_change_carries_the_command_that_puts_it_back() {
    let mut app = app();
    let Action::Write(change) = press(&mut app, 'l') else {
        panic!("advancing a status is a change");
    };
    assert_eq!(change.args, vec!["set", "3", "status=done"]);
    assert_eq!(
        change.undo.as_deref(),
        Some("cairn set 3 status=doing"),
        "the toast has to say how to undo it"
    );
}

#[test]
fn a_status_moves_one_step_at_a_time_and_stops_at_the_ends() {
    let mut app = app();
    assert_eq!(
        args(&press(&mut app, 'h')),
        vec!["set", "3", "status=backlog"]
    );

    // Item 2 is finished, so it has to be shown before it can be landed on.
    app.show_all = true;
    app.rebuild();
    app.select_id(2);
    assert_eq!(app.selected_item().map(|i| i.id), Some(2));
    assert_eq!(press(&mut app, 'l'), Action::None);
    let (message, _, _) = app.toast.as_ref().expect("and says why");
    assert!(message.contains("last status"), "{message}");
}

#[test]
fn a_status_never_steps_into_a_column_the_project_hid() {
    // `dropped` has board = false: it is not a stage of the work, so `l` and
    // `h` must not walk into it.
    let mut app = app();
    app.show_all = true;
    app.rebuild();
    app.select_id(2);
    assert_eq!(
        press(&mut app, 'l'),
        Action::None,
        "done is the last column shown"
    );
    assert_eq!(
        args(&press(&mut app, 'h')),
        vec!["set", "2", "status=doing"]
    );
}

#[test]
fn the_picker_sets_a_field_and_esc_sets_nothing() {
    let mut app = app();
    press(&mut app, 'p');
    let picker = app.picker.as_ref().expect("a priority picker");
    assert_eq!(picker.field, "priority");
    assert_eq!(picker.options.len(), 5, "four values and a way to clear it");
    assert_eq!(
        picker.selected, 1,
        "it opens on what the item already says, which is p1"
    );

    // A digit picks outright, which makes `p 1` the whole gesture.
    app.handle_key(KeyCode::Char('1'), KeyModifiers::NONE);
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(args(&action), vec!["set", "3", "priority=p0"]);

    press(&mut app, 'p');
    assert_eq!(
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE),
        Action::None
    );
    assert!(app.picker.is_none());
}

/// Three statuses can begin with the same letter. Stepping between them beats
/// silently landing on whichever was declared first.
#[test]
fn a_letter_steps_between_the_options_that_share_it() {
    let mut app = app();
    press(&mut app, 's');
    let value = |app: &App| {
        let p = app.picker.as_ref().expect("picker");
        p.options[p.selected].0.clone()
    };
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert_eq!(value(&app), "done");
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert_eq!(value(&app), "dropped");
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert_eq!(value(&app), "doing", "and round again");
}

#[test]
fn setting_a_field_to_what_it_already_says_is_not_a_change() {
    let mut app = app();
    press(&mut app, 's');
    // `doing` is where item 3 already is.
    let at = app
        .picker
        .as_ref()
        .expect("picker")
        .options
        .iter()
        .position(|(v, _, _)| v == "doing")
        .expect("doing is a status");
    app.picker.as_mut().expect("picker").selected = at;
    assert_eq!(
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE),
        Action::None
    );
}

#[test]
fn a_new_item_is_typed_rather_than_picked() {
    let mut app = app();
    press(&mut app, 'n');
    for c in "Write the readme".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(args(&action), vec!["new", "Write the readme"]);

    // An empty title creates nothing.
    press(&mut app, 'n');
    assert_eq!(
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE),
        Action::None
    );
}

#[test]
fn the_filter_narrows_as_it_is_typed() {
    let mut app = testkit::app();
    let before = app.rows.len();
    press(&mut app, '/');
    for c in "board".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert!(
        app.rows.len() < before,
        "the list should have narrowed already"
    );
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.filter, "board");

    // esc puts everything back.
    press(&mut app, '/');
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.rows.len(), before);
}

#[test]
fn a_saved_view_is_a_filter_the_project_wrote_down() {
    let mut app = testkit::app();
    app.view = Some("now".into());
    app.ingest(harrow::testkit::report());
    let ids: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            harrow::app::Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec![3], "`now` is category=active");

    app.run(Command::Back);
    assert!(app.view.is_none(), "esc backs out of a view");
}

#[test]
fn reading_an_item_scrolls_and_any_other_key_leaves() {
    let mut app = app();
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.reading);
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.read_scroll, 1);
    app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(!app.reading, "and q must close the reader, not quit harrow");
}

#[test]
fn history_is_asked_of_cairn_rather_than_of_git() {
    let mut app = app();
    match app.run(Command::History) {
        Action::History(id) => assert_eq!(id, 3),
        other => panic!("expected a history request, got {other:?}"),
    }

    // The overlay takes the keys while it is open, and any other key closes it.
    app.show_history(3, Ok("2026-09-02  somebody  created\n".into()));
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.history.as_ref().map(|h| h.scroll), Some(1));
    app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(
        app.history.is_none(),
        "and q closes it rather than quitting"
    );
}

#[test]
fn editing_hands_over_the_file_rather_than_the_id() {
    let mut app = app();
    match press(&mut app, 'e') {
        Action::Edit(path) => assert!(path.ends_with("0003-draw-the-list.md"), "{path:?}"),
        other => panic!("expected an edit, got {other:?}"),
    }
}

#[test]
fn the_selection_survives_switching_between_the_list_and_the_board() {
    let mut app = app();
    let before = app.selected_item().map(|i| i.id);
    app.run(Command::ViewBoard);
    assert_eq!(app.selected_item().map(|i| i.id), before);
    app.run(Command::ViewBoard);
    assert_eq!(app.selected_item().map(|i| i.id), before);
}

#[test]
fn moving_between_columns_keeps_the_cursor_somewhere_real() {
    let mut app = testkit::app();
    app.pane = harrow::app::Pane::Board;
    for _ in 0..12 {
        app.run(Command::NextGroup);
        assert!(
            app.check_invariants().is_ok(),
            "{:?}",
            app.check_invariants()
        );
    }
}

#[test]
fn nothing_selected_is_not_a_crash() {
    let mut app = App::new();
    for command in harrow::keys::Command::ALL {
        if command == Command::Quit {
            continue;
        }
        let _ = app.run(command);
        assert!(app.check_invariants().is_ok(), "{command:?}");
    }
}
