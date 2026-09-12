---
id: 67
title: Tick a criterion that has come true
type: feature
status: backlog
milestone: v0.5
depends_on:
- 64
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: write
---

## Problem

Acceptance criteria are the unit of progress in a cairn item. They decide
`cairn next`, they are half of what a milestone's percentage means, and the
agent loop has a step for them: *tick what is true, not what would let you
close*.

harrow draws them as a progress bar you cannot touch.

So the one judgement in the loop that is explicitly a person's — is this
actually true? — is the one thing harrow makes you leave to do.

## Proposal

The criteria are a list in the detail pane, each tickable, sent as
`cairn tick <ID> <N>`.

## Depends on cairn

`cairn tick` is written and not yet released: `src/cmd/tick.rs` exists in
cairn's tree, `mod.rs` declares it, and the binary does not offer it. So
harrow has to ask whether the cairn it has can do this, the way it already
asks whether there is a cairn at all, and offer the gesture only when the
answer is yes. A key that reports *unrecognized subcommand* is worse than no
key.

That probe is worth having for its own sake: it is the first time harrow has
had to care which cairn it is talking to, and it will not be the last.

## Acceptance criteria

- [ ] Criteria are listed individually, not only counted
- [ ] One can be ticked, through `cairn tick`
- [ ] The gesture is offered only where the installed cairn supports it
- [ ] A cairn too old to tick says so once, rather than failing per keystroke
