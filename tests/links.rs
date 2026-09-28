//! Every link in the detail pane is reachable without the mouse: `]` and `[`
//! pick one, `↵` follows it the way a click does, `esc` lets it go.

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{App, Hit};
use harrow::keys::Command;
use harrow::{testkit, ui};

/// 0005's pane: Waiting on 0004, then Builds on 0001, 0002 and 0003.
fn five() -> App {
    let mut app = testkit::concluded();
    app.select_id(5.into());
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    app
}

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(code, KeyModifiers::NONE);
    let _ = ui::render_frame(app, 110, 30, 0);
}

fn selected(app: &App) -> Option<harrow::identity::Id> {
    app.selected_item().map(|i| i.id)
}

#[test]
fn a_link_is_picked_named_and_followed_from_the_keyboard() {
    let mut app = five();
    key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.picked_link(), Some(0));
    key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.picked_link(), Some(1));
    let said = app.toast.clone().expect("a toast").0;
    assert!(said.starts_with("0001 Pick the file format"), "{said}");
    key(&mut app, KeyCode::Enter);
    assert_eq!(
        selected(&app),
        Some(1.into()),
        "followed, finished work and all"
    );
    assert!(
        !app.reading,
        "↵ followed the link rather than opening the reader"
    );
}

/// The same follow a click does, to the same place.
#[test]
fn following_by_key_and_by_click_reach_the_same_item() {
    let mut clicked = five();
    let area = clicked
        .hits
        .iter()
        .find(|(_, hit)| *hit == Hit::Link(2))
        .map(|(area, _)| *area)
        .expect("link 2 is on screen");
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        clicked.handle_mouse(MouseEvent {
            kind,
            column: area.x,
            row: area.y,
            modifiers: KeyModifiers::NONE,
        });
    }

    let mut keyed = five();
    for _ in 0..3 {
        key(&mut keyed, KeyCode::Char(']'));
    }
    key(&mut keyed, KeyCode::Enter);
    assert_eq!(selected(&keyed), selected(&clicked));
    assert_eq!(selected(&keyed), Some(2.into()));
}

#[test]
fn backwards_from_nothing_is_the_last_and_it_wraps() {
    let mut app = five();
    key(&mut app, KeyCode::Char('['));
    assert_eq!(app.picked_link(), Some(3));
    key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.picked_link(), Some(0), "wraps round");
}

/// `esc` lets it go, and `↵` is the reader again.
#[test]
fn esc_lets_go_and_enter_reads_again() {
    let mut app = five();
    key(&mut app, KeyCode::Char(']'));
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.picked_link(), None);
    key(&mut app, KeyCode::Enter);
    assert!(app.reading);
}

/// Picked on one item's pane, it is let go by moving to another.
#[test]
fn moving_on_lets_go() {
    let mut app = five();
    key(&mut app, KeyCode::Char(']'));
    app.select_id(4.into());
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    assert_eq!(app.picked_link(), None);
}

/// Drawn picked, so the eye finds what `↵` will follow.
#[test]
fn the_picked_link_is_drawn_picked() {
    let mut app = five();
    key(&mut app, KeyCode::Char(']'));
    let area = app
        .hits
        .iter()
        .find(|(_, hit)| *hit == Hit::Link(0))
        .map(|(area, _)| *area)
        .expect("link 0 is on screen");
    let buffer = ui::render_frame(&mut app, 110, 30, 0);
    let cell = &buffer[(area.x, area.y)];
    assert!(
        cell.modifier.contains(ratatui::style::Modifier::REVERSED),
        "{:?} at {area:?}",
        cell.symbol()
    );
}

#[test]
fn a_pane_with_no_links_says_so() {
    // 0003 builds on nothing and links nowhere.
    let mut app = testkit::concluded();
    app.show_all = true;
    app.rebuild();
    app.select_id(3.into());
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    assert!(
        app.links.is_empty(),
        "the fixture has changed: {:?}",
        app.links
    );
    key(&mut app, KeyCode::Char(']'));
    assert_eq!(app.picked_link(), None);
    assert!(
        app.toast
            .clone()
            .unwrap()
            .0
            .contains("nothing in this pane")
    );
}

#[test]
fn the_keys_are_in_the_help_on_one_row() {
    let app = testkit::app();
    let row = app
        .keymap
        .help_rows()
        .into_iter()
        .find(|(_, d)| *d == Command::NextLink.describe())
        .expect("in the help");
    assert!(row.0.contains('[') && row.0.contains(']'), "{row:?}");
}
