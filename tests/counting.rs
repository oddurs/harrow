//! What counts as work, counted the same way everywhere.
//!
//! A backlog has two kinds of thing in it: work, and the things work is filed
//! under. Every part of the interface has to draw the line in the same place,
//! and for a while they did not — the strip counted seven items in a project
//! whose list said there was nothing open, because a milestone was work to one
//! of them and not to the other. Two true-looking numbers that cannot both be
//! right is worse than either being wrong, because there is nothing on screen
//! to say which to believe.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use harrow::app::{App, Hit, Pane, Row};
use harrow::schema::Schema;
use harrow::{testkit, ui};

/// Click where the strip drew a status, the way a reader would. Going through
/// the hit map rather than calling the handler keeps the test honest about the
/// number being on screen in the first place.
fn click_the_strip(app: &mut App, status: &str) {
    let _ = ui::render_frame(app, 110, 26, 0);
    let (area, _) = app
        .hits
        .iter()
        .find(|(_, hit)| matches!(hit, Hit::Status(name) if name == status))
        .unwrap_or_else(|| panic!("the strip drew no cell for {status}"));
    let (column, row) = (area.x, area.y);
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    });
}

/// How many item rows the list is showing, headings excluded.
fn listed(app: &App) -> usize {
    app.rows
        .iter()
        .filter(|r| matches!(r, Row::Item(_)))
        .count()
}

/// The promise the strip makes, in its own words: *each count is exactly what
/// clicking it would give you*. It is the only reading that makes the numbers
/// useful for choosing, and it is the one that broke.
#[test]
fn every_number_in_the_strip_is_what_clicking_it_gives_you() {
    for (which, mut app) in [
        ("the sample project", testkit::app()),
        ("a finished format 2 project", testkit::finished()),
    ] {
        let counts: Vec<(String, usize)> = app
            .status_counts()
            .into_iter()
            .map(|(s, n)| (s.name.clone(), n))
            .collect();
        assert!(!counts.is_empty(), "{which}: the strip is empty");

        for (status, count) in counts {
            app.filter = String::new();
            app.rebuild();
            click_the_strip(&mut app, &status);
            assert_eq!(
                listed(&app),
                count,
                "{which}: the strip says {count} {status}, clicking it lists {}",
                listed(&app)
            );
        }
    }
}

/// The bug as it was reported: seven in the strip, nothing in the list.
#[test]
fn a_project_whose_work_is_finished_says_so_in_both_places() {
    let app = testkit::finished();
    // Every row is finished work, revealed because its milestone has nothing
    // left (0088). What matters here is that none of it is *open*, and that
    // the strip says the same.
    for row in &app.rows {
        if let Row::Item(i) = row {
            let item = &app.items[*i];
            assert!(item.category.is_closed(), "{} is open", item.id);
        }
    }
    let open: usize = app
        .status_counts()
        .into_iter()
        .filter(|(s, _)| !s.category.is_closed())
        .map(|(_, n)| n)
        .sum();
    assert_eq!(open, 0, "the strip is still counting the milestones");
}

/// A milestone is not a card either, and it is not a row in the type tally.
/// Both read the same flag, so both went wrong together.
#[test]
fn a_container_is_not_dealt_onto_the_board_and_is_not_work_in_the_statistics() {
    let mut app = testkit::finished();
    app.pane = Pane::Board;
    app.rebuild();
    for column in &app.columns {
        for i in &column.items {
            assert!(
                !app.items[*i].container,
                "{} is on the board",
                app.items[*i].title
            );
        }
    }
    assert!(
        !app.stats()
            .by_type
            .iter()
            .any(|(kind, _)| kind == "milestone"),
        "the statistics count milestones as work: {:?}",
        app.stats().by_type
    );
}

/// Format 2 has no `groups` key on a type, so what a container is has to come
/// from the field that targets it. Reading the type first answered "not a
/// container" for every format 2 project — which is most of them — and the
/// derived rule was never reached at all.
#[test]
fn a_format_2_project_still_knows_what_a_container_is() {
    let two = Schema::parse(testkit::FORMAT_2_TOML, std::path::PathBuf::from("/tmp/p"))
        .expect("format 2 parses");
    assert_eq!(two.format, 2);
    assert!(two.is_container("milestone"), "derived from the ref field");
    assert!(!two.is_container("feature"));

    // And from format 3 on, the type says so itself: a declared type without
    // `groups` is work, whatever any field points at.
    let three = Schema::parse(testkit::CAIRN_TOML, std::path::PathBuf::from("/tmp/p"))
        .expect("format 3 parses");
    assert_eq!(three.format, 3);
    assert!(three.is_container("milestone"));
    assert!(!three.is_container("feature"));
}

/// "Nothing open here" was true about the rows and false about the project,
/// and the reader had no way to tell which. cairn reached the same conclusion
/// about its own listing: *"the filter is not what was wrong, so saying the
/// filter found nothing sent people to rewrite it."*
#[test]
fn the_empty_state_says_what_is_being_left_out() {
    let mut app = testkit::finished();
    // Grouped by status rather than by milestone: a status group is not a
    // container, so the reveal in 0088 does not apply and the finished work
    // stays withheld — which is the case this message exists for.
    app.group_by = "status".into();
    app.rebuild();
    let hidden = app.hidden();
    assert_eq!(hidden.containers, 2);
    assert_eq!(hidden.closed, 3);
    assert_eq!(hidden.kind.as_deref(), Some("milestone"));

    let screen = ui::render_to_string(&mut app, 110, 26, 0);
    assert!(
        screen.contains("2 milestones still open"),
        "the open milestones are not named:\n{screen}"
    );
    assert!(
        !screen.contains("Nothing open here."),
        "still claiming there is nothing:\n{screen}"
    );

    // And once they are on screen there is nothing left to explain.
    app.show_all = true;
    app.rebuild();
    assert_eq!(app.hidden(), Default::default());
}

/// A milestone with every item under it finished, still open. cairn reports
/// this and refuses to act on it — `later` can be complete and meant to stay
/// open for good — so harrow reports it in the queue it keeps for what is
/// waiting on a person, and the closing stays a key somebody presses.
#[test]
fn a_finished_milestone_that_is_still_open_is_a_question_not_an_action() {
    use harrow::app::Asking;

    let app = testkit::finished();
    let asked: Vec<u32> = app
        .questions
        .iter()
        .filter(|q| q.asking == Asking::NothingUnfinished)
        .map(|q| q.id)
        .collect();
    assert_eq!(asked, vec![1, 2], "both milestones are complete and open");
    assert_eq!(
        Asking::NothingUnfinished.answers(),
        "x when it has shipped",
        "the key is offered, never taken"
    );

    // Nothing filed is vacuously complete: an empty milestone in a young
    // project must not be told it has finished the day it was written.
    let mut empty = testkit::finished();
    empty.items.retain(|i| i.container);
    empty.rebuild();
    assert!(
        !empty
            .questions
            .iter()
            .any(|q| q.asking == Asking::NothingUnfinished),
        "a milestone with nothing under it is not finished"
    );
}

/// cairn's rule, in cairn's words: *"`--all` means all: closed work and
/// containers alike. Without it, naming a type is how you ask for them, which
/// is the same rule closed items follow for status."*
#[test]
fn asking_for_something_by_name_brings_it_back() {
    let closed = |app: &mut App, filter: &str| {
        app.filter = filter.to_string();
        app.ingest(testkit::finished_report());
        listed(app)
    };
    let mut app = testkit::finished();
    assert_eq!(closed(&mut app, "status=done"), 3, "asked for by status");
    assert_eq!(
        closed(&mut app, "category=done"),
        3,
        "asked for by category"
    );
    assert_eq!(closed(&mut app, "type=milestone"), 2, "asked for by type");
    // Not zero: both milestones here have nothing left, so their finished
    // work is revealed under them (0088). What is withheld is work in a
    // group that still has something open, and there is none of that here.
    assert_eq!(closed(&mut app, ""), 3, "the finished milestones' work");
}
