---
id: 45
title: Soak it against backlogs nobody wrote by hand
type: chore
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: testing
---

## Problem

harrow is tested against one fixture: six items, one schema, four statuses,
one milestone. The randomised suite drives forty keys at it and asserts the
invariants, which is the right idea applied to a backlog nobody would mistake
for a real one.

The shapes that break a program of this kind are the ones nobody writes by
hand. Two hundred statuses. A project with no milestones at all. A schema
whose statuses are all `done`. Five thousand items in one group. An item
belonging to four containers. A title of six hundred characters, or one
character, or one emoji. A filter that matches nothing, typed into a backlog
of twenty thousand.

cairn found a real bug in its merge driver this way — "found by the soak, on a
seed CI drew and I had not" — and harrow is the program that has to keep
drawing whatever cairn will write.

## Proposal

A generator for projects rather than a fixture: a seeded schema — statuses,
categories, types, fields, views, container types — and a seeded backlog
against it. Then the existing randomised driver, against those, asserting the
invariants after every key and that every frame renders.

Seeded, printed, and reproducible from the seed, so a failure on CI is a
failure on the desk. Small by default so it belongs in every run; the
environment variable pattern `HARROW_FUZZ_SEEDS` already uses is how it goes
deeper.

The scale table in 0042 is the same generator, so build it here and measure
there.

## Cost

A generator that only produces reasonable projects finds nothing, and one that
produces impossible ones tests behaviour cairn would never write. The rule:
generate anything `cairn check` would accept, and no more — the schema is the
contract, and it is the contract harrow has to survive.

## Acceptance criteria

- [ ] A seeded generator produces a schema and a backlog against it
- [ ] Everything it produces would pass `cairn check`
- [ ] The randomised driver runs against generated projects, not only the fixture
- [ ] A failure prints the seed, and the seed reproduces it
- [ ] The default run is small enough to belong in every `scripts/task check`
