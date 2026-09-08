---
id: 15
title: Every cairn project on this machine, in one list
type: feature
status: backlog
milestone: later
created: 2026-09-08
updated: 2026-09-08
priority: p3
area: cli
---

## Problem

quarry's whole shape is machine-wide: every server, grouped by the repository it
came from. harrow is deliberately not that — it opens the project you are
standing in, the way git does.

But there are a dozen cairn projects on this machine and no way to ask what is
in progress across them.

## Why this is held rather than planned

A cross-project view is a different tool with a different centre of gravity. It
wants a scan, a cache, and an opinion about which directories to look in — all
of which harrow currently gets to not have. The version of this that is worth
building might be `quarry` for backlogs rather than a mode of this one.

Build it when somebody has actually wanted it twice.

## Acceptance criteria

- [ ] Reconsider once there are two people with more than one cairn project
