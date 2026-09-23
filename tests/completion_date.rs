use harrow::app::App;
use harrow::engine::{Project, Source};
use harrow::{testkit, ui};

#[test]
fn a_later_edit_does_not_move_recorded_completion() {
    let dir = testkit::project();
    std::fs::write(dir.path().join("items/0007-completed.md"), "---\nid: 7\ntitle: Completed and edited later\ntype: feature\nstatus: done\nclosed_at: 2026-08-01\nupdated: 2026-09-10\n---\n").unwrap();
    let mut source = Project::discover(dir.path()).unwrap();
    let mut app = App::new();
    app.now = 1_789_084_800; // 2026-09-11
    app.show_all = true;
    app.group_by = "none".into();
    app.filter = "id=7".into();
    app.ingest(source.load().unwrap());
    app.select_id(7);
    assert_eq!(
        app.selected_item().unwrap().closed_at.as_deref(),
        Some("2026-08-01")
    );
    assert_eq!(app.stats().closed_recently, [(7, 0), (30, 0), (90, 1)]);
    let screen = ui::render_to_string(&mut app, 160, 60, 0);
    assert!(screen.contains("closed_at"), "{screen}");
    assert!(screen.contains("2026-08-01"), "{screen}");
    assert!(
        screen.contains("2026-09-10"),
        "the independent edit date remains visible"
    );
}
