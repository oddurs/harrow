---
id: 29
title: Acceptance criteria are what the convention says they are
type: bug
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: read
---

## What happens

harrow counts every `- [ ]` and `- [x]` in a body as an acceptance criterion.
Specification §10.1, which is where the convention is written down, says three
things harrow does not do.

A box with **nothing after it** is a placeholder, not a criterion. cairn's own
templates end with a bare `- [ ]` prompting the author to write one — this
project's `cairn.toml` does it for `feature` and `bug` — so every item created
from a template reads as permanently 0 of 1 ticked, and the list shows a
`0/1` column for an item nobody has failed to do anything on.

The marker may be `-`, `*` or `+`. harrow accepts only `-`.

A project **may confine the count to one section**, named in
`project.criteria_section`, matched at any heading level and without regard to
case. harrow counts the whole body, so a ticked box in a "What I tried" note
counts as an acceptance criterion met.

## Proposal

Implement §10.1 as written: require something after the box, accept all three
markers, and honour `criteria_section` when the project declares it.

## Cost

The criteria count is on every row of the list and in two progress bars, so
this changes numbers people have been reading. That is the point — they have
been wrong — but it is worth a line in the changelog rather than a silent
correction.

## Acceptance criteria

- [x] A bare `- [ ]` is not counted
- [x] `*` and `+` markers count, and indentation is allowed
- [x] With `criteria_section` set, only boxes under a heading of that name count, at any level, matched without regard to case
- [x] With it unset, the whole body counts, as it does now
