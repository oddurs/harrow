---
id: 76
title: Read a ticket in a panel, not over the backlog
type: feature
status: done
milestone: v0.2
created: 2026-09-15
updated: 2026-09-15
priority: p1
area: chrome
---

## Problem

`↵` opens the whole item in a popover centred over everything. 0024 argued
against exactly this shape and then left it standing: *reading three more lines
of a proposal means opening an overlay over the list you were reading it
against, which is the one thing a two-pane layout is supposed to avoid.* It
gave the detail pane a scroll so `↵` would become a comfort rather than the
only way to see the end of an item — but the popover it was arguing with is
still the thing `↵` opens.

Three things follow from it being a popover rather than a panel.

It is modal. Any key that is not a scroll key closes it, so there is no moving
through the backlog while reading: every item costs an open and a close, and
the cursor you left behind is invisible while you are in there. Reading the
four items in a milestone is eight gestures and four screens that each forget
the last.

It shows the least about where an item stands. The popover has the id, the
title and the body. Status, type, milestone, priority, area, assignee, the
stale-claim warning and the criteria count are all in the *detail pane* — which
the popover is covering. The fullest view of an item is the one that says least
about it.

It is one size. `92` columns wide and `height - 4` tall whatever the terminal
is, so a wide screen wastes two thirds of itself on nothing and a short one
gets a letterbox.

## Proposal

Reading becomes a panel in the body, the way Linear reads: browse on the left,
inspect on the right, and the right follows the left.

- `↵` slims the lens and gives the rest of the body to a reader panel. `esc`
  puts the width back.
- **Placement is decided by the room**, not fixed. Wide enough for both and the
  lens keeps a browsing column beside the reader. Too narrow for two and the
  lens steps aside for as long as you are reading — the same judgement
  `split_off_detail` already makes for the detail pane, with a different
  threshold because a reader wants more than a summary does.
- **Metadata up top**: title, then where it stands, then the fields that answer
  *why is this p1 and who has it* — held in a block above a rule, so the body
  below it is uninterrupted prose.
- **The body is held to a measure.** A panel ninety columns wide is not ninety
  columns of readable line; text that runs edge to edge is the thing that makes
  a wide terminal worse to read in than a narrow one.
- `↑↓` move the selection and the reader follows, because a panel is not a
  modal. `J`/`K` scroll the reader, which is what they already do to the detail
  pane. The scroll offset stays keyed to the item, so stepping back onto one
  returns you to where you had got to.

The detail pane is not drawn while reading. It and the reader show the same
item, and showing it twice is two answers to one question.

The history overlay stays an overlay. It is a short, modal answer to *what
happened to this*, not a surface you browse from, and 0075 just gave it the
bound and the mouse it was missing.

## Cost

Two things get worse before they get better.

`any other key closes` was discoverable — you could not get stuck. A panel has
to be left deliberately with `esc`, which is one thing to know. The footer says
so while the panel is open.

The lens loses width while reading. That is the trade the arrangement makes,
and it is why placement is responsive rather than fixed: below the threshold,
the lens gives up all of it rather than both being unusable.

## Acceptance criteria

- [x] `↵` opens a reader panel in the body rather than an overlay over it
- [x] With room for both, the lens keeps a browsing column beside the reader
- [x] Without room for both, the lens steps aside until reading ends
- [x] The panel shows status, type, milestone, priority, area and assignee
- [x] The body is held to a readable measure however wide the panel is
- [x] Moving the selection while reading moves what the panel shows
- [x] The detail pane is not drawn beside the reader
- [x] `esc` closes the panel and puts the lens back
