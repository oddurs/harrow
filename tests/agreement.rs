//! harrow and cairn, asked the same question.
//!
//! harrow reads a `[[view]]` filter out of `cairn.toml` verbatim and cairn
//! owns that file, so harrow's filter grammar is not its own: it must
//! evaluate everything cairn accepts. A grammar narrower than cairn's does
//! not fail loudly, it returns a different answer — which is how `label=x`
//! came to select nothing here and twenty-six items there.
//!
//! So the test is about *agreement* rather than about either tool being
//! right in isolation. The same fixture, the same filter strings, the same
//! ids out of both.
//!
//! `#[ignore]`d because it needs cairn on PATH, and a test that silently
//! stops running is worse than no test. Run it with:
//!
//!     cargo test --test agreement -- --ignored

use std::path::Path;

use harrow::app::App;
use harrow::engine::{Project, Source};
use harrow::testkit;

/// Every spelling worth holding the two tools to, including the ones that
/// broke: cairn's aliases, `=` against a list, alternatives, presence, and
/// a negation.
const FILTERS: &[&str] = &[
    "labels=chrome",
    "labels~chrome",
    "label=chrome",
    "label~chrome",
    "type=bug",
    "kind=bug",
    "status=doing",
    "category=active",
    "priority=p0",
    "priority=p0|p1",
    "area=ui",
    "milestone=v0.1",
    "milestone=",
    "assignee=oddur",
    "assignee!=oddur",
    "priority!=p3",
    "blocked=true",
    "id=3",
];

fn cairn_ids(dir: &Path, filter: &str) -> Vec<u32> {
    let out = std::process::Command::new("cairn")
        .args([
            "-C",
            &dir.display().to_string(),
            "list",
            "--all",
            "-f",
            filter,
        ])
        .output()
        .expect("cairn runs");
    assert!(
        out.status.success(),
        "cairn refused {filter:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut ids: Vec<u32> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter_map(|w| w.parse().ok())
        .collect();
    ids.sort_unstable();
    ids
}

fn open(dir: &Path, filter: &str) -> App {
    let mut project = Project::discover(dir).expect("the project opens");
    let mut app = App::new();
    // `--all`, to match what cairn was asked. The list's rule about hiding
    // finished work is harrow's own and not the filter's; comparing through
    // it would be comparing two different questions.
    app.show_all = true;
    // Ungrouped, because a grouping turns containers into headings rather
    // than rows — a milestone is not a row whatever the grouping is, which
    // is deliberate and is about presentation. Comparing through it would
    // report harrow and cairn as disagreeing about `milestone=` when what
    // they disagree about is where a milestone is drawn.
    app.group_by = "none".to_string();
    app.filter = filter.to_string();
    // Ingest parses the filter and rebuilds, which is the path a reader
    // takes — so this measures what harrow actually shows, not what its
    // predicate would say in isolation.
    app.ingest(project.load().expect("it loads"));
    app
}

fn harrow_ids(dir: &Path, filter: &str) -> Vec<u32> {
    let app = open(dir, filter);
    assert!(
        app.filter_problem().is_none(),
        "harrow refused {filter:?}: {:?}",
        app.filter_problem()
    );
    let mut ids: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            harrow::app::Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    ids.sort_unstable();
    ids
}

#[test]
#[ignore = "needs cairn on PATH"]
fn harrow_and_cairn_select_the_same_items() {
    let dir = testkit::project();
    let mut disagreed = Vec::new();
    for filter in FILTERS {
        let (theirs, ours) = (
            cairn_ids(dir.path(), filter),
            harrow_ids(dir.path(), filter),
        );
        if theirs != ours {
            disagreed.push(format!("{filter:?}: cairn {theirs:?}, harrow {ours:?}"));
        }
    }
    assert!(
        disagreed.is_empty(),
        "{} of {} filters disagree:\n{}",
        disagreed.len(),
        FILTERS.len(),
        disagreed.join("\n")
    );
}

/// The other half of the contract: what cairn will not evaluate, harrow
/// should not silently evaluate either.
#[test]
#[ignore = "needs cairn on PATH"]
fn a_field_neither_tool_declares_is_refused_here() {
    let dir = testkit::project();
    let app = open(dir.path(), "nonsense=x");
    assert!(
        app.filter_problem().is_some(),
        "an unknown field must not read as an empty result"
    );
    // cairn returns nothing rather than refusing, which is the difference
    // this item decided to keep: cairn is strict in `check`, harrow has no
    // check time and so is strict at query time.
    assert!(cairn_ids(dir.path(), "nonsense=x").is_empty());
}
