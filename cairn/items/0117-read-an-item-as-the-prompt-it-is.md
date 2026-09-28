---
id: 117
uid: eb160d75-d6b4-4742-96ee-03978f1ef3b9
key: v0.8
title: Read an item as the prompt it is
type: milestone
status: done
depends_on:
- 102
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
---

Cairn now treats an item as a prompt that outlives the session (cairn 0153,
decision 0154). A finished item records what it concluded in a `## Result`
section. `cairn prompt <ID>` compiles an item and everything it rests on into
what an agent should read, `cairn split <ID>` turns numbered steps into child
items, and prompt checks say when an item will be misread.

Harrow is where a person watches that work, so it has to read Results the way
cairn does, show them where they matter, record one when it closes something,
and put cairn's new commands within one key. Each command stays cairn's.
Harrow shows what it reads and hands every write and every compilation to
cairn, which is the boundary 0101 drew.

## Release gate

A finished item's Result reads the same in harrow as in `cairn show --json`,
and the agreement suite holds it. Closing an item from harrow can record a
Result. An item's prompt is one key away. Every piece degrades to today's
behaviour against a cairn that lacks the command.

## Sequence

Reading Results first, because everything else shows or writes one. Then
showing them, then the pin that proves the reading against real cairn. Closing
with a Result needs that pinned cairn. Prompt, split and the checks wait on
cairn 0156-0158 and the second pin.

## 2026-09-28

Release gate, each part established by a merged item: a finished item's Result reads the same in harrow as in cairn show --json, held by the agreement gate against Cairn 9c29249 (0120, #110; failing without the fence rule). Closing from harrow records a Result where cairn takes --result (0121, #112). An item's prompt is one key away, P (0123, #113). Every piece degrades to today's behaviour against a cairn without the command (0121, 0123-0125; COMPATIBILITY.md via 0122, #116). Also landed in the milestone: Result and Builds on in the pane (0119, #111), split (0124, #114), prompt checks (0125, #115), check warnings (0130, #117), writes waited on rather than killed (0129, #118), pane links from the keyboard (0128, #119), README/CHANGELOG/man (0126, #120). 0119 criterion 5 stays unticked: two recorded screens changed, correctly, because sample 0005 builds on finished work.

## Result

Harrow reads a Result as cairn does and the agreement suite holds it, shows it first and under Builds on, records one on close, shows an item's prompt on P, splits from the palette and shows prompt checks — each degrading to the old behaviour against a cairn without the command.
