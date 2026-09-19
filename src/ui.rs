//! All drawing.
//!
//! The screen is a header, a status strip, a body and a footer; overlays (read,
//! help, picker, confirm) are painted on top. The body is a list and a detail
//! pane where there is room for both, and the list alone where there is not —
//! because the place this runs is often a narrow pane beside the work.
//!
//! Rendering is a pure function of [`App`]: nothing here reads the clock, the
//! environment or the filesystem, which is what makes a frame reproducible and
//! a snapshot test worth having.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, ListState, Padding, Paragraph};

use crate::app::{App, Door, Hit, Pane, ReadOnly, Row, Target, ToastKind};
use crate::diag;
use crate::item::Item;
use crate::keys::Command;
use crate::schema::{Category, Schema};
use crate::theme::Theme;

const SPINNER: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

/// Below this, the detail pane costs the list more than it gives. A right-hand
/// pane beside an editor is usually sixty columns, and two panes in sixty is
/// two unreadable panes.
const DETAIL_MIN_WIDTH: u16 = 96;
/// What the detail needs to be worth drawing beside an arrangement that is
/// already divided.
const DETAIL_WIDTH: u16 = 44;
/// What one board column needs to stay a card rather than a stub.
const COLUMN_MIN: u16 = 26;
/// What a column with nothing in it needs: its name, its nought, its edges.
const EMPTY_COLUMN: u16 = 15;
/// What the stats pane needs to lay itself out in two columns, which is what
/// it does whenever it has the room: its sections are short, and one column
/// of them is mostly whitespace.
const STATS_TWO_COLUMN: u16 = 88;
/// Below this, the header drops to the identity and the counts.
const ROOMY: u16 = 74;
/// The least a reader is worth drawing in. Narrower than this and the prose is
/// being squeezed rather than read, so the lens steps aside and gives it the
/// body instead.
const READER_MIN: u16 = 62;
/// What the filter panel asks for: the longest value a project is likely to
/// declare, its count, its box, and the edges around them.
const FILTER_WIDTH: u16 = 26;
/// What has to be left over for the backlog to still be worth narrowing. Below
/// it the panel takes the body, because a filter you cannot read the result of
/// is not a filter, and one you cannot read is not either.
const FILTER_MIN_REST: u16 = 44;
/// The widest a value row is drawn, however wide the panel gets. A count at
/// the far end of a wide terminal is not beside the thing it counts.
const FILTER_ROW: u16 = 32;
/// The least a browsing column is worth keeping — an id, a glyph, and enough
/// title to tell two items apart.
const BROWSE_MIN: u16 = 30;
/// A line longer than this stops being read and starts being scanned. Running
/// text edge to edge is what makes a wide terminal worse to read in than a
/// narrow one.
const READER_MEASURE: u16 = 76;
/// The measure, plus the padding and borders around it. What the panel asks
/// for when the body can spare it, and never more: a wider browsing column is
/// useful and a wider column of prose is not.
const READER_PANEL: u16 = READER_MEASURE + 6;

/// One glyph per state, so the screen still says everything it needs to when
/// there is no colour at all — `mono`, `NO_COLOR`, or a reader who cannot tell
/// the green from the red.
pub fn glyph(item: &Item) -> &'static str {
    if item.blocked && !item.category.is_closed() {
        return "⊘";
    }
    category_glyph(item.category)
}

pub fn category_glyph(category: Category) -> &'static str {
    match category {
        Category::Open => "○",
        Category::Active => "◐",
        Category::Done => "✓",
        Category::Dropped => "×",
    }
}

/// How somebody's name is drawn.
///
/// Two groups: yours, and everybody else's. That is the first-order question
/// on a backlog you share with programs, and it is as far as colour should
/// be pushed here.
///
/// The rank scale was the obvious way to give every actor its own colour and
/// is the wrong one: those four are the priority colours, so a contributor
/// would be drawn in the p0 red for no reason but their position in a sorted
/// list. Colour on this row already carries state, type, priority and
/// staleness; a fifth meaning that is merely decorative is how a screen
/// becomes colourful instead of legible.
///
/// A stale claim outranks both, because *this is not moving* matters more
/// than *whose it is*. And with no colour at all the glyph carries it, the
/// way the state column does: yours is `@name`, somebody else's is `·name`.
fn actor_style(app: &App, who: &str, stale: bool, t: &Theme) -> (String, Style) {
    // `·` only where there is a *you* for it to mean *not you*. Where harrow
    // cannot tell — no `CAIRN_USER`, no git name — everybody is `@`, because
    // marking a distinction that cannot be drawn is worse than not drawing
    // it. The colours still separate the cast either way.
    let mark = if app.me.is_empty() || app.is_me(who) {
        "@"
    } else {
        "·"
    };
    let colour = if stale {
        t.warn
    } else if app.me.is_empty() || app.is_me(who) {
        t.person
    } else {
        t.secondary
    };
    (mark.to_string(), Style::default().fg(colour))
}

fn state_color(item: &Item, t: &Theme, schema: &Schema) -> ratatui::style::Color {
    if item.blocked && !item.category.is_closed() {
        return t.blocked;
    }
    t.status(schema.status(&item.status))
}

pub fn draw(f: &mut Frame, app: &mut App, tick: usize) {
    // Cloned once per frame rather than borrowed, so the drawing code can keep
    // taking `&mut App` for its own bookkeeping.
    let t = app.theme.clone();
    let area = f.area();
    // Where everything clickable lands is recorded as it is drawn, so the two
    // can never disagree about what is where.
    app.hits.clear();
    app.screen = area;
    let strip = u16::from(area.height >= 12 && !app.status_counts().is_empty());
    // Dropped on a short terminal for the same reason the strip is, and one
    // row sooner: a backlog you cannot see is worse than a view you cannot
    // read the rules of.
    let view = u16::from(area.height >= 14);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),     // identity, tabs, freshness
            Constraint::Length(strip), // what is happening
            Constraint::Length(view),  // what is on screen, and why
            Constraint::Length(1),     // rule
            Constraint::Min(3),        // the work
            Constraint::Length(1),     // keys, or what just happened
        ])
        .split(area);

    draw_header(f, app, &t, chunks[0], tick);
    if strip == 1 {
        draw_strip(f, app, &t, chunks[1]);
    }
    if view == 1 {
        draw_view_line(f, app, &t, chunks[2]);
    }
    draw_rule(f, &t, chunks[3]);

    // The detail belongs to the selection, not to the list: the same item is
    // selected whichever lens is showing, and there is no reason a board
    // should know less about it than a list does. What differs is how much
    // room a lens needs before it can spare the width — which is a property
    // of the arrangement, so each lens says.
    // Reading rearranges the body rather than covering it. The detail pane is
    // not drawn beside the reader: they answer the same question, and two
    // answers to one question is the thing a second pane is supposed to avoid.
    // The filter takes its column off the left before anything else divides
    // what is left: it is what you are doing *to* the backlog, so the backlog
    // and everything about one item stay together beside it.
    let (filter, body) = split_off_filter(app, chunks[4]);
    let reading = app.reading && app.selected_item().is_some();
    let (lens, detail, reader) = match body {
        None => (None, None, None),
        Some(body) if reading => {
            let (lens, reader) = split_off_reader(app, body);
            (lens, None, Some(reader))
        }
        Some(body) => {
            let (lens, detail) = split_off_detail(app, body);
            (Some(lens), detail, None)
        }
    };
    // Left as it was when the lens is not drawn, so the page-height the
    // scrolling arithmetic reads is the last one that meant anything.
    if let Some(body) = lens {
        match app.pane {
            Pane::Needs => draw_needs(f, app, &t, body),
            Pane::Log => draw_log(f, app, &t, body),
            Pane::Stats => draw_stats(f, app, &t, body),
            Pane::Board => {
                app.board_area = body;
                draw_board(f, app, &t, body);
            }
            Pane::List => {
                app.list_area = body;
                draw_list(f, app, &t, body);
            }
        }
    }
    if let Some(detail) = detail {
        draw_detail(f, app, &t, detail);
    }
    if let Some(reader) = reader {
        draw_reader(f, app, &t, reader);
    }
    if let Some(filter) = filter {
        draw_filter(f, app, &t, filter);
    }
    draw_footer(f, app, &t, chunks[5]);

    if app.history.is_some() {
        draw_history(f, app, &t, area);
    }
    if app.help {
        draw_help(f, app, &t, area);
    }
    if app.diagnostics {
        draw_diagnostics(f, app, &t, area);
    }
    if app.palette.is_some() {
        draw_palette(f, app, &t, area);
    }
    if app.picker.is_some() {
        draw_picker(f, app, &t, area);
    }
    if app.confirm.is_some() {
        draw_confirm(f, app, &t, area);
    }
}

// ── The header ───────────────────────────────────────────────────────────────

fn draw_header(f: &mut Frame, app: &mut App, t: &Theme, area: Rect, tick: usize) {
    let roomy = area.width >= ROOMY;
    let mut tabs: Vec<(Rect, Pane)> = Vec::new();
    let mut left = vec![Span::raw(" ")];
    if roomy {
        left.push(Span::styled("harrow", Style::default().fg(t.faint)));
        left.push(Span::styled("  ", Style::default()));
    }
    left.push(Span::styled(
        app.schema.name.clone(),
        Style::default().fg(t.heading).bold(),
    ));

    // Tabs, because there are three views and two of them were keys nobody
    // knew about. They are clickable, which is the point of drawing them.
    left.push(Span::raw("   "));
    let mut x = area.x
        + left
            .iter()
            .map(|s| s.content.chars().count() as u16)
            .sum::<u16>();
    for pane in Pane::ALL {
        let active = pane == app.pane;
        let label = format!(" {} ", pane.name());
        let width = label.chars().count() as u16;
        tabs.push((
            Rect {
                x,
                y: area.y,
                width,
                height: 1,
            },
            pane,
        ));
        x += width;
        left.push(Span::styled(
            label,
            if active {
                Style::default().bg(t.selection).fg(t.text).bold()
            } else {
                Style::default().fg(t.faint)
            },
        ));
    }

    // Absent when there is nothing, because a counter that is always there
    // is a counter nobody reads. This is the whole of how the queue asks for
    // attention from the other lenses; it does not nag beyond it.
    let needs = app.questions.len();
    if needs > 0 && roomy {
        left.push(Span::raw("   "));
        left.push(Span::styled(
            format!("{needs} needs you"),
            Style::default().fg(t.accent),
        ));
    }
    // Marked is a state you are in, so it is said in the header rather than
    // left to be counted off the rows.
    if !app.marked.is_empty() {
        left.push(Span::raw("   "));
        left.push(Span::styled(
            format!(" {} marked ", app.marked.len()),
            Style::default().bg(t.secondary).fg(t.background).bold(),
        ));
    }
    if roomy && !app.filter.is_empty() {
        left.push(Span::styled("   ", Style::default()));
        left.push(Span::styled(
            format!("/{}", truncate(&app.filter, 24)),
            Style::default().fg(t.warn),
        ));
    }
    if roomy && let Some(view) = &app.view {
        left.push(Span::styled("   ", Style::default()));
        left.push(Span::styled(
            format!("view {view}"),
            Style::default().fg(t.secondary),
        ));
    }

    let right = if let Some(fail) = &app.failure {
        format!("⚠ cannot read the backlog ({}×) ", fail.count)
    } else if !app.watcher_alive {
        "⚠ watcher stopped ".to_string()
    } else if app.loading {
        format!("{} reading ", SPINNER[tick % SPINNER.len()])
    } else {
        match app.last_load {
            Some(at) => {
                let e = at.elapsed();
                if e.as_secs() < 2 {
                    "just now ".to_string()
                } else if roomy {
                    format!("updated {} ago ", ago(e))
                } else {
                    format!("{} ", ago(e))
                }
            }
            None => String::from("starting "),
        }
    };

    for (rect, pane) in tabs {
        app.hit(rect, Hit::Tab(pane));
    }
    f.render_widget(Line::from(left), area);
    let right_style = if app.is_stale() {
        Style::default().fg(t.error).bold()
    } else {
        Style::default().fg(t.faint)
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(right, right_style))).alignment(Alignment::Right),
        area,
    );
}

/// What is happening, in one line.
///
/// The reason to keep this open in a pane beside the work: how much is moving,
/// how much is stuck, how much is done, without reading a single row.
fn draw_strip(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1usize;
    let room = area.width as usize;
    let mut cells: Vec<(Rect, String)> = Vec::new();

    for (status, count) in app.status_counts() {
        if status.category == Category::Dropped && !app.show_all {
            continue;
        }
        let colour = t.status(Some(status));
        let cell = format!(
            "{} {count} {}",
            category_glyph(status.category),
            status.display()
        );
        // Everything that does not fit is dropped from the right, so the
        // leftmost — what is active — survives a narrow pane.
        if used + cell.chars().count() + 3 > room {
            break;
        }
        if used > 1 {
            spans.push(Span::raw("   "));
            used += 3;
        }
        cells.push((
            Rect {
                x: area.x + used as u16,
                y: area.y,
                width: cell.chars().count() as u16,
                height: 1,
            },
            status.name.clone(),
        ));
        used += cell.chars().count();
        spans.push(Span::styled(
            format!("{} ", category_glyph(status.category)),
            Style::default().fg(colour),
        ));
        spans.push(Span::styled(
            count.to_string(),
            Style::default().fg(t.text).bold(),
        ));
        spans.push(Span::styled(
            format!(" {}", status.display()),
            Style::default().fg(t.muted),
        ));
    }

    for (rect, name) in cells {
        app.hit(rect, Hit::Status(name));
    }

    let blocked = app
        .items
        .iter()
        .filter(|i| i.blocked && !i.category.is_closed() && !i.container)
        .count();
    if blocked > 0 && used + 12 <= room {
        spans.push(Span::raw("   "));
        spans.push(Span::styled("⊘ ", Style::default().fg(t.blocked)));
        spans.push(Span::styled(
            blocked.to_string(),
            Style::default().fg(t.blocked).bold(),
        ));
        spans.push(Span::styled(" blocked", Style::default().fg(t.blocked)));
    }
    f.render_widget(Line::from(spans), area);
}

/// What is on screen, and why.
///
/// Three segments — which items, in what order, grouped how — each carrying
/// the key that edits it. The dimmed letter is not decoration: it is where the
/// cursor goes when you press it, so the line teaches its own controls.
///
/// Stated rather than remembered. A filter that arrived from `--filter`, from
/// `--view` or from a click on the status strip was never typed anywhere, so
/// until now the rows were narrowed and the screen did not say why.
fn draw_view_line(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let view = app.view_line();
    let keys = &app.keymap;
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1usize;
    let mut cells: Vec<(Rect, Command)> = Vec::new();

    // `(key, glyph, text, colour, what it opens)`, in the order the questions
    // are asked. Everything is pushed and then clipped from the right, so a
    // narrow pane keeps the filter — which is the one that changes what is
    // there rather than how it is laid out.
    let segment = |spans: &mut Vec<Span<'static>>,
                   used: &mut usize,
                   command: Option<Command>,
                   glyph: &str,
                   text: String,
                   colour: Color,
                   cells: &mut Vec<(Rect, Command)>| {
        if text.is_empty() {
            return;
        }
        let key = command.and_then(|c| keys.keys_for(c).into_iter().next());
        let lead = match &key {
            Some(k) => format!("{k} "),
            None => String::new(),
        };
        let body = format!("{glyph}{text}");
        let width = lead.chars().count() + body.chars().count();
        if *used + width + 3 > area.width as usize {
            return;
        }
        if *used > 1 {
            spans.push(Span::raw("   "));
            *used += 3;
        }
        if let Some(command) = command {
            cells.push((
                Rect {
                    x: area.x + *used as u16,
                    y: area.y,
                    width: width as u16,
                    height: 1,
                },
                command,
            ));
        }
        if !lead.is_empty() {
            spans.push(Span::styled(lead, Style::default().fg(t.faint)));
        }
        spans.push(Span::styled(body, Style::default().fg(colour)));
        *used += width;
    };

    // The name where one was chosen, and only while it still means what it
    // said: editing a clause makes it no longer that view.
    let filter = match (&view.view, view.query.is_empty()) {
        (Some(name), true) => name.clone(),
        (Some(name), false) => format!("{name} · {}", view.query),
        (None, _) => view.query.clone(),
    };
    segment(
        &mut spans,
        &mut used,
        Some(Command::Filter),
        "",
        if filter.is_empty() {
            "everything".to_string()
        } else {
            filter
        },
        if view.query.is_empty() && view.view.is_none() {
            t.faint
        } else {
            t.accent
        },
        &mut cells,
    );
    if app.show_all {
        segment(
            &mut spans,
            &mut used,
            Some(Command::ToggleAll),
            "+ ",
            "finished".to_string(),
            t.done,
            &mut cells,
        );
    }
    segment(
        &mut spans,
        &mut used,
        Some(Command::Sort),
        "↓ ",
        view.sort.replace(',', " "),
        if view.sort_is_default {
            t.faint
        } else {
            t.label
        },
        &mut cells,
    );
    segment(
        &mut spans,
        &mut used,
        Some(Command::GroupBy),
        "⊞ ",
        match view.group_by.as_str() {
            "none" | "" => "flat".to_string(),
            other => other.to_string(),
        },
        t.milestone,
        &mut cells,
    );

    // The tally on the right, where the freshness is on the line above.
    let (shown, total) = view.shown_and_total;
    let tally = if shown == total {
        format!("{total} items")
    } else {
        format!("{shown} of {total}")
    };
    let room = area.width as usize;
    if used + tally.chars().count() + 2 <= room {
        spans.push(Span::raw(
            " ".repeat(room - used - tally.chars().count() - 1),
        ));
        spans.push(Span::styled(tally, Style::default().fg(t.faint)));
    }

    f.render_widget(Line::from(spans), area);
    for (rect, command) in cells {
        app.hit(rect, Hit::Run(command));
    }
}

fn draw_rule(f: &mut Frame, t: &Theme, area: Rect) {
    let rule = "─".repeat(area.width as usize);
    f.render_widget(
        Line::from(Span::styled(rule, Style::default().fg(t.border))),
        area,
    );
}

// ── The list ─────────────────────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    // Just the name. The grouping used to be here because there was nowhere
    // else to put it; the view line says it now, beside the other two things
    // that decide what is on screen.
    let title = " Backlog ".to_string();
    // The focused pane is the one the keys are driving, and the border is where
    // that gets said. Without it, `↵` moves the keyboard somewhere the screen
    // does not admit to.
    let focused = app.focus == crate::app::Focus::List;
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { t.border_focus } else { t.border }))
        .title(Line::from(Span::styled(
            title,
            Style::default().fg(if focused { t.text } else { t.muted }),
        )));

    if app.rows.is_empty() {
        f.render_widget(
            Paragraph::new(empty_message(app, t))
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize;
    let inner_height = area.height.saturating_sub(2) as usize;

    // Only the rows that will be on screen are built, so the cost of a frame
    // scales with the size of the window rather than the size of the backlog.
    app.offset = scroll_to(app.offset, app.selected, app.rows.len(), inner_height);
    let end = (app.offset + inner_height).min(app.rows.len());
    let window: Vec<Row> = app.rows[app.offset.min(end)..end].to_vec();

    let items: Vec<ListItem> = window
        .iter()
        .map(|row| match row {
            Row::Group(g) => group_line(app, t, *g, inner_width),
            Row::Item(i) => item_line(app, &app.items[*i], t, inner_width),
        })
        .collect();

    for n in 0..window.len() {
        app.hit(
            Rect {
                x: area.x + 1,
                y: area.y + 1 + n as u16,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            Hit::Row(app.offset + n),
        );
    }

    let list = List::new(items).block(block).highlight_style(t.selected());
    let mut state =
        ListState::default().with_selected(Some(app.selected.saturating_sub(app.offset)));
    f.render_stateful_widget(list, area, &mut state);
}

fn empty_message(app: &App, t: &Theme) -> Vec<Line<'static>> {
    // A filter that did not parse is not an empty set, and saying "No
    // matches" for one is the difference between a true answer and a
    // silence that looks like one.
    if let Some(why) = app.filter_problem() {
        return vec![
            Line::from(""),
            Line::from(Span::styled(why, Style::default().fg(t.warn))),
            Line::from(Span::styled(
                "esc clears the filter.",
                Style::default().fg(t.faint),
            )),
        ];
    }

    // The same distinction, one step further in. An empty list is not the
    // same claim as an empty backlog, and for a project whose work is
    // finished it was the wrong one: seven milestones were open, the strip
    // was counting them, and the list said there was nothing here. Saying
    // *what is being left out* is the only version a reader can act on —
    // and it is what sends them to `a` rather than to rewriting a filter
    // that was never the problem.
    let hidden = app.hidden();
    let show_all = app
        .keymap
        .keys_for(Command::ToggleAll)
        .into_iter()
        .next()
        .map(|key| format!("{key} shows them."));

    let (headline, hint) = if app.items.is_empty() {
        (
            "Nothing in the backlog yet.".to_string(),
            "Press n to write the first item.".to_string(),
        )
    } else if !hidden.any() {
        // Nothing is being withheld, so the filter really is the answer.
        if !app.filter.is_empty() || app.view.is_some() {
            (
                "No matches.".to_string(),
                "esc clears the filter.".to_string(),
            )
        } else {
            (
                "Nothing here at all.".to_string(),
                "Press n to write an item.".to_string(),
            )
        }
    } else {
        let headline = match (hidden.closed > 0, hidden.containers > 0) {
            // The case this was written for.
            (true, true) => "Every piece of work here is finished.".to_string(),
            (false, true) => format!("Nothing open but {}.", hidden.containers_said()),
            _ => "Nothing open here.".to_string(),
        };
        let what = match (hidden.closed > 0, hidden.containers > 0) {
            (true, true) => format!(
                "{} still open, and {} finished.",
                hidden.containers_said(),
                hidden.closed
            ),
            (false, true) => format!("{} still open.", hidden.containers_said()),
            _ => format!("{} finished.", hidden.closed),
        };
        let hint = match &show_all {
            Some(key) => format!("{what} {key}"),
            None => what,
        };
        (headline, hint)
    };

    vec![
        Line::from(""),
        Line::from(Span::styled(headline, Style::default().fg(t.muted))),
        Line::from(Span::styled(hint, Style::default().fg(t.faint))),
    ]
}

/// Keep `selected` inside a window of `height` rows, moving as little as
/// possible — the list should not jump when the cursor is already visible.
fn scroll_to(offset: usize, selected: usize, len: usize, height: usize) -> usize {
    if height == 0 || len == 0 {
        return 0;
    }
    let max_offset = len.saturating_sub(height);
    let mut offset = offset.min(max_offset);
    if selected < offset {
        offset = selected;
    } else if selected >= offset + height {
        offset = selected + 1 - height;
    }
    offset.min(max_offset)
}

fn group_line(app: &App, t: &Theme, idx: usize, width: usize) -> ListItem<'static> {
    let g = &app.groups[idx];
    let collapsed = app.collapsed.contains(&g.key);
    let marker = if collapsed { " ▸ " } else { " ▾ " };

    let percent = g.percent(&app.items);
    // What is left, not what is listed: "2 of 30" beside "93%" reads as two of
    // thirty done, which is the opposite of what it says.
    let left = g.count.saturating_sub(g.done);
    let count = if left == 0 {
        "done".to_string()
    } else {
        format!("{left} left")
    };

    // Degrade in a defined order — the bar, then the percentage — so a narrow
    // pane loses the decoration rather than the name of the thing.
    const MIN_NAME: usize = 14;
    let room = width.saturating_sub(marker.chars().count());
    let bar = progress_bar(percent, 6);
    let candidates = [
        format!("  {bar} {percent:>3}%  {count} "),
        format!("  {percent:>3}%  {count} "),
        format!("  {count} "),
    ];
    let right = candidates
        .iter()
        .find(|r| room.saturating_sub(r.chars().count()) >= MIN_NAME)
        .unwrap_or_else(|| candidates.last().expect("one always fits"))
        .clone();

    let budget = room.saturating_sub(right.chars().count());
    let name = truncate(&g.label, budget);
    let pad = budget.saturating_sub(name.chars().count());

    // The list's own axis: a group heading is part of the list, not the board.
    let name_style = match app.group_by.as_str() {
        "status" => Style::default()
            .fg(t.status(app.schema.status(&g.key)))
            .bold(),
        _ if g.key.is_empty() => Style::default().fg(t.faint).italic(),
        _ => Style::default().fg(t.milestone).bold(),
    };

    ListItem::new(Line::from(vec![
        Span::styled(marker, Style::default().fg(t.faint)),
        Span::styled(name, name_style),
        Span::raw(" ".repeat(pad)),
        Span::styled(
            right,
            Style::default().fg(if percent == 100 { t.done } else { t.faint }),
        ),
    ]))
}

/// One item, in columns that hold still.
///
/// Everything to the right of the title is fixed width and right-aligned, so
/// the eye can run down a column instead of hunting along each row. What drops
/// first when the pane narrows is what answers the least: how much of the
/// acceptance is ticked, then who has it, then how urgent it is.
fn item_line(app: &App, item: &Item, t: &Theme, width: usize) -> ListItem<'static> {
    let schema = &app.schema;
    let reference = schema.format_id(item.id);
    let recent = app.is_recent(item.id);

    let rank = rank_tag(item, schema);
    let criteria = match item.criteria() {
        (_, 0) => String::new(),
        (done, total) => format!("{done}/{total}"),
    };
    let (mark, who_style) = item
        .assignee
        .as_deref()
        .map(|a| actor_style(app, a, app.claim_is_stale(item), t))
        .unwrap_or_else(|| (String::new(), Style::default()));
    let who = item
        .assignee
        .as_deref()
        .map(|a| format!("{mark}{}", truncate(a, 8)))
        .unwrap_or_default();
    let proposed = !item.proposals.is_empty();

    let lead = 2 + 1 + 1 + reference.chars().count() + 1;
    let avail = width.saturating_sub(lead + 1);

    const MIN_TITLE: usize = 22;
    let cost = |s: &str| {
        if s.is_empty() {
            0
        } else {
            s.chars().count() + 2
        }
    };
    let (mut show_criteria, mut show_who, mut show_rank) = (true, true, true);
    for _ in 0..3 {
        let reserved = if show_criteria { cost(&criteria) } else { 0 }
            + if show_who { cost(&who) } else { 0 }
            + if show_rank { cost(&rank) } else { 0 };
        if avail.saturating_sub(reserved) >= MIN_TITLE {
            break;
        }
        if show_criteria {
            show_criteria = false;
        } else if show_who {
            show_who = false;
        } else {
            show_rank = false;
        }
    }
    let reserved = if show_criteria { cost(&criteria) } else { 0 }
        + if show_who { cost(&who) } else { 0 }
        + if show_rank { cost(&rank) } else { 0 }
        + usize::from(proposed) * 2;

    let title_width = avail.saturating_sub(reserved);
    let title = truncate(&item.title, title_width);
    let pad = title_width.saturating_sub(title.chars().count());

    let closed = item.category.is_closed();
    let title_style = if closed {
        Style::default().fg(t.faint)
    } else {
        Style::default().fg(t.text)
    };

    let marked = app.marked.contains(&item.id);
    let mut spans = vec![
        // Two characters carry both facts: whether this is in the set the next
        // change applies to, and whether it moved a moment ago. Neither costs
        // any width, because the two lead spaces were there anyway.
        Span::styled(
            if marked { "▌" } else { " " },
            Style::default().fg(t.secondary).bold(),
        ),
        Span::styled(
            if recent { "•" } else { " " },
            Style::default().fg(t.accent).bold(),
        ),
        Span::styled(
            glyph(item),
            Style::default().fg(state_color(item, t, schema)),
        ),
        Span::raw(" "),
        Span::styled(
            reference,
            Style::default().fg(if recent { t.accent } else { t.faint }),
        ),
        Span::raw(" "),
        Span::styled(title, title_style),
        Span::raw(" ".repeat(pad)),
    ];
    if show_criteria && !criteria.is_empty() {
        let (done, total) = item.criteria();
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            criteria,
            Style::default().fg(if done == total { t.done } else { t.faint }),
        ));
    }
    if show_who && !who.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(who, who_style));
    }
    if proposed {
        // Somebody is waiting on an answer, which is a different kind of fact
        // from anything else on the row.
        spans.push(Span::styled(" ?", Style::default().fg(t.accent).bold()));
    }
    if show_rank && !rank.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(rank.clone(), rank_style(item, schema, t)));
    }
    ListItem::new(Line::from(spans))
}

/// The first enum field the project marked as a column — `priority`, usually.
/// Generic because a project that calls it `severity` deserves the same row.
fn rank_field(schema: &Schema) -> Option<&crate::schema::Field> {
    schema
        .fields
        .iter()
        .find(|f| f.column && !f.values.is_empty())
        .or_else(|| schema.field("priority"))
}

fn rank_tag(item: &Item, schema: &Schema) -> String {
    rank_field(schema)
        .and_then(|f| item.field_str(&f.name))
        .unwrap_or_default()
        .to_string()
}

/// The colour a rank value wears, by where the project put it in its own
/// order. The same ramp the list paints a rank tag with, asked by value rather
/// than by item, because the panel has a value and no item to hand.
fn rank_colour(schema: &Schema, value: &str, t: &Theme) -> Color {
    let Some(field) = rank_field(schema) else {
        return t.muted;
    };
    let index = field.rank(value);
    if index == usize::MAX {
        return t.muted;
    }
    t.rank(index, field.values.len())
}

fn rank_style(item: &Item, schema: &Schema, t: &Theme) -> Style {
    let Some(field) = rank_field(schema) else {
        return Style::default().fg(t.muted);
    };
    let Some(value) = item.field_str(&field.name) else {
        return Style::default().fg(t.muted);
    };
    let index = field.rank(value);
    if index == usize::MAX {
        return Style::default().fg(t.muted);
    }
    Style::default().fg(t.rank(index, field.values.len()))
}

fn progress_bar(percent: u32, width: usize) -> String {
    let filled = (percent as usize * width).div_ceil(100).min(width);
    let mut bar = String::with_capacity(width * 3);
    for i in 0..width {
        bar.push(if i < filled { '▰' } else { '▱' });
    }
    bar
}

// ── The board ────────────────────────────────────────────────────────────────

fn draw_board(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    if app.columns.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "This project declares no board columns.",
                Style::default().fg(t.faint),
            )))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            ),
            area,
        );
        return;
    }

    // Every column with cards in it the same width. The rule that matters is
    // that a board's shape is learnable — the same lanes, in the same order,
    // and peers the same size — and an empty lane needs room for its name,
    // not an equal share. Five lanes with three empty gave the hundred and
    // nineteen items twenty-eight columns and spent ninety on nothing.
    let count = app.columns.len();
    let filled = app.columns.iter().filter(|c| !c.items.is_empty()).count();
    let constraints: Vec<Constraint> = if filled == 0 || filled == count {
        (0..count)
            .map(|_| Constraint::Ratio(1, count as u32))
            .collect()
    } else {
        app.columns
            .iter()
            .map(|c| {
                if c.items.is_empty() {
                    Constraint::Length(EMPTY_COLUMN)
                } else {
                    Constraint::Min(COLUMN_MIN)
                }
            })
            .collect()
    };
    let cells = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // One per column, in case the board was drawn before it was dealt.
    app.column_offsets.resize(app.columns.len(), 0);

    let mut regions: Vec<(Rect, Hit)> = Vec::new();
    let mut offsets: Vec<usize> = Vec::with_capacity(count);
    for (index, cell) in cells.iter().enumerate() {
        let column = &app.columns[index];
        let focused = index == app.column;
        // A column is coloured by what it stands for, which depends on what
        // the board is grouped by. Falling back to the rank scale gives a
        // spread that reads as an order, which is what a declared sequence of
        // values is.
        let color = match app.board_by.as_str() {
            _ if column.value.is_empty() => t.faint,
            "status" => t.status(app.schema.status(&column.value)),
            "type" => t.item_type(app.schema.item_type(&column.value)),
            "assignee" => t.person,
            other if app.schema.field(other).is_some_and(|f| f.values.is_empty()) => t.milestone,
            _ => t.rank(index, count),
        };
        let border = if focused { color } else { t.border };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border))
            // The count is the half that must survive a narrow lane: a
            // truncated name is still a name, and `in progress` cut to
            // `in progress` with the nought gone reads as damage.
            .title({
                let n = column.items.len().to_string();
                let room = (cell.width as usize).saturating_sub(n.chars().count() + 5);
                Line::from(vec![
                    Span::styled(
                        format!(" {} ", truncate(&column.label, room)),
                        Style::default().fg(color).bold(),
                    ),
                    Span::styled(format!("{n} "), Style::default().fg(t.faint)),
                ])
            });

        let inner_width = cell.width.saturating_sub(2) as usize;
        let inner_height = cell.height.saturating_sub(2) as usize;
        // Where the column was left, pulled back to what it can hold. The
        // focused one gives way to its cursor: a card you are about to act on
        // has to be on screen, wherever the column had been scrolled to.
        let offset = app.column_offsets[index].min(column.items.len().saturating_sub(inner_height));
        let offset = if focused {
            scroll_to(offset, app.column_row, column.items.len(), inner_height)
        } else {
            offset
        };
        offsets.push(offset);
        let end = (offset + inner_height).min(column.items.len());

        let cards: Vec<ListItem> = column.items[offset.min(end)..end]
            .iter()
            .map(|i| card_line(app, &app.items[*i], t, inner_width))
            .collect();

        // The whole column takes a drop, so a card let go of anywhere in it
        // lands there; the cards themselves are registered after, and win.
        regions.push((*cell, Hit::Column(index)));
        for n in 0..cards.len() {
            regions.push((
                Rect {
                    x: cell.x + 1,
                    y: cell.y + 1 + n as u16,
                    width: cell.width.saturating_sub(2),
                    height: 1,
                },
                Hit::Card(index, offset + n),
            ));
        }

        // A column showing twenty of a hundred and nineteen should say so.
        // The heading carries the total; this is the part below the fold.
        let hidden = column.items.len().saturating_sub(end);
        let block = if hidden > 0 && inner_width > 12 {
            block.title_bottom(Span::styled(
                format!(" {hidden} more "),
                Style::default().fg(t.faint),
            ))
        } else {
            block
        };

        let list = List::new(cards).block(block).highlight_style(if focused {
            t.selected()
        } else {
            Style::default()
        });
        let mut state = ListState::default().with_selected(
            (focused && !column.items.is_empty()).then(|| app.column_row.saturating_sub(offset)),
        );
        f.render_stateful_widget(list, *cell, &mut state);
    }
    app.column_offsets = offsets;
    for (rect, hit) in regions {
        app.hit(rect, hit);
    }
}

fn card_line(app: &App, item: &Item, t: &Theme, width: usize) -> ListItem<'static> {
    let schema = &app.schema;
    let reference = schema.format_id(item.id);
    let recent = app.is_recent(item.id);
    let rank = rank_tag(item, schema);

    let lead = 2 + 1 + 1 + reference.chars().count() + 1;
    let rank_cost = if rank.is_empty() {
        0
    } else {
        rank.chars().count() + 2
    };
    let title_width = width.saturating_sub(lead + rank_cost);
    let title = truncate(&item.title, title_width);
    let pad = title_width.saturating_sub(title.chars().count());

    let mut spans = vec![
        Span::styled(
            if recent { " •" } else { "  " },
            Style::default().fg(t.accent).bold(),
        ),
        Span::styled(
            glyph(item),
            Style::default().fg(state_color(item, t, schema)),
        ),
        Span::raw(" "),
        Span::styled(reference, Style::default().fg(t.faint)),
        Span::raw(" "),
        Span::styled(title, Style::default().fg(t.text)),
        Span::raw(" ".repeat(pad)),
    ];
    if !rank.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(rank.clone(), rank_style(item, schema, t)));
    }
    ListItem::new(Line::from(spans))
}

/// Where the arrangement goes, and where the detail goes beside it.
///
/// The threshold is the lens's own, and it is the width at which the lens can
/// still do its job properly with the detail beside it — not the width at
/// which both technically fit.
///
/// A list wants half the terminal and is readable in fifty columns. A board
/// is already divided, so what it can spare depends on how many columns the
/// project declared: three can give the detail its width at a hundred and
/// twenty-two, five not until a hundred and seventy-four. The stats lay out
/// in two columns whenever they have room, so they keep the detail only when
/// there is room for both — otherwise a wide terminal would trade a layout
/// the pane prefers for a pane it did not ask for.
///
/// Below its threshold a lens keeps the whole body, which is the rule the
/// list has always followed: the detail gets out of the way rather than
/// halving something already too small.
fn split_off_detail(app: &App, body: Rect) -> (Rect, Option<Rect>) {
    let needed = match app.pane {
        // A question is one line and the item it is about is the other half
        // of it, so this lens wants the detail more than any of them.
        Pane::Needs | Pane::Log => DETAIL_MIN_WIDTH,
        // Kept as a proportion rather than a fixed width, because a list
        // goes on being more useful the wider it is and the recorded screens
        // are taken at these proportions.
        Pane::List => DETAIL_MIN_WIDTH,
        Pane::Board => COLUMN_MIN * app.columns.len().max(1) as u16 + DETAIL_WIDTH,
        Pane::Stats => STATS_TWO_COLUMN + DETAIL_WIDTH,
    };
    if body.width < needed || app.selected_item().is_none() {
        return (body, None);
    }
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(match app.pane {
            Pane::List => [Constraint::Percentage(56), Constraint::Percentage(44)],
            // Everything else keeps the room it needs and the detail takes
            // what it needs, rather than both growing and neither using it.
            _ => [Constraint::Min(0), Constraint::Length(DETAIL_WIDTH)],
        })
        .split(body);
    (split[0], Some(split[1]))
}

/// Where the filter panel goes, and what is left for everything else.
///
/// Placed by the room, the way the reader is. With enough left over it takes a
/// column off the left and the backlog carries on beside it; without, it takes
/// the body — a panel you cannot read is no more use than a result you cannot.
fn split_off_filter(app: &App, body: Rect) -> (Option<Rect>, Option<Rect>) {
    if !app.filtering {
        return (None, Some(body));
    }
    if body.width < FILTER_WIDTH + FILTER_MIN_REST {
        return (Some(body), None);
    }
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(FILTER_WIDTH), Constraint::Min(0)])
        .split(body);
    (Some(split[0]), Some(split[1]))
}

/// What the backlog can be narrowed by, and what each choice would leave.
///
/// Every heading and every row comes from the project's own schema. The panel
/// has no opinion about what a backlog is filtered by; it reads what the
/// project declared and offers that.
fn draw_filter(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let focused = app.focus == crate::app::Focus::Filter;
    let (shown, total) = app.shown_and_total();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { t.border_focus } else { t.border }))
        .padding(Padding::horizontal(1))
        .title(Span::styled(" Filter ", Style::default().fg(t.muted)))
        .title_bottom(Span::styled(
            match (shown == total, app.unmanaged_clauses()) {
                // Said rather than implied. A bound the panel cannot draw is
                // still narrowing every count on it.
                (_, n) if n > 0 => format!(" {shown} of {total} · {n} typed "),
                (true, _) => format!(" {total} items "),
                (false, _) => format!(" {shown} of {total} "),
            },
            Style::default().fg(if shown == total { t.faint } else { t.accent }),
        ));
    let inner = block.inner(area);
    // A value and its count, and no more: when the panel has the body to
    // itself the rows stop growing rather than stranding a count at the far
    // edge of a terminal from the label it belongs to.
    let room = (inner.width as usize).min(FILTER_ROW as usize);

    let mut lines: Vec<Line<'static>> = Vec::new();
    // Where the cursor is, counted the way the cursor counts: over values,
    // because a heading is not somewhere it can be.
    let mut nth = 0usize;
    let mut cursor_line = 0usize;
    for facet in &app.facets {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            facet.label.clone(),
            Style::default().fg(t.heading).bold(),
        )));
        for value in &facet.values {
            let here = nth == app.facet;
            if here {
                cursor_line = lines.len();
            }
            // An empty box and a ticked one, rather than colour alone: the
            // state of a checkbox has to survive a terminal with no colour.
            let box_glyph = if value.ticked { "☑" } else { "☐" };
            let colour = match value.role {
                crate::app::FacetRole::Status => t.status(app.schema.status(&value.value)),
                crate::app::FacetRole::Type => t.item_type(app.schema.item_type(&value.value)),
                crate::app::FacetRole::Rank => rank_colour(&app.schema, &value.value, t),
                crate::app::FacetRole::Plain => t.text,
            };
            // Nothing to find is dimmed rather than dropped: "there are no
            // p0s" is an answer, and an absence is not.
            let colour = if value.count == 0 { t.faint } else { colour };
            let count = value.count.to_string();
            let label = truncate(&value.label, room.saturating_sub(count.chars().count() + 5));
            let gap = room.saturating_sub(label.chars().count() + count.chars().count() + 4);
            let row = Line::from(vec![
                Span::styled(
                    format!(" {box_glyph} "),
                    Style::default().fg(if value.ticked { t.accent } else { t.faint }),
                ),
                Span::styled(label, Style::default().fg(colour)),
                Span::raw(" ".repeat(gap)),
                Span::styled(count, Style::default().fg(t.faint)),
            ]);
            // The same lift the list gives its own cursor, so "here" looks the
            // same wherever you are. Only while the panel holds the keys: a
            // cursor in a pane that is not listening is a lie about where you
            // are.
            lines.push(if here && focused {
                row.style(t.selected())
            } else {
                row
            });
            nth += 1;
        }
    }

    // The cursor stays in view, which is the one thing a pane with a cursor
    // owes a pane without one.
    let height = inner.height as usize;
    let over = lines.len().saturating_sub(height);
    let want = cursor_line.saturating_sub(height / 2);
    app.facet_scroll = (app.facet_scroll as usize).clamp(
        cursor_line
            .saturating_sub(height.saturating_sub(1))
            .min(over),
        cursor_line.min(over),
    ) as u16;
    if over == 0 {
        app.facet_scroll = 0;
    } else if cursor_line < app.facet_scroll as usize
        || cursor_line >= app.facet_scroll as usize + height
    {
        app.facet_scroll = want.min(over) as u16;
    }

    f.render_widget(
        Paragraph::new(lines)
            .scroll((app.facet_scroll, 0))
            .block(block),
        area,
    );
    app.hit(area, Hit::Filter);
    // The rows on top of the pane, so a click lands on the value it is over.
    let mut nth = 0usize;
    let mut y = inner.y as i32 - app.facet_scroll as i32;
    for facet in &app.facets {
        if nth > 0 {
            y += 1;
        }
        y += 1; // the heading
        for _ in &facet.values {
            if y >= inner.y as i32 && y < (inner.y + inner.height) as i32 {
                app.hits.push((
                    Rect {
                        x: inner.x,
                        y: y as u16,
                        width: inner.width,
                        height: 1,
                    },
                    Hit::Facet(nth),
                ));
            }
            nth += 1;
            y += 1;
        }
    }
}

/// Where the reader panel goes, and what the lens keeps.
///
/// The same judgement `split_off_detail` makes, and for the same reason each
/// lens gets a say in it: browse on the left and inspect on the right while
/// both fit, and the reader alone once they do not. A lens squeezed under
/// what it needs is not a browsing column, it is a stub beside the thing you
/// actually wanted, so below the threshold it steps aside entirely.
fn split_off_reader(app: &App, body: Rect) -> (Option<Rect>, Rect) {
    let browse = match app.pane {
        // A board browses by having its columns beside each other. One
        // squeezed column is not a board, so it asks for all of them.
        Pane::Board => COLUMN_MIN * app.columns.len().max(1) as u16,
        _ => BROWSE_MIN,
    };
    if body.width < browse.saturating_add(READER_MIN) {
        return (None, body);
    }
    // The reader takes the measure it can use and the lens keeps the rest,
    // rather than both growing and only one of them able to spend it.
    let reader = READER_PANEL.min(body.width.saturating_sub(browse));
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(reader)])
        .split(body);
    (Some(split[0]), split[1])
}

// ── The detail pane ──────────────────────────────────────────────────────────

fn draw_detail(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(item) = app.selected_item() else {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Select an item",
                Style::default().fg(t.faint),
            )))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(t.border))
                    .title(Span::styled(" Detail ", Style::default().fg(t.muted))),
            ),
            area,
        );
        return;
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .padding(Padding::horizontal(1))
        .title(Line::from(Span::styled(
            format!(" {} ", app.schema.format_id(item.id)),
            Style::default().fg(t.muted),
        )));

    let inner = block.inner(area);
    let id = item.id;
    let prose = detail_prose(app, item, t, inner.width as usize);

    // Clamped here because here is where the height of the content is known.
    // Past the end of a short item is not a place the pane can be.
    let over = prose.lines.len().saturating_sub(inner.height as usize);
    app.detail.clamp(over as u16);
    let scroll = app.detail.at(id);

    // A pane holding more than it shows says so on its own edge, with the keys
    // that move it — taken from the bindings in force, not from ours.
    let hint = app
        .keymap
        .scroll_hint(Command::DetailUp, Command::DetailDown);
    let block = match (over > 0, hint) {
        (true, Some(keys)) => block.title_bottom(Span::styled(
            format!(" {keys} scroll "),
            Style::default().fg(t.faint),
        )),
        _ => block,
    };

    f.render_widget(
        Paragraph::new(prose.lines).scroll((scroll, 0)).block(block),
        area,
    );
    // The pane first, so a click that lands on nothing in particular still
    // scrolls the pane rather than the list; the links on top of it, because
    // the last thing registered wins.
    app.hit(area, Hit::Detail);
    app.links.clear();
    for (line, x, w, target) in prose.links {
        let y = inner.y as i32 + line as i32 - scroll as i32;
        if y >= inner.y as i32 && y < (inner.y + inner.height) as i32 {
            app.hit(
                Rect {
                    x: inner.x + x,
                    y: y as u16,
                    width: w.min(inner.width.saturating_sub(x)),
                    height: 1,
                },
                Hit::Link(app.links.len()),
            );
        }
        app.links.push(target);
    }
}

/// The body of the detail pane.
///
/// Built as whole lines rather than handed to a wrapping widget: ratatui's wrap
/// does not know about the indent a line started with, so a wrapped paragraph
/// loses its left edge and the pane stops having one.
///
/// Everything the item has, at whatever length that comes to. What fits is the
/// pane's business, and the pane scrolls.
fn detail_prose(app: &App, item: &Item, t: &Theme, width: usize) -> Prose {
    let schema = &app.schema;
    let mut lines = Prose::default();

    // The title, wrapped under its own glyph.
    let title_width = width.saturating_sub(3);
    for (n, part) in wrap(&item.title, title_width).into_iter().enumerate() {
        lines.push(Line::from(vec![
            if n == 0 {
                Span::styled(
                    format!("{} ", glyph(item)),
                    Style::default().fg(state_color(item, t, schema)),
                )
            } else {
                Span::raw("  ")
            },
            Span::styled(part, Style::default().fg(t.heading).bold()),
        ]));
    }

    // One line that says where it stands.
    let mut state = vec![Span::raw("  ")];
    state.push(Span::styled(
        schema
            .status(&item.status)
            .map(|s| s.display().to_string())
            .unwrap_or_else(|| item.status.clone()),
        Style::default().fg(t.status(schema.status(&item.status))),
    ));
    state.push(Span::styled(" · ", Style::default().fg(t.faint)));
    state.push(Span::styled(
        item.kind.clone(),
        Style::default().fg(t.item_type(schema.item_type(&item.kind))),
    ));
    if let Some(milestone) = item.milestone() {
        state.push(Span::styled(" · ", Style::default().fg(t.faint)));
        state.push(Span::styled(
            milestone.to_string(),
            Style::default().fg(t.milestone),
        ));
    }
    if let Some(who) = &item.assignee {
        state.push(Span::styled(" · ", Style::default().fg(t.faint)));
        let stale = app.claim_is_stale(item);
        let (mark, style) = actor_style(app, who, stale, t);
        state.push(Span::styled(format!("{mark}{who}"), style));
        // Marking it raises the question; the pane is where there is room to
        // answer it.
        if let Some(days) = app.claimed_days(item).filter(|_| stale) {
            state.push(Span::styled(
                format!(" · held {days} days"),
                Style::default().fg(t.warn),
            ));
        }
    }
    // Only where it says something the assignee does not. The two fields
    // exist to separate *who is doing it* from *who is answerable*, which is
    // a distinction that only appears when they differ — and with a program
    // working and a person answerable, they do.
    lines.push(Line::from(state));
    // On its own line rather than crowding the state, which is already four
    // facts wide and would simply truncate this one away.
    if let Some(owner) = item
        .owner
        .as_ref()
        .filter(|o| Some(*o) != item.assignee.as_ref())
    {
        let (mark, style) = actor_style(app, owner, false, t);
        lines.push(Line::from(vec![
            Span::styled("  answerable ", Style::default().fg(t.faint)),
            Span::styled(format!("{mark}{owner}"), style),
        ]));
    }

    // The newest thing anybody said, at the top, because on an item somebody
    // else is working on that is the reason you opened it. The body below
    // still reads in its own order — a thread is oldest to newest, and
    // reversing it to put the news first would make the history unreadable
    // to save a keystroke.
    if let Some((when, said)) = crate::item::latest_entry(&item.body) {
        lines.push(Line::from(""));
        lines.push(section("Latest", t, width));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(when, Style::default().fg(t.faint)),
        ]));
        for part in wrap(&said, width.saturating_sub(2)).into_iter().take(3) {
            lines.push(Line::from(Span::styled(
                format!("  {part}"),
                Style::default().fg(t.text),
            )));
        }
    }

    if item.blocked {
        lines.push(Line::from(""));
        lines.push(section("Waiting on", t, width));
        for id in &item.blockers {
            let title = app
                .items
                .iter()
                .find(|i| i.id == *id)
                .map(|i| i.title.clone())
                .unwrap_or_default();
            let reference = schema.format_id(*id);
            let width_of = reference.chars().count() as u16;
            let room = width.saturating_sub(reference.chars().count() + 3);
            let title = truncate(&title, room);
            let reach = width_of + 1 + title.chars().count() as u16;
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    reference,
                    Style::default().fg(t.blocked).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(title, Style::default().fg(t.muted)),
            ]));
            // The whole line, not just the id: the thing you want to reach is
            // the item you can read the title of, and a two-character target
            // is a target you miss.
            lines.link(2, reach, Target::Item(*id));
        }
    }

    for proposal in &item.proposals {
        lines.push(Line::from(""));
        lines.push(section("Proposed", t, width));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(proposal.field.clone(), Style::default().fg(t.muted)),
            Span::raw(" "),
            Span::styled(proposal.from.clone(), Style::default().fg(t.faint)),
            Span::styled(" → ", Style::default().fg(t.faint)),
            Span::styled(proposal.to.clone(), Style::default().fg(t.accent).bold()),
        ]));
        let by = if proposal.when.is_empty() {
            proposal.by.clone()
        } else {
            format!("{} · {}", proposal.by, proposal.when)
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                truncate(&by, width.saturating_sub(2)),
                Style::default().fg(t.person),
            ),
        ]));
        for part in wrap(&proposal.why, width.saturating_sub(2)) {
            lines.push(Line::from(Span::styled(
                format!("  {part}"),
                Style::default().fg(t.muted),
            )));
        }
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("A", Style::default().fg(t.accent).bold()),
            Span::styled(" to accept it", Style::default().fg(t.faint)),
        ]));
    }

    // A tally rather than a bar. A bar says *how far along*, which for a
    // milestone is the least useful thing about it — the question is where
    // the remaining work is, and the three numbers answer it in less room
    // than the bar took.
    if item.progress().is_some() {
        lines.push(Line::from(""));
        lines.push(section("Rollup", t, width));
        let scheduled = app.rollup(item);
        let mut spans = vec![Span::raw("  ")];
        for (n, (mark, count, colour, what)) in [
            ("✓", scheduled.done, t.done, "done"),
            ("◐", scheduled.active, t.active, "in flight"),
            ("○", scheduled.open, t.open, "to start"),
        ]
        .into_iter()
        .filter(|(_, count, _, _)| *count > 0)
        .enumerate()
        {
            if n > 0 {
                spans.push(Span::styled(" · ", Style::default().fg(t.faint)));
            }
            spans.push(Span::styled(
                format!("{mark} {count}"),
                Style::default().fg(colour).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {what}"),
                Style::default().fg(t.muted),
            ));
        }
        if scheduled.blocked > 0 {
            spans.push(Span::styled(" · ", Style::default().fg(t.faint)));
            spans.push(Span::styled(
                format!("⊘ {} blocked", scheduled.blocked),
                Style::default().fg(t.blocked),
            ));
        }
        lines.push(Line::from(spans));
    }

    // The criteria themselves, in the room the bar was using. `3 of 3 ticked`
    // says how many; this says which, and `t` ticks the one you are looking
    // at, so the pane is the place the work is recorded and not a readout of
    // it happening elsewhere.
    let criteria = crate::item::criteria_at(&item.body, app.schema.criteria_section.as_deref());
    let hoisted = hoisted_lines(&item.body, &criteria);
    if !criteria.is_empty() {
        let met = criteria.iter().filter(|(_, ticked, _)| *ticked).count();
        // The one `t` would offer first, marked, so the key and the pane
        // agree about which criterion is next.
        let next = app
            .can_tick
            .then(|| criteria.iter().position(|(_, ticked, _)| !ticked))
            .flatten();
        lines.push(Line::from(""));
        lines.push(section("Acceptance", t, width));
        for (n, (_, ticked, text)) in criteria.iter().enumerate() {
            let here = next == Some(n);
            lines.hanging(
                &inline(text),
                t,
                if *ticked { t.faint } else { t.text },
                (
                    "  ",
                    vec![
                        Span::styled(
                            if here { "▸ " } else { "  " },
                            Style::default().fg(t.accent),
                        ),
                        Span::styled(
                            if *ticked { "✓ " } else { "☐ " },
                            Style::default().fg(if *ticked { t.done } else { t.faint }),
                        ),
                    ],
                ),
                width.saturating_sub(6),
            );
        }
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{met} of {} ticked", criteria.len()),
                Style::default().fg(t.faint),
            ),
        ]));
    }

    if !item.body.trim().is_empty() {
        let rest = body_prose_without(&item.body, &hoisted, t, width.saturating_sub(2));
        if rest.lines.iter().any(|l| l.width() > 0) {
            lines.push(Line::from(""));
            lines.push(section("Body", t, width));
            lines.extend(rest);
        }
    }

    // Below the body, not above it. A grid of fields is reference material
    // and it is static; it had the top of the pane because it is easy to lay
    // out, not because it answers anything. What a reader wants from an item
    // somebody else is working on is the newest thing said about it.
    //
    // A grid, not a list: one column of labels, one of values, so the eye runs
    // down the labels instead of reading every line to find the one it wants.
    let mut fields: Vec<(String, String, Color)> = Vec::new();
    for field in &schema.fields {
        if field.name == "milestone" {
            continue;
        }
        if let Some(value) = item.field(&field.name).filter(|v| !v.is_empty()) {
            fields.push((field.name.clone(), value.display(), t.muted));
        }
    }
    // The one row in the grid that carries a role of its own. Labels are how
    // a project says what an item is *about*, across every other field, and
    // the theme names a colour for saying so.
    if !item.labels.is_empty() {
        fields.push(("labels".into(), item.labels.join(", "), t.label));
    }
    if let Some(by) = &item.created_by {
        fields.push(("filed by".into(), by.clone(), t.person));
    }
    for (label, value) in [("created", &item.created), ("updated", &item.updated)] {
        if let Some(value) = value {
            fields.push((label.into(), value.clone(), t.muted));
        }
    }
    if !fields.is_empty() {
        let label_width = fields
            .iter()
            .map(|(k, _, _)| k.chars().count())
            .max()
            .unwrap_or(0)
            .min(12);
        lines.push(Line::from(""));
        lines.push(section("Fields", t, width));
        for (label, value, colour) in fields {
            let room = width.saturating_sub(label_width + 3);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{label:<label_width$}"),
                    Style::default().fg(t.faint),
                ),
                Span::raw(" "),
                Span::styled(truncate(&value, room), Style::default().fg(colour)),
            ]));
        }
    }

    lines
}

/// Which lines of the body the Acceptance section has taken over.
///
/// The criteria themselves, and — where removing them empties a heading of
/// everything but blank lines — that heading too. A body that says nothing
/// under `## Acceptance criteria` but the criteria should not be left with a
/// heading standing over a hole.
fn hoisted_lines(
    body: &str,
    criteria: &[(usize, bool, String)],
) -> std::collections::HashSet<usize> {
    let mut skip: std::collections::HashSet<usize> = criteria.iter().map(|(n, _, _)| *n).collect();
    if skip.is_empty() {
        return skip;
    }
    let lines: Vec<&str> = body.lines().collect();
    let level = |s: &str| {
        let hashes = s.trim_start().chars().take_while(|c| *c == '#').count();
        (1..=6).contains(&hashes).then_some(hashes)
    };
    for (n, line) in lines.iter().enumerate() {
        let Some(depth) = level(line) else { continue };
        let end = lines
            .iter()
            .enumerate()
            .skip(n + 1)
            .find(|(_, l)| level(l).is_some_and(|d| d <= depth))
            .map(|(i, _)| i)
            .unwrap_or(lines.len());
        let empty = (n + 1..end).all(|i| skip.contains(&i) || lines[i].trim().is_empty());
        if empty {
            skip.extend(n..end);
        }
    }
    skip
}

/// A heading, with a rule running out to the edge. Cheaper to scan than a
/// column of capitals, and it gives the pane a horizontal rhythm.
fn section(name: &str, t: &Theme, width: usize) -> Line<'static> {
    let used = name.chars().count() + 4;
    let rule = "─".repeat(width.saturating_sub(used));
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            name.to_string(),
            Style::default().fg(t.muted).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(rule, Style::default().fg(t.border)),
    ])
}

/// A rendered body, and where the links in it landed.
///
/// The two travel together for the reason the stats pane's doors do: a link
/// drawn in one place and registered in another is a target that moves when
/// the layout does.
#[derive(Default)]
pub struct Prose {
    lines: Vec<Line<'static>>,
    /// `(line, x, width, target)`, x relative to the pane's inner area.
    links: Vec<(u16, u16, u16, Target)>,
}

impl Prose {
    fn push(&mut self, line: Line<'static>) {
        self.lines.push(line);
    }

    fn blank(&mut self) {
        self.lines.push(Line::from(""));
    }

    /// Record that the span just pushed, at `x` and `width` columns wide,
    /// leads somewhere.
    fn link(&mut self, x: u16, width: u16, target: Target) {
        let line = self.lines.len().saturating_sub(1) as u16;
        self.links.push((line, x, width, target));
    }

    /// Fold another body's lines in, keeping its links pointing at the same
    /// text now that it sits further down the pane.
    fn extend(&mut self, other: Prose) {
        let offset = self.lines.len() as u16;
        self.links.extend(
            other
                .links
                .into_iter()
                .map(|(l, x, w, d)| (l + offset, x, w, d)),
        );
        self.lines.extend(other.lines);
    }

    /// Lay pieces out at an indent, recording where any links landed.
    fn paragraph(&mut self, pieces: &[Piece], t: &Theme, base: Color, indent: &str, width: usize) {
        self.hanging(pieces, t, base, (indent, Vec::new()), width);
    }

    /// A quotation, under a gutter that runs its whole height.
    fn quote(&mut self, pieces: &[Piece], t: &Theme, width: usize) {
        for row in wrap_pieces(pieces, width) {
            let mut spans = vec![
                Span::raw("  "),
                Span::styled("▏ ", Style::default().fg(t.border)),
            ];
            for piece in row {
                spans.push(Span::styled(piece.text, ink(piece.kind, t, t.muted)));
            }
            self.lines.push(Line::from(spans));
        }
    }

    /// The same, under a marker the first line carries and the rest align to:
    /// a bullet, a number, a checkbox, a quotation's gutter.
    fn hanging(
        &mut self,
        pieces: &[Piece],
        t: &Theme,
        base: Color,
        (indent, marker): (&str, Vec<Span<'static>>),
        width: usize,
    ) {
        let lead: usize = marker.iter().map(|s| s.content.chars().count()).sum();
        for (n, row) in wrap_pieces(pieces, width).into_iter().enumerate() {
            let mut x = indent.chars().count() as u16;
            let mut spans = vec![Span::raw(indent.to_string())];
            if n == 0 {
                spans.extend(marker.iter().cloned());
            } else {
                // Aligned under the first line's text rather than its
                // marker, which is what makes a wrapped list still look
                // like a list.
                spans.push(Span::raw(" ".repeat(lead)));
            }
            x += lead as u16;
            for piece in row {
                let w = piece.text.chars().count() as u16;
                if let Some(href) = &piece.href {
                    // Trimmed of the space a wrap put in front of it, so the
                    // target is the words and not the gap before them.
                    let gap = u16::from(piece.text.starts_with(' '));
                    self.links.push((
                        self.lines.len() as u16,
                        x + gap,
                        w - gap,
                        Target::Url(href.clone()),
                    ));
                }
                spans.push(Span::styled(piece.text, ink(piece.kind, t, base)));
                x += w;
            }
            self.lines.push(Line::from(spans));
        }
    }
}

/// Markdown, at the fidelity a pane this size earns.
///
/// Not a CommonMark implementation and not trying to be. The line it holds:
/// this renders what cairn writes and what people write in cairn bodies, and
/// **anything it does not recognise comes out as the text that was typed**.
/// A renderer that swallows what it cannot parse is worse than one that
/// renders nothing, because the reader cannot tell what is missing.
fn body_prose(body: &str, t: &Theme, width: usize) -> Prose {
    body_prose_without(body, &std::collections::HashSet::new(), t, width)
}

/// The same, with certain lines left out — the detail pane hoists the
/// acceptance criteria above the body, and printing them twice would be the
/// pane arguing with itself.
/// `width` is how many columns the *text* may use; every line is drawn two
/// columns in from that, which is how the body lines up under the section
/// rules above it.
fn body_prose_without(
    body: &str,
    skip: &std::collections::HashSet<usize>,
    t: &Theme,
    width: usize,
) -> Prose {
    let mut out = Prose::default();
    let mut blank = false;
    let mut fence: Option<String> = None;
    let mut open = Open::None;

    for (n, raw) in body.lines().enumerate() {
        if skip.contains(&n) {
            continue;
        }
        let trimmed = raw.trim_start();

        // Inside a fence nothing is markup, which is the point of a fence.
        if let Some(marker) = &fence {
            if trimmed.starts_with(marker.as_str()) {
                fence = None;
            } else {
                out.push(code_line(raw, t, width));
                blank = false;
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            open.flush(&mut out, t, width);
            fence = Some(trimmed.chars().take(3).collect());
            blank = false;
            continue;
        }

        if trimmed.is_empty() {
            open.flush(&mut out, t, width);
            // One blank line between things, never three.
            if !out.lines.is_empty() && !blank {
                out.blank();
            }
            blank = true;
            continue;
        }

        let indent = raw.len() - trimmed.len();

        // A rule, which is a paragraph break somebody drew.
        if rule(trimmed) {
            open.flush(&mut out, t, width);
            out.push(Line::from(Span::styled(
                format!("  {}", "─".repeat(width)),
                Style::default().fg(t.border),
            )));
            blank = false;
            continue;
        }

        // Levels differ, and none of them looks like the pane's own section
        // headings — those carry a rule out to the edge, and a body heading
        // must not be mistaken for one.
        if let Some((level, text)) = heading(trimmed) {
            open.flush(&mut out, t, width);
            // Told apart by weight and not only by colour, so the levels are
            // still three different things on a monochrome terminal — and
            // none of them looks like the pane's own section headings, which
            // carry a rule out to the edge.
            let style = match level {
                1 => Style::default().fg(t.heading).bold().underlined(),
                2 => Style::default().fg(t.heading).bold(),
                _ => Style::default().fg(t.heading).italic(),
            };
            for row in wrap_pieces(&inline(text), width) {
                let text: String = row.iter().map(|p| p.text.as_str()).collect();
                out.push(Line::from(Span::styled(format!("  {text}"), style)));
            }
            blank = false;
            continue;
        }

        // An indented line is code or a command; it keeps its own shape. Only
        // where it does not continue a list, where four spaces is nesting.
        if raw.starts_with("    ") && !matches!(open, Open::Item { .. }) {
            open.flush(&mut out, t, width);
            out.push(code_line(raw.strip_prefix("    ").unwrap_or(raw), t, width));
            blank = false;
            continue;
        }

        // A second `>` continues the quotation rather than starting another
        // one: a quotation is hard-wrapped in the file like any paragraph.
        if trimmed.starts_with('>') && matches!(open, Open::Quote(_)) {
            open.continues(trimmed);
            blank = false;
            continue;
        }
        if let Some(started) = Open::starting(trimmed, indent) {
            open.flush(&mut out, t, width);
            open = started;
            blank = false;
            continue;
        }

        // Anything else continues whatever is open, joined back into one
        // paragraph: a body is hard-wrapped at the width its author was
        // working at, and re-wrapping each of those lines on its own
        // reproduces their ragged edge inside a pane of a different width.
        open.continues(trimmed);
        blank = false;
    }
    open.flush(&mut out, t, width);
    // A body that ends in blank lines, or one whose last section was hoisted
    // away, should not push a gap down in front of whatever follows it.
    while out.lines.last().is_some_and(|l| l.width() == 0) {
        out.lines.pop();
    }
    out
}

/// The block the reader is in the middle of.
///
/// Markdown's blocks run past their first line: a list item, a quotation and
/// a paragraph all continue onto the next line unless something ends them.
/// Held open until something does, so a wrapped line keeps the shape of the
/// thing it belongs to instead of starting a new one at the left margin.
enum Open {
    None,
    Para(String),
    Quote(String),
    /// A bullet, a number, or a checkbox — everything with a marker and a
    /// hanging indent under it.
    Item {
        marker: String,
        colour: Mark,
        indent: usize,
        text: String,
    },
}

/// What a list marker is, which is all the colour depends on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Bullet,
    Ticked,
    Unticked,
}

impl Open {
    /// Whether this line begins a block, and which.
    fn starting(trimmed: &str, indent: usize) -> Option<Open> {
        let item = |marker: String, colour: Mark, text: &str| Open::Item {
            marker,
            colour,
            indent,
            text: text.to_string(),
        };
        if let Some(rest) = trimmed
            .strip_prefix("> ")
            .or_else(|| trimmed.strip_prefix(">"))
        {
            return Some(Open::Quote(rest.trim().to_string()));
        }
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            // A box, where there is one after the marker.
            if let Some(after) = rest.strip_prefix("[ ]") {
                return Some(item("☐".into(), Mark::Unticked, after.trim_start()));
            }
            if let Some(after) = rest
                .strip_prefix("[x]")
                .or_else(|| rest.strip_prefix("[X]"))
            {
                return Some(item("✓".into(), Mark::Ticked, after.trim_start()));
            }
            return Some(item("·".into(), Mark::Bullet, rest));
        }
        // The author's own number, kept rather than renumbered: a list that
        // starts at 3 starts at 3 because somebody meant it to.
        ordered(trimmed).map(|(number, rest)| item(format!("{number}."), Mark::Bullet, rest))
    }

    /// Another line of the same block.
    fn continues(&mut self, trimmed: &str) {
        let text = match self {
            Open::None => {
                *self = Open::Para(trimmed.to_string());
                return;
            }
            Open::Para(text) | Open::Quote(text) | Open::Item { text, .. } => text,
        };
        // A quotation's continuation still carries its marker.
        let trimmed = trimmed.strip_prefix("> ").unwrap_or(trimmed);
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(trimmed.trim_end());
    }

    fn flush(&mut self, out: &mut Prose, t: &Theme, width: usize) {
        match std::mem::replace(self, Open::None) {
            Open::None => {}
            Open::Para(text) => {
                out.paragraph(&inline(&text), t, t.muted, "  ", width);
            }
            Open::Quote(text) => {
                // A gutter rather than a `>`: it says where the quotation
                // starts and stops without being read as part of it, and it
                // runs the whole height of the quotation rather than
                // marking only where it began.
                out.quote(&inline(&text), t, width.saturating_sub(2));
            }
            Open::Item {
                marker,
                colour,
                indent,
                text,
            } => {
                let (mark, body) = match colour {
                    Mark::Bullet => (t.faint, t.muted),
                    Mark::Ticked => (t.done, t.faint),
                    Mark::Unticked => (t.faint, t.muted),
                };
                let pad = " ".repeat(2 + indent);
                let lead = marker.chars().count() + 1;
                out.hanging(
                    &inline(&text),
                    t,
                    body,
                    (
                        &pad,
                        vec![Span::styled(
                            format!("{marker} "),
                            Style::default().fg(mark),
                        )],
                    ),
                    width.saturating_sub(indent + lead),
                );
            }
        }
    }
}

/// Code keeps every space it was written with, and is never wrapped: a
/// wrapped line of code is a line of code that has been altered.
fn code_line(raw: &str, t: &Theme, width: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ▏ ", Style::default().fg(t.border)),
        Span::styled(
            truncate(raw.trim_end(), width.saturating_sub(4)),
            Style::default().fg(t.code),
        ),
    ])
}

/// `# `, `## `, `### ` — the level, and what follows it.
fn heading(trimmed: &str) -> Option<(usize, &str)> {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    (1..=6)
        .contains(&hashes)
        .then(|| trimmed[hashes..].strip_prefix(' '))
        .flatten()
        .map(|rest| (hashes, rest))
}

/// `1. ` and friends. The author's own number is kept.
fn ordered(trimmed: &str) -> Option<(&str, &str)> {
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 || digits > 3 {
        return None;
    }
    let rest = trimmed[digits..].strip_prefix(". ")?;
    Some((&trimmed[..digits], rest))
}

/// `---`, `***`, `___` on a line of their own.
fn rule(trimmed: &str) -> bool {
    let t = trimmed.trim_end();
    t.len() >= 3
        && (t.chars().all(|c| c == '-')
            || t.chars().all(|c| c == '*')
            || t.chars().all(|c| c == '_'))
}

fn body_lines(body: &str, t: &Theme, width: usize) -> Vec<Line<'static>> {
    body_prose(body, t, width).lines
}

/// One piece of a line, once the markup has been read off it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Piece {
    text: String,
    kind: Ink,
    /// Where it goes, for the pieces that go somewhere.
    href: Option<String>,
}

/// How a piece is drawn. Not a style: the theme decides that, and this only
/// says what the markup claimed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ink {
    Plain,
    Strong,
    Emphasis,
    Code,
    Link,
}

/// Read the inline markup off a line.
///
/// Rendered rather than deleted, which is what this replaces: `plain_markdown`
/// stripped `**` and backticks, so emphasis became ordinary prose and a
/// command stopped looking like one.
///
/// The rule for everything it does not understand is that the source text
/// survives. An unclosed `**` is two asterisks somebody typed, not an
/// invitation to swallow the rest of the paragraph — which is the failure
/// mode of every half-written markdown renderer.
fn inline(text: &str) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    let mut plain = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    // `[text](url)`, taking the text and keeping the destination.
    let link_at = |from: usize| -> Option<(String, String, usize)> {
        if chars.get(from) != Some(&'[') {
            return None;
        }
        let close = (from + 1..chars.len()).find(|n| chars[*n] == ']')?;
        if chars.get(close + 1) != Some(&'(') {
            return None;
        }
        let end = (close + 2..chars.len()).find(|n| chars[*n] == ')')?;
        Some((
            chars[from + 1..close].iter().collect(),
            chars[close + 2..end].iter().collect(),
            end + 1,
        ))
    };

    // A bare `https://…`, which people write far more often than they write
    // the bracket form. It shows as itself — there is no other text to show —
    // but it is still a target.
    let url_at = |from: usize| -> Option<(String, usize)> {
        let rest: String = chars[from..].iter().collect();
        if !(rest.starts_with("https://") || rest.starts_with("http://")) {
            return None;
        }
        // Only at a word boundary, so `see_https://x` is not half a link.
        if from > 0 && !chars[from - 1].is_whitespace() && !"([<\"'".contains(chars[from - 1]) {
            return None;
        }
        let mut end = from + rest.find(char::is_whitespace).unwrap_or(rest.len());
        // A sentence ends in a full stop and the full stop is not part of the
        // address; nor is the bracket the address was written inside.
        while end > from && ".,;:!?)]}>\"'".contains(chars[end - 1]) {
            end -= 1;
        }
        let url: String = chars[from..end].iter().collect();
        (url.len() > "https://".len()).then_some((url, end))
    };

    // A run delimited by the same marker on both sides, with no blank
    // between: `**a**`, `*a*`, `` `a` ``.
    let run_at = |from: usize, marker: &str| -> Option<(String, usize)> {
        let m: Vec<char> = marker.chars().collect();
        if chars[from..].len() < m.len() * 2 + 1 || chars[from..from + m.len()] != m[..] {
            return None;
        }
        let start = from + m.len();
        // `a * b * c` is three words with asterisks between them, not
        // emphasis: an opener is not followed by a space and a closer is not
        // preceded by one. Without this every `*` in a sentence pairs up
        // with the next one and swallows what is between them.
        if chars.get(start).is_none_or(|c| c.is_whitespace()) {
            return None;
        }
        let mut n = start;
        while n + m.len() <= chars.len() {
            if chars[n..n + m.len()] == m[..] && !chars[n - 1].is_whitespace() {
                let body: String = chars[start..n].iter().collect();
                return (!body.trim().is_empty()).then_some((body, n + m.len()));
            }
            n += 1;
        }
        None
    };

    while i < chars.len() {
        // A backslash escapes the next character into the text as itself.
        if chars[i] == '\\' && i + 1 < chars.len() {
            plain.push(chars[i + 1]);
            i += 2;
            continue;
        }
        let found = if let Some((label, href, next)) = link_at(i) {
            Some((label, Ink::Link, Some(href), next))
        } else if let Some((url, next)) = url_at(i) {
            Some((url.clone(), Ink::Link, Some(url), next))
        } else if let Some((body, next)) = run_at(i, "`") {
            // Code first and literally: backticks quote the markup inside
            // them, which is how `**` gets written about at all.
            Some((body, Ink::Code, None, next))
        } else if let Some((body, next)) = run_at(i, "**") {
            Some((body, Ink::Strong, None, next))
        } else if let Some((body, next)) = run_at(i, "*") {
            Some((body, Ink::Emphasis, None, next))
        } else {
            None
        };
        match found {
            Some((body, kind, href, next)) => {
                if !plain.is_empty() {
                    out.push(Piece {
                        text: std::mem::take(&mut plain),
                        kind: Ink::Plain,
                        href: None,
                    });
                }
                out.push(Piece {
                    text: body,
                    kind,
                    href,
                });
                i = next;
            }
            None => {
                plain.push(chars[i]);
                i += 1;
            }
        }
    }
    if !plain.is_empty() {
        out.push(Piece {
            text: plain,
            kind: Ink::Plain,
            href: None,
        });
    }
    out
}

/// Break styled pieces across lines on word boundaries, the way `wrap` does
/// for a plain string.
///
/// A word is what is between two spaces in the *rendered* text, which is not
/// the same as a piece: `in`​`line`​`code` written with backticks in the middle
/// is three pieces and one word, and breaking between them would put half a
/// word on the next line. So the pieces are cut into words first, a word
/// keeping whatever pieces it spans, and the break only ever falls where a
/// space was actually written.
fn wrap_pieces(pieces: &[Piece], width: usize) -> Vec<Vec<Piece>> {
    if width == 0 {
        return vec![Vec::new()];
    }

    // One entry per word: the pieces it is made of, and how wide it draws.
    let mut words: Vec<(Vec<Piece>, usize)> = Vec::new();
    let mut space = true;
    for piece in pieces {
        for part in piece.text.split_inclusive(char::is_whitespace) {
            let trailing = part.ends_with(char::is_whitespace);
            let word = part.trim_end();
            if !word.is_empty() {
                let fragment = Piece {
                    text: word.to_string(),
                    kind: piece.kind,
                    href: piece.href.clone(),
                };
                match words.last_mut() {
                    // Still the same word: the piece changed, the text did
                    // not break.
                    Some((parts, len)) if !space => {
                        *len += word.chars().count();
                        parts.push(fragment);
                    }
                    _ => words.push((vec![fragment], word.chars().count())),
                }
            }
            space = trailing;
        }
    }

    let mut lines: Vec<Vec<Piece>> = Vec::new();
    let mut line: Vec<Piece> = Vec::new();
    let mut used = 0usize;
    for (parts, len) in words {
        let gap = usize::from(used > 0);
        if used > 0 && used + gap + len > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        for (n, mut piece) in parts.into_iter().enumerate() {
            if n == 0 && used > 0 {
                piece.text.insert(0, ' ');
            }
            used += piece.text.chars().count();
            // Runs of the same ink join, so the drawn line is as few spans
            // as it can be rather than one per word.
            match line.last_mut() {
                Some(last) if last.kind == piece.kind && last.href == piece.href => {
                    last.text.push_str(&piece.text);
                }
                _ => line.push(piece),
            }
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// What the markup claimed, in this theme.
///
/// Code takes the label colour rather than a background: a background on a
/// span inside prose fights the selection tint and loses on a terminal that
/// has neither.
fn ink(kind: Ink, t: &Theme, base: Color) -> Style {
    match kind {
        Ink::Plain => Style::default().fg(base),
        // `text`, not `heading`: strong prose is the body lifted out of
        // `muted`, and a heading is a different thing that may well be on
        // the line above it.
        Ink::Strong => Style::default().fg(t.text).bold(),
        Ink::Emphasis => Style::default().fg(base).italic(),
        Ink::Code => Style::default().fg(t.code),
        // Underlined as well as coloured, because a link that is only a
        // colour is not a link on a monochrome terminal.
        Ink::Link => Style::default().fg(t.link).underlined(),
    }
}

/// Break text on word boundaries. A word longer than the line is cut rather
/// than allowed to push the pane open.
fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let line_len = line.chars().count();
        if line_len == 0 {
            if word_len > width {
                let mut rest = word;
                while rest.chars().count() > width {
                    let head: String = rest.chars().take(width).collect();
                    out.push(head);
                    rest = &rest[rest
                        .char_indices()
                        .nth(width)
                        .map(|(i, _)| i)
                        .unwrap_or(rest.len())..];
                }
                line.push_str(rest);
            } else {
                line.push_str(word);
            }
        } else if line_len + 1 + word_len <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            out.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() || out.is_empty() {
        out.push(line);
    }
    out
}

// ── What needs you ───────────────────────────────────────────────────────────

/// Everything addressed to a person, and what answers it.
///
/// Every other lens answers *what is the shape of this*. This one answers
/// *what is waiting for me*, which is the question somebody sitting down
/// asks first and which nothing else here could be read for.
fn draw_needs(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .padding(Padding::horizontal(1))
        .title(Span::styled(" Needs you ", Style::default().fg(t.muted)));

    if app.questions.is_empty() {
        // The best screen this program can show, and until now there was no
        // way to see it. Said plainly and without decoration: an empty queue
        // is not an error state and should not look like one.
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Nothing needs you.",
                    Style::default().fg(t.muted),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "No proposals waiting, no claim gone cold, nothing finished and left open.",
                    Style::default().fg(t.faint),
                )),
            ])
            .alignment(Alignment::Center)
            .block(block),
            area,
        );
        return;
    }

    let inner = block.inner(area);
    let width = inner.width as usize;
    let cursor = app.question().min(app.questions.len() - 1);

    let rows: Vec<ListItem> = app
        .questions
        .iter()
        .map(|q| {
            let reference = app.schema.format_id(q.id);
            let colour = match &q.asking {
                // A proposal has somebody blocked on an answer; the rest are
                // degrees of untidy.
                crate::app::Asking::Proposal { .. } => t.accent,
                crate::app::Asking::ColdClaim { .. } => t.warn,
                crate::app::Asking::Finished => t.done,
                crate::app::Asking::NothingUnfinished => t.done,
                crate::app::Asking::Unowned { .. } => t.faint,
            };
            let question = q.asking.question(&reference);
            let answers = q.asking.answers();
            let room = width.saturating_sub(answers.chars().count() + 5);
            let shown = truncate(&question, room);
            let pad = room.saturating_sub(shown.chars().count());
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(shown, Style::default().fg(colour)),
                Span::raw(" ".repeat(pad)),
                // What the keys will do, said before they are pressed.
                Span::styled(format!("{answers}  "), Style::default().fg(t.faint)),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(cursor));
    f.render_stateful_widget(
        List::new(rows).block(block).highlight_style(t.selected()),
        area,
        &mut state,
    );
    for n in 0..app.questions.len() {
        app.hit(
            Rect {
                x: inner.x,
                y: inner.y + n as u16,
                width: inner.width,
                height: 1,
            },
            Hit::Question(n),
        );
    }
}

/// What changed across the project, most recent first.
///
/// The second question somebody sitting down asks, after *what needs me*.
/// The answer is in git — every item is a file and every change is a commit
/// — and cairn already proves it will read that, one item at a time.
fn draw_log(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            " What happened ",
            Style::default().fg(t.muted),
        ));

    let say = |f: &mut Frame, line: &str, colour: ratatui::style::Color| {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(colour),
            )))
            .alignment(Alignment::Center)
            .block(block.clone()),
            area,
        );
    };
    match &app.moments {
        None => return say(f, "Asking the repository…", t.faint),
        Some(Err(why)) => return say(f, why, t.warn),
        Some(Ok(m)) if m.is_empty() => {
            return say(f, "Nothing has changed here yet.", t.faint);
        }
        Some(Ok(_)) => {}
    }

    let inner = block.inner(area);
    let width = inner.width as usize;
    let me = app.me.clone();
    let rows: Vec<ListItem> = app
        .moments()
        .iter()
        .map(|m| {
            let reference = app.schema.format_id(m.id);
            // Telling *I did that* from *something else did that* is most of
            // what this lens is for.
            let mine = m.who.eq_ignore_ascii_case(&me);
            let who = truncate(&m.who, 14);
            let lead = 11 + 15 + reference.chars().count() + 3;
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", m.when), Style::default().fg(t.faint)),
                Span::styled(
                    format!("{who:<14} "),
                    Style::default().fg(if mine { t.person } else { t.secondary }),
                ),
                Span::styled(format!("{reference} "), Style::default().fg(t.faint)),
                Span::styled(
                    truncate(&m.what, width.saturating_sub(lead)),
                    Style::default().fg(t.muted),
                ),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(app.moment()));
    f.render_stateful_widget(
        List::new(rows).block(block).highlight_style(t.selected()),
        area,
        &mut state,
    );
    for n in 0..app.moments().len() {
        app.hit(
            Rect {
                x: inner.x,
                y: inner.y + n as u16,
                width: inner.width,
                height: 1,
            },
            Hit::Moment(n),
        );
    }
}

// ── The statistics ───────────────────────────────────────────────────────────

/// A backlog from a distance: how much of it there is, how much is moving, and
/// what is in the way.
fn draw_stats(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            format!(" {} ", app.schema.name),
            Style::default().fg(t.muted),
        ));
    let inner = block.inner(area);
    let stats = app.stats();

    // Built twice is not worth avoiding: the first pass only counts the
    // doors so the cursor can be clamped, and the second draws them knowing
    // which one it is on. Both are a few hundred short lines.
    let two_up = inner.width >= 84;
    let column_width = if two_up {
        inner.width as usize / 2 - 2
    } else {
        inner.width as usize
    };
    let build = |cursor: Option<usize>| -> (Sheet, Sheet) {
        let mut left = Sheet::default();
        stats_left(app, &stats, t, column_width, &mut left, cursor);
        let mut right = Sheet::default();
        if two_up {
            stats_right(app, &stats, t, column_width, &mut right, cursor);
        } else {
            left.blank();
            stats_right(app, &stats, t, column_width, &mut left, cursor);
        }
        (left, right)
    };

    let counted = build(None);
    let doors = counted.0.doors.len() + counted.1.doors.len();
    let figure = if doors == 0 {
        0
    } else {
        app.figure.min(doors - 1)
    };
    let (left, right) = build((doors > 0).then_some(figure));
    app.figure = figure;

    // One pane in two columns, so both move together and by the same amount.
    let tallest = left.lines.len().max(right.lines.len());
    let over = tallest.saturating_sub(inner.height as usize);
    // The pane follows the cursor, the way the list does, so a figure below
    // the fold can still be reached with the keys that reach the others.
    let on = left
        .doors
        .iter()
        .chain(right.doors.iter())
        .nth(app.figure)
        .map(|(line, ..)| *line as usize);
    if let Some(line) = on.filter(|_| doors > 0) {
        app.stats_scroll = scroll_to(
            app.stats_scroll as usize,
            line,
            tallest,
            inner.height as usize,
        ) as u16;
    }
    app.stats_scroll = app.stats_scroll.min(over as u16);
    let scroll = (app.stats_scroll, 0);

    let hint = app.keymap.scroll_hint(Command::Up, Command::Down);
    let block = match (over > 0, hint) {
        (true, Some(keys)) => block.title_bottom(Span::styled(
            format!(" {keys} move · ↵ opens "),
            Style::default().fg(t.faint),
        )),
        _ => block,
    };

    f.render_widget(block, area);
    let columns = if right.lines.is_empty() {
        vec![inner]
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner)
            .to_vec()
    };
    f.render_widget(Paragraph::new(left.lines).scroll(scroll), columns[0]);
    if let Some(second) = columns.get(1) {
        f.render_widget(Paragraph::new(right.lines).scroll(scroll), *second);
    }

    // Where each figure landed, so the pointer finds the same thing the
    // cursor does. Registered after drawing, against the same geometry.
    let mut n = 0;
    app.doors.clear();
    for (sheet, cell) in [
        (&left.doors, columns[0]),
        (&right.doors, *columns.last().unwrap()),
    ] {
        if std::ptr::eq(sheet, &right.doors) && columns.len() < 2 {
            break;
        }
        for (line, x, w, door) in sheet.iter() {
            let y = cell.y as i32 + *line as i32 - app.stats_scroll as i32;
            if y >= cell.y as i32 && y < (cell.y + cell.height) as i32 {
                app.hit(
                    Rect {
                        x: cell.x + x,
                        y: y as u16,
                        width: (*w).min(cell.width.saturating_sub(*x)),
                        height: 1,
                    },
                    Hit::Figure(n),
                );
            }
            app.doors.push(door.clone());
            n += 1;
        }
    }
}

/// A stats column, and where the figures on it lead.
///
/// Built together because they have to agree: a door drawn in one place and
/// registered in another is a target that moves when the layout does.
#[derive(Default)]
struct Sheet {
    lines: Vec<Line<'static>>,
    /// `(line, x, width, door)`, x relative to the column being built.
    doors: Vec<(u16, u16, u16, Door)>,
}

impl Sheet {
    fn push(&mut self, line: Line<'static>) {
        self.lines.push(line);
    }

    fn blank(&mut self) {
        self.lines.push(Line::from(""));
    }

    /// Record that the span just pushed, at `x` and `width` columns wide,
    /// leads somewhere.
    fn door(&mut self, x: u16, width: u16, door: Door) {
        let line = self.lines.len().saturating_sub(1) as u16;
        self.doors.push((line, x, width, door));
    }

    /// How a figure is drawn depends on whether the cursor is on it, so the
    /// count of doors so far is also the index of the next one.
    fn next_is_selected(&self, cursor: Option<usize>) -> bool {
        cursor == Some(self.doors.len())
    }
}

fn stats_left(
    app: &App,
    s: &crate::app::Stats,
    t: &Theme,
    width: usize,
    sheet: &mut Sheet,
    cursor: Option<usize>,
) {
    let lines = sheet;
    let closed = s.done + s.dropped;
    let percent = (closed * 100).checked_div(s.total).unwrap_or(0) as u32;

    lines.push(section("Where it stands", t, width));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            progress_bar(percent, 12),
            Style::default().fg(if percent == 100 { t.done } else { t.accent }),
        ),
        Span::styled(
            format!("  {percent}%  ·  {} of {} closed", closed, s.total),
            Style::default().fg(t.muted),
        ),
    ]));
    lines.blank();
    tally(
        lines,
        cursor,
        &[
            ("ready", s.ready, t.ready, "ready=true"),
            ("in flight", s.active, t.active, "category=active"),
            ("blocked", s.blocked, t.blocked, "blocked=true"),
        ],
        t,
    );
    tally(
        lines,
        cursor,
        &[
            ("open", s.open, t.open, "category=open"),
            ("claimed", s.claimed, t.person, "assignee!="),
            ("dropped", s.dropped, t.dropped, "category=dropped"),
        ],
        t,
    );

    if s.criteria.1 > 0 {
        lines.blank();
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} of {} ", s.criteria.0, s.criteria.1),
                Style::default().fg(t.text),
            ),
            Span::styled(
                "acceptance criteria ticked on open work",
                Style::default().fg(t.faint),
            ),
        ]));
    }

    lines.blank();
    lines.push(section("Closed", t, width));
    let mut spans = vec![Span::raw("  ")];
    for (window, count) in s.closed_recently {
        spans.push(Span::styled(
            count.to_string(),
            Style::default()
                .fg(if count > 0 { t.done } else { t.faint })
                .bold(),
        ));
        spans.push(Span::styled(
            format!(" in {window}d   "),
            Style::default().fg(t.faint),
        ));
    }
    lines.push(Line::from(spans));

    if let Some((id, title, days)) = &s.oldest {
        lines.blank();
        lines.push(section("Waiting longest", t, width));
        let here = lines.next_is_selected(cursor);
        let reference = app.schema.format_id(*id);
        let shown = truncate(title, width.saturating_sub(16));
        let span = (reference.chars().count() + 1 + shown.chars().count()) as u16;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(reference, door_style(t, here).fg(t.faint)),
            Span::raw(" "),
            Span::styled(shown, door_style(t, here).fg(t.muted)),
        ]));
        lines.door(2, span, Door::Item(*id));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{days} days open"), Style::default().fg(t.warn)),
        ]));
    }

    if let Some((id, title, count)) = &s.blocking {
        lines.blank();
        lines.push(section("In the way", t, width));
        let here = lines.next_is_selected(cursor);
        let reference = app.schema.format_id(*id);
        let shown = truncate(title, width.saturating_sub(16));
        let span = (reference.chars().count() + 1 + shown.chars().count()) as u16;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(reference, door_style(t, here).fg(t.faint)),
            Span::raw(" "),
            Span::styled(shown, door_style(t, here).fg(t.muted)),
        ]));
        lines.door(2, span, Door::Item(*id));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!(
                    "{count} {} waiting on it",
                    if *count == 1 { "item is" } else { "items are" }
                ),
                Style::default().fg(t.blocked),
            ),
        ]));
    }
}

fn stats_right(
    app: &App,
    s: &crate::app::Stats,
    t: &Theme,
    width: usize,
    sheet: &mut Sheet,
    cursor: Option<usize>,
) {
    let lines = sheet;

    if !s.milestones.is_empty() {
        lines.push(section("Milestones", t, width));
        let key_width = s
            .milestones
            .iter()
            .map(|(k, ..)| k.chars().count())
            .max()
            .unwrap_or(4)
            .min(10);
        for (key, title, percent, left, due) in &s.milestones {
            let here = lines.next_is_selected(cursor);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:<key_width$} ", truncate(key, key_width)),
                    door_style(t, here).fg(t.milestone).bold(),
                ),
                Span::styled(
                    progress_bar(*percent, 8),
                    Style::default().fg(if *percent == 100 { t.done } else { t.accent }),
                ),
                Span::styled(format!(" {percent:>3}%  "), Style::default().fg(t.muted)),
                Span::styled(
                    if *left == 0 {
                        "done".to_string()
                    } else {
                        format!("{left} left")
                    },
                    Style::default().fg(if *left == 0 { t.done } else { t.faint }),
                ),
            ]));
            lines.door(
                2,
                key_width as u16,
                Door::Filter(format!("milestone={key}")),
            );
            let note = match due {
                Some(due) => format!("{title} · due {due}"),
                None => title.clone(),
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(" ".repeat(key_width + 1)),
                Span::styled(
                    truncate(&note, width.saturating_sub(key_width + 4)),
                    Style::default().fg(t.faint),
                ),
            ]));
        }
        lines.blank();
    }

    let bars = |lines: &mut Sheet,
                rows: &[(String, usize)],
                colour: &dyn Fn(usize) -> ratatui::style::Color| {
        let most = rows.iter().map(|(_, n)| *n).max().unwrap_or(0).max(1);
        let label_width = rows
            .iter()
            .map(|(k, _)| k.chars().count())
            .max()
            .unwrap_or(4)
            .min(12);
        let bar_width = width.saturating_sub(label_width + 8).clamp(4, 14);
        for (i, (label, count)) in rows.iter().enumerate() {
            let filled = (count * bar_width).div_ceil(most).min(bar_width);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:<label_width$} ", truncate(label, label_width)),
                    Style::default().fg(t.muted),
                ),
                Span::styled("▇".repeat(filled), Style::default().fg(colour(i))),
                Span::styled(
                    "▁".repeat(bar_width - filled),
                    Style::default().fg(t.border),
                ),
                Span::styled(format!(" {count}"), Style::default().fg(t.faint)),
            ]));
        }
    };

    if !s.by_type.is_empty() {
        lines.push(section("By type", t, width));
        let types: Vec<(String, usize)> = s.by_type.clone();
        let schema = &app.schema;
        let colour = |i: usize| {
            types
                .get(i)
                .and_then(|(name, _)| schema.item_type(name))
                .map(|k| t.item_type(Some(k)))
                .unwrap_or(t.secondary)
        };
        bars(lines, &s.by_type, &colour);
        lines.blank();
    }

    for (field, values) in &s.by_field {
        // A short scale shows its empty steps — "no p3 open" is worth knowing.
        // A long one drops them, because fifteen rows of zero is not a
        // distribution, it is a list of the field's values.
        let rows: Vec<(String, usize)> = if values.len() > 5 {
            values.iter().filter(|(_, n)| *n > 0).cloned().collect()
        } else {
            values.clone()
        };
        if rows.iter().all(|(_, n)| *n == 0) {
            continue;
        }
        lines.push(section(&title_case(field), t, width));
        let len = rows.len();
        let colour = |i: usize| t.rank(i, len);
        bars(lines, &rows, &colour);
        lines.blank();
    }
}

/// A row of `count label` pairs, aligned so the numbers line up, each one a
/// door to the set it counted.
fn tally(
    sheet: &mut Sheet,
    cursor: Option<usize>,
    cells: &[(&str, usize, ratatui::style::Color, &str)],
    t: &Theme,
) {
    const CELL: u16 = 14;
    let mut spans = vec![Span::raw("  ")];
    let mut doors = Vec::new();
    for (n, (label, count, colour, filter)) in cells.iter().enumerate() {
        let here = cursor == Some(sheet.doors.len() + n);
        spans.push(Span::styled(
            format!("{count:>3} "),
            door_style(t, here).fg(*colour).bold(),
        ));
        spans.push(Span::styled(
            format!("{label:<10}"),
            door_style(t, here).fg(t.faint),
        ));
        doors.push((
            2 + n as u16 * CELL,
            CELL,
            Door::Filter((*filter).to_string()),
        ));
    }
    sheet.push(Line::from(spans));
    for (x, width, door) in doors {
        sheet.door(x, width, door);
    }
}

/// How a figure is drawn when the cursor is on it. The same lift the list
/// uses for its own selection, so "here" looks the same wherever you are.
fn door_style(t: &Theme, selected: bool) -> Style {
    if selected {
        t.selected()
    } else {
        Style::default()
    }
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ── Overlays ─────────────────────────────────────────────────────────────────

/// What an overlay says along its bottom edge. The scroll keys go there only
/// when they lead somewhere, the way a pane holding more than it shows is the
/// only one that puts them on its own border.
fn overlay_edge(scrollable: bool) -> &'static str {
    if scrollable {
        " ↑↓ scroll · any other key closes "
    } else {
        " any other key closes "
    }
}

/// What an item says about itself before its prose starts: where it stands,
/// what kind of thing it is, who has it. All of it was in the detail pane the
/// old popover covered, which made the fullest view of an item the one that
/// said least about it.
fn reader_masthead(app: &App, item: &Item, t: &Theme, width: usize) -> Vec<Line<'static>> {
    let schema = &app.schema;
    let mut lines = Vec::new();

    for (n, part) in wrap(&item.title, width).into_iter().enumerate() {
        lines.push(Line::from(vec![
            if n == 0 {
                Span::styled(
                    format!("{} ", glyph(item)),
                    Style::default().fg(state_color(item, t, schema)),
                )
            } else {
                Span::raw("  ")
            },
            Span::styled(part, Style::default().fg(t.heading).bold()),
        ]));
    }
    lines.push(Line::from(""));

    // Where it stands, then why it is ranked where it is. Two lines rather
    // than one because six facts on one line is a line that truncates.
    let mut standing = vec![Span::styled(
        schema
            .status(&item.status)
            .map(|s| s.display().to_string())
            .unwrap_or_else(|| item.status.clone()),
        Style::default().fg(t.status(schema.status(&item.status))),
    )];
    let dot = |spans: &mut Vec<Span<'static>>| {
        spans.push(Span::styled(" · ", Style::default().fg(t.faint)));
    };
    dot(&mut standing);
    standing.push(Span::styled(
        item.kind.clone(),
        Style::default().fg(t.item_type(schema.item_type(&item.kind))),
    ));
    if let Some(milestone) = item.milestone() {
        dot(&mut standing);
        standing.push(Span::styled(
            milestone.to_string(),
            Style::default().fg(t.milestone),
        ));
    }
    lines.push(Line::from(standing));

    // The rank the project ranks by, then the rest of the columns it chose.
    // Nothing here is named in this file: a project that ranks by `severity`
    // and files by `component` gets its own words, the way every other surface
    // in harrow does.
    let mut ranking: Vec<Span<'static>> = Vec::new();
    if let Some(rank) = rank_field(schema)
        && let Some(value) = item.field(&rank.name).filter(|v| !v.is_empty())
    {
        ranking.push(Span::styled(value.display(), rank_style(item, schema, t)));
    }
    for field in schema.fields.iter().filter(|f| f.column) {
        if rank_field(schema).is_some_and(|r| r.name == field.name) || field.name == "milestone" {
            continue;
        }
        if let Some(value) = item.field(&field.name).filter(|v| !v.is_empty()) {
            if !ranking.is_empty() {
                dot(&mut ranking);
            }
            ranking.push(Span::styled(value.display(), Style::default().fg(t.muted)));
        }
    }
    if let Some(who) = &item.assignee {
        if !ranking.is_empty() {
            dot(&mut ranking);
        }
        let stale = app.claim_is_stale(item);
        let (mark, style) = actor_style(app, who, stale, t);
        ranking.push(Span::styled(format!("{mark}{who}"), style));
        if let Some(days) = app.claimed_days(item).filter(|_| stale) {
            ranking.push(Span::styled(
                format!(" · held {days} days"),
                Style::default().fg(t.warn),
            ));
        }
    }
    if !ranking.is_empty() {
        lines.push(Line::from(ranking));
    }
    lines
}

/// The whole item, read beside the backlog rather than over it.
///
/// A panel, not a popover: the lens keeps its own column while there is room
/// for one, the selection goes on moving, and what is shown here follows it.
fn draw_reader(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(item) = app.selected_item() else {
        return;
    };
    let id = item.id;
    let focused = app.focus == crate::app::Focus::Reader;
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { t.border_focus } else { t.border }))
        .padding(Padding::horizontal(2))
        .title(Span::styled(
            format!(" {} ", app.schema.format_id(id)),
            Style::default().fg(if focused { t.text } else { t.muted }),
        ));
    let inner = block.inner(area);
    // Held to a measure. Everything past it is margin, so the panel can be as
    // wide as the terminal allows without the prose becoming unreadable.
    let measure = inner.width.min(READER_MEASURE) as usize;

    let mut lines = vec![Line::from("")];
    lines.extend(reader_masthead(app, item, t, measure));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─".repeat(measure),
        Style::default().fg(t.border),
    )));
    lines.push(Line::from(""));
    lines.extend(body_lines(&item.body, t, measure));

    // Clamped here because here is where the height of the content is known.
    let over = lines.len().saturating_sub(inner.height as usize);
    app.reader.clamp(over as u16);
    let scroll = app.reader.at(id);

    // What esc does depends on where the keys are: it hands them back before it
    // closes anything, so a panel that said "esc closes" while focused would be
    // telling you the second half of the answer.
    let hint = if focused {
        let scroll = if over > 0 { "↑↓ scroll · " } else { "" };
        format!(" {scroll}esc closes ")
    } else {
        let keys = app
            .keymap
            .scroll_hint(Command::DetailUp, Command::DetailDown);
        match (over > 0, keys) {
            (true, Some(keys)) => format!(" ↵ read · {keys} scroll · esc closes "),
            _ => " ↵ read · esc closes ".to_string(),
        }
    };
    let block = block.title_bottom(Span::styled(hint, Style::default().fg(t.faint)));

    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).scroll((scroll, 0)).block(block), area);
    app.hit(area, Hit::Reader);
}

/// How an item got the way it is.
///
/// The reason an item file is worth keeping in the repository rather than in a
/// database: its history is the answer to "when did this become p0, and who
/// decided that?", and it is already there.
fn draw_history(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(history) = &app.history else { return };
    let width = 84u16.min(area.width.saturating_sub(4));
    let room = width.saturating_sub(6) as usize;

    let mut lines = vec![Line::from("")];
    if let Some(why) = &history.unavailable {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(truncate(why, room), Style::default().fg(t.warn)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "An item's history is the repository's. Without one there is none.",
                Style::default().fg(t.faint),
            ),
        ]));
    } else if history.lines.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Nothing recorded yet — this item has not been committed.",
                Style::default().fg(t.faint),
            ),
        ]));
    }

    for line in &history.lines {
        // cairn prints `date  who  what`. The what is the part being read.
        let mut parts = line.splitn(3, "  ").map(str::trim);
        match (parts.next(), parts.next(), parts.next()) {
            (Some(date), Some(who), Some(what)) => lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(date.to_string(), Style::default().fg(t.faint)),
                Span::raw("  "),
                Span::styled(
                    format!("{:<18}", truncate(who, 18)),
                    Style::default().fg(t.person),
                ),
                Span::raw("  "),
                Span::styled(
                    truncate(what, room.saturating_sub(32)),
                    Style::default().fg(t.text),
                ),
            ])),
            _ => lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(truncate(line, room), Style::default().fg(t.muted)),
            ])),
        }
    }

    // Sized to what there is to say, which for a project with no repository is
    // a sentence and not a list.
    let height = ((lines.len() + 2) as u16).min(area.height.saturating_sub(4));
    let popup = centered(area, width, height);
    let over = lines
        .len()
        .saturating_sub(height.saturating_sub(2) as usize) as u16;
    let title = format!(" {} · history ", app.schema.format_id(history.id));

    // Clamped once the borrow on the history is done with, for the same reason
    // the reader is: here is where the height of the content is known. A
    // history short enough to fit does not scroll at all.
    let scroll = match app.history.as_mut() {
        Some(history) => {
            history.scroll = history.scroll.min(over);
            history.scroll
        }
        None => 0,
    };

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).scroll((scroll, 0)).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus))
                .title(Span::styled(title, Style::default().fg(t.muted)))
                .title_bottom(Span::styled(
                    overlay_edge(over > 0),
                    Style::default().fg(t.faint),
                )),
        ),
        popup,
    );
    app.hit(popup, Hit::Overlay);
}

/// Every command, by name, filtered as you type.
///
/// The key sits on the right of every row, so this teaches the keymap rather
/// than replacing it — and a command with no key shows a dash, which is the
/// honest way to say *this one lives here now*.
fn draw_palette(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(palette) = &app.palette else { return };
    let width = 64u16.min(area.width.saturating_sub(4));
    let rows = palette.matches.len().min(10) as u16;
    let height = (rows + 2).min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border_focus))
        .style(Style::default().bg(t.overlay))
        .padding(Padding::horizontal(1))
        .title(Line::from(vec![
            Span::styled(" : ", Style::default().fg(t.accent).bold()),
            Span::styled(palette.typed.clone(), Style::default().fg(t.text)),
            Span::styled("▏", Style::default().fg(t.accent)),
        ]))
        .title_bottom(Span::styled(
            if palette.matches.is_empty() {
                " nothing by that name ".to_string()
            } else {
                " ↵ run · the key on the right does it without this ".to_string()
            },
            Style::default().fg(t.faint),
        ));

    let inner = block.inner(popup);
    let key_col = 4usize;
    let room = (inner.width as usize).saturating_sub(key_col + 2);
    // Two columns of text: the stable name a config file would bind, and the
    // sentence the help overlay shows. Same source as the help overlay,
    // because a palette that could go stale would be worse than none.
    let name_col = room.min(26);

    // The window follows the cursor, so a match far down the list is reachable.
    let shown = rows as usize;
    let first = palette.selected.saturating_sub(shown.saturating_sub(1));
    let lines: Vec<Line> = palette
        .matches
        .iter()
        .enumerate()
        .skip(first)
        .take(shown)
        .map(|(n, (command, _))| {
            let here = n == palette.selected;
            let name = format!("{:<name_col$}", truncate(command.name(), name_col));
            let said = truncate(command.describe(), room.saturating_sub(name_col + 1));
            Line::from(vec![
                Span::styled(
                    if here { "▸ " } else { "  " },
                    Style::default().fg(t.accent),
                ),
                Span::styled(
                    name,
                    if here {
                        Style::default().fg(t.heading).bold()
                    } else {
                        Style::default().fg(t.text)
                    },
                ),
                Span::styled(format!(" {said}"), Style::default().fg(t.muted)),
            ])
            .style(if here {
                Style::default().bg(t.selection)
            } else {
                Style::default()
            })
        })
        .collect();

    f.render_widget(Clear, popup);
    f.render_widget(Paragraph::new(lines).block(block), popup);

    // The keys, right-aligned, drawn over the rows they belong to.
    for (row, (_, key)) in palette
        .matches
        .iter()
        .enumerate()
        .skip(first)
        .take(shown)
        .enumerate()
        .map(|(row, (_, m))| (row, m))
    {
        let said = key.clone().unwrap_or_else(|| "—".to_string());
        let cell = Rect {
            x: inner.x + inner.width.saturating_sub(said.chars().count() as u16),
            y: inner.y + row as u16,
            width: said.chars().count() as u16,
            height: 1,
        };
        if cell.y < inner.y + inner.height {
            f.render_widget(
                Line::from(Span::styled(
                    said,
                    Style::default().fg(if key.is_some() { t.label } else { t.faint }),
                )),
                cell,
            );
        }
    }

    // Clickable, like the picker is. Collected first, because registering a
    // hit borrows the app the palette was read out of.
    let rows: Vec<Command> = palette
        .matches
        .iter()
        .skip(first)
        .take(shown)
        .map(|(command, _)| *command)
        .collect();
    for (row, command) in rows.into_iter().enumerate() {
        app.hit(
            Rect {
                x: inner.x,
                y: inner.y + row as u16,
                width: inner.width,
                height: 1,
            },
            Hit::Run(command),
        );
    }
}

fn draw_picker(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(picker) = &app.picker else { return };
    let width = 54u16.min(area.width.saturating_sub(4));
    let height = ((picker.options.len() + 2) as u16).min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);

    // Numbered rather than lettered: three of a project's statuses can begin
    // with the same letter, and a shortcut that is ambiguous is not a shortcut.
    // The note is right-aligned against the edge, which is what makes a
    // column of categories read as a column. It only gives ground when the
    // label would starve: a project's own description of a saved view is a
    // sentence, and it used to truncate every name to an ellipsis, leaving a
    // list of things you could not tell apart.
    const LABEL_MIN: usize = 18;
    let room = (width as usize).saturating_sub(7);
    let note_col = picker
        .options
        .iter()
        .map(|(_, _, n)| n.chars().count())
        .max()
        .unwrap_or(0)
        .min(room.saturating_sub(LABEL_MIN.min(room)));
    let label_col = room.saturating_sub(note_col);

    let items: Vec<ListItem> = picker
        .options
        .iter()
        .enumerate()
        .map(|(i, (_, label, note))| {
            let key = if i < 9 { (b'1' + i as u8) as char } else { ' ' };
            let label = truncate(label, label_col);
            let pad = label_col.saturating_sub(label.chars().count());
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {key} "), Style::default().fg(t.accent).bold()),
                Span::styled(label, Style::default().fg(t.text)),
                Span::raw(" ".repeat(pad)),
                Span::styled(
                    format!("{} ", truncate(note, note_col)),
                    Style::default().fg(t.faint),
                ),
            ]))
        })
        .collect();

    let count = picker.options.len();
    let mut state = ListState::default().with_selected(Some(picker.selected));
    f.render_widget(Clear, popup);
    f.render_stateful_widget(
        List::new(items).highlight_style(t.selected()).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus))
                .title(Span::styled(
                    format!(" {} ", picker.title),
                    Style::default()
                        .fg(if picker.propose { t.warn } else { t.accent })
                        .bold(),
                ))
                // What the project says a program may do with this field, and
                // which of the two things Enter is about to do. Both belong
                // here rather than in a toast afterwards: this is where the
                // choice is made.
                .title_top(
                    Line::from(match picker.permission.note() {
                        Some(note) => {
                            Span::styled(format!(" project: {note} "), Style::default().fg(t.warn))
                        }
                        None => Span::raw(""),
                    })
                    .right_aligned(),
                )
                .title_bottom(Span::styled(
                    if picker.views {
                        // Nothing is being written, so nothing can be
                        // proposed: this changes what you are looking at.
                        " ↵ look · esc cancel "
                    } else if picker.propose {
                        " ↵ propose · ctrl-p to set instead · esc cancel "
                    } else {
                        " ↵ set · ctrl-p to propose · esc cancel "
                    },
                    Style::default().fg(t.faint),
                )),
        ),
        popup,
        &mut state,
    );
    for n in 0..count {
        app.hit(
            Rect {
                x: popup.x + 1,
                y: popup.y + 1 + n as u16,
                width: popup.width.saturating_sub(2),
                height: 1,
            },
            Hit::Option(n),
        );
    }
}

fn draw_help(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    // Generated from the active bindings. A help screen that lists the defaults
    // while the user runs something else is worse than no help screen.
    let rows = app.keymap.help_rows();

    // Two columns where there is room, because the list is now long enough to
    // run off a short terminal and a help screen you have to scroll is one
    // nobody finishes reading.
    let columns = if area.width >= 100 && rows.len() > 14 {
        2
    } else {
        1
    };
    let per_column = rows.len().div_ceil(columns);
    let column_width = 46usize;
    let width = ((column_width * columns + 4) as u16).min(area.width.saturating_sub(4));
    let height = ((per_column + 4) as u16).min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);

    let key_col = rows
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(8)
        .clamp(8, 14);
    let room = column_width.saturating_sub(key_col + 3);

    let cell = |(keys, description): &(String, &'static str)| {
        if keys.is_empty() {
            return vec![Span::raw(" ".repeat(column_width))];
        }
        let description = truncate(description, room);
        let pad = column_width.saturating_sub(key_col + 3 + description.chars().count());
        vec![
            Span::raw("  "),
            Span::styled(
                format!("{keys:<key_col$}"),
                Style::default().fg(t.accent).bold(),
            ),
            Span::raw(" "),
            Span::styled(description, Style::default().fg(t.muted)),
            Span::raw(" ".repeat(pad)),
        ]
    };

    let mut lines = vec![Line::from("")];
    for n in 0..per_column {
        let mut spans = cell(&rows[n]);
        if columns == 2
            && let Some(right) = rows.get(n + per_column)
        {
            spans.extend(cell(right));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus))
                .title(Span::styled(" Keys ", Style::default().fg(t.accent).bold())),
        ),
        popup,
    );
}

fn draw_diagnostics(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let width = 92u16.min(area.width.saturating_sub(4));
    let height = 26u16.min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);
    let (warns, errors) = diag::counts();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{warns} warnings · {errors} errors"),
                Style::default().fg(if errors > 0 { t.error } else { t.muted }),
            ),
        ]),
    ];
    if let Some(why) = &app.readonly {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(why.at_length(), Style::default().fg(t.warn)),
        ]));
    }
    if let Some(fail) = &app.failure {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("last read failed: {}", fail.detail),
                Style::default().fg(t.error),
            ),
        ]));
    }
    if !app.watcher_alive {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "the loader thread is not running — restart harrow",
                Style::default().fg(t.error),
            ),
        ]));
    }

    // What the backlog itself says: broken files, missing categories, ids used
    // twice. The same ground `cairn check` covers, without leaving the screen.
    if !app.warnings.is_empty() {
        lines.push(Line::from(""));
        lines.push(section("This backlog", t, width as usize - 2));
        for warning in app.warnings.iter().take(6) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    truncate(warning, width.saturating_sub(6) as usize),
                    Style::default().fg(t.warn),
                ),
            ]));
        }
    }
    // What cairn says, under its own heading. A finding of cairn's read as a
    // bug of harrow's is the confusion this separation exists to prevent.
    match &app.checked {
        None => {
            lines.push(Line::from(""));
            // From the bindings in force rather than from ours.
            let key = app
                .keymap
                .keys_for(Command::Check)
                .first()
                .cloned()
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(key, Style::default().fg(t.accent).bold()),
                Span::styled(
                    " runs the project's own `cairn check`",
                    Style::default().fg(t.faint),
                ),
            ]));
        }
        Some(Err(why)) => {
            lines.push(Line::from(""));
            lines.push(section("cairn check", t, width as usize - 2));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    truncate(why, width.saturating_sub(6) as usize),
                    Style::default().fg(t.error),
                ),
            ]));
        }
        Some(Ok(said)) => {
            lines.push(Line::from(""));
            lines.push(section("cairn check", t, width as usize - 2));
            for line in said.iter().take(8) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        truncate(line, width.saturating_sub(6) as usize),
                        Style::default().fg(if line.starts_with("ok") { t.ok } else { t.warn }),
                    ),
                ]));
            }
        }
    }
    lines.push(Line::from(""));

    let room = height.saturating_sub(lines.len() as u16 + 3) as usize;
    let events = diag::recent(room);
    if events.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("Nothing logged.", Style::default().fg(t.faint)),
        ]));
    }
    for e in events {
        let color = match e.level {
            diag::Level::Error => t.error,
            diag::Level::Warn => t.warn,
            diag::Level::Info => t.faint,
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<5} ", e.level), Style::default().fg(color)),
            Span::styled(format!("{:<9} ", e.scope), Style::default().fg(t.faint)),
            Span::styled(
                truncate(&e.message, width.saturating_sub(22) as usize),
                Style::default().fg(t.muted),
            ),
        ]));
    }

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.warn))
                .title(Span::styled(
                    " Diagnostics ",
                    Style::default().fg(t.warn).bold(),
                )),
        ),
        popup,
    );
}

fn draw_confirm(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(c) = &app.confirm else { return };
    let popup = centered(area, 60.min(area.width.saturating_sub(4)), 7);
    // The two answers, where they are drawn on the last line of the box.
    let answers = [
        (
            Rect {
                x: popup.x + 2,
                y: popup.y + 5,
                width: 10,
                height: 1,
            },
            true,
        ),
        (
            Rect {
                x: popup.x + 14,
                y: popup.y + 5,
                width: 14,
                height: 1,
            },
            false,
        ),
    ];
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(c.prompt.clone(), Style::default().fg(t.heading).bold()),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                truncate(&c.detail, popup.width.saturating_sub(4) as usize),
                Style::default().fg(t.faint),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("y", Style::default().fg(t.done).bold()),
            Span::styled(" confirm    ", Style::default().fg(t.muted)),
            Span::styled("n / esc", Style::default().fg(t.accent).bold()),
            Span::styled(" cancel", Style::default().fg(t.muted)),
        ]),
    ];
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus)),
        ),
        popup,
    );
    for (rect, yes) in answers {
        app.hit(rect, Hit::Answer(yes));
    }
}

// ── The footer ───────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    use crate::app::{Editing, Focus};
    if let Some(editing) = &app.editing {
        let (label, hint) = match editing {
            Editing::Filter => (" filter ", "   enter to keep · esc to clear"),
            Editing::Sort => (" sort   ", "   - reverses · enter to keep · esc to clear"),
            Editing::NewItem => (" title  ", "   enter to create · esc to cancel"),
            Editing::Note => (" note   ", "   enter to append · esc to cancel"),
            Editing::Reason => (" why?   ", "   enter to hand it back · esc to keep it"),
            Editing::Why => (
                " why?   ",
                "   enter to propose it · a proposal with no reason is a preference",
            ),
        };
        let mut spans = vec![
            Span::styled(label, Style::default().bg(t.accent).fg(t.background).bold()),
            Span::raw(" "),
            Span::styled(app.input.clone(), Style::default().fg(t.text)),
            Span::styled("▏", Style::default().fg(t.accent)),
        ];
        if editing == &Editing::Filter && !app.query.unknown.is_empty() {
            spans.push(Span::styled(
                format!("   no such field: {}", app.query.unknown.join(", ")),
                Style::default().fg(t.warn),
            ));
        } else if editing == &Editing::Sort && !app.unknown_sort().is_empty() {
            spans.push(Span::styled(
                format!("   no such field: {}", app.unknown_sort().join(", ")),
                Style::default().fg(t.warn),
            ));
        } else if editing == &Editing::Sort {
            // What this project can be ordered by, read off its schema. The
            // completion is the documentation: nothing here is a list harrow
            // keeps of what a backlog is allowed to have.
            let typed = app.input.rsplit(',').next().unwrap_or("").trim();
            let typed = typed.strip_prefix('-').unwrap_or(typed).to_lowercase();
            let offered: Vec<String> = app
                .sort_fields()
                .into_iter()
                .filter(|f| typed.is_empty() || f.starts_with(&typed))
                .collect();
            let room = (area.width as usize).saturating_sub(app.input.chars().count() + 12);
            let mut said = String::new();
            for field in offered {
                if said.chars().count() + field.chars().count() + 2 > room {
                    break;
                }
                if !said.is_empty() {
                    said.push(' ');
                }
                said.push_str(&field);
            }
            spans.push(Span::styled(
                format!("   {said}"),
                Style::default().fg(t.faint),
            ));
        } else {
            spans.push(Span::styled(hint, Style::default().fg(t.faint)));
        }
        f.render_widget(Line::from(spans), area);
        return;
    }

    // What the keys do right now, which is not what the ordinary hints say.
    if app.focus == Focus::Reader && app.toast.is_none() {
        f.render_widget(
            Line::from(vec![
                Span::styled(
                    " reading ",
                    Style::default().bg(t.accent).fg(t.background).bold(),
                ),
                Span::styled("  ↑↓", Style::default().fg(t.accent).bold()),
                Span::styled(" scroll  ", Style::default().fg(t.faint)),
                Span::styled("esc", Style::default().fg(t.accent).bold()),
                Span::styled(" closes  ", Style::default().fg(t.faint)),
                Span::styled("e", Style::default().fg(t.accent).bold()),
                Span::styled(" edit", Style::default().fg(t.faint)),
            ]),
            area,
        );
        return;
    }

    if let Some((msg, kind, _)) = &app.toast {
        let color = match kind {
            ToastKind::Good => t.ok,
            ToastKind::Bad => t.error,
            ToastKind::Info => t.accent,
        };
        f.render_widget(
            Line::from(vec![
                Span::styled(" ● ", Style::default().fg(color)),
                Span::styled(
                    truncate(msg, area.width.saturating_sub(4) as usize),
                    Style::default().fg(color),
                ),
            ]),
            area,
        );
        return;
    }

    if let Some(fail) = &app.failure {
        f.render_widget(
            Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(t.error).bold()),
                Span::styled(format!("{}  ", fail.detail), Style::default().fg(t.error)),
                Span::styled("D", Style::default().fg(t.accent).bold()),
                Span::styled(" diagnostics", Style::default().fg(t.faint)),
            ]),
            area,
        );
        return;
    }

    // Hints, as many as fit. They are in the order somebody learning the tool
    // needs them, so a narrow pane keeps the useful end.
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1usize;
    let mut buttons: Vec<(Rect, crate::keys::Command)> = Vec::new();
    for (key, label, command) in app.keymap.footer_hints() {
        let cost = key.chars().count() + label.chars().count() + 3;
        if used + cost > area.width as usize {
            break;
        }
        // A hint is a button. Somebody who reaches for the mouse should not
        // have to learn the key it is advertising first.
        buttons.push((
            Rect {
                x: area.x + used as u16,
                y: area.y,
                width: cost as u16,
                height: 1,
            },
            command,
        ));
        used += cost;
        spans.push(Span::styled(key, Style::default().fg(t.accent).bold()));
        spans.push(Span::styled(
            format!(" {label}  "),
            Style::default().fg(t.faint),
        ));
    }
    // The footer already said read-only; it now says which read-only, because
    // "cairn is missing" and "this project is newer than I am" call for
    // different things of the reader.
    if let Some(why) = app.readonly.as_ref().map(ReadOnly::briefly)
        && used + why.chars().count() + 2 <= area.width as usize
    {
        spans.push(Span::styled(format!(" {why}"), Style::default().fg(t.warn)));
    }
    for (rect, command) in buttons {
        app.hit(rect, Hit::Run(command));
    }
    f.render_widget(Line::from(spans), area);
}

// ── Helpers, and the seam the tests use ──────────────────────────────────────

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let keep = width.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

pub fn ago(d: Duration) -> String {
    let s = d.as_secs();
    match s {
        0 => "just now".into(),
        1..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        3600..=86399 => format!("{}h {}m", s / 3600, (s % 3600) / 60),
        _ => format!("{}d", s / 86400),
    }
}

/// Render the whole screen into plain text.
///
/// This is the seam the snapshot tests and `--screenshot` both use: it needs no
/// terminal, so a rendering regression is caught in CI rather than by eye.
pub fn render_to_string(app: &mut App, width: u16, height: u16, tick: usize) -> String {
    let buf = render_frame(app, width, height, tick);
    (0..buf.area.height)
        .map(|y| {
            let line: String = (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect();
            line.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the screen as a map of foreground colours — one character per cell,
/// with a legend. Plain text says where things are; this says how they read.
pub fn render_styles_to_string(app: &mut App, width: u16, height: u16, tick: usize) -> String {
    let buf = render_frame(app, width, height, tick);
    let mut legend: Vec<(ratatui::style::Color, char)> = Vec::new();
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();

    let mut grid = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if cell.symbol().trim().is_empty() {
                grid.push('.');
                continue;
            }
            let fg = cell.style().fg.unwrap_or(ratatui::style::Color::Reset);
            let ch = match legend.iter().find(|(c, _)| *c == fg) {
                Some((_, ch)) => *ch,
                None => {
                    let ch = *alphabet.get(legend.len()).unwrap_or(&'?');
                    legend.push((fg, ch));
                    ch
                }
            };
            grid.push(ch);
        }
        grid.push('\n');
    }

    let mut out = String::from("legend:\n");
    for (color, ch) in &legend {
        out.push_str(&format!("  {ch} = {color:?}\n"));
    }
    out.push('\n');
    out.push_str(&grid);
    out
}

/// Draw one frame into a buffer, with no terminal involved.
pub fn render_frame(
    app: &mut App,
    width: u16,
    height: u16,
    tick: usize,
) -> ratatui::buffer::Buffer {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("the test backend cannot fail to construct");
    terminal
        .draw(|f| draw(f, app, tick))
        .expect("the test backend cannot fail to draw");
    terminal.backend().buffer().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    #[test]
    fn a_progress_bar_reads_as_what_it_is() {
        assert_eq!(progress_bar(0, 4), "▱▱▱▱");
        assert_eq!(progress_bar(100, 4), "▰▰▰▰");
        assert_eq!(progress_bar(50, 4), "▰▰▱▱");
        // Anything above nothing shows something: rounding a real 4% down to an
        // empty bar says "not started", which is a different claim.
        assert!(progress_bar(4, 8).starts_with('▰'));
    }

    /// What the body renderer drew, as plain text, one line per line.
    fn rendered(body: &str, width: usize) -> Vec<String> {
        body_lines(body, &Theme::mono(), width)
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    /// The old renderer deleted `**` and backticks, which turned emphasis
    /// into prose and a command into a sentence.
    #[test]
    fn emphasis_is_rendered_rather_than_deleted() {
        let pieces = inline("a **bold** and `code` and *soft* word");
        let kinds: Vec<Ink> = pieces.iter().map(|p| p.kind).collect();
        assert_eq!(
            kinds,
            vec![
                Ink::Plain,
                Ink::Strong,
                Ink::Plain,
                Ink::Code,
                Ink::Plain,
                Ink::Emphasis,
                Ink::Plain
            ]
        );
        let text: String = pieces.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(text, "a bold and code and soft word");
    }

    /// The failure mode of a half-written markdown renderer is swallowing
    /// what it could not parse, and a reader cannot tell what is missing.
    #[test]
    fn anything_unrecognised_survives_as_what_was_typed() {
        for source in [
            "an **unclosed run",
            "~~strikethrough~~ and <span>html</span>",
            "a * b * c",
            "an empty ** ** run",
            "a [link with no destination]",
            "trailing backtick `",
        ] {
            let text: String = inline(source).iter().map(|p| p.text.as_str()).collect();
            assert_eq!(text, source, "mangled {source:?}");
        }
    }

    #[test]
    fn a_link_shows_its_text_and_keeps_its_destination() {
        let pieces = inline("see [the spec](https://example.org/a) for more");
        let link = pieces.iter().find(|p| p.kind == Ink::Link).expect("a link");
        assert_eq!(link.text, "the spec");
        assert_eq!(link.href.as_deref(), Some("https://example.org/a"));
        let text: String = pieces.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(text, "see the spec for more");
    }

    /// People write far more bare urls than bracketed ones.
    #[test]
    fn a_bare_url_is_a_link_without_the_punctuation_after_it() {
        let pieces = inline("at https://example.org/a, or (https://example.org/b).");
        let links: Vec<&str> = pieces
            .iter()
            .filter(|p| p.kind == Ink::Link)
            .map(|p| p.href.as_deref().unwrap())
            .collect();
        assert_eq!(
            links,
            vec!["https://example.org/a", "https://example.org/b"]
        );
        let text: String = pieces.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(
            text,
            "at https://example.org/a, or (https://example.org/b)."
        );
    }

    /// Markup in the middle of a word is still one word, and a line may not
    /// break inside it.
    #[test]
    fn a_word_split_across_markup_is_not_split_across_lines() {
        let rows = wrap_pieces(&inline("aaaa `bb`cc"), 6);
        let texts: Vec<String> = rows
            .iter()
            .map(|r| r.iter().map(|p| p.text.as_str()).collect())
            .collect();
        assert_eq!(texts, vec!["aaaa", "bbcc"]);
    }

    /// Rejoining pieces must not invent the whitespace markdown removed.
    #[test]
    fn markup_does_not_leave_a_space_behind_it() {
        let rows = wrap_pieces(&inline("some **bold text**, then more"), 80);
        let text: String = rows[0].iter().map(|p| p.text.as_str()).collect();
        assert_eq!(text, "some bold text, then more");
    }

    /// The bug this replaced: code went out in `label`, a link in `accent`
    /// and bold prose in `heading`, so a theme could not change any of them
    /// and setting `label` changed the wrong thing.
    #[test]
    fn markup_answers_to_the_role_named_after_it() {
        for name in ["night", "paper", "gotham"] {
            let t = Theme::resolve(name).expect("a built-in theme");
            let drawn = |source: &str| -> Vec<(String, Color)> {
                body_lines(source, &t, 60)
                    .iter()
                    .flat_map(|l| l.spans.clone())
                    .filter(|s| !s.content.trim().is_empty())
                    .map(|s| (s.content.trim().to_string(), s.style.fg.expect("a colour")))
                    .collect()
            };
            let prose = drawn("plain `code` and [a link](https://x.test) and **bold**.");
            assert!(
                prose.contains(&("code".into(), t.code)),
                "{name}: {prose:?}"
            );
            assert!(
                prose.contains(&("a link".into(), t.link)),
                "{name}: {prose:?}"
            );
            assert!(
                prose.contains(&("bold".into(), t.text)),
                "{name}: {prose:?}"
            );
            // Nothing in a body reaches for chrome any more. `label` is
            // not checked the same way: a narrow palette may spend one hex
            // on two roles, and the fix is that they are separate keys, not
            // that they must differ.
            for (text, colour) in &prose {
                assert_ne!(*colour, t.accent, "{name}: {text:?} is wearing `accent`");
            }

            let fenced = drawn("```\nfn main() {}\n```");
            assert!(
                fenced
                    .iter()
                    .any(|(text, c)| text.contains("fn main") && *c == t.code),
                "{name}: a fence is not drawn in `code`: {fenced:?}"
            );
            for level in ["# one", "## one", "### one"] {
                let heading = drawn(level);
                assert_eq!(heading[0].1, t.heading, "{name}: {level}");
            }
        }
    }

    #[test]
    fn heading_levels_are_told_apart() {
        let t = Theme::mono();
        let styles: Vec<Style> = ["# one", "## two", "### three"]
            .iter()
            .map(|h| body_lines(h, &t, 40)[0].spans[0].style)
            .collect();
        assert_ne!(styles[0], styles[1]);
        assert_ne!(styles[1], styles[2]);
        assert_ne!(styles[0], styles[2]);
    }

    /// A wrapped line of code is a line of code that has been altered.
    #[test]
    fn a_fence_keeps_the_shape_of_what_is_inside_it() {
        let out = rendered(
            "```rust
fn main() {
    indented();
}
```",
            40,
        );
        assert_eq!(out, vec!["  ▏ fn main() {", "  ▏     indented();", "  ▏ }"]);
    }

    /// Markup inside a fence is text, which is how `**` gets written about.
    #[test]
    fn a_fence_holds_its_markup_literally() {
        let out = rendered(
            "```
# not a heading **not bold**
```",
            60,
        );
        assert_eq!(out, vec!["  ▏ # not a heading **not bold**"]);
    }

    #[test]
    fn quotes_rules_and_ordered_lists_each_render() {
        let out = rendered(
            "> said

---

3. third
4. fourth",
            20,
        );
        assert_eq!(out[0], "  ▏ said");
        assert!(out[2].trim().starts_with('─'), "{:?}", out[2]);
        // The author's own numbers, not a renumbering from one.
        assert_eq!(out[4], "  3. third");
        assert_eq!(out[5], "  4. fourth");
    }

    /// A body is hard-wrapped at whatever width its author had. Wrapping each
    /// of those lines on its own reproduces their ragged edge here.
    #[test]
    fn a_hard_wrapped_paragraph_is_rewrapped_as_one() {
        let out = rendered(
            "one two
three four
five six",
            40,
        );
        assert_eq!(out, vec!["  one two three four five six"]);
    }

    /// A continuation line belongs to the list item above it, indented under
    /// its text rather than restarting at the margin.
    #[test]
    fn a_wrapped_list_item_keeps_its_hanging_indent() {
        let out = rendered("- one two three four five six seven", 16);
        assert_eq!(out[0], "  · one two three");
        for line in &out[1..] {
            assert!(line.starts_with("    "), "lost the hang: {line:?}");
        }
    }

    #[test]
    fn nesting_is_kept() {
        let out = rendered(
            "- outer
  - inner",
            40,
        );
        assert_eq!(out, vec!["  · outer", "    · inner"]);
    }

    /// A bar said how far along a milestone was, which is the least useful
    /// thing about it. This says where the remaining work is.
    #[test]
    fn a_container_gets_a_tally_rather_than_a_bar() {
        let app = testkit::app();
        let milestone = app
            .items
            .iter()
            .find(|i| i.scheduled > 0)
            .expect("the fixture has a milestone with work under it");
        let text: String = detail_prose(&app, milestone, &Theme::mono(), 60)
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<&str>>()
            .join("");
        assert!(text.contains("Rollup"), "{text}");
        assert!(text.contains("✓ 1 done"), "{text}");
        assert!(text.contains("◐ 1 in flight"), "{text}");
        assert!(text.contains("○ 2 to start"), "{text}");
        assert!(text.contains("⊘ 1 blocked"), "{text}");
        assert!(!text.contains('▰'), "still drawing a bar: {text}");
    }

    #[test]
    fn every_state_has_its_own_glyph() {
        let mut seen = Vec::new();
        for status in ["backlog", "doing", "done", "dropped"] {
            let mut item = testkit::item(1, "x", status);
            item.category = testkit::schema().category(status);
            seen.push(glyph(&item));
        }
        let mut blocked = testkit::item(1, "x", "backlog");
        blocked.blocked = true;
        seen.push(glyph(&blocked));

        let mut unique = seen.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            seen.len(),
            "two states share a glyph: {seen:?}"
        );
    }

    #[test]
    fn truncation_never_widens_a_row() {
        for width in 0..12 {
            assert!(truncate("a long title indeed", width).chars().count() <= width.max(1));
        }
    }

    /// The reason this is not `Paragraph::wrap`: that wraps to column zero and
    /// the pane stops having a left edge.
    #[test]
    fn wrapped_text_keeps_its_left_edge() {
        let t = Theme::mono();
        let lines = body_lines("one two three four five six seven eight", &t, 12);
        assert!(lines.len() > 1, "it has to have wrapped at all");
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.clone()).collect();
            assert!(text.starts_with("  "), "lost the indent: {text:?}");
            assert!(text.chars().count() <= 14, "ran past the width: {text:?}");
        }
    }

    #[test]
    fn a_word_longer_than_the_line_is_cut_rather_than_allowed_to_push() {
        let parts = wrap("supercalifragilistic", 8);
        assert!(parts.iter().all(|p| p.chars().count() <= 8), "{parts:?}");
        assert_eq!(parts.concat(), "supercalifragilistic");
    }

    #[test]
    fn wrapping_never_loses_a_word() {
        let text = "the body explains why the diff already says what";
        for width in 6..40 {
            assert_eq!(wrap(text, width).join(" ").split_whitespace().count(), 9);
        }
    }

    #[test]
    fn a_frame_is_produced_at_every_reasonable_size() {
        let mut app = testkit::app();
        for (w, h) in [(40u16, 10u16), (56, 20), (80, 24), (120, 40), (200, 60)] {
            let text = render_to_string(&mut app, w, h, 0);
            assert!(!text.is_empty(), "{w}x{h} produced nothing");
        }
    }

    #[test]
    fn a_frame_survives_a_terminal_too_small_to_be_useful() {
        // Not a size anybody works at, but a window being dragged passes
        // through it, and a panic there loses the session.
        let mut app = testkit::app();
        for (w, h) in [(1u16, 1u16), (4, 3), (20, 5), (8, 30)] {
            let _ = render_frame(&mut app, w, h, 0);
        }
    }

    #[test]
    fn a_narrow_pane_drops_the_detail_rather_than_halving_the_list() {
        let mut app = testkit::app();
        let narrow = render_to_string(&mut app, 60, 20, 0);
        assert!(!narrow.contains("Fields"), "the detail pane should be gone");
        assert!(narrow.contains("Backlog"), "and the list should not be");

        let wide = render_to_string(&mut app, 120, 24, 0);
        assert!(wide.contains("Fields"), "with room, it comes back");
    }

    #[test]
    fn the_strip_says_what_is_happening_before_any_row_is_read() {
        let mut app = testkit::app();
        let text = render_to_string(&mut app, 100, 24, 0);
        let strip = text.lines().nth(1).expect("the second line");
        assert!(strip.contains("in progress"), "{strip}");
        assert!(strip.contains("backlog"), "{strip}");
        // Active first: it is a summary, and a summary leads with what is live.
        let doing = strip.find("in progress").expect("doing");
        let backlog = strip.find("backlog").expect("backlog");
        assert!(doing < backlog, "{strip}");
    }

    #[test]
    fn a_row_that_just_moved_says_so() {
        let mut app = testkit::app();
        let before = render_to_string(&mut app, 100, 24, 0);
        assert!(!before.contains(" •"), "nothing has moved yet");

        app.changed.insert(3, app.now);
        let after = render_to_string(&mut app, 100, 24, 0);
        assert!(after.contains(" •"), "a change has to be visible:\n{after}");
    }

    #[test]
    fn every_overlay_draws() {
        let mut app = testkit::app();
        app.help = true;
        let _ = render_frame(&mut app, 100, 30, 0);
        app.help = false;
        app.diagnostics = true;
        let _ = render_frame(&mut app, 100, 30, 0);
        app.diagnostics = false;
        app.reading = true;
        let _ = render_frame(&mut app, 100, 30, 0);
        app.reading = false;
        app.open_picker("status");
        let _ = render_frame(&mut app, 100, 30, 0);
        app.picker = None;
        app.ask_close();
        let _ = render_frame(&mut app, 100, 30, 0);
    }

    #[test]
    fn the_board_draws_a_column_per_declared_status() {
        let mut app = testkit::app();
        app.pane = Pane::Board;
        let text = render_to_string(&mut app, 120, 30, 0);
        assert!(text.contains("backlog"), "{text}");
        assert!(text.contains("in progress"), "the label, not the name");
        assert!(
            !text.contains("dropped"),
            "that column asked not to be shown"
        );
    }

    #[test]
    fn an_empty_backlog_says_what_to_do_about_it() {
        let mut app = App::new();
        let text = render_to_string(&mut app, 100, 24, 0);
        assert!(text.contains("Nothing in the backlog"), "{text}");
    }
}
