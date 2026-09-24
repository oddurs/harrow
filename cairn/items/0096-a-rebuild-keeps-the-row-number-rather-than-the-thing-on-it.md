---
id: 9e1c822d-fd51-4d35-89d5-47f3d7511d88
title: A rebuild keeps the row number rather than the thing on it
type: bug
status: backlog
milestone: v1.0
created: 2026-09-21
updated: 2026-09-21
priority: p2
area: read
---

## What happens

`App::rebuild()` rebuilds `rows` and then calls `clamp()`, which keeps
`self.selected` as a row **index**. Whatever was at that index before is not
what is there after, if anything above it moved.

Found while making headings landable (0095). Setting `group_by` and calling
`rebuild()` directly left the cursor at row 4: item 0003 before the regroup,
the `in progress` heading after it.

```
before regroup: selected=4 item=Some(3)
after regroup:  selected=4 item=None row=Some(Group(1))
```

It had been hidden. A heading could not be stood on, so `clamp` slid off it to
the next landable row, which in that fixture happened to be 0003 again. The
selection looked kept and was not; it was lucky.

## Why it is not simply fixed

`set_grouping` already does it properly: it takes the selected id, rebuilds,
and calls `select_id`. The keys a reader presses for regrouping go through
that, so `v` is fine. But `rebuild()` has seventeen callers, and the fix that
removes the class — `rebuild()` restoring by identity itself — has two hazards
that want looking at rather than guessing:

- **`select_id` is not only a cursor.** It moves the needs queue's `question`
  and sets `needs_anchor`. Calling it from every rebuild would reach across
  lenses.
- **A reload may replace `self.items` before it rebuilds.** If it does, then
  at the top of `rebuild()` the old `rows` hold indices into the *new* items,
  and reading `Row::Item(i)` to learn what was selected reads the wrong item.
  Capturing identity has to happen before the items change, not at the top of
  the rebuild.

## What should happen

Anything that rebuilds keeps the cursor on the same item — or the same
heading, now that a heading is somewhere a cursor can be — unless that thing
is gone, in which case the nearest one that is not.

The external watch is the case that matters: harrow sits in a pane beside
somebody else's edits, and every edit they save rebuilds under the reader. A
cursor that drifts to a neighbour on each save is exactly the kind of small
wrongness that stops a person trusting a screen.

## Done when

- A test saves a change above the selected item from outside, lets the watch
  rebuild, and the selection is the same item.
- The same for a heading.
- The seventeen callers are either routed through the identity-keeping path
  or listed with the reason each does not need it.
