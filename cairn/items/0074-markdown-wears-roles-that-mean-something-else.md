---
id: e1e23d89-b963-4b68-8eec-6bf413eeb91b
title: Markdown wears roles that mean something else
type: bug
status: done
milestone: v0.5
created: 2026-09-13
updated: 2026-09-13
priority: p1
area: chrome
---

## What happens

A theme is a set of *roles* — what a colour is for, not what colour it is.
The body renderer added three concepts and gave none of them a role of its
own, so each borrowed one that already meant something else:

- **Inline code and fenced code are drawn in `label`.** `label` is the role
  for an item's labels. It is now the *only* thing `label` colours: the
  labels themselves go out in `muted`, in the fields grid. So a theme that
  sets `label` changes how code looks and leaves labels alone, which is the
  opposite of what it says.
- **A link is drawn in `accent`.** Accent is chrome — the selected tab, the
  cursor, every footer hint, the mark on something that just moved. A link
  inside prose is not chrome, and because accent is already everywhere, a
  link drawn in it does not stand out from the interface around it.
- **Bold text is drawn in `heading`.** Strong prose is not a heading. It
  collides with a real body heading drawn one line above it.

There is nothing a theme author can do about any of this, because the
concepts do not appear in the file format at all. `THEMES.md` lists every
role a file may name, and code and links are not among them.

## What should happen

Two new roles, `code` and `link`, defined everywhere a role is defined:
`auto`, `from_palette`, `mono`, the `apply` list, `ThemeFile`, the three
built-in theme files, and the table in `THEMES.md`. They are additive —
`deny_unknown_fields` rejects a key harrow does not know, but a file that
omits one inherits it, so every theme already written keeps working.

`label` goes back to labelling. Strong prose takes `text`, which is the
body lifted out of `muted` rather than a heading borrowed.

## Cost

Two roles is two more things a theme file may say and a reader of
`THEMES.md` has to skim. Worth it: the alternative is a renderer whose
colours cannot be changed, in a program whose whole theming argument is
that colours are named by purpose.

## Acceptance criteria

- [x] `code` and `link` are roles, settable from a theme file
- [x] Every way a theme is built defines them: `auto`, a measured palette, `mono`, and the built-in files
- [x] A theme file written before this still parses and still renders
- [x] `label` colours labels again
- [x] Bold prose does not wear the heading colour
- [x] `THEMES.md` lists every role the format accepts, and nothing it does not

## 2026-09-13

The roles, the on-disk form and the code that reads a file are now generated from one `roles!` list, because the drift this item is about was exactly a role added in one place and forgotten in the others. `Theme::roles()` comes from the same list, so the checks that must cover every role iterate rather than enumerate — two of them were already a hand-copied list missing `code` and `link` the moment those existed.

Two tests hold the format to its documentation by parsing both: the example under `## The roles` in THEMES.md must name exactly the roles harrow accepts, and so must each built-in. A theme somebody else wrote may say as little as it likes; one shipped here may not, because a built-in that omits a role silently inherits it from `auto` and puts two palettes on one screen.

Where a narrow palette spends one hex on two roles — gotham draws `link` and `label` in the same teal, as it already does for `open` and `muted` — that is left alone. The fix is that they are separate keys, not that they must differ. What must differ is `code` and `link` against `muted`: that is the colour the prose around them is drawn in, so a derived theme now falls back to something near the foreground rather than onto `muted` itself.
