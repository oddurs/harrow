//! The views a project declared, reachable from inside.
//!
//! A `cairn.toml` names the queries somebody decided were worth naming.
//! harrow has always read them and `--doctor` has always validated them, and
//! until now the only way to reach one was to quit and start again with a
//! flag — losing the marks, the filter and the item being read.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Picker};
use harrow::schema::{Schema, View};
use harrow::{testkit, ui};

fn press(app: &mut App, c: char) {
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
}

/// The sample project declares one view and no sort. These need a project
/// that declares both, so the view carries everything a view can carry.
fn app_with_views() -> App {
    let mut app = testkit::app();
    app.schema.views = vec![
        View {
            name: "now".into(),
            filter: Some("category=active".into()),
            sort: None,
            group_by: None,
            description: Some("What is actually being worked on".into()),
        },
        View {
            name: "triage".into(),
            filter: Some("priority=p3".into()),
            sort: Some("-id".into()),
            group_by: Some("status".into()),
            description: Some("Items that still need a priority".into()),
        },
    ];
    app.rebuild();
    app
}

fn picker(app: &App) -> &Picker {
    app.picker.as_ref().expect("the picker is open")
}

#[test]
fn a_key_lists_them_with_what_the_project_said_they_are_for() {
    let mut app = app_with_views();
    press(&mut app, 'V');
    let shown: Vec<(&str, &str)> = picker(&app)
        .options
        .iter()
        .map(|(_, name, note)| (name.as_str(), note.as_str()))
        .collect();
    assert_eq!(
        shown,
        vec![
            ("now", "What is actually being worked on"),
            ("triage", "Items that still need a priority"),
        ]
    );
}

/// A view is the project saying *this is how to look at this*, and its filter
/// is only one third of that.
#[test]
fn choosing_one_applies_its_filter_its_sort_and_its_grouping() {
    let mut app = app_with_views();
    press(&mut app, 'V');
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(app.view.as_deref(), Some("triage"));
    assert_eq!(app.sort, "-id");
    assert_eq!(app.group_by, "status");
}

/// It replaces rather than narrows: picking one on top of a half-typed filter
/// would be looking at something nobody described.
#[test]
fn it_replaces_what_was_typed() {
    let mut app = app_with_views();
    app.filter = "priority=p0".into();
    app.ingest(testkit::report());
    app.schema.views = app_with_views().schema.views;

    press(&mut app, 'V');
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.filter.is_empty(), "the typed filter survived");
    assert_eq!(app.view.as_deref(), Some("now"));
}

/// The name is what you chose; the grammar is what you got. Both, because a
/// name alone cannot be checked and a grammar alone loses what you asked for.
#[test]
fn the_view_in_force_is_named_on_the_line_beside_its_grammar() {
    let mut app = app_with_views();
    press(&mut app, 'V');
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    let line = ui::render_to_string(&mut app, 110, 26, 0)
        .lines()
        .nth(2)
        .unwrap_or_default()
        .to_string();
    assert!(line.contains("now"), "{line}");
    assert!(line.contains("category=active"), "{line}");
}

/// It says what it did, because a filter that changed without a word is the
/// thing the view line exists to prevent.
#[test]
fn it_says_which_view_it_took() {
    let mut app = app_with_views();
    press(&mut app, 'V');
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    let said = app.toast.as_ref().map(|(m, _, _)| m.clone()).unwrap();
    assert!(said.contains("now"), "{said}");
    assert!(said.contains("actually being worked on"), "{said}");
}

/// Nothing to offer is a thing to say, not a picker with no rows in it.
#[test]
fn a_project_with_no_views_says_so() {
    let mut app = testkit::app();
    app.schema.views.clear();
    press(&mut app, 'V');
    assert!(app.picker.is_none());
    let said = app.toast.as_ref().map(|(m, _, _)| m.clone()).unwrap();
    assert!(said.contains("no views"), "{said}");
}

/// A view whose filter does not parse must say so rather than match nothing —
/// the rule `--doctor` already applies to them from outside.
#[test]
fn a_view_that_does_not_parse_says_so_rather_than_matching_nothing() {
    let mut app = app_with_views();
    app.schema.views[0].filter = Some("severity=high".into());
    press(&mut app, 'V');
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(
        app.filter_problem().is_some_and(|p| p.contains("severity")),
        "{:?}",
        app.filter_problem()
    );
}

/// `v` and `V` pair honestly: both answer *how am I looking at this*.
#[test]
fn the_keys_that_answer_the_same_question_sit_together() {
    let map = harrow::keys::Keymap::default();
    assert_eq!(
        map.keys_for(harrow::keys::Command::GroupBy),
        vec!["v".to_string()]
    );
    assert_eq!(
        map.keys_for(harrow::keys::Command::Views),
        vec!["V".to_string()]
    );
    let _ = Schema::parse(testkit::CAIRN_TOML, std::path::PathBuf::from("/tmp/p"));
}
