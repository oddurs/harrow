//! Driving the interface with keys, and what comes back out of it.
//!
//! Every write is asserted as the `cairn` invocation it would produce. Nothing
//! here runs cairn: the point is that the decision and the doing are separable,
//! which is why a test can hold the whole write path to account.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App, Focus};
use harrow::keys::Command;
use harrow::testkit;
use harrow::ui;

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

/// The popover took every key and gave back only scrolling, so reading four
/// items was four opens and four closes. A panel leaves the backlog where it
/// is: the selection goes on moving and the panel follows it.
#[test]
fn the_panel_takes_the_keys_when_it_is_focused_and_gives_them_back() {
    let mut app = app();
    with_a_long_body(&mut app);
    let first = app.selected_item().map(|i| i.id).expect("an item");

    // `↵` opens it and focuses it: a panel you have to open and then reach for
    // is two gestures for one intention.
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.reading);
    assert_eq!(app.focus, Focus::Reader);

    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    ui::render_frame(&mut app, 140, 30, 0);
    assert_eq!(app.reader.at(first), 1, "the text moved");
    assert_eq!(
        app.selected_item().map(|i| i.id),
        Some(first),
        "and the cursor did not"
    );

    // One `esc` backs out of both, because the panel and the keys in it are
    // one thing. Two presses to leave one panel reads as the first one having
    // failed.
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(!app.reading, "the panel is closed");
    assert_eq!(app.focus, Focus::List, "and nothing is left focused on it");

    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_ne!(
        app.selected_item().map(|i| i.id),
        Some(first),
        "and the cursor moves again"
    );
}

/// Clicking a row while the panel is open says "look at this one" rather than
/// "I am done", so the panel stays and the keys go back to the list.
#[test]
fn the_pointer_can_leave_the_panel_open() {
    let mut app = app();
    with_a_long_body(&mut app);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.focus, Focus::Reader);

    ui::render_frame(&mut app, 140, 30, 0);
    let (area, _) = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, harrow::app::Hit::Row(_)))
        .expect("a row on screen");
    app.handle_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: area.x,
        row: area.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(app.focus, Focus::List, "the keys went back to the list");
    assert!(app.reading, "and the panel stayed open");
}

/// The offset belongs to the item, so walking the backlog with the panel open
/// starts each one at its top and returns you to where you had got to.
#[test]
fn the_panel_keeps_its_place_in_each_item_separately() {
    let mut app = app();
    with_a_long_body(&mut app);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    let first = app.selected_item().map(|i| i.id).expect("an item");

    app.run(Command::DetailDown);
    app.run(Command::DetailDown);
    ui::render_frame(&mut app, 140, 30, 0);
    assert_eq!(app.reader.at(first), 2);

    // Out of the panel, then along the list: the cursor moves, the panel
    // follows, and the place it was left at belongs to the item rather than to
    // the pane.
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    let second = app.selected_item().map(|i| i.id).expect("another item");
    assert_eq!(app.reader.at(second), 0, "a new item starts at its top");

    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(app.reader.at(first), 2, "and stepping back returns to it");
}

/// The keyboard's way into the pane without leaving the list, which is the
/// gesture for glancing at the end of an item rather than settling into it.
#[test]
fn the_detail_keys_scroll_the_panel_without_taking_the_focus() {
    let mut app = app();
    with_a_long_body(&mut app);
    let first = app.selected_item().map(|i| i.id).expect("an item");

    app.run(Command::DetailDown);
    ui::render_frame(&mut app, 140, 30, 0);
    assert_eq!(app.detail.at(first), 1, "the detail pane moved");
    assert_eq!(
        app.focus,
        Focus::List,
        "and the keys stayed where they were"
    );
}

/// A body long enough to need scrolling, so the frame it is read in is
/// smaller than the text in it.
fn with_a_long_body(app: &mut App) {
    let long = (1..=80)
        .map(|n| format!("Paragraph {n} of a proposal nobody will finish reading."))
        .collect::<Vec<_>>()
        .join("\n\n");
    for item in &mut app.items {
        item.body = long.clone();
    }
}

/// 0024 clamped every pane to the height of its own content, and the overlays
/// were not panes at the time. The reader kept counting: the text stopped
/// moving, the offset did not, and the body scrolled up out of its own frame
/// until nothing was left to read.
#[test]
fn a_reader_cannot_be_scrolled_past_the_end_of_the_item() {
    let mut app = app();
    with_a_long_body(&mut app);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.reading);

    // The draw is the only thing that knows how tall the body came out, so a
    // scroll is only as bounded as the frame that follows it.
    let id = app.selected_item().map(|i| i.id).expect("an item");
    for _ in 0..200 {
        app.run(Command::DetailDown);
    }
    ui::render_frame(&mut app, 110, 26, 0);
    let end = app.reader.at(id);
    assert!(end > 0, "a body this long has somewhere to scroll to");

    for _ in 0..200 {
        app.run(Command::DetailDown);
    }
    ui::render_frame(&mut app, 110, 26, 0);
    assert_eq!(
        app.reader.at(id),
        end,
        "the last line of the item is the end of the scroll"
    );
}

/// The other half of the same bound: an item that fits has nowhere to go, and
/// scrolling it was the quickest way to hide it completely.
#[test]
fn an_item_that_fits_its_frame_does_not_scroll_at_all() {
    let mut app = app();
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    let id = app.selected_item().map(|i| i.id).expect("an item");
    for _ in 0..40 {
        app.run(Command::DetailDown);
    }
    ui::render_frame(&mut app, 110, 26, 0);
    assert_eq!(app.reader.at(id), 0, "there was no more of it to show");
}

/// The history overlay counted forever for the same reason, and is held to the
/// same bound rather than to a second version of it.
#[test]
fn a_history_cannot_be_scrolled_past_its_last_line() {
    let mut app = app();
    let long = (1..=60)
        .map(|n| format!("2026-09-{:02}  somebody  touched it again", (n % 28) + 1))
        .collect::<Vec<_>>()
        .join("\n");
    app.show_history(3, Ok(long));

    for _ in 0..200 {
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    }
    ui::render_frame(&mut app, 110, 26, 0);
    let end = app.history.as_ref().map(|h| h.scroll);
    assert!(end.is_some_and(|s| s > 0), "a history this long scrolls");

    for _ in 0..200 {
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    }
    ui::render_frame(&mut app, 110, 26, 0);
    assert_eq!(
        app.history.as_ref().map(|h| h.scroll),
        end,
        "the last change recorded is the end of the scroll"
    );
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

/// harrow reports what it could not read. `cairn check` reports what the
/// project's own rules will not accept. They are different questions, and a
/// finding of cairn's read as a bug of harrow's is the confusion worth
/// avoiding.
#[test]
fn the_projects_own_check_is_asked_of_cairn_and_kept_apart() {
    let mut app = app();
    assert_eq!(app.run(Command::Check), Action::Check);
    assert!(app.diagnostics, "and it opens where the answer will appear");
    assert!(app.checked.is_none(), "nothing said until cairn answers");

    app.show_check(Ok("ok: 6 item(s), 0 warning(s)\n".into()));
    assert_eq!(
        app.checked,
        Some(Ok(vec!["ok: 6 item(s), 0 warning(s)".to_string()])),
        "including the line that says it passed — an empty section under a \
         heading is indistinguishable from a validator that never ran"
    );

    app.show_check(Err("no such file or directory".into()));
    assert!(matches!(app.checked, Some(Err(_))), "and a failure says so");
    assert!(
        app.warnings.is_empty(),
        "none of which became one of harrow's own warnings"
    );
}

/// cairn 0.2.0 enforces the permission model in the direction nobody expects:
/// arriving over the protocol is what makes a caller an agent, so a field
/// declared `propose` is refused there and allowed on the command line. harrow
/// is not an agent, so it *can* set such a field — which is right, since the
/// person at the terminal is who the proposal would have been addressed to.
/// What was missing is any sign that the project had an opinion.
#[test]
fn a_field_the_project_says_to_propose_opens_that_way() {
    let mut app = app();
    app.schema
        .fields
        .iter_mut()
        .find(|f| f.name == "priority")
        .expect("the fixture declares priority")
        .agent = harrow::schema::Agent::Propose;

    app.open_picker("priority");
    let picker = app.picker.as_ref().expect("a picker");
    assert!(picker.propose, "the project's opinion is the default");
    assert_eq!(picker.permission, harrow::schema::Agent::Propose);

    // And it is a default, not a restriction: the person at the terminal is
    // the one a proposal would have been addressed to.
    app.run(Command::Propose);
    assert!(!app.picker.as_ref().expect("still open").propose);
}

#[test]
fn proposing_asks_why_and_hands_it_to_cairn() {
    let mut app = app();
    app.open_picker("priority");
    app.run(Command::Propose);

    // Pick p0, then say why.
    app.handle_key(KeyCode::Char('1'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(
        app.editing.is_some(),
        "a proposal without a reason is a preference"
    );
    for c in "nobody can install this without it".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert_eq!(
        args(&app.handle_key(KeyCode::Enter, KeyModifiers::NONE)),
        vec![
            "propose".to_string(),
            "3".to_string(),
            "priority=p0".to_string(),
            "--why".to_string(),
            "nobody can install this without it".to_string(),
        ]
    );
}

#[test]
fn an_unproposed_picker_still_just_sets_the_field() {
    let mut app = app();
    app.open_picker("priority");
    assert!(!app.picker.as_ref().expect("open").propose);
    app.handle_key(KeyCode::Char('1'), KeyModifiers::NONE);
    let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(args(&action), vec!["set", "3", "priority=p0"]);
}

/// Every number on the stats pane is the answer to a question, and the
/// reader's next thought after each of them is *show me*.
#[test]
fn a_figure_opens_the_set_it_counted() {
    let mut app = app();
    app.pane = harrow::app::Pane::Stats;
    let _ = harrow::ui::render_frame(&mut app, 110, 30, 0);
    assert!(!app.doors.is_empty(), "the pane is made of figures");

    let ready = app
        .doors
        .iter()
        .position(|d| *d == harrow::app::Door::Filter("ready=true".into()))
        .expect("`N ready` is a figure");
    app.open_door(ready);

    assert_eq!(app.filter, "ready=true");
    assert_eq!(
        app.pane,
        harrow::app::Pane::List,
        "on the list, where you act"
    );
    assert!(
        app.rows.iter().all(|r| match r {
            harrow::app::Row::Item(i) => app.items[*i].ready(&app.schema),
            _ => true,
        }),
        "and showing exactly what it counted"
    );

    app.run(Command::Back);
    assert!(app.filter.is_empty(), "esc comes back out");
}

/// A figure that names one item goes to that item rather than to a set of one.
#[test]
fn a_figure_that_names_an_item_selects_it() {
    let mut app = app();
    app.pane = harrow::app::Pane::Stats;
    let _ = harrow::ui::render_frame(&mut app, 110, 30, 0);

    let (at, id) = app
        .doors
        .iter()
        .enumerate()
        .find_map(|(n, d)| match d {
            harrow::app::Door::Item(id) => Some((n, *id)),
            _ => None,
        })
        .expect("waiting longest, or what is in the way");
    app.open_door(at);

    assert_eq!(app.pane, harrow::app::Pane::List);
    assert_eq!(app.selected_item().map(|i| i.id), Some(id));
}

/// A door that leads nowhere is worse than a figure that plainly is not one.
#[test]
fn every_door_leads_somewhere_real() {
    let doors = {
        let mut app = app();
        app.pane = harrow::app::Pane::Stats;
        let _ = harrow::ui::render_frame(&mut app, 110, 30, 0);
        app.doors.clone()
    };
    assert!(doors.len() >= 6);

    for (n, door) in doors.iter().enumerate() {
        let mut fresh = app();
        fresh.pane = harrow::app::Pane::Stats;
        let _ = harrow::ui::render_frame(&mut fresh, 110, 30, 0);
        fresh.open_door(n);
        let app = fresh;
        match door {
            harrow::app::Door::Filter(_) => assert!(
                app.query.unknown.is_empty(),
                "{door:?} filters on something this project has not got: {:?}",
                app.query.unknown
            ),
            harrow::app::Door::Item(id) => assert!(
                app.items.iter().any(|i| i.id == *id),
                "{door:?} names no item"
            ),
        }
    }
}

/// Three lenses in a cycle means the cost of reaching one depends on where it
/// sits in an array. With a way back, every lens is one press from every
/// other — which is also why no direct key per lens was added.
#[test]
fn shift_tab_reaches_the_lens_before_this_one() {
    use harrow::app::Pane;
    let mut app = app();
    let first = Pane::ALL[0];
    let last = Pane::ALL[Pane::ALL.len() - 1];
    app.pane = first;

    app.run(Command::ViewBack);
    assert_eq!(app.pane, last, "back from the first is the last");
    app.run(Command::ViewBoard);
    assert_eq!(app.pane, first, "and forward undoes it");

    // Every lens is reachable from every other, which is the part that has
    // to stay true however many there are.
    for from in Pane::ALL {
        for to in Pane::ALL {
            app.pane = from;
            let mut presses = 0;
            while app.pane != to && presses < Pane::ALL.len() {
                app.run(Command::ViewBoard);
                presses += 1;
            }
            assert_eq!(app.pane, to, "{from:?} never reaches {to:?}");
        }
    }
}

/// 0062 recorded *every lens is one press from every other* as the reason
/// for adding no direct key per lens, and wrote a test so the reason would
/// fail rather than be quietly outgrown. Two more lenses outgrew it. This is
/// the rule that replaced it: positional, one key each, in the order the
/// tabs are in — so what you see is what you count.
#[test]
fn every_lens_has_a_direct_key_in_the_order_the_tabs_are_in() {
    use harrow::app::Pane;
    let mut app = app();
    for (n, lens) in Pane::ALL.iter().enumerate() {
        app.pane = Pane::ALL[Pane::ALL.len() - 1];
        app.run(Command::ViewLens(n as u8 + 1));
        assert_eq!(
            app.pane,
            *lens,
            "the {}th key is the {}th tab",
            n + 1,
            n + 1
        );
    }
}

/// A key for a lens that is not there does nothing rather than wrapping
/// round, because nine on a five-lens screen never meant anything.
#[test]
fn a_key_past_the_last_lens_does_nothing() {
    use harrow::app::Pane;
    let mut app = app();
    app.pane = Pane::List;
    app.run(Command::ViewLens(9));
    assert_eq!(app.pane, Pane::List);
}

/// Which is the first thing a lens owes the reader.
#[test]
fn the_selection_survives_going_back_as_well_as_forward() {
    let mut app = app();
    let was = app.selected_item().map(|i| i.id);
    assert!(was.is_some());
    for _ in 0..3 {
        app.run(Command::ViewBack);
    }
    assert_eq!(app.selected_item().map(|i| i.id), was);
}

/// Every answer is a key that already exists, on the item the question is
/// about — which is the point of the queue selecting that item.
#[test]
fn the_answer_to_a_question_is_the_key_that_already_did_it() {
    let mut app = app();
    app.pane = harrow::app::Pane::Needs;
    let at = app
        .questions
        .iter()
        .position(|q| matches!(q.asking, harrow::app::Asking::Proposal { .. }))
        .expect("the fixture has a proposal");
    app.select_question(at);
    let id = app.questions[at].id;

    assert_eq!(
        app.selected_item().map(|i| i.id),
        Some(id),
        "the cursor selects the item the question is about"
    );
    app.run(Command::Accept);
    assert!(app.confirm.is_some(), "and A accepts it, asking first");
}

/// Passing through a queue that cannot show your item must not lose it, the
/// way passing through the board does not lose an item it has no column for.
#[test]
fn going_through_the_queue_keeps_a_selection_it_cannot_show() {
    let mut app = app();
    let was = app.selected_item().map(|i| i.id);
    assert!(
        !app.questions.iter().any(|q| Some(q.id) == was),
        "0003 raises no question, which is what makes this the interesting case"
    );

    app.pane = harrow::app::Pane::Needs;
    app.select_id(was.expect("something selected"));
    assert_eq!(app.selected_item().map(|i| i.id), was, "held on the way in");

    app.pane = harrow::app::Pane::List;
    assert_eq!(app.selected_item().map(|i| i.id), was, "and on the way out");
}

/// Moving the cursor there is a choice, and takes over from what you brought.
#[test]
fn moving_in_the_queue_replaces_what_you_arrived_with() {
    let mut app = app();
    app.pane = harrow::app::Pane::Needs;
    app.select_id(3);
    app.move_by(1);
    assert_ne!(app.selected_item().map(|i| i.id), Some(3));
    assert_eq!(
        app.selected_item().map(|i| i.id),
        app.questions.get(app.question()).map(|q| q.id)
    );
}

/// The one judgement in cairn's agent loop that is explicitly a person's:
/// tick what is true, not what would let you close.
#[test]
fn ticking_a_criterion_goes_through_cairn() {
    let mut app = app();
    app.can_tick = true;
    press(&mut app, 't');
    let picker = app.picker.as_ref().expect("the criteria to choose from");
    assert!(picker.tick);
    assert_eq!(picker.options.len(), 2, "0003 has two criteria");
    assert_eq!(
        picker.selected, 1,
        "starting on the first that is not true yet"
    );

    // cairn numbers them from one, and takes the number.
    assert_eq!(
        args(&app.handle_key(KeyCode::Enter, KeyModifiers::NONE)),
        vec!["tick", "3", "2"]
    );
}

/// A key that offers something and then reports `unrecognized subcommand` is
/// worse than a key that is not offered.
#[test]
fn a_cairn_too_old_to_tick_says_so_rather_than_failing() {
    let mut app = app();
    app.can_tick = false;
    assert_eq!(press(&mut app, 't'), Action::None);
    assert!(app.picker.is_none(), "nothing is offered");
    let said = app
        .toast
        .as_ref()
        .map(|(m, _, _)| m.clone())
        .unwrap_or_default();
    assert!(said.contains("cairn tick"), "{said}");
}

#[test]
fn an_item_with_no_criteria_says_so() {
    let mut app = app();
    app.can_tick = true;
    // 0004 is work with a problem statement and nothing ticked out of it.
    app.select_id(4);
    assert_eq!(press(&mut app, 't'), Action::None);
    assert!(app.picker.is_none());
}

/// The log keeps what the repository said until the backlog changes, and then
/// asks again — from wherever it happens to be, rather than only on the way in.
#[test]
fn the_log_asks_again_after_a_re_read_rather_than_waiting_to_be_re_entered() {
    let mut app = testkit::app();
    app.pane = harrow::app::Pane::Log;

    // Sitting on it with nothing to show is a question, asked once.
    assert_eq!(app.pending(), Some(Action::Activity));
    assert_eq!(app.pending(), None, "and not asked again while it is out");

    app.show_activity(Ok(String::new()));
    assert_eq!(app.pending(), None, "answered, so nothing to ask");

    // A re-read throws away what the repository said — the watcher makes that
    // happen whenever anybody touches the backlog — and the pane must not sit
    // there saying "asking" at nobody.
    app.ingest(testkit::report());
    assert_eq!(
        app.pending(),
        Some(Action::Activity),
        "the log has to ask again on its own"
    );
}

#[test]
fn no_other_lens_asks_the_repository_for_anything() {
    for pane in harrow::app::Pane::ALL {
        if pane == harrow::app::Pane::Log {
            continue;
        }
        let mut app = testkit::app();
        app.pane = pane;
        assert_eq!(app.pending(), None, "{pane:?} asked for something");
    }
}
