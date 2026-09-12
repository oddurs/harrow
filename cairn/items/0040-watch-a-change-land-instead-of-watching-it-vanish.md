---
id: 40
title: Watch a change land instead of watching it vanish
type: bug
status: doing
milestone: v0.2
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## What happens

The board declares a `done` column and never puts anything in it. The status
strip says `✓ 1 done` and the column beside it says `done 0`, in the same
frame, about the same backlog.

```
 ◐ 1 in progress   ○ 3 backlog   ✓ 1 done   ⊘ 1 blocked
╭ backlog 3 ────────╮╭ in progress 1 ────╮╭ done 0 ───────────╮
```

The cause is that `board_statuses()` draws a column for every status with
`board = true`, while `rebuild` deals only the items `visible()` allows — and
`visible()` drops anything closed unless `a` is held. The column is the
project's, the filtering is the list's, and nobody reconciled them.

Closing an item is worse than wrong, it is unreadable. Press `x`, confirm, and
the row disappears. On the board the card disappears. Nothing lands anywhere;
the item is simply gone, and the only evidence that anything happened is a
toast that has left by the time you look up. Drag a card to `done` and you
watch it evaporate under the pointer.

harrow already knows an item just moved — `changed` records it and `is_recent`
marks it for forty-five seconds — but a row that has been filtered out cannot
carry a mark. The one moment the marker exists for is the one moment it cannot
be seen.

## What should happen

Two rules, one for each view, because the two views are asking different
questions.

**The board is dealt from its own set.** `a` is a question about the list —
whether finished work is worth a row among the work that is left. The board
has already answered it: a status is a column because the project wrote
`board = true`, and `dropped` has no column here because this project wrote
`board = false` on it. Applying the list's rule on top of the project's is
what produced the empty column. So the board takes the filter and the rule
that a container is not a card, and nothing else.

cairn's printed board goes the other way — `column_values` drops a closed
status unless `--all` — and for a printout that is right, because a printout
has no drop target. harrow's board does: dragging a card to `done` is how you
close one with the mouse, and a column that is not drawn cannot be dropped on.

**What leaves the list while you are looking at it is held where it landed
for a moment, and then goes.** An item you close stays on screen for a few
seconds, drawn as what it has become — the done glyph, the done colour, the
mark that says it just moved — and then leaves.

So a close reads as a movement between two states rather than a deletion: on
the board the card moves from one column into the next, and in the list the
row changes under the cursor before it goes.

## Proposal

The board needs no new state: `rebuild` builds a second set of indices beside
the list's, under the board's own rule, sorted the same way so a column reads
down the way a group does.

The list needs one piece: what the last build put on screen. Anything in it
that has *just changed* and no longer belongs starts a short exit, and stays
visible until it expires.

- `visible()` becomes *belongs, or is on its way out*.
- `tick_clock`, which the shell already calls once a frame, ends an exit that
  has run its course and asks for the rebuild that drops the row.

Two conditions on starting an exit, and both matter. It has to have been on
screen: an item somebody moves under a filter it never matched does not appear
for six seconds to announce itself, because that is noise and the toast
already covers it. And the *item* has to be what changed, not the view —
typing a filter is a deliberate act of exclusion, and a list that answered it
by holding on to what you just excluded would be arguing with you.

Six seconds, not the forty-five `is_recent` uses. They answer different
questions: forty-five is *what did I miss while I was looking at the editor*,
six is *did the key I just pressed do the thing*.

## Cost

An item on screen that the filter excludes is a small lie of its own, and
closing forty items at once holds forty rows for six seconds. The mitigation
is that it is brief, it is bounded by what was already on screen, and the row
is drawn as what the item now is rather than as what it was — nothing on
screen is stale, it is only there longer than the filter would have it.

The alternative — animating the card between two columns, showing it in the
old place and the new — needs the item to be in two places at once, and every
index into the list and the board would have to know which of the two it
means. The movement is legible without it: the thing changes where it sits,
and sitting there is what makes the change visible at all.

## Acceptance criteria

- [x] Every column the project declared holds the items whose status names it
- [x] The strip and the board never disagree about how much is done
- [x] A card dragged to `done` lands there and stays visible
- [x] A column reads down in the same order the matching group does
- [x] Closing an item leaves it in the list, drawn as done, for a few seconds
- [x] An item that was never on screen does not appear because it changed
- [x] Narrowing the filter drops rows at once rather than after the delay
- [x] The row leaves on its own, without waiting for the next keystroke
