# Cairn and Harrow

Cairn owns the format and every write. Harrow independently reads that format
and supplies the human interface. Neither needs a shared runtime library or a
Cairn process on each read.

The contract work in item 0099 is tested against Cairn **0.2.2**, revision
`335a4d38336da4f38a6c216e91e940c989a3efc8`. It reads formats 1–3; the frozen
older corpora stay in the suite. Cairn 0.3 adds explicit view selection without
changing format 3. Older Harrow 0.1.0 binaries may reject `closed_at` queries;
use a revision including 0099 until a companion release includes it.

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
- Range comparisons use the display value; missing values compare as empty
  strings. Use `closed_at!=,closed_at<2026-09-10` to exclude undated history.
- `closed_at` is the recorded completion date, unaffected by later edits.
  Completion statistics prefer it. For legacy items lacking it, statistics
  retain their old `updated` estimate; date filters never invent a value.
- Harrow can inspect newer formats read-only with a warning. This is recovery
  behavior, not a promise to understand semantics it has never tested.

For a mismatch, include both tool versions/revisions, `cairn config --json`,
`harrow --doctor`, the filter or view, and a small non-sensitive reproduction.
