---
id: 86
title: h and l mean what the arrows mean
type: bug
status: done
milestone: v0.6
depends_on:
- 85
created: 2026-09-18
updated: 2026-09-18
priority: p2
area: chrome
---

## What happens

`h` and `l` change an item's status. The arrow keys next to them move between
groups.

```rust
(K::Left,      n, C::PrevGroup),   // moves
(K::Right,     n, C::NextGroup),   // moves
(K::Char('h'), n, C::Retreat),     // writes
(K::Char('l'), n, C::Advance),     // writes
```

Every tool that binds `hjkl` binds it to the arrows. Here `j` and `k` are
`Down` and `Up` as expected, and then the horizontal pair is a mutation — so
the half of the reflex that is right teaches you to trust the half that is
wrong. The failure is silent and it is a write: `l` on a backlog item moves it
to the next status and tells you so in a toast.

## What should happen

`h` and `l` step between groups, the same as `←` and `→`.

Advance and retreat keep `<` and `>`, which are **already bound to them** — so
this removes two bindings and adds none, and the glyphs are the better ones
for the act anyway.

## Why it waits for the palette

This is one of three keymap corrections that should land together, once 0085
gives the demoted ones somewhere to live:

- `h` `l` — repurposed to match the arrows
- `m` — a mouse-reporting toggle should not hold a letter
- `D`, `ctrl-k` — diagnostics and check are occasional; `ctrl-k` was already a
  sign there was nothing left

Doing it before the palette exists takes capability away. Doing it after is
free, and the freed letters are what 0083 and 0087 need.

## Why it has to happen before 1.0

0044 promises that *the action names a `[keys]` table binds to* are fixed at
1.0. Fixing them is the right promise; making it while `l` writes to the
backlog is not. After 1.0 this correction costs somebody's config file.

## Acceptance criteria

- [x] `h` and `l` do what `←` and `→` do
- [x] `<` and `>` are the only bindings for advance and retreat
- [x] `m`, `D` and `ctrl-k` are reachable by name and hold no key
- [x] The help overlay and the footer hints follow, because they are generated
- [x] Nothing that had a key before is unreachable now

## 2026-09-18

Two tests had to go, and they were the same test twice: `the_defaults_cover_every_command` in keys.rs and the second half of `the_suite_presses_every_key_the_program_binds` both asserted that every command has a key. That is the zero-sum rule written down — it is *why* a mouse-reporting toggle held a letter, and it would have failed on any demotion however sensible.

Replaced rather than deleted. Reachability now means by a key *or* by name, and the palette is generated from `Command::ALL` so the second half is structural. The keys.rs one keeps a list of exactly what is demoted, so demoting anything else fails until somebody adds it to the list and says why — the same shape the GAPS table in tests/lenses.rs had.

Three tests in interaction.rs were pressing `l` and `h` to advance and retreat a status. They now press `>` and `<`, which is what they always meant.
