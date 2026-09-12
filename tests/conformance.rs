//! harrow's reader, held to cairn's own corpus.
//!
//! harrow reads cairn's files directly rather than shelling out, which makes
//! it a second implementation of somebody else's format. Every other test here
//! was written by the same person who wrote the parser, against the same
//! reading of the same prose — which proves the parser agrees with itself.
//!
//! This one does not. `tests/fixtures/conformance` is the corpus the reference
//! implementation holds itself to, vendored unmodified, and it deliberately
//! contains files a writer would not produce: a bare string where a sequence
//! belongs, a missing id, CRLF endings, keys from a version that does not
//! exist. Those are what people, editors and other tools write.
//!
//! `format-N/` holds items as that format wrote them, with the values they
//! parsed to then, so a project written by an older cairn still opens.

use std::path::{Path, PathBuf};

use serde_json::Value as Json;

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/conformance")
}

/// Every `NAME.md` with a `NAME.json` beside it, at any depth.
fn cases(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            cases(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md")
            && path.with_extension("json").exists()
        {
            out.push(path);
        }
    }
}

fn text(v: &Json) -> Option<&str> {
    v.as_str()
}

/// A key the expectation does not carry is a key that format had not invented
/// yet, which is the whole reason the per-format corpora exist.
fn expected<'a>(want: &'a Json, key: &str) -> Option<&'a Json> {
    want.get(key).filter(|v| !v.is_null())
}

#[test]
fn every_item_in_cairns_corpus_reads_the_way_cairn_says_it_does() {
    let root = corpus();
    let mut found = Vec::new();
    cases(&root, &mut found);
    assert!(
        found.len() >= 30,
        "the corpus looks truncated: {} cases",
        found.len()
    );

    for path in found {
        let name = path.strip_prefix(&root).unwrap_or(&path).display();
        let source = std::fs::read_to_string(&path).expect("read the item");
        let want: Json = serde_json::from_str(
            &std::fs::read_to_string(path.with_extension("json")).expect("read the expectation"),
        )
        .expect("the expectation is JSON");

        let got = harrow::item::parse(&source, &path).unwrap_or_else(|e| panic!("{name}: {e}"));

        assert_eq!(Some(u64::from(got.id)), want["id"].as_u64(), "{name}: id");
        assert_eq!(
            Some(got.title.as_str()),
            text(&want["title"]),
            "{name}: title"
        );
        assert_eq!(
            got.body,
            text(&want["body"]).unwrap_or_default(),
            "{name}: body"
        );

        // The optional scalars, each absent from the expectation when that
        // format had no such key.
        for (key, got) in [
            (
                "status",
                Some(got.status.as_str()).filter(|s| !s.is_empty()),
            ),
            ("type", Some(got.kind.as_str()).filter(|s| !s.is_empty())),
            ("key", got.key.as_deref()),
            ("created", got.created.as_deref()),
            ("updated", got.updated.as_deref()),
            ("claimed", got.claimed.as_deref()),
            ("assignee", got.assignee.as_deref()),
            ("owner", got.owner.as_deref()),
            ("created_by", got.created_by.as_deref()),
        ] {
            assert_eq!(got, expected(&want, key).and_then(text), "{name}: {key}");
        }

        let labels: Vec<&str> = expected(&want, "labels")
            .and_then(Json::as_array)
            .map(|a| a.iter().filter_map(text).collect())
            .unwrap_or_default();
        assert_eq!(got.labels, labels, "{name}: labels");

        let depends_on: Vec<u32> = expected(&want, "depends_on")
            .and_then(Json::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Json::as_u64)
                    .map(|n| n as u32)
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(got.depends_on, depends_on, "{name}: depends_on");

        // `milestone` and `source` are named keys to cairn and ordinary
        // frontmatter to harrow, which has no schema here to tell it
        // otherwise. They still have to arrive.
        for key in ["milestone", "source"] {
            let carried = got.fields.get(key).map(|v| v.display());
            assert_eq!(
                carried.as_deref(),
                expected(&want, key).and_then(text),
                "{name}: {key}"
            );
        }

        // Everything the project declares, and everything it does not: §4.1 is
        // the rule that makes version skew survivable, so a key from a format
        // that does not exist yet has to still be there.
        if let Some(fields) = expected(&want, "fields").and_then(Json::as_object) {
            for (key, value) in fields {
                let carried = got
                    .fields
                    .get(key)
                    .unwrap_or_else(|| panic!("{name}: dropped the field {key}"));
                if let Some(want) = text(value) {
                    assert_eq!(carried.display(), want, "{name}: field {key}");
                }
            }
        }
    }
}

/// The one thing the corpus cannot check, because a body's meaning is a
/// convention of the project rather than a property of the format.
#[test]
fn the_corpus_is_vendored_with_its_provenance() {
    let note = std::fs::read_to_string(corpus().join("PROVENANCE"))
        .expect("the corpus says where it came from");
    assert!(
        note.starts_with("cairn "),
        "the first line names the version it was taken at"
    );
}
