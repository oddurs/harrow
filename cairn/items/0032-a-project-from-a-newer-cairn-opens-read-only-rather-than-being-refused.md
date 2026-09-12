---
id: 32
title: A project from a newer cairn opens read-only rather than being refused
type: feature
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: read
---

## Correction

**The premise was wrong.** harrow does not refuse a newer project — `parse`
logs a diagnostic and carries on, and it always has. That first acceptance
criterion was met before the item was written, and it is ticked below because
it is true, not because anything was done for it.

The rest of the item stood, and was the work: the warning went only to the
diagnostics overlay, and every write was attempted as normal. So harrow would
hand cairn a write against an unmigrated project, and the reader would get
cairn's refusal with no idea harrow had known all along.

## Problem That was the right reading of specification §8 clause 4 —
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

Say it where it can be seen, and refuse the write.

harrow already had a read-only mode with a footer marker and a refusal toast,
for the case where cairn is not on PATH — but it was a `bool` with one
hardcoded sentence. A second reason needs the reason itself to travel, so
`writable: bool` becomes `readonly: Option<ReadOnly>`: `NoCairn`, or
`Format(n)`. The footer says which, so the marker answers *why* rather than
only *that*, and the refusal names `cairn migrate`.

Missing cairn outranks a newer format, because without cairn there is no
`cairn migrate` either.

Nothing needs doing about items that cannot be read: they are refused per
file already, and no format bump has ever changed what a key in an item
means — each changed only how the configuration says what it says.

## Cost

A project really could bump the format for a change that alters what an item
means, and then harrow would be reading it wrong while saying only that the
schema is new. §8 clause 2 says that requires a new version *and a migration
path*, and clause 3 says unknown keys are preserved, so the shape of that
change is one harrow would survive. The read-only banner is the hedge.

## Acceptance criteria

- [x] A project declaring a later format opens, lists and reads — already true, see the correction
- [x] The screen says the schema is newer than harrow knows, without a toast that scrolls away
- [x] Every write is refused, naming `cairn migrate`
- [x] A project whose items cannot be parsed is still refused rather than half-read
- [x] Read-only carries its reason rather than a flag with one hardcoded sentence
