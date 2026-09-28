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

fn cairn_ids(dir: &Path, filter: &str) -> Vec<harrow::identity::Id> {
    cairn_ids_with(dir, filter, true)
}

fn cairn_ids_with(dir: &Path, filter: &str, all: bool) -> Vec<harrow::identity::Id> {
    let root = dir.display().to_string();
    let mut args: Vec<&str> = vec!["-C", &root, "list", "--json", "-f", filter];
    if all {
        args.insert(3, "--all");
    }
    let out = std::process::Command::new("cairn")
        .args(&args)
        .output()
        .expect("cairn runs");
    assert!(
        out.status.success(),
        "cairn refused {filter:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let items: serde_json::Value = serde_json::from_slice(&out.stdout).expect("item JSON");
    let mut ids: Vec<harrow::identity::Id> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| serde_json::from_value(item["id"].clone()).expect("full identity"))
        .collect();
    ids.sort_unstable();
    ids
}

fn open(dir: &Path, filter: &str) -> App {
    open_with(dir, filter, true)
}

fn open_with(dir: &Path, filter: &str, all: bool) -> App {
    let mut project = Project::discover(dir).expect("the project opens");
    let mut app = App::new();
    // `--all`, to match what cairn was asked. The list's rule about hiding
    // finished work is harrow's own and not the filter's; comparing through
    // it would be comparing two different questions.
    app.show_all = all;
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

fn harrow_ids(dir: &Path, filter: &str) -> Vec<harrow::identity::Id> {
    harrow_ids_with(dir, filter, true)
}

fn harrow_ids_with(dir: &Path, filter: &str, all: bool) -> Vec<harrow::identity::Id> {
    let app = open_with(dir, filter, all);
    assert!(
        app.filter_problem().is_none(),
        "harrow refused {filter:?}: {:?}",
        app.filter_problem()
    );
    let mut ids: Vec<harrow::identity::Id> = app
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

/// One project both comparisons read, so they cannot drift apart.
fn fixture() -> testkit::TempProject {
    let dir = testkit::project();
    std::fs::write(dir.path().join("items/0007-completed.md"), "---\nid: 7\ntitle: Completed and edited later\ntype: feature\nstatus: done\nclosed_at: 2026-09-01\nupdated: 2026-09-20\n---\n\n- [x] Finished\n").unwrap();
    dir
}

#[test]
#[ignore = "needs cairn on PATH"]
fn harrow_and_cairn_select_the_same_items() {
    let dir = fixture();
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

/// The rule about what an ordinary listing leaves out — closed work, and
/// containers — is not harrow's own. cairn has the same rule, and a saved
/// view's item set, as a reader sees it, goes through it.
///
/// Every other comparison here passes `--all` on both sides, which comes to
/// the same thing as never testing this. It is why `category!=dropped` could
/// mean *everything that still counts* in cairn and *only open work* here,
/// differing by every closed item, with this suite green. 0106.
const ORDINARY_LISTING: &[&str] = &[
    // Constrained by an equality: the shapes that already agreed.
    "status=done",
    "category=done",
    "type=bug",
    "type=milestone",
    "type=milestone,status=done",
    // Constrained by a negation: the shapes that did not.
    "status!=done",
    "status!=dropped",
    "category!=dropped",
    "type!=feature",
    "type=milestone,category!=dropped",
    // Saying nothing about either, so both defaults stand.
    "priority=p1",
    "id>0",
];

#[test]
#[ignore = "needs cairn on PATH"]
fn the_rule_about_what_an_ordinary_listing_hides_agrees() {
    let dir = fixture();
    let mut disagreed = Vec::new();
    for filter in ORDINARY_LISTING {
        let theirs = cairn_ids_with(dir.path(), filter, false);
        let ours = harrow_ids_with(dir.path(), filter, false);
        if theirs != ours {
            disagreed.push(format!("{filter:?}: cairn {theirs:?}, harrow {ours:?}"));
        }
    }
    assert!(
        disagreed.is_empty(),
        "{} of {} ordinary listings disagree:\n{}",
        disagreed.len(),
        ORDINARY_LISTING.len(),
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
            .args(["list", "--view", &view.name, "--json"])
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
        let values: serde_json::Value = serde_json::from_slice(&cairn.stdout).unwrap();
        let mut theirs: Vec<String> = values
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                if item["id"].is_string() {
                    item["id"].as_str().unwrap().to_owned()
                } else {
                    item["ref"].as_str().unwrap().to_owned()
                }
            })
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

/// The fixture is format 3; `cairn migrate` takes it to the format the
/// pinned Cairn writes — numbers kept, a `uid` tag added to every item.
#[test]
#[ignore = "needs Cairn on PATH"]
fn migrated_numbers_renderings_and_tags_agree() {
    let dir = testkit::project();
    // The ordinary reader fixture intentionally has a dangling dependency.
    // Supply its target before asking the writer to validate a migration.
    std::fs::write(
        dir.path().join("items/0999-target.md"),
        "---\nid: 999\ntitle: Target\ntype: feature\nstatus: done\n---\n",
    )
    .unwrap();
    let migrate = std::process::Command::new("cairn")
        .args(["-C", &dir.path().display().to_string(), "migrate"])
        .env("CAIRN_NO_HOOKS", "1")
        .output()
        .unwrap();
    assert!(
        migrate.status.success(),
        "{}",
        String::from_utf8_lossy(&migrate.stderr)
    );
    for filter in FILTERS {
        assert_eq!(
            cairn_ids(dir.path(), filter),
            harrow_ids(dir.path(), filter),
            "{filter}"
        );
    }
    let mut project = Project::discover(dir.path()).unwrap();
    let report = project.load().unwrap();
    assert_eq!(report.schema.format, harrow::schema::KNOWN_FORMAT);
    for item in &report.items {
        let uid = item.uid.expect("migration tags every item");
        let mut spellings = vec![
            item.id.to_string(),
            report.schema.format_id(item.id),
            uid.to_string(),
        ];
        // A tag prefix needs a letter in it, or it is a number.
        let prefix = uid.simple().to_string()[..8].to_string();
        if harrow::identity::is_uid_prefix(&prefix) {
            spellings.push(prefix);
        }
        for field in ["id", "depends_on", "part_of", "contains", "blockers"] {
            for value in &spellings {
                let filter = format!("{field}={value}");
                assert_eq!(
                    cairn_ids(dir.path(), &filter),
                    harrow_ids(dir.path(), &filter),
                    "{filter}"
                );
            }
        }
    }
}

/// Bodies built around the parts of §10.2 that are easy to get wrong: a
/// heading inside a fence, either kind of fence, level and case, indentation
/// and closing `#`s, and a note that ends a Result whatever its level.
const RESULTS: &[(&str, &str)] = &[
    (
        "fenced",
        "## Result\n\nKept.\n\n```sh\n# not a heading\ncairn check\n```\n\nStill kept.\n\n## Next\n\nNot it.\n",
    ),
    (
        "tilde-fenced",
        "## Result\n\n~~~\n```\n## inside\n~~~\n\nAfter.\n\n## Next\n",
    ),
    (
        "level-and-case",
        "### result\n\nShort.\n\n## Notes\n\nNot it.\n",
    ),
    ("decorated", "  ## Result ##\n\nIndented, closed.\n"),
    (
        "above-a-note",
        "# Result\n\nThe answer.\n\n## 2026-09-27\n\nA note, not part of it.\n",
    ),
    ("empty", "## Result\n\n## Next\n\nNothing to quote.\n"),
    ("none", "## Approach\n\nNo result here.\n"),
];

/// Each item's Result, as `id → result`, from cairn's machine output.
fn cairn_results(dir: &Path) -> Vec<(harrow::identity::Id, Option<String>)> {
    let out = std::process::Command::new("cairn")
        .args(["-C", &dir.display().to_string(), "list", "--all", "--json"])
        .output()
        .expect("cairn runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let items: serde_json::Value = serde_json::from_slice(&out.stdout).expect("item JSON");
    let mut results: Vec<_> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                serde_json::from_value(item["id"].clone()).expect("an id"),
                item["result"].as_str().map(str::to_string),
            )
        })
        .collect();
    results.sort();
    results
}

fn harrow_results(dir: &Path) -> Vec<(harrow::identity::Id, Option<String>)> {
    let mut project = Project::discover(dir).expect("the project opens");
    let mut results: Vec<_> = project
        .load()
        .expect("it loads")
        .items
        .into_iter()
        .map(|item| (item.id, item.result))
        .collect();
    results.sort();
    results
}

/// A dependent quotes what its dependency concluded, in both tools, so the
/// two must find the same conclusion in every item: null for null.
#[test]
#[ignore = "needs cairn on PATH and CAIRN_PROJECT_DIR"]
fn every_item_concludes_the_same_in_both_tools() {
    let dir = std::env::var_os("CAIRN_PROJECT_DIR")
        .expect("set CAIRN_PROJECT_DIR to the pinned Cairn checkout; never skip this gate");
    let pinned = cairn_results(Path::new(&dir));
    assert!(
        pinned.iter().any(|(_, result)| result.is_some()),
        "the pinned Cairn records Results, so this compares some"
    );
    assert_eq!(harrow_results(Path::new(&dir)), pinned);

    let built = testkit::project();
    for (n, (name, body)) in RESULTS.iter().enumerate() {
        let id = 500 + n;
        std::fs::write(
            built.path().join(format!("items/{id:04}-{name}.md")),
            format!("---\nid: {id}\ntitle: {name}\ntype: feature\nstatus: done\n---\n\n{body}"),
        )
        .unwrap();
    }
    let theirs = cairn_results(built.path());
    assert_eq!(
        theirs.iter().filter(|(_, result)| result.is_some()).count(),
        5,
        "cairn finds a Result in every built body but the empty one and the one without: {theirs:?}"
    );
    assert_eq!(harrow_results(built.path()), theirs);
}
