---
id: 61
title: The stats pane answers the mouse
type: feature
status: done
milestone: v0.4
depends_on:
- 56
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

The README: *It is meant to be usable without learning any of the above.*
Every tab, every status in the strip, every row, every card, every footer
hint is clickable. The stats pane registers no hit regions at all.

So the one lens with no keyboard cursor also has no pointer. It is the only
part of harrow you cannot touch.

## Proposal

Hit regions for everything the pane draws that stands for something: each
figure, each named item, each milestone row. What a click does is 0059's
question — this is the machinery that lets it be answered, and the two should
land together or the regions have nothing to do.

## Acceptance criteria

- [x] Everything on the stats pane that stands for a set or an item is clickable
- [x] A click does what Enter on the same thing does
- [x] Nothing is clickable that does not respond, because a dead target is worse than none
