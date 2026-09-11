---
id: 19
title: Show what changed, from the repository's own history
type: feature
status: done
milestone: v0.2
created: 2026-09-08
updated: 2026-09-11
priority: p2
area: chrome
---

## Problem

An item file is versioned, so its history is the answer to "when did this become
p0, and who decided that?". `cairn log` prints it. From here you have to leave to
see it.

## Proposal

A pane over `cairn log`, or over `git log --follow` on the item's path. Read
only: the history is the repository's, and harrow has no business editing it.

## Acceptance criteria

- [x] The history of one item, without leaving
- [x] A project that is not in git says so rather than showing nothing
