---
id: 49
title: File an item the way the project files them
type: feature
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: write
---

## Problem

`n` sends `cairn new "<title>"` and nothing else. Every other field takes the
default: a bug filed from harrow is a `feature`, and nothing filed from harrow
is ever scheduled against a milestone.

The project has said what an item can be. `cairn.toml` declares the types,
with an icon and a template each, and `cairn new` takes `--type`,
`--milestone`, `--label`, `--assignee`, `--depends-on` and `--set` for
anything else. harrow uses one of them.

So filing from harrow produces an item that then has to be corrected — `s` for
the status it should have had, `M` for the milestone, and the type cannot be
changed from harrow at all. The gesture that was meant to save the round trip
to the shell ends in a round trip to the shell.

It also loses the template. A type's `template` seeds the body with the
project's own headings, and an item created as the wrong type gets the wrong
ones — which is the difference between an item somebody will read later and a
title with nothing under it.

## Proposal

`n` asks for the title, then offers the type, from the pickers harrow already
has for status, priority and milestone. Where the cursor sits when you press
it supplies the rest: filing while a milestone group is selected files under
that milestone, and filing on the board files into the column you are in.

Nothing becomes required. The fast path stays title-then-enter, with the
defaults the project declared — that is what `default_type` and
`default_status` are for.

## Acceptance criteria

- [ ] Filing offers the project's types, with their icons
- [ ] The milestone or column under the cursor is the default, and is visible as such
- [ ] Title-then-enter still files with the project's own defaults
- [ ] The type's template is what seeds the body, because cairn is what creates it
