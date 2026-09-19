//! Where harrow opens.
//!
//! Row nought, on any project past its first milestone, is the oldest
//! finished work there is. Every session began with the same scroll: the
//! first thing the program did with a backlog was show you the part of it
//! nobody is going to touch again.

use harrow::app::{App, Row};
use harrow::engine::{Source, Static};
use harrow::keys::Command;
use harrow::schema::Category;
use harrow::{testkit, ui};

/// A backlog with history behind it and work ahead — which is what a project
/// looks like on any day somebody is working on it, and what the fixture is
/// too small to be.
fn with_history() -> App {
    let schema = testkit::schema();
    let mut items = Vec::new();
    // The milestones themselves, so a finished one is a container at a
    // hundred per cent and shows the work that closed it — which is what
    // puts any history above the seam at all.
    for (n, key) in ["v0.1", "v0.2", "v0.3", "v0.4"].iter().enumerate() {
        let mut m = testkit::item(100 + n as u32, key, "backlog");
        m.kind = "milestone".into();
        m.key = Some((*key).to_string());
        items.push(m);
    }
    let mut push = |id: u32, milestone: &str, status: &str| {
        let mut item = testkit::item(id, "a thing", status);
        item.fields.insert(
            "milestone".into(),
            harrow::item::Value::One(milestone.into()),
        );
        items.push(item);
    };
    // Two finished milestones, then one under way, then one untouched.
    // Long enough that the pane can put the seam a third down without
    // running out of backlog to scroll — which a short one cannot.
    for id in 1..=20 {
        push(id, "v0.1", "done");
    }
    for id in 41..=60 {
        push(id, "v0.2", "done");
    }
    for id in 21..=23 {
        push(id, "v0.3", "done");
    }
    push(24, "v0.3", "doing");
    for id in 25..=27 {
        push(24 + id - 24, "v0.3", "backlog");
    }
    // And enough ahead that the pane is not simply showing the end of the
    // backlog: a third down is only reachable with two thirds still below.
    for id in 61..=90 {
        push(id, "v0.4", "backlog");
    }
    let mut source = Static { schema, items };
    let report = source.load().expect("it loads");
    let mut app = App::new();
    app.ingest(report);
    app
}

fn row_at(app: &App, n: usize) -> String {
    match &app.rows[n] {
        Row::Group(g) => format!("group {}", app.groups[*g].key),
        Row::Finished(g) => format!("fold {}", app.groups[*g].key),
        Row::Item(i) => format!("item {}", app.items[*i].id),
    }
}

#[test]
fn it_opens_where_the_work_is_rather_than_at_the_beginning() {
    let app = with_history();
    let (seam, _) = app.frontier().expect("a frontier");
    assert_eq!(row_at(&app, seam), "fold v0.3");
    assert_ne!(seam, 0, "it opened at the top of the project");
}

/// The cursor goes on the row a key would act on, not on the fold.
#[test]
fn the_cursor_lands_on_something_a_key_would_act_on() {
    let app = with_history();
    let item = app.selected_item().expect("something is selected");
    assert_eq!(item.id, 24);
    assert_eq!(item.category, Category::Active, "it is the work under way");
}

/// One forgotten item in an old milestone must not pin the reader to the past.
#[test]
fn work_under_way_outranks_work_merely_unstarted() {
    let mut app = with_history();
    // Something left open, two milestones back.
    let stale = app.items.iter().position(|i| i.id == 3).expect("0003");
    app.items[stale].status = "backlog".into();
    app.items[stale].category = Category::Open;
    app.rebuild();
    app.go_to_the_work();

    let (seam, _) = app.frontier().expect("a frontier");
    assert_eq!(
        row_at(&app, seam),
        "fold v0.3",
        "one stale item dragged it back to v0.1"
    );

    // With nothing under way, the first group with open work is the answer.
    let doing = app.items.iter().position(|i| i.id == 24).expect("0024");
    app.items[doing].status = "backlog".into();
    app.items[doing].category = Category::Open;
    app.rebuild();
    assert_eq!(row_at(&app, app.frontier().unwrap().0), "fold v0.1");
}

/// Two thirds of the pane for what is ahead, a third for what got you here.
#[test]
fn the_seam_sits_about_a_third_down_the_pane() {
    let mut app = with_history();
    let (seam, _) = app.frontier().expect("a frontier");
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    // 30 rows, less two of chrome, a rule and a footer, less the pane's own
    // border: about 24 of list.
    let on_screen = seam.saturating_sub(app.offset);
    assert!(
        (6..=10).contains(&on_screen),
        "the seam is {on_screen} rows down a pane about 24 tall"
    );

    // And the end of the backlog is what a short one shows instead, because
    // there is no scrolling past it.
    let mut short = testkit::app();
    let _ = ui::render_frame(&mut short, 110, 30, 0);
    assert_eq!(short.offset, 0, "a backlog shorter than the pane scrolled");
}

/// Homing on every rebuild would throw the scroll back on every claim, every
/// close and every keystroke into the filter box.
#[test]
fn a_reload_does_not_throw_you_back() {
    let mut app = with_history();
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    app.selected = app.rows.len() - 1;
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    let (was, offset) = (app.selected, app.offset);

    app.rebuild();
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    assert_eq!((app.selected, app.offset), (was, offset));
}

/// And the same place afterwards, by name — the gesture that opens the
/// program is one worth repeating after wandering.
#[test]
fn it_is_reachable_by_name() {
    let mut app = with_history();
    let home = app.selected;
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    app.selected = app.rows.len() - 1;
    let _ = ui::render_frame(&mut app, 110, 30, 0);

    app.run(Command::Frontier);
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    assert_eq!(app.selected, home);

    // With no key, so it costs the keymap nothing.
    assert!(
        harrow::keys::Keymap::default()
            .keys_for(Command::Frontier)
            .is_empty()
    );
}

/// A backlog with nothing finished opens at the top, because the top is where
/// the work is; one with nothing open opens somewhere rather than nowhere.
#[test]
fn a_backlog_at_either_end_opens_somewhere_sensible() {
    let mut fresh = testkit::app();
    fresh.filter = "category!=done".into();
    fresh.ingest(testkit::report());
    assert!(fresh.frontier().is_some(), "nothing to open on");

    let done = testkit::finished();
    let _ = done.frontier();
    assert!(done.selected < done.rows.len().max(1));
}
