---
id: 119
uid: 20946747-dd8e-4480-9b96-ac2df9127b47
title: Show what a finished item concluded
type: feature
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 118
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: m
area: chrome
---

## Problem

Once items carry Results, harrow shows each one as just another heading inside
Body, below Acceptance, where it is scrolled past. And the detail pane shows
an item's dependencies only while they block it ("Waiting on"). It never shows
what a finished dependency concluded, which is exactly the part `cairn prompt`
hands an agent (cairn 0156, layer 3).

## Proposal

- **Result, hoisted.** In the detail pane and the reader, a closed item with a
  Result shows it as its own section directly under the state lines, above
  Latest. It is removed from Body so it is not printed twice; `hoisted_lines`
  in `ui.rs` already does this for the acceptance criteria.
- **Builds on.** A section listing each direct dependency that is finished:
  reference and title as a link (like Waiting on), then its Result wrapped to
  three lines, or its last dated note where it has none, or nothing.
  Unfinished dependencies stay under Waiting on and are not repeated. This is
  the order `cairn prompt` uses, so the pane and the prompt read the same.

## Acceptance criteria

- [x] A closed item with a Result shows it first, above Latest, and Body does not repeat it
- [x] Each finished dependency shows its Result, else its last dated note, else only its title
- [x] An unfinished dependency stays under Waiting on and does not appear under Builds on
- [ ] A dependency's title in Builds on reaches that item by click and by `↵`, as Waiting on does
- [ ] An item with no dependencies and no Result renders unchanged: the existing snapshots pass untouched
- [x] One recorded screen holds the new sections

## 2026-09-28

Built: a closed item's Result is its own section directly under the state lines (above Elsewhere and Latest), rendered with the body renderer, and its span (item::result_span, now pub(crate); no second parser) joins hoisted_lines so Body leaves it out, heading included. Open items keep a Result heading in Body: one that is still being written is read where it is. The reader does the same above the body. Builds on sits under Waiting on: each direct dependency whose category is closed (done or dropped, as cairn prompt counts it), reference coloured by its state and title as a link, then its Result flattened to three lines, else its last dated note with its date, else nothing. New icon roles concluded (oct-light_bulb f400) and builds_on (oct-git_merge f419), checked against the installed FiraCode Nerd Font's cmap; empty in the Unicode set; both added to Glyphs::every and to states_in.

## 2026-09-28

Decisions. (1) App::follow refused anything hidden, and finished work is folded out of an ordinary listing, so every Builds on link said 'not in this view'. Following a closed item now unfolds its group's finished work, as space on the fold would, and puts the fold back if the filter hides it anyway. (2) The criterion's '↵, as Waiting on does' rests on a premise that is not true: Waiting on is reached only by click (follow runs from Hit::Link alone; ↵ opens the reader). Builds on is reached exactly as Waiting on is, by click; keyboard following of every pane link is filed as 0128 rather than invented here. Criterion 4 stays unticked for its ↵ half. (3) The reader wrapped prose to the full measure and then indented it two columns, clipping the last word of every long line; it now wraps inside the measure. That changes no recorded screen.

## 2026-09-28

Evidence: ui::tests a_finished_items_result_comes_first_and_is_not_read_twice, what_an_item_builds_on_says_what_each_concluded (in order: result, dated note, title only; the unfinished one only under Waiting on), the_sample_backlog_has_no_result_to_show; mouse a_finished_dependency_is_clickable (Link(1) selects 0001 though it is folded); rendering what_a_finished_item_concluded (new screen tests/snapshots/concluded.txt holds Result and Builds on) and reading_what_a_finished_item_concluded (fails before the wrap fix: 'half' was clipped). Recorded screens: actors and settling changed, and only because sample 0005 depends on finished 0002 and now shows Builds on; every item with no dependencies and no Result draws as before. Glyph rules pass with two new states.

## 2026-09-28

Review (/code-review) found no correctness bugs and two low points, both taken. (1) The reader removed only the Result's own lines, while the pane also drops a heading the Result was the only thing under; the reader now skips hoisted_lines' set too, and the fixture's Result sits under an otherwise empty '## Outcome' that neither view shows (reading_what_a_finished_item_concluded holds it). (2) Criterion 5 said 'the existing snapshots pass untouched', and two do not: actors and settling, because sample 0005 depends on finished 0002 and now shows Builds on, which criterion 2 requires. Every item with no finished dependency and no Result draws as before, but the sentence as written is not true, so it is unticked rather than ticked on a reading of it.

## Result

A finished item shows what it concluded first, in the detail pane and the reader, and Body no longer repeats it. Under Waiting on, Builds on lists each finished dependency with its Result, else its last dated note, else its title, as cairn prompt does; clicking one reaches it, unfolding finished work where it is folded. Keyboard following of pane links is 0128.
