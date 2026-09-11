//! Reading a project, from the files on disk to the rows on screen.

use harrow::app::{Action, App, Row};
use harrow::engine::{Project, Source};
use harrow::testkit;

fn app_for(dir: &std::path::Path) -> App {
    let mut project = Project::discover(dir).expect("the project is found");
    let report = project.load().expect("it loads");
    let mut app = App::new();
    app.ingest(report);
    app
}

#[test]
fn a_directory_of_markdown_becomes_a_backlog() {
    let dir = testkit::project();
    let app = app_for(dir.path());
    assert_eq!(app.schema.name, "sample");
    assert_eq!(app.items.len(), 6);
    // Grouped by milestone, and the milestone itself is the heading rather
    // than a row under one.
    assert!(
        app.groups.iter().any(|g| g.key == "v0.1"),
        "expected a milestone group"
    );
    assert!(
        !app.rows
            .iter()
            .any(|r| matches!(r, Row::Item(i) if app.items[*i].is_milestone())),
        "a milestone is a heading, not a row"
    );
}

#[test]
fn a_group_counts_what_the_filter_is_hiding() {
    let dir = testkit::project();
    let app = app_for(dir.path());
    let group = app.groups.iter().find(|g| g.key == "v0.1").expect("v0.1");
    assert_eq!(group.count, 4, "four items are scheduled against v0.1");
    assert_eq!(group.shown, 3, "one of them is finished and hidden");
    assert_eq!(group.percent(&app.items), 25, "and the bar says so anyway");
}

/// cairn keeps containers out of `next`, the board, the roadmap's item lists
/// and an ordinary `list`. harrow shows the same set, whatever it is grouped by.
#[test]
fn a_milestone_is_not_a_row_whatever_the_grouping_is() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    for axis in ["milestone", "status", "type", "priority", "none"] {
        app.group_by = axis.to_string();
        app.rebuild();
        assert!(
            !app.rows
                .iter()
                .any(|r| matches!(r, Row::Item(i) if app.items[*i].is_milestone())),
            "a milestone appeared as a row when grouped by {axis}"
        );
    }
}

#[test]
fn a_milestone_comes_back_when_you_ask_for_it() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    let milestones = |app: &App| {
        app.rows
            .iter()
            .filter(|r| matches!(r, Row::Item(i) if app.items[*i].is_milestone()))
            .count()
    };
    app.group_by = "status".into();
    app.rebuild();
    assert_eq!(milestones(&app), 0);

    // `--all` means all, which is cairn's word and cairn's meaning.
    app.run(harrow::keys::Command::ToggleAll);
    assert_eq!(milestones(&app), 1, "a is everything");

    // And asking for the type by name, which is what `cairn list -t` does.
    app.run(harrow::keys::Command::ToggleAll);
    type_filter(&mut app, "type=milestone");
    assert_eq!(milestones(&app), 1, "asked for by name");
}

fn type_filter(app: &mut App, expr: &str) {
    use crossterm::event::{KeyCode, KeyModifiers};
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    for c in expr.chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
}

#[test]
fn the_composition_graph_is_queryable() {
    let dir = testkit::project();
    let app = app_for(dir.path());
    let milestone = app.items.iter().find(|i| i.id == 1).expect("the milestone");
    assert_eq!(milestone.contains, vec![2, 3, 4, 5], "what belongs to it");
    assert_eq!(milestone.depth, 0, "it is a root");
    assert!(!milestone.is_leaf());
    assert!(milestone.container, "a reference field names its type");

    let leaf = app.items.iter().find(|i| i.id == 3).expect("item 3");
    assert_eq!(leaf.depth, 1, "one level under the milestone");
    assert!(leaf.is_leaf());
    assert!(!leaf.container);
}

#[test]
fn pressing_a_shows_what_is_finished() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    let before = app.rows.len();
    app.run(harrow::keys::Command::ToggleAll);
    assert!(app.rows.len() > before, "closed items should have appeared");
    app.run(harrow::keys::Command::ToggleAll);
    assert_eq!(app.rows.len(), before);
}

#[test]
fn the_cursor_stays_on_the_same_item_when_the_backlog_is_re_read() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.select_id(5);
    assert_eq!(app.selected_item().map(|i| i.id), Some(5));

    let mut project = Project::discover(dir.path()).expect("found");
    app.ingest(project.load().expect("loads"));
    assert_eq!(
        app.selected_item().map(|i| i.id),
        Some(5),
        "a refresh must not move the cursor"
    );
}

#[test]
fn a_collapsed_milestone_still_has_an_item_behind_it() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    // Land on the heading and fold it.
    app.selected = 0;
    app.toggle_group();
    assert!(matches!(app.rows[app.selected], Row::Group(_)));
    assert_eq!(
        app.selected_item().map(|i| i.id),
        Some(1),
        "the heading names the milestone, so that is what is selected"
    );
}

#[test]
fn grouping_cycles_through_what_the_project_actually_has() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    let axes = app.grouping_axes();
    assert!(axes.contains(&"milestone".to_string()));
    assert!(axes.contains(&"priority".to_string()), "{axes:?}");
    assert!(axes.contains(&"none".to_string()));
    assert!(
        !axes.contains(&"sprint".to_string()),
        "nothing the project has not got"
    );

    let start = app.group_by.clone();
    for _ in 0..axes.len() {
        app.cycle_grouping();
        assert!(app.check_invariants().is_ok(), "{}", app.group_by);
    }
    assert_eq!(app.group_by, start, "cycling has to come back round");
}

#[test]
fn the_board_deals_the_same_items_into_columns() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.board = true;
    let on_board: usize = app.columns.iter().map(|c| c.items.len()).sum();
    let in_list = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Item(_)))
        .count();
    assert_eq!(on_board, in_list, "the two views must show the same work");
    assert!(
        !app.columns.iter().any(|c| c.status == "dropped"),
        "a status with board = false has no column"
    );
}

#[test]
fn a_filter_narrows_both_views_the_same_way() {
    let dir = testkit::project();
    use crossterm::event::{KeyCode, KeyModifiers};
    let mut app = app_for(dir.path());
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    for c in "priority=p1".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    let listed: Vec<u32> = app
        .rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(app.items[*i].id),
            _ => None,
        })
        .collect();
    // Both are in v0.1, ordered by the status column they sit in: backlog
    // before doing, which is the project's own order rather than by id.
    assert_eq!(listed, vec![5, 3], "p1 and not finished");
    let on_board: usize = app.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(on_board, listed.len());
}

#[test]
fn a_broken_item_file_does_not_take_the_backlog_down_with_it() {
    let dir = testkit::project();
    std::fs::write(dir.path().join("items/0099-broken.md"), "no frontmatter\n")
        .expect("write a broken file");
    let app = app_for(dir.path());
    assert_eq!(app.items.len(), 6, "the good ones still load");
    assert!(
        app.warnings.iter().any(|w| w.contains("0099")),
        "and the bad one is reportable: {:?}",
        app.warnings
    );
}

fn ingested(view: Option<&str>, group_by: &str, dir: &std::path::Path) -> App {
    let mut project = Project::discover(dir).expect("found");
    let mut app = App::new();
    app.view = view.map(str::to_string);
    app.group_by = group_by.to_string();
    app.ingest(project.load().expect("loads"));
    app
}

#[test]
fn a_view_the_project_has_not_got_is_reported_and_stepped_over() {
    let dir = testkit::project();
    let app = ingested(Some("nonesuch"), "milestone", dir.path());
    assert!(
        app.view.is_none(),
        "a view that does not exist must not be in force"
    );
    let (message, _, _) = app.toast.as_ref().expect("and it has to say so");
    assert!(
        message.contains("now"),
        "naming the ones that do exist: {message}"
    );
}

#[test]
fn a_grouping_the_project_has_not_got_falls_back() {
    let dir = testkit::project();
    let app = ingested(None, "sprint", dir.path());
    assert_eq!(app.group_by, "milestone");
    let (message, _, _) = app.toast.as_ref().expect("and it has to say so");
    assert!(message.contains("sprint"), "{message}");
}

#[test]
fn a_read_only_backlog_says_so_rather_than_appearing_to_work() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.writable = false;
    app.select_id(3);
    assert_eq!(
        app.run(harrow::keys::Command::Claim),
        Action::None,
        "no change may leave when there is nothing to carry it out"
    );
    let (message, _, _) = app.toast.as_ref().expect("and it has to say why");
    assert!(message.contains("cairn"), "{message}");
}
