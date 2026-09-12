---
id: 35
title: Write down why, without leaving
type: feature
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: write
---

## Problem

`cairn note <id> "<text>"` appends to an item's body without replacing it. In
cairn's agent loop it is step 4 of five, and the documentation is pointed
about why it is not `update_item`: "a caller writing down its reasoning must
not be able to erase what came before."

harrow is the surface where a person reads what an agent wrote. It can read
the note and it cannot answer it. The only way to add a sentence is `e`, which
hands the whole file to `$EDITOR` — which drops the terminal, opens the raw
frontmatter, and invites exactly the hand-edit the architecture is arranged to
prevent.

## Proposal

A key that opens the one-line input harrow already has for a new item's title
and for the filter, and sends `cairn note <id> "<text>"`.

One line, not a body editor. The gesture being cheap is the whole value: a
note nobody writes because it costs an editor round trip is a note that does
not exist. Anything longer is what `e` is for.

Marks do not apply, which the item assumed they would. `cairn note` takes one
`<ID>` — no list, no `--filter` — and harrow does not invent a bulk path cairn
has not got, because every change here is one cairn invocation. So a note with
items marked says it goes on one item at a time rather than quietly noting
whichever one the cursor happened to be on.

## Cost

A one-line input cannot write a paragraph, and somebody will want to. That is
the trade — `e` is still there, and the alternative is a modal editor inside a
TUI, which is a program harrow is not.

## Acceptance criteria

- [x] A key takes a line of text and appends it through `cairn note`
- [x] The note appears in the detail pane without a reload
- [x] It is refused, with the existing read-only message, when cairn is not available
- [x] With items marked, it says a note goes on one item rather than guessing
