---
id: 94
title: The map is wrong in six places
type: bug
status: doing
milestone: v1.0
created: 2026-09-20
updated: 2026-09-20
priority: p1
area: chrome
---

## What happens

harrow spends five lenses, a toolbar and a 41-row overlay telling a reader
where things are. The telling has drifted from the thing told, in four places,
and nothing in the suite would notice.

**A. The help lists one command twice.** `Command::help_order()` names
`Command::SortBy` at index 9 and again at index 31. It renders in both
columns of the overlay, same key, same sentence:

```
 ctrl-s        type an order, in --sort's ow…   ctrl-s   type an order, in --sort's ow…
```

**B. `tab` says views, and so does `V`.** The footer hint for `ViewBoard` is
the string `"views"` (`keys.rs:567`), so the bottom line reads `tab views`.
`Command::Views` — `V`, the project's saved cairn views — is also `"views"`
(`keys.rs:180`). One screen, one word, two referents. Every other place in the
project calls the first one a **lens**: `CLAUDE.md`, `tests/lenses.rs`, the
`Pane` doc comments, and the overlay's own row, *the next lens, or the one
before it*. The footer is the only holdout, and it collides.

**C. A trap with no posted way out.** `term.rs:87` starts with
`mouse: true`, so drag-to-select is dead in the terminal from the first frame.
The way out is `:toggle-mouse`, which has no key, no footer hint and no help
row — `help_rows()` skips keyless commands, so it cannot appear there even in
principle. A reader who wants to copy a line of body text drags, gets nothing,
and is told nothing. `y` and `Y` cover an item's reference and the view as a
command line; arbitrary text on screen has no path at all.

**E. Paging is bound and undocumented.** `PageDown` and `PageUp` hold
`pgdn`/`ctrl-d` and `pgup`/`ctrl-u` — the four keys everyone reaches for in a
list — and appear in no row of the overlay.

**F. A row can shove the column it is drawn in.** `draw_help` pads the key
column to `clamp(8, 14)` and does not truncate it, because half a key name is
no use. So a row wider than fourteen is not clipped; it pushes its own
description right and takes the right-hand column out of register. Nothing
says fourteen anywhere near the rows that have to fit it.

**D. Nothing holds the help to the commands.** `Command::ALL` has 55 entries
and `help_order()` has 41. The difference is mostly deliberate — paired keys
share a row, and four commands are `BY_NAME_ONLY` — but no test says so, which
is why A survived. `tests/lenses.rs` already does this job for lenses: name
the contract, assert it for every member, and make an omission fail.

## What was found on the way

E and F were not in the audit. E fell out of D's test on its first run, which
is the argument for writing D at all. F fell out of fixing E: the honest row
for paging is `pgup/ctrl-u, pgdn/ctrl-d`, which is twenty-four characters, and
drawing it broke the overlay it was meant to improve. The row now names the
two keys people look for and the `ctrl-` forms stay bound, which is the one
place in the overlay that does not list every alias — a smaller cost than a
column out of register.

`ctrl-r` (reload the config and theme) also has a key and had no row, beside
`r` (re-read the backlog) which did. Its own row now.

## What should happen

A is a deletion. B is one string. C needs a decision rather than a fix, and
the decision is to make the exit visible without changing the default: the
overlay grows a row for the escape hatch, and the toast that fires on capture
already says the right sentence. D is the test that keeps all three paid.

The default stays `true`. Turning capture off by default would take
drag-a-card-to-a-column with it, and that is a real capability somebody chose;
the complaint here is that the trap is unposted, not that it exists.

## Reproduction

1. `?` in any lens. Read the two columns: `ctrl-s` appears in both.
2. Read the footer: `tab views`. Press `V`: a different thing, also views.
3. Drag across any body text in the detail pane. Nothing selects, and no
   affordance on screen says why or what to press.
4. `cargo test` is green through all three.

## Done when

- The overlay lists each command at most once, and `:toggle-mouse` is
  reachable from it.
- The footer says `lenses`, and `views` means only what `V` means.
- A test holds `help_order()` to `Command::ALL`: every command is in the
  overlay, paired with one that is, or excluded by name with a reason, and
  none is named twice.
- Paging has a row, and no row is wider than the column it is drawn in.
- `tests/snapshots/help.txt` re-recorded, diff read.
