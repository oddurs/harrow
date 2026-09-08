---
id: 11
title: Wear the terminal's own colours
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: theme
---

## Problem

A tool that hardcodes a palette overrides the one you already chose.

## Proposal

Colour as roles, defaulting to the terminal's own ANSI slots, with theme files
for anything else — including Ghostty's, read directly.

The wrinkle harrow has and quarry does not: the *project* has opinions too.
`cairn.toml` may say a bug is red and `doing` is yellow. Where it does, it wins;
the theme fills in the rest.

## Acceptance criteria

- [x] `auto` names no absolute colour anywhere
- [x] cairn.toml's own colours are honoured, and a theme can overrule them
- [x] `mono` carries every state in a glyph instead
