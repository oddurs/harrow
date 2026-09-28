---
id: 121
uid: f63f6ef9-8c8a-4254-94e1-0a3553a87711
title: Record a Result when closing
type: feature
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 120
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
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

- [x] With support, closing one item asks for a Result and writes `close <ID> --result <TEXT>`; an empty answer writes plain `close <ID>`
- [x] Without support, `x` is unchanged, held by a test with the capability off
- [x] Closing marked items never asks
- [x] Cairn's hint about open dependents with nothing to quote is shown in the footer
- [x] The help overlay says what `x` asks, generated from the keymap as today

## 2026-09-28

Built. main.rs asks once whether the cairn on PATH lists --result under close --help (cairn_takes; a word match, so '-r, --result <TEXT>' reads true), stored as App::can_record_result. With it, x on one item opens the footer editor as Editing::Result(id), carrying the id so nothing else can drift from it, prefilled with the item's current Result; Enter writes close <id> --result <text>, an empty line writes close <id>, Esc keeps it open. That box replaces the yes/no for one item: answering it is the confirmation, and asking twice on a one-keystroke gesture gets muscle-memoried past. Marked items keep the confirm and never ask. Without the capability x is unchanged. exec::run_advised returns stderr too; App::done puts cairn's first 'hint:' line in the footer with the news, ahead of the undo line (the undo is in the history; the hint is said once). Help: close is described as 'close, with what it concluded', which fits the overlay's column where the longer wording was clipped.

## 2026-09-28

Evidence: tests/interaction.rs closing_asks_what_it_concluded_where_cairn_records_it, an_empty_answer_closes_without_a_result, esc_keeps_it_open, the_answer_starts_from_the_result_it_has, closing_marked_items_never_asks_for_a_result, without_a_result_to_record_closing_is_the_confirm_it_was, cairns_advice_about_a_close_reaches_the_footer. Against cairn 9c29249: close --help lists '-r, --result <TEXT>'; closing an item with an open dependent and no result prints '  hint: 0006 depend(s) on 0005, and it has no result for them to quote — ...' on stderr with no colour codes when piped, the shape App::done reads. The help screen was re-recorded for the new description; palette and keymap tests now find close by its describe() rather than by old wording.

## 2026-09-28

Review (/code-review) found three things, all fixed. (1) A Result beginning with a dash was passed as its own argument and cairn read it as a flag: 'cairn close 5 --result "-5% latency"' exits 2 with 'unexpected argument'. Now one argument, --result=<text>; checked against cairn 9c29249 (exit 0, result '-5% latency') and held by a_result_that_begins_with_a_dash_is_still_a_result. (2) The footer repeated cairn's advice to type 'cairn close N --result', which harrow cannot do on a closed item, and it displaced the undo. Now the hint's first clause, then the undo: '0003 closed · 0005 depend(s) on 0003, and it has no result for them to quote · undo: cairn reopen 3'; the test uses cairn's exact wording. (3) The Result box replaced the confirm that said how many criteria were left; the box now says 'N of M criteria ticked' in the warning colour (the_result_box_says_what_is_left_unticked).

## Result

Closing one item from harrow asks what it concluded, prefilled with any Result it has, and writes close <id> --result=<text>; an empty answer closes without one, Esc keeps it open, and the box warns of unticked criteria. Marked items and a cairn without --result keep the confirm. Cairn's hint about open work left with nothing to quote reaches the footer, beside the undo.
