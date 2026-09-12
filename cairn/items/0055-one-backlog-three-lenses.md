---
id: 55
key: v0.4
title: One backlog, three lenses
type: milestone
status: backlog
depends_on:
- 25
created: 2026-09-12
updated: 2026-09-12
priority: p2
due: 2027-03-15
---

Three ways of looking at one backlog, that behave like one program.

## The problem

`tab` presents the list, the board and the stats as peers. They are not. They
are three different interaction models, and the differences are not design
decisions — they are things that were never carried across.

| | list | board | stats |
| --- | --- | --- | --- |
| a cursor | yes | yes | **no** |
| the detail pane | yes, at 96 columns | **no** | **no** |
| the filter applies | yes | yes | **no** |
| the grouping axis | yours, `v` cycles | **fixed to status** | n/a |
| the mouse | rows, headings, drag | cards, columns, drag | **inert** |
| marks | yes | yes | n/a |
| you can act on it | yes | yes | **no** |

Every **no** is something a reader learned in one lens and has to un-learn on
arriving in another. None of them is defended anywhere.

The deeper version: the panes are built as *modes*, and read as *lenses on one
selection*. A reader carries an intention across `tab` — this item, this
filter, these marks — and expects the arrangement to change and nothing else.
What is supposed to be invariant is not.

## The worst of it

`stats()` filters on `!container` and nothing else. Narrow to `priority=p0`,
press `tab` twice, and every number describes the whole backlog instead. The
screen does not say so. Two lenses obey the filter and the third silently
opts out, which is worse than a third that never had one.

And the stats pane is a dead end in a tool built for acting. Every number in
it is a query — *2 ready*, *1 blocked*, *0004 waiting longest*, *1 item is
waiting on it* — and not one of them goes anywhere. The status strip two rows
above does exactly this: click a status, get that filter. The affordance
exists in the header and is absent from the pane devoted to it.

## What this milestone is

Not more views. The same three, made into the same program: one selection,
one filter, one detail, one set of marks, one mouse, and an arrangement that
is the only thing that changes.

0056 writes down what a lens owes the reader and holds it with a test, so the
drift cannot happen again quietly. The rest pay off what has already drifted.
