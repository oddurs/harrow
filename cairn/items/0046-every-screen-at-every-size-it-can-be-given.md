---
id: 46
title: Every screen, at every size it can be given
type: chore
status: done
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: testing
---

## Problem

The recorded screens are taken at a handful of sizes — 110×26, 110×20, 62×26,
and two deliberately cramped ones. The randomised suite renders at 100×30 and
24×8. Between those lie every size a terminal actually is, and every overlay
at every one of them.

An overlay is the risk. The reader, the history, the help, the picker and the
confirmation all compute a centred rectangle from the available area and
subtract borders and padding from it. Every one of those subtractions is a
place where a width of three and a height of two produce either a panic or a
box drawn outside itself, and the two sizes the fuzz uses are not enough to
say they do not.

## Proposal

A test that renders every pane and every overlay across a grid of sizes, from
the smallest a terminal can be up to something wide, asserting that the frame
renders and that nothing is drawn outside its area.

Not snapshots — twelve screens times forty sizes is five hundred recorded
files nobody will ever read, and the review value of a snapshot comes from
somebody reading the diff. This asserts a property: it renders, and it stays
inside the lines.

## Acceptance criteria

- [x] Every pane and every overlay renders at every size in a grid down to
      the smallest, and survives being driven there rather than merely drawn
- [x] Nothing is *registered as clickable* outside the screen — which is the
      testable half, and it found 445 such regions. The header lays the tabs
      out at fixed offsets and registered all of them however narrow the
      terminal, so on forty columns the last tab was truncated away and still
      clickable: a click on the blank at the right edge switched to a lens
      that was not on screen. Clipped in `App::hit`, where there is one of it
      rather than a dozen
- [x] The test names the size that failed
- [x] It runs in `scripts/task check` without noticeably slowing it
