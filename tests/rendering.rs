//! What the screen looks like.
//!
//! A TUI's interface *is* the product, so it is held to a recorded screen
//! rather than to a description of one. `HARROW_UPDATE_SNAPSHOTS=1 cargo test`
//! accepts a deliberate change; anything else that moves the layout fails here.

mod support;

use harrow::app::App;
use harrow::theme::{Source, Theme};
use harrow::ui;

#[test]
fn the_list() {
    let mut app = support::app();
    support::assert_snapshot("list", &ui::render_to_string(&mut app, 110, 26, 0));
}

#[test]
fn the_board() {
    let mut app = support::app();
    app.board = true;
    support::assert_snapshot("board", &ui::render_to_string(&mut app, 110, 20, 0));
}

#[test]
fn grouped_by_status() {
    let mut app = support::app();
    app.group_by = "status".into();
    app.rebuild();
    support::assert_snapshot("by-status", &ui::render_to_string(&mut app, 110, 26, 0));
}

#[test]
fn reading_an_item() {
    let mut app = support::app();
    app.select_id(3);
    app.reading = true;
    support::assert_snapshot("reader", &ui::render_to_string(&mut app, 110, 24, 0));
}

#[test]
fn the_help_overlay() {
    let mut app = support::app();
    app.help = true;
    support::assert_snapshot("help", &ui::render_to_string(&mut app, 110, 32, 0));
}

#[test]
fn the_status_picker() {
    let mut app = support::app();
    app.select_id(3);
    app.open_picker("status");
    support::assert_snapshot("picker", &ui::render_to_string(&mut app, 110, 20, 0));
}

#[test]
fn a_narrow_terminal_loses_detail_not_its_shape() {
    let mut app = support::app();
    support::assert_snapshot("narrow", &ui::render_to_string(&mut app, 54, 18, 0));
}

/// Colour is data, so it is asserted like data. The plain-text snapshots say
/// where everything is; this one says how it reads.
#[test]
fn the_colours_a_row_is_drawn_in() {
    let mut app = support::app();
    app.theme = Theme::resolve("night").expect("a built-in theme");
    assert_eq!(app.theme.source, Source::Builtin);
    support::assert_snapshot(
        "list-colours",
        &ui::render_styles_to_string(&mut app, 110, 26, 0),
    );
}

#[test]
fn with_no_colour_at_all_the_glyphs_still_carry_it() {
    let mut app = support::app();
    app.theme = Theme::mono();
    app.show_closed = true;
    app.rebuild();
    let text = ui::render_to_string(&mut app, 110, 26, 0);
    // Blocked, active and finished have to be distinguishable with the colour
    // taken away — which is what `mono`, NO_COLOR and colour blindness all need.
    for glyph in ["⊘", "◐", "✓", "○"] {
        assert!(text.contains(glyph), "mono lost {glyph}:\n{text}");
    }
}

#[test]
fn an_empty_project_is_not_an_empty_screen() {
    let mut app = App::new();
    let text = ui::render_to_string(&mut app, 90, 20, 0);
    assert!(text.contains("harrow"), "the identity survives");
    assert!(
        text.contains("Press n"),
        "and says what to do next:\n{text}"
    );
}
