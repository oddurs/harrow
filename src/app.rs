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

/// One item changing, once, as the repository recorded it.
///
/// The unit is a change rather than a commit, because the reader is asking
/// about items: a commit that moves four of them is four lines.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Moment {
    pub id: u32,
    pub when: String,
    pub who: String,
    pub what: String,
}

/// Which way of looking at a backlog is on screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Pane {
    /// Everything addressed to a person: proposals to decide, claims gone
    /// cold, work finished but not closed. First in the order, because it is
    /// the first question somebody sitting down asks — but not the default,
    /// because opening on *nothing needs you* would spend the first screen
    /// saying nothing.
    Needs,
    #[default]
    List,
    Board,
    Stats,
    /// What changed across the project, most recent first, out of the git
    /// that is already underneath it.
    Log,
}

impl Pane {
    pub const ALL: [Pane; 5] = [Pane::Needs, Pane::List, Pane::Board, Pane::Stats, Pane::Log];

    pub fn name(self) -> &'static str {
        match self {
            Pane::Needs => "needs",
            Pane::Log => "log",
            Pane::List => "list",
            Pane::Board => "board",
            Pane::Stats => "stats",
        }
    }

    pub fn from_name(s: &str) -> Option<Pane> {
        Pane::ALL
            .into_iter()
            .find(|p| p.name() == s.trim().to_lowercase())
    }

    fn next(self) -> Pane {
        let at = Pane::ALL.iter().position(|p| *p == self).unwrap_or(0);
        Pane::ALL[(at + 1) % Pane::ALL.len()]
    }

    fn previous(self) -> Pane {
        let at = Pane::ALL.iter().position(|p| *p == self).unwrap_or(0);
        Pane::ALL[(at + Pane::ALL.len() - 1) % Pane::ALL.len()]
    }
}

/// Something on screen you can click.
///
/// The drawing code records where each of these ended up as it draws, and the
/// mouse handler looks up what is under the pointer. Keeping the map in one
/// place is what makes the interface mouse-first rather than mouse-tolerant:
/// anything drawn can be made clickable where it is drawn, instead of every
/// region's geometry being recomputed by hand in the input handler.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Hit {
    /// One of the three panes, in the header.
    Tab(Pane),
    /// A cell of the status strip. Clicking filters by it.
    Status(String),
    /// A row of the list, by index into `rows`.
    Row(usize),
    /// A board column heading.
    Column(usize),
    /// A card on the board: column, then position within it.
    Card(usize, usize),
    /// An option in the open picker.
    Option(usize),
    /// Yes or no, in the open confirmation.
    Answer(bool),
    /// A key hint in the footer, or a button in an overlay.
    Run(Command),
    /// The body of the detail pane, which scrolls on its own.
    Detail,
    /// An open overlay's own frame. Inert: it is there so that a click on the
    /// text you are reading is not also a click on whatever it covers.
    Overlay,
    /// The reader panel, which scrolls on its own beside the lens.
    Reader,
    /// The filter panel's own frame, so a click in its whitespace focuses it
    /// rather than reaching the lens behind.
    Filter,
    /// One value in the filter panel, by index over values alone.
    Facet(usize),
    /// A figure on the stats pane, by index into `doors`.
    Figure(usize),
    /// A question on the needs-you lens, by index into `questions`.
    Question(usize),
    /// A change on the log lens, by index into `moments`.
    Moment(usize),
    /// Something in the detail pane that leads somewhere, by index into
    /// `links`.
    Link(usize),
}

/// What is on screen and why. See [`App::view_line`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewLine {
    /// The saved view in force, where the reader chose one by name.
    pub view: Option<String>,
    /// The filter, in the grammar that would reproduce it.
    pub query: String,
    /// The sort keys, as `--sort` would take them.
    pub sort: String,
    /// Whether that sort is the project's default rather than a choice.
    pub sort_is_default: bool,
    pub group_by: String,
    pub shown_and_total: (usize, usize),
}

/// Quote a word for a shell, only where it needs it.
///
/// A filter is usually one bare word, and quoting it anyway makes the line
/// look harder than it is. A comma is safe bare and separates clauses in
/// nearly every filter there is; `|`, `<`, `>` and `!` are not, and `!`
/// least of all — interactive bash would expand it.
fn shell_word(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.=/:,+@".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

/// What an ordinary listing is leaving out. See [`App::hidden`].
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Hidden {
    /// Finished work, hidden until it is asked for.
    pub closed: usize,
    /// Open containers, hidden until they are asked for.
    pub containers: usize,
    /// What the project calls them, where they are all of one type.
    pub kind: Option<String>,
}

impl Hidden {
    pub fn any(&self) -> bool {
        self.closed > 0 || self.containers > 0
    }

    /// `7 milestones`, or `7 of them` where the project has more than one
    /// kind of container open.
    pub fn containers_said(&self) -> String {
        let n = self.containers;
        match &self.kind {
            Some(kind) if n == 1 => format!("1 {kind}"),
            Some(kind) => format!("{n} {kind}s"),
            None => format!("{n} of them"),
        }
    }
}

/// Where the work beneath a container stands. `blocked` overlaps the others:
/// a blocked item is still open or active, and saying so twice is the point —
/// it is both a thing not done and a thing nobody can do.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rollup {
    pub done: u32,
    pub active: u32,
    pub open: u32,
    pub blocked: u32,
}

/// Where something in a body or a field leads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Target {
    /// A URL written in the body, to be handed to the system.
    Url(String),
    /// Another item — a blocker, or a reference to one.
    Item(u32),
}

/// Something addressed to a person.
///
/// Ranked by what is actually waiting on a human rather than by severity in
/// the abstract: a proposal has somebody blocked on an answer, a cold claim
/// has work nobody is doing, finished-but-open is only untidy, and a missing
/// owner is a question about later. A queue that cries wolf is a queue
/// nobody reads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Asking {
    /// A program asked for a change only a person decides.
    Proposal {
        field: String,
        to: String,
        by: String,
    },
    /// Taken, and not honoured for longer than the project allows.
    ColdClaim { who: String, days: u64 },
    /// Every acceptance criterion is ticked and it is still open.
    Finished,
    /// A container with nothing unfinished under it, still open.
    ///
    /// Reported, never acted on — which is cairn's position and the right
    /// one: closing it automatically would be deciding that finished work
    /// means a shipped release, and those are different claims. A `later` or
    /// `Someday` milestone can have every item under it done and be meant to
    /// stay open for good. The judgement lives with the person, so this says
    /// what it can see and leaves the key to them.
    NothingUnfinished,
    /// Something filed it and nobody is answerable for it.
    Unowned { by: String },
}

impl Asking {
    /// What is waiting first. Lower sorts sooner.
    fn urgency(&self) -> u8 {
        match self {
            Asking::Proposal { .. } => 0,
            Asking::ColdClaim { .. } => 1,
            Asking::Finished => 2,
            Asking::NothingUnfinished => 3,
            Asking::Unowned { .. } => 4,
        }
    }

    /// The question, as a person would ask it.
    pub fn question(&self, reference: &str) -> String {
        match self {
            Asking::Proposal { field, to, by } => {
                format!("{by} asks that {reference} {field} become {to}")
            }
            Asking::ColdClaim { who, days } => {
                format!("{who} has held {reference} for {days} days")
            }
            Asking::Finished => format!("{reference} is all ticked and still open"),
            // cairn's own phrase for it. How much is under there is the
            // detail pane's rollup to say; this is the queue, and a queue
            // says what is waiting rather than how much of it there was.
            Asking::NothingUnfinished => format!("{reference} has nothing unfinished"),
            Asking::Unowned { by } => format!("{by} filed {reference} and nobody owns it"),
        }
    }

    /// What the keys will do, said before they are pressed.
    pub fn answers(&self) -> &'static str {
        match self {
            Asking::Proposal { .. } => "A accepts",
            Asking::ColdClaim { .. } => "C hands it back",
            Asking::Finished => "x closes it",
            Asking::NothingUnfinished => "x when it has shipped",
            Asking::Unowned { .. } => "c takes it",
        }
    }
}

/// A question and the item it is about.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Question {
    pub id: u32,
    pub asking: Asking,
}

/// What a figure on the stats pane stands for.
///
/// Every number on that pane is the answer to a question, and a reader's next
/// thought after each of them is *show me*. A door is the way back to the
/// items the figure counted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Door {
    /// The filter that produces the set this figure counted.
    Filter(String),
    /// One item, where the figure named one.
    Item(u32),
}

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
    /// Ask cairn how an item got the way it is.
    History(u32),
    /// Ask cairn whether the project is valid against its own schema.
    Check,
    /// Ask the repository what has changed across the whole backlog.
    Activity,
    /// Ask cairn to change something.
    Write(Change),
    /// Hand a URL to whatever the system opens URLs with.
    Open(String),
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
    /// The value this column stands for, in whatever the board is grouped by.
    /// Dropping a card here sets that field to this.
    pub value: String,
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

/// Why a backlog cannot be changed from here.
///
/// Both are legitimate ways to be reading one, and neither is an error — but
/// they ask different things of the reader, so the reason travels rather than
/// a flag that would have to be interpreted at every place it is shown.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ReadOnly {
    /// No `cairn` to run. A backlog is a directory of Markdown, so reading it
    /// needs nothing; changing it needs the tool that owns the lock.
    NoCairn,
    /// The project is written in a format harrow does not know. Its items
    /// still read — no format bump has ever changed what a key means — but
    /// what harrow understands of the schema may be short of what is there,
    /// and cairn will refuse the write itself until the project is migrated.
    Format(u32),
}

impl ReadOnly {
    /// For the edge of the screen, where there is room for a few words.
    pub fn briefly(&self) -> String {
        match self {
            ReadOnly::NoCairn => "read-only".to_string(),
            ReadOnly::Format(n) => format!("read-only · format {n}"),
        }
    }

    /// For a toast and the diagnostics, where there is room to say what to do.
    pub fn at_length(&self) -> String {
        match self {
            ReadOnly::NoCairn => {
                "cairn is not on PATH — harrow can read this backlog but not change it".to_string()
            }
            ReadOnly::Format(n) => format!(
                "this project is cairn format {n} and harrow reads {} —                  it reads but will not write; `cairn migrate` brings it up to date",
                crate::schema::KNOWN_FORMAT
            ),
        }
    }
}

pub struct Confirm {
    pub prompt: String,
    pub detail: String,
    pub change: Change,
}

/// Which pane the keyboard is driving.
///
/// The reader is a panel rather than a popover, so it is not modal and never
/// takes the keys by existing. It takes them by being focused, which `↵` does
/// and `esc` undoes — and the panel stays open across that, so moving through
/// the backlog with it open is still a matter of `esc j j ↵`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Focus {
    #[default]
    List,
    Reader,
    Filter,
}

/// One thing the backlog can be narrowed by, and what it takes.
///
/// Built from the project's own schema rather than from a list here: a project
/// that tracks `component` and `severity` gets those, and harrow has never
/// heard of either.
#[derive(Clone, Debug)]
pub struct Facet {
    pub field: String,
    pub label: String,
    pub values: Vec<FacetValue>,
}

/// One value of one facet, and what ticking it would leave you.
#[derive(Clone, Debug)]
pub struct FacetValue {
    pub value: String,
    pub label: String,
    /// What you would have if this were ticked, given everything ticked
    /// elsewhere. Nought is shown rather than hidden: *there are no p0s* is an
    /// answer, and an absence is not.
    pub count: usize,
    pub ticked: bool,
    /// The role this value paints in, where the theme names one — a status is
    /// its status colour here as much as anywhere else.
    pub role: FacetRole,
}

/// What a value is, so the panel can paint it the way the rest of harrow does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FacetRole {
    Status,
    Type,
    Rank,
    Plain,
}

/// How far the detail pane is scrolled, and which item that belongs to.
///
/// The offset belongs to the item rather than to the pane: selecting something
/// else starts at the top of it, rather than partway down whatever was under
/// the pointer a moment ago — and stepping back onto the one you were reading
/// returns you to where you had got to. Keeping the item beside the offset is
/// what makes that a read, rather than a reset every move of the cursor has to
/// remember to do.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct DetailScroll {
    at: u16,
    of: Option<u32>,
}

impl DetailScroll {
    pub fn at(self, item: u32) -> u16 {
        if self.of == Some(item) { self.at } else { 0 }
    }

    /// Straight to an offset. `u16::MAX` means the bottom: the draw is the
    /// only place that knows how tall the content came out, and it clamps.
    pub fn to(&mut self, item: u32, at: u16) {
        self.of = Some(item);
        self.at = at;
    }

    pub fn by(&mut self, item: u32, delta: isize) {
        let from = self.at(item);
        self.of = Some(item);
        self.at = from.saturating_add_signed(delta.clamp(-64, 64) as i16);
    }

    /// Pulled back to what the pane can actually show. The draw is the only
    /// place that knows how tall the content came out, so it is the place that
    /// says how far down is too far.
    pub fn clamp(&mut self, max: u16) {
        self.at = self.at.min(max);
    }
}

/// What the repository remembers about one item.
pub struct History {
    pub id: u32,
    pub lines: Vec<String>,
    pub scroll: u16,
    /// Set when there is no history to be had rather than none recorded — not
    /// a git repository, or no cairn to ask. The difference matters: one is a
    /// new item and the other is a missing tool.
    pub unavailable: Option<String>,
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
    /// What the project says a program may do with this field. Shown rather
    /// than enforced: the write goes through cairn, which decides. Showing it
    /// where the choice is made is what keeps a refusal from being a surprise.
    pub permission: crate::schema::Agent,
    /// Whether Enter proposes the choice instead of making it.
    pub propose: bool,
    /// Whether the options are acceptance criteria, chosen to be ticked.
    pub tick: bool,
}

/// A line of text being typed: the filter box, or a new item's title.
#[derive(PartialEq, Eq)]
pub enum Editing {
    Filter,
    NewItem,
    /// Why a change is being proposed rather than made.
    Why,
    /// A sentence to append to an item's body.
    Note,
    /// Why an item is being handed back, which cairn records as a note and
    /// shows to whoever takes it next.
    Reason,
}

pub struct App {
    pub schema: Schema,
    pub items: Vec<Item>,
    pub groups: Vec<Group>,
    pub rows: Vec<Row>,
    pub columns: Vec<Column>,
    /// Everything addressed to a person, worked out with the rest of the
    /// derived set so that every lens agrees about it.
    pub questions: Vec<Question>,
    /// Which question the cursor is on. Moved through `select_question`, so
    /// that it and the anchor cannot disagree.
    question: usize,
    /// What was selected on arriving here, when this lens cannot show it.
    ///
    /// A queue of questions holds only the items that raise one, so passing
    /// through it must not lose your place the way passing through the board
    /// does not lose an item the board has no column for. Moving the cursor
    /// here is a choice and replaces it.
    needs_anchor: Option<u32>,
    /// Which group each item belongs to, by item index.
    group_of: Vec<usize>,
    /// id → index, so a write can find what it changed without a linear scan.
    by_id: HashMap<u32, usize>,

    pub selected: usize,
    pub offset: usize,
    pub column: usize,
    pub column_row: usize,
    /// Where each board column has been scrolled to, by column. A column is a
    /// pane: what you can see in it is nobody else's business, including the
    /// column the cursor happens to be in.
    pub column_offsets: Vec<usize>,
    pub collapsed: HashSet<String>,
    /// Items marked for the next change, by id.
    ///
    /// Triage is one keystroke per item until the same decision applies to
    /// forty of them, at which point it is one decision and forty keystrokes.
    pub marked: HashSet<u32>,

    pub filter: String,
    pub query: Query,
    /// Whether the filter panel is open.
    pub filtering: bool,
    /// What the project can be narrowed by, rebuilt with the backlog.
    pub facets: Vec<Facet>,
    /// Which value the panel's cursor is on, counted over values alone —
    /// a heading is not somewhere the cursor can be.
    pub facet: usize,
    pub facet_scroll: u16,
    pub input: String,
    pub editing: Option<Editing>,
    pub group_by: String,
    /// What the board's columns are, which is not what the list's groups are.
    ///
    /// Grouping is arrangement, and arrangement is the one thing a lens is
    /// allowed to differ in. A list by milestone beside a board by status is
    /// a useful pair, and sharing one axis would cost whichever of the two
    /// you were not looking at.
    pub board_by: String,
    /// What the grouping was before a view replaced it. A view's grouping is
    /// a starting point rather than a cage: leaving the view puts back what
    /// you had, and regrouping by hand inside it does not drop out of it.
    grouping_before_view: Option<String>,
    pub sort: String,
    pub view: Option<String>,
    pub show_all: bool,
    pub pane: Pane,

    pub reading: bool,
    /// The reader panel's scroll. A `DetailScroll` rather than a counter
    /// because the panel now follows the selection: stepping onto the next
    /// item starts at the top of it, and stepping back returns you to where
    /// you had got to.
    pub reader: DetailScroll,
    /// Which pane the keyboard is driving.
    pub focus: Focus,
    /// Which acceptance criterion the detail pane is pointing at.
    ///
    /// Ticking one is the single judgement in cairn's agent loop that is
    /// explicitly a person's — *tick what is true, not what would let you
    /// close* — and it was the one thing harrow made you leave to do.
    pub criterion: usize,
    /// The detail pane's own scroll, so a long item can be read beside the
    /// list rather than in an overlay over it.
    pub detail: DetailScroll,
    /// The stats pane's own scroll. It has no cursor, so nothing else would
    /// reach the bottom of it on a short terminal.
    pub stats_scroll: u16,
    /// How the selected item changed, and where the reader is in it. Read from
    /// the repository, which is the only place that knows.
    pub history: Option<History>,
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
    /// Why harrow cannot change this backlog, when it cannot.
    ///
    /// A reason rather than a flag, because there are two of them and they
    /// call for different things of the reader. Read-only is a legitimate way
    /// to run either way, and is said out loud rather than discovered by
    /// pressing a key.
    pub readonly: Option<ReadOnly>,
    pub warnings: Vec<String>,
    /// Who harrow is, asked the way cairn asks it, so that *I did that* and
    /// *something else did that* can be told apart.
    pub me: String,
    /// Everybody who has work here, in a stable order.
    ///
    /// An actor is a thing harrow knows about rather than a string it
    /// prints: on a backlog with you and two programs on it, whose work is
    /// whose is the first-order question and `@name` in one colour answers
    /// it one row at a time.
    pub actors: Vec<String>,
    /// Whether the installed cairn can tick a criterion. Asked once at
    /// startup; an older cairn simply is not offered the gesture.
    pub can_tick: bool,
    /// The change waiting on a reason before it is proposed.
    proposing: Option<(u32, String, String)>,
    /// What the repository says has happened, most recent first.
    ///
    /// `None` until asked, because it is a process and harrow reloads on
    /// every file change; asking on a watch would be asking three times a
    /// keystroke. Cleared when the backlog changes, so the next visit to the
    /// lens asks again.
    pub moments: Option<Result<Vec<Moment>, String>>,
    /// An ask is out. Without this the loop would ask again on the next frame,
    /// and again, for as long as `git log` took to answer.
    asking_activity: bool,
    /// Which moment the cursor is on.
    moment: usize,
    /// What `cairn check` last said, if it has been asked.
    ///
    /// Kept apart from `warnings`, which is what harrow found reading the
    /// files. The two answer different questions — what could not be read,
    /// and what is invalid against the project's own rules — and a cairn
    /// complaint read as a harrow bug is the failure mode to avoid.
    pub checked: Option<Result<Vec<String>, String>>,

    /// When harrow noticed each item change, by id, in wall-clock seconds.
    ///
    /// The point of running this beside something that is doing the work: a row
    /// that just moved says so for a moment, so a glance catches what happened
    /// while you were looking at the other pane.
    pub changed: HashMap<u32, u64>,
    /// What the last rebuild put on screen, so the next one can tell what has
    /// just left rather than only what is currently there.
    shown: HashSet<u32>,
    /// Items on their way out, and when they stopped belonging.
    ///
    /// Something that leaves while you are looking at it — an item you closed,
    /// an item somebody moved out from under your filter — is held where it
    /// landed for a moment before it goes. Answering a keystroke by making the
    /// thing you pressed it on vanish does not say what happened; it only
    /// stops saying anything.
    leaving: HashMap<u32, u64>,
    pub toast: Option<(String, ToastKind, Instant)>,
    /// Wall-clock seconds, refreshed once per frame rather than read during a
    /// render. Rendering has to be a pure function of state, or a snapshot of
    /// the screen is not reproducible.
    pub now: u64,
    pub theme: Theme,
    pub keymap: Keymap,
    /// Where everything clickable ended up, in the order it was drawn. Later
    /// entries win, so an overlay covers what is beneath it.
    pub hits: Vec<(Rect, Hit)>,
    /// What each figure on the stats pane opens, rebuilt as it is drawn, the
    /// same way the hit map is.
    pub doors: Vec<Door>,
    /// Where the clickable things in the detail pane lead. Rebuilt by the
    /// renderer each frame, for the same reason `doors` is: only the renderer
    /// knows where they landed.
    pub links: Vec<Target>,
    /// Which figure the stats cursor is on.
    pub figure: usize,
    /// The last click, for telling a double-click from two single ones.
    last_click: Option<(u16, u16, Instant)>,
    /// The card being dragged, and where it started.
    pub dragging: Option<(usize, Hit)>,
    /// When harrow last asked cairn to change something.
    pub wrote: Option<Instant>,
    /// The whole screen, as this frame was given it. Recorded at the top of
    /// the draw so that everything the draw records can be held to it.
    pub screen: Rect,
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
            questions: Vec::new(),
            question: 0,
            needs_anchor: None,
            group_of: Vec::new(),
            by_id: HashMap::new(),
            selected: 0,
            offset: 0,
            column: 0,
            column_row: 0,
            column_offsets: Vec::new(),
            collapsed: HashSet::new(),
            marked: HashSet::new(),
            filter: String::new(),
            query: Query::default(),
            input: String::new(),
            editing: None,
            group_by: "milestone".to_string(),
            board_by: "status".to_string(),
            grouping_before_view: None,
            sort: String::new(),
            view: None,
            show_all: false,
            pane: Pane::List,
            reading: false,
            focus: Focus::default(),
            filtering: false,
            facets: Vec::new(),
            facet: 0,
            facet_scroll: 0,
            reader: DetailScroll::default(),
            criterion: 0,
            detail: DetailScroll::default(),
            stats_scroll: 0,
            history: None,
            picker: None,
            confirm: None,
            help: false,
            diagnostics: false,
            mouse: true,
            loading: true,
            last_load: None,
            failure: None,
            watcher_alive: true,
            readonly: None,
            warnings: Vec::new(),
            me: String::new(),
            actors: Vec::new(),
            can_tick: false,
            proposing: None,
            moments: None,
            asking_activity: false,
            moment: 0,
            checked: None,
            changed: HashMap::new(),
            shown: HashSet::new(),
            leaving: HashMap::new(),
            toast: None,
            now: unix_seconds(),
            theme: Theme::auto(true),
            keymap: Keymap::default(),
            hits: Vec::new(),
            doors: Vec::new(),
            links: Vec::new(),
            figure: 0,
            last_click: None,
            dragging: None,
            wrote: None,
            screen: Rect::default(),
            list_area: Rect::default(),
            board_area: Rect::default(),
            should_quit: false,
        }
    }

    pub fn writable(&self) -> bool {
        self.readonly.is_none()
    }

    /// How long a change stays marked. Long enough to catch on a glance back,
    /// short enough that the marks are never a second kind of status.
    pub const RECENT: u64 = 45;

    /// How long something that no longer belongs is held where it landed.
    ///
    /// Deliberately much shorter than `RECENT`, because the two answer
    /// different questions. Forty-five seconds is *what moved while I was
    /// looking at the editor*. Six is *did the key I just pressed do the
    /// thing* — and past that, a row the filter excludes is only clutter.
    pub const SETTLING: u64 = 6;

    /// Replace the backlog, keeping the cursor on the same item where we can.
    pub fn ingest(&mut self, report: Report) {
        let anchor = self.selected_item().map(|i| i.id);
        let moved = self.notice_changes(&report.items);

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
        // Decided here rather than at startup, because the format arrives
        // with the schema and the schema is re-read on every change. Nothing
        // to run outranks it: without cairn there is no `cairn migrate`
        // either, so that reason is the one worth saying.
        self.readonly = match self.readonly {
            Some(ReadOnly::NoCairn) => Some(ReadOnly::NoCairn),
            _ if self.schema.format > crate::schema::KNOWN_FORMAT => {
                Some(ReadOnly::Format(self.schema.format))
            }
            _ => None,
        };
        // The repository said something new, so what it said before is old.
        self.moments = None;
        self.check_settings();
        self.follow_view();
        self.reparse_filter();
        self.rebuild();

        if let Some(id) = anchor {
            self.select_id(id);
        }
        self.clamp();
        self.announce(moved);
    }

    /// What moved since the last reading, and when we noticed.
    ///
    /// The first reading marks nothing: everything is new the first time, and a
    /// screen that opened covered in "just changed" would be telling you about
    /// the last six months.
    fn notice_changes(&mut self, fresh: &[Item]) -> Vec<(u32, String)> {
        if self.items.is_empty() {
            return Vec::new();
        }
        // The frame's clock, not a fresh reading: everything that asks how
        // long ago this was — the mark, the exit — asks against `now`, and two
        // clocks a second apart would have an item leaving before it arrived.
        let now = self.now;
        let mut moved = Vec::new();
        for item in fresh {
            let before = self.by_id.get(&item.id).and_then(|i| self.items.get(*i));
            let changed = match before {
                None => true,
                Some(before) => {
                    before.status != item.status
                        || before.updated != item.updated
                        || before.assignee != item.assignee
                }
            };
            if changed {
                self.changed.insert(item.id, now);
                if before.is_none_or(|b| b.status != item.status) {
                    moved.push((item.id, item.status.clone()));
                }
            }
        }
        self.changed
            .retain(|_, at| now.saturating_sub(*at) <= Self::RECENT);
        moved
    }

    /// Say what somebody else did. A change harrow made says so already, so
    /// this keeps quiet for a moment after a write of our own.
    fn announce(&mut self, moved: Vec<(u32, String)>) {
        if moved.is_empty() || self.wrote_recently() {
            return;
        }
        let message = match moved.as_slice() {
            [(id, status)] => format!("{} → {status}", self.schema.format_id(*id)),
            many => format!("{} items moved", many.len()),
        };
        self.toast(message, ToastKind::Info);
    }

    fn wrote_recently(&self) -> bool {
        self.wrote
            .is_some_and(|at| at.elapsed() < Duration::from_secs(3))
    }

    /// Note that a change of ours has just landed, so the re-read it causes is
    /// not reported back to us as news.
    pub fn wrote(&mut self) {
        self.wrote = Some(Instant::now());
    }

    /// How many days an item has been claimed, where it is claimed at all.
    ///
    /// Against the frame's clock rather than a fresh reading, so that nothing
    /// drawn this frame disagrees with anything else drawn this frame.
    pub fn claimed_days(&self, item: &Item) -> Option<u64> {
        let taken = days_from_iso(item.claimed.as_deref()?)?;
        Some((self.now / 86_400).saturating_sub(taken))
    }

    /// A claim nobody has honoured for as long as the project says is long.
    ///
    /// Stale means *visible*, never revoked. Taking work away from somebody
    /// slow is worse than leaving it held, so this changes how a row is drawn
    /// and nothing else — which is also cairn's position, and the reason the
    /// threshold is the project's to set rather than harrow's to assume.
    pub fn claim_is_stale(&self, item: &Item) -> bool {
        item.claim_stale
    }

    /// Work it out for every item, against this frame's clock.
    fn age_claims(&mut self) {
        let Some(after) = self.schema.claim_stale_after.map(u64::from) else {
            for item in &mut self.items {
                item.claim_stale = false;
            }
            return;
        };
        let today = self.now / 86_400;
        for item in &mut self.items {
            item.claim_stale = !item.category.is_closed()
                && item.assignee.is_some()
                && item
                    .claimed
                    .as_deref()
                    .and_then(days_from_iso)
                    .is_some_and(|taken| today.saturating_sub(taken) > after);
        }
    }

    /// Whether an item moved recently enough to still be worth pointing at.
    pub fn is_recent(&self, id: u32) -> bool {
        self.changed
            .get(&id)
            .is_some_and(|at| self.now.saturating_sub(*at) <= Self::RECENT)
    }

    /// How many items are waiting on somebody to decide something.
    pub fn proposed(&self) -> usize {
        self.items
            .iter()
            .filter(|i| !i.proposals.is_empty() && !i.category.is_closed())
            .count()
    }

    /// Every status with something in it, ordered for a glance: what is active
    /// first, then what is open, then what is finished.
    ///
    /// Deliberately not the declared order the list and the board use. Those
    /// are a place you move through; this is a summary, and a summary leads
    /// with what is live.
    /// Deliberately the whole project, filter or no filter.
    ///
    /// The strip is not a lens; it is the control that sets the filter, and
    /// clicking a status *replaces* whatever is in the box. So each count is
    /// exactly what clicking it would give you, which is the only reading
    /// that makes the numbers useful for choosing. Narrowing them would also
    /// collapse the strip to the one status already chosen, leaving nothing
    /// to click your way back out to.
    pub fn status_counts(&self) -> Vec<(&crate::schema::Status, usize)> {
        let mut counts: Vec<(&crate::schema::Status, usize)> = self
            .schema
            .statuses
            .iter()
            .map(|status| {
                let n = self
                    .items
                    .iter()
                    .filter(|i| !i.container && i.status == status.name)
                    .count();
                (status, n)
            })
            .filter(|(_, n)| *n > 0)
            .collect();
        counts.sort_by_key(|(status, _)| match status.category {
            Category::Active => 0,
            Category::Open => 1,
            Category::Done => 2,
            Category::Dropped => 3,
        });
        counts
    }

    /// What is on screen, and why — the three answers the view line states.
    ///
    /// Which items, in what order, grouped how. All three are already state;
    /// only one of them has ever been on screen, and it was the one nobody
    /// could change. A view whose rules are off screen is a view nobody can
    /// check, which is how the strip and the list came to disagree twice.
    pub fn view_line(&self) -> ViewLine {
        ViewLine {
            view: self.view.clone(),
            query: self.query.source(),
            sort: self
                .sort_keys()
                .iter()
                .map(|k| {
                    if k.descending {
                        format!("-{}", k.field)
                    } else {
                        k.field.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
            // The sort has never been set by anything but argv, so saying
            // whether it is the project's default or a choice is most of
            // what there is to say about it.
            sort_is_default: self.sort.trim().is_empty(),
            group_by: self.group_by.clone(),
            shown_and_total: self.shown_and_total(),
        }
    }

    /// The same view, as the command that would reproduce it.
    ///
    /// A backlog narrowed by hand becomes something that can go in a script,
    /// a README, or somebody else's terminal — and the interface teaches the
    /// command line rather than hiding it.
    pub fn command_line(&self) -> String {
        let mut out = String::from("harrow");
        if self.show_all {
            out.push_str(" -a");
        }
        match (&self.view, self.query.source()) {
            // The name, where one was chosen and nothing has been added to
            // it: `--view triage` is the thing somebody would rather paste.
            (Some(view), _) if self.filter.trim().is_empty() => {
                out.push_str(&format!(" --view {}", shell_word(view)));
            }
            (_, query) if !query.is_empty() => {
                out.push_str(&format!(" -f {}", shell_word(&query)));
            }
            _ => {}
        }
        if !self.sort.trim().is_empty() {
            out.push_str(&format!(" --sort {}", shell_word(&self.sort)));
        }
        if !matches!(self.group_by.as_str(), "milestone") {
            out.push_str(&format!(" --group-by {}", shell_word(&self.group_by)));
        }
        out
    }

    /// What an ordinary listing is leaving out, of the things that match.
    ///
    /// Only what the filter in force would otherwise have shown: the empty
    /// state has to say why *this* list is empty, not what else is in the
    /// project. Both numbers are about the rule in `belongs` — the two sets
    /// an ordinary listing hides until you ask for them.
    pub fn hidden(&self) -> Hidden {
        let mut out = Hidden::default();
        if self.show_all {
            return out;
        }
        let mut kinds: Vec<&str> = Vec::new();
        for item in self.items.iter() {
            if !self.query.matches(item, &self.schema) {
                continue;
            }
            if item.container {
                // A closed milestone is hidden for being closed, and is not
                // something still waiting to be closed.
                if !item.category.is_closed() && !self.query.names("type", &item.kind) {
                    out.containers += 1;
                    if !kinds.contains(&item.kind.as_str()) {
                        kinds.push(&item.kind);
                    }
                }
            } else if item.category.is_closed()
                && !self.query.names("status", &item.status)
                && !self.query.names("category", item.category.name())
            {
                out.closed += 1;
            }
        }
        // Named only where there is one name to use. A project with releases
        // and epics both open gets the count without a noun rather than a
        // noun that is wrong for half of them.
        if let [only] = kinds[..] {
            out.kind = self
                .schema
                .item_type(only)
                .map(|t| t.display().to_string())
                .or_else(|| Some(only.to_string()));
        }
        out
    }

    /// What is wrong with the filter in force, if anything.
    ///
    /// A field nothing declares matches nothing, which is indistinguishable
    /// from a backlog with nothing in it — and that is the whole defect: a
    /// `[[view]]` cairn accepts opened here as an empty pane saying *No
    /// matches*. cairn is lenient at query time and strict in `check`;
    /// harrow has no check time, so it is strict here.
    pub fn filter_problem(&self) -> Option<String> {
        let unknown = &self.query.unknown;
        match unknown.len() {
            0 => None,
            1 => Some(format!(
                "no such field `{}` — this project does not declare it",
                unknown[0]
            )),
            _ => Some(format!("no such fields: {}", unknown.join(", "))),
        }
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
        // The queue can only point at items that raise a question. Where it
        // can, it does; where it cannot, it remembers rather than losing it.
        match self.questions.iter().position(|q| q.id == id) {
            Some(at) => {
                self.question = at;
                self.needs_anchor = None;
            }
            None => self.needs_anchor = Some(id),
        }
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
        self.belongs(item) || self.leaving.contains_key(&item.id)
    }

    /// How much of the backlog the filter is letting through, and how much
    /// there is. What the panel puts on its own edge, because a filter with no
    /// result in sight is a guess.
    pub fn shown_and_total(&self) -> (usize, usize) {
        let total = self.items.iter().filter(|i| !i.container).count();
        let shown = self
            .items
            .iter()
            .filter(|i| !i.container && self.query.matches(i, &self.schema))
            .count();
        (shown, total)
    }

    /// Whether an item is dealt onto the board.
    ///
    /// The filter applies — narrowing to one milestone should narrow the board
    /// too — and a container is not work and so is not a card. Nothing else:
    /// what is and is not a column is the project's to say.
    pub fn on_board(&self, item: &Item) -> bool {
        !item.container && self.query.matches(item, &self.schema)
    }

    /// Whether an item is part of what is being shown, on its own merits.
    fn belongs(&self, item: &Item) -> bool {
        if !self.show_all {
            // Two things an ordinary listing leaves out, under one rule:
            // absent unless you ask for them by name. That is cairn's rule,
            // and harrow reading it differently is how `status=done` came to
            // list nothing in a project with seventy finished items — an
            // answer a reader has no way to tell from an empty backlog.
            //
            // Finished work, asked for by its status or its category.
            if item.category.is_closed()
                && !self.query.names("status", &item.status)
                && !self.query.names("category", item.category.name())
            {
                return false;
            }
            // And a milestone, asked for by type. It is a thing work belongs
            // to rather than a piece of work, and listing it beside the work
            // it contains reads as a duplicate.
            if item.container && !self.query.names("type", &item.kind) {
                return false;
            }
        }
        self.query.matches(item, &self.schema)
    }

    /// Start the exit of anything that has just changed its way off the
    /// screen, and finish the exit of anything whose moment has passed.
    ///
    /// Two conditions, and both matter. It has to have been on screen: an item
    /// somebody moves under a filter it never matched does not appear for six
    /// seconds to announce itself, because that is noise and the toast already
    /// covers it. And the *item* has to be what changed, not the view — typing
    /// a filter is a deliberate act of exclusion, and a list that answered it
    /// by holding on to what you just excluded would be arguing with you.
    fn resettle(&mut self) {
        let now = self.now;
        for id in &self.shown {
            let Some(item) = self.by_id.get(id).and_then(|i| self.items.get(*i)) else {
                continue;
            };
            let moved = self
                .changed
                .get(id)
                .is_some_and(|at| now.saturating_sub(*at) <= Self::SETTLING);
            if moved && !self.belongs(item) {
                self.leaving.entry(*id).or_insert(now);
            }
        }
        self.leaving
            .retain(|_, at| now.saturating_sub(*at) <= Self::SETTLING);
    }

    /// Whether anything is still on its way out. The board asks, so a column
    /// can be drawn to receive what is landing in it.
    pub fn is_leaving(&self, id: u32) -> bool {
        self.leaving.contains_key(&id)
    }

    /// Let go of whatever has finished leaving, and say whether the screen has
    /// to be rebuilt because of it. Called once a frame by the shell, so a row
    /// goes when its moment is up rather than at the next keystroke.
    pub fn settle(&mut self) -> bool {
        let now = self.now;
        let before = self.leaving.len();
        self.leaving
            .retain(|_, at| now.saturating_sub(*at) <= Self::SETTLING);
        if self.leaving.len() == before {
            return false;
        }
        self.rebuild();
        true
    }

    /// Who has work here. Sorted so a colour does not move between frames,
    /// and with yours first, because yours is the one you pick out.
    fn find_actors(&mut self) {
        let mut who: Vec<String> = self
            .items
            .iter()
            .flat_map(|i| [i.assignee.clone(), i.owner.clone()])
            .flatten()
            .collect();
        who.sort_unstable();
        who.dedup();
        if let Some(at) = who.iter().position(|w| w.eq_ignore_ascii_case(&self.me)) {
            let me = who.remove(at);
            who.insert(0, me);
        }
        self.actors = who;
    }

    /// Where somebody falls in the cast, for a colour.
    pub fn actor(&self, who: &str) -> Option<usize> {
        self.actors.iter().position(|a| a == who)
    }

    /// Whether this is you. harrow asks the question cairn asks, so the two
    /// agree about whose work is whose.
    pub fn is_me(&self, who: &str) -> bool {
        !self.me.is_empty() && who.eq_ignore_ascii_case(&self.me)
    }

    /// Everything addressed to a person, ranked by what is waiting.
    ///
    /// Built from the same set the board is dealt from, so the filter applies
    /// and a container is not a question — a milestone nobody owns is not
    /// somebody failing to own it.
    fn build_questions(&mut self) {
        let on = self.question;
        let was = self.questions.get(on).cloned();
        let mut out: Vec<Question> = Vec::new();

        for item in self.items.iter().filter(|i| self.on_board(i)) {
            // A proposal is somebody blocked on an answer, so it comes first
            // and it comes even for closed work: an unanswered question does
            // not stop being one because the item moved on.
            if let Some(p) = item.proposals.last() {
                out.push(Question {
                    id: item.id,
                    asking: Asking::Proposal {
                        field: p.field.clone(),
                        to: p.to.clone(),
                        by: p.by.clone(),
                    },
                });
            }
            if item.category.is_closed() {
                continue;
            }
            if item.claim_stale
                && let Some(who) = &item.assignee
            {
                out.push(Question {
                    id: item.id,
                    asking: Asking::ColdClaim {
                        who: who.clone(),
                        days: self.claimed_days(item).unwrap_or(0),
                    },
                });
            }
            let (met, total) = item.criteria();
            if total > 0 && met == total {
                out.push(Question {
                    id: item.id,
                    asking: Asking::Finished,
                });
            }
            // Only where something else filed it. A person filing their own
            // work and not writing their name on it has not left a question.
            if let Some(by) = &item.created_by
                && item.owner.is_none()
                && item.assignee.is_none()
            {
                out.push(Question {
                    id: item.id,
                    asking: Asking::Unowned { by: by.clone() },
                });
            }
        }

        // A container raises no question about ownership or a stale claim, so
        // it is not in the loop above. It raises exactly one: everything
        // filed under it is finished and it is still open. In poptop that was
        // all seven milestones at once, and nothing anywhere said so.
        for item in self.items.iter().filter(|i| i.container) {
            if item.category.is_closed() || !self.query.matches(item, &self.schema) {
                continue;
            }
            let under = self.rollup(item);
            // Nothing filed is vacuously complete, and every empty milestone
            // in a young project would otherwise be told it had finished the
            // day it was written.
            if under.done > 0 && under.active == 0 && under.open == 0 {
                out.push(Question {
                    id: item.id,
                    asking: Asking::NothingUnfinished,
                });
            }
        }

        out.sort_by_key(|q| (q.asking.urgency(), q.id));
        // Stay on the same question across a rebuild where it survives, so
        // answering one does not move the cursor under the next.
        self.question = was
            .and_then(|q| out.iter().position(|o| *o == q))
            .unwrap_or_else(|| on.min(out.len().saturating_sub(1)));
        self.questions = out;
    }

    /// Move the queue's cursor, which is a choice and so takes over from
    /// whatever was selected on the way in.
    ///
    /// The only way to move it: leaving `question` settable on its own would
    /// let the cursor and the anchor disagree about what is selected, and
    /// they are two halves of one answer.
    pub fn select_question(&mut self, n: usize) {
        if self.questions.is_empty() {
            return;
        }
        self.question = n.min(self.questions.len() - 1);
        self.needs_anchor = None;
    }

    pub fn question(&self) -> usize {
        self.question
    }

    /// The item a question is about, which is what the detail pane shows.
    pub fn question_item(&self) -> Option<&Item> {
        let id = self.questions.get(self.question)?.id;
        self.items.iter().find(|i| i.id == id)
    }

    /// The columns a board draws, in the order the project declared them.
    ///
    /// Every axis takes its columns from what the project says exists rather
    /// than from what the items happen to carry, so a column stays put when
    /// the last card leaves it — a board whose columns move as work arrives
    /// is a board you cannot learn the shape of. Two axes cannot: an
    /// assignee is not declared anywhere, and neither is nothing.
    pub fn board_values(&self) -> Vec<String> {
        let mut values: Vec<String> = match self.board_by.as_str() {
            "none" | "" => return Vec::new(),
            // `board = false` is how a project says a status is not a column,
            // and it is the only axis with such a thing to say.
            "status" => {
                return self
                    .schema
                    .board_statuses()
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();
            }
            "type" => self
                .schema
                .types
                .iter()
                .filter(|k| k.groups.is_none())
                .map(|k| k.name.clone())
                .collect(),
            "assignee" => {
                let mut who: Vec<String> = self
                    .items
                    .iter()
                    .filter(|i| !i.container)
                    .filter_map(|i| i.assignee.clone())
                    .collect();
                who.sort();
                who.dedup();
                who
            }
            other => match self.schema.field(other) {
                // An enum's declared values, in their declared order.
                Some(f) if !f.values.is_empty() => f.values.clone(),
                // A reference: the items of the type it names, by key, in the
                // order the roadmap runs.
                Some(f) if matches!(f.kind, crate::schema::FieldKind::Ref) => self
                    .items
                    .iter()
                    .filter(|i| Some(i.kind.as_str()) == f.target.as_deref())
                    .filter_map(|i| i.key.clone())
                    .collect(),
                _ => Vec::new(),
            },
        };
        // Somewhere for the items that have no value, but only if any have
        // none: an empty "no milestone" column is a column about nothing.
        if self
            .items
            .iter()
            .any(|i| self.on_board(i) && self.key_by(i, &self.board_by).is_empty())
        {
            values.push(String::new());
        }
        values
    }

    /// The value an item is grouped under, and what to call it.
    fn group_key(&self, item: &Item) -> String {
        self.key_by(item, &self.group_by)
    }

    fn key_by(&self, item: &Item, axis: &str) -> String {
        match axis {
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
    /// Only by key, which is the rule and not a shortcut.
    ///
    /// A project may render its identifiers with padding, so `milestone: 0042`
    /// could name either a key or a number depending on what happens to exist.
    /// What keeps it unambiguous is a pair of rules — a key may not look like
    /// a rendered identifier, and a key-addressed reference resolves by key
    /// alone — and the second is worth nothing without the first. Falling back
    /// to an id here is what made the pair worthless.
    ///
    /// Case-insensitively, as the reference implementation compares them, but
    /// by key and by nothing else.
    pub fn item_named(&self, key: &str) -> Option<&Item> {
        self.items.iter().find(|i| {
            i.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(key))
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

    /// What the backlog can be narrowed by, and what each choice would leave.
    ///
    /// Everything here comes from the project's schema — its statuses, its
    /// types, the enum fields it declared — so the panel says `severity` in a
    /// project that tracks severity and has never heard of `priority`.
    fn rebuild_facets(&mut self) {
        let mut facets: Vec<Facet> = Vec::new();

        let mut facet =
            |field: &str, label: &str, role: FacetRole, values: Vec<(String, String)>| {
                // A facet's own ticks do not count against its own values, so
                // ticking one status leaves the others reachable rather than
                // reading nought beside every one of them.
                let rest = self.query.without(field);
                let ticked = self.query.ticked(field);
                let values = values
                    .into_iter()
                    .map(|(value, label)| FacetValue {
                        count: self
                            .items
                            .iter()
                            .filter(|item| {
                                !item.container
                                    && rest.matches(item, &self.schema)
                                    && crate::filter::Query::parse(
                                        &format!("{field}={value}"),
                                        &self.schema,
                                    )
                                    .matches(item, &self.schema)
                            })
                            .count(),
                        ticked: ticked.iter().any(|t| t.eq_ignore_ascii_case(&value)),
                        role,
                        value,
                        label,
                    })
                    .collect();
                facets.push(Facet {
                    field: field.to_string(),
                    label: label.to_string(),
                    values,
                });
            };

        facet(
            "status",
            "status",
            FacetRole::Status,
            self.schema
                .statuses
                .iter()
                .filter(|s| self.show_all || s.category != Category::Dropped)
                .map(|s| (s.name.clone(), s.display().to_string()))
                .collect(),
        );
        facet(
            "type",
            "type",
            FacetRole::Type,
            self.schema
                .types
                .iter()
                .map(|t| (t.name.clone(), t.display().to_string()))
                .collect(),
        );
        // Only the enums. A free-text field has no list of values to tick, and
        // a box of every title anybody wrote is not a facet.
        let enums: Vec<(String, Vec<String>)> = self
            .schema
            .fields
            .iter()
            .filter(|f| !f.values.is_empty())
            .map(|f| (f.name.clone(), f.values.clone()))
            .collect();
        for (name, values) in enums {
            let rank = self
                .schema
                .fields
                .iter()
                .any(|f| f.name == name && f.column);
            facet(
                &name,
                &name.clone(),
                if rank {
                    FacetRole::Rank
                } else {
                    FacetRole::Plain
                },
                values.into_iter().map(|v| (v.clone(), v)).collect(),
            );
        }

        // Nothing to tick is not a facet worth a heading.
        facets.retain(|f| !f.values.is_empty());
        self.facets = facets;
        let total = self.facet_count();
        self.facet = self.facet.min(total.saturating_sub(1));
    }

    /// How many values the panel's cursor can be on.
    pub fn facet_count(&self) -> usize {
        self.facets.iter().map(|f| f.values.len()).sum()
    }

    /// The facet and value the cursor is on.
    pub fn facet_at(&self, n: usize) -> Option<(&Facet, &FacetValue)> {
        let mut seen = 0;
        for facet in &self.facets {
            if n < seen + facet.values.len() {
                return Some((facet, &facet.values[n - seen]));
            }
            seen += facet.values.len();
        }
        None
    }

    /// Tick or untick what the cursor is on, and write the result back as the
    /// expression somebody could have typed.
    pub fn toggle_facet(&mut self) {
        let Some((facet, value)) = self.facet_at(self.facet) else {
            return;
        };
        let (field, value) = (facet.field.clone(), value.value.clone());
        let mut chosen: Vec<String> = self.query.ticked(&field);
        if let Some(at) = chosen.iter().position(|v| v.eq_ignore_ascii_case(&value)) {
            chosen.remove(at);
        } else {
            chosen.push(value);
        }

        // Everything the panel does not manage is written back as it was read,
        // so a range or a free-text word survives a tick.
        let managed: Vec<String> = self.facets.iter().map(|f| f.field.clone()).collect();
        let mut clauses = self.query.except(&managed);
        for f in &self.facets {
            let values = if f.field == field {
                chosen.clone()
            } else {
                self.query.ticked(&f.field)
            };
            if !values.is_empty() {
                clauses.push(format!("{}={}", f.field, values.join("|")));
            }
        }
        self.filter = clauses.join(",");
        self.reparse_filter();
        self.rebuild();
    }

    pub fn rebuild(&mut self) {
        self.age_claims();
        self.resettle();
        let heading_type = self.heading_type().map(str::to_string);
        let keys: Vec<String> = self.items.iter().map(|i| self.group_key(i)).collect();
        let ranks: Vec<(u8, String, u64)> = keys.iter().map(|k| self.group_rank(k)).collect();
        let sort = self.sort_keys();

        let order = |a: &usize, b: &usize| {
            ranks[*a]
                .cmp(&ranks[*b])
                .then_with(|| keys[*a].cmp(&keys[*b]))
                .then_with(|| self.compare(*a, *b, &sort))
        };

        let mut indices: Vec<usize> = (0..self.items.len())
            .filter(|i| self.visible(&self.items[*i]))
            .filter(|i| self.is_row(&self.items[*i], heading_type.as_deref()))
            .collect();
        indices.sort_by(order);

        // The board's own set, in the list's order: a column reads down the
        // same way a group does, or the two views of one backlog disagree
        // about what comes first.
        let mut cards: Vec<usize> = (0..self.items.len())
            .filter(|i| self.on_board(&self.items[*i]))
            .collect();
        // By the sort keys alone. The list's grouping is invisible on a
        // board grouped by something else, so carrying its rank here made a
        // status board read as unsorted: priority ran p0, p0, p2, p2, p0 —
        // which was milestone order, correctly applied and meaningless.
        cards.sort_by(|a, b| self.compare(*a, *b, &sort));

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

        self.build_board(&cards);
        self.find_actors();
        self.build_questions();
        // What this build put on screen, which is what the next one measures
        // against to know what has just left.
        self.shown = indices.iter().map(|i| self.items[*i].id).collect();
        self.clamp();
        // Last, because a facet counts what the rebuild just settled.
        self.rebuild_facets();
    }

    /// What is under a heading, counting what the filter is hiding.
    /// Whether an item is a row in the list, or the heading over one.
    ///
    /// An item of the type being grouped by is drawn as the heading rather
    /// than as a row under it, or it would be on screen twice.
    ///
    /// Unless the reader asked for it. `a` promises "finished work and
    /// milestones" and `type=milestone` asks in so many words, and under this
    /// grouping — the default, where the heading type *is* the milestone —
    /// both answered with nothing at all, while every other grouping answered
    /// with all seven. A rule that depends on how the list happens to be
    /// arranged is a rule the reader cannot see. cairn returns them for
    /// either question, and so does this now.
    fn is_row(&self, item: &Item, heading_type: Option<&str>) -> bool {
        heading_type != Some(item.kind.as_str())
            || self.show_all
            || self.query.names("type", &item.kind)
    }

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
        self.label_by(key, &self.group_by)
    }

    fn label_by(&self, key: &str, axis: &str) -> String {
        if key.is_empty() {
            return match axis {
                "status" | "type" => "unset".to_string(),
                "assignee" => "unassigned".to_string(),
                other => format!("no {other}"),
            };
        }
        match axis {
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
    /// The board is the same set of items, dealt into the columns the project
    /// asked for.
    ///
    /// Which set is not the list's. `a` is a question about the list — whether
    /// finished work is worth a row among the work that is left — and the
    /// board has already answered it: a status is a column because the project
    /// wrote `board = true`, and `dropped` is absent here because this project
    /// wrote `board = false` on it. Applying the list's rule as well is what
    /// produced a `done` column beside `✓ 1 done` in the strip, permanently
    /// holding nothing, with cards dragged into it disappearing on arrival.
    fn build_board(&mut self, cards: &[usize]) {
        self.columns = self
            .board_values()
            .into_iter()
            .map(|value| Column {
                label: self.label_by(&value, &self.board_by),
                value,
                items: Vec::new(),
            })
            .collect();
        for &i in cards {
            let key = self.key_by(&self.items[i], &self.board_by);
            if let Some(column) = self.columns.iter_mut().find(|c| c.value == key) {
                column.items.push(i);
            }
        }
        // Kept by position rather than rebuilt, so a column you had scrolled
        // stays where you left it when something elsewhere changes.
        self.column_offsets.resize(self.columns.len(), 0);
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

    /// Adopt or put back the grouping a view asks for.
    ///
    /// Called wherever the view changes, so the two cannot disagree — the
    /// thing that made harrow show a screen the project had not described
    /// and the user had not asked for.
    fn follow_view(&mut self) {
        let wanted = self
            .view
            .as_deref()
            .and_then(|v| self.schema.view(v))
            .and_then(|v| v.group_by.clone());
        match (wanted, self.grouping_before_view.take()) {
            (Some(group_by), before) => {
                self.grouping_before_view = Some(before.unwrap_or_else(|| self.group_by.clone()));
                self.group_by = group_by;
            }
            (None, Some(before)) => self.group_by = before,
            (None, None) => {}
        }
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

        // A column that shrank under its own offset would be scrolled past
        // everything it holds until the next draw pulled it back. How far the
        // pane can actually see is the draw's business; that there is a card
        // down there at all is this one's.
        self.column_offsets.resize(self.columns.len(), 0);
        for (offset, column) in self.column_offsets.iter_mut().zip(&self.columns) {
            *offset = (*offset).min(column.items.len().saturating_sub(1));
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
        if self.pane == Pane::Log {
            let id = self.moments().get(self.moment)?.id;
            return self.items.iter().position(|i| i.id == id);
        }
        if self.pane == Pane::Needs {
            let id = match self.needs_anchor {
                Some(anchor) => anchor,
                None => self.questions.get(self.question)?.id,
            };
            return self.items.iter().position(|i| i.id == id);
        }
        if self.pane == Pane::Board {
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
        if self.pane == Pane::Log {
            let len = self.moments().len() as isize;
            if len == 0 {
                return;
            }
            let at = (self.moment as isize + delta.signum() * delta.abs().min(len)).rem_euclid(len)
                as usize;
            self.select_moment(at);
            return;
        }
        if self.pane == Pane::Needs {
            if self.questions.is_empty() {
                return;
            }
            // Moving here is a choice, so it takes over from whatever was
            // selected on the way in.
            self.needs_anchor = None;
            let len = self.questions.len() as isize;
            self.question = (self.question as isize + delta.signum() * delta.abs().min(len))
                .rem_euclid(len) as usize;
            return;
        }
        // The stats pane has a cursor of its own, over the figures that lead
        // somewhere. The pane scrolls to keep it in view, which is what the
        // list does and is why the keys are the same keys.
        if self.pane == Pane::Stats {
            if self.doors.is_empty() {
                self.stats_scroll = self
                    .stats_scroll
                    .saturating_add_signed(delta.clamp(-64, 64) as i16);
                return;
            }
            let len = self.doors.len() as isize;
            self.figure = (self.figure as isize + delta.signum() * delta.abs().min(len))
                .rem_euclid(len) as usize;
            return;
        }
        if self.pane == Pane::Board {
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
        if self.pane == Pane::Log {
            self.select_moment(if to_end { usize::MAX } else { 0 });
            return;
        }
        if self.pane == Pane::Needs {
            self.needs_anchor = None;
            self.question = if to_end {
                self.questions.len().saturating_sub(1)
            } else {
                0
            };
            return;
        }
        if self.pane == Pane::Stats {
            if self.doors.is_empty() {
                // Past the end is pulled back by the draw, which is the only
                // thing that knows how tall the stats came out.
                self.stats_scroll = if to_end { u16::MAX } else { 0 };
            } else {
                self.figure = if to_end { self.doors.len() - 1 } else { 0 };
            }
            return;
        }
        if self.pane == Pane::Board {
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
        if self.pane == Pane::Board {
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

    /// The axis the lens in front of you is arranged by.
    pub fn axis(&self) -> &str {
        if self.pane == Pane::Board {
            &self.board_by
        } else {
            &self.group_by
        }
    }

    pub fn cycle_grouping(&mut self) {
        let axes = self.grouping_axes();
        let at = axes.iter().position(|a| a == self.axis()).unwrap_or(0);
        // On the board, step over an axis that cannot be a board rather than
        // landing on an empty screen: `none` is a list with the grouping off,
        // which is a thing to want, and is not a thing a board can be.
        let usable = |axis: &str| self.pane != Pane::Board || !matches!(axis, "none" | "");
        let mut next = (at + 1) % axes.len();
        for _ in 0..axes.len() {
            if usable(&axes[next]) {
                break;
            }
            next = (next + 1) % axes.len();
        }
        if self.pane == Pane::Board {
            self.board_by = axes[next].clone();
        } else {
            self.group_by = axes[next].clone();
        }
        // Choosing an axis by hand inside a view is a choice, not a drift: it
        // keeps the view and keeps the grouping, and leaving the view now
        // puts back what was in force before the view rather than the view's.
        self.grouping_before_view = None;
        let id = self.selected_item().map(|i| i.id);
        self.rebuild();
        if let Some(id) = id {
            self.select_id(id);
        }
        let axis = self.axis().to_string();
        self.toast(
            match self.pane {
                Pane::Board => format!("columns by {axis}"),
                _ => format!("grouped by {axis}"),
            },
            ToastKind::Info,
        );
    }

    // ── Changing an item ─────────────────────────────────────────────────────

    /// What the next change applies to: what is marked, or what is under the
    /// cursor. Sorted, so the command reads the way the list does.
    pub fn targets(&self) -> Vec<u32> {
        if self.marked.is_empty() {
            return self.selected_item().map(|i| i.id).into_iter().collect();
        }
        let mut ids: Vec<u32> = self.marked.iter().copied().collect();
        ids.sort_unstable();
        ids
    }

    pub fn toggle_mark(&mut self) {
        let Some(id) = self.selected_item().map(|i| i.id) else {
            return;
        };
        if !self.marked.remove(&id) {
            self.marked.insert(id);
        }
        self.move_by(1);
    }

    /// Mark everything between the cursor and a row, which is what a person
    /// expects shift-click to do.
    fn mark_range(&mut self, to: usize) {
        let (from, to) = if to < self.selected {
            (to, self.selected)
        } else {
            (self.selected, to)
        };
        for row in from..=to {
            if let Some(Row::Item(i)) = self.rows.get(row)
                && let Some(item) = self.items.get(*i)
            {
                self.marked.insert(item.id);
            }
        }
        self.selected = to;
        self.clamp();
    }

    /// Hand a change over, asking first when it touches more than one item.
    ///
    /// One keystroke changing forty things is exactly the gesture that wants a
    /// sentence between the intention and the write.
    fn write(&mut self, change: Change, count: usize, what: &str) -> Action {
        if let Some(why) = self.readonly.clone() {
            self.refuse(&why);
            return Action::None;
        }
        if count <= 1 {
            return Action::Write(change);
        }
        self.confirm = Some(Confirm {
            prompt: format!("{what} {count} items?"),
            detail: change
                .args
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join(" "),
            change,
        });
        Action::None
    }

    fn refuse(&mut self, why: &ReadOnly) {
        self.toast(why.at_length(), ToastKind::Bad);
    }

    fn set_field(&mut self, field: &str, value: &str) -> Action {
        // Only the ones the change would actually move. Telling cairn to set a
        // field to what it already says is a write, a hook run and a line of
        // history for nothing.
        let targets: Vec<u32> = self
            .targets()
            .into_iter()
            .filter(|id| {
                self.by_id
                    .get(id)
                    .and_then(|i| self.items.get(*i))
                    .is_some_and(|item| current_value(item, field).as_deref() != Some(value))
            })
            .collect();

        match targets.len() {
            0 => {
                let shown = if value.is_empty() { "unset" } else { value };
                self.toast(format!("{field} is already {shown}"), ToastKind::Info);
                Action::None
            }
            1 => {
                let id = targets[0];
                let was = self
                    .by_id
                    .get(&id)
                    .and_then(|i| self.items.get(*i))
                    .and_then(|item| current_value(item, field));
                let reference = self.schema.format_id(id);
                let shown = if value.is_empty() { "cleared" } else { value };
                let change = Change {
                    args: vec!["set".into(), id.to_string(), format!("{field}={value}")],
                    describe: format!("{reference} {field} → {shown}"),
                    undo: was.map(|w| format!("cairn set {id} {field}={w}")),
                };
                self.write(change, 1, "set")
            }
            n => {
                let shown = if value.is_empty() { "cleared" } else { value };
                let change = Change {
                    args: self.bulk_args("set", &targets, Some(format!("{field}={value}"))),
                    describe: format!("{n} items · {field} → {shown}"),
                    undo: None,
                };
                self.write(change, n, "Set the field on")
            }
        }
    }

    /// The argument vector for a change to many items.
    ///
    /// Where the marks are exactly what the filter is showing, this becomes one
    /// `--filter` invocation instead of a list of ids: it is the same change,
    /// it is what somebody would have typed, and it stays correct if the set
    /// moves underneath between the decision and the write.
    fn bulk_args(&self, command: &str, targets: &[u32], assignment: Option<String>) -> Vec<String> {
        // What the filter is *showing*, which is not the same as what passes
        // it: a milestone is the heading its items sit under rather than a row.
        let heading = self.heading_type().map(str::to_string);
        let showing: Vec<u32> = self
            .items
            .iter()
            .filter(|i| self.visible(i))
            .filter(|i| heading.as_deref() != Some(i.kind.as_str()))
            .map(|i| i.id)
            .collect();
        let same = showing.len() == targets.len() && showing.iter().all(|id| targets.contains(id));

        let mut args = vec![command.to_string()];
        if same && !self.filter.trim().is_empty() {
            args.push("--filter".into());
            args.push(self.filter.clone());
            args.push("--yes".into());
        } else {
            args.extend(targets.iter().map(u32::to_string));
        }
        args.extend(assignment);
        args
    }

    pub fn claim(&mut self, take: bool) -> Action {
        if take {
            return self.hand(true, None);
        }
        // Giving something up is the one moment where the person letting go
        // knows exactly why and the next person to pick it up is about to
        // need it. cairn records the reason as a note and shows it on the
        // next claim, so the whole of asking is this box.
        if self.targets().is_empty() {
            return Action::None;
        }
        self.editing = Some(Editing::Reason);
        self.input.clear();
        Action::None
    }

    fn release_with(&mut self, reason: Option<&str>) -> Action {
        self.hand(false, reason)
    }

    fn hand(&mut self, take: bool, reason: Option<&str>) -> Action {
        let targets = self.targets();
        let (command, undo) = if take {
            ("claim", "release")
        } else {
            ("release", "claim")
        };
        let with_reason = |mut args: Vec<String>| {
            if let Some(why) = reason {
                args.push("--reason".into());
                args.push(why.to_string());
            }
            args
        };
        match targets.as_slice() {
            [] => Action::None,
            [id] => {
                let change = Change {
                    args: with_reason(vec![command.into(), id.to_string()]),
                    describe: format!("{} {command}ed", self.schema.format_id(*id)),
                    undo: Some(format!("cairn {undo} {id}")),
                };
                self.write(change, 1, command)
            }
            many => {
                let n = many.len();
                let change = Change {
                    args: with_reason(self.bulk_args(command, many, None)),
                    describe: format!("{n} items {command}ed"),
                    undo: None,
                };
                self.write(change, n, if take { "Claim" } else { "Release" })
            }
        }
    }

    /// Closing is the one change that asks first. Not because it cannot be
    /// undone — `u` reopens — but because it is a declaration that something is
    /// finished, and it runs the project's hooks.
    pub fn ask_close(&mut self) {
        let targets: Vec<u32> = self
            .targets()
            .into_iter()
            .filter(|id| {
                self.by_id
                    .get(id)
                    .and_then(|i| self.items.get(*i))
                    .is_some_and(|item| item.category != Category::Done)
            })
            .collect();

        let (prompt, detail, change) = match targets.as_slice() {
            [] => {
                self.toast("already closed", ToastKind::Info);
                return;
            }
            [id] => {
                let Some(item) = self.by_id.get(id).and_then(|i| self.items.get(*i)) else {
                    return;
                };
                let reference = self.schema.format_id(*id);
                let (met, total) = item.criteria();
                let detail = if total > 0 && met < total {
                    format!("{met} of {total} acceptance criteria are ticked")
                } else {
                    item.title.clone()
                };
                (
                    format!("Close {reference}?"),
                    detail,
                    Change {
                        args: vec!["close".into(), id.to_string()],
                        describe: format!("{reference} closed"),
                        undo: Some(format!("cairn reopen {id}")),
                    },
                )
            }
            many => {
                let n = many.len();
                let unticked = many
                    .iter()
                    .filter_map(|id| self.by_id.get(id).and_then(|i| self.items.get(*i)))
                    .filter(|item| {
                        let (met, total) = item.criteria();
                        total > 0 && met < total
                    })
                    .count();
                let detail = if unticked > 0 {
                    format!("{unticked} of them have acceptance criteria left unticked")
                } else {
                    many.iter()
                        .map(|id| self.schema.format_id(*id))
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                (
                    format!("Close {n} items?"),
                    detail,
                    Change {
                        args: self.bulk_args("close", many, None),
                        describe: format!("{n} items closed"),
                        undo: None,
                    },
                )
            }
        };

        self.confirm = Some(Confirm {
            prompt,
            detail,
            change,
        });
    }

    pub fn resolve_confirm(&mut self, go: bool) -> Action {
        let Some(c) = self.confirm.take() else {
            return Action::None;
        };
        if !go {
            return Action::None;
        }
        if let Some(why) = self.readonly.clone() {
            self.refuse(&why);
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
            permission: self.permission_for(field),
            field: field.to_string(),
            id,
            // A field the project says a program should propose rather than
            // set opens in that mode. A person at a terminal is the person a
            // proposal would have been addressed to, so it is a default and
            // not a restriction — the toggle is right there.
            propose: self.permission_for(field) == crate::schema::Agent::Propose,
            tick: false,
        });
    }

    /// Go where a figure points.
    ///
    /// A set becomes the filter that produced it, on the list, because the
    /// list is where you act on a set. One item becomes that item, selected.
    /// `esc` comes back out of either, which it already did.
    /// Where the work under a container stands.
    ///
    /// A percentage answers *how far along*, which for a milestone is the
    /// least useful thing about it. The question a reader actually has is
    /// where the remaining work **is** — and four numbers answer it in less
    /// room than a bar took.
    ///
    /// Transitive, matching the rollup the engine computes, so this and the
    /// count beside a group heading never disagree.
    pub fn rollup(&self, item: &Item) -> Rollup {
        let mut out = Rollup::default();
        let mut seen = std::collections::HashSet::new();
        let mut stack = item.contains.clone();
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(child) = self.by_id.get(&id).and_then(|&i| self.items.get(i)) else {
                continue;
            };
            stack.extend(child.contains.iter().copied());
            match self.schema.category(&child.status) {
                Category::Done | Category::Dropped => out.done += 1,
                Category::Active => out.active += 1,
                Category::Open => out.open += 1,
            }
            if child.blocked {
                out.blocked += 1;
            }
        }
        out
    }

    /// Follow something in the detail pane: out to the browser, or across to
    /// another item.
    pub fn follow(&mut self, n: usize) -> Action {
        match self.links.get(n).cloned() {
            Some(Target::Url(url)) => {
                self.toast(format!("opening {url}"), ToastKind::Info);
                Action::Open(url)
            }
            Some(Target::Item(id)) => {
                let shown = self.by_id.get(&id).is_some_and(|&i| {
                    self.rows
                        .iter()
                        .any(|r| matches!(r, Row::Item(j) if *j == i))
                });
                if shown {
                    self.select_id(id);
                } else {
                    // It exists — it is a blocker — but the filter in force is
                    // hiding it, and jumping nowhere in silence is worse than
                    // saying why nothing happened.
                    self.toast(
                        format!("{} is not in this view", self.schema.format_id(id)),
                        ToastKind::Info,
                    );
                }
                Action::None
            }
            None => Action::None,
        }
    }

    pub fn open_door(&mut self, n: usize) -> Action {
        let Some(door) = self.doors.get(n).cloned() else {
            return Action::None;
        };
        self.figure = n;
        match door {
            Door::Filter(filter) => {
                self.filter = filter;
                self.pane = Pane::List;
                self.reparse_filter();
                self.rebuild();
                self.jump(false);
                if self.rows.is_empty() {
                    self.toast("nothing matches that any more", ToastKind::Info);
                }
            }
            Door::Item(id) => {
                self.pane = Pane::List;
                self.rebuild();
                self.select_id(id);
            }
        }
        Action::None
    }

    /// What the project allows a program to do with a field.
    ///
    /// `status` is a field like any other to the picker, but it is declared
    /// on the status table rather than in `[[field]]`, so both are asked.
    fn permission_for(&self, field: &str) -> crate::schema::Agent {
        if field == "status" {
            // The strictest thing any status says, because the picker offers
            // all of them and the answer has to cover the one you pick.
            return self
                .schema
                .statuses
                .iter()
                .map(|s| s.agent)
                .max_by_key(|a| match a {
                    crate::schema::Agent::Unrestricted => 0,
                    crate::schema::Agent::Propose => 1,
                    crate::schema::Agent::ReadOnly => 2,
                })
                .unwrap_or_default();
        }
        self.schema
            .field(field)
            .map(|f| f.agent)
            .unwrap_or_default()
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
        if picker.tick {
            let reference = self.schema.format_id(picker.id);
            let what = picker
                .options
                .get(picker.selected)
                .map(|(_, label, _)| label.clone())
                .unwrap_or_default();
            return self.write(
                Change {
                    args: vec!["tick".into(), picker.id.to_string(), value],
                    describe: format!(
                        "{reference} · {}",
                        what.chars().take(44).collect::<String>()
                    ),
                    undo: None,
                },
                1,
                "tick",
            );
        }
        if picker.propose {
            // A proposal without a reason is a preference rather than an
            // argument, which is cairn's phrasing and the reason to ask.
            self.proposing = Some((picker.id, picker.field, value));
            self.editing = Some(Editing::Why);
            self.input.clear();
            return Action::None;
        }
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
            // One line, and the gesture being cheap is the value: a note
            // nobody writes because it costs an editor round trip is a note
            // that does not exist. `e` is still there for a paragraph.
            Some(Editing::Note) => {
                let note = text.trim().to_string();
                let Some(id) = self.selected_item().map(|i| i.id) else {
                    return Action::None;
                };
                if note.is_empty() {
                    return Action::None;
                }
                // Appended by cairn, never written here: a caller recording
                // its reasoning must not be able to erase what came before,
                // which is the whole reason `note` exists rather than a body
                // that replaces. Under today's date, which is cairn's own
                // default and makes the body read as a log.
                self.write(
                    Change {
                        args: vec!["note".into(), id.to_string(), note],
                        describe: format!("{} noted", self.schema.format_id(id)),
                        undo: None,
                    },
                    1,
                    "note",
                )
            }
            // Skipping it hands the item back the way it always did: a
            // prompt on a one-keystroke gesture has to be answerable with one
            // keystroke, or it gets muscle-memoried past, which is worse than
            // not asking.
            Some(Editing::Why) => {
                let Some((id, field, value)) = self.proposing.take() else {
                    return Action::None;
                };
                let reference = self.schema.format_id(id);
                let why = text.trim().to_string();
                let mut args = vec![
                    "propose".to_string(),
                    id.to_string(),
                    format!("{field}={value}"),
                ];
                if !why.is_empty() {
                    args.push("--why".into());
                    args.push(why);
                }
                self.write(
                    Change {
                        args,
                        describe: format!("{reference} {field} → {value} proposed"),
                        undo: None,
                    },
                    1,
                    "propose",
                )
            }
            Some(Editing::Reason) => self.release_with(Some(text.trim()).filter(|t| !t.is_empty())),
            Some(Editing::NewItem) => {
                let title = text.trim().to_string();
                if title.is_empty() {
                    return Action::None;
                }
                if let Some(why) = self.readonly.clone() {
                    self.refuse(&why);
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

    pub fn moment(&self) -> usize {
        self.moment
    }

    /// Move the log's cursor.
    pub fn select_moment(&mut self, n: usize) {
        let len = self.moments().len();
        if len > 0 {
            self.moment = n.min(len - 1);
        }
    }

    /// What the log lens is showing, which is nothing until it has asked.
    ///
    /// The filter applies, because *what happened* means *what happened to
    /// what I am looking at* — the same set the board is dealt from. An item
    /// the repository remembers and the backlog no longer has is dropped: a
    /// row whose detail cannot be shown is a row that breaks the contract.
    pub fn moments(&self) -> Vec<&Moment> {
        let Some(Ok(moments)) = &self.moments else {
            return Vec::new();
        };
        moments
            .iter()
            .filter(|m| self.items.iter().any(|i| i.id == m.id && self.on_board(i)))
            .collect()
    }

    /// Read what the repository said.
    ///
    /// `git log` was asked for `%h\x1f%an\x1f%aI\x1f%s` and then the paths,
    /// so a line with three separators begins a commit and everything after
    /// it until the blank line is what that commit touched.
    ///
    /// One record per item per commit: a commit that moved four items is
    /// four lines, because the reader is asking about items, not commits.
    pub fn show_activity(&mut self, result: Result<String, String>) {
        self.asking_activity = false;
        self.moment = 0;
        self.moments = Some(result.map(|text| {
            let mut out: Vec<Moment> = Vec::new();
            let mut head: Option<(String, String, String)> = None;
            for line in text.lines() {
                let fields: Vec<&str> = line.split('\u{1f}').collect();
                if let [_hash, who, when, what] = fields.as_slice() {
                    head = Some((
                        // The day, which is the resolution a person reads a
                        // backlog at. The time of day answers nothing here.
                        when.split('T').next().unwrap_or(when).to_string(),
                        who.to_string(),
                        what.to_string(),
                    ));
                    continue;
                }
                let Some((when, who, what)) = &head else {
                    continue;
                };
                let Some(id) = crate::item::id_from_path(std::path::Path::new(line.trim())) else {
                    continue;
                };
                // A commit that touches one item twice — renamed and edited
                // — is still one thing happening to it.
                if out
                    .iter()
                    .any(|m| m.id == id && m.when == *when && m.what == *what)
                {
                    continue;
                }
                out.push(Moment {
                    id,
                    when: when.clone(),
                    who: who.clone(),
                    what: what.clone(),
                });
            }
            out
        }));
    }

    /// Take what cairn said about the project.
    ///
    /// Every line kept, including the one that says it passed: an empty
    /// section under a heading that says a validator ran is indistinguishable
    /// from a validator that did not.
    pub fn show_check(&mut self, result: Result<String, String>) {
        self.checked = Some(result.map(|out| {
            out.lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        }));
    }

    /// Take what cairn said about an item's history.
    pub fn show_history(&mut self, id: u32, result: Result<String, String>) {
        self.history = Some(match result {
            Ok(text) => History {
                id,
                lines: text.lines().map(str::to_string).collect(),
                scroll: 0,
                unavailable: None,
            },
            Err(why) => History {
                id,
                lines: Vec::new(),
                scroll: 0,
                unavailable: Some(why),
            },
        });
    }

    pub fn toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toast = Some((msg.into(), kind, Instant::now()));
    }

    /// Called once per frame by the shell.
    pub fn tick_clock(&mut self) {
        self.now = unix_seconds();
    }

    /// What the interface needs before it can draw something true.
    ///
    /// Asked every pass rather than on arriving at a lens. Arriving is not the
    /// only way to end up on the log with nothing to show: a re-read throws
    /// away what the repository said, and the watcher makes that happen
    /// whenever anybody touches the backlog — so sitting on the log while an
    /// agent worked in the other pane left it saying "asking the repository"
    /// at nobody, forever. Opening straight onto the log did the same, because
    /// no lens was ever arrived at.
    pub fn pending(&mut self) -> Option<Action> {
        if self.pane == Pane::Log && self.moments.is_none() && !self.asking_activity {
            self.asking_activity = true;
            return Some(Action::Activity);
        }
        None
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
        if let Some(history) = self.history.as_mut() {
            match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    history.scroll = history.scroll.saturating_add(1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    history.scroll = history.scroll.saturating_sub(1)
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    history.scroll = history.scroll.saturating_add(10)
                }
                KeyCode::PageUp => history.scroll = history.scroll.saturating_sub(10),
                _ => self.history = None,
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
                | Command::Note
                | Command::Tick
        );
        if writes && let Some(why) = self.readonly.clone() {
            self.refuse(&why);
            return Action::None;
        }

        // While the reader has the keys, the movement commands move the text
        // rather than the cursor. Everything else still acts on the selected
        // item, because the panel is beside the backlog rather than over it.
        // The panel takes the keys that move a cursor while it holds focus,
        // the way the reader does. Everything else still reaches the backlog.
        if self.focus == Focus::Filter && self.filtering {
            let last = self.facet_count().saturating_sub(1);
            match command {
                Command::Down => {
                    self.facet = (self.facet + 1).min(last);
                    return Action::None;
                }
                Command::Up => {
                    self.facet = self.facet.saturating_sub(1);
                    return Action::None;
                }
                Command::PageDown => {
                    self.facet = (self.facet + 10).min(last);
                    return Action::None;
                }
                Command::PageUp => {
                    self.facet = self.facet.saturating_sub(10);
                    return Action::None;
                }
                Command::First => {
                    self.facet = 0;
                    return Action::None;
                }
                Command::Last => {
                    self.facet = last;
                    return Action::None;
                }
                // `space` is the tick here, which is what it is in the list
                // for a mark: the same gesture for the same kind of choice.
                Command::ToggleGroup => {
                    self.toggle_facet();
                    return Action::None;
                }
                Command::Back => {
                    self.filtering = false;
                    self.focus = Focus::List;
                    return Action::None;
                }
                _ => {}
            }
        }
        if self.focus == Focus::Reader
            && self.reading
            && let Some(id) = self.selected_item().map(|i| i.id)
        {
            match command {
                Command::Down => {
                    self.reader.by(id, 1);
                    return Action::None;
                }
                Command::Up => {
                    self.reader.by(id, -1);
                    return Action::None;
                }
                Command::PageDown => {
                    self.reader.by(id, 10);
                    return Action::None;
                }
                Command::PageUp => {
                    self.reader.by(id, -10);
                    return Action::None;
                }
                Command::First => {
                    self.reader.to(id, 0);
                    return Action::None;
                }
                Command::Last => {
                    self.reader.to(id, u16::MAX);
                    return Action::None;
                }
                _ => {}
            }
        }

        match command {
            Command::Quit => return Action::Quit,
            Command::Back => {
                // Backing out closes what is open; with nothing open it does
                // nothing, rather than quitting out from under you. Marks go
                // first: they are the most recent thing you did.
                //
                // The panel and the keys in it are one thing, not two. Handing
                // the keys back and leaving the panel open was meant to make
                // `esc j j ↵` cheap, and it is exactly the same four keys
                // either way — so all the extra step bought was an `esc` that
                // looked like it had not worked.
                if self.reading {
                    self.reading = false;
                    self.focus = Focus::List;
                } else if !self.marked.is_empty() {
                    let n = self.marked.len();
                    self.marked.clear();
                    self.toast(format!("{n} unmarked"), ToastKind::Info);
                } else if self.view.is_some() {
                    self.view = None;
                    self.follow_view();
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
            // The keyboard's way into the pane the pointer would scroll. A
            // line at a time, because the wheel is already the coarse gesture.
            Command::DetailDown | Command::DetailUp => {
                let delta = if command == Command::DetailDown {
                    1
                } else {
                    -1
                };
                if let Some(id) = self.selected_item().map(|i| i.id) {
                    // One gesture, whichever pane the item is being read in.
                    if self.reading {
                        self.reader.by(id, delta);
                    } else {
                        self.detail.by(id, delta);
                    }
                }
            }
            // On a heading, fold. On an item, mark it — which is where the
            // gesture is going anyway once there is more than one thing to do.
            Command::ToggleGroup => match self.rows.get(self.selected) {
                Some(Row::Group(_)) if self.pane == Pane::List => self.toggle_group(),
                _ => self.toggle_mark(),
            },
            Command::PrevGroup => self.step_group(false),
            Command::NextGroup => self.step_group(true),
            // Positional, in the order the tabs are in, so what you see is
            // what you count. Out of range does nothing rather than
            // wrapping: `9` on a five-lens screen meant nothing.
            Command::ViewLens(n) => {
                let Some(&lens) = Pane::ALL.get(usize::from(n).saturating_sub(1)) else {
                    return Action::None;
                };
                if lens == self.pane {
                    return Action::None;
                }
                let id = self.selected_item().map(|i| i.id);
                self.pane = lens;
                if let Some(id) = id {
                    self.select_id(id);
                }
                self.clamp();
            }
            Command::ViewBoard | Command::ViewBack => {
                let id = self.selected_item().map(|i| i.id);
                self.pane = if command == Command::ViewBack {
                    self.pane.previous()
                } else {
                    self.pane.next()
                };
                // The selection is the same item in every lens that has one,
                // which is the first thing a lens owes the reader.
                if let Some(id) = id {
                    self.select_id(id);
                }
                self.clamp();
            }
            Command::GroupBy => self.cycle_grouping(),
            // On the stats pane, `enter` is how you follow a figure to what
            // it counted; everywhere else it reads the selected item.
            Command::Read if self.pane == Pane::Stats && !self.doors.is_empty() => {
                return self.open_door(self.figure);
            }
            // On the log, `enter` goes from *this item changed* to *how this
            // item got the way it is*, which is the overlay that exists.
            Command::Read if self.pane == Pane::Log => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                return Action::History(item.id);
            }
            // Opening it and focusing it are one gesture: a panel you have to
            // open and then reach for is two.
            Command::Read => {
                if self.selected_item().is_some() {
                    self.reading = true;
                    self.focus = Focus::Reader;
                }
            }
            // A proposal is somebody asking; accepting is cairn's own command,
            // so the change lands the way it would have if they had the say.
            Command::Accept => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                let Some(proposal) = item.proposals.last().cloned() else {
                    self.toast("nothing proposed on this one", ToastKind::Info);
                    return Action::None;
                };
                let (id, reference) = (item.id, self.schema.format_id(item.id));
                self.confirm = Some(Confirm {
                    prompt: format!(
                        "{reference} · {} {} → {}?",
                        proposal.field, proposal.from, proposal.to
                    ),
                    detail: if proposal.why.is_empty() {
                        format!("{} proposed it", proposal.by)
                    } else {
                        format!("{}: {}", proposal.by, proposal.why)
                    },
                    change: Change {
                        args: vec!["proposals".into(), "--accept".into(), id.to_string()],
                        describe: format!("{reference} {} → {}", proposal.field, proposal.to),
                        undo: Some(format!(
                            "cairn set {id} {}={}",
                            proposal.field, proposal.from
                        )),
                    },
                });
            }
            Command::History => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                return Action::History(item.id);
            }
            // On demand, never on load: `check` is a process, harrow reloads
            // on every file change, and a validator on a watch is a validator
            // that runs three times a keystroke.
            Command::Check => {
                self.diagnostics = true;
                return Action::Check;
            }
            Command::Edit => {
                if let Some(item) = self.selected_item() {
                    return Action::Edit(item.path.clone());
                }
            }
            // A modifier on the gesture that already exists rather than a
            // second set of keys, so there is one way to choose a value and
            // the confirmation says which of the two it is about to do.
            Command::Propose => match self.picker.as_mut() {
                Some(picker) => {
                    picker.propose = !picker.propose;
                }
                None => self.toast(
                    "open a picker first — proposing is a way of choosing",
                    ToastKind::Info,
                ),
            },
            // Reusing the picker rather than inventing a cursor in the
            // detail pane: choosing one of a small numbered set is exactly
            // what it is for, and `cairn tick` takes the same number.
            Command::Tick => {
                if !self.can_tick {
                    self.toast(
                        "this cairn cannot tick a criterion — `cairn tick` is newer",
                        ToastKind::Info,
                    );
                    return Action::None;
                }
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                let (id, reference) = (item.id, self.schema.format_id(item.id));
                let all =
                    crate::item::criteria_in(&item.body, self.schema.criteria_section.as_deref());
                if all.is_empty() {
                    self.toast("this one has no acceptance criteria", ToastKind::Info);
                    return Action::None;
                }
                let options: Vec<(String, String, String)> = all
                    .iter()
                    .enumerate()
                    .map(|(n, (ticked, text))| {
                        (
                            (n + 1).to_string(),
                            text.clone(),
                            if *ticked { "ticked" } else { "" }.to_string(),
                        )
                    })
                    .collect();
                // The first one that is not true yet, which is the one
                // somebody reaching for this key almost always means.
                let selected = all.iter().position(|(t, _)| !t).unwrap_or(0);
                self.picker = Some(Picker {
                    title: format!("tick on {reference}"),
                    options,
                    selected,
                    permission: crate::schema::Agent::default(),
                    field: "criteria".to_string(),
                    id,
                    propose: false,
                    tick: true,
                });
            }
            Command::Claim => return self.claim(true),
            Command::Release => return self.claim(false),
            Command::Close => self.ask_close(),
            Command::Reopen => {
                let targets: Vec<u32> = self
                    .targets()
                    .into_iter()
                    .filter(|id| {
                        self.by_id
                            .get(id)
                            .and_then(|i| self.items.get(*i))
                            .is_some_and(|item| item.category.is_closed())
                    })
                    .collect();
                return match targets.as_slice() {
                    [] => {
                        self.toast("that one is already open", ToastKind::Info);
                        Action::None
                    }
                    [id] => {
                        let change = Change {
                            args: vec!["reopen".into(), id.to_string()],
                            describe: format!("{} reopened", self.schema.format_id(*id)),
                            undo: Some(format!("cairn close {id}")),
                        };
                        self.write(change, 1, "reopen")
                    }
                    many => {
                        let n = many.len();
                        let change = Change {
                            args: self.bulk_args("reopen", many, None),
                            describe: format!("{n} items reopened"),
                            undo: None,
                        };
                        self.write(change, n, "Reopen")
                    }
                };
            }
            Command::New => {
                self.editing = Some(Editing::NewItem);
                self.input.clear();
            }
            // One item, because `cairn note` takes one item. Marks are for
            // the changes cairn can make to a set at once, and saying so is
            // better than quietly noting whichever one the cursor was on.
            Command::Note => {
                if self.selected_item().is_none() {
                    return Action::None;
                }
                if !self.marked.is_empty() {
                    self.toast("a note goes on one item at a time", ToastKind::Info);
                    return Action::None;
                }
                self.editing = Some(Editing::Note);
                self.input.clear();
            }
            Command::Status => self.open_picker("status"),
            Command::Priority => self.open_picker("priority"),
            Command::Milestone => self.open_picker("milestone"),
            Command::Advance => return self.step_status(true),
            Command::Retreat => return self.step_status(false),
            // The whole view, as the command that would reproduce it. The
            // interface then teaches the command line rather than hiding it.
            Command::CopyView => return Action::Copy(self.command_line()),
            Command::Copy => {
                let Some(item) = self.selected_item() else {
                    return Action::None;
                };
                return Action::Copy(self.schema.format_id(item.id));
            }
            // The panel, which is the discoverable way in. The box behind
            // `/` still takes an expression no checkbox can express.
            Command::Facets => {
                self.filtering = !self.filtering;
                self.focus = if self.filtering {
                    Focus::Filter
                } else {
                    Focus::List
                };
            }
            Command::Filter => {
                self.editing = Some(Editing::Filter);
                self.input = self.filter.clone();
            }
            Command::ToggleAll => {
                self.show_all = !self.show_all;
                self.rebuild();
                // "All" is cairn's word and cairn's meaning: finished, dropped,
                // and the containers work belongs to.
                let msg = if self.show_all {
                    "showing everything"
                } else {
                    "showing open work only"
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

    /// Record where something clickable was drawn.
    ///
    /// Clipped to the screen, because a region beyond the edge is a click on
    /// something nobody can see. The header is where this bites: it lays the
    /// tabs out at fixed offsets and registers all of them, so on a
    /// forty-column terminal — a right-hand pane beside an editor — the last
    /// tab was truncated away and still clickable, and a click on the blank
    /// at the right edge switched to a lens that was not on screen.
    ///
    /// Here rather than at each call site, because there are a dozen of them
    /// and the next one would forget.
    pub fn hit(&mut self, area: Rect, what: Hit) {
        let screen = self.screen;
        let x = area.x.min(screen.width);
        let y = area.y.min(screen.height);
        let clipped = Rect {
            x,
            y,
            width: area.width.min(screen.width.saturating_sub(x)),
            height: area.height.min(screen.height.saturating_sub(y)),
        };
        if clipped.width > 0 && clipped.height > 0 {
            self.hits.push((clipped, what));
        }
    }

    /// What is under the pointer. Last drawn wins, so an overlay takes the
    /// click rather than the list behind it.
    pub fn hit_at(&self, column: u16, row: u16) -> Option<&Hit> {
        self.hits
            .iter()
            .rev()
            .find(|(area, _)| {
                column >= area.x
                    && column < area.x + area.width
                    && row >= area.y
                    && row < area.y + area.height
            })
            .map(|(_, what)| what)
    }

    /// Two clicks in the same place, close enough together to mean one gesture.
    fn is_double(&mut self, column: u16, row: u16) -> bool {
        let double = self.last_click.is_some_and(|(x, y, at)| {
            x == column && y == row && at.elapsed() < Duration::from_millis(400)
        });
        self.last_click = if double {
            None // A third click starts again rather than reading as a second.
        } else {
            Some((column, row, Instant::now()))
        };
        double
    }

    pub fn handle_mouse(&mut self, m: MouseEvent) -> Action {
        match m.kind {
            MouseEventKind::ScrollDown => self.scroll(3, m.column, m.row),
            MouseEventKind::ScrollUp => self.scroll(-3, m.column, m.row),
            MouseEventKind::Down(MouseButton::Left) => {
                let double = self.is_double(m.column, m.row);
                // The modifiers everything else on the screen uses for the
                // same two gestures.
                if m.modifiers.contains(KeyModifiers::SHIFT) {
                    if let Some(Hit::Row(idx)) = self.hit_at(m.column, m.row).cloned() {
                        self.mark_range(idx);
                        return Action::None;
                    }
                } else if m
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && let Some(Hit::Row(idx)) = self.hit_at(m.column, m.row).cloned()
                {
                    self.selected = idx;
                    self.clamp();
                    self.toggle_mark();
                    self.move_by(-1);
                    return Action::None;
                }
                return self.click(m.column, m.row, double);
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag(m.column, m.row),
            MouseEventKind::Up(MouseButton::Left) => return self.drop(m.column, m.row),
            _ => {}
        }
        Action::None
    }

    /// Scrolling moves the view, and takes the cursor with it only when the
    /// cursor would otherwise leave. Scrolling a list and watching the
    /// selection run away from the pointer is the thing that makes a terminal
    /// interface feel unlike everything else on the screen.
    ///
    /// Which view moves is decided by what the pointer is over, not by what
    /// holds the cursor: the panes are beside each other precisely so that one
    /// can be read without disturbing the other.
    fn scroll(&mut self, delta: isize, column: u16, row: u16) {
        if let Some(history) = self.history.as_mut() {
            history.scroll = history
                .scroll
                .saturating_add_signed(delta.clamp(-32, 32) as i16);
            return;
        }
        if self.reading
            && matches!(self.hit_at(column, row), Some(Hit::Reader))
            && let Some(id) = self.selected_item().map(|i| i.id)
        {
            self.reader.by(id, delta);
            return;
        }
        match self.pane {
            // One cursor, one list: scrolling and moving are the same thing
            // where every row is a question rather than a line of text.
            Pane::Needs | Pane::Log => self.move_by(delta.signum()),
            Pane::Stats => {
                self.stats_scroll = self
                    .stats_scroll
                    .saturating_add_signed(delta.clamp(-32, 32) as i16);
            }
            // The column the pointer is over, which is usually not the one the
            // cursor is in — peering into `done` should not move what you were
            // about to claim.
            Pane::Board => {
                let over = match self.hit_at(column, row) {
                    Some(Hit::Card(c, _) | Hit::Column(c)) => *c,
                    _ => self.column,
                };
                self.scroll_column(over, delta);
            }
            Pane::List => match self.hit_at(column, row) {
                Some(Hit::Detail) => {
                    if let Some(id) = self.selected_item().map(|i| i.id) {
                        self.detail.by(id, delta);
                    }
                }
                _ => self.scroll_list(delta),
            },
        }
    }

    fn scroll_list(&mut self, delta: isize) {
        let height = self.list_area.height.saturating_sub(2) as usize;
        let max = self.rows.len().saturating_sub(height);
        self.offset = self.offset.saturating_add_signed(delta).min(max);
        if self.selected < self.offset {
            self.selected = self.offset;
        } else if height > 0 && self.selected >= self.offset + height {
            self.selected = self.offset + height - 1;
        }
        self.clamp();
    }

    /// One board column, by the same rule as the list — except that only the
    /// focused column has a cursor to keep in view. The others are being
    /// looked at, not worked in.
    fn scroll_column(&mut self, index: usize, delta: isize) {
        let Some(count) = self.columns.get(index).map(|c| c.items.len()) else {
            return;
        };
        // A height of zero means the board has not been drawn yet, which is
        // not a reason to allow scrolling past the last card.
        let height = (self.board_area.height.saturating_sub(2) as usize).max(1);
        let max = count.saturating_sub(height);
        let Some(offset) = self.column_offsets.get_mut(index) else {
            return;
        };
        *offset = offset.saturating_add_signed(delta).min(max);
        if index != self.column || count == 0 {
            return;
        }
        let offset = *offset;
        if self.column_row < offset {
            self.column_row = offset;
        } else if height > 0 && self.column_row >= offset + height {
            self.column_row = offset + height - 1;
        }
        self.clamp();
    }

    fn click(&mut self, column: u16, row: u16, double: bool) -> Action {
        // An overlay covers what is behind it, so a click behind it is a
        // dismissal rather than a gesture meant for the surface it is
        // covering. `any other key closes` on its edge promises as much, and a
        // click was the one gesture that neither closed it nor was ignored.
        if self.history.is_some() {
            if !matches!(self.hit_at(column, row), Some(Hit::Overlay)) {
                self.history = None;
            }
            return Action::None;
        }
        let Some(what) = self.hit_at(column, row).cloned() else {
            return Action::None;
        };
        match what {
            Hit::Tab(pane) => {
                let id = self.selected_item().map(|i| i.id);
                self.pane = pane;
                if let Some(id) = id {
                    self.select_id(id);
                }
                self.clamp();
            }
            // Clicking a status is the fastest way to ask the only question a
            // strip invites: show me those.
            Hit::Status(name) => {
                let filter = format!("status={name}");
                self.filter = if self.filter == filter {
                    String::new()
                } else {
                    filter
                };
                self.reparse_filter();
                self.rebuild();
                let message = if self.filter.is_empty() {
                    "filter cleared".to_string()
                } else {
                    format!("filtered to {name}")
                };
                self.toast(message, ToastKind::Info);
            }
            Hit::Row(idx) => match self.rows.get(idx) {
                Some(Row::Group(_)) => {
                    self.focus = Focus::List;
                    self.selected = idx;
                    self.toggle_group();
                }
                Some(Row::Item(_)) => {
                    self.focus = Focus::List;
                    self.selected = idx;
                    if double {
                        self.reading = true;
                    }
                }
                None => {}
            },
            Hit::Column(index) => {
                self.column = index.min(self.columns.len().saturating_sub(1));
                self.column_row = 0;
                self.clamp();
            }
            Hit::Card(col, at) => {
                self.column = col;
                self.column_row = at;
                self.clamp();
                self.dragging = Some((at, Hit::Card(col, at)));
                if double {
                    self.reading = true;
                }
            }
            Hit::Option(index) => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.selected = index.min(picker.options.len().saturating_sub(1));
                }
                return self.resolve_picker(true);
            }
            Hit::Answer(yes) => return self.resolve_confirm(yes),
            Hit::Run(command) => return self.run(command),
            // Clicking a pane is how a pointer says which one it means, and it
            // is the same statement `↵` makes with a key.
            Hit::Reader => self.focus = Focus::Reader,
            // Clicking into a panel focuses it, which is what sends the keys
            // that move a cursor to the thing you are looking at.
            Hit::Filter => self.focus = Focus::Filter,
            Hit::Facet(n) => {
                self.focus = Focus::Filter;
                self.facet = n.min(self.facet_count().saturating_sub(1));
                self.toggle_facet();
            }
            Hit::Detail | Hit::Overlay => {}
            Hit::Figure(n) => return self.open_door(n),
            // Selecting it, not answering it: the answers are keys, and a
            // click that closed an item would be a click nobody meant.
            Hit::Question(n) => self.select_question(n),
            Hit::Moment(n) => self.select_moment(n),
            Hit::Link(n) => return self.follow(n),
        }
        Action::None
    }

    /// Dragging a card marks the column it is over, so the drop is predictable.
    fn drag(&mut self, column: u16, row: u16) {
        if self.dragging.is_none() || self.pane != Pane::Board {
            return;
        }
        if let Some(Hit::Card(col, _) | Hit::Column(col)) = self.hit_at(column, row).cloned() {
            self.column = col.min(self.columns.len().saturating_sub(1));
        }
    }

    /// A card dropped in another column is a status change, which is the one
    /// gesture a board exists for.
    fn drop(&mut self, column: u16, row: u16) -> Action {
        let Some((_, Hit::Card(from, at))) = self.dragging.take() else {
            return Action::None;
        };
        let Some(Hit::Card(to, _) | Hit::Column(to)) = self.hit_at(column, row).cloned() else {
            return Action::None;
        };
        if to == from {
            return Action::None;
        }
        let Some(item) = self
            .columns
            .get(from)
            .and_then(|c| c.items.get(at))
            .and_then(|i| self.items.get(*i))
        else {
            return Action::None;
        };
        let Some(value) = self.columns.get(to).map(|c| c.value.clone()) else {
            return Action::None;
        };
        let field = self.board_by.clone();
        if let Some(why) = self.readonly.clone() {
            self.refuse(&why);
            return Action::None;
        }
        let id = item.id;
        self.select_id(id);
        self.set_field(&field, &value)
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
                self.column_row, column.value
            ));
        }
        if self.column_offsets.len() != self.columns.len() {
            return Err(format!(
                "{} scroll offsets for {} columns",
                self.column_offsets.len(),
                self.columns.len()
            ));
        }
        for (index, offset) in self.column_offsets.iter().enumerate() {
            let len = self.columns[index].items.len();
            if *offset > 0 && *offset >= len {
                return Err(format!(
                    "column {} is scrolled to card {offset} of {len}",
                    self.columns[index].value
                ));
            }
        }
        // Every lens's cursor is an index into a `Vec` something else
        // rebuilds, which is exactly the shape of bug these exist to catch.
        for (lens, at, len) in [
            ("needs", self.question, self.questions.len()),
            ("stats", self.figure, self.doors.len()),
            ("log", self.moment(), self.moments().len()),
        ] {
            if len == 0 {
                if at != 0 {
                    return Err(format!("{lens} cursor at {at} with nothing to point at"));
                }
            } else if at >= len {
                return Err(format!("{lens} cursor at {at} of {len}"));
            }
        }
        // The anchor names an item, or it names nothing.
        if let Some(id) = self.needs_anchor
            && !self.items.iter().any(|i| i.id == id)
        {
            return Err(format!("the queue is holding item {id}, which is not here"));
        }
        // Whoever is in the cast has work here, and nobody is in it twice.
        for (n, who) in self.actors.iter().enumerate() {
            if self.actors[..n].contains(who) {
                return Err(format!("{who} is in the cast twice"));
            }
            if !self
                .items
                .iter()
                .any(|i| i.assignee.as_ref() == Some(who) || i.owner.as_ref() == Some(who))
            {
                return Err(format!("{who} is in the cast with no work here"));
            }
        }
        let heading_type = self.heading_type().map(str::to_string);
        let visible = self
            .items
            .iter()
            .filter(|i| self.visible(i))
            .filter(|i| self.is_row(i, heading_type.as_deref()))
            .count();
        if !matches!(self.group_by.as_str(), "none" | "") {
            let counted: usize = self.groups.iter().map(|g| g.shown).sum();
            if counted != visible {
                return Err(format!(
                    "group counts total {counted} but {visible} items are visible"
                ));
            }
        }
        // The board is dealt from its own set, not the list's, so it is held
        // to that one: every card is an item the board would deal.
        let dealt: usize = self.columns.iter().map(|c| c.items.len()).sum();
        let eligible = self.items.iter().filter(|i| self.on_board(i)).count();
        if dealt > eligible {
            return Err(format!(
                "the board shows {dealt} cards but only {eligible} items belong on it"
            ));
        }
        Ok(())
    }
}

/// What a backlog looks like from a distance.
///
/// Computed rather than stored, from what is on disk and the clock the frame
/// was drawn at, so none of it can go stale or disagree with the list.
pub struct Stats {
    pub total: usize,
    pub open: usize,
    pub active: usize,
    pub done: usize,
    pub dropped: usize,
    pub ready: usize,
    pub blocked: usize,
    pub claimed: usize,
    pub closed_recently: [(u32, usize); 3],
    pub criteria: (u32, u32),
    /// `(key, title, percent, left, due)`, in the order the roadmap runs.
    pub milestones: Vec<(String, String, u32, usize, Option<String>)>,
    pub by_type: Vec<(String, usize)>,
    /// One distribution per enum field the project marked as a column.
    pub by_field: Vec<(String, Vec<(String, usize)>)>,
    /// The open item that has been waiting longest, and for how many days.
    pub oldest: Option<(u32, String, i64)>,
    /// What the most things are waiting on.
    pub blocking: Option<(u32, String, usize)>,
}

impl App {
    /// Everything the statistics pane shows.
    pub fn stats(&self) -> Stats {
        // The filter applies here as it applies to the list and the board.
        // One backlog, one filter, three lenses — and a reader who has just
        // narrowed the list has every reason to think the next screen is
        // about what they narrowed to.
        //
        // Not `visible`: that is the list's rule, which also hides finished
        // work, and a summary that could not count what is done would have
        // nothing to say. The filter, and the rule that a container is not
        // work, which is the same pair the board is dealt from.
        let work: Vec<&Item> = self
            .items
            .iter()
            .filter(|i| !i.container && self.query.matches(i, &self.schema))
            .collect();
        let today = self.now / 86_400;
        let days_ago = |date: Option<&String>| {
            date.and_then(|d| days_from_iso(d))
                .map(|d| today as i64 - d as i64)
        };

        let mut closed_recently = [(7u32, 0usize), (30, 0), (90, 0)];
        for item in work.iter().filter(|i| i.category == Category::Done) {
            if let Some(age) = days_ago(item.updated.as_ref()) {
                for (window, count) in closed_recently.iter_mut() {
                    if age >= 0 && age <= i64::from(*window) {
                        *count += 1;
                    }
                }
            }
        }

        let mut criteria = (0, 0);
        for item in work.iter().filter(|i| !i.category.is_closed()) {
            let (done, total) = item.criteria();
            criteria.0 += done;
            criteria.1 += total;
        }

        let milestones = self
            .items
            .iter()
            .filter(|i| i.container)
            .map(|m| {
                let left = self
                    .items
                    .iter()
                    .filter(|i| {
                        !i.container
                            && !i.category.is_closed()
                            && m.key.as_deref().is_some_and(|k| i.milestone() == Some(k))
                    })
                    .count();
                (
                    m.key.clone().unwrap_or_else(|| self.schema.format_id(m.id)),
                    m.title.clone(),
                    m.progress().unwrap_or(0),
                    left,
                    m.field_str("due").map(str::to_string),
                )
            })
            .collect();

        let count_of = |f: &dyn Fn(&Item) -> bool| work.iter().filter(|i| f(i)).count();
        let mut by_type: Vec<(String, usize)> = self
            .schema
            .types
            .iter()
            .filter(|t| !self.schema.is_container(&t.name))
            .map(|t| (t.name.clone(), count_of(&|i| i.kind == t.name)))
            .filter(|(_, n)| *n > 0)
            .collect();
        by_type.sort_by_key(|(_, n)| std::cmp::Reverse(*n));

        let by_field = self
            .schema
            .fields
            .iter()
            .filter(|f| f.column && !f.values.is_empty())
            .map(|f| {
                let values = f
                    .values
                    .iter()
                    .map(|v| {
                        (
                            v.clone(),
                            count_of(&|i| {
                                !i.category.is_closed() && i.field_str(&f.name) == Some(v)
                            }),
                        )
                    })
                    .collect();
                (f.name.clone(), values)
            })
            .collect();

        let oldest = work
            .iter()
            .filter(|i| !i.category.is_closed())
            .filter_map(|i| days_ago(i.created.as_ref()).map(|age| (i, age)))
            .filter(|(_, age)| *age >= 0)
            .max_by_key(|(_, age)| *age)
            .map(|(i, age)| (i.id, i.title.clone(), age));

        // What the most things are waiting on. One item holding up six others
        // is the most useful sentence a backlog can say about itself.
        let blocking = work
            .iter()
            .filter(|i| !i.category.is_closed())
            .map(|i| {
                let n = work
                    .iter()
                    .filter(|o| !o.category.is_closed() && o.blockers.contains(&i.id))
                    .count();
                (i, n)
            })
            .filter(|(_, n)| *n > 0)
            .max_by_key(|(_, n)| *n)
            .map(|(i, n)| (i.id, i.title.clone(), n));

        Stats {
            total: work.len(),
            open: count_of(&|i| i.category == Category::Open),
            active: count_of(&|i| i.category == Category::Active),
            done: count_of(&|i| i.category == Category::Done),
            dropped: count_of(&|i| i.category == Category::Dropped),
            ready: count_of(&|i| i.ready(&self.schema)),
            blocked: count_of(&|i| i.blocked && !i.category.is_closed()),
            claimed: count_of(&|i| i.assignee.is_some() && !i.category.is_closed()),
            closed_recently: [
                (7, closed_recently[0].1),
                (30, closed_recently[1].1),
                (90, closed_recently[2].1),
            ],
            criteria,
            milestones,
            by_type,
            by_field,
            oldest,
            blocking,
        }
    }
}

/// Days since the epoch for `YYYY-MM-DD`, by Howard Hinnant's civil algorithm.
///
/// Worth the twelve lines: the alternative is a date library for one question,
/// and cairn writes exactly one date format.
fn days_from_iso(date: &str) -> Option<u64> {
    let mut parts = date.trim().splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = y - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    u64::try_from(era * 146_097 + doe - 719_468).ok()
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

#[cfg(test)]
mod date_tests {
    use super::days_from_iso;

    #[test]
    fn iso_dates_become_days_that_subtract_correctly() {
        assert_eq!(days_from_iso("1970-01-01"), Some(0));
        assert_eq!(days_from_iso("1970-01-02"), Some(1));
        assert_eq!(days_from_iso("2000-03-01"), Some(11017));
        // A leap day, and the day after it.
        let feb29 = days_from_iso("2024-02-29").expect("a real date");
        assert_eq!(days_from_iso("2024-03-01"), Some(feb29 + 1));
        // A year apart is a year apart.
        let a = days_from_iso("2026-09-11").expect("a");
        let b = days_from_iso("2025-09-11").expect("b");
        assert_eq!(a - b, 365);
    }

    #[test]
    fn anything_that_is_not_a_date_is_not_guessed_at() {
        for bad in ["", "today", "2026", "2026-13-01", "2026-01-99", "x-y-z"] {
            assert_eq!(days_from_iso(bad), None, "accepted {bad:?}");
        }
    }
}
