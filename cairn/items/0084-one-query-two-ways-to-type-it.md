---
id: 45943011-8a23-4fa1-beab-e27a2f3c7226
title: One query, two ways to type it
type: feature
status: done
milestone: v0.6
depends_on:
- 449588bb-ac84-4170-abea-759389f5cfb1
created: 2026-09-18
updated: 2026-09-18
priority: p2
area: filter
---

## Problem

There are two doors onto the same query and they cannot see each other.

`/` opens the filter box and takes cairn's grammar, typed. `f` opens the facet
panel, built from the project's own schema, and ticks values with live counts
beside them. Both write `App::filter`. Neither states what the other did.

The panel is the narrower of the two by construction: it offers the values a
field declares, so it can say `priority=p0` and cannot say `due<2026-10-01`,
`priority!=p3`, or any free-text clause. Tick a facet after typing a date
bound and the bound is either silently lost or silently kept — and there is
nothing on screen either way.

Two controls that edit one value and disagree about it is the shape of 0080,
which cost two releases to find.

## Proposal

The panel stops being a rival to the query and becomes an input method for it.

Ticking `p0` inserts the clause `priority=p0` into the query segment of the
view line (0082). Unticking removes that clause and leaves every other clause
alone. The panel's ticks are then a *rendering* of the query rather than a
second copy of it: opening `f` on a hand-typed filter shows the clauses it
understands already ticked, and the ones it cannot express still sitting in
the line where they can be read.

That inverts the current relationship. Today the panel can do less than the
line and hides the difference; afterwards the panel can do less than the line
and shows it, which is the honest version and the one a reader can act on.

It also removes a whole class of question. There is no longer "what does `f`
do to a filter I typed" — there is one query, and two ways to type into it.

## Cost

Clause-level editing is harder than replacing a string. Removing `priority=p0`
from `due<2026-10-01,priority=p0,status=doing` means the panel has to parse,
edit and re-render the grammar rather than append to it — so `filter.rs` needs
`Query` to render back to source, which it currently cannot do.

That rendering is worth having anyway: 0082 needs it to state a view that came
from `--view` rather than from the box.

## Acceptance criteria

- [x] Ticking a facet writes one clause; unticking removes that clause and no other
- [x] Opening the panel on a typed filter shows the clauses it understands as ticked
- [x] A clause the panel cannot express survives being ticked around
- [x] `Query` renders back to grammar that reparses to the same query

## 2026-09-18

**Filed against a premise that did not hold.** The body says a date bound is silently lost or kept when you tick a facet. It is not: `toggle_facet` has always rebuilt the filter as `query.except(&managed)` plus the panel's own clauses, so an unmanaged clause is written back as it was read. `ticked()` reads a typed `priority=p1` as a tick for the same reason. Both were right before this item existed, and I should have read the function before writing the item rather than after.

What was actually missing was smaller and is now done: the panel narrowed every count on it by a clause it could not draw and said nothing about it. It now says `5 of 79 · 1 typed`, so the panel is honest standing alone rather than only beside the view line.

The rest of this is tests. Four behaviours nobody had held: a tick writes one clause and an untick removes that one, a bound survives being ticked around, a typed clause opens already ticked, and `Query::source` round-trips. They are the agreement between the two controls, and the agreement is the thing 0080 cost two releases to find the absence of.
