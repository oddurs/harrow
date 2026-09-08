//! Fixtures.
//!
//! One sample project, built the same way in every test, so a test that fails
//! says something about the code rather than about the data it happened to use.
//! The fixture is written to disk rather than constructed in memory: the file
//! format is half of what harrow does, and a fixture that skipped it would test
//! the other half twice.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::engine::{Report, Source};
use crate::item::Item;
use crate::schema::Schema;

pub const CAIRN_TOML: &str = r#"
format = 2

[project]
name = "sample"
dir = "items"
id_width = 4
default_type = "feature"
default_status = "backlog"

[[type]]
name = "feature"
icon = "+"
color = "cyan"

[[type]]
name = "bug"
icon = "!"
color = "red"

[[type]]
name = "chore"
icon = "~"
color = "gray"

[[type]]
name = "milestone"

[[status]]
name = "backlog"
category = "open"
color = "gray"

[[status]]
name = "doing"
label = "in progress"
category = "active"
color = "yellow"

[[status]]
name = "done"
category = "done"
color = "green"

[[status]]
name = "dropped"
category = "dropped"
color = "gray"
board = false

[[field]]
name = "milestone"
kind = "ref"
target = "milestone"
by = "key"
rollup = true

[[field]]
name = "depends_on"
kind = "ref"
target = "*"
cardinality = "many"

[[field]]
name = "part_of"
kind = "ref"
target = "*"
cardinality = "many"
rollup = true

[[field]]
name = "priority"
kind = "enum"
values = ["p0", "p1", "p2", "p3"]
default = "p2"
column = true

[[field]]
name = "area"
kind = "enum"
values = ["ui", "core", "docs"]
column = true

[[view]]
name = "now"
filter = "category=active"
"#;

/// The item files, in the order a listing would show them.
pub const ITEMS: &[(&str, &str)] = &[
    (
        "0001-first-usable-version.md",
        "---\nid: 1\nkey: v0.1\ntitle: First usable version\ntype: milestone\nstatus: backlog\ncreated: 2026-09-01\ndue: 2026-12-01\n---\n\nEnough to dogfood.\n",
    ),
    (
        "0002-read-the-item-files.md",
        "---\nid: 2\ntitle: Read the item files\ntype: feature\nstatus: done\nmilestone: v0.1\ncreated: 2026-09-01\nupdated: 2026-09-02\npriority: p0\narea: core\n---\n\n## Problem\n\nSomething has to parse the frontmatter.\n\n- [x] parses a block sequence\n- [x] keeps unknown keys\n",
    ),
    (
        "0003-draw-the-list.md",
        "---\nid: 3\ntitle: Draw the list\ntype: feature\nstatus: doing\nmilestone: v0.1\ncreated: 2026-09-02\npriority: p1\narea: ui\nassignee: oddur\nclaimed: 2026-09-03\n---\n\n## Problem\n\nA backlog you cannot move through is a file you could have opened.\n\n- [x] rows\n- [ ] groups\n",
    ),
    (
        "0004-draw-the-board.md",
        "---\nid: 4\ntitle: Draw the board\ntype: feature\nstatus: backlog\nmilestone: v0.1\ndepends_on:\n- 3\ncreated: 2026-09-02\npriority: p2\narea: ui\n---\n\n## Problem\n\nColumns, once there are rows.\n",
    ),
    (
        "0005-the-detail-pane-scrolls-past-its-pane.md",
        "---\nid: 5\ntitle: The detail pane scrolls past its pane\ntype: bug\nstatus: backlog\nmilestone: v0.1\ndepends_on:\n- 2\nlabels: [chrome]\ncreated: 2026-09-04\npriority: p1\narea: ui\n---\n\n## What happens\n\nA long body runs off the bottom.\n",
    ),
    (
        "0006-write-the-readme.md",
        "---\nid: 6\ntitle: Write the readme\ntype: chore\nstatus: backlog\ndepends_on:\n- 999\ncreated: 2026-09-05\npriority: p3\n---\n\nA dangling dependency, on purpose.\n",
    ),
];

/// A sample project on disk, removed when the handle drops.
///
/// Hand-rolled rather than pulled from a crate because `testkit` ships in the
/// library — the integration tests use it — and a fixture is not worth a
/// dependency in everybody's build.
pub struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    pub fn path(&self) -> &std::path::Path {
        &self.dir
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub fn project() -> TempProject {
    // Unique per process and per call: `cargo test` runs these concurrently,
    // and two tests sharing a directory is a failure that only shows up on a
    // fast machine.
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("harrow-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let items = dir.join("items");
    std::fs::create_dir_all(&items).expect("create the items directory");
    std::fs::write(dir.join("cairn.toml"), CAIRN_TOML).expect("write cairn.toml");
    for (name, body) in ITEMS {
        std::fs::write(items.join(name), body).expect("write an item");
    }
    TempProject { dir }
}

/// The sample schema, with no directory behind it.
pub fn schema() -> Schema {
    Schema::parse(CAIRN_TOML, PathBuf::from("/tmp/sample")).expect("the fixture parses")
}

/// The sample backlog, loaded and derived, with no filesystem involved.
pub fn report() -> Report {
    let items = ITEMS
        .iter()
        .map(|(name, body)| {
            crate::item::parse(body, &PathBuf::from("items").join(name)).expect("a fixture parses")
        })
        .collect();
    let mut source = crate::engine::Static {
        schema: schema(),
        items,
    };
    source.load().expect("the fixture loads")
}

/// One item, for the tests that need a shape rather than a story.
pub fn item(id: u32, title: &str, status: &str) -> Item {
    Item {
        id,
        title: title.to_string(),
        kind: "feature".to_string(),
        status: status.to_string(),
        fields: BTreeMap::new(),
        path: PathBuf::from(format!("items/{id:04}-{title}.md")),
        ..Default::default()
    }
}

/// An app holding the sample backlog, at a size the snapshots use.
pub fn app() -> crate::app::App {
    let report = report();
    let mut app = crate::app::App::new();
    app.ingest(report);
    app
}
