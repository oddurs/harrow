---
id: 117
uid: eb160d75-d6b4-4742-96ee-03978f1ef3b9
key: v0.8
title: Read an item as the prompt it is
type: milestone
status: backlog
depends_on:
- 102
created: 2026-09-27
updated: 2026-09-27
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
