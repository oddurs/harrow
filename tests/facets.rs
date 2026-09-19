//! One query, two ways to type it.
//!
//! `/` takes cairn's grammar typed; `f` ticks values built from the schema.
//! They write the same string, and the panel is the narrower of the two by
//! construction: it offers the values a field declares, so it can say
//! `priority=p0` and cannot say `created<2026-09-04`.
//!
//! Two controls that edit one value and disagree about it is the shape of the
//! bug 0080 cost two releases to find. These hold the agreement.

use harrow::app::App;
use harrow::filter::Query;
use harrow::{testkit, ui};

/// Put the facet cursor on one value, wherever the panel happens to put it.
fn point_at(app: &mut App, field: &str, value: &str) {
    let mut n = 0;
    for facet in &app.facets {
        for v in &facet.values {
            if facet.field == field && v.value == value {
                app.facet = n;
                return;
            }
            n += 1;
        }
    }
    panic!("the panel offers no {field}={value}");
}

fn with_filter(filter: &str) -> App {
    let mut app = testkit::app();
    app.filter = filter.to_string();
    app.ingest(testkit::report());
    app.filtering = true;
    app.rebuild();
    app
}

#[test]
fn ticking_writes_one_clause_and_unticking_removes_that_one() {
    let mut app = with_filter("created<2026-09-04,priority!=p3");
    point_at(&mut app, "priority", "p1");

    app.toggle_facet();
    assert_eq!(app.filter, "created<2026-09-04,priority!=p3,priority=p1");

    app.toggle_facet();
    assert_eq!(app.filter, "created<2026-09-04,priority!=p3");
}

/// A bound the panel cannot express has to survive being ticked around it,
/// or the two controls are quietly overwriting each other.
#[test]
fn a_clause_the_panel_cannot_express_survives() {
    let mut app = with_filter("created<2026-09-04");
    for value in ["p1", "p2", "p1"] {
        point_at(&mut app, "priority", value);
        app.toggle_facet();
        assert!(
            app.filter.contains("created<2026-09-04"),
            "the bound was lost: {}",
            app.filter
        );
    }
}

/// Opening the panel on a hand-typed filter has to show what it understands
/// as already chosen, or the ticks are a second copy of the query rather than
/// a rendering of it.
#[test]
fn a_typed_clause_opens_already_ticked() {
    let app = with_filter("priority=p1");
    let priority = app
        .facets
        .iter()
        .find(|f| f.field == "priority")
        .expect("the project declares a priority");
    let ticked: Vec<&str> = priority
        .values
        .iter()
        .filter(|v| v.ticked)
        .map(|v| v.value.as_str())
        .collect();
    assert_eq!(ticked, vec!["p1"]);
}

/// The panel narrows every count on it by a clause it cannot draw, so it has
/// to say that something is in force rather than look like the whole story.
#[test]
fn the_panel_says_when_something_it_cannot_draw_is_narrowing_it() {
    let mut app = with_filter("created<2026-09-04");
    assert_eq!(app.unmanaged_clauses(), 1);
    let screen = ui::render_to_string(&mut app, 120, 26, 0);
    assert!(screen.contains("1 typed"), "{screen}");

    let mut plain = with_filter("priority=p1");
    assert_eq!(plain.unmanaged_clauses(), 0);
    let screen = ui::render_to_string(&mut plain, 120, 26, 0);
    assert!(!screen.contains("typed"), "{screen}");
}

/// What lets the view line state a query nobody typed, and what lets a
/// narrowed backlog be handed over as a command line.
#[test]
fn a_query_renders_back_to_grammar_that_reparses_to_itself() {
    let schema = testkit::schema();
    for source in [
        "priority=p1",
        "created<2026-09-04,priority!=p3",
        "priority=p0|p1,area~ui",
        "oauth",
        "",
    ] {
        let once = Query::parse(source, &schema);
        let twice = Query::parse(&once.source(), &schema);
        assert_eq!(
            once.source(),
            twice.source(),
            "{source:?} does not survive the round trip"
        );
    }
}
