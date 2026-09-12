//! The interface, driven by the pointer.
//!
//! Every one of these renders a frame first: the map of what is clickable is
//! built as the screen is drawn, so a click on a region nothing drew is a click
//! on nothing. That is the property worth testing — not that a handler exists,
//! but that what you can see is what you can hit.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{Action, App, Hit, Pane};
use harrow::testkit;
use harrow::ui;

fn at(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn click(app: &mut App, column: u16, row: u16) -> Action {
    app.handle_mouse(at(MouseEventKind::Down(MouseButton::Left), column, row))
}

/// Where something was drawn this frame.
fn find(app: &App, want: &Hit) -> (u16, u16) {
    let (area, _) = app
        .hits
        .iter()
        .find(|(_, hit)| hit == want)
        .unwrap_or_else(|| panic!("nothing on screen for {want:?}"));
    (area.x, area.y)
}

fn drawn(app: &mut App) -> (u16, u16) {
    let _ = ui::render_frame(app, 110, 26, 0);
    (110, 26)
}

#[test]
fn the_tabs_are_buttons() {
    let mut app = testkit::app();
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Tab(Pane::Stats));
    click(&mut app, x, y);
    assert_eq!(app.pane, Pane::Stats);

    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Tab(Pane::Board));
    click(&mut app, x, y);
    assert_eq!(app.pane, Pane::Board);
}

#[test]
fn a_status_in_the_strip_filters_by_itself_and_clicking_again_clears_it() {
    let mut app = testkit::app();
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Status("doing".into()));
    click(&mut app, x, y);
    assert_eq!(app.filter, "status=doing");
    let ids: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            harrow::app::Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec![3], "only what is in progress");

    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Status("doing".into()));
    click(&mut app, x, y);
    assert!(app.filter.is_empty(), "clicking it again puts it back");
}

#[test]
fn clicking_a_row_selects_it_and_clicking_twice_reads_it() {
    let mut app = testkit::app();
    drawn(&mut app);

    // The third row of the list: a group heading, then items.
    let (x, y) = find(&app, &Hit::Row(2));
    click(&mut app, x, y);
    assert_eq!(app.selected, 2);
    assert!(!app.reading, "one click is a selection");

    click(&mut app, x, y);
    assert!(app.reading, "two is a read");
}

#[test]
fn clicking_a_group_heading_folds_it() {
    let mut app = testkit::app();
    drawn(&mut app);
    let before = app.rows.len();

    let (x, y) = find(&app, &Hit::Row(0));
    click(&mut app, x, y);
    assert!(app.rows.len() < before, "the group should have folded");

    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Row(0));
    click(&mut app, x, y);
    assert_eq!(app.rows.len(), before, "and unfolded");
}

#[test]
fn a_card_dragged_to_another_column_changes_its_status() {
    let mut app = testkit::app();
    app.pane = Pane::Board;
    drawn(&mut app);

    // The first card of the backlog column, dropped on `done`.
    let (x, y) = find(&app, &Hit::Card(0, 0));
    let id = app.columns[0]
        .items
        .first()
        .map(|i| app.items[*i].id)
        .expect("a card to drag");
    click(&mut app, x, y);
    assert!(app.dragging.is_some(), "the drag has to start");

    let (tx, ty) = find(&app, &Hit::Column(2));
    app.handle_mouse(at(MouseEventKind::Drag(MouseButton::Left), tx, ty));
    let action = app.handle_mouse(at(MouseEventKind::Up(MouseButton::Left), tx, ty));

    match action {
        Action::Write(change) => assert_eq!(
            change.args,
            vec!["set".to_string(), id.to_string(), "status=done".to_string()]
        ),
        other => panic!("expected a status change, got {other:?}"),
    }
    assert!(app.dragging.is_none(), "and end");
}

#[test]
fn a_card_dropped_where_it_started_changes_nothing() {
    let mut app = testkit::app();
    app.pane = Pane::Board;
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Card(0, 0));
    click(&mut app, x, y);
    let action = app.handle_mouse(at(MouseEventKind::Up(MouseButton::Left), x, y));
    assert_eq!(action, Action::None);
}

#[test]
fn a_read_only_backlog_refuses_a_drag_rather_than_appearing_to_work() {
    let mut app = testkit::app();
    app.pane = Pane::Board;
    app.writable = false;
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Card(0, 0));
    click(&mut app, x, y);
    let (tx, ty) = find(&app, &Hit::Column(2));
    let action = app.handle_mouse(at(MouseEventKind::Up(MouseButton::Left), tx, ty));
    assert_eq!(action, Action::None);
    let (message, _, _) = app.toast.as_ref().expect("and it has to say why");
    assert!(message.contains("cairn"), "{message}");
}

#[test]
fn the_picker_and_the_confirmation_are_clickable() {
    let mut app = testkit::app();
    app.select_id(3);
    app.open_picker("status");
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Option(0));
    match click(&mut app, x, y) {
        Action::Write(change) => assert_eq!(change.args[2], "status=backlog"),
        other => panic!("expected a status change, got {other:?}"),
    }

    app.select_id(3);
    app.ask_close();
    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Answer(false));
    assert_eq!(click(&mut app, x, y), Action::None);
    assert!(app.confirm.is_none(), "no is an answer");

    app.ask_close();
    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Answer(true));
    match click(&mut app, x, y) {
        Action::Write(change) => assert_eq!(change.args[0], "close"),
        other => panic!("expected a close, got {other:?}"),
    }
}

#[test]
fn a_footer_hint_is_a_button() {
    let mut app = testkit::app();
    drawn(&mut app);
    let (x, y) = find(&app, &Hit::Run(harrow::keys::Command::Claim));
    match click(&mut app, x, y) {
        Action::Write(change) => assert_eq!(change.args[0], "claim"),
        other => panic!("expected a claim, got {other:?}"),
    }
}

/// Scrolling a list and watching the selection run away from the pointer is the
/// thing that makes a terminal interface feel unlike everything else on screen.
#[test]
fn the_wheel_moves_the_view_and_takes_the_cursor_only_when_it_must() {
    let mut app = testkit::app();
    app.show_all = true;
    app.group_by = "none".into();
    app.rebuild();
    // A backlog taller than the pane, so there is something to scroll.
    let _ = ui::render_frame(&mut app, 80, 9, 0);
    assert!(app.rows.len() > 4, "the fixture has to overflow the pane");

    app.selected = 0;
    app.handle_mouse(at(MouseEventKind::ScrollDown, 10, 5));
    assert!(app.offset > 0, "the view has to have moved");
    assert!(
        app.selected >= app.offset,
        "and the cursor came along rather than scrolling off"
    );

    app.handle_mouse(at(MouseEventKind::ScrollUp, 10, 5));
    app.handle_mouse(at(MouseEventKind::ScrollUp, 10, 5));
    assert_eq!(app.offset, 0, "and back to the top");
}

#[test]
fn a_click_on_nothing_does_nothing() {
    let mut app = testkit::app();
    drawn(&mut app);
    for (x, y) in [(0, 0), (109, 25), (60, 2)] {
        assert_eq!(click(&mut app, x, y), Action::None);
        assert!(app.check_invariants().is_ok());
    }
}

#[test]
fn an_overlay_takes_the_click_rather_than_the_list_behind_it() {
    let mut app = testkit::app();
    app.select_id(3);
    app.open_picker("priority");
    drawn(&mut app);

    // A point inside the picker that also sits over a list row.
    let (x, y) = find(&app, &Hit::Option(1));
    assert!(matches!(app.hit_at(x, y), Some(Hit::Option(1))));
}

/// The reason the panes are beside each other: one can be read without
/// disturbing the other.
#[test]
fn the_wheel_moves_the_pane_under_the_pointer_rather_than_the_one_with_the_cursor() {
    let mut app = testkit::app();
    app.select_id(3);
    drawn(&mut app);

    let (offset, selected) = (app.offset, app.selected);
    let (x, y) = find(&app, &Hit::Detail);
    app.handle_mouse(at(MouseEventKind::ScrollDown, x + 2, y + 2));

    assert!(app.detail.at(3) > 0, "the detail pane has to have moved");
    assert_eq!(app.offset, offset, "and the list stayed where it was");
    assert_eq!(app.selected, selected, "along with the cursor in it");
}

#[test]
fn a_board_column_scrolls_where_it_sits_without_moving_the_selection() {
    let mut app = testkit::app();
    app.pane = Pane::Board;
    // Everything, so that some column has more cards than a short board shows.
    app.show_all = true;
    app.rebuild();

    let (tallest, cards) = app
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.items.len()))
        .max_by_key(|(_, n)| *n)
        .expect("the fixture deals a board");
    assert!(cards >= 2, "the fixture has to overflow a column");

    // Two borders and the chrome above and below, less one row than the column
    // needs: the height at which that column cannot show its last card.
    let _ = ui::render_frame(&mut app, 110, cards as u16 + 4, 0);

    // The cursor somewhere else, so anything that moved it would be visible.
    let elsewhere = (0..app.columns.len())
        .find(|i| *i != tallest && !app.columns[*i].items.is_empty())
        .expect("a second column with something in it");
    app.column = elsewhere;
    app.column_row = 0;
    let was = app.selected_item().map(|i| i.id);

    let (x, y) = find(&app, &Hit::Column(tallest));
    app.handle_mouse(at(MouseEventKind::ScrollDown, x + 1, y + 1));

    assert!(
        app.column_offsets[tallest] > 0,
        "the column under the pointer has to have moved"
    );
    assert_eq!(app.column, elsewhere, "and the cursor stayed in its own");
    assert_eq!(app.selected_item().map(|i| i.id), was);
}
