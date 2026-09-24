---
id: 5246d741-9d67-4c57-ba31-1823b64bacfd
title: Decide what the 1.0 promise covers
type: decision
status: backlog
milestone: v1.0
owner: oddurs
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: docs
effort: m
---

## Context

v1.0 holds fourteen open items and is called *Stable release*. That is a
bucket, not a milestone: it mixes real promises — 0044 says what is stable,
0047 says which platforms, 0043 ships a man page and completions — with
ordinary defects that happen not to have shipped yet.

Nothing says what *stable* would cover, so nothing can say when v1.0 is
reached. 0100 recorded this and deliberately did not settle it, because the
scope of a compatibility promise is not a side effect of an assessment.

## Options and tradeoffs

The surfaces that could be promised, each with a cost measured in years:

- **The action names** a `[keys]` table binds to. Cheap to hold, and 0044
  already assumes it. `tests/keymap.rs` holds them today.
- **The filter grammar**, which is not harrow's to promise alone: it is
  Cairn's, and `COMPATIBILITY.md` is where the two agree.
- **`--plain` output**, now three shapes across five lenses (v0.7). Promising
  it makes harrow scriptable on purpose rather than by accident; it also
  means a new column is a breaking change.
- **The config file and theme files**, which people edit by hand.
- **The `--screenshot` and `--doctor` output**, which are diagnostics and
  probably should *not* be promised, so they stay free to improve.

Promising everything is a way of promising nothing, because the first
inconvenient one gets quietly broken.

## Decision

Not made. This needs the project owner.

## Revisit when

It blocks: until it is settled, v1.0 cannot be scoped, 0044 cannot be
written, and each new output format is an unbounded commitment taken by
accident.

## Acceptance criteria

- [ ] Each candidate surface is listed as promised or explicitly not
- [ ] `COMPATIBILITY.md` carries the ones shared with Cairn
- [ ] 0044 can be written from this without further argument
- [ ] v1.0's remaining items are scoped against it, or moved out
