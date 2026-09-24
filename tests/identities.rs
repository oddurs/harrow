//! Immutable identity is shared by every lens and every write surface.
use harrow::app::{Action, App, Pane};
use harrow::engine::{Project, Report, Source};
use harrow::identity::Id;
use harrow::keys::Command;
use harrow::schema::Schema;
use std::path::Path;

const A: &str = "a47c3bd2-0000-4000-8000-000000000001";
const B: &str = "a47c3bd2-1000-4000-8000-000000000002";
const C: &str = "972ab460-0000-4000-8000-000000000003";

fn schema() -> Schema {
    Schema::parse(
        &harrow::testkit::CAIRN_TOML.replace("format = 3", "format = 4"),
        "/tmp/uuid".into(),
    )
    .unwrap()
}

fn report(with_collision: bool) -> Report {
    let schema = schema();
    let mut items = vec![
        harrow::item::parse_for_schema(&format!("---\nid: {A}\ntitle: First\ntype: feature\nstatus: backlog\n---\nFirst body.\n"), Path::new("first.md"), &schema).unwrap(),
        harrow::item::parse_for_schema(&format!("---\nid: {C}\ntitle: Child\ntype: feature\nstatus: backlog\ndepends_on: [{A}]\npart_of: [{A}]\n---\nChild body.\n"), Path::new("child.md"), &schema).unwrap(),
    ];
    if with_collision {
        items.push(
            harrow::item::parse_for_schema(
                &format!(
                    "---\nid: {B}\ntitle: Similar prefix\ntype: feature\nstatus: backlog\n---\n"
                ),
                Path::new("second.md"),
                &schema,
            )
            .unwrap(),
        );
    }
    let mut warnings = Vec::new();
    harrow::engine::derive(&mut items, &schema, &mut warnings);
    Report {
        schema,
        items,
        warnings,
        stamp: None,
    }
}

#[test]
fn full_ids_drive_dependencies_hierarchy_and_short_prefix_filters() {
    let report = report(false);
    let parent = report.items.iter().find(|i| i.id.to_string() == A).unwrap();
    let child = report.items.iter().find(|i| i.id.to_string() == C).unwrap();
    assert_eq!(child.blockers, vec![A.parse::<Id>().unwrap()]);
    assert_eq!(parent.contains, vec![child.id]);
    assert_eq!(child.depth, 1);
    for (filter, expected) in [
        ("id=a47c3bd2", A),
        ("depends_on=a47c3bd2", C),
        ("contains=972ab460", A),
        ("part_of=a47c3bd2", C),
    ] {
        let query = harrow::filter::Query::parse(filter, &report.schema);
        assert!(query.errors.is_empty(), "{filter}: {:?}", query.errors);
        let ids: Vec<_> = report
            .items
            .iter()
            .filter(|i| query.matches(i, &report.schema))
            .map(|i| i.id.to_string())
            .collect();
        assert_eq!(ids, vec![expected], "{filter}");
    }
}

#[test]
fn display_expands_and_ambiguous_input_is_not_an_empty_success() {
    let report = report(true);
    let a = A.parse::<Id>().unwrap();
    assert_eq!(report.schema.format_id(a), "a47c3bd20");
    assert!(
        report
            .schema
            .parse_id("a47c3bd2")
            .unwrap_err()
            .contains("ambiguous")
    );
    assert_eq!(report.schema.parse_id("a47c3bd20").unwrap(), a);
    assert_eq!(report.schema.parse_id(&A.to_uppercase()).unwrap(), a);
    let query = harrow::filter::Query::parse("id=a47c3bd2", &report.schema);
    assert_eq!(query.errors.len(), 1);
    let mut app = App::new();
    app.filter = "id=a47c3bd2".into();
    app.ingest(report);
    assert!(app.filter_problem().unwrap().contains("ambiguous"));
}

#[test]
fn selection_marks_and_scroll_survive_a_prefix_change_on_reload() {
    let mut app = App::new();
    app.group_by = "none".into();
    app.show_all = true;
    app.ingest(report(false));
    let id = A.parse::<Id>().unwrap();
    app.select_id(id);
    app.marked.insert(id);
    app.detail.to(id, 2);
    assert_eq!(app.schema.format_id(id), "a47c3bd2");
    app.ingest(report(true));
    assert_eq!(app.schema.format_id(id), "a47c3bd20");
    assert_eq!(app.selected_item().unwrap().id, id);
    assert!(app.marked.contains(&id));
    assert_eq!(app.detail.at(id), 2);
    app.show_activity(Ok(format!(
        "abc123\u{1f}Author\u{1f}2026-09-23T09:00:00Z\u{1f}Create item\nitems/{A}-first.md\n"
    )));
    for pane in Pane::ALL {
        app.pane = pane;
        app.select_id(id);
        assert!(app.marked.contains(&id));
        assert_eq!(app.selected_item().unwrap().id, id);
    }
}

#[test]
fn writes_undo_history_and_clipboard_use_the_full_identity() {
    let mut app = App::new();
    app.group_by = "none".into();
    app.ingest(report(true));
    let id = A.parse::<Id>().unwrap();
    app.select_id(id);
    let Action::Write(change) = app.run(Command::Claim) else {
        panic!("claim must write")
    };
    assert_eq!(change.args, vec!["claim", A]);
    assert!(change.undo.unwrap().contains(A));
    assert!(matches!(app.run(Command::History), Action::History(got) if got == id));
    assert!(matches!(app.run(Command::Copy), Action::Copy(got) if got == A));
}

#[test]
fn format_four_never_infers_or_truncates_identity() {
    let schema = schema();
    for front in [
        "title: No ID".to_owned(),
        "id: 12".into(),
        "id: a47c3bd2".into(),
        format!("id: {A}\ndepends_on: [a47c3bd2]"),
        format!("id: {A}\npart_of: [12]"),
        "id: a47c3bd2-0000-1000-8000-000000000001".into(),
    ] {
        let text = format!("---\n{front}\n---\n");
        assert!(
            harrow::item::parse_for_schema(&text, Path::new("0012-old.md"), &schema).is_err(),
            "{text}"
        );
    }
    let upper = A.replace('-', "").to_uppercase();
    let item = harrow::item::parse_for_schema(
        &format!("---\nid: {upper}\npart_of: [{upper}]\n---\n"),
        Path::new("name.md"),
        &schema,
    )
    .unwrap();
    assert_eq!(item.id.to_string(), A);
    assert_eq!(item.fields["part_of"].items(), vec![A]);
}

#[test]
fn a_pending_migration_never_becomes_a_partial_backlog() {
    let dir = harrow::testkit::project();
    std::fs::write(dir.path().join("items/.identity-migration.json"), "{}").unwrap();
    let mut project = Project::discover(dir.path()).unwrap();
    let error = match project.load() {
        Ok(_) => panic!("pending migration loaded"),
        Err(e) => e,
    };
    assert!(error.transient);
    assert!(error.detail.contains("cairn migrate"));
}

#[test]
fn legacy_aliases_and_activity_paths_bridge_migration() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("items")).unwrap();
    std::fs::write(
        dir.path().join("cairn.toml"),
        "format = 4\n[project]\nname = \"Migrated\"\ndir = \"items\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("items/_legacy-ids.toml"),
        format!("version = 1\nid_format = \"MP-{{n}}\"\n[ids]\n\"42\" = \"{A}\"\n"),
    )
    .unwrap();
    let schema = Schema::load(&dir.path().join("cairn.toml")).unwrap();
    schema.remember_ids([A.parse::<Id>().unwrap()]);
    for alias in ["42", "#42", "MP-42", "mp-42"] {
        assert_eq!(schema.parse_id(alias).unwrap().to_string(), A);
    }
    for name in [
        "0042-original.md".to_owned(),
        "MP-42-original.md".into(),
        format!("{A}-new.md"),
    ] {
        assert_eq!(
            schema.id_from_path(Path::new(&name)).unwrap().to_string(),
            A
        );
    }
    assert!(schema.parse_id("43").is_err());
}
