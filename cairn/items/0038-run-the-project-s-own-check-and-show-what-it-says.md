---
id: 38
title: Run the project's own check and show what it says
type: feature
status: backlog
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

`cairn check` validates every item against the schema, and in 0.2.0 it
validates the schema first: a status with no category, a `render.group_by` or
saved view naming a field nothing declares, a filter that does not parse, a
missing header, `link_items` with no `project.url`. The agent loop ends with
it — "`cairn check` must pass before the work is considered done".

harrow has diagnostics of its own, reached with `D`, which report what harrow
could not read. They are not the same thing, and harrow's are the weaker half:
a file harrow parsed happily can still be invalid against the project's own
rules, and harrow will show it as an ordinary row.

## Proposal

Run `cairn check` on demand and show what it says in the diagnostics overlay,
under its own heading, distinct from the problems harrow found itself. Where a
line names an item, make it selectable — the value of a validator inside the
tool is that the failure and the item are one keystroke apart.

On demand, not on load: `check` is a process, harrow reloads on every file
change, and a validator that runs on a watch is a validator that runs three
times a keystroke.

## Cost

Diagnostics become two things from two sources, and the overlay has to make
which is which obvious or a cairn complaint will be read as a harrow bug.

## Acceptance criteria

- [ ] A key runs `cairn check` and shows its output in the diagnostics overlay
- [ ] cairn's findings are visibly distinct from harrow's own
- [ ] A finding that names an item selects it
- [ ] With cairn unavailable, the overlay says so rather than showing an empty section
