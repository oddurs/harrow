//! What arrives while somebody is watching.
//!
//! An agent filing and working items in its own worktree, and a person
//! watching the backlog in another. These hold that what the agent files is
//! on screen as it is written, that the view goes to what changed when the
//! person has sat back to watch, and that it leaves their cursor alone while
//! they are the one moving it.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App};
use harrow::engine::Report;
use harrow::identity::Id;
use harrow::keys::Command;
use harrow::{testkit, ui};

/// The sample backlog, with 7 filed on an agent's branch.
fn with_a_filing(status: &str) -> Report {
    let mut report = testkit::report();
    let mut seven = testkit::item(7, "Write the changelog", status);
    seven.category = report.schema.category(status);
    seven.filed_on = Some("feat/changelog".into());
    report.filed.push(seven);
    report
}

/// Opened, and left alone for longer than it takes to count as watching.
fn watching() -> App {
    let mut app = testkit::app();
    app.select_id(3.into());
    app.touched = app.now.saturating_sub(App::IDLE);
    app
}

fn said(app: &App) -> String {
    app.toast
        .as_ref()
        .map(|(m, _, _)| m.clone())
        .unwrap_or_default()
}

fn selected(app: &App) -> Option<Id> {
    app.selected_item().map(|i| i.id)
}

#[test]
fn an_item_filed_on_another_branch_arrives_as_a_row_that_says_where_it_lives() {
    let mut app = testkit::app();
    app.ingest(with_a_filing("backlog"));
    let screen = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(
        screen
            .lines()
            .any(|l| l.contains("0007 Write the changelog") && l.contains("feat/changelog")),
        "no row for it, or the row does not say where it lives:\n{screen}"
    );
    assert!(
        said(&app).contains("0007 filed · feat/changelog"),
        "{}",
        said(&app)
    );
    assert!(app.is_recent(7.into()), "and it is marked as new");
}

#[test]
fn a_reader_who_is_watching_is_taken_to_what_arrived() {
    let mut app = watching();
    app.ingest(with_a_filing("doing"));
    assert_eq!(selected(&app), Some(7.into()));
    // A third of the way down rather than on the bottom edge, so what came
    // before it is still on screen to read it against.
    assert!(app.homing.is_some());
}

#[test]
fn a_change_in_this_checkout_is_followed_too() {
    let mut app = watching();
    let mut report = testkit::report();
    let five = report.items.iter_mut().find(|i| i.id == 5).expect("5");
    five.status = "doing".into();
    five.category = harrow::schema::Category::Active;
    app.ingest(report);
    assert_eq!(selected(&app), Some(5.into()));
}

#[test]
fn a_reader_who_is_driving_keeps_their_cursor() {
    let mut app = watching();
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    let before = selected(&app);
    app.ingest(with_a_filing("doing"));
    assert_eq!(
        selected(&app),
        before,
        "the cursor is theirs while they use it"
    );
    assert!(
        said(&app).contains("filed"),
        "and they are still told: {}",
        said(&app)
    );
}

#[test]
fn a_reader_in_the_middle_of_reading_is_left_where_they_are() {
    let mut app = watching();
    app.run(Command::Read);
    app.touched = app.now.saturating_sub(App::IDLE);
    app.ingest(with_a_filing("doing"));
    assert_eq!(selected(&app), Some(3.into()));
}

#[test]
fn nothing_arriving_moves_nothing() {
    let mut app = watching();
    app.ingest(testkit::report());
    assert_eq!(selected(&app), Some(3.into()));
}

#[test]
fn a_filing_elsewhere_cannot_be_changed_from_here() {
    let mut app = testkit::app();
    app.ingest(with_a_filing("backlog"));
    app.select_id(7.into());
    for command in [
        Command::Claim,
        Command::Close,
        Command::Status,
        Command::Note,
        Command::Tick,
        Command::Split,
    ] {
        app.toast = None;
        assert!(
            matches!(app.run(command), Action::None),
            "{command:?} wrote to an item this checkout cannot see"
        );
        assert!(app.picker.is_none(), "{command:?} opened a picker first");
        assert!(
            said(&app).contains("0007 is filed on feat/changelog"),
            "{command:?}: {}",
            said(&app)
        );
    }
    app.run(Command::ToggleGroup);
    assert!(app.marked.is_empty(), "nor marked for a change");
}

#[test]
fn filing_something_new_still_works_with_a_filing_selected() {
    let mut app = testkit::app();
    app.ingest(with_a_filing("backlog"));
    app.select_id(7.into());
    app.run(Command::New);
    assert!(
        app.editing.is_some(),
        "a new item belongs to nobody's branch"
    );
}

#[test]
fn the_detail_says_where_a_filing_lives() {
    let mut app = testkit::app();
    app.ingest(with_a_filing("backlog"));
    app.select_id(7.into());
    let screen = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(screen.contains("filed on feat/changelog"), "{screen}");
    assert!(screen.contains("not in this checkout yet"), "{screen}");
}

#[test]
fn the_board_follows_as_the_list_does() {
    let mut app = watching();
    app.pane = harrow::app::Pane::Board;
    app.rebuild();
    app.ingest(with_a_filing("doing"));
    assert_eq!(selected(&app), Some(7.into()));
    let screen = ui::render_to_string(&mut app, 110, 20, 0);
    assert!(screen.contains("Write the changelog"), "{screen}");
}
