//! What an ordinary listing leaves out, and how you ask for it back.
//!
//! Two things are absent unless asked for: work that is finished or dropped,
//! and containers. Naming the field they hang off is how you ask — and
//! *naming* means saying something about that field, with any operator.
//!
//! Harrow used to read that as *asking for one value of it*, so `status=done`
//! brought finished work back and `category!=dropped` did not. cairn reads it
//! the wider way, which meant the two disagreed by every closed item on a
//! filter shape a project reaches for constantly — this repository's own
//! `[render] include` is `category!=dropped`. 0106.
//!
//! The cross-tool proof is in `agreement.rs` and needs cairn on PATH. This
//! holds the same rule without it, so an ordinary checkout catches a change.

use harrow::app::{App, Row};
use harrow::engine::{Project, Source};
use harrow::testkit;

/// The fixture has a milestone (1), one finished item (2), and four open
/// ones (3–6).
fn listed(filter: &str) -> Vec<u32> {
    let dir = testkit::project();
    let mut project = Project::discover(dir.path()).expect("the project opens");
    let mut app = App::new();
    // Ungrouped: a grouping draws a container as a heading rather than a row,
    // which is presentation and not what this is about.
    app.group_by = "none".to_string();
    app.filter = filter.to_string();
    app.ingest(project.load().expect("it loads"));
    assert!(
        app.filter_problem().is_none(),
        "harrow refused {filter:?}: {:?}",
        app.filter_problem()
    );
    let mut ids: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    ids.sort_unstable();
    ids
}

#[test]
fn an_ordinary_listing_leaves_out_finished_work_and_containers() {
    assert_eq!(listed(""), vec![3, 4, 5, 6]);
}

#[test]
fn an_equality_on_status_or_category_asks_for_finished_work() {
    assert_eq!(listed("status=done"), vec![2]);
    assert_eq!(listed("category=done"), vec![2]);
}

/// The bug. A negation says as much about the field as an equality does.
#[test]
fn a_negation_on_status_or_category_asks_for_it_too() {
    assert_eq!(listed("status!=dropped"), vec![2, 3, 4, 5, 6]);
    assert_eq!(listed("category!=dropped"), vec![2, 3, 4, 5, 6]);
}

/// And a negation that happens to exclude the finished work still lifts the
/// default — the predicate does the excluding, not a hidden rule behind it.
#[test]
fn a_negation_that_excludes_finished_work_reads_the_same_either_way() {
    assert_eq!(listed("status!=done"), vec![3, 4, 5, 6]);
}

#[test]
fn naming_the_type_asks_for_containers_with_either_operator() {
    assert_eq!(listed("type=milestone"), vec![1]);
    assert_eq!(listed("type!=feature"), vec![1, 5, 6]);
}

/// A container that is finished is behind both defaults, so it takes both
/// questions to reach it.
#[test]
fn a_finished_container_needs_the_type_and_the_status() {
    assert!(!listed("type=milestone").contains(&2));
    let both = listed("type=milestone,category!=dropped");
    assert!(both.contains(&1), "{both:?}");
}
