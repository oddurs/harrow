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

/// A click as a terminal reports one: the button going down and coming up.
/// A list chooses on the second, the way a menu does, so a helper that sent
/// only the first was testing half a gesture no terminal ever sends.
fn click(app: &mut App, column: u16, row: u16) -> Action {
    let down = app.handle_mouse(at(MouseEventKind::Down(MouseButton::Left), column, row));
    let up = app.handle_mouse(at(MouseEventKind::Up(MouseButton::Left), column, row));
    if down == Action::None { up } else { down }
}

/// The button going down and staying down: the start of a drag, which is not
/// a click and must not end in one.
fn press(app: &mut App, column: u16, row: u16) -> Action {
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

/// A panel is not a modal. The list beside it goes on answering the pointer,
/// and what the panel shows follows what the click selected — which is the
/// whole reason for reading beside the backlog rather than over it.
#[test]
fn clicking_the_list_while_reading_moves_what_the_panel_shows() {
    let mut app = testkit::app();
    app.reading = true;
    let _ = ui::render_frame(&mut app, 140, 30, 0);

    // The lens keeps a column at this width, so its rows are still clickable.
    let (x, y) = find(&app, &Hit::Row(2));
    click(&mut app, x, y);
    let first = app.selected_item().map(|i| i.id);
    assert!(app.reading, "clicking the list does not dismiss the panel");

    let _ = ui::render_frame(&mut app, 140, 30, 0);
    let (x, y) = find(&app, &Hit::Row(3));
    click(&mut app, x, y);
    assert!(app.reading, "still reading");
    assert_ne!(
        app.selected_item().map(|i| i.id),
        first,
        "and the panel is showing the item the click selected"
    );
}

/// The wheel goes to the pane under the pointer, which is the rule everywhere
/// else on the screen. The panel used to swallow every scroll while it was up.
#[test]
fn the_wheel_scrolls_the_panel_or_the_list_by_where_it_is() {
    let mut app = testkit::app();
    app.select_id(3);
    app.reading = true;
    let long = (1..=80)
        .map(|n| format!("Paragraph {n} of a proposal nobody will finish reading."))
        .collect::<Vec<_>>()
        .join("\n\n");
    for item in &mut app.items {
        item.body = long.clone();
    }
    let _ = ui::render_frame(&mut app, 140, 30, 0);
    let id = app.selected_item().map(|i| i.id).expect("an item");

    let (rx, ry) = find(&app, &Hit::Reader);
    app.handle_mouse(at(MouseEventKind::ScrollDown, rx + 4, ry + 4));
    let _ = ui::render_frame(&mut app, 140, 30, 0);
    assert!(app.reader.at(id) > 0, "the pointer was over the panel");

    // Over the list instead: the panel stays where it was put.
    let moved = app.reader.at(id);
    let (lx, ly) = find(&app, &Hit::Row(2));
    app.handle_mouse(at(MouseEventKind::ScrollDown, lx, ly));
    let _ = ui::render_frame(&mut app, 140, 30, 0);
    assert_eq!(
        app.reader
            .at(app.selected_item().map(|i| i.id).expect("an item")),
        moved,
        "a scroll over the list is not a scroll of the panel"
    );
}

/// Every row in the panel is a checkbox, and clicking one is the whole
/// gesture — the vocabulary was the hard part, not the typing.
#[test]
fn clicking_a_value_in_the_filter_panel_ticks_it() {
    let mut app = testkit::app();
    app.filtering = true;
    app.rebuild();
    let _ = ui::render_frame(&mut app, 140, 28, 0);
    assert!(app.filter.is_empty(), "nothing filtered to begin with");

    let (x, y) = find(&app, &Hit::Facet(0));
    click(&mut app, x, y);
    assert!(
        !app.filter.is_empty(),
        "a click on a value is a filter, without typing any of it"
    );

    let _ = ui::render_frame(&mut app, 140, 28, 0);
    let (x, y) = find(&app, &Hit::Facet(0));
    click(&mut app, x, y);
    assert!(app.filter.is_empty(), "and clicking it again puts it back");
}

/// The history overlay is the same surface and gets the same answer.
#[test]
fn a_click_outside_the_history_closes_it() {
    let mut app = testkit::app();
    app.select_id(3);
    app.show_history(3, Ok("2026-09-02  somebody  created\n".into()));
    drawn(&mut app);
    find(&app, &Hit::Overlay);

    click(&mut app, 0, 0);
    assert!(app.history.is_none(), "a click behind it dismisses it");
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
    press(&mut app, x, y);
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
    app.readonly = Some(harrow::app::ReadOnly::NoCairn);
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Card(0, 0));
    press(&mut app, x, y);
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

/// The stats pane was the one part of harrow you could not touch: no cursor,
/// and no hit regions either. The mouse is not a second-class way to drive
/// this, so a figure is clickable and a click does what Enter does.
#[test]
fn a_figure_on_the_stats_pane_is_clickable() {
    let mut app = testkit::app();
    app.pane = Pane::Stats;
    let _ = ui::render_frame(&mut app, 110, 30, 0);

    let ready = app
        .doors
        .iter()
        .position(|d| *d == harrow::app::Door::Filter("ready=true".into()))
        .expect("`N ready` is a figure");
    let (x, y) = find(&app, &Hit::Figure(ready));
    click(&mut app, x, y);

    assert_eq!(app.filter, "ready=true", "the same door the key opens");
    assert_eq!(app.pane, Pane::List);
}

/// The criterion the item is filed under: a link shows its text, not its URL,
/// and opens when clicked.
#[test]
fn a_link_in_the_body_opens_when_it_is_clicked() {
    let mut app = testkit::app();
    app.select_id(5);
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Link(0));
    // What it says is the link text; the destination is not on the screen.
    let screen = ui::render_frame(&mut app, 110, 26, 0);
    let row: String = (0..110)
        .map(|c| screen[(c, y)].symbol().to_string())
        .collect();
    assert!(
        row.contains("report"),
        "the link text is not drawn: {row:?}"
    );
    assert!(
        !row.contains("example.org"),
        "the url is on screen: {row:?}"
    );

    assert_eq!(
        click(&mut app, x, y),
        Action::Open("https://example.org/report".into())
    );
}

/// A blocker is the one thing in the pane you always want to go to next.
#[test]
fn a_blocker_is_clickable() {
    let mut app = testkit::app();
    app.select_id(4);
    drawn(&mut app);

    let (x, y) = find(&app, &Hit::Link(0));
    click(&mut app, x, y);
    assert_eq!(app.selected_item().map(|i| i.id), Some(3));
}

/// Registered against where they were drawn, so scrolling moves the targets
/// with the text rather than leaving them behind.
#[test]
fn scrolling_the_detail_pane_moves_what_is_clickable() {
    let mut app = testkit::app();
    app.select_id(5);
    // Short enough that the pane holds more than it shows; a pane with
    // nothing to scroll would pass this without meaning anything. One row
    // taller than it used to be, because the view line now takes one.
    let _ = ui::render_frame(&mut app, 110, 15, 0);
    let before = find(&app, &Hit::Link(0));

    app.run(harrow::keys::Command::DetailDown);
    let _ = ui::render_frame(&mut app, 110, 15, 0);
    let after = find(&app, &Hit::Link(0));
    assert_eq!(after.0, before.0);
    assert_eq!(after.1 + 1, before.1);
}

/// Pressed third in a column of three and dragged over a column of one, the
/// cursor stays on a card that exists.
#[test]
fn dragging_over_a_shorter_column_keeps_the_cursor_on_a_card() {
    let mut app = testkit::app();
    app.pane = harrow::app::Pane::Board;
    app.rebuild();
    drawn(&mut app);
    let (tall, short) = {
        let by_len = |want: usize| app.columns.iter().position(|c| c.items.len() == want);
        (
            by_len(3).expect("a column of three"),
            by_len(1).expect("a column of one"),
        )
    };
    let (x, y) = find(&app, &Hit::Card(tall, 2));
    press(&mut app, x, y);
    let (tx, ty) = find(&app, &Hit::Column(short));
    app.handle_mouse(at(MouseEventKind::Drag(MouseButton::Left), tx, ty));
    assert_eq!(app.column, short);
    assert!(
        app.check_invariants().is_ok(),
        "{:?}",
        app.check_invariants()
    );
    assert!(
        app.selected_item().is_some(),
        "the detail pane has a card to show"
    );
}
