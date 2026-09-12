---
id: 35
title: Write down why, without leaving
type: feature
status: backlog
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

Marks apply, the way they do to every other write: noting the same thing
against six items is a real triage gesture.

## Cost

A one-line input cannot write a paragraph, and somebody will want to. That is
the trade — `e` is still there, and the alternative is a modal editor inside a
TUI, which is a program harrow is not.

## Acceptance criteria

- [ ] A key takes a line of text and appends it through `cairn note`
- [ ] The note appears in the detail pane without a reload
- [ ] It is refused, with the existing read-only message, when cairn is not available
- [ ] With items marked, it notes all of them
