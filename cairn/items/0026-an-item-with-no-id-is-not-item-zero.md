---
id: 26
title: An item with no id is not item zero
type: bug
status: done
milestone: v0.2
depends_on:
- 39
created: 2026-09-12
updated: 2026-09-12
priority: p0
area: read
---

## What happens

`item::parse` leaves `id` at its default when the frontmatter has no `id:`
key. The file loads, and it loads as item 0.

cairn's conformance corpus has a file for this — `0010-id-from-filename.md`,
whose expected reading is `id: 10` — and harrow reads it as 0.

Two items with no id therefore have the same id. `by_id` keeps one of them, a
`depends_on: 0` anywhere resolves to whichever survived, and `cairn set 0 ...`
would be run against the wrong file if one were selected. Nothing warns.

## What should happen

Specification §4.1 permits taking the id from a leading run of digits in the
filename, and §4.2 requires that a reader which cannot determine the
identifier report the file as invalid rather than guess:

> A reader that cannot determine the identifier **must** report the file as
> invalid rather than guess: an item whose identity is unknown is worse than
> an item that fails to load.

So: take it from the filename when the frontmatter omits it, and when the
filename does not begin with digits either, reject the file with the warning
the loader already has a channel for.

## Reproduction

1. `cairn/items/no-id.md` with frontmatter carrying a title and a status and
   no `id`.
2. Open harrow. The item is there, listed, with reference `0000`.

## Acceptance criteria

- [x] An item whose frontmatter omits `id` takes it from the filename's leading digits
- [x] An item whose id cannot be determined from either is a warning, not item zero
- [x] Two such files never collide, because neither loads
