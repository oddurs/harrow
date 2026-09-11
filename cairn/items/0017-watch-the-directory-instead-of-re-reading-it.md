---
id: 17
title: Watch the directory instead of re-reading it
type: feature
status: done
milestone: v0.2
created: 2026-09-08
updated: 2026-09-11
priority: p2
area: runtime
---

## Problem

harrow re-reads the item directory every three seconds and sends the result on
only when it differs. A thousand-item backlog costs a few milliseconds, so this
is cheap enough — but it is polling, and it is three seconds of lag behind an
editor save or a `git checkout`.

## Proposal

A filesystem watcher, falling back to the poll where one is not available. The
fallback has to stay: watchers do not work everywhere, and a network mount is
exactly the case where one silently does not.

## What would settle whether it is worth it

Whether three seconds is actually noticeable in use. It has not been yet.

## Acceptance criteria

- [x] A change made in another window shows up without pressing `r`
- [x] The poll is still there and still works where the watcher does not
- [x] No busy loop when something rewrites the directory continuously
