---
id: 82
title: The view says what it is, in one line
type: feature
status: backlog
milestone: v0.6
created: 2026-09-18
updated: 2026-09-18
priority: p1
area: chrome
---

## Problem

The list pane's title says `Backlog · by milestone`. That is one of the three
things that decide what is on screen. The other two are not stated anywhere:
the filter that produced these rows, and the order they are in.

A filter typed into the box is at least visible while the box is open. A
filter that came from `--filter` at startup, or from `--view`, or from
clicking a cell in the status strip, is not: the rows are narrowed and the
screen does not say why. The sort order is never stated under any
circumstance, because there is no way to change it either (0083).

This is the same class of defect as 0080, which was found twice before it was
fixed: a screen that looks true and does not say what it is leaving out. The
strip said seven and the list said nothing, and neither was lying. A view
whose rules are off screen is a view nobody can check.

## Proposal

One line, above the rule, in every lens — because the arrangement is the only
thing that differs between lenses, and what is being shown is not part of the
arrangement.

```
 harrow  poptop    needs  list  board  stats  log    3 needs you        just now
 ◐ 2 doing   ○ 18 backlog   ✓ 70 done   ⊘ 3 blocked
 / status=doing,backlog priority<=p1   S ↓ priority id   v ⊞ milestone   20 of 99
─────────────────────────────────────────────────────────────────────────────────
```

Three segments. The dimmed letter before each is not decoration: it is the key
that edits that segment, so the line teaches its own controls.

This step is **read-only**. No new editing and no new keys — `/`, `f` and `v`
keep doing exactly what they do, and the line states the result. The three
items that follow each take one segment and make it editable.

Where the segments come from:

- the query: `App::filter`, or the name of the view in force, rendered back as
  the grammar it parsed from, so what is shown is what would reproduce it
- the sort: `sort_keys()` — which already falls back to `status,priority,id`,
  a default no reader has ever been told about
- the grouping: `App::group_by`, which is what the title says today
- the tally: `shown_and_total()`, which the filter panel already puts on its
  own edge

**The line is the command.** `y` on it should yield
`harrow -f '…' --sort '…' --group-by '…'`. A backlog narrowed by hand becomes
a command that can go in a script, a README, or somebody else's terminal, and
the interface teaches the command line rather than hiding it.

## Cost

One row of vertical space, in a program whose whole premise is that it runs in
a narrow pane beside the work. The strip already spends a row and is dropped
below twenty-four rows of height; this line should follow the same rule and
for the same reason.

It is worth the row for the same reason the filter panel's `20 of 99` is worth
its corner: a number you cannot check is a number you cannot use.

## Acceptance criteria

- [ ] The filter, the sort and the grouping in force are all on screen, in every lens
- [ ] A filter that came from `--filter`, `--view` or a click on the strip is stated, not just a typed one
- [ ] The query segment reads as cairn's grammar, and reparses to what is shown
- [ ] The line is dropped on a terminal too short for it, the way the strip already is
- [ ] `y` yields the command line that reproduces the view
