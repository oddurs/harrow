---
id: 81
key: v0.6
title: Compose a view; reach a command
type: milestone
status: done
depends_on:
- 63
created: 2026-09-18
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

Three questions decide what is on screen — which items, in what order,
grouped how — and harrow can only be asked one and a half of them. This is the
milestone that gives each one a door, and gives the keymap somewhere to put
the next five years of them.

It comes before 1.0 because of one sentence in 0044:

> A 1.0 fixes the config file's keys, **the action names a `[keys]` table
> binds to**, the theme roles a theme file sets, and the command-line flags.

The keymap is about to become a promise. It should be one worth keeping.

## What is wrong with it now

harrow has one flat namespace doing three different jobs. `x` closes an item,
`y` copies an id, `v` changes the grouping, `m` toggles mouse reporting — a
write to the backlog, a read, a change of view, and a debug switch, in the
same space, with nothing in the interface separating them.

That works at thirty commands. At forty-eight, across fifty-eight bindings and
forty-one spent letters, it is why a debug toggle owns a prime letter, why
`check` ended up on `ctrl-k`, and why there is nowhere left to put sorting.

The consequences are not a matter of taste:

- **The backlog answers three questions — which items, in what order, grouped
  how — and harrow can only be asked one and a half of them.** `App::sort`
  exists and `sort_keys()` reads it; nothing but argv sets it. To reorder by
  priority you quit and start again.
- **`h` and `l` advance and retreat status while the arrow keys beside them
  step between groups.** The strongest muscle memory in the tool points at a
  mutation.
- **`/` and `f` edit the same query and cannot see each other's work.** The
  panel cannot express a date bound or a negation, and neither states what the
  other did.
- **The `[[view]]` tables a project declares are unreachable.** harrow reads
  them and `--doctor` validates them; no key switches between them.

## The shape of the answer

Split the namespace by what the act does, into three surfaces.

**The view line.** One always-visible line stating the whole view as a
sentence — query, sort, grouping — with each segment labelled by the key that
edits it. The pane title already says half of it; this is that line promoted
to a control, and it is the same string the command line takes.

**The palette.** A fuzzy list built from `Command::ALL`, its descriptions and
`keys_for()` — the same three things that already generate the help overlay.
It changes the economics of the keymap: a new command lands in the palette and
graduates to a key only once it has earned one.

**The verb set.** The ten direct keys you press hourly, unchanged, applied to
the marks if there are any and the cursor otherwise. Everything rare demoted.

Four keys added (`:` `S` `V` and a corrected `<`/`>`), two repurposed, three
demoted. The point of that tally is how little has to move once the surfaces
exist.

## What is deliberately not here

- **No modes.** A normal/insert split would buy keyspace and cost the property
  that a key does the same thing whenever you press it. The palette buys the
  same room without it.
- **No second grammar.** The view line is cairn's filter grammar verbatim, for
  the reason already in CLAUDE.md: a grammar narrower than cairn's does not
  fail loudly, it silently returns a different answer.
- **No menu bar.** The mouse already reaches everything drawn.
- **Undo is not in this milestone.** 0041 already has it, already at p1 in
  v1.0, and 0002 already names it as a condition of the number. Two design
  points from this work are noted against it there.

## Acceptance criteria

- [x] All three questions — which, in what order, grouped how — are askable without restarting
- [x] The view in force is stated on screen, in the grammar that would reproduce it
- [x] Every command is reachable by name, and the rare ones no longer hold letters
- [x] The keymap is settled, so 0044 can promise it

## 2026-09-18

Planned from a written proposal rather than straight into items, because the argument is about the shape of the whole interface and the items only make sense against it: https://claude.ai/artifact/7hJNAvQQgeM7x5rTHuwSRr

The counts in the body are read from src/keys.rs and src/app.rs at 51c8135 — 48 commands, 58 bindings, 41 letters spent — and they are the reason this is a milestone rather than four loose items. Any one of these on its own is a feature; together they are the difference between a keymap that has run out and one that can absorb the next five years of features.

## 2026-09-23

Reconciled 2026-09-23. All twelve items are done: the toolbar and its dropdowns, saved views reachable by name, the command palette, the view line, and sorting as a key rather than a flag. Closed on the reconciliation date.

## 2026-09-23

Reconciled 2026-09-23. All twelve items are done: the toolbar and its dropdowns, saved views reachable by name, the command palette, the view line, and sorting as a key rather than a flag.

Criteria verified against the shipped binary rather than ticked to clear the gate:

1. Which / in what order / grouped how are all askable without restarting — `f` and `/` edit the query, `S` and `ctrl-s` the order, `v` and `ctrl-v` the grouping; all six are bound in src/keys.rs.
2. The toolbar states the view in force, and `Y` (copy-view) yields the command line that reproduces it.
3. Frontier, Diagnostics, Check and ToggleMouse hold no letters and are reached by name through the palette; src/keys.rs holds that list in a test.
4. The keymap is settled: action names round-trip in a test, and `l` no longer writes to the backlog. 0044 stays open because making the promise publicly is its own work, not this milestone's.

Closed on the reconciliation date.
