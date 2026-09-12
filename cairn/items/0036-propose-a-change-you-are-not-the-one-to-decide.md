---
id: 36
title: Propose a change you are not the one to decide
type: feature
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: write
---

## Problem

harrow can accept a proposal and cannot make one. `A` runs
`cairn proposals --accept`; there is no way to propose.

That is the half of the feature a person does not need and an agent does —
except that cairn 0.2.0 enforces the permission model in the direction nobody
expected. A field or status declared `agent = "propose"` is now refused on the
command line for a caller arriving over the protocol, and *allowed* for one
that is not. harrow is not an agent. So harrow can set, directly, fields the
project has said should be proposed.

That is correct as far as cairn is concerned — a person at a terminal is the
person the proposal would have been addressed to — but harrow shows no sign
that a field is one the project treats that way, and offers no way to route a
change through the queue when the person at the terminal is not the one who
should decide.

## Proposal

Two parts, in order.

Show the permission. harrow already reads `agent` on both fields and statuses
and does nothing with it. A field declared `propose` or `read-only` should say
so in the picker rather than looking like every other field.

Then: a modifier on the existing pickers that sends `cairn propose <id>
<field>=<value> --why "..."` instead of `cairn set`, and a place to see the
queue — `cairn proposals` is a list, and harrow's is a filter away
(`proposed=true`) once the list knows how to ask.

## Cost

Two ways to change a field is two ways to get it wrong. The mitigation is that
the proposing one is a modifier on the gesture that already exists rather than
a second set of keys, and the confirmation says which it is about to do.

## Acceptance criteria

- [x] A field the project declares `propose` or `read-only` is marked as such where it is set
- [x] A change can be sent as a proposal, with a reason, through `cairn propose`
- [x] What is waiting to be decided is reachable as a filter, not only item by item
