//! Backlogs nobody wrote by hand.
//!
//! Every other test here runs against one fixture: six items, four statuses,
//! one milestone, a schema written by the same person who wrote the parser.
//! The shapes that break a program of this kind are the ones nobody would
//! think to write — two hundred statuses, a project with no milestones, a
//! schema whose statuses are all `done`, a title of one emoji, five thousand
//! items in one group.
//!
//! So the schema is generated too, not just the items. The rule the
//! generator follows is that everything it produces would pass `cairn
//! check`: the schema is the contract, and it is the contract harrow has to
//! survive. Generating impossible projects would test behaviour against data
//! cairn would never write.
//!
//! Seeded, printed, reproducible. `HARROW_SOAK_SEEDS=5000 cargo test
//! --release --test soak` runs it harder; the default belongs in every run.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use harrow::app::App;
use harrow::engine::{Project, Source};
use harrow::keys::Keymap;
use harrow::ui;

/// A named, seeded generator. `Math.random` in a test is a bug you cannot
/// reproduce.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }

    fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in.max(1)) == 0
    }

    fn pick<'a, T>(&mut self, from: &'a [T]) -> &'a T {
        &from[self.below(from.len())]
    }
}

/// Titles that are awkward for reasons a real backlog eventually finds.
const TITLES: &[&str] = &[
    "Do the thing",
    "日本語のタイトル",
    "🎯",
    "A title: with a colon, a #hash and a *star",
    "x",
    "Reticulating splines across a very long line indeed, at some length, on and on",
    "  leading and trailing  ",
    "quoted \"inside\" it",
];

/// A project, schema and all.
///
/// Everything here would pass `cairn check`: at least one status, exactly
/// one category per status, declared values for every enum, and references
/// that name a type that exists.
fn project(rng: &mut Rng, dir: &Path) -> usize {
    let statuses = 1 + rng.below(12);
    let mut toml = String::from("format = 3\n\n[project]\nname = \"soak\"\ndir = \"items\"\n");
    // The renderings a project may choose, all of which harrow has to draw
    // — and which the filenames then have to match, because cairn checks
    // that they do.
    let render: Box<dyn Fn(usize) -> String> = match rng.below(4) {
        0 => {
            toml.push_str("id_width = 1\n");
            Box::new(|n| n.to_string())
        }
        1 => {
            toml.push_str("id_format = \"SOAK-{n:05}\"\n");
            Box::new(|n| format!("SOAK-{n:05}"))
        }
        2 => {
            toml.push_str("id_format = \"A{n}\"\n");
            Box::new(|n| format!("A{n}"))
        }
        _ => {
            toml.push_str("id_width = 4\n");
            Box::new(|n| format!("{n:04}"))
        }
    };
    if rng.chance(3) {
        toml.push_str("claim_stale_after = 1\n");
    }
    if rng.chance(3) {
        toml.push_str("criteria_section = \"Acceptance\"\n");
    }

    // A project whose statuses are all done is a project somebody finished.
    let categories = ["open", "active", "done", "dropped"];
    let mut names = Vec::new();
    for n in 0..statuses {
        let name = format!("s{n}");
        let category = if rng.chance(6) {
            "done"
        } else {
            rng.pick(&categories)
        };
        let _ = write!(
            toml,
            "\n[[status]]\nname = \"{name}\"\ncategory = \"{category}\"\n"
        );
        if rng.chance(4) {
            toml.push_str("board = false\n");
        }
        names.push(name);
    }
    toml.push_str("\n[[type]]\nname = \"task\"\n");
    // Sometimes the project groups its work, and sometimes it does not.
    let grouped = rng.chance(2);
    if grouped {
        toml.push_str("\n[[type]]\nname = \"epic\"\ngroups = \"one\"\ninverse = \"holds\"\n");
    } else {
        // The roadmap groups by milestone unless told otherwise, and a
        // project with no grouping type has no milestones to group by.
        toml.push_str("\n[render]\ngroup_by = \"status\"\n");
    }
    let has_priority = rng.chance(2);
    if has_priority {
        toml.push_str(
            "\n[[field]]\nname = \"priority\"\nkind = \"enum\"\nvalues = [\"p0\", \"p1\"]\ncolumn = true\n",
        );
    }
    if rng.chance(3) {
        toml.push_str(
            "\n[[view]]\nname = \"now\"\nfilter = \"category=active\"\ngroup_by = \"status\"\n",
        );
    }

    std::fs::create_dir_all(dir.join("items")).expect("make the items directory");
    std::fs::write(dir.join("cairn.toml"), toml).expect("write the schema");

    // Occasionally none at all, which is a project on its first day.
    let count = if rng.chance(8) { 0 } else { 1 + rng.below(60) };
    let mut epics = Vec::new();
    for id in 1..=count {
        let kind = if grouped && rng.chance(6) {
            "epic"
        } else {
            "task"
        };
        // Quoted, because the format says a writer must quote anything that
        // would change meaning when read back — and half of these would. The
        // awkwardness is kept; only the ambiguity goes.
        let title = rng.pick(TITLES).replace('\\', "\\\\").replace('"', "\\\"");
        let mut front = format!(
            "---\nid: {id}\ntitle: \"{title}\"\ntype: {kind}\nstatus: {}\n",
            rng.pick(&names)
        );
        if kind == "epic" {
            let key = format!("e{id}");
            let _ = writeln!(front, "key: {key}");
            epics.push(key);
        } else if grouped && !epics.is_empty() && rng.chance(2) {
            let _ = writeln!(front, "epic: {}", rng.pick(&epics));
        }
        if rng.chance(3) {
            let _ = writeln!(front, "assignee: {}", rng.pick(&["someone", "an agent"]));
            let _ = writeln!(front, "claimed: 2020-01-01");
        }
        if rng.chance(4) {
            let _ = writeln!(front, "owner: somebody else");
        }
        if rng.chance(5) {
            let _ = writeln!(front, "created_by: an agent");
        }
        // Only where the project declared it: an undeclared field is a
        // warning cairn is right to raise, and not what this is testing.
        if has_priority && rng.chance(3) {
            let _ = writeln!(front, "priority: {}", rng.pick(&["p0", "p1"]));
        }
        // Strictly backwards, so the graph is acyclic by construction:
        // `rng.below(id) + 1` can return `id` itself, which is a cycle of
        // one and a thing cairn refuses.
        if rng.chance(4) && id > 1 {
            let _ = writeln!(front, "depends_on: [{}]", rng.below(id - 1) + 1);
        }
        let mut body = String::from("\n---\n\n## Problem\n\nSomething.\n");
        if rng.chance(3) {
            body.push_str("\n## Acceptance\n\n- [x] one\n- [ ] two\n- [ ]\n");
        }
        if rng.chance(4) {
            body.push_str("\n## 2026-09-11\n\nA note about what was tried.\n");
        }
        if rng.chance(5) {
            body.push_str("\n## Proposed priority: p1 -> p0 (an agent, 2026-09-11)\n\nBecause.\n");
        }
        std::fs::write(
            dir.join(format!(
                "items/{}-{}.md",
                render(id),
                slug(title_of(&front))
            )),
            format!("{front}{body}"),
        )
        .expect("write an item");
    }
    count
}

/// The title back out of the frontmatter, which is where it was just put.
fn title_of(front: &str) -> &str {
    front
        .lines()
        .find_map(|l| l.strip_prefix("title: "))
        .unwrap_or("item")
        .trim_matches('"')
}

/// `<id>-<slug>.md`, where the slug is the title reduced to lowercase
/// alphanumerics separated by single hyphens. cairn checks that a filename
/// matches its item, so a generator that does not do this writes a project
/// cairn refuses — which would make the soak a test of the wrong thing.
fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() { "item".into() } else { out }
}

fn seeds() -> u64 {
    std::env::var("HARROW_SOAK_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60)
}

/// Unique per process *and* per call: `cargo test` runs these concurrently,
/// and two of them sharing a directory read each other's items — which is a
/// failure that only shows up on a fast machine, and did.
fn scratch(seed: u64) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("harrow-soak-{}-{n}-{seed}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_generated_project_opens_and_can_be_driven() {
    let keys = Keymap::default().every_key();
    for seed in 0..seeds() {
        let mut rng = Rng(0x50a4_0000_0000_0001 ^ seed.wrapping_mul(0x9e37_79b9));
        let dir = scratch(seed);
        let count = project(&mut rng, &dir);

        let mut source = Project::discover(&dir)
            .unwrap_or_else(|e| panic!("seed {seed}: a generated project did not open: {e:?}"));
        let report = source
            .load()
            .unwrap_or_else(|e| panic!("seed {seed}: it did not load: {e:?}"));
        let mut app = App::new();
        app.ingest(report);
        assert!(
            app.check_invariants().is_ok(),
            "seed {seed} on opening: {:?}",
            app.check_invariants()
        );
        assert_eq!(app.items.len(), count, "seed {seed}: every item read");

        // Then driven, at a size that is not the one the snapshots use.
        for n in 0..40 {
            let (code, mods) = keys[rng.below(keys.len())];
            let _ = app.handle_key(code, mods);
            let _ = ui::render_frame(&mut app, 100, 30, 0);
            if let Err(problem) = app.check_invariants() {
                panic!("seed {seed} after {n} keys ({code:?} {mods:?}): {problem}");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// The size that is always wrong, against schemas that are always different.
#[test]
fn a_generated_project_draws_on_a_terminal_too_small_for_it() {
    for seed in 0..seeds().min(30) {
        let mut rng = Rng(0xbeef_0000_0000_0001 ^ seed.wrapping_mul(0x9e37_79b9));
        let dir = scratch(seed);
        project(&mut rng, &dir);
        let mut source = Project::discover(&dir).expect("it opens");
        let mut app = App::new();
        app.ingest(source.load().expect("it loads"));

        for lens in harrow::app::Pane::ALL {
            app.pane = lens;
            for (w, h) in [(1u16, 1u16), (17, 4), (38, 9), (240, 60)] {
                let _ = ui::render_frame(&mut app, w, h, 0);
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Kept out of the suite because it needs cairn on PATH and a test that
/// silently stops running is worse than none. Run it by hand when the
/// generator changes: `cargo test --test soak -- --ignored --nocapture`.
#[test]
#[ignore = "needs cairn on PATH"]
fn everything_the_generator_writes_would_pass_cairn_check() {
    for seed in 0..40u64 {
        let mut rng = Rng(0xc4ec_0000_0000_0001 ^ seed.wrapping_mul(0x9e37_79b9));
        let dir = scratch(seed);
        project(&mut rng, &dir);
        let out = std::process::Command::new("cairn")
            .args(["-C", &dir.display().to_string(), "check"])
            .output()
            .expect("cairn runs");
        assert!(
            out.status.success(),
            "seed {seed}: cairn refuses a project the soak generated\n{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
