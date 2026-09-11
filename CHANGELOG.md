# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Day-to-day work is tracked as [cairn items](cairn/items) and rendered into
[ROADMAP.md](ROADMAP.md); this file records what landed in a release.

## [Unreleased]

### Changed

- Containers — any type a reference field names, which for most projects means
  `milestone` — are no longer rows in the list or cards on the board. They are
  the headings work belongs to. `a` brings them back, as does asking for the
  type by name in the filter. This follows cairn, which removed them from
  `next`, the board and an ordinary `list` for the same reason.
- `a` and `--all` mean *all*: finished, dropped, and containers. The config key
  is `show_all`, replacing `show_closed`.

### Added

- `contains`, `descendants`, `depth`, `leaf`, `container`, `owner` and
  `created_by` resolve in the filter box, matching cairn's derived keys.
- The schema reads `agent` on a field or a status, so a project that restricts
  what tooling may change can say so. harrow shows the restriction where the
  choice is made and lets cairn enforce it.
- `.githooks/post-merge` settles ids and re-derives the roadmap after a merge,
  and `scripts/setup` registers cairn's merge driver.

[unreleased]: https://github.com/oddurs/harrow/commits/main
