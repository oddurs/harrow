---
id: 131
uid: 11cd5d9b-b63e-4371-9251-6b77d9077bea
title: An accepted proposal is no longer waiting
type: bug
status: done
milestone: v1.0
assignee: Oddur Sigurdsson
created: 2026-11-27
updated: 2026-11-27
closed_at: 2026-11-27
priority: p1
area: read
effort: s
---

## What happens

harrow lists a proposal as waiting after it has been accepted. It also lists
one whose field already holds the proposed value. The needs lens, the count
of proposals, the row mark, the detail pane and `A` all treat it as open.

harrow reads every `## Proposed` heading in the body. `cairn proposals
--accept` leaves that heading in place and adds an `## Accepted` note, so the
heading alone does not say what is waiting.

## What should happen

harrow applies cairn's rule from `cairn proposals`: a proposal is open only
while the field differs from the proposed value. A proposal that has landed is
history. The body still shows it, with the `## Accepted` note under it.

## Reproduction

1. In a project where `cairn proposals` lists one proposal, and the bodies
   carry three more that were accepted or already applied, run
   `harrow --plain --lens needs`.
2. harrow prints four proposals. cairn prints one.

## Acceptance criteria

- [x] A proposal whose field already holds the proposed value is not in `Item::proposals`
- [x] The needs lens leaves out an accepted proposal, shown by a test that fails without the fix
- [x] A proposal whose field still differs stays waiting
- [x] `harrow --plain --lens needs` and `cairn proposals` agree on a real backlog with accepted proposals

## 2026-11-27

Filtered once, in item::parse, against the raw frontmatter, the way cairn compares item.get(field).display() with the proposed value. Every reader of Item::proposals (needs lens, count, row mark, detail pane, A) follows from that one place. The detail pane no longer draws a settled proposal in its Proposed section, because that section offers A to accept; the body still shows the heading and the Accepted note as history. On the horsegirl backlog, harrow --plain --lens needs went from 0118, 0125, 0146, 0160 to 0125 only, which matches cairn proposals.
