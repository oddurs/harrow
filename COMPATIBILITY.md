# Cairn and Harrow

Cairn owns the format and every write. Harrow independently reads that format
and supplies the human interface. Neither needs a shared runtime library or a
Cairn process on each read.

Harrow is tested against Cairn **0.3.0**, revision
`80443135bfae0f2d2804e60f2636005d1590e7ca`. It reads formats 1–3; the frozen
older corpora stay in the suite. Format 3 is unchanged between 0.2.2 and
0.3.0, and the vendored corpus is byte-identical across the two: advancing the
pin moved `PROVENANCE` and nothing else. Older Harrow 0.1.0 binaries may
reject `closed_at` queries; use a revision including 0099 until a companion
release includes it.

## The gate

`scripts/task check` always runs the vendored format corpus. CI also requires
`scripts/task agreement`, against the pinned Cairn checkout in `ci.yml`:

```sh
PATH=/path/to/cairn/target/debug:$PATH \
CAIRN_PROJECT_DIR=/path/to/cairn scripts/task agreement
```

Missing executables, fixtures, or mismatches fail. The comparison is marked
ignored only in the standalone Rust suite, so a checkout remains testable
without Cairn. Required CI explicitly runs it; it is not an optional release
check. It covers Cairn's project views via `cairn --ids` and `harrow --plain`,
plus query fixtures for dates, categories, dependencies, criteria and hierarchy.

To advance the pin, inspect Cairn's format/CLI changes, refresh the corpus with
`CAIRN_REPO=/path/to/cairn scripts/task conformance:refresh`, review every diff,
update CI's exact revision, and run both suites. `PROVENANCE` identifies the
corpus source. Do not edit an expectation to conceal a disagreement.

Refresh from a **clean checkout of the release being pinned**, not from a
working copy. A sibling checkout is somebody's desk: ours held uncommitted
work toward a later version, and refreshing from it would have vendored an
unreleased corpus under a released version's name. Clone the tag, and check
that the tree is clean and `cairn --version` is the version you mean —
`target/debug/cairn` in a dirty checkout is not the release it is beside.

## Deliberate differences and shared semantics

- Harrow accepts free text in its interactive filter; Cairn uses `search` for
  that. Shared saved views should use field predicates.
- Harrow rejects an unknown filter field immediately. Cairn warns at query
  time and checks schema problems with `check`. Neither should quietly claim a
  misspelled field is a meaningful empty backlog.
- Grouping can draw containers as headings. Compare ungrouped item sets, not
  screen rows or their order. `next` additionally excludes containers, ranks
  active work first, and applies dependency readiness.
- `ready` means unfinished and dependency-ready, including active items. It
  does not mean unassigned or approved. Use your project's saved view.
- `criteria_met` is true when no criteria are stated; `criteria>0` distinguishes
  an explicitly completed checklist from no checklist.
- An ordinary listing leaves out closed work and containers. Naming the field
  is how you ask for them back, and naming it means saying anything about it —
  `status=done`, `status!=dropped` and `category!=dropped` all lift it, with
  the predicate itself doing any excluding. A closed container is behind both
  defaults and needs the type and the status. Both tools read it this way;
  harrow once required an equality, which made `category!=dropped` differ by
  every closed item (0106).
- Range comparisons use the display value; missing values compare as empty
  strings. Use `closed_at!=,closed_at<2026-09-10` to exclude undated history.
- `closed_at` is the recorded completion date, unaffected by later edits.
  Completion statistics prefer it. For legacy items lacking it, statistics
  retain their old `updated` estimate; date filters never invent a value.
- Harrow can inspect newer formats read-only with a warning. This is recovery
  behavior, not a promise to understand semantics it has never tested.

For a mismatch, include both tool versions/revisions, `cairn config --json`,
`harrow --doctor`, the filter or view, and a small non-sensitive reproduction.
