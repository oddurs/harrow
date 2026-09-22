---
id: 95
title: The tree does not fold from the keyboard
type: bug
status: done
milestone: v1.0
created: 2026-09-21
updated: 2026-09-21
priority: p0
area: chrome
---

## What happens

The list is a tree: milestone headings with items under them, and `space` on a
heading folds it away. Headings start expanded.

`is_selectable` says a heading is landable only while it is **collapsed**:

```rust
Some(Row::Group(g)) => self
    .groups
    .get(*g)
    .is_some_and(|g| self.collapsed.contains(&g.key)),
```

So the cursor can never reach an expanded heading, and `space` on an item
marks the item rather than folding its group. Pressed from every landable row,
against every letter, every `ctrl-` letter, space, the arrows, `enter`,
`home`, `<` and `>`, **nothing collapses a group**. The only route is clicking
the heading with the pointer.

Which means the fold is mouse-only in a keyboard-first program — and it is
worse than that, because `:toggle-mouse` exists to hand the pointer back to
the terminal for selecting text (0094). Turn capture off to copy a line and
the tree can no longer be folded at all.

Two smaller things fall out of the same rule:

- **`home` does not go to the top.** Row 0 is an expanded heading, so `First`
  lands on row 1 — the finished-work fold — and the first row of the list is
  unreachable.
- **A heading cannot be read.** `selected_item()` already resolves
  `Row::Group(g)` to `g.item`, so a milestone heading has a detail pane and
  has had one all along. The rule keeps the cursor away from it.

## Why the rule was there

The comment says a heading is skipped so "the cursor would [not] stop on a row
with nothing behind it in the detail pane". That is already untrue for the
collapsed case, which lands fine and shows the milestone. It is only true for
a heading with no item behind it — `no milestone`, or any grouping by status
or priority — and the answer to that is to show the group, which is a thing
worth reading: a label, a count, how much is done, how much is blocked.

## What should happen

A heading is landable whether it is open or shut, because the fold is a
control and a control you cannot reach is not one. `space` keeps doing what
the help already says: *mark it for the next change — a heading folds*.

Where a heading has an item behind it, the detail pane keeps showing it. Where
it has none, the detail pane shows the group rather than `Select an item`,
because the cursor is on something and the pane should say what.

## What was found on the way

Making a heading landable is one line. Three things had been leaning on it
not being, and each needed its own answer.

**A run of marks would fold a group mid-run.** `space` marks and moves on with
a plain `move_by(1)`, which now stops on a heading — so `space space space` to
mark a run would, at the first group boundary, fold the next group instead of
marking. `toggle_mark` now steps to the next *item*, the way `mark_range`
already did for shift-click. A test holds it, and fails if the landability
change is kept without it.

**Back reached past its own heading from a group's first row only.**
`step_group` searched for the last heading before `selected - 1`: from a
group's first item that skipped its own heading, from any other item it did
not. The `- 1` stood in for a heading that could not be stood on. `h` is now
the last heading before the cursor — the parent first, then the group above,
the way a tree goes — and one press from folding what you are in. Its
fall-through to `header + 1` was dead and is gone.

**The pane dropped out under a heading with no item.** `split_off_detail`
hid the detail pane whenever `selected_item()` was `None`, so arrowing across
the headings of a status-grouped list would have moved the whole list sideways
on every crossing. A heading keeps the pane and the pane shows the group:
label, count, progress, blocked, and how to fold it in the key this user has
it on.

The first draft of that pane said `1 hidden by the filter` over a backlog with
no filter on. The fourth item was the milestone itself, which `count` includes
and no row ever shows. It now says `4 items · 3 listed` and gives no reason,
because it does not know the reason and a wrong one is worse than none.

**And one it did not cause.** A test set `group_by` and called `rebuild()`
directly, and the cursor stayed on row 4 — item 0003 before, a heading after.
`rebuild()` keeps the row number, not the thing on it; the old rule hid that
by sliding off the heading onto 0003 by coincidence. The keys a reader
presses go through `set_grouping`, which restores by id, so the test now does
too. The general fix has seventeen callers and two real hazards and is 0096.

## Reproduction

1. Open the list on any grouped backlog. Every heading is expanded.
2. Press every key on the board. No group folds.
3. Press `home`. The cursor lands on row 1, not row 0.
4. Click a heading with the pointer. It folds — so the capability is there,
   behind a device the program does not otherwise require.

## Done when

- A test presses `space` on a heading and the group folds, and presses it
  again and the group opens, with no pointer involved.
- `home` reaches row 0.
- A heading with no item behind it shows the group in the detail pane.
- Snapshots re-recorded, diff read.
