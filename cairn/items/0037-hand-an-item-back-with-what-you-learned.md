---
id: 37
title: Hand an item back with what you learned
type: feature
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: write
---

## Problem

cairn 0.2.0: "`cairn release --reason` passes on what the last person learned,
and `cairn claim` shows it to whoever takes the item next."

harrow sends `cairn release <id>` with no reason, and shows nothing when
claiming. So the one moment where the backlog could carry a handover — the
person giving something up knows precisely why, and the next person to pick it
up is about to need that — passes silently.

## Proposal

`C` asks for a line before releasing, and skipping it releases with no reason
the way it does now. `c` on an item that carries one shows it before the claim
lands rather than after.

## Cost

A prompt on a gesture that is currently one keystroke. It has to be skippable
with a single key or it will be resented, and resented prompts get muscle-
memoried past, which is worse than not asking.

## Acceptance criteria

- [x] Releasing offers a reason and takes no reason for an answer
- [x] Claiming an item that carries one shows it — cairn does this itself, since `--reason` is recorded as a note on the item and `cairn claim` prints it
- [x] The reason reaches cairn as `--reason`, not as a note
