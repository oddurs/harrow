---
id: 54
title: The derived theme paints chrome in the warning colour
type: bug
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: theme
---

## What happens

`Theme::from_palette` builds the `auto` theme out of the terminal's own
colours, and then says:

```rust
accent: warn,
```

So harrow's chrome is painted in the warning colour. The selected tab, the
cursor in the filter box, every key in the footer hints, the `•` that marks
an item as just moved, the accent half of a progress bar — all the same
yellow as the read-only marker, a stale claim, and anything else that means
*look at this, something is off*.

A colour that means "this is where you are" and a colour that means "this is
wrong" cannot be the same colour. Accent is everywhere and warn is
exceptional, and when the ubiquitous one wears the exceptional one's colour,
the exceptional one stops being visible at all.

## What the project already decided

Both hand-written themes separate them:

| | accent | warn |
| --- | --- | --- |
| gotham | `#edb54b` amber | `#d26939` orange |
| paper | `#2a6f9e` blue | `#9a6410` brown |

So this is not a question of taste to be settled — the authored themes are
the design, and the derived one does not meet it.

## Proposal

`accent` takes the same hue as `border_focus`, which is `secondary`. Focus
and accent are the same idea — *this is where you are, this is what responds*
— and paper already paints them identically. Yellow keeps `warn` and
`active`, which are both attention and neither is chrome.

Six hues cannot give eight roles one each, so accent has to share with
something. Sharing with the focus ring costs nothing, because the two are
never making different claims. Sharing with the warning costs the warning.

The durable half is a test: a small set of roles that must never collapse
onto one colour, asserted for every theme harrow can produce. That is what
stops the next derivation from quietly reintroducing it.

## Acceptance criteria

- [x] A derived theme's accent is not its warning colour
- [x] Nor its error colour
- [x] A test holds the roles that must stay distinct, for every built-in and for a derived palette
- [x] The hand-written themes are unchanged, since they already got this right
