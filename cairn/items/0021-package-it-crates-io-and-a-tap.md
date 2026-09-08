---
id: 21
title: 'Package it: crates.io and a tap'
type: chore
status: backlog
milestone: v0.2
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: packaging
---

## Problem

`cargo install --path .` is the only way in, which means the only people who can
use this are the people who have already cloned it.

## Acceptance criteria

- [ ] `cargo install harrow`
- [ ] `brew install oddurs/tap/harrow`
- [ ] A release workflow that builds both from a tag
