---
id: 68
title: Whose work is whose
type: feature
status: done
milestone: v0.5
depends_on:
- 64
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

cairn records who did everything. `whoami` is `CAIRN_USER`, then git's
`user.name`, then the login; over the protocol it is the name the client gave
in its handshake, so an agent's claims appear under the agent's name. The
model separates `assignee` from `owner` precisely so that a program working
and a person answerable are two facts.

harrow shows `@name` in the muted person colour and nothing else. On a
backlog with you and two agents on it, whose work is whose — the first-order
question of supervision — is something you read one row at a time.

## Proposal

An actor is a thing harrow knows about, not a string it prints.

- Each gets a colour, stable across the session, from the theme's own scale.
- `owner` is shown where it differs from `assignee`, because the difference
  is the whole point of there being two fields.
- Yours is distinguishable from everybody else's, since harrow can ask the
  same question cairn does.

## Cost

Colour is already carrying status, type, priority and staleness. Another
dimension on the same row risks a screen that is merely colourful, which is
the failure mode of every dashboard. The rank scale exists and is bounded;
better to reuse it and cap the number of distinct actors than to invent a
scale that cannot fail gracefully.

And it must survive `mono` and `NO_COLOR`, where the glyph has to carry it —
which is the rule the state column already follows.

## Acceptance criteria

- [x] Two actors are distinguishable at a glance, without reading the names —
      as *yours* and *not yours*, which is the first-order question. A colour
      per actor was tried and rejected: the rank scale is the priority scale,
      so a contributor would be drawn in the p0 red for no reason but their
      position in a sorted list, and this row already carries state, type,
      priority and staleness in colour
- [x] `owner` is visible where it differs from `assignee`
- [x] Your own work is distinguishable from everybody else's
- [x] It degrades to something readable with no colour at all: the glyph
      carries it, `@` for yours and `·` for somebody else's
- [x] Where harrow cannot tell who you are, nobody is marked as *not you*,
      because a distinction that cannot be drawn should not be suggested
