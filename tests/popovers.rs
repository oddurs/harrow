//! Anything that opens over the screen takes the pointer the way a GUI menu
//! does.
//!
//! None of them did. An open dropdown ignored every click that was not on one
//! of its own rows — and did not ignore it quietly: the click went straight
//! through to whatever was underneath. Clicking beside the grouping dropdown
//! selected the row behind it; clicking a tab changed the lens under it while
//! the dropdown went on floating on top; clicking its own segment again opened
//! it again. The only way out was a key.
//!
//! What a pointer is owed, for every popover there is:

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{App, Hit, Pane};
use harrow::keys::Command;
use harrow::{testkit, ui};

const SIZE: (u16, u16) = (110, 26);

/// One event, then a frame: the map of what is where is rebuilt as it is
/// drawn, which is what a real terminal does between two reports.
fn send(app: &mut App, kind: MouseEventKind, (column, row): (u16, u16)) {
    app.handle_mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    });
    ui::render_frame(app, SIZE.0, SIZE.1, 0);
}

fn click(app: &mut App, at: (u16, u16)) {
    send(app, MouseEventKind::Down(MouseButton::Left), at);
    send(app, MouseEventKind::Up(MouseButton::Left), at);
}

fn find(app: &App, want: &Hit) -> (u16, u16) {
    (0..SIZE.1)
        .flat_map(|y| (0..SIZE.0).map(move |x| (x, y)))
        .find(|&(x, y)| app.hit_at(x, y) == Some(want))
        .unwrap_or_else(|| panic!("{want:?} is not on screen"))
}

/// A point nothing opens over at this size: well down the list, left edge.
const BEHIND: (u16, u16) = (5, 7);

fn with_grouping_open() -> App {
    let mut app = testkit::app();
    ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
    let segment = find(&app, &Hit::Run(Command::GroupBy));
    click(&mut app, segment);
    assert!(app.dropdown.is_some(), "a click on the segment opens it");
    app
}

#[test]
fn a_click_outside_a_dropdown_closes_it_and_goes_no_further() {
    let mut app = with_grouping_open();
    let (selected, pane) = (app.selected, app.pane);
    assert!(
        matches!(app.hit_at(BEHIND.0, BEHIND.1), Some(Hit::Row(_))),
        "a row is behind it"
    );

    click(&mut app, BEHIND);
    assert!(app.dropdown.is_none(), "it closes");
    assert_eq!(
        app.selected, selected,
        "and the row behind it is not selected"
    );

    let mut app = with_grouping_open();
    let at = find(&app, &Hit::Tab(Pane::Board));
    click(&mut app, at);
    assert!(app.dropdown.is_none());
    assert_eq!(app.pane, pane, "nor the lens behind it changed");
}

/// A menu bar: its own button shuts it, a neighbour's opens the neighbour.
#[test]
fn a_segment_toggles_its_own_dropdown_and_switches_to_another() {
    let mut app = with_grouping_open();
    let at = find(&app, &Hit::Run(Command::GroupBy));
    click(&mut app, at);
    assert!(app.dropdown.is_none(), "its own segment shuts it");

    let mut app = with_grouping_open();
    let at = find(&app, &Hit::Run(Command::Sort));
    click(&mut app, at);
    assert_eq!(
        app.dropdown.as_ref().map(|d| d.of),
        Some(Command::Sort),
        "one click, not two"
    );
}

fn option(app: &App, value: &str) -> usize {
    app.dropdown
        .as_ref()
        .expect("open")
        .options
        .iter()
        .position(|(v, _, _)| v == value)
        .unwrap_or_else(|| panic!("no {value}"))
}

/// The button coming up is the choice, the way it is in a menu — so a press
/// can still change its mind.
#[test]
fn an_option_is_chosen_when_the_button_comes_up_on_it() {
    let mut app = with_grouping_open();
    let status = find(&app, &Hit::Choose(option(&app, "status")));
    click(&mut app, status);
    assert_eq!(app.group_by, "status");
    assert!(app.dropdown.is_none());

    // Pressed on one, dragged to another: the second.
    let mut app = with_grouping_open();
    let from = find(&app, &Hit::Choose(option(&app, "status")));
    let to = find(&app, &Hit::Choose(option(&app, "priority")));
    send(&mut app, MouseEventKind::Down(MouseButton::Left), from);
    assert!(app.dropdown.is_some(), "pressing does not choose");
    send(&mut app, MouseEventKind::Drag(MouseButton::Left), to);
    send(&mut app, MouseEventKind::Up(MouseButton::Left), to);
    assert_eq!(app.group_by, "priority");

    // Pressed on one, dragged off the list: nothing.
    let mut app = with_grouping_open();
    let from = find(&app, &Hit::Choose(option(&app, "status")));
    send(&mut app, MouseEventKind::Down(MouseButton::Left), from);
    send(&mut app, MouseEventKind::Drag(MouseButton::Left), BEHIND);
    send(&mut app, MouseEventKind::Up(MouseButton::Left), BEHIND);
    assert_eq!(app.group_by, "milestone", "nothing was chosen");
}

/// Pressing on the segment and dragging down into the list it opens, then
/// letting go on an option: one gesture, the one a native menu answers.
#[test]
fn press_on_the_segment_drag_into_the_list_and_let_go() {
    let mut app = testkit::app();
    ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
    let segment = find(&app, &Hit::Run(Command::GroupBy));
    send(&mut app, MouseEventKind::Down(MouseButton::Left), segment);
    let to = find(&app, &Hit::Choose(option(&app, "type")));
    send(&mut app, MouseEventKind::Drag(MouseButton::Left), to);
    send(&mut app, MouseEventKind::Up(MouseButton::Left), to);
    assert_eq!(app.group_by, "type");
    assert!(app.dropdown.is_none());
}

/// Its own border and padding are part of it, not holes onto what it covers.
#[test]
fn a_click_on_the_popover_itself_is_not_a_click_outside() {
    let mut app = with_grouping_open();
    let popover = app.popover.expect("it says where it is");
    click(&mut app, (popover.x, popover.y));
    assert!(app.dropdown.is_some(), "the corner of its frame");
}

#[test]
fn the_wheel_moves_the_list_it_is_over_and_nothing_behind() {
    let mut app = with_grouping_open();
    let over = find(&app, &Hit::Choose(0));
    let before = app.dropdown.as_ref().unwrap().selected;
    send(&mut app, MouseEventKind::ScrollDown, over);
    assert_eq!(app.dropdown.as_ref().unwrap().selected, before + 1);

    let selected = app.selected;
    send(&mut app, MouseEventKind::ScrollDown, BEHIND);
    assert_eq!(app.selected, selected, "the list behind stays put");
    assert!(app.dropdown.is_some(), "and the wheel is not a click");
}

/// How a case opens its popover, and how it tells the popover is still up.
type Opens = fn(&mut App);
type IsOpen = fn(&App) -> bool;

/// Every popover, by the same rule.
#[test]
fn every_popover_closes_on_a_click_outside() {
    let cases: [(&str, Opens, IsOpen); 5] = [
        (
            "the milestone picker",
            |a| {
                a.run(Command::Milestone);
            },
            |a| a.picker.is_some(),
        ),
        (
            "the palette",
            |a| {
                a.run(Command::Palette);
            },
            |a| a.palette.is_some(),
        ),
        (
            "help",
            |a| {
                a.run(Command::Help);
            },
            |a| a.help,
        ),
        (
            "diagnostics",
            |a| {
                a.run(Command::Diagnostics);
            },
            |a| a.diagnostics,
        ),
        (
            "a confirmation",
            |a| {
                a.run(Command::Close);
            },
            |a| a.confirm.is_some(),
        ),
    ];
    for (name, open, is_open) in cases {
        let mut app = testkit::app();
        ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
        open(&mut app);
        ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
        assert!(is_open(&app), "{name} opens");
        let popover = app.popover.expect("claims its ground");
        // The screen's far corner, outside anything centred on it.
        let outside = (0, SIZE.1 - 2);
        assert!(!(popover.x..popover.x + popover.width).contains(&outside.0));
        click(&mut app, outside);
        assert!(!is_open(&app), "{name} closes on a click outside it");
    }
}

/// Answered no, as any key but `y` answers it — the stray click is the
/// pointer's stray key, and no is the answer that changes nothing.
#[test]
fn a_click_outside_a_confirmation_answers_it_no() {
    let mut app = testkit::app();
    ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
    let id = app.selected_item().map(|i| i.id).expect("an item");
    app.run(Command::Close);
    ui::render_frame(&mut app, SIZE.0, SIZE.1, 0);
    let action = {
        app.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: SIZE.1 - 2,
            modifiers: KeyModifiers::NONE,
        })
    };
    assert!(app.confirm.is_none());
    assert_eq!(
        action,
        harrow::app::Action::None,
        "no `cairn close` for {id}"
    );
}

#[test]
fn home_and_end_reach_the_ends_of_an_open_list() {
    use crossterm::event::KeyCode;
    let mut app = with_grouping_open();
    let last = app.dropdown.as_ref().unwrap().options.len() - 1;
    app.handle_key(KeyCode::End, KeyModifiers::NONE);
    assert_eq!(app.dropdown.as_ref().unwrap().selected, last);
    app.handle_key(KeyCode::Home, KeyModifiers::NONE);
    assert_eq!(app.dropdown.as_ref().unwrap().selected, 0);
}
