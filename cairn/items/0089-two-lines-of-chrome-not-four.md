---
id: 89
title: Two lines of chrome, not four
type: feature
status: done
milestone: v0.6
created: 2026-09-18
updated: 2026-09-18
priority: p1
area: chrome
---

## Problem

Four rows before any backlog:

```
 harrow  poptop    needs  list  board  stats  log                      just now
 ○ 11 backlog   ✓ 80 done
 / everything   S ↓ status priority id   v ⊞ milestone                 91 items
────────────────────────────────────────────────────────────────────────────────
```

On the eighty-by-twenty-four terminal this is meant to run in, that is a sixth
of the screen spent before anything is shown. Each row was added for a reason
and none of them was weighed against the others.

What is actually being said, item by item:

- **`harrow`** — the name of the program you just launched, beside the name of
  the project, in a window you opened. It is already dropped below eighty
  columns, so the judgement that it is optional has been made once already.
- **`just now`** — when the backlog was last read. harrow watches the files
  and re-reads them on any change, so this says *just now* essentially always.
  A fact that is almost always the same is not worth a permanent place.
- **`○ 11 backlog   ✓ 80 done`** — the status strip, which is a *filter
  control*: its own doc comment says each count is exactly what clicking it
  would give you. It is spelling out words that the glyphs already carry.
- **`/ everything`** — thirteen columns to say that no filter is in force.
- **`91 items`** — the tally, which is the same kind of thing the strip is.

Two rows are counting, on two different lines, in two different shapes.

## Proposal

Two rows, and a rule.

```
 poptop   needs  list  board  stats  log                          6 needs you
 / filter    ↓ status,priority,id    ⊞ milestone     ○ 11  ✓ 80          91
────────────────────────────────────────────────────────────────────────────────
```

- The program's own name goes. The project names the window.
- Freshness appears **only when it is not fresh** — which is the only time it
  was ever telling anybody anything.
- The strip joins the toolbar, compacted to glyph and count. It is a filter
  control and the toolbar is where the filter lives; the words `backlog` and
  `done` are what the glyphs are for. It stays clickable.
- A segment with nothing chosen shows the key that would choose something;
  one with a choice shows the choice. When there is something to say, say it;
  when there is not, say how to change it.

## Cost

The strip loses its words, and somebody who does not know `○` from `✓` has to
learn four glyphs. They are already learning them: every row in the list is
drawn with them, and the help overlay names them.

A row is the right thing to spend here. Four rows of chrome on a
twenty-four-row terminal is the interface talking about itself.

## Acceptance criteria

- [x] The chrome is two rows and a rule
- [x] Freshness is shown only when the backlog is not fresh
- [x] The status counts are still clickable and still filter
- [x] A segment with nothing chosen says which key chooses
- [x] Everything still drops from the right in the order that keeps the useful end, at eighty columns and at forty

## 2026-09-18

The strip is a filter control — its own doc comment says each count is exactly what clicking it would give you — so it belongs on the toolbar with the other filter controls rather than on a row of its own. Losing its words cost nothing: `backlog` and `done` are what `○` and `✓` are for, and every row in the list is already drawn with them.

The tally now appears only when something is narrowing. Unfiltered it read `91 items` beside counts that already added up to ninety-one, which is a third status with a number on it.

Freshness had a permanent corner of the header to say `just now`, which is what it said essentially always, because harrow watches the files. It appears after thirty seconds, which is the only time it was ever telling anybody anything.

Segments show the key *and* the value, and the key is what goes first when the room runs out — it is the half you only need once. That falls out of laying the right-hand end out before the left, so the segments know what room they are laying themselves out in.
