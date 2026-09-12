---
id: 25
key: v0.3
title: Read everything cairn writes, and nothing it does not
type: milestone
status: backlog
depends_on:
- 16
created: 2026-09-12
updated: 2026-09-12
priority: p2
due: 2027-02-15
---

harrow reads cairn's files directly. That is the decision the whole program
rests on: the backlog opens in a millisecond, and it opens on a machine where
cairn is not installed. The price is that harrow owns a second implementation
of somebody else's format, and a second implementation that has never been
held to the specification is a rumour about it.

cairn 0.2.0 is the first release with a written, normative format
specification and a conformance corpus beside it. Run harrow's reader over
that corpus and it gets eleven of twelve files right and the twelfth silently
wrong — an item with no `id` becomes item zero, and two of them become the
same item. Read the specification and there are more: a `...` frontmatter
terminator loses the entire body, labels written as one string become one
label, a dependency written `#3` is dropped, subdirectories are never
searched.

Separately, cairn 0.2.0 declares things in `cairn.toml` that harrow does not
look at — how a project renders its identifiers, where its acceptance criteria
live, when a claim has gone stale, what a saved view groups by — and offers
commands harrow never reaches: `note`, `propose`, `release --reason`, `check`.
Every one of them is the project saying something about itself that harrow
currently overrides with a guess.

This milestone is the two halves of that: be right about the files, and honour
what the project declares.

The order matters. The reader bugs lose information and belong in v0.2; what
follows them here is the part where harrow stops inventing defaults for
questions the project has already answered.
