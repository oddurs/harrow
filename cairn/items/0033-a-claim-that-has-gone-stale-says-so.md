---
id: 33
title: A claim that has gone stale says so
type: feature
status: backlog
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

cairn 0.2.0 added `project.claim_stale_after`, in days. A claim nobody has
honoured for that long is offered back by `cairn next`, with who holds it and
for how long. Nothing is ever released automatically — the configuration
comment is explicit that stale means *visible*, never revoked, "because taking
work away from somebody slow is worse than leaving it held".

harrow already reads `claimed` off an item and shows `@who` beside it. It has
no notion of the threshold, so the one case the field exists for — an item
that looks taken and is not — looks exactly like an item somebody is working
on right now.

This is the thing harrow is for. It is a window kept open beside the work; the
question it should answer at a glance is what is actually moving.

## Proposal

Read `claim_stale_after`. Where it is set and a claim is older, mark the
assignee — the same treatment a blocked item gets, in the warning colour
rather than the person colour — and say how long in the detail pane: *claimed
by oddur, 9 days ago*. Make it filterable, so `claim=stale` narrows to
exactly the pile that needs a conversation.

Inert when the project has not set it, which is the default and is the correct
default: how long is too long is a property of the project.

## Cost

One more thing computed against the clock, which means one more thing that
cannot be computed during a draw — the derived flag belongs beside `blocked`,
worked out when the set is loaded and refreshed on the tick that already
updates `now`.

## Acceptance criteria

- [ ] With `claim_stale_after` set, a claim older than it is visibly distinct from a fresh one
- [ ] The detail pane says how long it has been held
- [ ] A project that has not set it sees no change at all
- [ ] The filter can select stale claims
