---
id: 71
title: A filter cairn accepts must not open as an empty backlog
type: bug
status: done
milestone: v0.3
created: 2026-09-13
updated: 2026-09-13
priority: p0
area: filter
---

## What happens

A filter cairn accepts opens in harrow as an empty pane, with nothing saying
anything went wrong. Measured against a real project of 147 items, 26 of them
labelled `thesis`:

| filter | cairn | harrow |
| --- | ---: | ---: |
| `labels~thesis` | 26 | 26 |
| `labels=thesis` | 26 | 26 |
| `label=thesis` | 26 | **0** |
| `kind=optics` | 68 | **0** |
| `nonsense=x` | 0 | 0 |

Both tools exit 0 on all of them and neither says a word.

That is bad on the command line and worse in the interface, because harrow
reads `[[view]] filter` out of `cairn.toml` **verbatim** and cairn owns that
file. So a view cairn accepts, renders into ROADMAP.md and passes `cairn
check` on opens here as an empty list saying *No matches. esc clears the
filter.* — which reads as a true empty set. `harrow --doctor` reports every
line ok and finishes with "harrow can read this backlog", which was true and
useless.

## What is actually wrong, having checked

Two things, not the three it looks like.

**Aliases are missing, and there are two of them.** cairn keeps
`Item::ALIASES = ["kind", "label"]` and resolves `"labels" | "label"` and
`"type" | "kind"` to the same field. harrow knows neither. That is the whole
of why `label=thesis` returns nothing.

**An unknown field matches nothing, silently.** `Query` already collects
`unknown`, and it is surfaced in exactly one place — the footer, while the
filter box is open. Press enter, pass `-f`, or load a view and it is gone.

**`=` against a list already works.** It looked like a third defect and is
not: `labels=thesis` returns 26 in both tools, because harrow's `Op::Eq` over
a `Field::List` already means *any element equals*, which is what cairn's
`eq_field` does. Worth recording so nobody fixes it twice.

## Is harrow's grammar meant to be cairn's?

Yes, and this is the answer that should have been written down before the
bug: **harrow must evaluate everything cairn accepts, and may be louder about
what it cannot.**

It has no choice about the first half. A view's filter is a string the
project wrote for cairn, which harrow reads without asking cairn anything. A
grammar narrower than cairn's does not fail — it silently returns a different
answer, which is this bug.

The second half is a deliberate difference. cairn returns an empty list for
`nonsense=x`; harrow will refuse it. That is not a grammar disagreement —
cairn also thinks the field is invalid, which is what `cairn check` says
about a saved view, and the comment beside `ALIASES` records it learning that
by calling a working view a typo. cairn is lenient at query time and strict
at check time. harrow has no check time, so it is strict at query time.

**Not by sharing code, for now.** The obvious fix is one grammar in one
place, and it is wrong today: cairn is not published, and harrow's whole
architecture is that it reads a backlog with cairn not installed. The
precedent is the item format — harrow is a second implementation held to
cairn's own corpus rather than linked against it. So the same answer: a
differential test. If cairn ever publishes its filter as a library, revisit.

## Acceptance criteria

- [x] An unknown field in `-f` is an error, named, with a non-zero exit
- [x] An unknown field in a `[[view]]` filter is reported and the view refused, never applied as an empty result
- [x] The interface never says "No matches" for a filter it could not parse
- [x] `label` and `kind` resolve, as they do in cairn
- [x] `--doctor` parses every saved view and names any it cannot evaluate
- [x] A test asserts harrow and cairn return the same ids for the same filter
- [x] What the grammar is meant to be is written where somebody will meet it
