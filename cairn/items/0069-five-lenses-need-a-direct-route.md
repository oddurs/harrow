---
id: 69
title: Five lenses need a direct route
type: feature
status: backlog
milestone: v0.5
depends_on:
- 64
- 65
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: chrome
---

## Problem

0062 added `shift-tab` and recorded a reason for adding no direct key per
lens: with a way back, every lens is one press from every other, so three
more bindings would buy nothing. It wrote a test asserting exactly that, so
that the reason would expire on its own rather than be quietly outgrown.

v0.5 adds two lenses. The test fails. That is the mechanism working, and the
argument is now spent: with five, reaching the far one costs two presses and
the cost depends on where it sits in an array.

## Proposal

A direct key per lens. 0062 left the question of which open and named the
tension: the digits are free, positional and learnable in a second, but they
are the obvious keys for a future *jump to the nth thing*.

With five lenses the positional argument gets stronger and the alternative —
initials — gets worse: `l` is advance, `s` is status, `b` and `w` are free
but arbitrary. Decide, bind, and leave the reasoning where the next person
will find it.

## Acceptance criteria

- [ ] Every lens has a direct key
- [ ] The choice is recorded with what it cost, including what the keys are no longer available for
- [ ] The one-press test from 0062 is replaced by one that holds the new rule
- [ ] The help lists them, generated from the bindings as it already is
