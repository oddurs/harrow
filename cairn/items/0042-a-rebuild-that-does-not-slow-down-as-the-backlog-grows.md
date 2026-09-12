---
id: 42
title: A rebuild that does not slow down as the backlog grows
type: bug
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: read
---

## What happens

The rebuild gets slower faster than the backlog gets bigger. Measured in
release, on generated projects:

| items | milestones | load | ingest | rebuild | frame |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 500 | 20 | 7.5 ms | 0.6 ms | **0.50 ms** | 0.5 ms |
| 2 000 | 50 | 27.9 ms | 3.6 ms | **3.54 ms** | 0.3 ms |
| 5 000 | 100 | 70.8 ms | 14.6 ms | **14.85 ms** | 0.4 ms |

Ten times the items, thirty times the rebuild. Loading is linear and off the
main thread, and drawing is flat because a frame only renders what fits — the
curve is entirely in `rebuild`, and `rebuild` is not a background job. It runs
on every keystroke typed into the filter box, because the list narrowing as
you type is the thing that makes the filter worth having.

Two causes, both in the grouping:

- `tally()` walks every item to count what is under one heading, and is called
  once per heading. That is O(groups × items) — at 5 000 items and 100
  milestones, half a million comparisons per keystroke.
- `item_named()` is a linear scan over every item, called once per group to
  find the item a group key names.

At 5 000 items a keystroke costs 15 ms, which is survivable. The problem is
the shape: at 20 000 it is four times worse again for five times the data, and
a filter box that stutters is a filter box people stop typing into.

## Proposal

One pass over the items building every group's tally at once, and a
key-to-item index built with `by_id` rather than scanned per group.

Both are bookkeeping rather than algorithms — the counts are already being
computed, just once per heading instead of once. Neither changes a result, so
the existing tests are the proof, and the table above becomes a test with a
budget rather than a paragraph.

## Cost

A budget in a test is a flaky test on a loaded CI runner. Assert the *shape*
rather than the wall clock: rebuilding a backlog ten times larger must not
cost more than about ten times as much. That fails on a quadratic and passes
on a slow machine, which is the right way round.

## Acceptance criteria

- [ ] Rebuilding 5 000 items costs about ten times rebuilding 500, not thirty
- [ ] `tally` is one pass over the items, not one per heading
- [ ] A group finds the item it names without scanning the backlog
- [ ] A test holds the scaling, and fails on a quadratic rather than on a slow machine
- [ ] No screen changes: the recorded snapshots are the proof
