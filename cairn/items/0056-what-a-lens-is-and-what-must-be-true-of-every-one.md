---
id: 56
title: What a lens is, and what must be true of every one
type: spike
status: done
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

**A lens is an arrangement. Everything else is shared.**

Five things every lens owes the reader, now in CLAUDE.md beside the other
rules that must stay true, and held by `tests/lenses.rs` for every `Pane`:

- it **obeys the filter**
- it **answers the mouse**
- it **reaches the detail** of whatever is selected
- it **keeps the selection** across a switch
- it **keeps the marks**, meaning the same thing

Fifteen combinations, eleven already true. The four that are not are listed
in the test as `GAPS`, each naming the item that closes it — so the test
fails if a gap is forgotten *and* fails if a gap is closed without deleting
its row. The list is a debt with a due date rather than a description of how
things are.

**Held by a test, not a trait.** A trait was the tempting answer and is the
wrong one: the three lenses genuinely differ in shape — the board has drag,
the stats has no cursor to speak of yet — so a trait that fitted all three
would either be too loose to enforce anything or would bend the board's
gestures through a hole made for the list. The test costs no abstraction,
catches the regression, and reads as the definition.

**Prose as well, because a test is only met by somebody already writing
code.** The rule belongs where a person decides what to build, which is
CLAUDE.md.

### The open question, answered

*Is a whole-backlog overview a legitimate thing to want, or must the filter
always apply?*

The filter always applies. An unfiltered overview is already one keystroke
away — `esc` clears the filter — so a second mode would be a second way to
reach something you can reach, at the cost of a reader never being sure which
of the two they are looking at. 0057 narrows the stats and adds nothing else.

### What this changes about the rest of v0.4

**0058 was too absolute.** "The detail is reachable" is the invariant; *drawn
beside the arrangement at all times* is not. A board of four columns at
ninety-six columns, minus a forty-four-column detail pane, leaves thirteen
columns a card — which is not a board. So the width a lens needs before it
can spare the detail is a property of the lens, and the board's is higher
than the list's. The item already suspected this; it is now the expected
answer rather than an open one.

**0060 is not about the contract at all.** The grouping axis is arrangement,
so it may legitimately differ — stats has no axis and never will. What is
wrong with the board is narrower than the item implies: not that it groups
by status, but that it says *the board is grouped by status* as though that
were a fact about boards rather than a thing harrow has not built. Both the
capability and the honesty are worth having; only the second is owed.

**0062 shrinks.** With `shift-tab`, every lens is one press away from every
other, so a direct key per lens buys nothing and spends three bindings that
a future "jump to the nth thing" will want. Add the reverse, record the
reason, leave the digits.

## Acceptance criteria

- [x] The invariants are written down where somebody adding a lens will meet them
- [x] Each is either held by a test or has a recorded reason why it cannot be
- [x] The remaining v0.4 items are re-read against it, and any that disagree are changed or closed
