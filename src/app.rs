//! UI state and the actions bound to keys.
//!
//! Nothing here performs a side effect. Changing an item produces a [`Change`]
//! — the `cairn` invocation that would do it — and hands it back for the shell
//! to run. That is what lets the whole interaction layer, including every write,
//! be driven from a test with no terminal and no repository underneath it.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::diag;
use crate::engine::Report;
use crate::filter::Query;
use crate::item::Item;
use crate::keys::{Command, Keymap};
use crate::schema::{Category, Schema};
use crate::theme::Theme;

/// A row in the list. The board has its own geometry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Group(usize),
    Item(usize),
}

/// A `cairn` invocation. The only way harrow changes anything: it does not write
/// item files itself, because cairn owns the locking, the hooks and the rules
/// about what a valid item is, and a second writer would own none of them.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Change {
    pub args: Vec<String>,
    /// What to say happened, once it has.
    pub describe: String,
    /// What would put it back, for the toast. Not run automatically.
    pub undo: Option<String>,
}

/// What the shell around `App` must do after an input.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    Quit,
    Refresh,
    /// Re-read the config and theme from disk.
    Reload,
    SetMouse(bool),
    /// Put text on the system clipboard.
    Copy(String),
    /// Suspend the interface and open a file in the user's editor.
    Edit(std::path::PathBuf),
    /// Ask cairn to change something.
    Write(Change),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Good,
    Bad,
}

/// A heading in the list: the value everything under it shares.
pub struct Group {
    /// The raw value — `v0.1`, `doing`, or empty for "not set".
    pub key: String,
    /// What to print, which for a milestone is its title rather than its key.
    pub label: String,
    /// Every item under this heading, whether or not the current filter shows
    /// it. Progress that moved when you pressed `a` would be progress nobody
    /// could believe.
    pub count: usize,
    pub done: usize,
    pub blocked: usize,
    /// How many of them are on screen right now.
    pub shown: usize,
    /// The item the group is named after, where the heading is itself an item.
    pub item: Option<usize>,
}

impl Group {
    /// How far along, counting everything under the heading. A milestone with
    /// a rollup of its own uses that, so work nested two levels down still
    /// counts towards it.
    pub fn percent(&self, items: &[Item]) -> u32 {
        if let Some(item) = self.item.and_then(|i| items.get(i))
            && let Some(percent) = item.progress()
        {
            return percent;
        }
        if self.count == 0 {
            return 0;
        }
        (self.done * 100 / self.count) as u32
    }
}

/// A column on the board.
pub struct Column {
    pub status: String,
    pub label: String,
    pub items: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Failure {
    pub detail: String,
    pub transient: bool,
    pub since: Instant,
    pub count: u32,
}

pub struct Confirm {
    pub prompt: String,
    pub detail: String,
    pub change: Change,
}

/// A list of values to choose between: a status, a priority, a milestone.
/// Triage is picking from a small set over and over, and typing the value each
/// time is the thing a screen is supposed to remove.
pub struct Picker {
    pub title: String,
    /// `(value, label, note)`.
    pub options: Vec<(String, String, String)>,
    pub selected: usize,
    /// The field being set. `status` is a field like any other here.
    pub field: String,
    pub id: u32,
}

/// A line of text being typed: the filter box, or a new item's title.
#[derive(PartialEq, Eq)]
pub enum Editing {
    Filter,
    NewItem,
}

pub struct App {
    pub schema: Schema,
    pub items: Vec<Item>,
    pub groups: Vec<Group>,
    pub rows: Vec<Row>,
    pub columns: Vec<Column>,
    /// Which group each item belongs to, by item index.
    group_of: Vec<usize>,
    /// id → index, so a write can find what it changed without a linear scan.
    by_id: HashMap<u32, usize>,

    pub selected: usize,
    pub offset: usize,
    pub column: usize,
    pub column_row: usize,
    pub collapsed: HashSet<String>,

    pub filter: String,
    pub query: Query,
    pub input: String,
    pub editing: Option<Editing>,
    pub group_by: String,
    pub sort: String,
    pub view: Option<String>,
    pub show_closed: bool,
    pub board: bool,

    pub reading: bool,
    pub read_scroll: u16,
    pub picker: Option<Picker>,
    pub confirm: Option<Confirm>,
    pub help: bool,
    pub diagnostics: bool,
    pub mouse: bool,

    pub loading: bool,
    pub last_load: Option<Instant>,
    pub failure: Option<Failure>,
    /// False if the background thread has died — the UI keeps working and says so.
    pub watcher_alive: bool,
    /// Whether `cairn` is available to write with. Read-only is a legitimate
    /// way to run, and is said out loud rather than discovered by pressing a key.
    pub writable: bool,
    pub warnings: Vec<String>,

    pub toast: Option<(String, ToastKind, Instant)>,
    /// Wall-clock seconds, refreshed once per frame rather than read during a
    /// render. Rendering has to be a pure function of state, or a snapshot of
    /// the screen is not reproducible.
    pub now: u64,
    pub theme: Theme,
    pub keymap: Keymap,
    pub list_area: Rect,
    pub board_area: Rect,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            schema: Schema::parse("", std::path::PathBuf::from(".")).expect("an empty schema"),
            items: Vec::new(),
            groups: Vec::new(),
            rows: Vec::new(),
            columns: Vec::new(),
            group_of: Vec::new(),
            by_id: HashMap::new(),
            selected: 0,
            offset: 0,
            column: 0,
            column_row: 0,
            collapsed: HashSet::new(),
            filter: String::new(),
            query: Query::default(),
            input: String::new(),
            editing: None,
            group_by: "milestone".to_string(),
            sort: String::new(),
            view: None,
            show_closed: false,
            board: false,
            reading: false,
            read_scroll: 0,
            picker: None,
            confirm: None,
            help: false,
            diagnostics: false,
            mouse: true,
            loading: true,
            last_load: None,
            failure: None,
            watcher_alive: true,
            writable: true,
            warnings: Vec::new(),
            toast: None,
            now: unix_seconds(),
            theme: Theme::auto(true),
            keymap: Keymap::default(),
            list_area: Rect::default(),
            board_area: Rect::default(),
            should_quit: false,
        }
    }

    /// Replace the backlog, keeping the cursor on the same item where we can.
    pub fn ingest(&mut self, report: Report) {
        let anchor = self.selected_item().map(|i| i.id);

        self.schema = report.schema;
        self.items = report.items;
        self.warnings = report.warnings;
        self.by_id = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| (item.id, i))
            .collect();
        self.loading = false;
        self.failure = None;
        self.last_load = Some(Instant::now());
        self.check_settings();
        self.reparse_filter();
        self.rebuild();

        if let Some(id) = anchor {
            self.select_id(id);
        }
        self.clamp();
    }

    /// A view or a grouping named on the command line may not exist in this
    /// project. Say so and fall back, rather than showing an empty screen with
    /// a heading that claims a filter is in force.
    fn check_settings(&mut self) {
        if let Some(view) = self.view.clone()
            && self.schema.view(&view).is_none()
        {
            let known: Vec<&str> = self.schema.views.iter().map(|v| v.name.as_str()).collect();
            self.warn(if known.is_empty() {
                format!("this project has no saved views, so {view:?} is not one")
            } else {
                format!(
                    "no view called {view:?} — this project has {}",
                    known.join(", ")
                )
            });
            self.view = None;
        }
        let axes = self.grouping_axes();
        if !axes.contains(&self.group_by) {
            self.warn(format!(
                "nothing to group by called {:?} — try {}",
                self.group_by,
                axes.join(", ")
            ));
            self.group_by = axes.first().cloned().unwrap_or_else(|| "none".to_string());
        }
    }

    /// Move the cursor to an item by id, wherever it now is. What keeps the
    /// selection still while a status change rearranges everything around it.
    pub fn select_id(&mut self, id: u32) {
        let Some(&index) = self.by_id.get(&id) else {
            return;
        };
        if let Some(row) = self
            .rows
            .iter()
            .position(|r| matches!(r, Row::Item(i) if *i == index))
        {
            self.selected = row;
        }
        if let Some((c, r)) = self
            .columns
            .iter()
            .enumerate()
            .find_map(|(c, col)| col.items.iter().position(|i| *i == index).map(|r| (c, r)))
        {
            self.column = c;
            self.column_row = r;
        }
    }

    /// The load itself failed. Keep showing the last good backlog, marked stale.
    pub fn load_failed(&mut self, detail: String, transient: bool) {
        self.loading = false;
        let count = self.failure.as_ref().map(|f| f.count + 1).unwrap_or(1);
        self.failure = Some(Failure {
            detail,
            transient,
            since: Instant::now(),
            count,
        });
    }

    pub fn warn(&mut self, message: String) {
        diag::warn("app", message.clone());
        self.toast(message, ToastKind::Bad);
    }

    /// How many items are actually listed. Not the same as "not filtered out":
    /// a milestone that is a heading is not also a row, and a count that said
    /// otherwise would disagree with what the reader can see.
    pub fn shown(&self) -> usize {
        let heading = self.heading_type().map(str::to_string);
        self.items
            .iter()
            .filter(|i| self.visible(i))
            .filter(|i| heading.as_deref() != Some(i.kind.as_str()))
            .count()
    }

    /// True when what is on screen is older than it should be.
    pub fn is_stale(&self) -> bool {
        self.failure.is_some() || !self.watcher_alive
    }

    // ── Building what is on screen ───────────────────────────────────────────

    fn visible(&self, item: &Item) -> bool {
        if !self.show_closed && item.category.is_closed() {
            return false;
        }
        self.query.matches(item, &self.schema)
    }

    /// The value an item is grouped under, and what to call it.
    fn group_key(&self, item: &Item) -> String {
        match self.group_by.as_str() {
            "none" | "" => String::new(),
            "status" => item.status.clone(),
            "type" => item.kind.clone(),
            "category" => item.category.name().to_string(),
            "assignee" => item.assignee.clone().unwrap_or_default(),
            other => item.field_str(other).unwrap_or_default().to_string(),
        }
    }

    /// Where a group sits: `(class, text, number)`. Declared orders first — the
    /// status table, an enum's values, a milestone's due date — and alphabetical
    /// only where the project has expressed no opinion. A group for items with
    /// nothing set sorts last, because "not filled in" is not a stage of work.
    fn group_rank(&self, key: &str) -> (u8, String, u64) {
        if key.is_empty() {
            return (2, String::new(), 0);
        }
        let declared = |n: usize| (0u8, String::new(), n as u64);
        match self.group_by.as_str() {
            "status" => declared(self.schema.status_index(key)),
            "type" => declared(
                self.schema
                    .types
                    .iter()
                    .position(|t| t.name == key)
                    .unwrap_or(usize::MAX),
            ),
            "category" => declared(Category::from_name(key).map(|c| c as usize).unwrap_or(9)),
            _ => {
                if let Some(field) = self.schema.field(&self.group_by)
                    && !field.values.is_empty()
                {
                    return declared(field.rank(key));
                }
                // A reference: order by the item it names, and by its due date
                // first, so milestones read as a schedule rather than as an
                // index. One with no date sorts after every one that has one —
                // an undated milestone is not the next thing to do.
                if let Some(item) = self.item_named(key) {
                    let due = item.field_str("due").unwrap_or("9999-99-99").to_string();
                    return (1, due, item.id as u64);
                }
                (1, key.to_lowercase(), u64::MAX)
            }
        }
    }

    /// The item a group key names, for the reference fields where a group is
    /// itself an item: `v0.1` is a milestone with a title and a progress bar.
    pub fn item_named(&self, key: &str) -> Option<&Item> {
        self.items.iter().find(|i| {
            i.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(key))
                || key.parse::<u32>().is_ok_and(|id| id == i.id)
        })
    }

    /// Sort keys within a group. What the user asked for, or the order that
    /// answers "what should I look at first": furthest along, then most
    /// important, then oldest.
    fn sort_keys(&self) -> Vec<crate::filter::SortKey> {
        if !self.sort.trim().is_empty() {
            return crate::filter::parse_sort(&self.sort);
        }
        let mut spec = String::from("status");
        if self.schema.field("priority").is_some() {
            spec.push_str(",priority");
        }
        spec.push_str(",id");
        crate::filter::parse_sort(&spec)
    }

    /// The type whose items *are* the headings, when grouping by a reference
    /// that names them: grouping by milestone, the milestones are the headings
    /// rather than rows under one.
    fn heading_type(&self) -> Option<&str> {
        let field = self.schema.field(&self.group_by)?;
        let target = field.target.as_deref()?;
        self.schema.item_type(target).map(|t| t.name.as_str())
    }

    pub fn rebuild(&mut self) {
        let heading_type = self.heading_type().map(str::to_string);
        let keys: Vec<String> = self.items.iter().map(|i| self.group_key(i)).collect();
        let ranks: Vec<(u8, String, u64)> = keys.iter().map(|k| self.group_rank(k)).collect();
        let sort = self.sort_keys();

        let mut indices: Vec<usize> = (0..self.items.len())
            .filter(|i| self.visible(&self.items[*i]))
            .filter(|i| heading_type.as_deref() != Some(self.items[*i].kind.as_str()))
            .collect();
        indices.sort_by(|a, b| {
            ranks[*a]
                .cmp(&ranks[*b])
                .then_with(|| keys[*a].cmp(&keys[*b]))
                .then_with(|| self.compare(*a, *b, &sort))
        });

        self.groups.clear();
        self.rows.clear();
        self.group_of.clear();
        self.group_of.resize(self.items.len(), usize::MAX);

        let flat = matches!(self.group_by.as_str(), "none" | "");
        let mut current: Option<&str> = None;
        for i in indices.iter().copied() {
            let key = keys[i].as_str();
            if !flat && current != Some(key) {
                let (count, done, blocked) = self.tally(key, heading_type.as_deref());
                self.groups.push(Group {
                    key: key.to_string(),
                    label: self.group_label(key),
                    count,
                    done,
                    blocked,
                    shown: 0,
                    item: self
                        .item_named(key)
                        .and_then(|m| self.by_id.get(&m.id))
                        .copied(),
                });
                self.rows.push(Row::Group(self.groups.len() - 1));
                current = Some(key);
            }
            if !flat {
                let group_idx = self.groups.len() - 1;
                self.group_of[i] = group_idx;
                let g = self.groups.last_mut().expect("a group was pushed above");
                g.shown += 1;
            }
            if flat || !self.collapsed.contains(key) {
                self.rows.push(Row::Item(i));
            }
        }

        self.build_board(&indices);
        self.clamp();
    }

    /// What is under a heading, counting what the filter is hiding.
    fn tally(&self, key: &str, heading_type: Option<&str>) -> (usize, usize, usize) {
        let mut count = 0;
        let mut done = 0;
        let mut blocked = 0;
        for item in self.items.iter() {
            if heading_type == Some(item.kind.as_str()) || self.group_key(item) != key {
                continue;
            }
            count += 1;
            done += usize::from(item.category.is_closed());
            blocked += usize::from(item.blocked && !item.category.is_closed());
        }
        (count, done, blocked)
    }

    fn group_label(&self, key: &str) -> String {
        if key.is_empty() {
            return match self.group_by.as_str() {
                "status" | "type" => "unset".to_string(),
                "assignee" => "unassigned".to_string(),
                other => format!("no {other}"),
            };
        }
        match self.group_by.as_str() {
            "status" => self
                .schema
                .status(key)
                .map(|s| s.display().to_string())
                .unwrap_or_else(|| key.to_string()),
            "type" => self
                .schema
                .item_type(key)
                .map(|t| t.display().to_string())
                .unwrap_or_else(|| key.to_string()),
            _ => match self.item_named(key) {
                Some(item) => format!("{key}  {}", item.title),
                None => key.to_string(),
            },
        }
    }

    /// The board is the same set of items, dealt into the columns the project
    /// asked for. Items in a status with no column are not lost; they simply
    /// have nowhere to be, which `cairn board` does too.
    fn build_board(&mut self, indices: &[usize]) {
        self.columns = self
            .schema
            .board_statuses()
            .iter()
            .map(|s| Column {
                status: s.name.clone(),
                label: s.display().to_string(),
                items: Vec::new(),
            })
            .collect();
        for &i in indices {
            let status = &self.items[i].status;
            if let Some(column) = self.columns.iter_mut().find(|c| c.status == *status) {
                column.items.push(i);
            }
        }
    }

    fn compare(&self, a: usize, b: usize, keys: &[crate::filter::SortKey]) -> std::cmp::Ordering {
        for key in keys {
            let va = crate::filter::sort_value(&self.items[a], &self.schema, &key.field);
            let vb = crate::filter::sort_value(&self.items[b], &self.schema, &key.field);
            let order = if key.descending {
                vb.cmp(&va)
            } else {
                va.cmp(&vb)
            };
            if order != std::cmp::Ordering::Equal {
                return order;
            }
        }
        self.items[a].id.cmp(&self.items[b].id)
    }

    fn reparse_filter(&mut self) {
        // A saved view is a filter the project wrote down, ANDed with whatever
        // is in the box: picking "now" and then typing narrows it further.
        let mut text = self.filter.clone();
        if let Some(view) = self.view.as_deref().and_then(|v| self.schema.view(v))
            && let Some(filter) = &view.filter
        {
            text = if text.trim().is_empty() {
                filter.clone()
            } else {
                format!("{filter},{text}")
            };
        }
        self.query = Query::parse(&text, &self.schema);
    }

    // ── Moving about ─────────────────────────────────────────────────────────

    /// Group headers are only landable when collapsed — otherwise the cursor
    /// would stop on a row with nothing behind it in the detail pane.
    fn is_selectable(&self, idx: usize) -> bool {
        match self.rows.get(idx) {
            Some(Row::Item(_)) => true,
            Some(Row::Group(g)) => self
                .groups
                .get(*g)
                .is_some_and(|g| self.collapsed.contains(&g.key)),
            None => false,
        }
    }

    pub fn clamp(&mut self) {
        if self.rows.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.rows.len() - 1);
            if !self.is_selectable(self.selected)
                && let Some(next) = self.nearest_selectable(self.selected)
            {
                self.selected = next;
            }
        }

        if self.columns.is_empty() {
            self.column = 0;
            self.column_row = 0;
            return;
        }
        self.column = self.column.min(self.columns.len() - 1);
        let len = self.columns[self.column].items.len();
        self.column_row = self.column_row.min(len.saturating_sub(1));
    }

    fn nearest_selectable(&self, from: usize) -> Option<usize> {
        (from + 1..self.rows.len())
            .chain((0..from).rev())
            .find(|i| self.is_selectable(*i))
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.items.get(self.selected_index()?)
    }

    fn selected_index(&self) -> Option<usize> {
        if self.board {
            let column = self.columns.get(self.column)?;
            return column.items.get(self.column_row).copied();
        }
        match self.rows.get(self.selected)? {
            Row::Item(i) => Some(*i),
            // A collapsed heading that names an item selects that item, so a
            // milestone can be read and acted on without being a row of its own.
            Row::Group(g) => self.groups.get(*g).and_then(|g| g.item),
        }
    }

    pub fn move_by(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }
        if self.board {
            let Some(column) = self.columns.get(self.column) else {
                return;
            };
            if column.items.is_empty() {
                return;
            }
            let len = column.items.len() as isize;
            self.column_row = (self.column_row as isize + delta).rem_euclid(len) as usize;
            return;
        }
        if self.rows.is_empty() {
            return;
        }
        let len = self.rows.len() as isize;
        let step = delta.signum();
        let mut i = self.selected as isize;
        for _ in 0..delta.abs() {
            // Walk one landable row at a time, wrapping at the ends.
            for _ in 0..len {
                i = (i + step).rem_euclid(len);
                if self.is_selectable(i as usize) {
                    break;
                }
            }
        }
        self.selected = i as usize;
    }

    pub fn jump(&mut self, to_end: bool) {
        if self.board {
            let len = self
                .columns
                .get(self.column)
                .map(|c| c.items.len())
                .unwrap_or(0);
            self.column_row = if to_end { len.saturating_sub(1) } else { 0 };
            return;
        }
        if self.rows.is_empty() {
            return;
        }
        self.selected = if to_end { self.rows.len() - 1 } else { 0 };
        self.clamp();
    }

    /// Previous or next group — on the board, the column beside this one. The
    /// selection follows the item where it can, so stepping sideways past an
    /// empty column does not lose your place.
    pub fn step_group(&mut self, forward: bool) {
        if self.board {
            if self.columns.is_empty() {
                return;
            }
            let len = self.columns.len() as isize;
            let step = if forward { 1 } else { -1 };
            self.column = ((self.column as isize + step).rem_euclid(len)) as usize;
            let len = self.columns[self.column].items.len();
            self.column_row = self.column_row.min(len.saturating_sub(1));
            return;
        }
        let headers: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| matches!(r, Row::Group(_)))
            .map(|(i, _)| i)
            .collect();
        if headers.is_empty() {
            return;
        }
        let target = if forward {
            headers.iter().find(|h| **h > self.selected).copied()
        } else {
            headers
                .iter()
                .rev()
                .find(|h| **h < self.selected.saturating_sub(1))
                .copied()
        };
        let Some(header) = target.or_else(|| headers.first().copied()) else {
            return;
        };
        self.selected = header;
        if !self.is_selectable(header) {
            self.selected = header + 1;
        }
        self.clamp();
    }

    pub fn toggle_group(&mut self) {
        let key = match self.rows.get(self.selected) {
            Some(Row::Group(g)) => self.groups[*g].key.clone(),
            Some(Row::Item(i)) => self.group_key(&self.items[*i]),
            None => return,
        };
        let collapsing = !self.collapsed.contains(&key);
        if collapsing {
            self.collapsed.insert(key.clone());
        } else {
            self.collapsed.remove(&key);
        }
        self.rebuild();

        // Keep the cursor on the group we just acted on: its header when
        // collapsed, its first item when expanded.
        if let Some(header) = self
            .rows
            .iter()
            .position(|r| matches!(r, Row::Group(g) if self.groups[*g].key == key))
        {
            self.selected = if collapsing { header } else { header + 1 };
            self.clamp();
        }
    }

    /// The next axis to group by: whatever the project actually has.
    pub fn grouping_axes(&self) -> Vec<String> {
        let mut axes = vec![
            "milestone".to_string(),
            "status".to_string(),
            "type".to_string(),
        ];
        for field in self.schema.groupable_fields() {
            if !axes.contains(&field.name) {
                axes.push(field.name.clone());
            }
        }
        axes.retain(|a| a == "status" || a == "type" || self.schema.field(a).is_some());
        if self.items.iter().any(|i| i.assignee.is_some()) {
            axes.push("assignee".to_string());
        }
        axes.push("none".to_string());
        axes
    }

    pub fn cycle_grouping(&mut self) {
        let axes = self.grouping_axes();
        let at = axes.iter().position(|a| *a == self.group_by).unwrap_or(0);
        self.group_by = axes[(at + 1) % axes.len()].clone();
        let id = self.selected_item().map(|i| i.id);
        self.rebuild();
        if let Some(id) = id {
            self.select_id(id);
        }
        let axis = self.group_by.clone();
        self.toast(format!("grouped by {axis}"), ToastKind::Info);
    }

    // ── Changing an item ─────────────────────────────────────────────────────

    /// Build a `cairn` invocation for the selected item.
    fn change(&self, args: Vec<String>, describe: String, undo: Option<String>) -> Action {
        if !self.writable {
            return Action::None;
        }
        Action::Write(Change {
            args,
            describe,
            undo,
        })
    }

    fn refuse_readonly(&mut self) {
        self.toast(
            "cairn is not on PATH — harrow can read this backlog but not change it",
            ToastKind::Bad,
        );
    }

    fn set_field(&mut self, field: &str, value: &str) -> Action {
        let Some(item) = self.selected_item() else {
            return Action::None;
        };
        let (id, was) = (item.id, current_value(item, field));
        if was.as_deref() == Some(value) {
            self.toast(format!("{field} is already {value}"), ToastKind::Info);
            return Action::None;
        }
        let reference = self.schema.format_id(id);
        let shown = if value.is_empty() { "cleared" } else { value };
        self.change(
            vec!["set".into(), id.to_string(), format!("{field}={value}")],
            format!("{reference} {field} → {shown}"),
            was.map(|w| format!("cairn set {id} {field}={w}")),
        )
    }

    pub fn claim(&mut self, take: bool) -> Action {
        let Some(item) = self.selected_item() else {
            return Action::None;
        };
        let (id, reference) = (item.id, self.schema.format_id(item.id));
        if take {
            self.change(
                vec!["claim".into(), id.to_string()],
                format!("{reference} claimed"),
                Some(format!("cairn release {id}")),
            )
        } else {
            self.change(
                vec!["release".into(), id.to_string()],
                format!("{reference} released"),
                Some(format!("cairn claim {id}")),
            )
        }
    }

    /// Closing is the one change that asks first. Not because it cannot be
    /// undone — `u` reopens — but because it is a declaration that something is
    /// finished, and it runs the project's hooks.
    pub fn ask_close(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        if item.category == Category::Done {
            self.toast("already closed", ToastKind::Info);
            return;
        }
        let (id, title) = (item.id, item.title.clone());
        let reference = self.schema.format_id(id);
        let (met, total) = item.criteria();
        let detail = if total > 0 && met < total {
            format!("{} of {total} acceptance criteria are ticked", met)
        } else {
            title
        };
        self.confirm = Some(Confirm {
            prompt: format!("Close {reference}?"),
            detail,
            change: Change {
                args: vec!["close".into(), id.to_string()],
                describe: format!("{reference} closed"),
                undo: Some(format!("cairn reopen {id}")),
            },
        });
    }

    pub fn resolve_confirm(&mut self, go: bool) -> Action {
        let Some(c) = self.confirm.take() else {
            return Action::None;
        };
        if !go {
            return Action::None;
        }
        if !self.writable {
            self.refuse_readonly();
            return Action::None;
        }
        Action::Write(c.change)
    }

    /// One status forward or back through the project's own order. Statuses
    /// with no board column are skipped: they are not part of the path a
    /// project laid out, and stepping into one is never what was meant.
    pub fn step_status(&mut self, forward: bool) -> Action {
        let Some(item) = self.selected_item() else {
            return Action::None;
        };
        let path: Vec<String> = self
            .schema
            .board_statuses()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        if path.is_empty() {
            return Action::None;
        }
        let at = path.iter().position(|s| *s == item.status);
        let next = match (at, forward) {
            (Some(i), true) if i + 1 < path.len() => i + 1,
            (Some(i), false) if i > 0 => i - 1,
            (Some(_), _) => {
                self.toast(
                    if forward {
                        "already at the last status"
                    } else {
                        "already at the first status"
                    },
                    ToastKind::Info,
                );
                return Action::None;
            }
            (None, _) => 0,
        };
        let value = path[next].clone();
        self.set_field("status", &value)
    }

    pub fn open_picker(&mut self, field: &str) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let (id, current) = (item.id, current_value(item, field).unwrap_or_default());

        let mut options: Vec<(String, String, String)> = Vec::new();
        match field {
            "status" => {
                for status in &self.schema.statuses {
                    options.push((
                        status.name.clone(),
                        status.display().to_string(),
                        status.category.name().to_string(),
                    ));
                }
            }
            "milestone" => {
                for milestone in self.items.iter().filter(|i| i.is_milestone()) {
                    let Some(key) = &milestone.key else { continue };
                    let note = match milestone.progress() {
                        Some(p) => format!("{p}%"),
                        None => String::new(),
                    };
                    options.push((key.clone(), milestone.title.clone(), note));
                }
                options.push((String::new(), "— none —".to_string(), String::new()));
            }
            other => {
                let Some(schema_field) = self.schema.field(other) else {
                    self.toast(format!("this project has no {other}"), ToastKind::Info);
                    return;
                };
                if schema_field.values.is_empty() {
                    self.toast(
                        format!("{other} is not a list to choose from — use `e` to edit"),
                        ToastKind::Info,
                    );
                    return;
                }
                for value in &schema_field.values {
                    options.push((value.clone(), value.clone(), String::new()));
                }
                options.push((String::new(), "— none —".to_string(), String::new()));
            }
        }

        let selected = options
            .iter()
            .position(|(v, _, _)| *v == current)
            .unwrap_or(0);
        self.picker = Some(Picker {
            title: format!("{field} for {}", self.schema.format_id(id)),
            options,
            selected,
            field: field.to_string(),
            id,
        });
    }

    fn resolve_picker(&mut self, go: bool) -> Action {
        let Some(picker) = self.picker.take() else {
            return Action::None;
        };
        if !go {
            return Action::None;
        }
        let Some((value, _, _)) = picker.options.get(picker.selected).cloned() else {
            return Action::None;
        };
        self.select_id(picker.id);
        self.set_field(&picker.field, &value)
    }

    fn submit_input(&mut self) -> Action {
        let editing = self.editing.take();
        let text = std::mem::take(&mut self.input);
        match editing {
            Some(Editing::Filter) => {
                self.filter = text;
                self.reparse_filter();
                self.rebuild();
                Action::None
            }
            Some(Editing::NewItem) => {
                let title = text.trim().to_string();
                if title.is_empty() {
                    return Action::None;
                }
                if !self.writable {
                    self.refuse_readonly();
                    return Action::None;
                }
                Action::Write(Change {
                    args: vec!["new".into(), title.clone()],
                    describe: format!("created “{title}”"),
                    undo: None,
                })
            }
            None => Action::None,
        }
    }

    pub fn toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toast = Some((msg.into(), kind, Instant::now()));
    }

    /// Called once per frame by the shell.
    pub fn tick_clock(&mut self) {
        self.now = unix_seconds();
    }

    pub fn expire_toast(&mut self) {
        if let Some((_, _, at)) = &self.toast
            && at.elapsed() > Duration::from_millis(3200)
        {
            self.toast = None;
        }
    }

    // ── Input ────────────────────────────────────────────────────────────────

    /// The whole keyboard interface. Modal layers get first refusal, in order,
    /// and only then does the keymap get a look.
    pub fn handle_key(&mut self, code: KeyCode, mods: KeyModifiers) -> Action {
        if self.confirm.is_some() {
            return match code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.resolve_confirm(true)
                }
                _ => self.resolve_confirm(false),
            };
        }
        if self.picker.is_some() {
            let len = self.picker.as_ref().map(|p| p.options.len()).unwrap_or(0);
            return match code {
                KeyCode::Enter => self.resolve_picker(true),
                KeyCode::Esc => self.resolve_picker(false),
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(p) = self.picker.as_mut()
                        && len > 0
                    {
                        p.selected = (p.selected + 1) % len;
                    }
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(p) = self.picker.as_mut()
                        && len > 0
                    {
                        p.selected = (p.selected + len - 1) % len;
                    }
                    Action::None
                }
                // A digit picks outright, which makes `s 2` the whole gesture.
                // A letter steps to the next option beginning with it, so three
                // statuses starting with `d` are three presses rather than an
                // ambiguity.
                KeyCode::Char(c) => {
                    if let Some(p) = self.picker.as_mut() {
                        if let Some(n) = c.to_digit(10).filter(|n| *n >= 1) {
                            let at = n as usize - 1;
                            if at < p.options.len() {
                                p.selected = at;
                            }
                        } else if let Some(at) = (1..=p.options.len())
                            .map(|step| (p.selected + step) % p.options.len())
                            .find(|i| {
                                p.options[*i]
                                    .0
                                    .to_lowercase()
                                    .starts_with(c.to_ascii_lowercase())
                            })
                        {
                            p.selected = at;
                        }
                    }
                    Action::None
                }
                _ => Action::None,
            };
        }
        if self.editing.is_some() {
            match code {
                KeyCode::Esc => {
                    let was = self.editing.take();
                    self.input.clear();
                    if was == Some(Editing::Filter) {
                        self.filter.clear();
                        self.reparse_filter();
                        self.rebuild();
                    }
                }
                KeyCode::Enter => return self.submit_input(),
                KeyCode::Backspace => {
                    self.input.pop();
                    self.preview_filter();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    self.preview_filter();
                }
                _ => {}
            }
            return Action::None;
        }
        if self.reading {
            match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.read_scroll = self.read_scroll.saturating_add(1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.read_scroll = self.read_scroll.saturating_sub(1)
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    self.read_scroll = self.read_scroll.saturating_add(10)
                }
                KeyCode::PageUp => self.read_scroll = self.read_scroll.saturating_sub(10),
                // Everything else leaves, including the key that opened it.
                _ => {
                    self.reading = false;
                    self.read_scroll = 0;
                }
            }
            return Action::None;
        }
        if self.help || self.diagnostics {
            self.help = false;
            self.diagnostics = false;
            return Action::None;
        }

        match self.keymap.lookup(code, mods) {
            Some(command) => self.run(command),
            None => Action::None,
        }
    }

    /// The list narrows as you type, so a filter is something you watch rather
    /// than something you submit and hope about.
    fn preview_filter(&mut self) {
        if self.editing == Some(Editing::Filter) {
            self.filter = self.input.clone();
            self.reparse_filter();
            self.rebuild();
        }
    }

    /// Perform a named command. Everything with an effect outside the process
    /// leaves here as an `Action` for the shell to carry out.
    pub fn run(&mut self, command: Command) -> Action {
        // One place, rather than a guard in each of eight arms.
        let writes = matches!(
            command,
            Command::Claim
                | Command::Release
                | Command::Close
                | Command::Reopen
                | Command::New
                | Command::Status
                | Command::Priority
                | Command::Milestone
                | Command::Advance
                | Command::Retreat
        );
        if writes && !self.writable {
            self.refuse_readonly();
            return Action::None;
        }

        match command {
            Command::Quit => return Action::Quit,
            Command::Back => {
                // Backing out closes what is open; with nothing open it does
                // nothing, rather than quitting out from under you.
                if self.view.is_some() {
                    self.view = None;
                    self.reparse_filter();
                    self.rebuild();
                    self.toast("view cleared", ToastKind::Info);
                } else if !self.filter.is_empty() {
                    self.filter.clear();
                    self.reparse_filter();
                    self.rebuild();
                    self.toast("filter cleared", ToastKind::Info);
                }
            }
            Command::Down => self.move_by(1),
            Command::Up => self.move_by(-1),
            Command::PageDown => self.move_by(10),
            Command::PageUp => self.move_by(-10),
            Command::First => self.jump(false),
            Command::Last => self.jump(true),
            Command::ToggleGroup => {
                if !self.board {
                    self.toggle_group()
                }
            }
            Command::PrevGroup => self.step_group(false),
            Command::NextGroup => self.step_group(true),
            Command::ViewBoard => {
                let id = self.selected_item().map(|i| i.id);
                self.board = !self.board;
                if let Some(id) = id {
                    self.select_id(id);
                }
                self.clamp();
            }
            Command::GroupBy => {
                if self.board {
                    self.toast("the board is grouped by status", ToastKind::Info);
                } else {
                    self.cycle_grouping();
                }
            }
            Command::Read => {
                if self.selected_item().is_some() {
                    self.reading = true;
                    self.read_scroll = 0;
                }
            }
            Command::Edit => {
                if let Some(item) = self.selected_item() {
                    return Action::Edit(item.path.clone());
                }
            }
            Command::Claim => return self.claim(true),
            Command::Release => return self.claim(false),
            Command::Close => self.ask_close(),
            Command::Reopen => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                if !item.category.is_closed() {
                    self.toast("that one is already open", ToastKind::Info);
                    return Action::None;
                }
                let (id, reference) = (item.id, self.schema.format_id(item.id));
                return self.change(
                    vec!["reopen".into(), id.to_string()],
                    format!("{reference} reopened"),
                    Some(format!("cairn close {id}")),
                );
            }
            Command::New => {
                self.editing = Some(Editing::NewItem);
                self.input.clear();
            }
            Command::Status => self.open_picker("status"),
            Command::Priority => self.open_picker("priority"),
            Command::Milestone => self.open_picker("milestone"),
            Command::Advance => return self.step_status(true),
            Command::Retreat => return self.step_status(false),
            Command::Copy => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                return Action::Copy(self.schema.format_id(item.id));
            }
            Command::Filter => {
                self.editing = Some(Editing::Filter);
                self.input = self.filter.clone();
            }
            Command::ToggleClosed => {
                self.show_closed = !self.show_closed;
                self.rebuild();
                let msg = if self.show_closed {
                    "showing finished items"
                } else {
                    "hiding finished items"
                };
                self.toast(msg, ToastKind::Info);
            }
            Command::Refresh => {
                self.loading = true;
                return Action::Refresh;
            }
            Command::Reload => return Action::Reload,
            Command::Diagnostics => self.diagnostics = true,
            Command::Help => self.help = true,
            Command::ToggleMouse => {
                self.mouse = !self.mouse;
                let msg = if self.mouse {
                    "mouse capture on"
                } else {
                    "mouse capture off — text is selectable"
                };
                self.toast(msg, ToastKind::Info);
                return Action::SetMouse(self.mouse);
            }
        }
        Action::None
    }

    /// Which row a click landed on, if any.
    pub fn row_at(&self, column: u16, row: u16) -> Option<usize> {
        let area = self.list_area;
        let inside = column > area.x
            && column < area.x + area.width.saturating_sub(1)
            && row > area.y
            && row < area.y + area.height.saturating_sub(1);
        if !inside {
            return None;
        }
        let idx = self.offset + usize::from(row - area.y - 1);
        (idx < self.rows.len()).then_some(idx)
    }

    /// Clicking an item selects it; clicking a group header folds it.
    pub fn click_row(&mut self, idx: usize) {
        match self.rows.get(idx) {
            Some(Row::Item(_)) => self.selected = idx,
            Some(Row::Group(_)) => {
                self.selected = idx;
                self.toggle_group();
            }
            None => {}
        }
    }

    pub fn handle_mouse(&mut self, m: MouseEvent) -> Action {
        match m.kind {
            MouseEventKind::ScrollDown => self.move_by(1),
            MouseEventKind::ScrollUp => self.move_by(-1),
            MouseEventKind::Down(MouseButton::Left) => {
                if !self.board
                    && let Some(idx) = self.row_at(m.column, m.row)
                {
                    self.click_row(idx);
                }
            }
            _ => {}
        }
        Action::None
    }

    /// Everything that must be true of the state after any sequence of inputs.
    /// Asserted by the randomised tests, and cheap enough to call anywhere.
    pub fn check_invariants(&self) -> Result<(), String> {
        if self.rows.is_empty() {
            if self.selected != 0 {
                return Err(format!("selection {} with no rows", self.selected));
            }
        } else {
            if self.selected >= self.rows.len() {
                return Err(format!(
                    "selection {} is past the last row {}",
                    self.selected,
                    self.rows.len() - 1
                ));
            }
            if !self.is_selectable(self.selected) {
                return Err(format!(
                    "selection {} rests on an expanded group header",
                    self.selected
                ));
            }
        }
        for row in &self.rows {
            match row {
                Row::Item(i) if *i >= self.items.len() => {
                    return Err(format!("row points at missing item {i}"));
                }
                Row::Group(g) if *g >= self.groups.len() => {
                    return Err(format!("row points at missing group {g}"));
                }
                _ => {}
            }
        }
        if self.column > self.columns.len() && !self.columns.is_empty() {
            return Err(format!("column {} does not exist", self.column));
        }
        if let Some(column) = self.columns.get(self.column)
            && !column.items.is_empty()
            && self.column_row >= column.items.len()
        {
            return Err(format!(
                "board row {} is past the end of {}",
                self.column_row, column.status
            ));
        }
        let heading_type = self.heading_type().map(str::to_string);
        let visible = self
            .items
            .iter()
            .filter(|i| self.visible(i))
            .filter(|i| heading_type.as_deref() != Some(i.kind.as_str()))
            .count();
        if !matches!(self.group_by.as_str(), "none" | "") {
            let counted: usize = self.groups.iter().map(|g| g.shown).sum();
            if counted != visible {
                return Err(format!(
                    "group counts total {counted} but {visible} items are visible"
                ));
            }
        }
        let on_board: usize = self.columns.iter().map(|c| c.items.len()).sum();
        if on_board > visible {
            return Err(format!(
                "the board shows {on_board} items but only {visible} are visible"
            ));
        }
        Ok(())
    }
}

/// The value a field currently holds, for the fields a change can name.
fn current_value(item: &Item, field: &str) -> Option<String> {
    match field {
        "status" => Some(item.status.clone()),
        "type" => Some(item.kind.clone()),
        "assignee" => item.assignee.clone(),
        other => item.field_str(other).map(str::to_string),
    }
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
