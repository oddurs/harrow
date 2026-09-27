---
id: 112
uid: 197f02bf-3380-4d55-9d21-e1951986b54f
title: Pin the merged format-5 Cairn
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

harrow#103 pinned Cairn c5435a7, the head of the unmerged format-5 PR. That PR
merged as 1a0e469 after taking in Cairn main, so the pin named a commit on a
branch that no longer exists.

## What changes

CI, COMPATIBILITY.md and the corpus provenance pin Cairn 1a0e469. The
refreshed corpus is byte-identical.

## Done when

- [x] `scripts/task agreement` passes against Cairn 1a0e469
- [x] `scripts/task check` passes

## 2026-09-27

Agreement 5/5 against a scratch build of Cairn 1a0e469 with CAIRN_PROJECT_DIR the merged checkout. scripts/task check: 558 passed.
