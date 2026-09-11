//! All drawing. The screen is a title strip, a two-pane body, and a status
//! strip; overlays (read, help, picker, confirm) are painted on top.
//!
//! Rendering is a pure function of [`App`]: nothing here reads the clock, the
//! environment or the filesystem, which is what makes a frame reproducible and
//! a snapshot test worth having.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap,
};

use crate::app::{App, Row, ToastKind};
use crate::diag;
use crate::item::Item;
use crate::schema::{Category, Schema};
use crate::theme::Theme;

const SPINNER: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

/// One glyph per state, so the screen still says everything it needs to when
/// there is no colour at all — `mono`, `NO_COLOR`, or a reader who cannot tell
/// the green from the red.
pub fn glyph(item: &Item) -> &'static str {
    if item.blocked && !item.category.is_closed() {
        return "⊘";
    }
    match item.category {
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(area);

    draw_titlebar(f, app, &t, chunks[0], tick);
    draw_rule(f, &t, chunks[1]);

    if app.board {
        app.board_area = chunks[2];
        draw_board(f, app, &t, chunks[2]);
    } else {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(chunks[2]);
        app.list_area = split[0];
        draw_list(f, app, &t, split[0]);
        draw_detail(f, app, &t, split[1]);
    }
    draw_status(f, app, &t, chunks[3]);

    if app.reading {
        draw_reader(f, app, &t, area);
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

fn draw_titlebar(f: &mut Frame, app: &App, t: &Theme, area: Rect, tick: usize) {
    let shown = app.shown();
    let ready = app
        .items
        .iter()
        .filter(|i| i.ready(&app.schema) && !i.is_milestone())
        .count();
    // Milestones are excluded from both counts: a milestone waiting on the one
    // before it is the schedule working, not work that is stuck.
    let blocked = app
        .items
        .iter()
        .filter(|i| i.blocked && !i.category.is_closed() && !i.is_milestone())
        .count();

    // Everything after the count is optional; a narrow terminal keeps the
    // identity and the number, and drops the rest rather than colliding.
    let roomy = area.width >= 78;
    let mut left = vec![
        Span::styled(" harrow", Style::default().fg(t.accent).bold()),
        Span::styled("  ", Style::default()),
        Span::styled(
            app.schema.name.clone(),
            Style::default().fg(t.milestone).bold(),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(format!("{shown}"), Style::default().fg(t.text).bold()),
        Span::styled(" items", Style::default().fg(t.muted)),
    ];
    if roomy {
        left.push(Span::styled(" · ", Style::default().fg(t.faint)));
        left.push(Span::styled(
            format!("{ready}"),
            Style::default().fg(t.ready).bold(),
        ));
        left.push(Span::styled(" ready", Style::default().fg(t.muted)));
        if blocked > 0 {
            left.push(Span::styled(" · ", Style::default().fg(t.faint)));
            left.push(Span::styled(
                format!("{blocked}"),
                Style::default().fg(t.blocked).bold(),
            ));
            left.push(Span::styled(" blocked", Style::default().fg(t.blocked)));
        }
        if let Some(view) = &app.view {
            left.push(Span::styled(" · ", Style::default().fg(t.faint)));
            left.push(Span::styled(
                format!("view {view}"),
                Style::default().fg(t.secondary),
            ));
        }
        if !app.filter.is_empty() {
            left.push(Span::styled(" · ", Style::default().fg(t.faint)));
            left.push(Span::styled(
                format!("filter “{}”", app.filter),
                Style::default().fg(t.warn),
            ));
        }
    }

    let right = if let Some(fail) = &app.failure {
        format!(
            "⚠ cannot read the backlog ({}×){} ",
            fail.count,
            if fail.transient { ", retrying" } else { "" }
        )
    } else if !app.watcher_alive {
        "⚠ watcher stopped ".to_string()
    } else if app.loading {
        format!("{} reading ", SPINNER[tick % SPINNER.len()])
    } else {
        match app.last_load {
            Some(at) => {
                let e = at.elapsed();
                if e.as_secs() == 0 {
                    "updated just now ".to_string()
                } else {
                    format!("updated {} ago ", ago(e))
                }
            }
            None => String::from("starting "),
        }
    };

    f.render_widget(Line::from(left), area);
    let right_style = if app.is_stale() {
        Style::default().fg(t.error).bold()
    } else {
        Style::default().fg(t.muted)
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(right, right_style))).alignment(Alignment::Right),
        area,
    );
}

fn draw_rule(f: &mut Frame, t: &Theme, area: Rect) {
    let rule = "─".repeat(area.width as usize);
    f.render_widget(
        Line::from(Span::styled(rule, Style::default().fg(t.faint))),
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
        .border_style(Style::default().fg(t.faint))
        .title(Line::from(Span::styled(
            title,
            Style::default().fg(t.text).bold(),
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
    let window = &app.rows[app.offset.min(end)..end];

    let items: Vec<ListItem> = window
        .iter()
        .map(|row| match row {
            Row::Group(g) => group_line(app, t, *g, inner_width),
            Row::Item(i) => item_line(&app.items[*i], &app.schema, t, inner_width),
        })
        .collect();

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

    // The count and the bar are the point of the row, so they get their width
    // first; the name fills whatever is left. The count is of everything under
    // the heading, with what the filter is showing said separately — a bar that
    // moved when you pressed `a` would be a bar nobody could believe.
    let percent = g.percent(&app.items);
    let bar = progress_bar(percent, 8);
    let count = if g.shown == g.count {
        format!("{:>3}", g.count)
    } else {
        format!("{} of {}", g.shown, g.count)
    };
    let blocked = if g.blocked > 0 {
        format!("⊘{}  ", g.blocked)
    } else {
        String::new()
    };

    // Degrade in a defined order — the bar, then the percentage, then the count
    // of what is hidden — so a narrow terminal loses the decoration rather than
    // the name of the thing.
    const MIN_NAME: usize = 12;
    let room = width.saturating_sub(marker.chars().count());
    let candidates = [
        format!("{bar} {percent:>3}%  {blocked}{count} "),
        format!("{percent:>3}%  {blocked}{count} "),
        format!("{blocked}{count} "),
        format!("{count} "),
    ];
    let right = candidates
        .iter()
        .find(|r| room.saturating_sub(r.chars().count()) >= MIN_NAME)
        .unwrap_or_else(|| candidates.last().expect("one candidate always exists"))
        .clone();

    let budget = room.saturating_sub(right.chars().count());
    let name = truncate(&g.label, budget);
    let pad = budget.saturating_sub(name.chars().count());

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

fn item_line(item: &Item, schema: &Schema, t: &Theme, width: usize) -> ListItem<'static> {
    let reference = schema.format_id(item.id);
    let icon = schema
        .item_type(&item.kind)
        .and_then(|k| k.icon.clone())
        .unwrap_or_else(|| " ".to_string());

    // Degrade in a defined order — the rank tag, then the marks, then the title
    // — so a narrow terminal loses detail instead of losing its shape.
    let rank = rank_tag(item, schema);
    let marks = marks(item);
    let lead = 2 + 1 + 1 + reference.chars().count() + 1 + icon.chars().count() + 1;
    let avail = width.saturating_sub(lead + 1);

    const MIN_TITLE: usize = 8;
    let rank_cost = if rank.is_empty() {
        0
    } else {
        rank.chars().count() + 1
    };
    let mark_cost = if marks.is_empty() {
        0
    } else {
        marks.chars().count() + 1
    };
    let (show_rank, show_marks) = if avail >= rank_cost + mark_cost + MIN_TITLE {
        (true, true)
    } else if avail >= rank_cost + MIN_TITLE {
        (true, false)
    } else {
        (false, false)
    };

    let reserved = if show_rank { rank_cost } else { 0 } + if show_marks { mark_cost } else { 0 };
    let title_width = avail.saturating_sub(reserved);
    let title = truncate(&item.title, title_width);
    let pad = title_width.saturating_sub(title.chars().count());

    let dim = item.category.is_closed();
    let title_style = if dim {
        Style::default()
            .fg(t.faint)
            .add_modifier(Modifier::CROSSED_OUT)
    } else {
        Style::default().fg(t.text)
    };

    let mut spans = vec![
        Span::raw("  "),
        Span::styled(
            glyph(item),
            Style::default().fg(state_color(item, t, schema)),
        ),
        Span::raw(" "),
        Span::styled(reference, Style::default().fg(t.faint)),
        Span::raw(" "),
        Span::styled(
            icon,
            Style::default().fg(t.item_type(schema.item_type(&item.kind))),
        ),
        Span::raw(" "),
        Span::styled(title, title_style),
        Span::raw(" ".repeat(pad)),
    ];
    if show_marks && !marks.is_empty() {
        spans.push(Span::styled(
            format!(" {marks}"),
            Style::default().fg(t.muted),
        ));
    }
    if show_rank && !rank.is_empty() {
        spans.push(Span::raw(" "));
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

/// The short marks after a title: who holds it, how much of it is ticked off,
/// and whether anybody has said anything about it.
fn marks(item: &Item) -> String {
    let mut out = String::new();
    if item.assignee.is_some() {
        out.push('@');
    }
    let (done, total) = item.criteria();
    if total > 0 {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&format!("{done}/{total}"));
    }
    if !item.labels.is_empty() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push('#');
    }
    out
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
                    .border_style(Style::default().fg(t.faint)),
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

    for (index, cell) in cells.iter().enumerate() {
        let column = &app.columns[index];
        let focused = index == app.column;
        let status = app.schema.status(&column.status);
        let color = t.status(status);
        let border = if focused { color } else { t.faint };

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
        let selected = if focused { app.column_row } else { usize::MAX };
        let offset = if focused {
            scroll_to(0, app.column_row, column.items.len(), inner_height)
        } else {
            0
        };
        let end = (offset + inner_height).min(column.items.len());

        let cards: Vec<ListItem> = column.items[offset.min(end)..end]
            .iter()
            .map(|i| card_line(&app.items[*i], &app.schema, t, inner_width))
            .collect();

        let list = List::new(cards).block(block).highlight_style(if focused {
            t.selected()
        } else {
            Style::default()
        });
        let mut state = ListState::default().with_selected(
            (focused && !column.items.is_empty()).then(|| selected.saturating_sub(offset)),
        );
        f.render_stateful_widget(list, *cell, &mut state);
    }
}

fn card_line(item: &Item, schema: &Schema, t: &Theme, width: usize) -> ListItem<'static> {
    let reference = schema.format_id(item.id);
    let rank = rank_tag(item, schema);
    let lead = 1 + 1 + 1 + reference.chars().count() + 1;
    let rank_cost = if rank.is_empty() {
        0
    } else {
        rank.chars().count() + 1
    };
    let title_width = width.saturating_sub(lead + rank_cost);
    let title = truncate(&item.title, title_width);
    let pad = title_width.saturating_sub(title.chars().count());

    let mut spans = vec![
        Span::raw(" "),
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
        spans.push(Span::raw(" "));
        spans.push(Span::styled(rank.clone(), rank_style(item, schema, t)));
    }
    ListItem::new(Line::from(spans))
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
                    .border_style(Style::default().fg(t.faint))
                    .title(Span::styled(" Detail ", Style::default().fg(t.text).bold())),
            ),
            area,
        );
        return;
    };
    let schema = &app.schema;
    let reference = schema.format_id(item.id);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.faint))
        .padding(Padding::horizontal(1))
        .title(Line::from(vec![
            Span::styled(format!(" {reference} "), Style::default().fg(t.text).bold()),
            Span::styled(
                format!("{} ", item.kind),
                Style::default().fg(t.item_type(schema.item_type(&item.kind))),
            ),
        ]));

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} ", glyph(item)),
            Style::default().fg(state_color(item, t, schema)),
        ),
        Span::styled(item.title.clone(), Style::default().fg(t.text).bold()),
    ]));

    // The one line that says where this stands.
    let mut state = vec![
        Span::raw("  "),
        Span::styled(
            schema
                .status(&item.status)
                .map(|s| s.display().to_string())
                .unwrap_or_else(|| item.status.clone()),
            Style::default().fg(t.status(schema.status(&item.status))),
        ),
    ];
    if let Some(milestone) = item.milestone() {
        state.push(Span::styled(" · ", Style::default().fg(t.faint)));
        state.push(Span::styled(
            milestone.to_string(),
            Style::default().fg(t.milestone),
        ));
    }
    if let Some(who) = &item.assignee {
        state.push(Span::styled(" · ", Style::default().fg(t.faint)));
        state.push(Span::styled(
            format!("@{who}"),
            Style::default().fg(t.person),
        ));
    }
    lines.push(Line::from(state));
    lines.push(Line::from(""));

    if item.blocked {
        lines.push(section("Waiting on", t));
        for id in &item.blockers {
            let title = app
                .items
                .iter()
                .find(|i| i.id == *id)
                .map(|i| i.title.clone())
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(schema.format_id(*id), Style::default().fg(t.blocked)),
                Span::raw(" "),
                Span::styled(title, Style::default().fg(t.muted)),
            ]));
        }
        lines.push(Line::from(""));
    }

    if let Some(percent) = item.progress() {
        lines.push(section("Progress", t));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                progress_bar(percent, 12),
                Style::default().fg(if percent == 100 { t.done } else { t.accent }),
            ),
            Span::styled(
                format!(
                    "  {percent}% · {} of {}",
                    item.scheduled_done, item.scheduled
                ),
                Style::default().fg(t.muted),
            ),
        ]));
        lines.push(Line::from(""));
    }

    let (met, total) = item.criteria();
    if total > 0 {
        lines.push(section("Acceptance", t));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{met} of {total} ticked"),
                Style::default().fg(if met == total { t.done } else { t.muted }),
            ),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(section("Fields", t));
    for field in &schema.fields {
        if field.name == "milestone" {
            continue;
        }
        if let Some(value) = item.field(&field.name).filter(|v| !v.is_empty()) {
            lines.push(kv(&field.name, &value.display(), t));
        }
    }
    if !item.labels.is_empty() {
        lines.push(kv("labels", &item.labels.join(", "), t));
    }
    if let Some(created) = &item.created {
        lines.push(kv("created", created, t));
    }
    if let Some(updated) = &item.updated {
        lines.push(kv("updated", updated, t));
    }
    if let Some(claimed) = &item.claimed {
        lines.push(kv("claimed", claimed, t));
    }
    lines.push(Line::from(""));

    // The body, as much of it as fits. `enter` reads the rest.
    let inner = block.inner(area);
    let room = (inner.height as usize).saturating_sub(lines.len() + 2);
    if room > 1 {
        lines.push(section("Body", t));
        let mut shown = 0;
        for line in item.body.lines() {
            if shown >= room.saturating_sub(1) {
                lines.push(Line::from(Span::styled(
                    "  … ↵ to read it all",
                    Style::default().fg(t.faint).italic(),
                )));
                break;
            }
            if line.trim().is_empty() && shown == 0 {
                continue;
            }
            lines.push(body_line(line, t));
            shown += 1;
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// Markdown, at the small amount of fidelity a pane this size earns: headings
/// stand out, checkboxes read as ticked or not, and everything else is text.
fn body_line(line: &str, t: &Theme) -> Line<'static> {
    let trimmed = line.trim_start();
    if let Some(heading) = trimmed.strip_prefix("## ").or(trimmed.strip_prefix("# ")) {
        return Line::from(Span::styled(
            format!("  {heading}"),
            Style::default().fg(t.heading).bold(),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("- [") {
        let ticked = !rest.starts_with(' ');
        let text = plain_markdown(rest.split_once("] ").map(|x| x.1).unwrap_or(""));
        return Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if ticked { "✓ " } else { "☐ " },
                Style::default().fg(if ticked { t.done } else { t.faint }),
            ),
            Span::styled(text, Style::default().fg(t.muted)),
        ]);
    }
    Line::from(Span::styled(
        format!("  {}", plain_markdown(line)),
        Style::default().fg(t.muted),
    ))
}

/// The two bits of inline markup that are noise rather than emphasis when the
/// emphasis cannot be rendered. Everything else is left exactly as written.
fn plain_markdown(line: &str) -> String {
    line.replace("**", "").replace('`', "")
}

fn section(name: &str, t: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        name.to_uppercase(),
        Style::default().fg(t.faint).add_modifier(Modifier::BOLD),
    ))
}

fn kv(key: &str, value: &str, t: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{key:<9}"), Style::default().fg(t.faint)),
        Span::styled(value.to_string(), Style::default().fg(t.muted)),
    ])
}

// ── Overlays ─────────────────────────────────────────────────────────────────

/// The whole item, which is the thing cairn keeps that a listing cannot show:
/// the problem, the proposal, and what was decided.
fn draw_reader(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(item) = app.selected_item() else {
        return;
    };
    let width = 96u16.min(area.width.saturating_sub(4));
    // Below the title strip: the identity of what you are reading stays on
    // screen while you read it.
    let height = area.height.saturating_sub(4);
    let popup = centered(area, width, height);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(app.schema.format_id(item.id), Style::default().fg(t.faint)),
            Span::raw("  "),
            Span::styled(item.title.clone(), Style::default().fg(t.text).bold()),
        ]),
        Line::from(""),
    ];
    for line in item.body.lines() {
        lines.push(body_line(line, t));
    }

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .scroll((app.read_scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(t.accent))
                    .padding(Padding::horizontal(1))
                    .title(Span::styled(" Item ", Style::default().fg(t.accent).bold()))
                    .title_bottom(Span::styled(
                        " ↑↓ scroll · any other key closes ",
                        Style::default().fg(t.faint),
                    )),
            ),
        popup,
    );
}

fn draw_picker(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(picker) = &app.picker else { return };
    let width = 52u16.min(area.width.saturating_sub(4));
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

    let mut state = ListState::default().with_selected(Some(picker.selected));
    f.render_widget(Clear, popup);
    f.render_stateful_widget(
        List::new(items).highlight_style(t.selected()).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.accent))
                .title(Span::styled(
                    format!(" {} ", picker.title),
                    Style::default().fg(t.accent).bold(),
                ))
                .title_bottom(Span::styled(
                    " ↵ set · esc cancel ",
                    Style::default().fg(t.faint),
                )),
        ),
        popup,
        &mut state,
    );
}

fn draw_help(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    // Generated from the active bindings. A help screen that lists the defaults
    // while the user runs something else is worse than no help screen.
    let rows = app.keymap.help_rows();
    let width = 74u16.min(area.width.saturating_sub(4));
    let height = (rows.len() as u16 + 4).min(area.height.saturating_sub(2));
    let popup = centered(area, width, height);

    let key_col = rows
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(8)
        .clamp(8, 18);
    let room = (width as usize).saturating_sub(key_col + 5);

    let mut lines = vec![Line::from("")];
    for (keys, description) in rows {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{keys:<key_col$}"),
                Style::default().fg(t.accent).bold(),
            ),
            Span::raw(" "),
            Span::styled(truncate(description, room), Style::default().fg(t.muted)),
        ]));
    }

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.accent))
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
    if !app.writable {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "cairn is not on PATH — this backlog is read-only",
                Style::default().fg(t.warn),
            ),
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
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("THIS BACKLOG", Style::default().fg(t.faint).bold()),
        ]));
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

fn draw_confirm(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(c) = &app.confirm else { return };
    let popup = centered(area, 60.min(area.width.saturating_sub(4)), 7);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(c.prompt.clone(), Style::default().fg(t.text).bold()),
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
                .border_style(Style::default().fg(t.accent)),
        ),
        popup,
    );
}

// ── The status strip ─────────────────────────────────────────────────────────

fn draw_status(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    use crate::app::Editing;
    if let Some(editing) = &app.editing {
        let (label, hint) = match editing {
            Editing::Filter => (" filter ", "   enter to keep · esc to clear"),
            Editing::NewItem => (" title  ", "   enter to create · esc to cancel"),
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
                Span::styled(msg.clone(), Style::default().fg(color)),
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

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, label)) in app.keymap.footer_hints().into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(key, Style::default().fg(t.accent).bold()));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(t.faint),
        ));
    }
    if !app.writable {
        spans.push(Span::styled("   read-only", Style::default().fg(t.warn)));
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
        60..=3599 => format!("{}m {}s", s / 60, s % 60),
        3600..=86399 => format!("{}h {}m", s / 3600, (s % 3600) / 60),
        _ => format!("{}d {}h", s / 86400, (s % 86400) / 3600),
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

    #[test]
    fn a_frame_is_produced_at_every_reasonable_size() {
        let mut app = testkit::app();
        for (w, h) in [(40u16, 10u16), (80, 24), (120, 40), (200, 60)] {
            let text = render_to_string(&mut app, w, h, 0);
            assert!(!text.is_empty(), "{w}x{h} produced nothing");
        }
    }

    #[test]
    fn a_frame_survives_a_terminal_too_small_to_be_useful() {
        // Not a size anybody works at, but a window being dragged passes
        // through it, and a panic there loses the session.
        let mut app = testkit::app();
        for (w, h) in [(1u16, 1u16), (4, 3), (20, 5)] {
            let _ = render_frame(&mut app, w, h, 0);
        }
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
        app.board = true;
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
