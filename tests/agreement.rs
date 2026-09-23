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
//! stops running is worse than no test. CI explicitly runs them against a
//! pinned Cairn; local runs need its binary and a project checkout:
//!
//!     CAIRN_PROJECT_DIR=../cairn cargo test --test agreement -- --ignored

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
    "id>2",
    "id>=10",
    "category=done|dropped",
    "leaf=true",
    "contains=0002",
    "descendants>0",
    "contains=",
    "priority!=p0|p1",
    "status==doing",
    "labels~",
    "labels!~",
    "created_by=",
    "depth=0",
    "criteria>0",
    "criteria_met=true",
    "ready=true",
    "closed_at>=2026-09-01",
    "closed_at<2026-09-10",
    "closed_at=",
    "updated>=2026-09-10",
];

fn cairn_ids(dir: &Path, filter: &str) -> Vec<u32> {
    let out = std::process::Command::new("cairn")
        .args([
            "-C",
            &dir.display().to_string(),
            "list",
            "--all",
            "--ids",
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
        .map(|line| line.parse().expect("cairn --ids emits an identifier"))
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
    std::fs::write(dir.path().join("items/0007-completed.md"), "---\nid: 7\ntitle: Completed and edited later\ntype: feature\nstatus: done\nclosed_at: 2026-09-01\nupdated: 2026-09-20\n---\n\n- [x] Finished\n").unwrap();
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

#[test]
#[ignore = "needs cairn on PATH and CAIRN_PROJECT_DIR"]
fn the_cairn_projects_saved_views_agree_through_machine_output() {
    let config = tempfile::NamedTempFile::new().unwrap();
    let dir = std::env::var_os("CAIRN_PROJECT_DIR")
        .expect("set CAIRN_PROJECT_DIR to the pinned Cairn checkout; never skip this gate");
    let mut project = Project::discover(Path::new(&dir)).expect("Cairn project exists");
    let report = project.load().unwrap();
    assert!(
        report.schema.views.len() >= 8,
        "expected Cairn's configured project views"
    );
    for view in &report.schema.views {
        let cairn = std::process::Command::new("cairn")
            .arg("-C")
            .arg(&dir)
            .args(["list", "--view", &view.name, "--ids"])
            .output()
            .expect("cairn runs");
        let harrow = std::process::Command::new(env!("CARGO_BIN_EXE_harrow"))
            .arg("-C")
            .arg(&dir)
            .arg("--config")
            .arg(config.path())
            .args(["--view", &view.name, "--plain", "--group-by", "none"])
            .output()
            .expect("harrow runs");
        assert!(
            cairn.status.success(),
            "{}: {}",
            view.name,
            String::from_utf8_lossy(&cairn.stderr)
        );
        assert!(
            harrow.status.success(),
            "{}: {}",
            view.name,
            String::from_utf8_lossy(&harrow.stderr)
        );
        let mut theirs: Vec<String> = String::from_utf8_lossy(&cairn.stdout)
            .lines()
            .map(str::to_owned)
            .collect();
        let mut ours: Vec<String> = String::from_utf8_lossy(&harrow.stdout)
            .lines()
            .map(|line| line.split('\t').next().unwrap().to_owned())
            .collect();
        theirs.sort();
        ours.sort();
        assert_eq!(ours, theirs, "view {}", view.name);
    }
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
