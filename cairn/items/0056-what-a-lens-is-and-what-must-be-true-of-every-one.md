---
id: 56
title: What a lens is, and what must be true of every one
type: spike
status: backlog
milestone: v0.4
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Question

What is a lens, and what must be true of all of them?

harrow has three and they disagree about nearly everything — the cursor, the
detail, the filter, the mouse, whether you can act at all. Each difference
arrived one at a time and none was ever argued for. Fixing them one by one
fixes today's list; it does not stop the next pane, or the next feature on an
existing pane, from drifting the same way.

## Why it has to be answered before the work

Every other item in v0.4 is an instance of the same question, and without an
answer they are six separate opinions. With one, they are one decision
applied six times — and the seventh, whenever somebody adds a lens, is
answered before it is written.

There is precedent for this working: the one rule that has held is *nothing
about a workflow is hardcoded*, and it has held because it is written down
where a contributor trips over it.

## Options

**A contract in prose, in CLAUDE.md.** Cheapest. Held by whoever remembers.
This is what the existing architectural rules do, and they have held — but
they are about what harrow must not *do*, which is easier to notice in review
than a missing capability.

**A contract as a trait.** Each lens implements `draw`, `hit`, `cursor`. The
compiler holds it. Real risk: the three genuinely differ in shape, and a
trait that fits all three may be so loose it enforces nothing, or so tight
that the board's drag has to be bent through it.

**A contract as a test.** For every `Pane`, assert the invariants hold: a
filter narrows it, the selection survives arriving and leaving, something is
clickable, the detail is reachable. Costs no abstraction, catches the
regression, and reads as documentation of what a lens is.

## What would settle it

Name the invariants first, then pick. A draft to argue with:

- **The selection is the same item** in every lens that has a cursor, before
  and after switching.
- **The filter applies**, or the lens says out loud that it does not.
- **The detail of the selected item is reachable**, whatever the arrangement.
- **Anything drawn can be clicked**, because the mouse is not a second-class
  way to drive this.
- **Marks survive** a switch and mean the same thing.
- **The arrangement is the only difference.**

Is "the filter applies" truly universal, or is a whole-backlog overview a
legitimate thing to want? If it is, it is a *mode of the stats lens* and has
to say so on screen — not an accident of which function got a `query` and
which did not.

## Answer

<!-- Filled in when the spike closes. A spike that closes with no answer
     recorded was a waste of the time it took. -->

## Acceptance criteria

- [ ] The invariants are written down where somebody adding a lens will meet them
- [ ] Each is either held by a test or has a recorded reason why it cannot be
- [ ] The remaining v0.4 items are re-read against it, and any that disagree are changed or closed
