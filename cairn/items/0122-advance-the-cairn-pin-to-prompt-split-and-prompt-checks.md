---
id: 122
uid: c658bc66-fdb3-4741-b6c0-02e77edcc27b
title: Advance the Cairn pin to prompt, split and prompt checks
type: chore
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 120
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
effort: s
area: read
---

## Why now

Blocked until cairn 0156 (`cairn prompt`), 0157 (`cairn split`) and 0158
(prompt checks) merge. The three harrow items that call them are tested
against this pin. May land in steps if cairn merges them apart; say which
revision covers which command in COMPATIBILITY.md.

## What changes

- CI, COMPATIBILITY.md and the corpus provenance pin the revision that has all
  three, by the procedure in COMPATIBILITY.md.
- COMPATIBILITY.md lists, for each new command, the first Cairn revision
  harrow uses it with, and what harrow does without it.

## Done when

- [x] CI builds a Cairn with `prompt`, `split` and `check --prompts`
- [x] COMPATIBILITY.md says what each needs and how harrow degrades without it

## 2026-09-28

The pin that covers prompt, split and check --prompts landed with 0120: Cairn 9c29249, built by CI's agreement job. This adds what COMPATIBILITY.md was missing: for each command harrow now uses, the Cairn merge that first had it and what harrow does without it — and the rule behind all of them, that harrow asks the installed cairn once, from --help, and offers nothing it lacks. Checked each row against the code that implements it (0121 can_record_result, 0123/0124 App::cannot and Keymap::withhold, 0125 can_check_prompts) and each revision against cairn's history (cd547d3 #112 0155, 5569798 #113 0156, f91642a #115 0157, 2fb5f31 #116 0158).

## Result

CI builds Cairn 9c29249, which has prompt, split and check --prompts, and COMPATIBILITY.md says for each command harrow uses the first Cairn that has it and what harrow does without it.
