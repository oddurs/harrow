---
id: 123
uid: 2600c6d4-15a0-47a7-aac6-c6511b7d168d
title: Hand an item's prompt to an agent
type: feature
status: backlog
milestone: v0.8
depends_on:
- 122
created: 2026-09-27
updated: 2026-09-27
priority: p1
effort: m
area: chrome
---

## Problem

`cairn prompt <ID>` compiles an item and what it rests on into what an agent
should read (cairn 0156). From harrow, getting that into an agent's session
means leaving the program and typing the command with the right id. Harrow is
where somebody is looking at the item when they decide to hand it off.

## Proposal

- A command `prompt` (palette name `prompt`, default key `P`) returns a new
  `Action::Prompt(id)`. The shell in `main.rs` runs `cairn prompt <ID>`, as it
  runs `cairn history` for `Action::History`. Nothing in `app.rs` runs a
  process.
- The output opens in an overlay like the history, scrollable, with one key
  (`y`) that copies it whole to the clipboard through the existing
  `Action::Copy` path.
- Offered only where `cairn prompt --help` succeeds, asked once at startup.

Open question for the PR, with the screen in front of the reviewer: copying is
probably the common case (pasting into an agent). If so, `P` should copy
directly and say "prompt for 0042 copied, 1.8k characters", and reading it
becomes the second key. Decide from use, and record the choice as a note here.

## Acceptance criteria

- [ ] The command runs `cairn prompt <ID>` through the shell, and `app.rs` only returns the action
- [ ] The prompt can be read in harrow and copied whole with one key
- [ ] Against a cairn without `prompt`, the key is not bound and the help overlay does not list it
- [ ] A failing `cairn prompt` shows its stderr in the footer, never an empty overlay
- [ ] The choice between copy-first and read-first is recorded on this item
