# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Day-to-day work is tracked as [cairn items](cairn/items) and rendered into
[ROADMAP.md](ROADMAP.md); this file records what landed in a release.

## [Unreleased]

### Changed

- The layout drops the detail pane below ninety-six columns rather than halving
  the list, and the rows keep fixed columns that degrade in a defined order.
- `tab` cycles three panes rather than toggling two. The config key is `pane`,
  replacing `board`.
- The detail pane wraps its own text, so a wrapped paragraph keeps its left
  edge, and reflows paragraphs rather than re-wrapping somebody else's line
  breaks.

- Containers — any type a reference field names, which for most projects means
  `milestone` — are no longer rows in the list or cards on the board. They are
  the headings work belongs to. `a` brings them back, as does asking for the
  type by name in the filter. This follows cairn, which removed them from
  `next`, the board and an ordinary `list` for the same reason.
- `a` and `--all` mean *all*: finished, dropped, and containers. The config key
  is `show_all`, replacing `show_closed`.

### Added

- A filesystem watcher, so a change made in another window shows up at once
  instead of on the next poll. The poll stays as the backstop for the places a
  watcher does not work, bursts are settled before reading, and there is a floor
  between reads so a directory being rewritten continuously cannot become a busy
  loop. `watch = false` turns it off; `harrow --doctor` says which is in use.

- Marking. `space` marks the item under the cursor, ctrl-click marks one and
  shift-click a range, and the next change applies to all of them — in a single
  `cairn` invocation, after a confirmation that says how many. Where the marked
  set is exactly what the filter is showing, it becomes one `cairn set --filter`
  rather than a list of ids.

- A mouse interface. Tabs, statuses, rows, group headings, board columns,
  picker options, the confirmation and the footer hints are all clickable;
  double-click reads an item; a card dragged between columns sets its status.
  The map of what is clickable is built as the screen is drawn, so what you can
  see is what you can hit.

- A statistics pane, on `tab` or `--stats`: how much is closed, what is ready,
  blocked or claimed, what closed in the last week, what has waited longest,
  what is in the way of the most other things, how each milestone stands, and
  where the work sits by type and by priority.
- A status strip under the header, so what is happening is legible without
  reading a row, and a mark on anything that moved in the last forty-five
  seconds.
- The terminal's own palette, read with OSC queries at startup. Surfaces,
  borders and the selection are derived from it, and every hue is checked for
  contrast against the real background before it is used.

- `contains`, `descendants`, `depth`, `leaf`, `container`, `owner` and
  `created_by` resolve in the filter box, matching cairn's derived keys.
- The schema reads `agent` on a field or a status, so a project that restricts
  what tooling may change can say so. harrow shows the restriction where the
  choice is made and lets cairn enforce it.
- `.githooks/post-merge` settles ids and re-derives the roadmap after a merge,
  and `scripts/setup` registers cairn's merge driver.

[unreleased]: https://github.com/oddurs/harrow/commits/main
