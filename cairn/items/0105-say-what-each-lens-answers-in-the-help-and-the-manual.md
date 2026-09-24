---
id: 105
title: Say what each lens answers, in the help and the manual
type: docs
status: planned
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
priority: p2
area: docs
effort: s
part_of:
- 102
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

- [ ] `--help` names every lens and both aliases
- [ ] The man page and completions include the flag
- [ ] The README says what each lens answers, in one line each
- [ ] `harrow --doctor` still passes and the snapshots are re-recorded
