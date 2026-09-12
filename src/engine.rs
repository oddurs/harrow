//! Loading a backlog, and working out everything the files do not say.
//!
//! An item file records intent. It does not record whether the thing it depends
//! on has landed, how much of a milestone is finished, or what is ready to start
//! — all of that is a property of the *set*, and is derived here, once, when the
//! set is read.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::diag;
use crate::item::Item;
use crate::schema::{Schema, SchemaError};

/// One reading of the backlog.
pub struct Report {
    pub schema: Schema,
    pub items: Vec<Item>,
    /// Files that could not be read, and schema problems. Shown in the
    /// diagnostics overlay rather than thrown away or printed over the screen.
    pub warnings: Vec<String>,
    /// The newest modification time seen, so the watcher can tell whether
    /// anything has changed without re-reading every file.
    pub stamp: Option<SystemTime>,
}

#[derive(Debug)]
pub struct LoadError {
    pub detail: String,
    /// True when trying again might work — a directory being rewritten under us
    /// rather than a project that does not exist.
    pub transient: bool,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.detail)
    }
}

impl std::error::Error for LoadError {}

impl From<SchemaError> for LoadError {
    fn from(e: SchemaError) -> Self {
        LoadError {
            transient: matches!(e, SchemaError::Io { .. }),
            detail: e.to_string(),
        }
    }
}

/// Where the backlog is read from. A trait so the whole pipeline can be driven
/// from a fixture: nothing above this line touches the filesystem.
pub trait Source: Send {
    fn config_path(&self) -> PathBuf;
    fn load(&mut self) -> Result<Report, LoadError>;
}

/// The real one: a `cairn.toml` and the directory it points at.
#[derive(Debug)]
pub struct Project {
    config: PathBuf,
}

impl Project {
    /// Walk up from `start` for a project, as git does for a repository.
    pub fn discover(start: &Path) -> Result<Project, SchemaError> {
        match Schema::find(start) {
            Some(config) => Ok(Project { config }),
            None => Err(SchemaError::NotFound {
                from: start.to_path_buf(),
            }),
        }
    }

    pub fn at(config: PathBuf) -> Project {
        Project { config }
    }
}

impl Source for Project {
    fn config_path(&self) -> PathBuf {
        self.config.clone()
    }

    fn load(&mut self) -> Result<Report, LoadError> {
        let schema = Schema::load(&self.config)?;
        let mut warnings = schema.problems();
        let dir = schema.items_dir();

        // Fail on the items directory itself, but never on a subdirectory
        // inside it: half a backlog read as the whole backlog is worse than a
        // warning, and the top level not being there at all is the case that
        // means "this is not a project".
        let mut files = Vec::new();
        walk(&dir, &mut files, &mut warnings).map_err(|e| LoadError {
            detail: format!("{}: {e}", dir.display()),
            transient: e.kind() != std::io::ErrorKind::NotFound,
        })?;

        let mut items = Vec::new();
        let mut stamp: Option<SystemTime> = None;
        for path in files {
            if let Ok(meta) = std::fs::metadata(&path)
                && let Ok(modified) = meta.modified()
            {
                stamp = Some(stamp.map_or(modified, |s: SystemTime| s.max(modified)));
            }
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    warnings.push(format!("{}: {e}", name_of(&path)));
                    continue;
                }
            };
            match crate::item::parse(&text, &path) {
                Ok(item) => items.push(item),
                Err(e) => warnings.push(format!("{}: {e}", name_of(&path))),
            }
        }

        derive(&mut items, &schema, &mut warnings);
        for w in &warnings {
            diag::warn("backlog", w.clone());
        }
        Ok(Report {
            schema,
            items,
            warnings,
            stamp,
        })
    }
}

/// Every item file under a directory, subdirectories included.
///
/// The format says subdirectories must be searched, and that entries whose
/// names begin with `.` or `_`, and `README.md`, are not items. Those three
/// are skipped in silence rather than warned about: a project that keeps a
/// README beside its items has not made a mistake, and a warning per reload
/// per file is how a diagnostics pane becomes something nobody reads.
fn walk(dir: &Path, out: &mut Vec<PathBuf>, warnings: &mut Vec<String>) -> std::io::Result<()> {
    let mut subdirectories = Vec::new();
    for entry in std::fs::read_dir(dir)?.filter_map(Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || name.starts_with('_') || name == "README.md" {
            continue;
        }
        match entry.file_type() {
            Ok(t) if t.is_dir() => subdirectories.push(path),
            _ if path.extension().and_then(|e| e.to_str()) == Some("md") => out.push(path),
            _ => {}
        }
    }
    // Sorted, so the order a backlog loads in is the order it is stored in
    // rather than whatever the filesystem felt like saying this time.
    subdirectories.sort();
    for sub in subdirectories {
        if let Err(e) = walk(&sub, out, warnings) {
            warnings.push(format!("{}: {e}", name_of(&sub)));
        }
    }
    out.sort();
    Ok(())
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

/// Everything that is a property of the set rather than of one file: category,
/// what is blocked by what, and how much of each larger piece of work is done.
pub fn derive(items: &mut [Item], schema: &Schema, warnings: &mut Vec<String>) {
    items.sort_by_key(|i| i.id);

    let section = schema.criteria_section.as_deref();
    for item in items.iter_mut() {
        let (met, total) = crate::item::count_criteria(&item.body, section);
        item.criteria_met = met;
        item.criteria_total = total;
    }

    let mut duplicates: Vec<u32> = Vec::new();
    let mut known: HashSet<u32> = HashSet::new();
    for i in items.iter() {
        if !known.insert(i.id) {
            duplicates.push(i.id);
        }
    }
    duplicates.dedup();
    for id in duplicates {
        // cairn has `renumber` for exactly this, so point at it rather than
        // silently showing one of the two.
        warnings.push(format!(
            "id {id} is used by more than one item — `cairn renumber` repairs it"
        ));
    }

    let mut closed: HashSet<u32> = HashSet::new();
    let mut by_key: HashMap<String, u32> = HashMap::new();
    let mut kind_of: HashMap<u32, String> = HashMap::new();
    for item in items.iter_mut() {
        item.category = schema.category(&item.status);
        if item.category.is_closed() {
            closed.insert(item.id);
        }
        if let Some(key) = &item.key {
            by_key.insert(key.to_lowercase(), item.id);
        }
        kind_of.insert(item.id, item.kind.clone());
    }

    // A reference may declare what it points at, and one that does does not
    // point at anything else: `milestone: v0.1` names a milestone, and a
    // feature that happened to take the key `v0.1` is not one.
    let of_target = |field: &crate::schema::Field, id: u32| match field.target.as_deref() {
        None | Some("*") => true,
        Some(want) => kind_of.get(&id).is_some_and(|k| k == want),
    };

    // Dependencies. A reference to an item that does not exist is a schema
    // problem, not a blocker: refusing to surface work because of a typo
    // somewhere else would be worse than the typo.
    for item in items.iter_mut() {
        item.blockers = item
            .depends_on
            .iter()
            .copied()
            .filter(|d| known.contains(d) && !closed.contains(d))
            .collect();
        item.blocked = !item.blockers.is_empty();
    }

    // Composition. Both rollup references — `milestone`, which names an item by
    // key, and `part_of`, which names one by id — feed the same tree, which is
    // what lets a milestone row carry a progress bar.
    let rollups: Vec<&crate::schema::Field> = schema
        .fields
        .iter()
        .filter(|f| matches!(f.kind, crate::schema::FieldKind::Ref) && f.rollup)
        .collect();

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut parents: HashMap<u32, Vec<u32>> = HashMap::new();
    for item in items.iter() {
        for field in &rollups {
            let Some(value) = item.fields.get(&field.name) else {
                continue;
            };
            for raw in value.items() {
                let raw = raw.trim();
                if raw.is_empty() {
                    continue;
                }
                // How a field names its target is declared, not guessed.
                // Accepting both would make `milestone: 42` mean the item
                // with that key or the item with that id depending on which
                // happens to exist, and two spellings must not be able to
                // name different items.
                let parent = match field.by {
                    crate::schema::Addressing::Key => by_key
                        .get(&raw.to_lowercase())
                        .copied()
                        .filter(|id| of_target(field, *id)),
                    crate::schema::Addressing::Id => raw
                        .trim_start_matches('#')
                        .parse::<u32>()
                        .ok()
                        .filter(|id| known.contains(id) && of_target(field, *id)),
                };
                if let Some(parent) = parent
                    && parent != item.id
                {
                    children.entry(parent).or_default().push(item.id);
                    parents.entry(item.id).or_default().push(parent);
                }
            }
        }
    }

    let mut counts: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
    for &parent in children.keys() {
        counts.insert(
            parent,
            beneath(parent, &children, &closed, &mut HashSet::new()),
        );
    }
    // Depth is a fact about the graph rather than a label somebody maintains,
    // so it is derived here and cannot go stale.
    let mut depth: HashMap<u32, u32> = HashMap::new();
    for id in known.iter().copied() {
        depth.insert(id, depth_of(id, &parents, &mut HashSet::new()));
    }

    for item in items.iter_mut() {
        if let Some((total, done)) = counts.get(&item.id) {
            item.scheduled = *total;
            item.scheduled_done = *done;
        }
        if let Some(kids) = children.get(&item.id) {
            item.contains = kids.clone();
            item.contains.sort_unstable();
            item.contains.dedup();
        }
        item.depth = depth.get(&item.id).copied().unwrap_or(0);
        item.container = schema.is_container(&item.kind);
    }
}

/// How far below a root an item sits. The visited set makes a cycle finite
/// rather than fatal, for the same reason `beneath` has one.
fn depth_of(id: u32, parents: &HashMap<u32, Vec<u32>>, seen: &mut HashSet<u32>) -> u32 {
    if !seen.insert(id) {
        return 0;
    }
    parents
        .get(&id)
        .into_iter()
        .flatten()
        .map(|p| 1 + depth_of(*p, parents, seen))
        .max()
        .unwrap_or(0)
}

/// `(total, done)` for everything under an item, transitively. The visited set
/// makes a cycle finite rather than fatal; cairn's `acyclic = true` means it
/// should never happen, and a hand-edited file means it sometimes does.
fn beneath(
    id: u32,
    children: &HashMap<u32, Vec<u32>>,
    closed: &HashSet<u32>,
    seen: &mut HashSet<u32>,
) -> (u32, u32) {
    if !seen.insert(id) {
        return (0, 0);
    }
    let mut total = 0;
    let mut done = 0;
    for child in children.get(&id).into_iter().flatten() {
        total += 1;
        if closed.contains(child) {
            done += 1;
        }
        let (t, d) = beneath(*child, children, closed, seen);
        total += t;
        done += d;
    }
    (total, done)
}

/// A backlog held in memory, for tests and screenshots.
pub struct Static {
    pub schema: Schema,
    pub items: Vec<Item>,
}

impl Source for Static {
    fn config_path(&self) -> PathBuf {
        self.schema.root.join("cairn.toml")
    }

    fn load(&mut self) -> Result<Report, LoadError> {
        let mut items = self.items.clone();
        let mut warnings = Vec::new();
        derive(&mut items, &self.schema, &mut warnings);
        Ok(Report {
            schema: self.schema.clone(),
            items,
            warnings,
            stamp: None,
        })
    }
}

/// A source that always fails, for testing what the interface does about it.
pub struct Failing(pub String);

impl Source for Failing {
    fn config_path(&self) -> PathBuf {
        PathBuf::from("cairn.toml")
    }

    fn load(&mut self) -> Result<Report, LoadError> {
        Err(LoadError {
            detail: self.0.clone(),
            transient: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    #[test]
    fn a_project_is_read_from_disk() {
        let dir = testkit::project();
        let mut source = Project::discover(dir.path()).expect("the project is found");
        let report = source.load().expect("it loads");
        assert_eq!(report.schema.name, "sample");
        assert_eq!(report.items.len(), 6);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    #[test]
    fn categories_come_from_the_projects_own_status_table() {
        use crate::schema::Category;
        let report = testkit::report();
        let done = report
            .items
            .iter()
            .filter(|i| i.category == Category::Done)
            .count();
        assert_eq!(done, 1);
    }

    #[test]
    fn an_unfinished_dependency_blocks_and_a_finished_one_does_not() {
        let report = testkit::report();
        let blocked = report.items.iter().find(|i| i.id == 4).expect("item 4");
        assert!(blocked.blocked, "4 depends on 3, which is not done");
        assert_eq!(blocked.blockers, vec![3]);

        let free = report.items.iter().find(|i| i.id == 5).expect("item 5");
        assert!(!free.blocked, "5 depends on 2, which is done");
    }

    #[test]
    fn a_dependency_on_something_that_does_not_exist_does_not_block() {
        // Otherwise a typo in one file would hide work in another.
        let report = testkit::report();
        let item = report.items.iter().find(|i| i.id == 6).expect("item 6");
        assert!(
            !item.blocked,
            "a dangling reference is a check error, not a wall"
        );
    }

    #[test]
    fn a_milestone_rolls_up_what_is_scheduled_against_it() {
        let report = testkit::report();
        let milestone = report.items.iter().find(|i| i.id == 1).expect("milestone");
        assert_eq!(milestone.scheduled, 4);
        assert_eq!(milestone.scheduled_done, 1);
        assert_eq!(milestone.progress(), Some(25));
    }

    #[test]
    fn an_ordinary_item_has_no_progress_to_report() {
        // Not zero: zero would sort every leaf below every empty milestone.
        let report = testkit::report();
        let leaf = report.items.iter().find(|i| i.id == 3).expect("item 3");
        assert_eq!(leaf.progress(), None);
    }

    #[test]
    fn a_file_that_will_not_parse_is_reported_and_the_rest_still_load() {
        let dir = testkit::project();
        std::fs::write(dir.path().join("items/9999-broken.md"), "not an item\n")
            .expect("write a broken file");
        let mut source = Project::discover(dir.path()).expect("found");
        let report = source.load().expect("one bad file is not a bad backlog");
        assert_eq!(report.items.len(), 6, "the good files still arrive");
        assert!(
            report.warnings.iter().any(|w| w.contains("9999-broken")),
            "the bad one is reported: {:?}",
            report.warnings
        );
    }

    #[test]
    fn a_missing_project_is_reported_rather_than_guessed_at() {
        let err = Project::discover(std::path::Path::new("/nonexistent-harrow-dir")).unwrap_err();
        assert!(err.to_string().contains("cairn.toml"), "{err}");
    }

    #[test]
    fn a_cycle_is_finite_rather_than_fatal() {
        let schema = testkit::schema();
        let mut items = vec![
            testkit::item(1, "a", "backlog"),
            testkit::item(2, "b", "backlog"),
        ];
        items[0]
            .fields
            .insert("part_of".into(), crate::item::Value::One("2".into()));
        items[1]
            .fields
            .insert("part_of".into(), crate::item::Value::One("1".into()));
        let mut warnings = Vec::new();
        derive(&mut items, &schema, &mut warnings); // Hangs here if the guard regresses.
    }

    #[test]
    fn duplicate_ids_are_reported_with_the_command_that_repairs_them() {
        let schema = testkit::schema();
        let mut items = vec![
            testkit::item(1, "a", "backlog"),
            testkit::item(1, "b", "backlog"),
        ];
        let mut warnings = Vec::new();
        derive(&mut items, &schema, &mut warnings);
        assert!(
            warnings.iter().any(|w| w.contains("renumber")),
            "{warnings:?}"
        );
    }
}
