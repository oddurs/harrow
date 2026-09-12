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
    /// Counted once when the set is loaded, because the count depends on the
    /// project's configuration and every row of every frame asks for it.
    pub criteria_met: u32,
    pub criteria_total: u32,
    /// Claimed for longer than the project says a claim should last.
    ///
    /// Refreshed by the frame rather than derived at load, because it depends
    /// on the clock and `engine` has none — deliberately, so that deriving a
    /// backlog is a pure function of the files.
    pub claim_stale: bool,
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
        (self.criteria_met, self.criteria_total)
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
        // Normalised, because this body is only ever drawn. A stray carriage
        // return inside a line is a control character the terminal acts on,
        // and harrow hands `$EDITOR` the path rather than the text, so there
        // is nothing here that owes the file its own line endings back.
        body: if body.contains('\r') {
            body.replace("\r\n", "\n")
        } else {
            body.to_string()
        },
        path: path.to_path_buf(),
        ..Default::default()
    };
    let mut has_id = false;

    for (key, value) in map {
        match key.as_str() {
            "id" => {
                item.id = value
                    .as_str()
                    .parse()
                    .map_err(|_| format!("id {:?} is not a number", value.as_str()))?;
                has_id = true;
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
            "labels" => item.labels = comma_separated(&value),
            "depends_on" => {
                item.depends_on = comma_separated(&value)
                    .iter()
                    // A reference written the way a person writes one. cairn
                    // prints `#12` and somebody will paste it back.
                    .filter_map(|s| s.trim_start_matches('#').trim().parse().ok())
                    .collect();
            }
            _ => {
                item.fields.insert(key, value);
            }
        }
    }

    // An id the frontmatter omits comes from the filename, which is where the
    // format says to look for it. An id that cannot be found in either is not
    // a zero: an item whose identity is unknown is worse than an item that
    // fails to load, because two of them are the same item.
    if !has_id {
        item.id = id_from_filename(path)
            .ok_or_else(|| "no id, and the filename does not begin with one either".to_string())?;
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

/// A sequence, however it was written.
///
/// The format says a reader must accept a single string where a sequence
/// belongs, split on commas with surrounding whitespace discarded — because
/// `labels: auth, backend` is what a person types, and reading it as one label
/// called "auth, backend" is a filter that never matches and a column that
/// never lines up.
fn comma_separated(value: &Value) -> Vec<String> {
    value
        .items()
        .iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Acceptance criteria in a body, as the convention defines them.
///
/// A line counts when, after leading whitespace, it begins with a list marker
/// — `-`, `*` or `+` — a space, a box, and then *something else*. A box with
/// nothing after it is a placeholder rather than a criterion: the templates
/// cairn ships end with a bare `- [ ]` prompting the author to write one, so
/// counting it would leave every item ever created permanently one short of
/// its own criteria. That is the noise that gets a feature switched off.
///
/// A project may confine the count to one section. With none named, the whole
/// body counts — an item that keeps its criteria under a different heading
/// still meant them.
pub fn count_criteria(body: &str, section: Option<&str>) -> (u32, u32) {
    let (mut met, mut total) = (0, 0);
    // Before any heading, with no section named, we are already counting, so
    // a body with no headings at all still works.
    let mut counting = section.is_none();

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix('#') {
            if let Some(want) = section {
                // Any level, and without regard to case: a project that says
                // `Acceptance criteria` should not care whether the item
                // wrote `##` or `###`.
                counting = heading
                    .trim_start_matches('#')
                    .trim()
                    .eq_ignore_ascii_case(want);
            }
            continue;
        }
        if !counting {
            continue;
        }
        let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        else {
            continue;
        };
        let rest = rest.trim_start();
        let (ticked, after) = if let Some(after) = rest.strip_prefix("[ ]") {
            (false, after)
        } else if let Some(after) = rest
            .strip_prefix("[x]")
            .or_else(|| rest.strip_prefix("[X]"))
        {
            (true, after)
        } else {
            continue;
        };
        if !after.starts_with(char::is_whitespace) || after.trim().is_empty() {
            continue;
        }
        total += 1;
        met += u32::from(ticked);
    }
    (met, total)
}

/// The leading run of digits in a filename, which is how cairn names an item
/// and the only part of the name anything is allowed to depend on.
///
/// A project may render its identifiers with a prefix — `MP-1002` — in which
/// case the fallback does not apply and the file has to carry its own id. That
/// is the format's rule rather than a shortcut taken here: the rendering lives
/// in the project's configuration, and an item reader is not required to have
/// read it.
fn id_from_filename(path: &Path) -> Option<u32> {
    let name = path.file_stem()?.to_str()?;
    let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
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
        // Either delimiter closes it. `...` is rarer than `---` and is what a
        // YAML writer emits when it means *the document ends here*; a reader
        // that only knows one of them silently swallows the body.
        if matches!(line.trim_end(), "---" | "...") {
            let body = &rest[offset + line.len()..];
            // The blank line writers leave under the delimiter is part of the
            // punctuation, not of the body. The specification does not say
            // either way — §3 is explicit that two readers may disagree about
            // what a body is — so harrow reads it the way the reference
            // implementation does, because one fewer difference is worth
            // more than the freedom to have this one.
            let body = body
                .strip_prefix("\r\n")
                .or_else(|| body.strip_prefix('\n'));
            return Some((
                &rest[..offset],
                body.unwrap_or(&rest[offset + line.len()..]),
            ));
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

/// Unquote, drop a trailing comment, and resolve what YAML resolves.
///
/// A quoted value is returned exactly as written: quoting is how a writer says
/// *this is a string*, and the whole reason the format tells writers to quote
/// anything that would change meaning.
fn scalar(raw: &str) -> String {
    let s = raw.trim();
    for quote in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(quote) && s.ends_with(quote) {
            return s[1..s.len() - 1].replace(&format!("\\{quote}"), &quote.to_string());
        }
    }
    let s = match s.split_once(" #") {
        Some((before, _)) => before.trim(),
        None => s,
    };
    resolve(s).unwrap_or_else(|| s.to_string())
}

/// An unquoted scalar, under the **YAML 1.2 core schema**.
///
/// The version is the point. 1.1 and 1.2 disagree about exactly the values
/// people write by hand, and several widely used YAML libraries still default
/// to 1.1 — so `0042` is thirty-four under one and forty-two under the other,
/// and `no` is a boolean under one and the word "no" under the other. Naming
/// which one this is makes the disagreement somebody else's bug rather than a
/// silent difference of opinion about what a file says.
///
/// Only the numeric forms are resolved, because they are the only ones whose
/// rendering changes. Under 1.2 core `yes`, `no`, `on` and `off` are strings
/// and `12:30` is a string, which is what leaving them alone already produces.
fn resolve(s: &str) -> Option<String> {
    let (sign, digits) = match s.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, s.strip_prefix('+').unwrap_or(s)),
    };
    if digits.is_empty() {
        return None;
    }

    let radix = |prefix: &str, base: u32| -> Option<String> {
        let body = digits.strip_prefix(prefix)?;
        let n = i64::from_str_radix(body, base).ok()?;
        Some((sign * n).to_string())
    };
    if let Some(n) = radix("0x", 16).or_else(|| radix("0o", 8)) {
        return Some(n);
    }
    if digits.bytes().all(|b| b.is_ascii_digit()) {
        // Leading zeros go, which is the whole difference from 1.1 reading the
        // same text as octal.
        return digits.parse::<i64>().ok().map(|n| (sign * n).to_string());
    }

    // A float, and only in the shapes the core schema calls one: an exponent
    // needs a digit before it, and `.` on its own is not a number.
    let numeric = digits
        .bytes()
        .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-'));
    if !numeric || !digits.bytes().any(|b| b.is_ascii_digit()) {
        return None;
    }
    let value: f64 = digits.parse().ok()?;
    value.is_finite().then(|| (sign as f64 * value).to_string())
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

    /// Everything below the delimiter, less the blank line under it — which
    /// is punctuation rather than body, and is how the reference reads it.
    #[test]
    fn the_body_survives_intact() {
        let i = item();
        assert!(i.body.starts_with("## Problem"), "{:?}", i.body);
        assert!(i.body.ends_with("- [ ] not this one\n"), "{:?}", i.body);
    }

    /// The rule the convention states, including the part that makes every
    /// item created from a template stop reporting a criterion nobody wrote.
    #[test]
    fn a_box_is_a_criterion_only_when_something_follows_it() {
        let body = "- [x] done one\n- [ ] not this one\n- [ ]\n- [ ]   \n";
        assert_eq!(count_criteria(body, None), (1, 2));
    }

    #[test]
    fn every_list_marker_carries_a_criterion() {
        let body = "- [x] dash\n* [x] star\n+ [ ] plus\n  - [X] nested and ticked\n";
        assert_eq!(count_criteria(body, None), (3, 4));
    }

    /// A ticked box in a note is not an acceptance criterion met.
    #[test]
    fn a_project_may_confine_the_count_to_one_section() {
        let body = "## Notes\n\n- [x] not a criterion\n\n### acceptance CRITERIA\n\n                    - [x] one\n- [ ] two\n\n## After\n\n- [x] nor this\n";
        assert_eq!(count_criteria(body, Some("Acceptance criteria")), (1, 2));
        assert_eq!(
            count_criteria(body, None),
            (3, 4),
            "unconfined, all of them"
        );
    }

    #[test]
    fn a_section_that_no_heading_matches_counts_nothing() {
        let body = "## Problem\n\n- [x] a\n";
        assert_eq!(count_criteria(body, Some("Acceptance criteria")), (0, 0));
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

    /// Two items that both came out as zero would be the same item: `by_id`
    /// would keep one of them, and every reference to the other would resolve
    /// to it.
    #[test]
    fn an_item_with_no_id_takes_it_from_the_filename() {
        let i = parse(
            "---\ntitle: The id comes from the filename\nstatus: backlog\n---\n\nBody.\n",
            Path::new("cairn/items/0010-id-from-filename.md"),
        )
        .expect("parses");
        assert_eq!(i.id, 10);
    }

    #[test]
    fn an_id_that_is_in_neither_the_file_nor_its_name_is_refused() {
        let err = parse(
            "---\ntitle: Nameless\nstatus: backlog\n---\n",
            Path::new("cairn/items/notes.md"),
        )
        .expect_err("an item whose identity is unknown does not load");
        assert!(err.contains("no id"), "{err}");
    }

    /// The frontmatter is authoritative when it is there. A file renamed by
    /// hand does not silently renumber the item inside it.
    #[test]
    fn the_frontmatter_wins_over_the_filename() {
        let i = parse(
            "---\nid: 7\ntitle: Seven\n---\n",
            Path::new("cairn/items/0099-seven.md"),
        )
        .expect("parses");
        assert_eq!(i.id, 7);
    }

    /// `...` is what a YAML writer emits when it means the document ends here.
    /// A reader that only knows `---` swallows the whole body as frontmatter.
    #[test]
    fn either_delimiter_closes_the_frontmatter() {
        let i = parse(
            "---\nid: 1\ntitle: T\nstatus: backlog\n...\nThe body.\n",
            Path::new("items/0001-t.md"),
        )
        .expect("parses");
        assert_eq!(i.title, "T");
        assert_eq!(i.body.trim(), "The body.");
    }

    /// What a person types, rather than what cairn writes. One label called
    /// "auth, backend, ops" is a filter that never matches.
    #[test]
    fn a_sequence_written_as_one_string_is_still_a_sequence() {
        let i = parse(
            "---\nid: 1\ntitle: T\nlabels: auth, backend, ops\n---\n",
            Path::new("items/0001-t.md"),
        )
        .expect("parses");
        assert_eq!(i.labels, ["auth", "backend", "ops"]);
    }

    #[test]
    fn a_dependency_is_read_however_it_was_written() {
        for written in [
            "depends_on: 3, 4",
            "depends_on: [#3, 4]",
            "depends_on:\n- '#3'\n- 4",
            "depends_on:\n- 3\n- 4",
        ] {
            let i = parse(
                &format!("---\nid: 1\ntitle: T\n{written}\n---\n"),
                Path::new("items/0001-t.md"),
            )
            .expect("parses");
            assert_eq!(i.depends_on, [3, 4], "{written}");
        }
    }

    /// The core schema, named on purpose: 1.1 reads four of these differently,
    /// and several widely used YAML libraries still default to it.
    #[test]
    fn unquoted_scalars_resolve_under_the_yaml_1_2_core_schema() {
        let i = parse(
            "---\nid: 1\ntitle: T\nassignee: no\nlabels: [12:30, 0x1F, 1.20, 0o17, 0042]\n---\n",
            Path::new("items/0001-t.md"),
        )
        .expect("parses");
        assert_eq!(i.assignee.as_deref(), Some("no"), "not the boolean false");
        assert_eq!(i.labels, ["12:30", "31", "1.2", "15", "42"]);
    }

    /// Quoting is how a writer says *this is a string*, which is the remedy
    /// the format prescribes for exactly these values.
    #[test]
    fn a_quoted_scalar_is_left_exactly_as_written() {
        let i = parse(
            "---\nid: 1\ntitle: T\nlabels: [\"0x1F\", \"1.20\"]\n---\n",
            Path::new("items/0001-t.md"),
        )
        .expect("parses");
        assert_eq!(i.labels, ["0x1F", "1.20"]);
    }

    /// Everything that merely looks numeric and is not. A date resolved as a
    /// number would be a date nothing could sort.
    #[test]
    fn what_is_not_a_number_is_left_alone() {
        let i = parse(
            "---\nid: 1\ntitle: T\ncreated: 2026-09-12\nlabels: [p1, v0.1, s, 1.2.3, -, 12:30]\n---\n",
            Path::new("items/0001-t.md"),
        )
        .expect("parses");
        assert_eq!(i.created.as_deref(), Some("2026-09-12"));
        assert_eq!(i.labels, ["p1", "v0.1", "s", "1.2.3", "-", "12:30"]);
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
