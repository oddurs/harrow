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
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, ListState, Padding, Paragraph};

use crate::app::{App, Door, Hit, Pane, ReadOnly, Row, ToastKind};
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
/// What the stats pane needs to lay itself out in two columns, which is what
/// it does whenever it has the room: its sections are short, and one column
/// of them is mostly whitespace.
const STATS_TWO_COLUMN: u16 = 88;
/// Below this, the header drops to the identity and the counts.
const ROOMY: u16 = 74;

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
    let strip = u16::from(area.height >= 12 && !app.status_counts().is_empty());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),     // identity, tabs, freshness
            Constraint::Length(strip), // what is happening
            Constraint::Length(1),     // rule
            Constraint::Min(3),        // the work
            Constraint::Length(1),     // keys, or what just happened
        ])
        .split(area);

    draw_header(f, app, &t, chunks[0], tick);
    if strip == 1 {
        draw_strip(f, app, &t, chunks[1]);
    }
    draw_rule(f, &t, chunks[2]);

    // The detail belongs to the selection, not to the list: the same item is
    // selected whichever lens is showing, and there is no reason a board
    // should know less about it than a list does. What differs is how much
    // room a lens needs before it can spare the width — which is a property
    // of the arrangement, so each lens says.
    let (body, detail) = split_off_detail(app, chunks[3]);
    match app.pane {
        Pane::Needs => draw_needs(f, app, &t, body),
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
    if let Some(detail) = detail {
        draw_detail(f, app, &t, detail);
    }
    draw_footer(f, app, &t, chunks[4]);

    if app.reading {
        draw_reader(f, app, &t, area);
    }
    if app.history.is_some() {
        draw_history(f, app, &t, area);
    }
    if app.help {
        draw_help(f, app, &t, area);
    }
    if app.diagnostics {
        draw_diagnostics(f, app, &t, area);
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

fn draw_rule(f: &mut Frame, t: &Theme, area: Rect) {
    let rule = "─".repeat(area.width as usize);
    f.render_widget(
        Line::from(Span::styled(rule, Style::default().fg(t.border))),
        area,
    );
}

// ── The list ─────────────────────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let title = if matches!(app.group_by.as_str(), "none" | "") {
        " Backlog ".to_string()
    } else {
        format!(" Backlog · by {} ", app.group_by)
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .title(Line::from(Span::styled(
            title,
            Style::default().fg(t.muted),
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
    let (headline, hint) = if app.items.is_empty() {
        (
            "Nothing in the backlog yet.",
            "Press n to write the first item.",
        )
    } else if !app.filter.is_empty() || app.view.is_some() {
        ("No matches.", "esc clears the filter.")
    } else {
        (
            "Nothing open here.",
            "Press a to show finished work and milestones.",
        )
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
    let who = item
        .assignee
        .as_deref()
        .map(|a| format!("@{}", truncate(a, 8)))
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
        // An item that looks taken and is not looks exactly like an item
        // somebody is working on right now, which is the one thing a pane
        // beside the work is supposed to distinguish.
        let colour = if app.claim_is_stale(item) {
            t.warn
        } else {
            t.person
        };
        spans.push(Span::styled(who, Style::default().fg(colour)));
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

    // Every column the same width. A board whose columns move as items arrive
    // is a board you cannot learn the shape of.
    let count = app.columns.len();
    let constraints: Vec<Constraint> = (0..count)
        .map(|_| Constraint::Ratio(1, count as u32))
        .collect();
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
            .title(Line::from(vec![
                Span::styled(
                    format!(" {} ", column.label),
                    Style::default().fg(color).bold(),
                ),
                Span::styled(
                    format!("{} ", column.items.len()),
                    Style::default().fg(t.faint),
                ),
            ]));

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
        Pane::Needs => DETAIL_MIN_WIDTH,
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
    let lines = detail_lines(app, item, t, inner.width as usize);

    // Clamped here because here is where the height of the content is known.
    // Past the end of a short item is not a place the pane can be.
    let over = lines.len().saturating_sub(inner.height as usize);
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

    f.render_widget(Paragraph::new(lines).scroll((scroll, 0)).block(block), area);
    app.hit(area, Hit::Detail);
}

/// The body of the detail pane.
///
/// Built as whole lines rather than handed to a wrapping widget: ratatui's wrap
/// does not know about the indent a line started with, so a wrapped paragraph
/// loses its left edge and the pane stops having one.
///
/// Everything the item has, at whatever length that comes to. What fits is the
/// pane's business, and the pane scrolls.
fn detail_lines(app: &App, item: &Item, t: &Theme, width: usize) -> Vec<Line<'static>> {
    let schema = &app.schema;
    let mut lines: Vec<Line> = Vec::new();

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
        state.push(Span::styled(
            format!("@{who}"),
            Style::default().fg(if stale { t.warn } else { t.person }),
        ));
        // Marking it raises the question; the pane is where there is room to
        // answer it.
        if let Some(days) = app.claimed_days(item).filter(|_| stale) {
            state.push(Span::styled(
                format!(" · held {days} days"),
                Style::default().fg(t.warn),
            ));
        }
    }
    lines.push(Line::from(state));

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
            let room = width.saturating_sub(reference.chars().count() + 3);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(reference, Style::default().fg(t.blocked)),
                Span::raw(" "),
                Span::styled(truncate(&title, room), Style::default().fg(t.muted)),
            ]));
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

    if let Some(percent) = item.progress() {
        lines.push(Line::from(""));
        lines.push(section("Progress", t, width));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                progress_bar(percent, 10),
                Style::default().fg(if percent == 100 { t.done } else { t.accent }),
            ),
            Span::styled(
                format!("  {} of {} done", item.scheduled_done, item.scheduled),
                Style::default().fg(t.muted),
            ),
        ]));
    }

    let (met, total) = item.criteria();
    if let Some(percent) = (met * 100).checked_div(total) {
        lines.push(Line::from(""));
        lines.push(section("Acceptance", t, width));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                progress_bar(percent, 10),
                Style::default().fg(if met == total { t.done } else { t.accent }),
            ),
            Span::styled(
                format!("  {met} of {total} ticked"),
                Style::default().fg(t.muted),
            ),
        ]));
    }

    // A grid, not a list: one column of labels, one of values, so the eye runs
    // down the labels instead of reading every line to find the one it wants.
    let mut fields: Vec<(String, String)> = Vec::new();
    for field in &schema.fields {
        if field.name == "milestone" {
            continue;
        }
        if let Some(value) = item.field(&field.name).filter(|v| !v.is_empty()) {
            fields.push((field.name.clone(), value.display()));
        }
    }
    if !item.labels.is_empty() {
        fields.push(("labels".into(), item.labels.join(", ")));
    }
    if let Some(owner) = &item.owner {
        fields.push(("owner".into(), owner.clone()));
    }
    if let Some(by) = &item.created_by {
        fields.push(("filed by".into(), by.clone()));
    }
    for (label, value) in [("created", &item.created), ("updated", &item.updated)] {
        if let Some(value) = value {
            fields.push((label.into(), value.clone()));
        }
    }
    if !fields.is_empty() {
        let label_width = fields
            .iter()
            .map(|(k, _)| k.chars().count())
            .max()
            .unwrap_or(0)
            .min(12);
        lines.push(Line::from(""));
        lines.push(section("Fields", t, width));
        for (label, value) in fields {
            let room = width.saturating_sub(label_width + 3);
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{label:<label_width$}"),
                    Style::default().fg(t.faint),
                ),
                Span::raw(" "),
                Span::styled(truncate(&value, room), Style::default().fg(t.muted)),
            ]));
        }
    }

    if !item.body.trim().is_empty() {
        lines.push(Line::from(""));
        lines.push(section("Body", t, width));
        lines.extend(body_lines(&item.body, t, width.saturating_sub(2)));
    }

    lines
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

/// Markdown, at the fidelity a pane this size earns: headings stand out,
/// checkboxes read as ticked or not, and everything else is text that keeps its
/// left edge when it wraps.
fn body_lines(body: &str, t: &Theme, width: usize) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let mut blank = false;
    // A paragraph in the file is hard-wrapped at whatever width its author was
    // working at. Re-wrapping each of those lines on its own reproduces their
    // ragged edge inside a pane of a different width, so consecutive prose
    // lines are joined back into a paragraph first and wrapped once.
    let mut paragraph = String::new();

    macro_rules! flush {
        () => {
            if !paragraph.is_empty() {
                for part in wrap(&plain_markdown(&paragraph), width) {
                    out.push(Line::from(Span::styled(
                        format!("  {part}"),
                        Style::default().fg(t.muted),
                    )));
                }
                paragraph.clear();
            }
        };
    }

    for raw in body.lines() {
        let trimmed = raw.trim_start();

        if trimmed.is_empty() {
            flush!();
            // One blank line between things, never three.
            if !out.is_empty() && !blank {
                out.push(Line::from(""));
            }
            blank = true;
            continue;
        }
        blank = false;

        if trimmed.starts_with('#')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || raw.starts_with("    ")
        {
            flush!();
        }

        if let Some(heading) = trimmed
            .strip_prefix("### ")
            .or_else(|| trimmed.strip_prefix("## "))
            .or_else(|| trimmed.strip_prefix("# "))
        {
            for part in wrap(&plain_markdown(heading), width) {
                out.push(Line::from(Span::styled(
                    format!("  {part}"),
                    Style::default().fg(t.heading).bold(),
                )));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- [") {
            let ticked = !rest.starts_with(' ');
            let text = plain_markdown(rest.split_once("] ").map(|x| x.1).unwrap_or(""));
            for (n, part) in wrap(&text, width.saturating_sub(2)).into_iter().enumerate() {
                out.push(Line::from(vec![
                    Span::raw("  "),
                    if n == 0 {
                        Span::styled(
                            if ticked { "✓ " } else { "☐ " },
                            Style::default().fg(if ticked { t.done } else { t.faint }),
                        )
                    } else {
                        Span::raw("  ")
                    },
                    Span::styled(part, Style::default().fg(t.muted)),
                ]));
            }
            continue;
        }

        if let Some(text) = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")) {
            for (n, part) in wrap(&plain_markdown(text), width.saturating_sub(2))
                .into_iter()
                .enumerate()
            {
                out.push(Line::from(vec![
                    Span::raw("  "),
                    if n == 0 {
                        Span::styled("· ", Style::default().fg(t.faint))
                    } else {
                        Span::raw("  ")
                    },
                    Span::styled(part, Style::default().fg(t.muted)),
                ]));
            }
            continue;
        }

        // An indented line is code or a command; it keeps its own shape.
        if raw.starts_with("    ") {
            out.push(Line::from(Span::styled(
                format!("  {}", truncate(raw.trim_end(), width)),
                Style::default().fg(t.faint),
            )));
            continue;
        }

        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed.trim_end());
    }
    flush!();
    out
}

/// The two bits of inline markup that are noise rather than emphasis when the
/// emphasis cannot be rendered. Everything else is left exactly as written.
fn plain_markdown(line: &str) -> String {
    line.replace("**", "").replace('`', "")
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

/// The whole item, which is the thing cairn keeps that a listing cannot show:
/// the problem, the proposal, and what was decided.
fn draw_reader(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(item) = app.selected_item() else {
        return;
    };
    let width = 92u16.min(area.width.saturating_sub(4));
    // Below the header, so what you are reading stays identified while you read.
    let height = area.height.saturating_sub(4);
    let popup = centered(area, width, height);
    let inner = width.saturating_sub(4) as usize;

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}  ", app.schema.format_id(item.id)),
                Style::default().fg(t.faint),
            ),
            Span::styled(
                truncate(&item.title, inner.saturating_sub(10)),
                Style::default().fg(t.heading).bold(),
            ),
        ]),
        Line::from(""),
    ];
    lines.extend(body_lines(&item.body, t, inner));

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).scroll((app.read_scroll, 0)).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus))
                .padding(Padding::horizontal(1))
                .title(Span::styled(" Item ", Style::default().fg(t.muted)))
                .title_bottom(Span::styled(
                    " ↑↓ scroll · any other key closes ",
                    Style::default().fg(t.faint),
                )),
        ),
        popup,
    );
}

/// How an item got the way it is.
///
/// The reason an item file is worth keeping in the repository rather than in a
/// database: its history is the answer to "when did this become p0, and who
/// decided that?", and it is already there.
fn draw_history(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
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

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).scroll((history.scroll, 0)).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus))
                .title(Span::styled(
                    format!(" {} · history ", app.schema.format_id(history.id)),
                    Style::default().fg(t.muted),
                ))
                .title_bottom(Span::styled(
                    " ↑↓ scroll · any other key closes ",
                    Style::default().fg(t.faint),
                )),
        ),
        popup,
    );
}

fn draw_picker(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let Some(picker) = &app.picker else { return };
    let width = 54u16.min(area.width.saturating_sub(4));
    let height = ((picker.options.len() + 2) as u16).min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);

    // Numbered rather than lettered: three of a project's statuses can begin
    // with the same letter, and a shortcut that is ambiguous is not a shortcut.
    let note_col = picker
        .options
        .iter()
        .map(|(_, _, n)| n.chars().count())
        .max()
        .unwrap_or(0);
    let label_col = (width as usize).saturating_sub(note_col + 7);
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
                Span::styled(format!("{note} "), Style::default().fg(t.faint)),
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
                    if picker.propose {
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
    use crate::app::Editing;
    if let Some(editing) = &app.editing {
        let (label, hint) = match editing {
            Editing::Filter => (" filter ", "   enter to keep · esc to clear"),
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
        } else {
            spans.push(Span::styled(hint, Style::default().fg(t.faint)));
        }
        f.render_widget(Line::from(spans), area);
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
