//! Work under way in another worktree, seen from this one.
//!
//! An agent claims an item in its own worktree, and the claim lives in that
//! worktree's copy until the branch merges. These hold that harrow sees it
//! anyway — beside the record, never in place of it — and that nothing a
//! branch did not do is ever reported as something it did.

use std::path::Path;

use harrow::engine::{Project, Report, Source};
use harrow::schema::Category;
use harrow::testkit;

fn load(dir: &Path) -> Report {
    Project::discover(dir)
        .expect("the project is found")
        .load()
        .expect("it loads")
}

fn claim(dir: &Path, file: &str, who: &str) {
    let path = dir.join("items").join(file);
    let text = std::fs::read_to_string(&path).expect("read the item");
    let text = text.replacen(
        "status: backlog",
        &format!("status: doing\nassignee: {who}"),
        1,
    );
    std::fs::write(&path, text).expect("write the claim");
}

#[test]
fn a_claim_in_another_worktree_is_seen_beside_the_record() {
    let (main, agent) = testkit::with_worktree("feat/0004-board");
    claim(agent.path(), "0004-draw-the-board.md", "an agent");

    let report = load(main.path());
    let board = report.items.iter().find(|i| i.id == 4).expect("item 4");
    assert_eq!(board.status, "backlog", "the record is this checkout's");
    assert_eq!(board.category, Category::Open);
    let there = board.active_elsewhere().expect("under way elsewhere");
    assert_eq!(there.branch, "feat/0004-board");
    assert_eq!(there.status, "doing");
    assert_eq!(there.assignee.as_deref(), Some("an agent"));
    assert_eq!(board.holder(), Some("an agent"));
    assert_eq!(report.elsewhere.len(), 1, "the other worktree is watched");
    assert!(report.registry.is_some(), "and so is the list of worktrees");
}

#[test]
fn a_committed_claim_is_seen_as_well_as_an_uncommitted_one() {
    let (main, agent) = testkit::with_worktree("feat/0004-board");
    claim(agent.path(), "0004-draw-the-board.md", "an agent");
    testkit::commit(agent.path(), "claim 0004");

    let report = load(main.path());
    let board = report.items.iter().find(|i| i.id == 4).expect("item 4");
    assert!(board.active_elsewhere().is_some());
}

#[test]
fn an_item_the_branch_never_touched_is_not_reported_however_old_its_copy() {
    // The branch was cut before this checkout moved on, so its copy of 5 is
    // older than ours and differs from it. That is not work on the branch.
    let (main, agent) = testkit::with_worktree("feat/0004-board");
    claim(
        main.path(),
        "0005-the-detail-pane-scrolls-past-its-pane.md",
        "oddur",
    );
    testkit::commit(main.path(), "take 0005 here");

    let report = load(main.path());
    assert!(
        report.items.iter().all(|i| i.elsewhere.is_empty()),
        "{:?}",
        report
            .items
            .iter()
            .filter(|i| !i.elsewhere.is_empty())
            .map(|i| i.id)
            .collect::<Vec<_>>()
    );
    drop(agent);
}

#[test]
fn an_item_filed_on_a_branch_is_not_mistaken_for_one_here() {
    // In a counted format, a branch's new 7 and this checkout's new 7 are
    // different items with the same id.
    let (main, agent) = testkit::with_worktree("feat/filing");
    let item = "---\nid: 7\ntitle: {t}\ntype: feature\nstatus: backlog\n---\n";
    std::fs::write(
        main.path().join("items/0007-here.md"),
        item.replace("{t}", "Filed here"),
    )
    .expect("write");
    testkit::commit(main.path(), "file 7 here");
    std::fs::write(
        agent.path().join("items/0007-there.md"),
        item.replace("{t}", "Filed there")
            .replace("backlog", "doing"),
    )
    .expect("write");
    // Committed, because an untracked file is invisible to the diff anyway.
    testkit::commit(agent.path(), "file 7 there");

    let report = load(main.path());
    let seven = report.items.iter().find(|i| i.id == 7).expect("item 7");
    assert!(seven.elsewhere.is_empty());
    assert_eq!(report.items.len(), 7, "and nothing is added to the set");
}

#[test]
fn a_rewrite_into_what_it_already_is_here_is_not_news() {
    let (main, agent) = testkit::with_worktree("feat/same");
    let file = "0004-draw-the-board.md";
    claim(main.path(), file, "oddur");
    testkit::commit(main.path(), "take 0004 here");
    claim(agent.path(), file, "oddur");

    let report = load(main.path());
    let board = report.items.iter().find(|i| i.id == 4).expect("item 4");
    assert!(board.elsewhere.is_empty());
}

#[test]
fn a_worktree_on_another_format_is_left_alone() {
    let (main, agent) = testkit::with_worktree("feat/old");
    let config = agent.path().join("cairn.toml");
    let text = std::fs::read_to_string(&config).expect("read");
    std::fs::write(&config, text.replacen("format = 3", "format = 2", 1)).expect("write");
    claim(agent.path(), "0004-draw-the-board.md", "an agent");

    let report = load(main.path());
    assert!(report.items.iter().all(|i| i.elsewhere.is_empty()));
    assert!(report.elsewhere.is_empty());
}

#[test]
fn a_project_outside_git_reads_exactly_as_it_did() {
    let dir = testkit::project();
    let report = load(dir.path());
    assert!(report.elsewhere.is_empty());
    assert!(report.registry.is_none());
    assert!(report.items.iter().all(|i| i.elsewhere.is_empty()));
}

#[test]
fn the_other_worktrees_see_this_one_too() {
    // Symmetric: opened in the agent's worktree, the primary checkout is the
    // one elsewhere.
    let (main, agent) = testkit::with_worktree("feat/0004-board");
    claim(main.path(), "0004-draw-the-board.md", "oddur");

    let report = load(agent.path());
    let board = report.items.iter().find(|i| i.id == 4).expect("item 4");
    assert_eq!(
        board.active_elsewhere().map(|e| e.branch.as_str()),
        Some("main")
    );
}

// ── What a reader sees ──────────────────────────────────────────────────────

use harrow::item::Elsewhere;

/// The sample backlog, with 4 claimed by an agent on its own branch.
fn claimed_elsewhere() -> Report {
    let mut report = testkit::report();
    let board = report.items.iter_mut().find(|i| i.id == 4).expect("item 4");
    board.elsewhere.push(Elsewhere {
        branch: "feat/0004-board".into(),
        status: "doing".into(),
        category: Category::Active,
        assignee: Some("an agent".into()),
        latest: Some(("2026-09-24".into(), "Columns drawn; cards next.".into())),
    });
    report
}

fn screen(app: &mut harrow::app::App, tick: usize) -> String {
    let buffer = harrow::ui::render_frame(app, 140, 30, tick);
    let area = buffer.area;
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn row_of<'a>(text: &'a str, title: &str) -> &'a str {
    text.lines()
        .find(|l| l.contains(title))
        .unwrap_or_else(|| panic!("no row for {title}:\n{text}"))
}

#[test]
fn work_under_way_on_another_branch_turns_like_work_under_way_here() {
    let mut app = harrow::app::App::new();
    app.ingest(claimed_elsewhere());
    let at_rest = screen(&mut app, 0);
    let turned = screen(&mut app, 2);
    assert!(row_of(&at_rest, "Draw the board").contains('◐'));
    assert!(row_of(&turned, "Draw the board").contains('◓'), "it moves");
    assert!(
        row_of(&turned, "Draw the board").contains("an agent"),
        "and says who has it"
    );
    // Beside the record: the item has not moved status here.
    let item = app.items.iter().find(|i| i.id == 4).expect("item 4");
    assert_eq!(item.status, "backlog");
}

#[test]
fn the_detail_names_the_branch_and_what_it_did() {
    let mut app = harrow::app::App::new();
    app.ingest(claimed_elsewhere());
    app.select_id(4.into());
    let text = screen(&mut app, 0);
    assert!(text.contains("Elsewhere"), "{text}");
    assert!(text.contains("feat/0004-board"), "{text}");
    assert!(text.contains("backlog → in progress"), "{text}");
    assert!(text.contains("Columns drawn; cards next."), "{text}");
}

#[test]
fn a_claim_elsewhere_is_announced_as_it_arrives() {
    let mut app = harrow::app::App::new();
    app.ingest(testkit::report());
    app.ingest(claimed_elsewhere());
    let said = app
        .toast
        .as_ref()
        .map(|(m, _, _)| m.clone())
        .unwrap_or_default();
    assert!(said.contains("doing · feat/0004-board"), "{said}");
    assert!(app.is_recent(4.into()), "and the row says it just moved");

    // Read again with nothing new: nothing new is said.
    app.toast = None;
    app.ingest(claimed_elsewhere());
    assert!(app.toast.is_none());
}

#[test]
fn the_doctor_says_what_the_other_worktrees_hold() {
    let (main, agent) = testkit::with_worktree("feat/0004-board");
    claim(agent.path(), "0004-draw-the-board.md", "an agent");
    let checks = harrow::doctor::run(
        &harrow::config::Config::default(),
        None,
        &harrow::theme::Theme::mono(),
        main.path(),
    );
    let line = checks
        .iter()
        .find(|c| c.name == "worktrees")
        .expect("a worktrees line");
    assert!(line.ok);
    assert_eq!(line.detail, "1 other — 1 item changed there, 1 under way");
}
