---
id: 32
title: A project from a newer cairn opens read-only rather than being refused
type: feature
status: backlog
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: read
---

## Problem

harrow refuses to open a project whose `format` is higher than the one it
knows, and says so. That was the right reading of specification §8 clause 4 —
"a reader encountering a version it does not understand must refuse the
project and say so" — but §8 qualifies it two paragraphs earlier:

> The version covers the on-disk shape of a **project**, not only of an item.
> [...] A reader of items alone is unaffected by such a change, and **should**
> say so rather than refusing a project it can in fact read.

Every format bump so far — 1 to 2, 2 to 3 — changed only the configuration.
NEWS for both says *no item file changes*. cairn 0.2.0 adopted the same
position for itself: "A project written by an older cairn is read, not
refused. Every command that only reads works against it; writing asks for
`cairn migrate`."

harrow is almost entirely a reader, and the one thing that would genuinely
break — writing — already goes through cairn, which will refuse for itself
with a better message than harrow can give.

## Proposal

Open a project written in a newer format, read every item, and say plainly at
the top of the screen that the schema is from a format harrow does not know
and that parts of it may be missing. Refuse to write, the way harrow already
refuses when cairn is not installed, and name `cairn migrate` as what changes
that.

Keep refusing when the *items* cannot be read, which is what clause 4 is
actually protecting: misreading data is worse than declining to read it.

## Cost

A project really could bump the format for a change that alters what an item
means, and then harrow would be reading it wrong while saying only that the
schema is new. §8 clause 2 says that requires a new version *and a migration
path*, and clause 3 says unknown keys are preserved, so the shape of that
change is one harrow would survive. The read-only banner is the hedge.

## Acceptance criteria

- [ ] A project declaring a later format opens, lists and reads
- [ ] The screen says the schema is newer than harrow knows, without a toast that scrolls away
- [ ] Every write is refused, naming `cairn migrate`
- [ ] A project whose items cannot be parsed is still refused rather than half-read
