---
id: 14
title: Document theming and configuration
type: docs
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: docs
---

## Problem

Two files decide how harrow looks and behaves — `~/.config/harrow/config.toml`
and a theme — and neither is discoverable from the interface.

## Proposal

A README that says what this is and why it exists rather than listing features,
and a THEMES.md with the role table, so writing a theme does not mean reading
the source.

## Acceptance criteria

- [x] `harrow config --write` produces a file that documents itself
- [x] The role table is written down, including the ones cairn.toml overrules
- [x] The README says what harrow deliberately does not do
