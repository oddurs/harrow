//! cairn's filter grammar, and the sort keys that go with it.
//!
//! Implemented here rather than delegated to `cairn list --filter` because the
//! filter box runs on every keystroke, and a process per keystroke is a
//! different program. The grammar is cairn's, deliberately: an expression that
//! works on the command line has to work in the box, or there are two grammars
//! and one of them is wrong.
//!
//! One addition. A clause with no operator in it is a free-text search, so `/`
//! is useful before you have learned any of this.

use crate::item::Item;
use crate::schema::Schema;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Eq,
    Ne,
    Contains,
    NotContains,
    Gt,
    Ge,
    Lt,
    Le,
}

impl Op {
    fn parse(clause: &str) -> Option<(usize, usize, Op)> {
        // Longest first: `!=` must not be read as `=` with a stray `!`.
        const OPS: &[(&str, Op)] = &[
            ("!=", Op::Ne),
            ("!~", Op::NotContains),
            (">=", Op::Ge),
            ("<=", Op::Le),
            ("=", Op::Eq),
            ("~", Op::Contains),
            (">", Op::Gt),
            ("<", Op::Lt),
        ];
        let mut best: Option<(usize, usize, Op)> = None;
        for (text, op) in OPS {
            if let Some(at) = clause.find(text) {
                let candidate = (at, at + text.len(), *op);
                // The earliest operator wins, and the longest at that position.
                if best.is_none_or(|(b, e, _)| at < b || (at == b && candidate.1 > e)) {
                    best = Some(candidate);
                }
            }
        }
        best
    }
}

#[derive(Clone, Debug)]
enum Clause {
    /// `field OP value|value`.
    Compare {
        field: String,
        op: Op,
        values: Vec<String>,
    },
    /// A bare word: everything a reader would expect it to match.
    Text(String),
}

/// A parsed filter. Empty matches everything, which is what an empty box means.
#[derive(Clone, Debug, Default)]
pub struct Query {
    clauses: Vec<Clause>,
    /// Fields named in the expression that the project does not have. Reported
    /// in the box rather than silently matching nothing.
    pub unknown: Vec<String>,
}

impl Query {
    pub fn parse(text: &str, schema: &Schema) -> Query {
        let mut query = Query::default();
        for clause in text.split(',') {
            let clause = clause.trim();
            if clause.is_empty() {
                continue;
            }
            match Op::parse(clause) {
                Some((start, end, op)) if start > 0 => {
                    let field = clause[..start].trim().to_lowercase();
                    let values: Vec<String> = clause[end..]
                        .split('|')
                        .map(|v| v.trim().to_string())
                        .collect();
                    if !known_field(&field, schema) {
                        query.unknown.push(field.clone());
                    }
                    query.clauses.push(Clause::Compare { field, op, values });
                }
                _ => query.clauses.push(Clause::Text(clause.to_lowercase())),
            }
        }
        query
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Clauses are ANDed; alternatives within a clause are ORed.
    pub fn matches(&self, item: &Item, schema: &Schema) -> bool {
        self.clauses.iter().all(|clause| match clause {
            Clause::Text(needle) => item.matches(needle, schema),
            Clause::Compare { field, op, values } => {
                let actual = resolve(item, schema, field);
                values.iter().any(|v| compare(&actual, *op, v))
            }
        })
    }
}

/// What a key resolves to. `Missing` is not the empty string: `milestone=`
/// tests for absence, and an item whose milestone is literally empty is absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Missing,
    Text(String),
    List(Vec<String>),
}

/// Keys answered without looking at the frontmatter.
pub const DERIVED: &[&str] = &[
    "id",
    "ref",
    "key",
    "title",
    "type",
    "status",
    "category",
    "closed",
    "done",
    "blocked",
    "ready",
    "blockers",
    "labels",
    "assignee",
    "owner",
    "claimed",
    "created",
    "updated",
    "body",
    "progress",
    "criteria",
    "criteria_done",
    "criteria_met",
];

fn known_field(name: &str, schema: &Schema) -> bool {
    DERIVED.contains(&name) || schema.field(name).is_some()
}

pub fn resolve(item: &Item, schema: &Schema, key: &str) -> Field {
    let text = |s: String| {
        if s.is_empty() {
            Field::Missing
        } else {
            Field::Text(s)
        }
    };
    match key {
        "id" => Field::Text(item.id.to_string()),
        "ref" => Field::Text(item.reference(schema)),
        "key" => item.key.clone().map(Field::Text).unwrap_or(Field::Missing),
        "title" => Field::Text(item.title.clone()),
        "type" => Field::Text(item.kind.clone()),
        "status" => Field::Text(item.status.clone()),
        "category" => Field::Text(item.category.name().to_string()),
        "closed" | "done" => Field::Text(item.category.is_closed().to_string()),
        "blocked" => Field::Text(item.blocked.to_string()),
        "ready" => Field::Text(item.ready(schema).to_string()),
        "blockers" => Field::List(item.blockers.iter().map(u32::to_string).collect()),
        "labels" => {
            if item.labels.is_empty() {
                Field::Missing
            } else {
                Field::List(item.labels.clone())
            }
        }
        "assignee" => item
            .assignee
            .clone()
            .map(Field::Text)
            .unwrap_or(Field::Missing),
        "owner" => item
            .owner
            .clone()
            .map(Field::Text)
            .unwrap_or(Field::Missing),
        "claimed" => item
            .claimed
            .clone()
            .map(Field::Text)
            .unwrap_or(Field::Missing),
        "created" => item
            .created
            .clone()
            .map(Field::Text)
            .unwrap_or(Field::Missing),
        "updated" => item
            .updated
            .clone()
            .map(Field::Text)
            .unwrap_or(Field::Missing),
        "body" => text(item.body.clone()),
        // Missing rather than zero for a leaf: an item containing nothing has
        // no progress to report, and zero would sort every one of them last.
        "progress" => match item.progress() {
            Some(p) => Field::Text(p.to_string()),
            None => Field::Missing,
        },
        "criteria" => Field::Text(item.criteria().1.to_string()),
        "criteria_done" => Field::Text(item.criteria().0.to_string()),
        "criteria_met" => {
            let (done, total) = item.criteria();
            Field::Text((total > 0 && done == total).to_string())
        }
        other => match item.fields.get(other) {
            Some(crate::item::Value::One(s)) => text(s.clone()),
            Some(crate::item::Value::Many(v)) if v.is_empty() => Field::Missing,
            Some(crate::item::Value::Many(v)) => Field::List(v.clone()),
            None => Field::Missing,
        },
    }
}

fn compare(actual: &Field, op: Op, wanted: &str) -> bool {
    // An empty value tests presence: `milestone=` has none, `milestone!=` has one.
    if wanted.is_empty() {
        let missing = *actual == Field::Missing;
        return match op {
            Op::Eq => missing,
            Op::Ne => !missing,
            _ => false,
        };
    }
    let values: Vec<&str> = match actual {
        Field::Missing => Vec::new(),
        Field::Text(s) => vec![s.as_str()],
        Field::List(v) => v.iter().map(String::as_str).collect(),
    };
    match op {
        Op::Eq => values.iter().any(|v| v.eq_ignore_ascii_case(wanted)),
        Op::Ne => !values.iter().any(|v| v.eq_ignore_ascii_case(wanted)),
        Op::Contains => values.iter().any(|v| contains(v, wanted)),
        Op::NotContains => !values.iter().any(|v| contains(v, wanted)),
        Op::Gt | Op::Ge | Op::Lt | Op::Le => values.iter().any(|v| ordered(v, op, wanted)),
    }
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Numeric when both sides are numbers, lexical otherwise — which is why ISO
/// dates compare correctly without a date type.
fn ordered(actual: &str, op: Op, wanted: &str) -> bool {
    let order = match (actual.parse::<f64>(), wanted.parse::<f64>()) {
        (Ok(a), Ok(b)) => a.partial_cmp(&b),
        _ => Some(actual.to_lowercase().cmp(&wanted.to_lowercase())),
    };
    let Some(order) = order else { return false };
    match op {
        Op::Gt => order.is_gt(),
        Op::Ge => order.is_ge(),
        Op::Lt => order.is_lt(),
        Op::Le => order.is_le(),
        _ => false,
    }
}

/// One sort key. `-` sorts descending; empty values sort last either way, so a
/// backlog does not open on the items nobody has filled in.
#[derive(Clone, Debug)]
pub struct SortKey {
    pub field: String,
    pub descending: bool,
}

pub fn parse_sort(spec: &str) -> Vec<SortKey> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| match s.strip_prefix('-') {
            Some(rest) => SortKey {
                field: rest.trim().to_lowercase(),
                descending: true,
            },
            None => SortKey {
                field: s.to_lowercase(),
                descending: false,
            },
        })
        .collect()
}

/// A comparable rendering of one key for one item. Status sorts by the declared
/// order and enums by their declared values, so `p0` comes before `p1` because
/// the schema says so rather than because it happens to sort that way.
pub fn sort_value(item: &Item, schema: &Schema, key: &str) -> (u8, u64, String) {
    // The leading flag is the "empty sorts last" rank; nothing else needs to
    // know about it.
    match key {
        "id" => (0, item.id as u64, String::new()),
        "status" => (0, schema.status_index(&item.status) as u64, String::new()),
        "type" => (
            0,
            schema
                .types
                .iter()
                .position(|t| t.name == item.kind)
                .unwrap_or(usize::MAX) as u64,
            String::new(),
        ),
        _ => {
            if let Some(field) = schema.field(key)
                && !field.values.is_empty()
            {
                return match item.field_str(key) {
                    Some(v) => (0, field.rank(v) as u64, String::new()),
                    None => (1, u64::MAX, String::new()),
                };
            }
            match resolve(item, schema, key) {
                Field::Missing => (1, u64::MAX, String::new()),
                Field::Text(s) => match s.parse::<u64>() {
                    Ok(n) => (0, n, String::new()),
                    Err(_) => (0, 0, s.to_lowercase()),
                },
                Field::List(v) => (0, 0, v.join(",").to_lowercase()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    fn matching(expr: &str) -> Vec<u32> {
        let report = testkit::report();
        let query = Query::parse(expr, &report.schema);
        report
            .items
            .iter()
            .filter(|i| query.matches(i, &report.schema))
            .map(|i| i.id)
            .collect()
    }

    #[test]
    fn an_empty_filter_matches_everything() {
        assert_eq!(matching("").len(), 6);
        assert!(Query::parse("", &testkit::schema()).is_empty());
    }

    #[test]
    fn equality_resolves_through_the_status_table() {
        assert_eq!(matching("status=doing"), vec![3]);
        assert_eq!(matching("category=active"), vec![3]);
        assert_eq!(matching("category=done"), vec![2]);
    }

    #[test]
    fn clauses_and_alternatives_combine_the_way_cairn_says() {
        assert_eq!(matching("priority=p0|p1"), vec![2, 3, 5]);
        assert_eq!(matching("priority=p0|p1,category!=done"), vec![3, 5]);
    }

    #[test]
    fn an_empty_value_tests_presence() {
        // The milestone itself has none, which is the correct answer and a
        // reminder that a milestone is an item like any other.
        assert_eq!(matching("milestone="), vec![1, 6]);
        assert_eq!(matching("milestone!=").len(), 4);
    }

    #[test]
    fn contains_searches_inside_a_value_and_a_list() {
        assert_eq!(matching("title~board"), vec![4]);
        assert_eq!(matching("labels~chrome"), vec![5]);
        assert_eq!(matching("body~frontmatter"), vec![2]);
    }

    #[test]
    fn ordered_comparison_is_numeric_for_numbers_and_lexical_for_dates() {
        assert_eq!(matching("created<2026-09-02"), vec![1, 2]);
        assert_eq!(matching("id>=5"), vec![5, 6]);
    }

    #[test]
    fn the_dependency_graph_is_queryable() {
        assert_eq!(matching("blocked=true"), vec![4]);
        assert_eq!(matching("ready=true"), vec![1, 5, 6]);
    }

    #[test]
    fn a_bare_word_is_a_free_text_search() {
        assert_eq!(matching("board"), vec![4]);
        assert_eq!(matching("0003"), vec![3]);
    }

    #[test]
    fn a_field_the_project_does_not_have_is_reported() {
        let query = Query::parse("sprint=3", &testkit::schema());
        assert_eq!(query.unknown, vec!["sprint"]);
        // And it still parses, so the box does not simply stop working while
        // you are halfway through typing something else.
        assert!(!query.is_empty());
    }

    #[test]
    fn half_typed_expressions_do_not_misbehave() {
        for partial in ["status", "status=", "status=d", "=", "~", "a,", ",,"] {
            let query = Query::parse(partial, &testkit::schema());
            let report = testkit::report();
            let _ = report
                .items
                .iter()
                .filter(|i| query.matches(i, &report.schema))
                .count();
        }
    }

    #[test]
    fn sorting_follows_the_declared_order_not_the_alphabet() {
        let schema = testkit::schema();
        let report = testkit::report();
        let by_id = |id: u32| report.items.iter().find(|i| i.id == id).unwrap().clone();
        let p0 = sort_value(&by_id(2), &schema, "priority");
        let p1 = sort_value(&by_id(3), &schema, "priority");
        assert!(p0 < p1, "p0 must come before p1");

        let doing = sort_value(&by_id(3), &schema, "status");
        let done = sort_value(&by_id(2), &schema, "status");
        assert!(doing < done, "the status column order is the sort order");
    }

    #[test]
    fn an_item_with_nothing_in_the_field_sorts_last() {
        let schema = testkit::schema();
        let report = testkit::report();
        let unset = report.items.iter().find(|i| i.id == 1).unwrap();
        let set = report.items.iter().find(|i| i.id == 2).unwrap();
        assert!(sort_value(set, &schema, "priority") < sort_value(unset, &schema, "priority"));
    }

    #[test]
    fn sort_keys_parse_with_their_direction() {
        let keys = parse_sort("priority,-updated");
        assert_eq!(keys.len(), 2);
        assert!(!keys[0].descending);
        assert!(keys[1].descending);
        assert_eq!(keys[1].field, "updated");
    }
}
