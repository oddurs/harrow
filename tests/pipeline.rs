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

/// `a` is a question about the list. The board has already answered it: a
/// status is a column because the project wrote `board = true`, and drawing a
/// column the list's rule then refuses to fill is how `done 0` came to sit
/// beside `✓ 1 done` in the strip.
#[test]
fn the_board_deals_what_the_project_gave_a_column_rather_than_what_the_list_shows() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.pane = harrow::app::Pane::Board;

    let in_list = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Item(_)))
        .count();
    let on_board: usize = app.columns.iter().map(|c| c.items.len()).sum();
    assert!(
        on_board > in_list,
        "the list hides finished work and the board has a column for it"
    );

    let done: Vec<u32> = app
        .columns
        .iter()
        .find(|c| c.status == "done")
        .expect("the fixture declares a done column")
        .items
        .iter()
        .map(|i| app.items[*i].id)
        .collect();
    assert_eq!(done, vec![2], "and that column holds the finished item");

    assert!(
        !app.columns.iter().any(|c| c.status == "dropped"),
        "a status with board = false has no column"
    );
}

/// Two readings of one backlog in the same frame. They were disagreeing.
#[test]
fn the_strip_and_the_board_agree_about_how_much_is_done() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.pane = harrow::app::Pane::Board;

    let in_strip: usize = app
        .status_counts()
        .into_iter()
        .filter(|(status, _)| status.name == "done")
        .map(|(_, n)| n)
        .sum();
    let in_column = app
        .columns
        .iter()
        .find(|c| c.status == "done")
        .map(|c| c.items.len())
        .unwrap_or(0);
    assert_eq!(in_strip, in_column);
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
    app.readonly = Some(harrow::app::ReadOnly::NoCairn);
    app.select_id(3);
    assert_eq!(
        app.run(harrow::keys::Command::Claim),
        Action::None,
        "no change may leave when there is nothing to carry it out"
    );
    let (message, _, _) = app.toast.as_ref().expect("and it has to say why");
    assert!(message.contains("cairn"), "{message}");
}

/// Answering a keystroke by making the thing you pressed it on vanish does not
/// say what happened; it only stops saying anything. So an item that has just
/// changed its way off the screen is held where it landed for a moment.
#[test]
fn an_item_closed_under_you_is_watched_out_rather_than_deleted() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    let ids = |app: &App| -> Vec<u32> {
        app.rows
            .iter()
            .filter_map(|r| match r {
                Row::Item(i) => Some(app.items[*i].id),
                _ => None,
            })
            .collect()
    };
    assert!(ids(&app).contains(&3), "0003 starts on screen");

    // What `cairn set 3 status=done` leaves behind, read back the way the
    // watcher would read it.
    let path = dir.path().join("items/0003-draw-the-list.md");
    let text = std::fs::read_to_string(&path).expect("the item is there");
    std::fs::write(&path, text.replace("status: doing", "status: done")).expect("write it back");
    let mut project = Project::discover(dir.path()).expect("the project is found");
    app.ingest(project.load().expect("it reloads"));

    let item = app.items.iter().find(|i| i.id == 3).expect("0003");
    assert!(item.category.is_closed(), "it is done now");
    assert!(
        ids(&app).contains(&3),
        "and still on screen, so the change can be seen happening"
    );
    assert!(
        app.columns
            .iter()
            .find(|c| c.status == "done")
            .is_some_and(|c| c.items.iter().any(|i| app.items[*i].id == 3)),
        "the board shows it arriving in done"
    );

    // And then it goes, without waiting for a keystroke to notice.
    app.now += App::SETTLING + 1;
    assert!(app.settle(), "the moment is up");
    assert!(!ids(&app).contains(&3), "so it leaves");
    assert!(app.check_invariants().is_ok());
}

/// Typing a filter is a deliberate act of exclusion. A list that answered it by
/// holding on to what you just excluded would be arguing with you.
#[test]
fn narrowing_the_filter_drops_rows_at_once() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    use crossterm::event::{KeyCode, KeyModifiers};
    let before = app.rows.len();
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    for c in "priority=p0".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert!(
        app.rows.len() < before,
        "it narrows now, not in six seconds"
    );
}

/// A project that files its items in subdirectories showed harrow an empty
/// backlog, with nothing saying there was anything to miss.
#[test]
fn an_item_in_a_subdirectory_is_still_an_item() {
    let dir = testkit::project();
    let deep = dir.path().join("items/archive/2026");
    std::fs::create_dir_all(&deep).expect("make a subdirectory");
    std::fs::write(
        deep.join("0099-filed-away.md"),
        "---\nid: 99\ntitle: Filed away\nstatus: done\n---\n\nBody.\n",
    )
    .expect("write it");

    let app = app_for(dir.path());
    assert_eq!(app.items.len(), 7, "six at the top level and one below");
    assert!(app.items.iter().any(|i| i.id == 99));
    assert!(app.warnings.is_empty(), "{:?}", app.warnings);
}

/// A project that keeps a README beside its items has not made a mistake, and
/// a warning per file per reload is how a diagnostics pane stops being read.
#[test]
fn what_the_format_says_is_not_an_item_is_neither_an_item_nor_a_warning() {
    let dir = testkit::project();
    for name in ["README.md", "_template.md", ".draft.md"] {
        std::fs::write(dir.path().join("items").join(name), "not an item\n").expect("write it");
    }
    let app = app_for(dir.path());
    assert_eq!(app.items.len(), 6, "still only the six");
    assert!(app.warnings.is_empty(), "and silently: {:?}", app.warnings);
}

/// The counter is unit-tested; this is the wiring — that a heading named in
/// `cairn.toml` reaches the count every row of the list asks for.
#[test]
fn a_project_that_names_its_criteria_section_is_obeyed() {
    let dir = testkit::project();
    let cfg = dir.path().join("cairn.toml");
    let text = std::fs::read_to_string(&cfg).expect("the fixture config");
    std::fs::write(
        &cfg,
        text.replace(
            "id_width = 4",
            "id_width = 4\ncriteria_section = \"Acceptance\"",
        ),
    )
    .expect("name a section");
    std::fs::write(
        dir.path().join("items/0098-sectioned.md"),
        "---\nid: 98\ntitle: Sectioned\nstatus: backlog\n---\n\n\
         ## Notes\n\n- [x] not a criterion\n\n## Acceptance\n\n- [x] one\n- [ ] two\n- [ ]\n",
    )
    .expect("write an item");

    let app = app_for(dir.path());
    assert_eq!(
        app.schema.criteria_section.as_deref(),
        Some("Acceptance"),
        "the project said where they live"
    );
    let item = app.items.iter().find(|i| i.id == 98).expect("0098");
    assert_eq!(
        item.criteria(),
        (1, 2),
        "the ticked box under Notes is not a criterion met, and the bare box is a placeholder"
    );
}

/// `milestone: 0042` must not mean either a key or a number depending on what
/// happens to exist. Two rules keep that unambiguous — a key may not look like
/// a rendered identifier, and a key-addressed reference resolves by key alone
/// — and the second is worth nothing without the first.
#[test]
fn a_reference_addressed_by_key_resolves_only_by_key() {
    let dir = testkit::project();
    // `v0.1` is item 1's key. A reference naming the number instead names
    // nothing, which is a reference that names nothing, not item 1.
    std::fs::write(
        dir.path().join("items/0097-by-the-number.md"),
        "---\nid: 97\ntitle: Filed against a number\nstatus: backlog\nmilestone: 1\n---\n",
    )
    .expect("write it");

    let app = app_for(dir.path());
    assert!(
        app.item_named("v0.1").is_some_and(|i| i.id == 1),
        "the key still resolves"
    );
    assert!(app.item_named("1").is_none(), "and the id does not");
    assert!(
        app.item_named("V0.1").is_some(),
        "case is not what distinguishes two keys"
    );

    let scheduled = app.items.iter().find(|i| i.id == 1).expect("the milestone");
    assert!(
        !scheduled.contains.contains(&97),
        "so nothing was filed under it by number"
    );
}

/// An id-addressed field is the other half of the same rule: `part_of: 3`
/// names item 3, and a key that happens to read as a number names nothing.
#[test]
fn an_id_addressed_reference_resolves_only_by_id() {
    let dir = testkit::project();
    std::fs::write(
        dir.path().join("items/0096-composed.md"),
        "---\nid: 96\ntitle: Composed\nstatus: backlog\npart_of: '#3'\n---\n",
    )
    .expect("write it");
    std::fs::write(
        dir.path().join("items/0095-by-key.md"),
        "---\nid: 95\ntitle: Named by a key\nstatus: backlog\npart_of: v0.1\n---\n",
    )
    .expect("write it");

    let app = app_for(dir.path());
    let three = app.items.iter().find(|i| i.id == 3).expect("0003");
    assert!(
        three.contains.contains(&96),
        "a leading # is how references are printed, so it is how they are pasted back"
    );
    let milestone = app.items.iter().find(|i| i.id == 1).expect("0001");
    assert!(
        !milestone.contains.contains(&95),
        "an id-addressed field does not fall back to a key"
    );
}

/// The rendering is the project's, and it reaches every screen and both ways
/// of asking for an item by name.
#[test]
fn a_project_that_renders_its_identifiers_its_own_way_is_obeyed() {
    let dir = testkit::project();
    let cfg = dir.path().join("cairn.toml");
    let text = std::fs::read_to_string(&cfg).expect("the fixture config");
    std::fs::write(
        &cfg,
        text.replace("id_width = 4", "id_format = \"MP-{n:03}\""),
    )
    .expect("name a rendering");

    let mut app = app_for(dir.path());
    let item = app.items.iter().find(|i| i.id == 3).expect("0003");
    assert_eq!(item.reference(&app.schema), "MP-003");

    let screen = harrow::ui::render_to_string(&mut app, 110, 26, 0);
    assert!(screen.contains("MP-003"), "it reaches the list");
    assert!(!screen.contains(" 0003 "), "and replaces the old rendering");

    use crossterm::event::{KeyCode, KeyModifiers};
    for spelling in ["MP-003", "3"] {
        app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
        for c in format!("id={spelling}").chars() {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        let ids: Vec<u32> = app
            .rows
            .iter()
            .filter_map(|r| match r {
                Row::Item(i) => Some(app.items[*i].id),
                _ => None,
            })
            .collect();
        assert_eq!(ids, vec![3], "asking by {spelling}");
        // Esc backs out of the filter, so the next spelling starts clean.
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    }
}

/// No format bump has ever changed what a key in an item means — each changed
/// only how the configuration says what it says — so a project from a newer
/// cairn is one harrow can read. What it must not do is write to it, because
/// cairn will refuse that itself until the project is migrated.
#[test]
fn a_project_from_a_newer_cairn_reads_but_does_not_write() {
    let dir = testkit::project();
    let cfg = dir.path().join("cairn.toml");
    let text = std::fs::read_to_string(&cfg).expect("the fixture config");
    let ahead = harrow::schema::KNOWN_FORMAT + 1;
    std::fs::write(
        &cfg,
        text.replace("format = 3", &format!("format = {ahead}")),
    )
    .expect("write a newer format");

    let mut app = app_for(dir.path());
    assert_eq!(app.items.len(), 6, "every item still reads");
    assert!(!app.rows.is_empty(), "and lists");

    assert_eq!(
        app.readonly,
        Some(harrow::app::ReadOnly::Format(ahead)),
        "and says why it will not write"
    );
    app.select_id(3);
    assert_eq!(
        app.run(harrow::keys::Command::Claim),
        Action::None,
        "so a write is refused here rather than by cairn confusingly"
    );
    let said = app
        .toast
        .as_ref()
        .map(|(m, _, _)| m.clone())
        .unwrap_or_default();
    assert!(said.contains("cairn migrate"), "naming the remedy: {said}");

    // And once the toast has gone, the footer is still saying it — which is
    // the difference between being told and being able to check.
    app.toast = None;
    let screen = harrow::ui::render_to_string(&mut app, 110, 26, 0);
    assert!(
        screen.contains(&format!("read-only · format {ahead}")),
        "the footer says which read-only"
    );
}

/// Stale means visible, never revoked. Nothing is released automatically —
/// taking work away from somebody slow is worse than leaving it held — so the
/// whole of this is how a row is drawn and what a filter can select.
#[test]
fn a_claim_the_project_calls_stale_says_so() {
    let dir = testkit::project();
    let cfg = dir.path().join("cairn.toml");
    let text = std::fs::read_to_string(&cfg).expect("the fixture config");
    std::fs::write(
        &cfg,
        text.replace("id_width = 4", "id_width = 4\nclaim_stale_after = 5"),
    )
    .expect("set a threshold");

    let mut app = app_for(dir.path());
    // 0003 is claimed on 2026-09-03. A week later that is past five days.
    app.now = 1_789_084_800; // 2026-09-11
    app.rebuild();
    let held = app.items.iter().find(|i| i.id == 3).expect("0003");
    assert!(held.claim_stale, "eight days is past five");

    // The day it was taken, it is not.
    app.now = 1_788_480_000; // 2026-09-04
    app.rebuild();
    let fresh = app.items.iter().find(|i| i.id == 3).expect("0003");
    assert!(!fresh.claim_stale, "the day after is not stale");
}

/// A project that has not set a threshold sees no change at all: how long is
/// too long is a property of the project, not of the tool.
#[test]
fn without_a_threshold_no_claim_is_ever_stale() {
    let dir = testkit::project();
    let mut app = app_for(dir.path());
    app.now = 4_000_000_000; // years later
    app.rebuild();
    assert!(app.schema.claim_stale_after.is_none());
    assert!(app.items.iter().all(|i| !i.claim_stale));
}
