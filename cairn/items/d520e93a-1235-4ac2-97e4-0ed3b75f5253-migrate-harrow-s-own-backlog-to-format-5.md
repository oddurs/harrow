---
id: d520e93a-1235-4ac2-97e4-0ed3b75f5253
title: Migrate Harrow's own backlog to format 5
type: chore
status: doing
assignee: Oddur Sigurdsson
claimed: 2026-09-27
created: 2026-09-27
updated: 2026-09-27
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

- [ ] The backlog is format 5 in one migration commit, verified by migrate
- [ ] Harrow reads it: `harrow --doctor` and `scripts/task check` pass
- [ ] `cairn check --render --strict` passes with a format-5 cairn
