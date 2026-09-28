---
id: 120
uid: d8e9c9ca-af73-484e-917b-32bb2bc180d2
title: Advance the Cairn pin to Results
type: chore
status: backlog
milestone: v0.8
depends_on:
- 118
created: 2026-09-27
updated: 2026-09-27
priority: p1
effort: s
area: read
---

## Why now

Blocked until cairn 0155 (`feat/0155-result`) merges. Until then, harrow's
Result reader is checked against the specification text and cairn's unit
cases, and not against cairn itself.

## What changes

- CI (`.github/workflows/ci.yml`), COMPATIBILITY.md and the corpus provenance
  pin the merge commit, following the procedure in COMPATIBILITY.md.
- The corpus is refreshed from a clean checkout of that revision. Where it
  gained Result cases, `tests/conformance.rs` compares `result`.
- `tests/agreement.rs` gains a gate: for every item in the pinned Cairn
  checkout, harrow's `result` equals the `result` in `cairn show --json`,
  null for null.

## Done when

- [ ] CI builds the merged Cairn, and COMPATIBILITY.md names it
- [ ] The agreement gate compares every item's Result and passes
- [ ] Removing the fence rule from harrow's reader fails that gate
