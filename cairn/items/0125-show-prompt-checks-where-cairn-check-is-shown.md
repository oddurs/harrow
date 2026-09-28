---
id: 125
uid: 586ebbdf-b301-4924-914e-fa9c9fcc6ec1
title: Show prompt checks where cairn check is shown
type: feature
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 122
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
priority: p2
effort: s
area: chrome
---

## Problem

`cairn check --prompts` reports open items that an agent will misread (cairn
0158): no criteria, no context, a finished dependency with no Result, and
ticked criteria with no note. Harrow's diagnostics overlay runs `cairn check`
on `C`, and it would not show any of these.

## Proposal

Where the cairn on PATH supports it, `C` runs `cairn check --prompts`, and
the overlay shows the prompt findings under their own heading, **Prompts**, as
warnings. Advice must not read as a broken backlog. Harrow does not evaluate
the checks itself: they are cairn's rules, and a second implementation drifts
from the first. That is the lesson of the filter grammar.

Deferred, with the reason: putting prompt findings in the needs queue. On an
item you own, "this will be misread" is arguably a question for you. But
nobody has lived with the checks yet. Revisit once they have run on this
backlog for a milestone.

## Acceptance criteria

- [x] With support, the diagnostics overlay shows prompt findings under Prompts, as warnings
- [x] Without support, the overlay is unchanged cell for cell
- [x] No harrow code evaluates a prompt check; every finding shown is cairn's output

## 2026-09-28

Claimed past 0122 for the reason on 0123. Built: main.rs asks once whether 'cairn check --help' lists --prompts (App::can_check_prompts). With it, C runs 'cairn check --color never --prompts' through run_advised, and App::show_check picks the lines cairn marks ': prompt: ' out of its stderr — cairn 9c29249 prints 'cairn: <file>: prompt: <finding>' there and the summary on stdout — keeping each finding as cairn wrote it and reading the item off the file name. The diagnostics overlay shows them under their own heading, Prompts, in the warning colour, after cairn check's section, capped at eight with a count of the rest. Harrow evaluates no check: the only thing it does with a finding is find the line. Without --prompts cairn prints no such line, so the overlay is what it was; a test renders it with and without other stderr and compares the screens.

## 2026-09-28

Review (/code-review) found one bug, fixed: when a check fails, cairn prints its warnings, prompt findings among them, before its errors, and the failure was reported as the first stderr line — a prompt finding, in the error colour, as the reason the check failed. show_check now takes the ExecError, keeps the findings as advice, and names the failure by the first line that is not one (a_failed_check_names_its_error_and_keeps_its_advice). Found along the way and filed as 0130: harrow drops every other line cairn check prints on stderr, so the overlay says '6 warning(s)' and shows none; left alone here to keep the overlay unchanged without --prompts.

## Result

Where cairn's check takes --prompts, C runs it and the diagnostics overlay shows cairn's prompt findings under Prompts, as warnings, each as cairn wrote it; a failed check names its real error and keeps the findings. Without --prompts the overlay is unchanged. Harrow evaluates none of the checks.
