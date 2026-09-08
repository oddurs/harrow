---
id: 12
title: Read an item in full without leaving
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: chrome
---

## Problem

The reason cairn is worth more than a ticket tracker is the body: the problem,
the proposal, what was decided and why. A backlog browser that shows titles has
thrown away the part that matters.

## Proposal

`enter` opens the whole item, scrollable, with the small amount of Markdown a
pane this size earns: headings, checkboxes, and text.

## Acceptance criteria

- [x] Any key other than the scroll keys closes it
- [x] Acceptance criteria read as ticked or not
- [x] The detail pane shows as much of the body as fits, and says there is more
