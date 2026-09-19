---
id: 88
title: A milestone with nothing left shows the work that finished it
type: feature
status: done
milestone: v0.6
depends_on:
- 82
created: 2026-09-18
updated: 2026-09-18
priority: p2
area: chrome
---

## Problem

An ordinary listing hides finished work, and it is right to: beside work that
is still open, a closed item is noise.

A group with *no* open work left is the case where that stops being true.
0080 found it from one side — poptop's seven milestones were complete and
still open, the strip counted them and the list said there was nothing here.
This is the other side of it. Once the milestones are visible, a reader
looking at one that is finished sees a heading claiming `100%` with nothing
under it, because everything under it is closed and closed work is hidden.

The heading is making a claim the rows are not allowed to support.

There is a second, smaller thing wrong on the same screen. Done, in progress,
and waiting on something are three different states of a piece of work, and
the only thing telling them apart is a glyph and a hue. On a monochrome
terminal, or to a reader who does not know the glyphs yet, they read as one
list of things.

## Proposal

**Reveal what finished it.** A group whose open work is nought — and which is
itself still open — lists the items that closed it, rather than nothing. The
condition is exactly the one 0080 put in the needs queue, so the two agree
about what "nothing unfinished" means.

The rule is narrow on purpose: it is not `a` and it is not per-group
collapsing. Closed work stays hidden wherever there is open work to compare it
against, which is everywhere else.

**Say the state in motion, not only in hue.** Three states, three treatments
that survive `mono`:

- **in progress** — the glyph turns: `◐ ◓ ◑ ◒`, on the frame tick the spinner
  already runs on. Work being done is the only thing on a backlog that is
  *happening*, and it should be the only thing moving.
- **waiting** — dim, with the blocker's reference in reach. It is not stalled
  because nobody picked it up; it is stalled on something nameable.
- **done** — struck through where the terminal draws it, so a finished row
  reads as finished with the colour taken away.

A turning glyph rather than the terminal's blink attribute: `SLOW_BLINK` is
ignored by a good half of the terminals harrow runs in, and where it is
honoured it is honoured on the whole cell. The tick is already passed to
`render_frame` and is already what the spinner uses, so this animates
everywhere and still snapshots — a recorded screen is taken at tick 0.

## Cost

Motion in a list is a strong claim and it is easy to overspend. It is spent on
exactly one state here, and only on the glyph — not the row, not the title.
A backlog where three things are in progress has three small turning marks; a
backlog where thirty are has a problem the screen is right to be loud about.

## Acceptance criteria

- [x] A milestone with no open work left lists the work that closed it
- [x] Closed work stays hidden in every group that still has open work
- [x] The condition is the same one the needs queue uses for a finished container
- [x] In-progress items turn, on the tick the spinner already runs on
- [x] Waiting and done are told apart from each other, and from in progress, with the colour taken away
- [x] A recorded screen is still reproducible

## 2026-09-18

The reveal is one predicate, `withheld`, read by the rows and by the empty state — so the screen and its explanation of itself cannot disagree. That was the shape of 0080, and it is the second time the two have had to be made to read the same rule.

It only applies where the grouping *is* the container. Grouped by status there is no such thing as a group with nothing left, and the empty state is still reachable there — which is where its test now lives.

Blinking, as asked, but not the terminal's blink attribute: half the terminals harrow runs in ignore `SLOW_BLINK` and the ones that honour it blink the whole cell. The glyph turns instead, a quarter per two frames on the tick the spinner already uses. That animates everywhere, and a recorded screen is taken at tick 0 where it is `◐` — so it still snapshots.

Three tests in counting.rs asserted the old behaviour of exactly this fixture. Updated rather than deleted: the point each was making still holds, and what changed is which rows are under a milestone with nothing left.
