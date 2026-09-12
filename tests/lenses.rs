//! What a lens owes the reader, held for every lens there is.
//!
//! harrow has three ways of looking at one backlog, and `tab` presents them as
//! peers. They drifted apart one capability at a time — the detail pane never
//! reached the board, the filter never reached the stats, the mouse never
//! reached either — and each difference arrived without an argument for it.
//!
//! The rule this file exists to hold: **the arrangement is the only thing that
//! differs**. A reader carries an intention across `tab` — this item, this
//! filter, these marks — and everything except the shape on screen should
//! survive the trip.
//!
//! It was built with a `GAPS` table — four lenses-by-capability that did not
//! hold yet, each naming the item that would close it, asserted as still
//! broken so neither forgetting one nor fixing one quietly could pass. The
//! table is gone because every row was paid off. What is left is the plain
//! statement: all of it, for all of them.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Pane, Row};
use harrow::keys::Command;
use harrow::{testkit, ui};

/// A capability every lens is supposed to have.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Owes {
    /// A filter narrows what the lens shows.
    ObeysTheFilter,
    /// Something the lens draws can be clicked.
    AnswersTheMouse,
    /// The detail of the selected item is reachable.
    ShowsTheDetail,
    /// The selection is the same item after a round trip.
    KeepsTheSelection,
    /// Marks survive, and mean the same thing.
    KeepsTheMarks,
}

/// Wide enough that nothing is dropped for want of room: every lens is being
/// asked what it does when it has the space, not what it does when squeezed.
const WIDE: (u16, u16) = (140, 34);

fn app_on(lens: Pane) -> App {
    let mut app = testkit::app();
    app.loading = false;
    app.last_load = None;
    // A fixed day, so nothing measured against the clock — how long a claim
    // has been held, what closed this week — answers differently tomorrow.
    app.now = 1_789_084_800;
    app.rebuild();
    app.pane = lens;
    app.select_id(3);
    app
}

fn holds(lens: Pane, owes: Owes) -> bool {
    match owes {
        Owes::ObeysTheFilter => {
            let mut app = app_on(lens);
            let before = shown(&mut app, lens);
            // Through the keys, because that is the only way a reader has of
            // narrowing anything, and the point is what they then see.
            app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
            for c in "priority=p0".chars() {
                app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
            }
            app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
            shown(&mut app, lens) < before
        }
        Owes::AnswersTheMouse => {
            let mut app = app_on(lens);
            let _ = ui::render_frame(&mut app, WIDE.0, WIDE.1, 0);
            // Somewhere below the header and above the footer: the lens's own
            // ground, rather than the tabs and hints that frame every lens.
            app.hits
                .iter()
                .any(|(area, _)| area.y >= 4 && area.y + area.height < WIDE.1 - 1)
        }
        Owes::ShowsTheDetail => {
            let mut app = app_on(lens);
            let screen = ui::render_to_string(&mut app, WIDE.0, WIDE.1, 0);
            // A heading only the detail pane draws, for the item selected.
            screen.contains("Fields")
        }
        Owes::KeepsTheSelection => {
            let mut app = app_on(lens);
            let was = app.selected_item().map(|i| i.id);
            for _ in 0..Pane::ALL.len() {
                app.run(Command::ViewBoard);
            }
            was.is_some() && app.selected_item().map(|i| i.id) == was
        }
        Owes::KeepsTheMarks => {
            let mut app = app_on(lens);
            app.run(Command::ToggleGroup);
            let marked = app.marked.clone();
            for _ in 0..Pane::ALL.len() {
                app.run(Command::ViewBoard);
            }
            !marked.is_empty() && app.marked == marked
        }
    }
}

/// How much of the backlog a lens is showing.
fn shown(app: &mut App, lens: Pane) -> usize {
    match lens {
        Pane::List => app
            .rows
            .iter()
            .filter(|r| matches!(r, Row::Item(_)))
            .count(),
        Pane::Board => app.columns.iter().map(|c| c.items.len()).sum(),
        Pane::Stats => app.stats().total,
        Pane::Needs => app.questions.len(),
    }
}

#[test]
fn every_lens_owes_the_reader_the_same_things() {
    const EVERYTHING: [Owes; 5] = [
        Owes::ObeysTheFilter,
        Owes::AnswersTheMouse,
        Owes::ShowsTheDetail,
        Owes::KeepsTheSelection,
        Owes::KeepsTheMarks,
    ];

    let mut broken = Vec::new();
    for lens in Pane::ALL {
        for owes in EVERYTHING {
            if !holds(lens, owes) {
                broken.push(format!("the {} lens no longer {owes:?}", lens.name()));
            }
        }
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

/// Adding a lens means adding it here, and the contract is what it has to
/// meet before it is one.
#[test]
fn the_contract_covers_every_lens_there_is() {
    assert_eq!(
        Pane::ALL.len(),
        4,
        "a lens was added or removed — the contract above covers every one"
    );
}
