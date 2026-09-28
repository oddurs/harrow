---
id: 121
uid: f63f6ef9-8c8a-4254-94e1-0a3553a87711
title: Record a Result when closing
type: feature
status: backlog
milestone: v0.8
depends_on:
- 120
created: 2026-09-27
updated: 2026-09-27
priority: p0
effort: m
area: write
---

## Problem

`cairn close <ID> --result "<TEXT>"` writes an item's Result (cairn 0155).
Harrow's `x` closes with no way to say what was concluded, so every item closed
from harrow closes without one. Cairn's hint when that leaves open dependents
with nothing to quote is also lost, because harrow only reports success or
failure.

## Proposal

- Ask the cairn on PATH once whether `close --help` lists `--result`, the way
  `main.rs` already asks about `tick` (`cairn_can`). Store it as
  `App::can_record_result`.
- With support, `x` on one item opens the footer editor labelled `result`
  (a new `Editing::Result`), prefilled with the item's current Result.
  Enter closes with `--result <TEXT>`. An empty line closes without the flag.
  Esc cancels.
- Closing marked items does not ask. One conclusion for forty items is not a
  conclusion.
- Whatever cairn prints about dependents reaches the footer with the success
  toast.
- Without support, `x` is exactly what it is today.

## Acceptance criteria

- [ ] With support, closing one item asks for a Result and writes `close <ID> --result <TEXT>`; an empty answer writes plain `close <ID>`
- [ ] Without support, `x` is unchanged, held by a test with the capability off
- [ ] Closing marked items never asks
- [ ] Cairn's hint about open dependents with nothing to quote is shown in the footer
- [ ] The help overlay says what `x` asks, generated from the keymap as today
