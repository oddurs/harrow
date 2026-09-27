---
id: 115
uid: d520e93a-1235-4ac2-97e4-0ed3b75f5253
title: Migrate Harrow's own backlog to format 5
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

Harrow reads format 5 (#103) and Cairn writes it (oddurs/cairn#105). Cairn
item 0147 migrates harrow, rim and nun; Harrow goes first because nothing
else depends on its backlog and no agent is working in it.

rim and nun wait: the installed cairn stays at format 4 until they migrate,
because a format-5 cairn refuses writes to every format-4 project and rim has
agents working. Until then, writing this backlog needs a format-5 cairn
build, not the installed one.

## What changes

`cairn migrate --commit` with a format-5 build: every number comes back from
`_legacy-ids.toml`, each UUID becomes the item's `uid`, references are
rewritten to numbers, and the table is removed. Migrate verifies that bodies
and every non-identity value are unchanged before it writes anything.

## Done when

- [x] The backlog is format 5 in one migration commit, verified by migrate
- [x] Harrow reads it: `harrow --doctor` and `scripts/task check` pass
- [x] `cairn check --render --strict` passes with a format-5 cairn

## 2026-09-27

Migrated with cairn 5caf5c6 (migrate --commit, 149dc1e): 115 items, 110 numbers restored from the map, 5 numbered 111-115, 34 references follow their items; migrate verified bodies and every non-identity value before writing. cairn check --render --strict ok; harrow --doctor with a format-5 cairn agrees on 115 items; scripts/task check 558 passed. The installed cairn stays format 4 until rim and nun migrate, so writing this backlog needs a format-5 build meanwhile.
