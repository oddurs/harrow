---
id: 124
uid: 0fba71a1-0a86-4ea1-a151-e938f3a27aec
title: Split an item into its steps from harrow
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
area: write
---

## Problem

`cairn split <ID>` turns an item's numbered Approach into child items that
depend on each other in order (cairn 0157). A person triaging in harrow is
the one who notices an item is three items, and would have to leave harrow to
split it.

## Proposal

A palette command `split`, with no default key because it creates items in
bulk. It runs `cairn split <ID> --dry-run` through the shell and shows the
children cairn names in the confirm box. `y` runs `cairn split <ID>`, and the
children arrive with the next reading like any other new items. Cairn's
refusals ("no numbered steps", "already split") reach the footer verbatim.
Offered only where `cairn split --help` succeeds.

## Acceptance criteria

- [x] The confirm lists every child the dry run names before anything is written
- [x] `y` writes through `cairn split`, and the children appear as rows on the next reading
- [x] Cairn's refusal text reaches the footer unchanged
- [x] Against a cairn without `split`, the palette does not offer it

## 2026-09-28

Claimed past 0122 for the reason noted on 0123. Built: a palette command split with no key (it creates items in bulk; listed among the by-name-only commands) returns Action::Split(id) for the one target, the mark or the cursor, as every write acts on. main.rs runs 'cairn split <id> --dry-run' and App::show_split puts cairn's child lines, verbatim, in the confirm, now drawn with as many detail lines as it has (single-line confirms draw exactly as before) and '…and N more' past the screen. y writes 'cairn split <id>' through the ordinary write path; the children arrive with the next reading. A refusal reaches the footer in cairn's own words through ExecError::said(), which keeps the program's sentence rather than 'exited 1: …'. Split is a write, so it sits behind the readonly and filed-elsewhere guards (tests/following.rs). Withheld through App::cannot where 'cairn split --help' fails.

## 2026-09-28

Review (/code-review) found four things, all taken: split used the cursor even with one item marked, so the filed-elsewhere guard checked a different item from the one split — now targets(); the confirm's count was repeated in the toast as fact, though an agent filing an item before y moves the numbers — the toast now says '0003 split', and what was made arrives with the re-read; a child list longer than the screen was cut silently — now '…and N more'; and a comment had been displaced. Evidence: tests/splitting.rs (10: by name only, dry run first; every child in the confirm; y writes split; n writes nothing; cairn's refusal verbatim; ExecError::said; withheld from the palette; one mark wins over the cursor; a long split says how many more; one item at a time), following.rs covers Split, a glyph state for the confirm. Checked against cairn 9c29249: --dry-run prints two-space-indented child lines then 'dry run: …' on stdout; a refusal is one line on stderr.

## Result

The palette's split asks cairn for a dry run, shows every child it names in the confirm, and y writes cairn split; the children arrive with the next reading. Cairn's refusals reach the footer in its own words, and a cairn without split is not offered it.
