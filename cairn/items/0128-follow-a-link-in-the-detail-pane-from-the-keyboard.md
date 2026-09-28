---
id: 128
uid: f73e2de8-4d63-431b-90ba-b8537ca9ee78
title: Follow a link in the detail pane from the keyboard
type: feature
status: done
milestone: v0.8
assignee: oddurs
created_by: cairn-26
depends_on:
- 119
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p2
area: chrome
---

## Problem

Links in the detail pane (Waiting on, Builds on, and URLs in the body) are
reached only by clicking. App::follow runs only from Hit::Link, and ↵ opens
the reader. 0119 assumed Waiting on could be followed by ↵; it cannot.
Someone driving harrow from the keyboard cannot move to the item a
dependency names.

## Proposal

A link cursor in the detail pane: a key that moves through `app.links` in
the order they are drawn, with the current one highlighted, and a key that
follows it through `App::follow`, the same path a click takes. That path
already unfolds finished work (0119). Whether ↵ does this when the pane has
focus is the decision to make with the screen in front of you.

## Acceptance criteria

- [x] Every link in the pane can be reached and followed without the mouse
- [x] The key is listed in the help overlay, generated from the keymap
- [x] Following by key and by click select the same item

## 2026-09-28

Decided: ] and [ pick the next and previous link in the detail pane (both were free), drawn reversed and scrolled into sight, with a toast naming where it goes and '↵ follows, esc lets go'. While a link is picked, ↵ follows it through App::follow — the path a click takes, so finished work unfolds the same way — and otherwise ↵ reads as before; esc lets it go before it does anything else. The pick is keyed by the item, so moving on lets it go without bookkeeping. One help row, '[, ]', paired as the other two-way moves are. Evidence: tests/links.rs (pick, name and follow; key and click reach the same item; [ from nothing is the last and it wraps; esc lets go and ↵ reads again; moving on lets go; the picked link is drawn reversed in the buffer; a pane with no links says so; one help row). The help screen was re-recorded.

## Result

Every link in the detail pane — what an item waits on, what it builds on, a URL in its body — can be picked with ] and [ and followed with ↵, by the same path a click takes; esc lets it go.
