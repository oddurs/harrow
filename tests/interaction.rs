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
fn claiming_is_the_command_it_looks_like() {
    let mut app = app();
    assert_eq!(args(&press(&mut app, 'c')), vec!["claim", "3"]);
}

/// The one moment where the person letting go knows exactly why and the next
/// person to pick it up is about to need it. cairn records the reason as a
/// note and shows it on the next claim.
#[test]
fn handing_an_item_back_asks_why_and_takes_no_answer_for_an_answer() {
    let mut app = app();
    press(&mut app, 'C');
    assert!(app.editing.is_some(), "it asks before it writes");
    for c in "blocked on the parser".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert_eq!(
        args(&app.handle_key(KeyCode::Enter, KeyModifiers::NONE)),
        vec!["release", "3", "--reason", "blocked on the parser"]
    );
}

/// A prompt on a one-keystroke gesture has to be answerable with one
/// keystroke, or it gets muscle-memoried past — which is worse than not
/// asking at all.
#[test]
fn skipping_the_reason_hands_it_back_the_way_it_always_did() {
    let mut app = app();
    press(&mut app, 'C');
    assert_eq!(
        args(&app.handle_key(KeyCode::Enter, KeyModifiers::NONE)),
        vec!["release", "3"]
    );
}

#[test]
fn esc_keeps_the_item_rather_than_handing_it_back_without_a_reason() {
    let mut app = app();
    press(&mut app, 'C');
    assert_eq!(
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE),
        Action::None
    );
    assert!(app.editing.is_none(), "and the box closes");
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

/// A proposal is somebody asking. Accepting is cairn's own command, so the
/// change lands the way it would have if they had the say in the first place.
#[test]
fn a_proposal_is_accepted_through_cairn_and_asks_first() {
    let mut app = testkit::app();
    app.select_id(6);
    let item = app.selected_item().expect("item 6");
    assert_eq!(item.proposals.len(), 1, "the fixture has one");

    assert_eq!(app.run(Command::Accept), Action::None, "it asks first");
    let confirm = app.confirm.as_ref().expect("a confirmation");
    assert!(confirm.prompt.contains("p3 → p0"), "{}", confirm.prompt);
    assert!(confirm.detail.contains("Nobody can install"), "and why");

    let action = app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE);
    match action {
        Action::Write(change) => {
            assert_eq!(change.args, vec!["proposals", "--accept", "6"]);
            assert_eq!(
                change.undo.as_deref(),
                Some("cairn set 6 priority=p3"),
                "and it says how to put it back"
            );
        }
        other => panic!("expected an accept, got {other:?}"),
    }
}

#[test]
fn accepting_nothing_says_so() {
    let mut app = testkit::app();
    app.select_id(3);
    assert_eq!(app.run(Command::Accept), Action::None);
    assert!(app.confirm.is_none());
    let (message, _, _) = app.toast.as_ref().expect("it says so");
    assert!(message.contains("nothing proposed"), "{message}");
}

#[test]
fn the_detail_pane_scrolls_without_taking_the_list_with_it() {
    let mut app = app();
    let (offset, selected) = (app.offset, app.selected);

    press(&mut app, 'J');
    press(&mut app, 'J');
    assert_eq!(app.detail.at(3), 2);
    assert_eq!((app.offset, app.selected), (offset, selected));

    press(&mut app, 'K');
    assert_eq!(app.detail.at(3), 1);
}

#[test]
fn the_detail_pane_starts_at_the_top_of_whatever_is_selected() {
    let mut app = app();
    press(&mut app, 'J');
    assert_eq!(app.detail.at(3), 1);

    app.select_id(4);
    assert_eq!(app.detail.at(4), 0, "a different item is not part-read");

    app.select_id(3);
    assert_eq!(
        app.detail.at(3),
        1,
        "and the one you left is where you left"
    );
}

/// There is nothing on the stats pane to select, so the keys that would move a
/// cursor move the pane — otherwise its bottom is unreachable on a short
/// terminal, and the list's cursor moves where nobody can see it.
#[test]
fn the_stats_pane_scrolls_with_the_keys_that_move_a_cursor_elsewhere() {
    let mut app = app();
    app.pane = harrow::app::Pane::Stats;
    let selected = app.selected;

    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.stats_scroll, 2);
    assert_eq!(app.selected, selected, "the list's cursor did not move");

    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(app.stats_scroll, 1);

    // The end of the pane is wherever the drawing says it is.
    app.run(Command::Last);
    let _ = harrow::ui::render_frame(&mut app, 62, 24, 0);
    assert!(app.stats_scroll > 0, "there is more than one screen of it");
    assert!(
        app.stats_scroll < u16::MAX,
        "and it was pulled back to the end rather than left past it"
    );
}

/// `cairn note` appends and never replaces, which is the whole reason it
/// exists rather than a body that overwrites: somebody writing down their
/// reasoning must not be able to erase what came before.
#[test]
fn a_note_is_appended_through_cairn_rather_than_written_here() {
    let mut app = app();
    press(&mut app, 'N');
    for c in "tried the obvious thing; it deadlocks".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        args(&action),
        vec![
            "note".to_string(),
            "3".to_string(),
            "tried the obvious thing; it deadlocks".to_string()
        ]
    );
}

#[test]
fn an_empty_note_writes_nothing() {
    let mut app = app();
    press(&mut app, 'N');
    assert_eq!(
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE),
        Action::None
    );
    assert!(app.editing.is_none(), "and the box closes");
}

/// `cairn note` takes one id. Quietly noting whichever item the cursor was on
/// while three are marked would be the wrong kind of surprise.
#[test]
fn a_note_says_it_goes_on_one_item_rather_than_guessing() {
    let mut app = app();
    app.run(Command::ToggleGroup);
    app.run(Command::ToggleGroup);
    assert!(!app.marked.is_empty());

    assert_eq!(press(&mut app, 'N'), Action::None);
    assert!(app.editing.is_none(), "it does not even open the box");
    let said = app
        .toast
        .as_ref()
        .map(|(m, _, _)| m.clone())
        .unwrap_or_default();
    assert!(said.contains("one item"), "{said}");
}
