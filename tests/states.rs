//! Three states of a piece of work, and the one group that shows its finished.
//!
//! Done, in progress, and waiting on something are different things, and the
//! only thing telling them apart was a glyph and a hue. And a group with no
//! open work left showed a heading claiming a hundred per cent with nothing
//! under it — the claim the rows were not allowed to support.

use harrow::app::{App, Asking, Row};
use harrow::{testkit, ui};

fn listed(app: &App) -> Vec<u32> {
    app.rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect()
}

/// The sample's v0.1 has open work; `finished` has a milestone whose work is
/// all closed, which is the case this exists for.
#[test]
fn a_milestone_with_nothing_left_shows_the_work_that_closed_it() {
    let app = testkit::finished();
    let shown = listed(&app);
    assert!(
        !shown.is_empty(),
        "still nothing under a finished milestone"
    );
    for id in &shown {
        let item = app.items.iter().find(|i| i.id == *id).unwrap();
        assert!(
            item.category.is_closed(),
            "{id} is open, so this is not the finished case"
        );
    }
}

/// Closed work stays hidden wherever there is open work to compare it
/// against, which is everywhere else.
#[test]
fn a_group_with_open_work_still_hides_its_finished() {
    let app = testkit::app();
    for id in listed(&app) {
        let item = app.items.iter().find(|i| i.id == id).unwrap();
        assert!(!item.category.is_closed(), "{id} should still be hidden");
    }
}

/// Both read the same condition, so the two cannot disagree about what
/// *finished* means.
#[test]
fn it_is_the_same_condition_the_needs_queue_asks_about() {
    let app = testkit::finished();
    let asked: Vec<u32> = app
        .questions
        .iter()
        .filter(|q| q.asking == Asking::NothingUnfinished)
        .map(|q| q.id)
        .collect();
    assert!(!asked.is_empty(), "the queue has nothing to say");

    // Every revealed item belongs to a milestone the queue is asking about.
    for id in listed(&app) {
        let item = app.items.iter().find(|i| i.id == id).unwrap();
        let milestone = item.milestone().unwrap_or_default();
        let container = app.item_named(milestone).expect("a milestone");
        assert!(
            asked.contains(&container.id),
            "{id} was revealed under a milestone the queue is silent about"
        );
    }
}

/// The empty state and the rows read one rule, so the screen and its
/// explanation of itself cannot disagree.
#[test]
fn the_revealed_work_is_not_also_counted_as_hidden() {
    let app = testkit::finished();
    assert_eq!(
        app.hidden().closed,
        0,
        "counted as withheld while being shown"
    );
}

/// Work being done is the only thing on a backlog that is happening, and it
/// should be the only thing moving.
#[test]
fn what_is_in_progress_turns() {
    let mut app = testkit::app();
    let seen: Vec<char> = (0..8)
        .map(|tick| {
            ui::render_to_string(&mut app, 80, 20, tick)
                .lines()
                .find(|l| l.contains("Draw the list"))
                .and_then(|l| l.trim_start().chars().nth(3))
                .expect("the in-progress row")
        })
        .collect();
    let mut unique = seen.clone();
    unique.sort_unstable();
    unique.dedup();
    assert!(unique.len() >= 3, "it is not turning: {seen:?}");
    // A recorded screen is taken at tick 0, and has to stay reproducible.
    assert_eq!(seen[0], '◐');
}

/// Waiting is not work you can pick up, and a blocked item should not turn.
#[test]
fn what_is_waiting_does_not_turn() {
    let app = testkit::app();
    let blocked = app
        .items
        .iter()
        .find(|i| i.blocked && !i.category.is_closed())
        .expect("the fixture has a blocked item");
    for tick in 0..8 {
        assert_eq!(ui::turning(blocked, tick), "⊘");
    }
}

/// Told apart with the colour taken away, for `mono`, for a colourblind
/// reader, and for anybody who does not know the glyphs yet.
#[test]
fn the_three_states_differ_without_colour() {
    let mut app = testkit::app();
    app.theme = harrow::theme::Theme::mono();
    app.show_all = true;
    app.rebuild();
    let styles = ui::render_styles_to_string(&mut app, 110, 26, 0);
    // `mono` leaves every colour at Reset, so anything that still differs is
    // differing by weight — which is the point.
    assert!(styles.contains("legend"), "{styles}");

    let text = ui::render_to_string(&mut app, 110, 26, 0);
    for mark in ["✓", "◐", "⊘", "○"] {
        assert!(text.contains(mark), "mono lost {mark}");
    }
}
