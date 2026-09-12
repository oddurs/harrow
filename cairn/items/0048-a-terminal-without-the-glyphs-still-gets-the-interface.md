---
id: 48
title: A terminal without the glyphs still gets the interface
type: feature
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: theme
---

## Problem

The interface is built out of characters a terminal may not have: `○ ◐ ✓ ⊘ ×`
for state, `▰ ▱` for progress, `▾` for a fold, `•` for a change, `↑ ↓ ↵ ·` in
every hint, and rounded box-drawing throughout. On a terminal without them,
every one of those is a replacement glyph, and the interface is not degraded
but destroyed — the state column, which is the first thing the eye reads,
becomes a column of identical boxes.

There is already a test that the glyphs carry the state when colour is gone,
which is the same principle applied to the other axis. The glyphs are the
fallback for colour; nothing is the fallback for the glyphs.

This is not hypothetical for the audience: a serial console, a `TERM=linux`
virtual terminal, `docker run` without a locale, an old PuTTY, CI logs.

## Proposal

One ASCII set beside the Unicode one — `o @ x ! -` for the states, `#` and `.`
for the bars, `>` for a fold, `+-|` for the boxes — chosen so the state column
stays as scannable as it is now.

Detected rather than configured, from the locale the way every other terminal
program does it, and overridable both ways with a flag and a config key,
because detection is a guess and the user knows.

The recorded screens gain one ASCII rendering, which is also the clearest way
to review whether the substitution reads.

## Cost

Two character sets is two ways every screen can look, and a snapshot suite
that only records one of them is only testing one. One recorded ASCII screen
per pane, not per case.

## Acceptance criteria

- [ ] An ASCII rendering of every glyph, chosen so the state column is still scannable
- [ ] Detected from the locale, and overridable with a flag and a config key
- [ ] One recorded screen per pane in ASCII
- [ ] Box-drawing degrades with it, rather than being handled separately
