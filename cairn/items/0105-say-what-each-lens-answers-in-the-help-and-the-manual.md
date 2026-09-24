---
id: 35cb07c4-cc07-45e1-ab2b-17ccb6302faa
title: Say what each lens answers, in the help and the manual
type: docs
status: done
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
area: docs
effort: s
part_of:
- 39ac484e-b657-4b46-92ce-89118076115a
---

## What a reader needs

Harrow's five lenses are named in the tab strip and nowhere else does a reader
learn what each one answers. `--help` lists flags; the manual describes the
interface. With 0103 and 0104 adding doors, the names become part of the
command-line contract and have to be documented as such.

## Where it goes

- `--help`: the new flag, with the five names and the two aliases.
- The man page and the shell completions, which are generated from the same
  command table.
- `README.md`: a short line per lens saying the question it answers.
- `COMPATIBILITY.md` only if the plain formats become part of the cross-tool
  contract — they are harrow's own output, not Cairn's, so probably not.

## Done when

- [x] `--help` names every lens and both aliases
- [x] The man page and completions include the flag
- [x] The README says what each lens answers, in one line each
- [x] `harrow --doctor` still passes and the snapshots are re-recorded

## 2026-09-23

Shipped. The help, the man page and the shell completions all generate from the one flag table in src/cli.rs, so naming the flag once was enough for three. The README gains a table of the five lenses and the question each answers.
