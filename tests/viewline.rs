//! The view line: what is on screen, and why.
//!
//! Three things decide what a lens is showing — which items, in what order,
//! grouped how. Two of them used to be nowhere on screen, and the one that
//! was said it in a pane title that only the list had. A view whose rules are
//! off screen is a view nobody can check, which is how the strip and the list
//! came to disagree twice about what counted as work.

use harrow::app::{App, Pane};
use harrow::{testkit, ui};

/// The toolbar is the second row: the status counts joined it in 0089, so
/// the chrome is two rows and a rule rather than three and a rule.
fn line(app: &mut App) -> String {
    ui::render_to_string(app, 110, 26, 0)
        .lines()
        .nth(1)
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[test]
fn the_filter_the_sort_and_the_grouping_are_all_stated() {
    let mut app = testkit::app();
    let said = line(&mut app);
    assert!(said.contains("filter"), "no filter segment: {said}");
    assert!(said.contains("status"), "no sort segment: {said}");
    assert!(said.contains("milestone"), "no grouping segment: {said}");
}

/// The case the line exists for. A filter that arrived from `--filter`, from
/// a saved view or from a click on the status strip was never typed anywhere,
/// so the rows were narrowed and the screen did not say why.
#[test]
fn a_filter_nobody_typed_is_still_stated() {
    let mut app = testkit::app();
    app.filter = "priority=p1".into();
    app.ingest(testkit::report());
    let said = line(&mut app);
    assert!(said.contains("priority=p1"), "{said}");
    assert!(said.contains(" of "), "no tally beside it: {said}");
}

/// The name is what you chose; the grammar is what you got.
#[test]
fn a_saved_view_is_named_and_spelled_out() {
    let mut app = testkit::app();
    app.view = Some("now".into());
    app.filter = "category=active".into();
    app.ingest(testkit::report());
    let said = line(&mut app);
    assert!(said.contains("now"), "the name is missing: {said}");
    assert!(
        said.contains("category=active"),
        "the grammar is missing: {said}"
    );
}

/// It belongs to all five lenses: the arrangement is the only thing that
/// differs between them, and what is being shown is not the arrangement.
#[test]
fn every_lens_says_what_it_is_showing() {
    for pane in [Pane::Needs, Pane::List, Pane::Board, Pane::Stats, Pane::Log] {
        let mut app = testkit::app();
        app.pane = pane;
        app.filter = "priority=p1".into();
        app.ingest(testkit::report());
        assert!(
            line(&mut app).contains("priority=p1"),
            "{pane:?} does not say what it is showing"
        );
    }
}

/// `a` changes which items are there, so it is part of the same sentence.
#[test]
fn showing_everything_is_part_of_the_view() {
    let mut app = testkit::app();
    assert!(!line(&mut app).contains("finished"), "{}", line(&mut app));
    app.show_all = true;
    app.rebuild();
    assert!(line(&mut app).contains("finished"), "{}", line(&mut app));
}

/// A backlog narrowed by hand should be something you can hand to somebody
/// else, or paste into a script.
#[test]
fn the_view_is_a_command_you_can_paste() {
    let mut app = testkit::app();
    assert_eq!(app.command_line(), "harrow");

    app.filter = "priority=p1,status=backlog".into();
    app.ingest(testkit::report());
    assert_eq!(app.command_line(), "harrow -f priority=p1,status=backlog");

    app.sort = "-priority".into();
    app.group_by = "status".into();
    app.show_all = true;
    app.ingest(testkit::report());
    assert_eq!(
        app.command_line(),
        "harrow -a -f priority=p1,status=backlog --sort -priority --group-by status"
    );

    // A view somebody named is the thing they would rather paste.
    let mut named = testkit::app();
    named.view = Some("now".into());
    named.ingest(testkit::report());
    assert_eq!(named.command_line(), "harrow --view now");
}

/// A filter with a space or a quote in it has to survive the round trip.
#[test]
fn a_filter_that_needs_quoting_gets_it() {
    let mut app = testkit::app();
    app.filter = "title~two words".into();
    app.ingest(testkit::report());
    assert_eq!(app.command_line(), "harrow -f 'title~two words'");
}

/// The strip is dropped on a short terminal and so is this, one row sooner:
/// a backlog you cannot see is worse than a view you cannot read the rules of.
#[test]
fn a_short_terminal_keeps_the_backlog_instead() {
    let mut app = testkit::app();
    app.filter = "priority=p1".into();
    app.ingest(testkit::report());
    // The header carries the filter too; what must be gone is the row.
    let cramped = ui::render_to_string(&mut app, 110, 11, 0);
    let second = cramped.lines().nth(1).unwrap_or_default();
    assert!(
        second.trim_start().starts_with('─'),
        "the line is still taking a row it cannot spare:\n{cramped}"
    );
}
