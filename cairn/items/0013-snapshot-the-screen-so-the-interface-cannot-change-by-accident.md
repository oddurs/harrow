---
id: 13
title: Snapshot the screen, so the interface cannot change by accident
type: chore
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: testing
---

## Problem

A TUI is the one component that cannot be tested the way the rest is: the suite
asserts on values, and this thing's output is a screen.

## Proposal

Render to a buffer with no terminal involved, and hold the result to a recorded
screen. Two kinds: the text, which says where everything is, and a map of the
foreground colours, which says how it reads. Plus randomised key sequences held
to the state invariants, because enumerating the interactions by hand is
hopeless.

## Acceptance criteria

- [x] `HARROW_UPDATE_SNAPSHOTS=1` accepts a deliberate change; nothing else does
- [x] Colour is asserted, not just layout
- [x] No sequence of keys leaves the state inconsistent or a frame undrawable
- [x] It all runs in under a second, so nobody is tempted to skip it
