---
id: 91
title: The toolbar draws a glyph the terminal has not got
type: bug
status: done
milestone: v0.6
created: 2026-09-19
updated: 2026-09-19
priority: p1
area: chrome
---

## What happens

The grouping segment draws `⊞` — U+229E SQUARED PLUS — and a terminal whose
font has not got it draws a box instead. So the toolbar reads

    v ▯ milestone

and the first thing on the line is a character that is not anything.

## Why it is there at all

Two of the three segments were given a decorative mark to say which axis they
were: `⊙` for the filter and `⊞` for the arrangement. Counting the glyphs in
every recorded screen says how that went:

```
  ▾  U+25BE     32   BLACK DOWN-POINTING SMALL TRIANGLE
  ⊞  U+229E     23   SQUARED PLUS          ← new, and broken
  ⊙  U+2299      1   CIRCLED DOT OPERATOR  ← new
```

Every other glyph harrow draws has been on screen for months. The only two
that are new are the only two that break, and both were introduced in one
change without being checked against anything.

## What should happen

**Drop them.** The marks were never carrying the meaning — the key letter
beside each segment already says which control it is, and it is ASCII, so it
cannot fail to draw.

**Say it opens a list.** What the segments actually lacked is the one thing a
dropdown control has to signal, and there is a glyph for it that harrow has
drawn thirty-two times already: `▾`. A caret at the end of a segment is what
every interface that has ever had a dropdown uses, and it is the affordance,
not decoration.

```
 f everything ▾   S status priority id ▾   v milestone ▾      ○ 11  ✓ 80
```

**Connect the list to its word.** While a segment's list is open, its caret
turns — `▴` — and the segment takes the selection colour. Then the thing that
opened and the word it came from are visibly one control, which is the whole
argument for anchoring the list under the segment in the first place.

**Make them read as controls.** Each segment sits on `surface`, a shade above
the page, so the row reads as a row of things you can press rather than a
sentence. It degrades to nothing under `auto`, where surface and background
are the same — which is why the caret has to carry the affordance on its own.

## And a guard

A test that reads every recorded screen and fails on a character outside a
list of what harrow is known to draw. Adding a glyph then means adding it to
that list on purpose, with somebody to ask whether the terminals this runs in
have got it. 0048 is the general version of this problem; this is the cheap
half that stops it getting worse.

## Acceptance criteria

- [x] Nothing on screen uses a glyph outside the set harrow already drew
- [x] Each dropdown segment says that it opens a list
- [x] The segment whose list is open is visibly the one it came from
- [x] A new glyph cannot reach a recorded screen without being declared

## 2026-09-19

Counting the non-ASCII in every recorded screen was the whole diagnosis: every glyph harrow draws had been on screen for months except two, and the only two that were new were the only two that broke. `⊞` twenty-three times, `⊙` once, both added in one change, neither checked against anything.

The marks were never carrying the meaning. The key letter beside each segment already says which control it is and is ASCII, so it cannot fail to draw. What the segments actually lacked is the one thing a dropdown has to signal, and `▾` — which harrow had drawn thirty-two times in its group headings — is what every interface that has ever had a dropdown uses for it. It turns to `▴` while the list is open and the segment takes the selection colour, so the list and the word it came from read as one control.

The filter segment is keyed to `f` rather than `/` now: the caret promises a chooser and the panel is the chooser. `/` still types one.

The guard failed its own first test. Written against a set of rendered states, it passed a deliberately broken `⊞` because no state it rendered had the grouping off — the one arrangement with a word of its own. Three more states later it catches it. A guard is only as wide as what it draws, and that is worth knowing about this one before trusting it.
