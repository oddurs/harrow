//! The project's schema, read from `cairn.toml`.
//!
//! cairn does not have one set of statuses; it has the ones your project
//! declared. Everything harrow draws — the columns on the board, the order of
//! the groups, which items count as finished, what colour a type is — comes
//! from this file rather than from anything hardcoded here.
//!
//! The file is read directly rather than through `cairn config --json`. It is
//! the source of truth either way, parsing it costs a millisecond, and it is
//! the only place the appearance keys (`icon`, `color`) survive: the resolved
//! schema cairn prints has already dropped them.
//!
//! Unknown keys are ignored on purpose. cairn's format grows, and a project
//! written for a newer cairn must still open here.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::diag;

/// The on-disk format harrow knows how to read. A project written in a later
/// one still opens — every key harrow does not know is skipped — but it says so,
/// because a silently half-read backlog is worse than a warning.
pub const KNOWN_FORMAT: u32 = 2;

/// What a status *means*, as opposed to what a project calls it. The only part
/// of the status table anything reasons about.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    #[default]
    Open,
    Active,
    Done,
    Dropped,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Open,
        Category::Active,
        Category::Done,
        Category::Dropped,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Category::Open => "open",
            Category::Active => "active",
            Category::Done => "done",
            Category::Dropped => "dropped",
        }
    }

    pub fn from_name(s: &str) -> Option<Category> {
        Category::ALL
            .into_iter()
            .find(|c| c.name() == s.trim().to_lowercase())
    }

    /// Finished, one way or the other — the two categories hidden until `a`.
    pub fn is_closed(self) -> bool {
        matches!(self, Category::Done | Category::Dropped)
    }
}

#[derive(Clone, Debug)]
pub struct Status {
    pub name: String,
    pub label: Option<String>,
    pub category: Category,
    /// False when the file left `category` out. cairn treats that as a schema
    /// problem rather than a default, and so does `harrow --doctor`.
    pub category_declared: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub board: bool,
}

impl Status {
    /// What to print. A status can be spelled `doing` and read "in progress".
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

#[derive(Clone, Debug)]
pub struct ItemType {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
}

impl ItemType {
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldKind {
    Text,
    Enum,
    List,
    Date,
    Number,
    Bool,
    Ref,
}

impl FieldKind {
    fn parse(s: &str) -> FieldKind {
        match s.trim().to_lowercase().as_str() {
            "enum" => FieldKind::Enum,
            "list" => FieldKind::List,
            "date" => FieldKind::Date,
            "number" => FieldKind::Number,
            "bool" => FieldKind::Bool,
            "ref" => FieldKind::Ref,
            _ => FieldKind::Text,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub kind: FieldKind,
    /// Declared order, for enums. `p0` before `p1` because the file says so,
    /// not because it sorts that way.
    pub values: Vec<String>,
    pub default: Option<String>,
    pub description: Option<String>,
    pub column: bool,
    pub target: Option<String>,
    pub many: bool,
    /// Whether the reference composes a hierarchy. `milestone` and `part_of` do;
    /// `depends_on` names a prerequisite rather than a parent, and does not.
    pub rollup: bool,
}

impl Field {
    /// Where a value falls in the declared order. Unknown values sort last.
    pub fn rank(&self, value: &str) -> usize {
        self.values
            .iter()
            .position(|v| v.eq_ignore_ascii_case(value))
            .unwrap_or(usize::MAX)
    }
}

#[derive(Clone, Debug)]
pub struct View {
    pub name: String,
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Schema {
    /// The directory holding `cairn.toml`.
    pub root: PathBuf,
    pub name: String,
    pub description: Option<String>,
    /// Where item files live, relative to the root.
    pub dir: PathBuf,
    pub id_width: usize,
    pub url: Option<String>,
    pub format: u32,
    pub types: Vec<ItemType>,
    pub statuses: Vec<Status>,
    pub fields: Vec<Field>,
    pub views: Vec<View>,
    pub render_target: Option<PathBuf>,
}

impl Schema {
    /// Walk up from `start` for a `cairn.toml`, as git does for `.git`.
    pub fn find(start: &Path) -> Option<PathBuf> {
        let mut dir = Some(start);
        while let Some(d) = dir {
            let candidate = d.join("cairn.toml");
            if candidate.is_file() {
                return Some(candidate);
            }
            dir = d.parent();
        }
        None
    }

    pub fn load(path: &Path) -> Result<Schema, SchemaError> {
        let body = std::fs::read_to_string(path).map_err(|e| SchemaError::Io {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;
        let root = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Schema::parse(&body, root)
    }

    pub fn parse(body: &str, root: PathBuf) -> Result<Schema, SchemaError> {
        let file: SchemaFile = toml::from_str(body).map_err(|e| SchemaError::Parse {
            detail: e.to_string().lines().next().unwrap_or("").to_string(),
        })?;

        let format = file.format.unwrap_or(1);
        if format > KNOWN_FORMAT {
            diag::warn(
                "schema",
                format!(
                    "this project is cairn format {format}; harrow reads {KNOWN_FORMAT} \
                     and will ignore anything newer"
                ),
            );
        }

        let project = file.project.unwrap_or_default();
        let mut statuses: Vec<Status> = file
            .status
            .into_iter()
            .map(|s| Status {
                category: s.category.unwrap_or_default(),
                category_declared: s.category.is_some(),
                name: s.name,
                label: s.label,
                color: s.color,
                icon: s.icon,
                board: s.board.unwrap_or(true),
            })
            .collect();

        // cairn refuses a project with no statuses, so reaching here means the
        // file is being read some other way — a fragment, a test, a work in
        // progress. Fall back rather than refuse: harrow is a reader, and a
        // reader that will not open the file is no use for finding out why.
        if statuses.is_empty() {
            diag::warn("schema", "no [[status]] blocks; assuming cairn's defaults");
            statuses = default_statuses();
        }

        let types: Vec<ItemType> = file
            .r#type
            .into_iter()
            .map(|t| ItemType {
                name: t.name,
                label: t.label,
                icon: t.icon,
                color: t.color,
                description: t.description,
            })
            .collect();

        let fields: Vec<Field> = file
            .field
            .into_iter()
            .map(|f| Field {
                name: f.name,
                kind: FieldKind::parse(f.kind.as_deref().unwrap_or("text")),
                values: f.values,
                default: f.default,
                description: f.description,
                column: f.column.unwrap_or(false),
                target: f.target,
                many: f.cardinality.as_deref() == Some("many") || f.kind.as_deref() == Some("list"),
                rollup: f.rollup.unwrap_or(false),
            })
            .collect();

        let views: Vec<View> = file
            .view
            .into_iter()
            .map(|v| View {
                name: v.name,
                filter: v.filter,
                sort: v.sort,
                description: v.description,
            })
            .collect();

        Ok(Schema {
            name: project.name.unwrap_or_else(|| {
                root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "project".to_string())
            }),
            description: project.description,
            dir: PathBuf::from(project.dir.unwrap_or_else(|| "items".to_string())),
            id_width: project.id_width.unwrap_or(4).clamp(1, 12),
            url: project.url,
            format,
            types,
            statuses,
            fields,
            views,
            render_target: file.render.and_then(|r| r.target).map(PathBuf::from),
            root,
        })
    }

    pub fn items_dir(&self) -> PathBuf {
        self.root.join(&self.dir)
    }

    pub fn status(&self, name: &str) -> Option<&Status> {
        self.statuses.iter().find(|s| s.name == name)
    }

    /// Position in the declared status list — the order of the board columns
    /// and of everything sorted by status.
    pub fn status_index(&self, name: &str) -> usize {
        self.statuses
            .iter()
            .position(|s| s.name == name)
            .unwrap_or(usize::MAX)
    }

    pub fn category(&self, status: &str) -> Category {
        self.status(status).map(|s| s.category).unwrap_or_default()
    }

    pub fn item_type(&self, name: &str) -> Option<&ItemType> {
        self.types.iter().find(|t| t.name == name)
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn view(&self, name: &str) -> Option<&View> {
        self.views.iter().find(|v| v.name == name)
    }

    /// The statuses that get a column on the board, in declared order.
    pub fn board_statuses(&self) -> Vec<&Status> {
        self.statuses.iter().filter(|s| s.board).collect()
    }

    /// The first `done` status — where closing an item sends it.
    pub fn done_status(&self) -> Option<&Status> {
        self.statuses.iter().find(|s| s.category == Category::Done)
    }

    /// `0042`, as the filenames and the printed references spell it.
    pub fn format_id(&self, id: u32) -> String {
        format!("{:0width$}", id, width = self.id_width)
    }

    /// Fields worth offering as a grouping axis: the enums and the references,
    /// which are the ones with few enough values to make readable groups.
    pub fn groupable_fields(&self) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|f| {
                matches!(f.kind, FieldKind::Enum | FieldKind::Ref)
                    && !f.many
                    && f.name != "depends_on"
            })
            .collect()
    }

    /// Schema problems worth reporting but not worth refusing to open over.
    /// The same ground `cairn check` covers, from the reader's side.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for s in &self.statuses {
            if !s.category_declared {
                out.push(format!(
                    "status {:?} declares no category; treated as open",
                    s.name
                ));
            }
        }
        if self.done_status().is_none() {
            out.push("no status has category = \"done\"; nothing can be closed".to_string());
        }
        if !self.items_dir().is_dir() {
            out.push(format!("{} does not exist", self.items_dir().display()));
        }
        out
    }
}

fn default_statuses() -> Vec<Status> {
    [
        ("backlog", Category::Open),
        ("doing", Category::Active),
        ("done", Category::Done),
    ]
    .into_iter()
    .map(|(name, category)| Status {
        name: name.to_string(),
        label: None,
        category,
        category_declared: false,
        color: None,
        icon: None,
        board: true,
    })
    .collect()
}

#[derive(Debug)]
pub enum SchemaError {
    NotFound { from: PathBuf },
    Io { path: PathBuf, detail: String },
    Parse { detail: String },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::NotFound { from } => write!(
                f,
                "no cairn.toml in {} or any parent directory",
                from.display()
            ),
            SchemaError::Io { path, detail } => write!(f, "{}: {detail}", path.display()),
            SchemaError::Parse { detail } => write!(f, "cairn.toml: {detail}"),
        }
    }
}

impl std::error::Error for SchemaError {}

// ─── The on-disk form ────────────────────────────────────────────────────────
// Everything optional, and unknown keys ignored: this is somebody else's file
// format, and harrow reading less of it than cairn writes must never be fatal.

#[derive(Debug, Default, Deserialize)]
struct SchemaFile {
    format: Option<u32>,
    project: Option<ProjectFile>,
    #[serde(default)]
    r#type: Vec<TypeFile>,
    #[serde(default)]
    status: Vec<StatusFile>,
    #[serde(default)]
    field: Vec<FieldFile>,
    #[serde(default)]
    view: Vec<ViewFile>,
    render: Option<RenderFile>,
}

#[derive(Debug, Default, Deserialize)]
struct ProjectFile {
    name: Option<String>,
    description: Option<String>,
    dir: Option<String>,
    id_width: Option<usize>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TypeFile {
    name: String,
    label: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatusFile {
    name: String,
    label: Option<String>,
    category: Option<Category>,
    color: Option<String>,
    icon: Option<String>,
    board: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct FieldFile {
    name: String,
    kind: Option<String>,
    #[serde(default)]
    values: Vec<String>,
    default: Option<String>,
    description: Option<String>,
    column: Option<bool>,
    target: Option<String>,
    cardinality: Option<String>,
    rollup: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ViewFile {
    name: String,
    filter: Option<String>,
    sort: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenderFile {
    target: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
format = 2

[project]
name = "quarry"
dir = "cairn/items"
id_width = 4

[[type]]
name = "feature"
icon = "+"
color = "cyan"

[[type]]
name = "bug"
icon = "!"
color = "red"

[[status]]
name = "backlog"
category = "open"

[[status]]
name = "doing"
label = "in progress"
category = "active"

[[status]]
name = "done"
category = "done"

[[status]]
name = "dropped"
category = "dropped"
board = false

[[field]]
name = "priority"
kind = "enum"
values = ["p0", "p1", "p2"]
default = "p2"
column = true

[[view]]
name = "now"
filter = "category=active"
"#;

    fn sample() -> Schema {
        Schema::parse(SAMPLE, PathBuf::from("/tmp/project")).expect("the sample parses")
    }

    #[test]
    fn the_declared_order_is_the_order() {
        let s = sample();
        assert_eq!(s.status_index("backlog"), 0);
        assert!(s.status_index("done") > s.status_index("doing"));
        assert_eq!(s.board_statuses().len(), 3, "dropped asked not to be shown");
    }

    #[test]
    fn appearance_survives_the_read() {
        // The whole reason for parsing the file rather than asking cairn for the
        // resolved schema: `cairn config --json` has already dropped these.
        let s = sample();
        assert_eq!(
            s.item_type("bug").and_then(|t| t.color.clone()),
            Some("red".into())
        );
        assert_eq!(
            s.item_type("feature").and_then(|t| t.icon.clone()),
            Some("+".into())
        );
    }

    #[test]
    fn a_label_is_what_gets_printed() {
        let s = sample();
        assert_eq!(s.status("doing").unwrap().display(), "in progress");
        assert_eq!(s.status("done").unwrap().display(), "done");
    }

    #[test]
    fn categories_resolve_through_the_status_table() {
        let s = sample();
        assert_eq!(s.category("doing"), Category::Active);
        assert_eq!(
            s.category("nonesuch"),
            Category::Open,
            "unknown reads as open"
        );
        assert_eq!(s.done_status().map(|s| s.name.clone()), Some("done".into()));
    }

    #[test]
    fn enum_order_is_declaration_order_not_alphabetical() {
        let s = sample();
        let f = s.field("priority").expect("priority is declared");
        assert!(f.rank("p0") < f.rank("p2"));
        assert_eq!(f.rank("nonesuch"), usize::MAX, "unknown values sort last");
        assert_eq!(f.default.as_deref(), Some("p2"));
    }

    #[test]
    fn a_key_from_a_later_format_is_ignored_rather_than_fatal() {
        let body = format!(
            "{SAMPLE}\n[[status]]\nname = \"shipped\"\ncategory = \"done\"\nagent = \"never\"\n"
        );
        let s = Schema::parse(&body, PathBuf::from("/tmp/p")).expect("unknown keys are skipped");
        assert_eq!(
            s.status("shipped").map(|s| s.category),
            Some(Category::Done)
        );
    }

    #[test]
    fn a_file_that_will_not_parse_says_so() {
        let err = Schema::parse("format = ", PathBuf::from("/tmp/p")).unwrap_err();
        assert!(err.to_string().contains("cairn.toml"), "{err}");
    }

    #[test]
    fn a_schema_with_no_statuses_still_opens() {
        let s = Schema::parse("[project]\nname = \"x\"\n", PathBuf::from("/tmp/p"))
            .expect("a fragment is readable");
        assert!(!s.statuses.is_empty(), "harrow must not refuse to open");
        assert!(
            s.problems().iter().any(|p| p.contains("category")),
            "and must say the categories were assumed: {:?}",
            s.problems()
        );
    }

    #[test]
    fn ids_are_formatted_the_way_the_filenames_are() {
        assert_eq!(sample().format_id(42), "0042");
    }

    #[test]
    fn finding_walks_up_like_git_does() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).expect("nested dirs");
        std::fs::write(dir.path().join("cairn.toml"), SAMPLE).expect("write");
        assert_eq!(
            Schema::find(&nested),
            Some(dir.path().join("cairn.toml")),
            "a subdirectory of a project is in the project"
        );
        assert_eq!(Schema::find(Path::new("/")), None);
    }
}
