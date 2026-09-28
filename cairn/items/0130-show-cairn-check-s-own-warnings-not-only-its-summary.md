---
id: 130
uid: 7352b7b9-83f3-448b-a3b6-b743f76204a6
title: Show cairn check's own warnings, not only its summary
type: bug
status: done
milestone: v0.8
assignee: oddurs
created_by: cairn-26
depends_on:
- 125
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p2
area: chrome
---

## Problem

`C` runs `cairn check` and shows what it printed on stdout, which is the one
summary line (`ok: 9 item(s), 6 warning(s)`). The warnings themselves go to
stderr, and harrow drops them, so the overlay says there are six warnings and
shows none. When the check fails outright, only the first stderr line is
shown. Found while building 0125, which picks the `: prompt: ` lines out of
that stderr and had to leave the rest alone to keep the overlay unchanged
without `--prompts`.

## Proposal

Show cairn's other stderr findings under the cairn check heading, as cairn
wrote them, capped with a count of the rest, as the Prompts section is.

## Acceptance criteria

- [x] A warning cairn check prints is shown in the overlay, word for word
- [x] A failed check shows cairn's findings, not only the first line

## 2026-09-28

Built: App::checked now holds every line cairn check said, as Result<Vec, Vec> — its summary first, so the count survives the list being cut, then each finding word for word: for a check that passed, stdout's summary and stderr's warnings; for one that failed, stderr's 'failed: …' summary and every finding before it. Prompt findings stay out of it (they have their own section, 0125). The overlay draws the summary in the colour of the outcome, the findings as warnings (cairn does not mark which of its lines is which), eight at most, with '…and N more — cairn check has them all'. Checked against cairn 9c29249: warnings are 'cairn: <file>[:line]: …' on stderr with 'ok: …' on stdout; a failed check prints nothing on stdout and ends stderr with 'failed: N error(s), M warning(s) across K item(s)'. 0125's test that other stderr leaves the overlay unchanged is replaced by one that holds what still matters: no Prompts section without prompt findings.

## 2026-09-28

Evidence: tests/prompt_checks.rs a_checks_warnings_are_shown_word_for_word, a_failed_check_shows_every_finding_and_its_summary_first, a_long_check_says_how_many_more_there_are, without_prompt_findings_there_is_no_prompts_section; interaction's check test unchanged in meaning.

## Result

The diagnostics overlay shows what cairn check found, not only its summary: warnings word for word under the summary, and for a failed check every finding with the failure count first, capped with a count of the rest.
