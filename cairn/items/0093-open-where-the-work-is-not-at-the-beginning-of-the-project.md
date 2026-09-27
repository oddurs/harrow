---
id: c3c3a3cd-ffda-4adf-a567-269f971b2b37
title: Open where the work is, not at the beginning of the project
type: feature
status: done
milestone: v0.6
created: 2026-09-19
updated: 2026-09-19
priority: p1
area: chrome
---

## Problem

harrow opens at row nought, which on any project past its first milestone is
the oldest finished work there is. In harrow's own backlog that is v0.1,
thirteen items, every one of them closed a month ago — and v1.0, where the
work actually is, is a hundred rows down.

Every session starts with the same scroll. The first thing the program does
with a backlog is show you the part of it nobody is going to touch again.

## Proposal

Open on the seam: the first group that still has open work, placed about a
third of the way down the pane.

```
│ ▾ v0.5  The work is somebody else's…  ▰▰▰▰▰▰ 100%  done │   ← a third above
│ ▾ v0.6  Compose a view; reach a com…  ▰▰▰▰▰▰ 100%  done │
│ ▾ v1.0  Stable release                ▰▰▱▱▱▱  20%  12 left │
│  ✓ 3 done                                       ↵ shows │   ← the seam
│  ○ 0021 Package it: crates.io and a tap      0/5  p1    │   ← two thirds below
│  ○ 0041 Undo the last thing you did          0/5  p1    │
```

A third rather than the top, because a backlog has a direction: what is behind
you is context and what is ahead is the work. Two thirds of the pane for what
is ahead and a third for what got you here is the ratio that reads right — at
the top there is no context at all, and in the middle half the pane is spent
on history.

**The seam is the fold row where there is one** — 0092's `✓ 3 done`, which is
exactly the boundary between what is finished and what is not — and the group
heading where there is not. The cursor goes on the first open item under it,
because that is the row a key would act on.

**Only on the first frame.** Homing on every rebuild would fight the reader:
every claim, every close, every keystroke into the filter box would throw the
scroll back. It happens once, when the backlog first arrives.

**And by name.** `go to the work` in the palette, so the gesture that opens
the program is one you can repeat after wandering. No key: it is worth having
and not worth a letter, which is what the palette is for.

## Cost

A reader who wants the top has to press `g`, which they were already going to
press to get anywhere. Against that, every reader who wanted the frontier —
which is every reader on any day they are working rather than reading — stops
scrolling to find it.

## Acceptance criteria

- [x] It opens on the first group with open work rather than at row nought
- [x] That row sits about a third down the pane, not at the top
- [x] The cursor lands on something a key would act on
- [x] Scrolling away and rebuilding does not throw you back
- [x] A backlog with nothing finished, or nothing open, opens somewhere sensible
- [x] The same place is reachable by name afterwards

## 2026-09-19

The rule is two clauses, and the second is the fallback: the first group with work *under way*, else the first with any open work. One clause is not enough — a single forgotten item in v0.1 would pin every session to the past, and `doing` is a signal the reader set themselves.

The seam is 0092's fold row where there is one, which is the boundary between finished and unfinished by construction. That the fold happened to be exactly the right anchor is luck; it is the row that means *this is where it changes*.

Two things the arithmetic cannot do, and should not try: a backlog shorter than the pane does not scroll, and one with less than two thirds of a pane below the seam shows its end instead. Both are `scroll_to` clamping, and both are right — there is nothing to put down there.

The cursor goes on the row the frontier was found by rather than the first item under the seam. Under the default order, `status,priority,id`, backlog sorts before doing, so "first row under the fold" was a backlog item and the thing actually in progress was four rows further down.
