---
id: 92
title: A milestone in progress hides the work it has finished
type: bug
status: done
milestone: v0.6
created: 2026-09-19
updated: 2026-09-19
priority: p1
area: read
---

## What happens

In cornell, working through v0.7:

```
╭ Backlog ───────────────────────────────────────────────╮
│ ▾ v0.7  Roughness                  ▰▰▰▰▱▱  53%  6 left │
│  ○ 0160 What a stochastic BSDF does to the three-…  p0 │
│  ○ 0161 The microsurface as a place a ray tr…  0/3  p0 │
│  ○ 0162 Verify: the rough conductor finally vanis…  p0 │
│  ○ 0088 Anisotropy: two roughnesses and a ta…  0/2  p2 │
│  ○ 0089 Roughness is not alpha, and the rema…  0/2  p2 │
│  ○ 0090 Prose: what a statistical surface is        p2 │
╰────────────────────────────────────────────────────────╯
```

The heading says **53%** and **6 left**. Seven items are done in that
milestone and not one of them is on screen. The heading is making a claim that
nothing under it supports — which is word for word what 0088 was about, and
0088 fixed it only at the endpoint: a milestone with *nothing* left reveals
its work, a milestone half way through reveals none of it.

The two are visible on one screen. Scrolled up, v0.1 at 100% lists all
sixteen of its finished items; v0.7 at 53% lists none of its seven. A rule
that shows everything at one end and nothing in the middle is worse than
either, because a reader cannot tell which rule they are looking at.

## Why the work was hidden in the first place

Because beside open work a closed item is noise, and a backlog of five
hundred where four hundred are done is unreadable. That is right about a
*backlog*. It is wrong about the *milestone you are in*, where the finished
work is the reason the percentage is what it is, and where it is bounded —
seven items, not four hundred.

## Proposal

A group with finished work says so, in one row, directly under its heading:

```
│ ▾ v0.7  Roughness                  ▰▰▰▰▱▱  53%  6 left │
│  ✓ 7 done                                      ↵ shows │
│  ○ 0160 What a stochastic BSDF does to the three-…  p0 │
```

The claim and its evidence are then together: the percentage is on one line
and the thing it counts is on the next, one keystroke away. Pressing `↵` or
`space` on it unfolds the seven above the open work — done first, then what is
left, which is the order the heading already reads in.

Three properties this has that the alternatives do not:

- **Nothing moves until you ask.** A rule that revealed finished work when the
  cursor entered a group would make rows appear and disappear as you moved,
  and take the scroll position with them.
- **It costs one row, not seven.** A group with nothing finished gets no row
  at all, so a fresh backlog looks exactly as it does now.
- **It is the same gesture as collapsing.** A heading folds a group away; this
  unfolds what the group has already done. Both on `space`, on the row that
  says so.

0088's rule stays: a milestone with nothing left still shows its work without
being asked, because there the row would be the only thing under the heading
and folding it would leave a hole.

## Acceptance criteria

- [x] A group with finished work and open work says how much it has finished
- [x] Unfolding it lists that work above the open work
- [x] Nothing appears or disappears as the cursor moves
- [x] A group with nothing finished is unchanged
- [x] A milestone with nothing left still shows its work without being asked
- [x] The fold survives a reload, the way a collapsed group does

## 2026-09-19

0088 fixed this at the endpoint and left the middle alone, which made it worse rather than half better: on one screen, v0.1 at a hundred per cent listed all sixteen of its finished items and v0.7 at fifty-three listed none of its seven. A reader cannot tell which rule they are looking at.

A fold rather than a reveal, for one reason worth writing down: the obvious alternative is to show a group's finished work when the cursor is inside it, and that makes rows appear and disappear as you move, taking the scroll position with them. Nothing here moves until it is asked to.

The fold is a container and its contents go under it, which is a rearrangement inside a group rather than a different sort. The reader's order still applies within the block — the default is `status,priority,id`, which would otherwise have put the finished work *below* the open work, at the bottom of a group you have to scroll past to reach.

The count is of what the filter would show, not `Group::done`. That tally deliberately ignores the filter so a heading's progress does not move when you narrow the list; the fold is an offer to show particular rows, so it has to count the rows it would show.

The glyph guard from 0091 earned itself here: `space` on the first row of a group used to mark an item and now folds, so nothing in the guard's states marked anything and `▌` failed the declared-but-undrawn check. That is exactly the drift it was written to catch, one change after it was written.
