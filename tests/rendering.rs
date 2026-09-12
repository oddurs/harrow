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
    app.pane = harrow::app::Pane::Board;
    support::assert_snapshot("board", &ui::render_to_string(&mut app, 110, 20, 0));
}

/// The detail belongs to the selection, not to the list. A board wide enough
/// to spare the width knows as much about the selected card as the list does.
#[test]
fn a_board_wide_enough_for_the_detail() {
    let mut app = support::app();
    app.pane = harrow::app::Pane::Board;
    app.select_id(3);
    support::assert_snapshot("board-detail", &ui::render_to_string(&mut app, 140, 22, 0));
}

#[test]
fn the_statistics() {
    let mut app = support::app();
    app.pane = harrow::app::Pane::Stats;
    support::assert_snapshot("stats", &ui::render_to_string(&mut app, 110, 26, 0));
}

#[test]
fn the_statistics_in_one_column() {
    let mut app = support::app();
    app.pane = harrow::app::Pane::Stats;
    support::assert_snapshot("stats-narrow", &ui::render_to_string(&mut app, 62, 26, 0));
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

/// The end of a long item, beside the list rather than in an overlay over it.
#[test]
fn the_detail_pane_scrolled_into_a_long_item() {
    let mut app = support::app();
    app.select_id(3);
    app.detail.by(3, 6);
    support::assert_snapshot(
        "detail-scrolled",
        &ui::render_to_string(&mut app, 110, 22, 0),
    );
}

/// A close reads as a movement between two states rather than a deletion: the
/// row is still there, drawn as what it has become, with the mark that says it
/// just moved. A moment later it goes.
#[test]
fn an_item_settling_after_it_was_closed() {
    let mut app = support::app();
    let mut report = harrow::testkit::report();
    for item in &mut report.items {
        if item.id == 3 {
            item.status = "done".into();
            item.category = harrow::schema::Category::Done;
        }
    }
    app.ingest(report);
    support::assert_snapshot("settling", &ui::render_to_string(&mut app, 110, 20, 0));
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

/// The shape this is most often used in: a pane beside the work, with an agent
/// changing the backlog in the other one.
#[test]
fn a_narrow_pane() {
    let mut app = support::app();
    support::assert_snapshot("narrow", &ui::render_to_string(&mut app, 62, 22, 0));
}

#[test]
fn a_pane_too_narrow_for_words() {
    let mut app = support::app();
    support::assert_snapshot("cramped", &ui::render_to_string(&mut app, 40, 14, 0));
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
    app.show_all = true;
    app.rebuild();
    let text = ui::render_to_string(&mut app, 110, 26, 0);
    // Blocked, active and finished have to be distinguishable with the colour
    // taken away — which is what `mono`, NO_COLOR and colour blindness all need.
    for glyph in ["⊘", "◐", "✓", "○"] {
        assert!(text.contains(glyph), "mono lost {glyph}:\n{text}");
    }
}

/// What "the highlight is janky" was. Reverse video swaps the foreground and
/// background of every cell in the row, so the selected row becomes a bright
/// bar with the page colour punched through it — and every colour the row was
/// carrying, the status glyph included, turns into the background.
#[test]
fn the_selected_row_is_a_lift_of_the_page_rather_than_an_inversion() {
    use ratatui::style::Modifier;

    let mut app = support::app();
    app.theme = Theme::resolve("ghostty:gotham")
        .or_else(|_| Theme::resolve("night"))
        .expect("a derived theme");
    app.select_id(3);

    let buffer = ui::render_frame(&mut app, 100, 24, 0);
    // The list pane only. The detail pane carries the same title, and finding
    // that one instead is how this test first went wrong.
    let list_width = app.list_area.width;
    let row = (0..buffer.area.height)
        .find(|y| {
            (0..list_width)
                .map(|x| buffer[(x, *y)].symbol().to_string())
                .collect::<String>()
                .contains("Draw the list")
        })
        .expect("the selected item is on screen");

    let cell = &buffer[(4, row)];
    assert_eq!(
        cell.style().bg,
        Some(app.theme.selection),
        "the selected row has to carry the selection colour"
    );
    assert!(
        !cell.style().add_modifier.contains(Modifier::REVERSED),
        "and must not be drawn by inverting the page"
    );
    assert_ne!(
        cell.style().bg,
        Some(app.theme.background),
        "and has to differ from an unselected one"
    );
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

#[test]
fn the_history_of_one_item() {
    let mut app = support::app();
    app.select_id(3);
    app.show_history(
        3,
        Ok("2026-09-02  Oddur Sigurdsson  created\n\
            2026-09-03  Oddur Sigurdsson  status backlog -> doing\n\
            2026-09-03  an agent          priority p2 -> p1\n"
            .to_string()),
    );
    support::assert_snapshot("history", &ui::render_to_string(&mut app, 100, 20, 0));
}

/// A project that is not in git has no history rather than an empty one, and
/// saying which is the whole difference between "nothing happened" and "this
/// cannot be answered here".
#[test]
fn a_project_without_a_repository_says_so() {
    let mut app = support::app();
    app.select_id(3);
    app.show_history(3, Err("not a git repository".into()));
    let text = ui::render_to_string(&mut app, 100, 20, 0);
    assert!(text.contains("not a git repository"), "{text}");
    assert!(text.contains("the repository's"), "and why: {text}");
}

#[test]
fn an_item_with_a_proposal_is_visible_as_such() {
    let mut app = support::app();
    app.select_id(6);
    let text = ui::render_to_string(&mut app, 100, 24, 0);
    assert!(text.contains(" ?"), "the row says so:\n{text}");
    assert!(
        text.contains("1 needs you"),
        "and the header counts what is waiting on a person, of which this is one"
    );
    assert!(text.contains("Proposed"), "and the detail pane shows what");
    assert!(text.contains("p3 → p0"), "{text}");
}

/// Everything addressed to a person, ranked by what is waiting.
#[test]
fn what_needs_you() {
    let mut app = support::app();
    app.pane = harrow::app::Pane::Needs;
    support::assert_snapshot("needs", &ui::render_to_string(&mut app, 110, 22, 0));
}

/// The best screen this program can show, and until now unreachable.
#[test]
fn nothing_needs_you() {
    let mut app = support::app();
    app.pane = harrow::app::Pane::Needs;
    app.questions.clear();
    support::assert_snapshot("needs-empty", &ui::render_to_string(&mut app, 110, 14, 0));
}
