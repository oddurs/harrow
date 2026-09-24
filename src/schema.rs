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

use crate::identity::Id;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::diag;

/// The on-disk format harrow knows how to read. A project written in a later
/// one still opens — every key harrow does not know is skipped — but it says so,
/// because a silently half-read backlog is worse than a warning.
pub const KNOWN_FORMAT: u32 = 4;

/// What a project allows a tool to do with a field or a status.
///
/// cairn's own words: a guard rail rather than a security boundary. Nothing
/// detects an agent, and a command line cannot ask. harrow enforces nothing —
/// the write goes through `cairn`, which decides — but it shows the restriction
/// where the choice is made, so a refusal is never a surprise.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    #[default]
    Unrestricted,
    ReadOnly,
    Propose,
}

impl Agent {
    /// How to say it in the one line a picker has room for.
    pub fn note(self) -> Option<&'static str> {
        match self {
            Agent::Unrestricted => None,
            Agent::ReadOnly => Some("read-only"),
            Agent::Propose => Some("by proposal"),
        }
    }
}

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
    pub agent: Agent,
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

/// Whether a type is a kind of work, or a thing work is filed under.
///
/// Declared on the type since format 3. It used to be derived from the other
/// end — a type was a container because some field named it as a `target` —
/// which meant adding a field silently changed another type's behaviour and
/// you could not read the answer off the type at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Groups {
    /// Work belongs to exactly one: a release.
    One,
    /// Work may belong to several: an epic, a theme.
    Many,
}

#[derive(Clone, Debug)]
pub struct ItemType {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    /// Set when this type groups work rather than being some of it.
    pub groups: Option<Groups>,
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

/// How a project writes its identifiers.
///
/// `MP-{n}` gives `MP-1002`, `A{n}` gives `A24`, `{n:04}` gives `0001`. It is
/// a rendering and nothing more: the value under `id` is an unsigned integer
/// whatever this says, so adopting a project key is a display change rather
/// than a change to any file.
///
/// One template rather than three settings for prefix, separator and padding,
/// because a template shows you what it produces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdFormat {
    prefix: String,
    pad: usize,
    suffix: String,
}

impl Default for IdFormat {
    fn default() -> Self {
        IdFormat::padded(4)
    }
}

impl IdFormat {
    /// The older `id_width` spelling: `0001`.
    pub fn padded(width: usize) -> IdFormat {
        IdFormat {
            prefix: String::new(),
            pad: width,
            suffix: String::new(),
        }
    }

    /// `MP-{n}`, `A{n}`, `{n:04}`. An unusable template is an error here and
    /// a warning at the call site: a project whose identifiers are spelled
    /// oddly should still open.
    pub fn compile(template: &str) -> Result<IdFormat, String> {
        let open = template
            .find('{')
            .ok_or_else(|| format!("id_format `{template}` has no `{{n}}`"))?;
        let close = template[open..]
            .find('}')
            .map(|i| open + i)
            .ok_or_else(|| format!("id_format `{template}`: `{{` is never closed"))?;
        let suffix = template[close + 1..].to_string();
        if suffix.contains('{') {
            return Err(format!(
                "id_format `{template}` has more than one placeholder: an item has one identifier"
            ));
        }
        let inner = &template[open + 1..close];
        let pad = match inner {
            "n" => 1,
            _ => inner
                .strip_prefix("n:0")
                .and_then(|d| d.parse::<usize>().ok())
                .ok_or_else(|| {
                    format!("id_format `{template}`: `{{{inner}}}` should be `{{n}}` or `{{n:0W}}`")
                })?,
        };
        Ok(IdFormat {
            prefix: template[..open].to_string(),
            pad: pad.clamp(1, 12),
            suffix,
        })
    }

    pub fn render(&self, id: u32) -> String {
        format!(
            "{}{:0width$}{}",
            self.prefix,
            id,
            self.suffix,
            width = self.pad
        )
    }

    /// An identifier as somebody typed it: rendered, or the bare number, with
    /// or without the `#` that references are printed with.
    ///
    /// Both, because a reader that accepts identifiers as input should accept
    /// both — the number is what the file says and the rendering is what every
    /// screen shows, and a person has no reason to prefer one.
    pub fn parse(&self, text: &str) -> Option<u32> {
        let s = text.trim().trim_start_matches('#').trim();
        let s = s.to_ascii_lowercase();
        let bare = s
            .strip_prefix(self.prefix.to_ascii_lowercase().as_str())
            .and_then(|r| r.strip_suffix(self.suffix.to_ascii_lowercase().as_str()))
            .unwrap_or(&s);
        bare.parse().ok()
    }
}

/// How a reference names what it points at.
///
/// Declared per field, and the reason it has to be declared: a key is not a
/// second identity. Identity is the integer under `id`; a key is a handle,
/// unique only among items of one type and permitted to change. Which of the
/// two a field uses is the project's to say, and a reader that guessed would
/// make `milestone: 42` mean two things depending on what happens to exist.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Addressing {
    /// The identifier. What identity is, and the default.
    #[default]
    Id,
    /// A short handle, which is what keeps `milestone: v0.1` readable.
    Key,
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
    pub by: Addressing,
    pub many: bool,
    /// Whether the reference composes a hierarchy. `milestone` and `part_of` do;
    /// `depends_on` names a prerequisite rather than a parent, and does not.
    pub rollup: bool,
    pub agent: Agent,
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
    /// What the project says this way of looking is grouped by. A view is the
    /// project saying *this is how to look at this*, and its grouping is half
    /// of that.
    pub group_by: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct IdentityIndex {
    ids: BTreeSet<Id>,
    legacy: Option<LegacyMap>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyMap {
    version: u32,
    id_format: String,
    ids: BTreeMap<String, Id>,
}

impl LegacyMap {
    fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported legacy identity map version".into());
        }
        IdFormat::compile(&self.id_format)?;
        let mut targets = BTreeSet::new();
        for (key, id) in &self.ids {
            let n = key
                .parse::<u32>()
                .map_err(|_| "invalid legacy identity map key")?;
            if n.to_string() != *key || !id.is_uuid() || !targets.insert(*id) {
                return Err(
                    "legacy identity map must map canonical numbers to distinct UUIDv4 identities"
                        .into(),
                );
            }
            // Serde's UUID parser accepts other versions; the format does not.
            id.to_string().parse::<Id>()?;
        }
        Ok(())
    }
    fn resolve(&self, raw: &str) -> Option<Id> {
        let n = IdFormat::compile(&self.id_format).ok()?.parse(raw)?;
        self.ids.get(&n.to_string()).copied()
    }
}

#[derive(Clone, Debug)]
pub struct Schema {
    /// The directory holding `cairn.toml`.
    pub root: PathBuf,
    pub name: String,
    pub description: Option<String>,
    /// Where item files live, relative to the root.
    pub dir: PathBuf,
    pub id_format: IdFormat,
    identities: RefCell<IdentityIndex>,
    /// The heading acceptance criteria live under, where the project keeps
    /// them somewhere specific. Absent, every box in a body counts.
    pub criteria_section: Option<String>,
    /// How long a claim may go untouched before the project calls it stale,
    /// in days. Absent, and inert when absent: a project where a claim means
    /// an afternoon and one where it means a quarter are both real, and
    /// neither is harrow's to guess.
    pub claim_stale_after: Option<u32>,
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
        let schema = Schema::parse(&body, root)?;
        let map_path = schema.items_dir().join("_legacy-ids.toml");
        match std::fs::read_to_string(&map_path) {
            Ok(text) => {
                let map: LegacyMap = toml::from_str(&text).map_err(|e| SchemaError::Parse {
                    detail: format!("{}: {e}", map_path.display()),
                })?;
                map.validate()
                    .map_err(|detail| SchemaError::Parse { detail })?;
                schema.identities.borrow_mut().legacy = Some(map);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(SchemaError::Io {
                    path: map_path,
                    detail: e.to_string(),
                });
            }
        }
        Ok(schema)
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
        // A template that will not compile is a warning and the padding is
        // used, rather than a project that refuses to open because its
        // identifiers are spelled oddly.
        let id_format = match project.id_format.as_deref().map(str::trim) {
            Some(t) if !t.is_empty() => IdFormat::compile(t).unwrap_or_else(|why| {
                diag::warn("schema", why);
                IdFormat::padded(project.id_width.unwrap_or(4))
            }),
            _ => IdFormat::padded(project.id_width.unwrap_or(4)),
        };
        let mut statuses: Vec<Status> = file
            .status
            .into_iter()
            .map(|s| Status {
                category: s.category.unwrap_or_default(),
                category_declared: s.category.is_some(),
                agent: s.agent.unwrap_or_default(),
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
                groups: t.groups,
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
                by: match f.by.as_deref().map(str::trim) {
                    Some("key") => Addressing::Key,
                    _ => Addressing::Id,
                },
                many: f.cardinality.as_deref() == Some("many") || f.kind.as_deref() == Some("list"),
                rollup: f.rollup.unwrap_or(false),
                agent: f.agent.unwrap_or_default(),
            })
            .collect();

        // A type that groups work declares the field rather than the file
        // spelling it out: `groups = "one"` on a type called `milestone` is
        // what gives every item a `milestone:` key, by the target's own key,
        // with the rollup and the acyclicity implied. Synthesised here so
        // everything downstream meets one vocabulary.
        let mut fields = fields;
        for kind in &types {
            let Some(groups) = kind.groups else { continue };
            if fields.iter().any(|f| f.name == kind.name) {
                continue;
            }
            fields.push(Field {
                name: kind.name.clone(),
                kind: FieldKind::Ref,
                values: Vec::new(),
                default: None,
                description: kind.description.clone(),
                column: false,
                target: Some(kind.name.clone()),
                // By key, which is the whole point of a grouping type: the
                // field exists so that `milestone: v0.1` reads as itself.
                by: Addressing::Key,
                many: groups == Groups::Many,
                rollup: true,
                agent: Agent::default(),
            });
        }

        let views: Vec<View> = file
            .view
            .into_iter()
            .map(|v| View {
                name: v.name,
                filter: v.filter,
                sort: v.sort,
                group_by: v.group_by.filter(|g| !g.trim().is_empty()),
                description: v.description,
            })
            .collect();

        Ok(Schema {
            name: project.name.unwrap_or_else(|| {
                repository_name(&root).unwrap_or_else(|| {
                    root.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "project".to_string())
                })
            }),
            description: project.description,
            dir: PathBuf::from(project.dir.unwrap_or_else(|| "items".to_string())),
            id_format,
            identities: RefCell::new(IdentityIndex::default()),
            criteria_section: project.criteria_section.filter(|s| !s.trim().is_empty()),
            claim_stale_after: project.claim_stale_after.filter(|d| *d > 0),
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

    /// Types a reference field names specifically — `milestone`, and anything
    /// else a project points a `target` at.
    ///
    /// cairn keeps these out of `next`, the board, the roadmap's item lists and
    /// an ordinary `list`: a milestone is a thing work belongs to rather than a
    /// piece of work, and listing it beside the work it contains reads as a
    /// duplicate. They come back with `--all`, or when asked for by type.
    pub fn is_container(&self, kind: &str) -> bool {
        // Cairn's rule, and deliberately the same shape: *either* the type
        // declares `groups`, *or* some field names it as a specific `target`.
        // Not one or the other by format — both, always. A project part-way
        // through `cairn migrate` satisfies one and not the other, and either
        // answer alone is wrong for somebody.
        //
        // Asking the type first and *returning* what it said is what broke:
        // every type an item can have is declared, so `groups.is_some()` —
        // false — was the answer for every format 2 project and the second
        // rule was never reached. Most projects on disk are still format 2,
        // and in all of them `milestone` quietly stopped being a container:
        // counted as work in the strip, dealt onto the board as a card,
        // tallied by type in the statistics, while the list went on leaving
        // it out. Two true-looking numbers that cannot both be right.
        //
        // `target = "*"` does not make everything a container, which is what
        // stops `depends_on` emptying the backlog.
        self.item_type(kind).is_some_and(|t| t.groups.is_some())
            || self.fields.iter().any(|f| {
                matches!(f.kind, FieldKind::Ref)
                    && f.target.as_deref().is_some_and(|t| t != "*" && t == kind)
            })
    }

    pub fn container_types(&self) -> Vec<&str> {
        self.types
            .iter()
            .map(|t| t.name.as_str())
            .filter(|n| self.is_container(n))
            .collect()
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
    pub fn format_id(&self, id: impl Into<Id>) -> String {
        let id = id.into();
        let Id::Uuid(_) = id else {
            let Id::Legacy(n) = id else { unreachable!() };
            return self.id_format.render(n);
        };
        let compact = id.compact();
        let index = self.identities.borrow();
        for width in 8..=32 {
            let prefix = &compact[..width];
            if !index
                .ids
                .iter()
                .any(|other| *other != id && other.compact().starts_with(prefix))
                && index
                    .legacy
                    .as_ref()
                    .and_then(|m| m.resolve(prefix))
                    .is_none_or(|other| other == id)
            {
                return prefix.to_owned();
            }
        }
        id.to_string()
    }

    pub fn remember_ids(&self, ids: impl IntoIterator<Item = Id>) {
        self.identities.borrow_mut().ids = ids.into_iter().collect();
    }

    /// Git activity names paths, including the filenames retained by migration.
    pub fn id_from_path(&self, path: &Path) -> Option<Id> {
        if let Some(id) = crate::item::id_from_path(path) {
            if id.is_uuid() || self.format < 4 {
                return Some(id);
            }
            return self
                .identities
                .borrow()
                .legacy
                .as_ref()?
                .ids
                .get(&id.to_string())
                .copied();
        }
        let name = path.file_stem()?.to_str()?;
        for (at, _) in name.char_indices().filter(|(_, c)| *c == '-') {
            if let Ok(id) = self.parse_id(&name[..at]) {
                return Some(id);
            }
        }
        None
    }

    pub fn parse_id(&self, raw: &str) -> Result<Id, String> {
        if self.format < 4 {
            return self
                .id_format
                .parse(raw)
                .map(Id::Legacy)
                .ok_or_else(|| format!("invalid item id: {raw}"));
        }
        let raw = raw.trim().trim_start_matches('#').trim();
        if let Ok(id @ Id::Uuid(_)) = raw.parse::<Id>() {
            return Ok(id);
        }
        let index = self.identities.borrow();
        let mut matches = BTreeSet::new();
        if let Some(id) = index.legacy.as_ref().and_then(|map| map.resolve(raw)) {
            matches.insert(id);
        }
        if (8..=32).contains(&raw.len()) && raw.bytes().all(|b| b.is_ascii_hexdigit()) {
            let prefix = raw.to_ascii_lowercase();
            matches.extend(
                index
                    .ids
                    .iter()
                    .filter(|id| id.is_uuid() && id.compact().starts_with(&prefix))
                    .copied(),
            );
        }
        match matches.len() {
            1 => Ok(*matches.first().unwrap()),
            0 => Err(format!(
                "unknown item id {raw}; use a full UUID or an unambiguous prefix of at least 8 hex digits"
            )),
            _ => Err(format!(
                "ambiguous item id {raw}; use a longer prefix or full UUID"
            )),
        }
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
        agent: Agent::default(),
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
    id_format: Option<String>,
    criteria_section: Option<String>,
    claim_stale_after: Option<u32>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TypeFile {
    name: String,
    label: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    description: Option<String>,
    groups: Option<Groups>,
}

#[derive(Debug, Deserialize)]
struct StatusFile {
    name: String,
    label: Option<String>,
    category: Option<Category>,
    agent: Option<Agent>,
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
    by: Option<String>,
    cardinality: Option<String>,
    rollup: Option<bool>,
    agent: Option<Agent>,
}

#[derive(Debug, Deserialize)]
struct ViewFile {
    name: String,
    filter: Option<String>,
    sort: Option<String>,
    group_by: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenderFile {
    target: Option<String>,
}

/// What the repository is called, for a project that did not say.
///
/// The directory is the wrong answer often enough to be worth the walk. This
/// project's own worktrees live at `../.worktrees/<repo>/<type>/<slug>`, so the
/// directory name is a branch slug and two agents on one repository would see
/// two different project names, neither of them the repository.
///
/// Read rather than asked: `git` is a process, and finding out what a project
/// is called is not a reason to start one. Both answers here are files.
fn repository_name(root: &Path) -> Option<String> {
    let git = git_dir(root)?;
    // `origin` first. It is what the project is called where other people
    // refer to it, which is what a name is for.
    if let Ok(config) = fs::read_to_string(git.join("config"))
        && let Some(name) = origin_name(&config)
    {
        return Some(name);
    }
    // Otherwise the directory holding `.git`, which for a worktree is the main
    // checkout rather than the worktree itself.
    git.parent()
        .and_then(|d| d.file_name())
        .map(|n| n.to_string_lossy().to_string())
}

/// The `.git` belonging to `root` or to a directory above it, following the
/// one-line file a worktree has in place of a directory.
fn git_dir(root: &Path) -> Option<PathBuf> {
    let mut at = Some(root);
    while let Some(dir) = at {
        let candidate = dir.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if candidate.is_file() {
            // `gitdir: /path/to/repo/.git/worktrees/<slug>` — the repository's
            // own `.git` is two directories up from there.
            let text = fs::read_to_string(&candidate).ok()?;
            let path = PathBuf::from(text.strip_prefix("gitdir:")?.trim());
            return match path.parent().and_then(|p| p.parent()) {
                Some(main) if main.file_name() == Some(OsStr::new(".git")) => {
                    Some(main.to_path_buf())
                }
                _ => Some(path),
            };
        }
        at = dir.parent();
    }
    None
}

/// The repository name out of a remote URL, whichever way it is written.
/// `git@host:owner/name.git` and `https://host/owner/name` are the same name.
fn origin_name(config: &str) -> Option<String> {
    let mut in_origin = false;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_origin = line.replace(' ', "") == "[remote\"origin\"]";
            continue;
        }
        if !in_origin {
            continue;
        }
        if let Some(url) = line.strip_prefix("url") {
            let url = url
                .trim_start()
                .strip_prefix('=')?
                .trim()
                .trim_end_matches('/');
            let name = url.rsplit(['/', ':']).next()?;
            let name = name.strip_suffix(".git").unwrap_or(name);
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory nobody else is using, removed when the test ends.
    struct Dir(PathBuf);

    impl Dir {
        fn new(tag: &str) -> Dir {
            let at =
                std::env::temp_dir().join(format!("harrow-schema-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&at);
            fs::create_dir_all(&at).expect("a temp dir");
            Dir(at)
        }
        fn at(&self, rel: &str) -> PathBuf {
            self.0.join(rel)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(at: &Path, text: &str) {
        if let Some(parent) = at.parent() {
            fs::create_dir_all(parent).expect("a parent");
        }
        fs::write(at, text).expect("a write");
    }

    const ORIGIN: &str = "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = git@github.com:oddurs/harrow.git\n";

    /// The directory is the wrong answer in a worktree, where it is a branch
    /// slug — so two agents on one repository saw two different project names.
    #[test]
    fn a_worktree_is_named_after_its_repository_not_its_slug() {
        let dir = Dir::new("worktree");
        write(&dir.at("repo/.git/config"), ORIGIN);
        let slug = dir.at("worktrees/harrow/fix/some-slug");
        fs::create_dir_all(&slug).expect("a worktree");
        write(
            &slug.join(".git"),
            &format!(
                "gitdir: {}\n",
                dir.at("repo/.git/worktrees/some-slug").display()
            ),
        );
        assert_eq!(repository_name(&slug).as_deref(), Some("harrow"));
    }

    #[test]
    fn an_ordinary_checkout_is_named_after_its_repository() {
        let dir = Dir::new("checkout");
        write(&dir.at("anything/.git/config"), ORIGIN);
        assert_eq!(
            repository_name(&dir.at("anything")).as_deref(),
            Some("harrow")
        );
    }

    /// A directory inside the repository is still the repository.
    #[test]
    fn a_directory_below_the_root_still_finds_it() {
        let dir = Dir::new("below");
        write(&dir.at("repo/.git/config"), ORIGIN);
        let deep = dir.at("repo/a/b/c");
        fs::create_dir_all(&deep).expect("a deep dir");
        assert_eq!(repository_name(&deep).as_deref(), Some("harrow"));
    }

    /// No remote, so the directory holding `.git` answers instead.
    #[test]
    fn a_repository_with_no_remote_is_named_after_its_own_directory() {
        let dir = Dir::new("noremote");
        write(&dir.at("standalone/.git/config"), "[core]\n");
        assert_eq!(
            repository_name(&dir.at("standalone")).as_deref(),
            Some("standalone")
        );
    }

    #[test]
    fn a_directory_that_is_not_a_repository_has_no_repository_name() {
        let dir = Dir::new("bare");
        fs::create_dir_all(dir.at("plain")).expect("a dir");
        assert_eq!(repository_name(&dir.at("plain")), None);
    }

    /// Both ways a remote is written name the same repository.
    #[test]
    fn a_remote_names_the_same_thing_however_it_is_written() {
        for url in [
            "git@github.com:oddurs/harrow.git",
            "https://github.com/oddurs/harrow.git",
            "https://github.com/oddurs/harrow",
            "https://github.com/oddurs/harrow/",
        ] {
            let config = format!("[remote \"origin\"]\n\turl = {url}\n");
            assert_eq!(origin_name(&config).as_deref(), Some("harrow"), "for {url}");
        }
    }

    /// The bug itself, through the door harrow actually uses. The helpers
    /// above can all be right while the schema goes on reading the folder.
    #[test]
    fn a_project_with_no_name_of_its_own_loads_as_its_repository() {
        let dir = Dir::new("load");
        write(&dir.at("repo/.git/config"), ORIGIN);
        let slug = dir.at("worktrees/harrow/fix/repo-name-over-folder");
        fs::create_dir_all(slug.join("items")).expect("a worktree");
        write(
            &slug.join(".git"),
            &format!(
                "gitdir: {}\n",
                dir.at("repo/.git/worktrees/repo-name-over-folder")
                    .display()
            ),
        );
        // No `name` in it, which is the whole case.
        write(&slug.join("cairn.toml"), "[project]\ndir = \"items\"\n");

        let schema = Schema::load(&slug.join("cairn.toml")).expect("it loads");
        assert_eq!(
            schema.name, "harrow",
            "the branch slug is not the name of the project"
        );
    }

    /// And a name it did say still wins, wherever it is opened.
    #[test]
    fn a_project_that_names_itself_keeps_its_name() {
        let dir = Dir::new("named");
        write(&dir.at("repo/.git/config"), ORIGIN);
        let at = dir.at("repo/somewhere");
        fs::create_dir_all(at.join("items")).expect("a dir");
        write(
            &at.join("cairn.toml"),
            "[project]\nname = \"quarry\"\ndir = \"items\"\n",
        );
        let schema = Schema::load(&at.join("cairn.toml")).expect("it loads");
        assert_eq!(schema.name, "quarry");
    }

    /// Another remote is not the project's name.
    #[test]
    fn only_origin_names_the_project() {
        let config = "[remote \"upstream\"]\n\turl = git@github.com:someone/else.git\n";
        assert_eq!(origin_name(config), None);
    }

    const SAMPLE: &str = r#"
format = 3

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

[[type]]
name = "milestone"
groups = "one"

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
    fn a_project_may_spell_its_identifiers_its_own_way() {
        let f = IdFormat::compile("MP-{n}").expect("compiles");
        assert_eq!(f.render(1002), "MP-1002");
        assert_eq!(f.render(7), "MP-7");
        assert_eq!(
            IdFormat::compile("A{n}").expect("compiles").render(24),
            "A24"
        );
        assert_eq!(
            IdFormat::compile("{n:04}").expect("compiles").render(1),
            "0001"
        );
        // The older spelling means the same thing.
        assert_eq!(IdFormat::padded(4).render(1), "0001");
    }

    /// A reader that accepts identifiers as input should accept both the
    /// rendered form and the bare integer, because one is what the screen
    /// shows and the other is what the file says.
    #[test]
    fn an_identifier_is_accepted_however_it_is_written() {
        let f = IdFormat::compile("MP-{n:04}").expect("compiles");
        for written in ["MP-0042", "42", "#42", " MP-0042 ", "#MP-0042"] {
            assert_eq!(f.parse(written), Some(42), "{written}");
        }
        assert_eq!(f.parse("nonsense"), None);
    }

    #[test]
    fn a_template_that_cannot_work_is_refused_rather_than_guessed_at() {
        for bad in ["no placeholder", "{n", "{n}-{n}", "{nope}"] {
            assert!(IdFormat::compile(bad).is_err(), "{bad}");
        }
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
    fn a_reference_target_makes_a_type_a_container() {
        // cairn keeps these out of an ordinary listing: a milestone is a thing
        // work belongs to rather than a piece of work.
        let s = sample();
        assert!(
            s.is_container("milestone"),
            "the milestone field targets it"
        );
        assert!(!s.is_container("feature"));
        assert_eq!(s.container_types(), vec!["milestone"]);
    }

    #[test]
    fn what_an_agent_may_touch_is_read_from_the_schema() {
        let s = Schema::parse(
            "[[status]]\nname = \"done\"\ncategory = \"done\"\nagent = \"read-only\"\n\
             [[field]]\nname = \"priority\"\nkind = \"enum\"\nagent = \"propose\"\n",
            PathBuf::from("/tmp"),
        )
        .expect("parses");
        assert_eq!(s.status("done").map(|s| s.agent), Some(Agent::ReadOnly));
        assert_eq!(s.field("priority").map(|f| f.agent), Some(Agent::Propose));
        assert_eq!(Agent::ReadOnly.note(), Some("read-only"));
        assert_eq!(Agent::Unrestricted.note(), None, "silence is the default");
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
            "{SAMPLE}\n[[status]]\nname = \"shipped\"\ncategory = \"done\"\nsomething_later = 3\n"
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
