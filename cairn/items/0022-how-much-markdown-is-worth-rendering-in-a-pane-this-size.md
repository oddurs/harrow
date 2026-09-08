---
id: 22
title: How much Markdown is worth rendering in a pane this size
type: spike
status: backlog
milestone: later
created: 2026-09-08
updated: 2026-09-08
priority: p2
area: chrome
---

## Question

The detail pane renders headings, checkboxes and text. It strips `**` and
backticks rather than styling them. Where does that stop being enough?

## Why it has to be answered before the work

A full Markdown renderer is a large dependency and a lot of surface for
something read in a forty-column pane. Doing half of it well is plausible; doing
all of it is a different project.

## Options

- Leave it. Headings and checkboxes are the structure cairn's templates create.
- Style inline emphasis and code, and nothing else.
- A real renderer, wrapped to the pane.

## What would settle it

Reading a dozen real item bodies from other people's projects and seeing what is
actually in them.

## Answer

<!-- Filled in when the spike closes. -->
