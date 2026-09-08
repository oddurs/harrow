# Themes

A theme is a set of **roles** — what a colour is for, not what colour it is. The
drawing code names roles; a theme file says what they resolve to. That is what
lets the palette change without a line of layout changing with it.

There are two other sources of colour, and they win in this order:

1. **A theme file's `[types]` / `[statuses]` table** — a per-name override, for
   when a project's own choices are not to your taste.
2. **`cairn.toml`** — if the project says a bug is red and `doing` is yellow,
   that is what they are. It is the project's schema, and it came with an
   opinion.
3. **The roles below** — everything the first two did not say.

## Where themes come from

```sh
harrow themes             # everything harrow can find, and where from
harrow themes gruv        # filtered — Ghostty ships hundreds
harrow --theme gotham     # for one run
```

Or permanently, in `~/.config/harrow/config.toml`:

```toml
theme = "ghostty:gotham"     # your terminal's own theme file
```

| spec | resolves to |
|---|---|
| `auto` | the terminal's own ANSI palette (the default) |
| `mono` | no colour at all |
| `gotham` `night` `paper` | built in, compiled from the files in `themes/` |
| `<name>` | `~/.config/harrow/themes/<name>.toml` |
| `ghostty:<name>` | a Ghostty theme file, read directly |
| `./path/to/file` | either format; harrow works out which |

`auto` maps every role onto an ANSI slot, never an RGB value. The terminal
substitutes its own colours, so harrow matches whatever is already on screen and
follows it when it changes. The cost is that roles wanting a shade *between* two
slots do not get one — `surface` and `background` are the same, and the
selection is reverse video. Correct in every terminal beats ideal in one.

`NO_COLOR`, `--no-color` and `TERM=dumb` all select `mono`, where every state is
carried by a glyph instead: `○` open, `◐` active, `✓` done, `×` dropped, `⊘`
blocked. That is not a fallback for colour blindness bolted on afterwards — it
is the same glyph the coloured screen draws.

## The roles

```toml
name = "Gotham"
dark = true

background   = "#0a0f14"   # the page
surface      = "#0c1418"   # panes, a shade above it
overlay      = "#11202a"   # popups, which must read as floating
border       = "#0e2129"
border_focus = "#33859d"   # the focused board column
selection    = "#245361"

text         = "#98d1ce"   # item titles
muted        = "#599caa"   # secondary text that still has to be read
faint        = "#245361"   # borders, references, things you look past
heading      = "#d3ebe9"   # headings inside an item's body

accent       = "#edb54b"   # keys in the footer, the overlay borders
secondary    = "#33859d"

# The four categories. A status resolves through the table in cairn.toml to one
# of these, which is the only part of a status anything reasons about.
open         = "#599caa"
active       = "#edb54b"
done         = "#26a98b"
dropped      = "#245361"

# What the dependency graph says.
blocked      = "#c23127"
ready        = "#2aa889"

# Messages: toasts, the diagnostics overlay, a failed read.
ok           = "#26a98b"
warn         = "#d26939"
error        = "#c23127"

milestone    = "#888ca6"   # group headings, and the milestone in a detail pane
label        = "#33859d"
person       = "#195466"   # an assignee

# Emphasis by position in a declared enum: the first value gets the first
# colour, the last the last. Usually `priority`, so p0 is the one you notice —
# but a project that calls it `severity` gets the same treatment for free.
ranks        = ["#c23127", "#d26939", "#33859d", "#245361"]

# Optional, and only if you want to overrule the project's own choices.
[types]
bug = "#c23127"

[statuses]
blocked = "#c23127"
```

Every key is optional. A file that names only `accent` is valid: unmentioned
roles inherit from `auto`, so an incomplete theme cannot produce an unreadable
screen. An unknown key is an error, so a typo is visible rather than silent, and
a colour that will not parse leaves that one role alone.

Colours may be written as `#rrggbb`, `#rgb`, `rgb:RR/GG/BB` as xterm writes it,
an ANSI slot number, a name (`cyan`, `bright-blue`), or `reset` for "whatever
the terminal already uses". A bare small number is an ANSI slot, which is how a
theme asks to follow the terminal for one particular role.

`selection_reverse = true` draws the selected row in reverse video instead of on
`selection`. It is what `auto` does, because the ground colour is unknown there.
A file that names a `selection` colour turns it off.

## Ghostty themes

Ghostty theme files are read directly, so a palette you already chose does not
have to be transcribed. `background`, `foreground`, `selection-background` and
the sixteen `palette` slots are mapped onto the roles above — bright slots
first where a role has to carry over a dark ground rather than recede into it.

## Writing one

Start from the built-in nearest what you want:

```sh
cp themes/night.toml ~/.config/harrow/themes/mine.toml
harrow --theme mine
```

`ctrl-r` re-reads it without restarting, so choosing colours is a loop rather
than a series of relaunches.
