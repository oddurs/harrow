---
id: 90
title: One toolbar, with a dropdown under each part of it
type: feature
status: backlog
milestone: v0.6
depends_on:
- 89
created: 2026-09-18
updated: 2026-09-18
priority: p1
area: chrome
---

## Problem

The toolbar has four controls and four different ways of working:

| | | |
|---|---|---|
| filter | `/` | a text box in the footer |
| filter | `f` | a panel down the left of the body |
| sort | `S` | a text box in the footer |
| grouping | `v` | cycles blindly, no list, no way back except round again |
| views | `V` | a picker centred over the middle of the screen |

They sit next to each other on one line and answer the same kind of question —
*how am I looking at this* — and not one of them behaves like its neighbour.
`v` is the worst of them: it steps to the next axis with no way to see what
the axes are, so finding `assignee` in a project with six of them means
pressing `v` six times and reading the line each time to find out where you
landed.

None of them appears where it belongs, either. A menu that opens in the middle
of the screen to change a thing at the top left is a menu that has lost its
anchor: nothing on screen connects the list to the word it is about.

## Proposal

One dropdown, anchored under the segment it belongs to.

```
 / everything    ↓ status,priority    ⊞ milestone          ○ 11  ✓ 80      91
                                      ╭──────────────╮
                                      │ ▸ milestone  │
                                      │   status     │
                                      │   type       │
                                      │   area       │
                                      │   assignee   │
                                      │   flat       │
                                      ╰──────────────╯
```

The same object for sorting, grouping and views: a list under the word it
changes, `↑↓` to move, `↵` to take it, `esc` to leave, and typing narrows it.
Clicking the segment opens it; clicking a row takes it.

Three things it is not:

- **Not the filter.** A filter is compositional — clauses, ranges, negation —
  and a list of choices cannot express one. `/` stays a text box and `f` stays
  the panel; the filter segment opens the panel it already has.
- **Not a mode.** It is one more modal layer beside the picker and the
  confirm, taking keys first and returning an `Action` like everything else.
- **Not a new list of options.** Sorting offers `App::sort_fields`, grouping
  offers `App::grouping_axes`, views offer the project's own — all three
  already exist and all three come from the schema.

Sorting is the one that needs more than a list: an order has a direction and
can have more than one key. The dropdown shows the direction against each
field and `↵` toggles it when it is already the primary key, which covers what
anybody does ninety per cent of the time. `S` still opens the text box for the
rest, because `-priority,updated,id` is a thing somebody should be able to
type.

## Cost

A fourth modal layer, and a mechanism that has to know where it was anchored —
which is state the renderer computes and the core cannot. The anchor is
recorded the way every other hit is: as it is drawn, so the two cannot
disagree.

The centred picker stays for what it is for: choosing a value to *write* to an
item. A dropdown under a toolbar and a picker over the middle then mean two
different things — changing the view, and changing the backlog — which is the
same split the whole milestone is about.

## Acceptance criteria

- [ ] Sorting, grouping and views open a list under the segment they belong to
- [ ] `v` shows the axes rather than cycling blindly through them
- [ ] Typing narrows the list; `↵` takes it; `esc` leaves it unchanged
- [ ] The sort dropdown shows and toggles direction
- [ ] Clicking a segment opens its dropdown, and clicking a row takes it
- [ ] The dropdown stays on screen when its segment is near the right edge
