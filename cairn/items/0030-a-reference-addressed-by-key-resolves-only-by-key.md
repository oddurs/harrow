---
id: 30
title: A reference addressed by key resolves only by key
type: bug
status: backlog
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: read
---

## What happens

`App::item_named` resolves a key by comparing against `key`, and then falls
back to parsing the value as an id:

```rust
i.key.as_deref().is_some_and(|k| k.eq_ignore_ascii_case(key))
    || key.parse::<u32>().is_ok_and(|id| id == i.id)
```

Specification §4.3 forbids the fallback in as many words:

> A key-addressed reference resolves **only** by key. A reader **must not**
> fall back to matching an identifier or a title.

The reason is in the same section: a project may render its identifiers as
`0042`, so `milestone: 0042` must not mean either a key or a number depending
on what happens to exist. The rule that keeps it unambiguous is that a key may
not look like a rendered identifier, and it is only worth anything if readers
do not resolve keys as identifiers anyway.

The case-insensitive comparison is harrow's invention too. Nothing in the
specification says keys are case-insensitive, and `v0.1` and `V0.1` are
allowed to be two keys.

## Proposal

Resolve a key-addressed reference by key, exactly, and nothing else. An item
whose key names nothing is a reference that names nothing, which §4.3 already
says is fine.

Pairs with 0031, on rendered identifiers: both are about the boundary between what an identifier is and how it is written.

## Cost

A backlog somewhere has a `milestone: 42` meaning item 42, which will stop
resolving. That is a project cairn itself would refuse, and the failure is
visible — the item shows no milestone — rather than silent.

## Acceptance criteria

- [ ] A key-addressed reference matching only an id resolves to nothing
- [ ] Key comparison is exact, including case
- [ ] An unresolved reference is a reference that names nothing, not a warning storm
