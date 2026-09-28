---
id: 123
uid: 2600c6d4-15a0-47a7-aac6-c6511b7d168d
title: Hand an item's prompt to an agent
type: feature
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 122
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
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

- [x] The command runs `cairn prompt <ID>` through the shell, and `app.rs` only returns the action
- [x] The prompt can be read in harrow and copied whole with one key
- [x] Against a cairn without `prompt`, the key is not bound and the help overlay does not list it
- [x] A failing `cairn prompt` shows its stderr in the footer, never an empty overlay
- [x] The choice between copy-first and read-first is recorded on this item

## 2026-09-28

Claimed past its dependency on 0122 on purpose: 0122's CI half, a pin with prompt, split and check --prompts, landed with 0120 (Cairn 9c29249), and its other half, COMPATIBILITY.md saying how each command degrades, is better written once 0123-0125 exist to describe. So 0122 lands after them.

## 2026-09-28

Decision, copy-first or read-first: read-first. P opens the prompt in an overlay and y copies it whole; the toast says 'copied # 0006 Report parse time… — 1834 characters'. A prompt is seven layers cairn assembled from other items, and 0158's checks exist because prompts get misread: one look before an agent acts on all of it is worth the second key, and it never overwrites the clipboard unasked. Revisit if use says the look is always skipped.

## 2026-09-28

Built: Command::Prompt (palette 'prompt', key P) returns Action::Prompt(id); main.rs runs 'cairn prompt <id>' as it runs 'cairn log' for history, and App::show_prompt opens the overlay or, for a failure or empty output, puts cairn's error in the footer. The overlay renders the Markdown through the body renderer; j/k/space/pgup scroll; y returns Action::Copy with the text untouched; any other key closes. Capability: Keymap::withhold(command) unbinds a command and drops it from help and from the palette; App::cannot lists what the installed cairn lacks (asked once: cairn prompt --help) and App::set_keymap applies it to every keymap, including one a config reload hands over. 0124 and 0125 can use the same. Evidence: tests/prompting.rs, 8 tests (the key returns the action; read and copy whole; other keys close; a failure and empty output reach the footer and open no overlay; withheld: key unbound, help and palette silent; reload keeps it withheld; the copy message); a glyph state for the overlay; the help screen re-recorded with the P row. A real 'cairn prompt' from cairn 9c29249 renders legibly; harrow's renderer does not do _underscore emphasis_, so cairn's provenance lines keep their underscores on screen (the copy is exact).

## 2026-09-28

Review (/code-review) found two things, both fixed. The mouse wheel fell through the prompt overlay to the pane behind it, so on Needs or Log it moved the selection under the prompt being read; the overlay now takes the wheel as the history does (the_wheel_scrolls_the_prompt_and_not_the_rows_behind_it). And draw_prompt had been put between draw_history and its doc comment.

## Result

P shows an item as cairn prompt compiles it, in an overlay that scrolls, and y copies it whole; a failing or empty prompt reports cairn's words in the footer instead. Where the installed cairn has no prompt, the key is unbound and neither the help nor the palette offers it, through Keymap::withhold, which survives a config reload.
