---
id: 120
uid: d8e9c9ca-af73-484e-917b-32bb2bc180d2
title: Advance the Cairn pin to Results
type: chore
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 118
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
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

- [x] CI builds the merged Cairn, and COMPATIBILITY.md names it
- [x] The agreement gate compares every item's Result and passes
- [x] Removing the fence rule from harrow's reader fails that gate

## 2026-09-28

Pinned Cairn 9c29249 (main after cairn#121), which has Result (0155), prompt (0156), split (0157), prompt checks (0158), the broken-pipe exit, and the lock that waits on a moving queue. Corpus refreshed from a clean detached checkout of exactly that revision (git status clean, cairn --version 1.0.0-alpha.1): it gained result.md, result-empty.md and result-above-a-note.md, 66 cases; tests/conformance.rs already compared result and passes. New agreement gate every_item_concludes_the_same_in_both_tools: cairn list --all --json against harrow's loaded items, id by id, over the pinned Cairn checkout (11 items with a Result) and over seven built bodies (fenced, tilde-fenced, level and case, decorated heading, above a note, empty, none; cairn finds 5 Results). Agreement 6/6 against that build. Deleting the fence rule in item::headings_from (the fence.is_some() continue) fails the gate on the built bodies; restoring it passes.

## 2026-09-28

CI on #110 (head 2721add): the agreement job checked out and built Cairn 9c29249 and ran scripts/task agreement, pass; check on ubuntu and macos, pass.

## Result

CI builds Cairn 9c29249, which writes Results, prompts, splits and prompt checks. The corpus has its three Result cases, and the agreement gate holds every item's Result to cairn's reading, null for null, over the pinned Cairn and over bodies built around fences, heading levels and notes; without the fence rule it fails.
