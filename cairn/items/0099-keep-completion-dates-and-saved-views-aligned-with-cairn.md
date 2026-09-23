---
id: 99
title: Keep completion dates and saved views aligned with Cairn
type: bug
status: doing
assignee: codex
claimed: 2026-09-23
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: read
---

## What happens

Cairn records `closed_at` independently of later edits. Harrow carries it as an
unknown field, rejects completion-date filters, and dates completion statistics
from `updated`. Its optional agreement checks miss the regression. Coordinates
with Cairn item 0137 in its v0.3 milestone.

## What should happen

Keep the independent reader and Cairn-owned writes. Read, display and query the
completion date; use it for statistics when present. Preserve the documented
legacy estimate from `updated` for old items without a recorded completion date.
Refresh the upstream corpus and require a pinned cross-tool comparison in CI.

## Reproduction

1. Open a finished item with `closed_at: 2026-09-01` and `updated: 2026-09-20`.
2. Filter `closed_at>=2026-09-01`; current Harrow rejects the built-in.

## Acceptance criteria

- [ ] Completion date survives parsing, appears in details, filters correctly,
  and a later edit does not move recorded completion statistics.
- [ ] All Cairn corpus cases pass with pinned provenance.
- [ ] Agreement tests compare machine-readable item sets for project views,
  date queries, dependencies, criteria, categories and composition.
- [ ] Required CI runs the comparison against a pinned Cairn revision and fails
  if either tool or the project fixture is missing.
- [ ] Supported versions and deliberate query differences are documented.

## 2026-09-23

Expanded comparison exposed and fixed readiness for active items, empty criteria, hierarchy fields, negative alternatives, range/presence semantics, and missing metadata lookup in addition to closed_at. All eight live Cairn project views and the query fixtures now agree. Recorded completion remains independent of later edits. Refreshed the unchanged upstream format corpus from Cairn 335a4d3 and documented compatibility; statistics snapshots intentionally change only ready from 2 to 3.
