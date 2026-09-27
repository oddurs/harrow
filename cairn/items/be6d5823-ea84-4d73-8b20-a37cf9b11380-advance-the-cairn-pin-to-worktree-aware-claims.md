---
id: be6d5823-ea84-4d73-8b20-a37cf9b11380
title: Advance the Cairn pin to worktree-aware claims
type: chore
status: done
assignee: Oddur Sigurdsson
created: 2026-09-27
updated: 2026-09-27
closed_at: 2026-09-27
priority: p1
area: read
effort: s
---

## Why now

Cairn merged three things on 2026-09-27: claims seen across worktrees (#107),
`next` counting blocked work within its own selection (#104), and a dependency
bump (#91). Harrow's agreement job still built Cairn at `ec982f8`, so none of it
was tested against this reader.

## What changes

- CI, COMPATIBILITY.md and the corpus provenance pin Cairn `8cf3747`. The
  refreshed corpus is byte-identical; only its provenance moves.
- A container's rollup no longer counts finished work as blocked. Cairn #104
  made the same rule explicit ("finished items no longer count as blocked in
  any summary"), and Harrow's own `Rollup` doc already said a blocked item is
  open or active. The strip and the heading tally already excluded closed
  items; the rollup was the one place that did not.

Deliberately unchanged: Harrow's worktree annotation (#101) marks every item
another worktree changed, where Cairn refuses only what is claimed or no longer
open there, and Cairn's `cairn worktrees` also lists items filed on a branch.
The board shows more than the claim refuses; they do not contradict each other.

## Done when

- [x] `scripts/task agreement` passes against Cairn `8cf3747`
- [x] A finished item depending on unfinished work is not counted as blocked
      under its container, with a test that fails without the fix
- [x] `scripts/task check` passes

## 2026-09-27

Agreement 5/5 against Cairn 8cf3747 (PATH debug build, CAIRN_PROJECT_DIR the cairn checkout). scripts/task check: 548 passed. The rollup test fails with the old count (blocked 2, expected 1) and passes with the fix.
