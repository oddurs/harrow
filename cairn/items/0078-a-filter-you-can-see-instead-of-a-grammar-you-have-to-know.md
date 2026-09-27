---
id: 54393457-9403-40be-a785-ac67b80ccb90
title: A filter you can see, instead of a grammar you have to know
type: feature
status: done
milestone: v0.2
created: 2026-09-16
updated: 2026-09-16
priority: p1
area: chrome
---

## Problem

The only way to narrow the backlog is `/` and then cairn's grammar, typed from
memory. 0022 put that grammar in the box on purpose and it was the right call —
an expression that works on the command line has to work in the box. But it is
the *only* way in, and it asks you to already know two things harrow never
shows you: which fields this project has, and what values they take.

A project declares its own statuses, types and fields in `cairn.toml`. That is
the whole point of harrow — nothing about a workflow is hardcoded. It also
means the filter vocabulary is different in every project, and the only way to
learn it is to read `cairn.toml` or to guess and watch the box say *no such
field*.

What is on screen makes it worse rather than better. The status strip shows
every status with a count and filters when you click one, and nothing about it
says so; it reads as statistics. A filter, once applied, appears only as text
you typed — so `status=doing,priority=p1` is legible to whoever typed it and to
nobody else, including you tomorrow.

The result is a backlog browser where the browsing is the part you cannot do
without documentation.

## Proposal

A filter panel, on the left, the way the reader is a panel on the right:
browsing on one side and the thing you are doing to it on the other.

- `f` opens it. It lists every facet the project declares, and under each one
  every value it takes, with a count and a checkbox.
- **The counts are faceted.** A value's count is what you would have if you
  ticked it, given everything else already ticked — and a facet does not count
  against itself, so ticking `backlog` does not drop `in progress` to zero and
  strand you. That is the number that answers *where do I go next*.
- Ticking composes cairn's own grammar into the same filter string the box
  holds: several values of one field become `status=a|b`, several fields
  become clauses. **There is one filter, in one syntax** — the panel writes
  what you could have typed, and typing it shows ticked in the panel.
- **Placement is decided by the room**, the way the reader's is. Wide enough
  for three and the panel sits beside the lens with the detail still there;
  wide enough for two and the detail stands down; too narrow for two and the
  panel takes the body until you close it.
- Values nothing matches are dimmed rather than hidden, because *there are no
  p0s* is an answer and an absence is not.

## Cost

A line of the lens's width while it is open, and a third pane to place on a
wide terminal. Both are the same trade the reader made, and the panel closes.

It does not replace the box. `/` still takes an expression, including the parts
no checkbox can express — ranges, `!=`, free text. The panel covers the common
case, which is picking values off a list somebody else defined.

## Acceptance criteria

- [x] `f` opens a filter panel and `esc` closes it
- [x] Every status, type and declared field appears with its values
- [x] A value shows how many items ticking it would leave
- [x] A facet's own counts ignore that facet's own ticks
- [x] Ticking writes cairn's grammar into the same filter the box holds
- [x] A filter typed in the box shows as ticked in the panel
- [x] The panel is placed by the room available, down to a terminal too narrow for two
- [x] A value nothing matches is shown and dimmed, not hidden
