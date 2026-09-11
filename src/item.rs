//! An item, and the file it comes from.
//!
//! An item file is Markdown with YAML frontmatter. harrow reads the files
//! rather than shelling out to `cairn export` for the same reason quarry reads
//! the kernel rather than parsing `lsof`: it is the same information, it costs a
//! millisecond instead of a process, and it keeps working when the tool that
//! wrote it is not installed. Writes still go through `cairn`, which owns the
//! locking, the hooks and the renumbering — reading is the half that is safe to
//! do directly.
//!
//! The parser covers the subset of YAML cairn writes: `key: value`, block
//! sequences, and flow sequences. Anything else is kept as text rather than
//! guessed at, so an unusual file loses a field instead of failing to open.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::schema::{Category, Schema};

/// A frontmatter value. cairn writes scalars and lists; nothing nests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    One(String),
    Many(Vec<String>),
}

impl Value {
    pub fn as_str(&self) -> &str {
        match self {
            Value::One(s) => s,
            Value::Many(v) => v.first().map(String::as_str).unwrap_or(""),
        }
    }

    pub fn items(&self) -> Vec<&str> {
        match self {
            Value::One(s) => vec![s.as_str()],
            Value::Many(v) => v.iter().map(String::as_str).collect(),
        }
    }

    /// How the value reads in a single cell.
    pub fn display(&self) -> String {
        match self {
            Value::One(s) => s.clone(),
            Value::Many(v) => v.join(", "),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Value::One(s) => s.is_empty(),
            Value::Many(v) => v.is_empty(),
        }
    }
}

/// A change somebody asked for and a person decides.
///
/// cairn writes it into the body as a section, which is why harrow can read it
/// the same way it reads everything else: it is part of the record, not a
/// sidecar. Accepting it is still cairn's to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub field: String,
    pub from: String,
    pub to: String,
    pub by: String,
    pub when: String,
    pub why: String,
}

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub id: u32,
    pub key: Option<String>,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub claimed: Option<String>,
    /// Who is working on it. `cairn claim` sets this.
    pub assignee: Option<String>,
    /// Who is answerable for it, which with an agent working is somebody else.
    pub owner: Option<String>,
    /// Set only when an agent filed it; a person filing something is the
    /// ordinary case and leaves no mark.
    pub created_by: Option<String>,
    pub labels: Vec<String>,
    pub depends_on: Vec<u32>,
    /// Everything the schema calls a field, plus anything else the file
    /// carried. Unrecognised keys are kept: a field harrow does not know is
    /// still a field the reader wants to see.
    pub fields: BTreeMap<String, Value>,
    pub body: String,
    pub path: PathBuf,

    // ── Derived once, when the set is loaded ─────────────────────────────────
    pub category: Category,
    /// Waiting on something unfinished.
    pub blocked: bool,
    pub blockers: Vec<u32>,
    /// Items naming this one in `milestone` or `part_of`, and how many of them
    /// are finished. What makes a milestone row carry a progress bar.
    pub scheduled: u32,
    pub scheduled_done: u32,
    /// The items directly under this one, by id.
    pub contains: Vec<u32>,
    /// How far below a root of the composition graph this sits.
    pub depth: u32,
    /// Changes waiting for a person to decide.
    pub proposals: Vec<Proposal>,
    /// True when a reference field names this item's type, so it is a thing
    /// work belongs to rather than a piece of work. Kept on the item because
    /// every listing has to ask.
    pub container: bool,
}

impl Item {
    /// `0042`, as the filename spells it.
    pub fn reference(&self, schema: &Schema) -> String {
        schema.format_id(self.id)
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn field_str(&self, name: &str) -> Option<&str> {
        self.fields
            .get(name)
            .map(Value::as_str)
            .filter(|s| !s.is_empty())
    }

    pub fn milestone(&self) -> Option<&str> {
        self.field_str("milestone")
    }

    pub fn is_milestone(&self) -> bool {
        self.kind == "milestone"
    }

    /// Nothing belongs to it. A leaf is ordinary work.
    pub fn is_leaf(&self) -> bool {
        self.contains.is_empty()
    }

    /// Ready to start: open, and nothing unfinished in its way. The same
    /// question `cairn next` answers.
    pub fn ready(&self, schema: &Schema) -> bool {
        !self.blocked && schema.category(&self.status) == Category::Open
    }

    /// Percent finished, for anything with items scheduled against it.
    pub fn progress(&self) -> Option<u32> {
        if self.scheduled == 0 {
            return None;
        }
        Some(self.scheduled_done * 100 / self.scheduled)
    }

    /// Checkbox lines in the body — cairn's acceptance criteria.
    pub fn criteria(&self) -> (u32, u32) {
        let mut done = 0;
        let mut total = 0;
        for line in self.body.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("- [") {
                match rest.as_bytes().first() {
                    Some(b']') => continue,
                    Some(b' ') => total += 1,
                    Some(_) => {
                        total += 1;
                        done += 1;
                    }
                    None => continue,
                }
            }
        }
        (done, total)
    }

    /// The haystack a free-text filter searches. Body included: `oauth` should
    /// find the item that discusses oauth, not only the one titled after it.
    pub fn matches(&self, needle: &str, schema: &Schema) -> bool {
        if needle.is_empty() {
            return true;
        }
        let needle = needle.to_lowercase();
        self.title.to_lowercase().contains(&needle)
            || self.reference(schema).contains(&needle)
            || self.id.to_string() == needle
            || self.kind.contains(&needle)
            || self.status.contains(&needle)
            || self
                .key
                .as_deref()
                .is_some_and(|k| k.to_lowercase().contains(&needle))
            || self
                .labels
                .iter()
                .any(|l| l.to_lowercase().contains(&needle))
            || self
                .fields
                .values()
                .any(|v| v.display().to_lowercase().contains(&needle))
            || self.body.to_lowercase().contains(&needle)
    }
}

/// Read one item file.
pub fn parse(text: &str, path: &Path) -> Result<Item, String> {
    let (front, body) = split(text).ok_or_else(|| "no YAML frontmatter".to_string())?;
    let map = parse_frontmatter(front);

    let mut item = Item {
        body: body.to_string(),
        path: path.to_path_buf(),
        ..Default::default()
    };

    for (key, value) in map {
        match key.as_str() {
            "id" => {
                item.id = value
                    .as_str()
                    .parse()
                    .map_err(|_| format!("id {:?} is not a number", value.as_str()))?;
            }
            "title" => item.title = value.as_str().to_string(),
            "type" => item.kind = value.as_str().to_string(),
            "status" => item.status = value.as_str().to_string(),
            "key" => item.key = non_empty(value.as_str()),
            "created" => item.created = non_empty(value.as_str()),
            "updated" => item.updated = non_empty(value.as_str()),
            "claimed" => item.claimed = non_empty(value.as_str()),
            "assignee" => item.assignee = non_empty(value.as_str()),
            "owner" => item.owner = non_empty(value.as_str()),
            "created_by" => item.created_by = non_empty(value.as_str()),
            "labels" => {
                item.labels = value.items().iter().map(|s| s.to_string()).collect();
            }
            "depends_on" => {
                item.depends_on = value
                    .items()
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();
            }
            _ => {
                item.fields.insert(key, value);
            }
        }
    }

    item.proposals = parse_proposals(&item.body);

    if item.title.is_empty() {
        item.title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("item {}", item.id));
    }
    Ok(item)
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty() && s != "null" && s != "~").then(|| s.to_string())
}

/// Pull the proposals out of a body.
///
/// cairn writes `## Proposed <field>: <from> -> <to> (<who>, <date>)` and the
/// reason underneath. Anything that does not match that shape is prose that
/// happens to start with the word, and is left alone.
fn parse_proposals(body: &str) -> Vec<Proposal> {
    let mut out = Vec::new();
    let mut lines = body.lines().peekable();

    while let Some(line) = lines.next() {
        let Some(rest) = line.trim().strip_prefix("## Proposed ") else {
            continue;
        };
        let Some((field, rest)) = rest.split_once(": ") else {
            continue;
        };
        let Some((change, who)) = rest.rsplit_once(" (") else {
            continue;
        };
        let Some((from, to)) = change.split_once(" -> ") else {
            continue;
        };
        let who = who.trim_end_matches(')');
        let (by, when) = who.rsplit_once(", ").unwrap_or((who, ""));

        // The reason is whatever follows, up to the next heading.
        let mut why = String::new();
        while let Some(next) = lines.peek() {
            if next.trim_start().starts_with("## ") {
                break;
            }
            let next = lines.next().unwrap_or_default().trim();
            if next.is_empty() {
                if why.is_empty() {
                    continue;
                }
            } else if !why.is_empty() {
                why.push(' ');
            }
            why.push_str(next);
        }

        out.push(Proposal {
            field: field.trim().to_string(),
            from: from.trim().to_string(),
            to: to.trim().to_string(),
            by: by.trim().to_string(),
            when: when.trim().to_string(),
            why: why.trim().to_string(),
        });
    }
    out
}

/// Split `---\n…\n---\n` off the front. Tolerates a leading byte-order mark and
/// CRLF, both of which arrive from editors on other platforms.
fn split(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some((&rest[..offset], &rest[offset + line.len()..]));
        }
        offset += line.len();
    }
    // A file with an opening fence and no closing one is frontmatter all the
    // way down, which is what a half-written item looks like.
    Some((rest, ""))
}

/// The subset of YAML cairn writes. Deliberately not a YAML parser: this reads
/// what the format specifies and keeps everything else as text.
fn parse_frontmatter(front: &str) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = Vec::new();
    let mut lines = front.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() || trimmed.trim_start().starts_with('#') {
            continue;
        }
        // A continuation line with no key of its own belongs to nothing we can
        // attach it to; skipping beats inventing a key.
        let Some((key, rest)) = trimmed.split_once(':') else {
            continue;
        };
        if key.starts_with(char::is_whitespace) || key.starts_with('-') {
            continue;
        }
        let key = key.trim().to_string();
        let rest = rest.trim();

        if rest.is_empty() {
            // A block sequence: the indented `- value` lines that follow.
            let mut values = Vec::new();
            while let Some(next) = lines.peek() {
                let t = next.trim_start();
                if !t.starts_with("- ") && t != "-" {
                    break;
                }
                let value = scalar(t.trim_start_matches('-').trim());
                if !value.is_empty() {
                    values.push(value);
                }
                lines.next();
            }
            out.push((
                key,
                if values.is_empty() {
                    Value::One(String::new())
                } else {
                    Value::Many(values)
                },
            ));
            continue;
        }

        if let Some(inner) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            let values: Vec<String> = inner
                .split(',')
                .map(|p| scalar(p.trim()))
                .filter(|p| !p.is_empty())
                .collect();
            out.push((key, Value::Many(values)));
            continue;
        }

        out.push((key, Value::One(scalar(rest))));
    }
    out
}

/// Unquote, and drop a trailing comment on an unquoted value.
fn scalar(raw: &str) -> String {
    let s = raw.trim();
    for quote in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(quote) && s.ends_with(quote) {
            return s[1..s.len() - 1].replace(&format!("\\{quote}"), &quote.to_string());
        }
    }
    match s.split_once(" #") {
        Some((before, _)) => before.trim().to_string(),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: &str = "---\n\
id: 22\n\
title: Draw the rows on screen, not the rows on the machine\n\
type: feature\n\
status: done\n\
milestone: v0.3\n\
depends_on:\n\
- 10\n\
- 11\n\
labels: [chrome, perf]\n\
created: 2026-09-08\n\
priority: p1\n\
---\n\
\n\
## Problem\n\
\n\
- [x] done one\n\
- [ ] not this one\n";

    fn item() -> Item {
        parse(ITEM, Path::new("cairn/items/0022-draw-the-rows.md")).expect("the sample parses")
    }

    #[test]
    fn the_fields_cairn_writes_all_arrive() {
        let i = item();
        assert_eq!(i.id, 22);
        assert_eq!(i.kind, "feature");
        assert_eq!(i.status, "done");
        assert_eq!(i.milestone(), Some("v0.3"));
        assert_eq!(i.depends_on, vec![10, 11]);
        assert_eq!(i.labels, vec!["chrome", "perf"]);
        assert_eq!(i.field_str("priority"), Some("p1"));
    }

    #[test]
    fn a_proposal_is_read_out_of_the_body() {
        let i = parse(
            "---\nid: 5\ntitle: A thing\npriority: p2\n---\n\n\
             ## Problem\n\nSomething.\n\n\
             ## Proposed priority: p2 -> p0 (Oddur Sigurdsson, 2026-09-11)\n\n\
             it blocks the release\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.proposals.len(), 1);
        let p = &i.proposals[0];
        assert_eq!(p.field, "priority");
        assert_eq!((p.from.as_str(), p.to.as_str()), ("p2", "p0"));
        assert_eq!(p.by, "Oddur Sigurdsson");
        assert_eq!(p.when, "2026-09-11");
        assert_eq!(p.why, "it blocks the release");
    }

    #[test]
    fn prose_that_happens_to_start_with_the_word_is_not_a_proposal() {
        let i = parse(
            "---\nid: 5\ntitle: t\n---\n\n## Proposal\n\n## Proposed approach\n\nwords\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert!(i.proposals.is_empty(), "{:?}", i.proposals);
    }

    #[test]
    fn a_reason_stops_at_the_next_heading() {
        let i = parse(
            "---\nid: 5\ntitle: t\n---\n\n\
             ## Proposed status: backlog -> doing (an agent, 2026-09-11)\n\n\
             because it is started\n\n## Notes\n\nnot the reason\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.proposals[0].why, "because it is started");
    }

    #[test]
    fn a_title_may_contain_a_colon() {
        let i = parse(
            "---\nid: 1\ntitle: Fix this: properly\n---\nbody\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.title, "Fix this: properly");
    }

    #[test]
    fn the_body_survives_intact() {
        let i = item();
        assert!(i.body.starts_with("\n## Problem"), "{:?}", i.body);
        assert_eq!(i.criteria(), (1, 2));
    }

    #[test]
    fn an_unknown_key_is_kept_rather_than_dropped() {
        let i = parse(
            "---\nid: 1\ntitle: t\nsomething_new: yes\n---\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.field_str("something_new"), Some("yes"));
    }

    #[test]
    fn quotes_and_comments_come_off_scalars() {
        let i = parse(
            "---\nid: 1\ntitle: \"Quoted, with a comma\"\narea: chrome # the drawing\n---\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.title, "Quoted, with a comma");
        assert_eq!(i.field_str("area"), Some("chrome"));
    }

    #[test]
    fn a_file_with_no_frontmatter_is_reported_not_guessed_at() {
        let err = parse("# just markdown\n", Path::new("x.md")).unwrap_err();
        assert!(err.contains("frontmatter"), "{err}");
    }

    #[test]
    fn a_half_written_item_still_opens() {
        // No closing fence — what a file looks like mid-edit.
        let i = parse("---\nid: 3\ntitle: Half\n", Path::new("x.md")).expect("parses");
        assert_eq!(i.id, 3);
        assert_eq!(i.title, "Half");
    }

    #[test]
    fn an_item_with_no_title_falls_back_to_its_filename() {
        let i = parse(
            "---\nid: 9\n---\n",
            Path::new("cairn/items/0009-a-thing.md"),
        )
        .expect("parses");
        assert_eq!(i.title, "0009-a-thing");
    }

    #[test]
    fn crlf_and_a_byte_order_mark_are_tolerated() {
        let i = parse(
            "\u{feff}---\r\nid: 4\r\ntitle: Windows\r\n---\r\nbody\r\n",
            Path::new("x.md"),
        )
        .expect("parses");
        assert_eq!(i.id, 4);
        assert_eq!(i.title, "Windows");
    }

    #[test]
    fn an_id_that_is_not_a_number_is_an_error_not_a_zero() {
        // Silently reading it as item 0 would collide with every other broken
        // file in the directory.
        let err = parse("---\nid: banana\ntitle: t\n---\n", Path::new("x.md")).unwrap_err();
        assert!(err.contains("banana"), "{err}");
    }

    #[test]
    fn searching_covers_the_body_as_well_as_the_title() {
        let schema = crate::schema::Schema::parse("[project]\nname=\"p\"\n", PathBuf::from("/tmp"))
            .expect("schema");
        let i = item();
        assert!(i.matches("rows", &schema), "title");
        assert!(i.matches("problem", &schema), "body");
        assert!(i.matches("0022", &schema), "reference");
        assert!(!i.matches("nothing here", &schema));
    }
}
