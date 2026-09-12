---
id: 64
title: A lens for what needs you
type: feature
status: backlog
milestone: v0.5
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Problem

Everything a person is needed for in a cairn backlog is scattered across the
lenses that exist, or is invisible.

A proposal is a program asking for a decision only a person can make. harrow
shows it on the item, if you happen to select that item. A stale claim is a
worker that did not come back; harrow colours the assignee, if you happen to
be looking at that row. An item whose acceptance criteria are all ticked and
which is still open is finished work nobody closed; nothing shows that at
all. `cairn check` findings live behind `ctrl-k` in the diagnostics overlay.

Each of these is *a question addressed to you*. They have nothing in common
except that — and that is the most important thing about them, because it is
the thing that decides what you do next.

## Proposal

A lens whose rows are questions, ranked by how much they are holding up, and
whose answers are single keys.

- **A proposal is waiting.** `field: from → to`, who asked, why. Accept, or
  leave it.
- **A claim has gone cold.** Who holds it and for how long. Release, or leave
  it.
- **Everything is ticked and it is still open.** Close it, or leave it.
- **cairn refuses the project.** What `check` said, once it has been asked.
- **Nobody is answerable.** An item a program filed with no owner.

Every row carries the item it is about, so the detail pane answers *what is
this* without leaving — which the lens contract already requires.

The empty state is the point. *Nothing needs you* is the best screen this
program can show and there is currently no way to see it, because no screen
makes the claim. It should be reachable, and it should be the first thing
harrow opens on when it is true.

## Cost

A queue that cries wolf is a queue nobody reads, and four of the five rows
above are judgement calls the project should be able to influence. Two are
already the project's: `claim_stale_after` decides cold, and `agent` decides
what should have been proposed. The other two are harrow's opinion, and
harrow should hold them quietly — no counter in the header when the queue is
empty, and nothing that nags.

The ranking matters more than the contents. A proposal blocks a person; a
missing owner does not. Sorted by what is actually waiting on a human, not by
severity in the abstract.

## Acceptance criteria

- [ ] One lens lists every question addressed to a person, ranked by what is waiting
- [ ] Each row's answers are single keys, and say what they will do before they are pressed
- [ ] Selecting a row selects its item, so the detail pane answers what it is about
- [ ] It meets the lens contract: the filter, the mouse, the detail, the selection, the marks
- [ ] Its empty state says so plainly, and is reachable
- [ ] Nothing in it nags when there is nothing to do
