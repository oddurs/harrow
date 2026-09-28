//! What `cairn check --prompts` says will be misread, shown where `cairn check`
//! is shown, as advice. Every finding is cairn's; harrow evaluates none.

use harrow::app::App;
use harrow::{testkit, ui};

/// What cairn 9c29249 prints for `check --prompts`, stdout then stderr.
const SUMMARY: &str = "ok: 6 item(s), 3 warning(s)\n";
const ADVICE: &str = "cairn: items/0003-draw-the-list.md: prompt: no context: the body says nothing beyond headings and criteria\n\
cairn: items/0005-the-detail-pane-scrolls-past-its-pane.md: prompt: no acceptance criteria: nothing says when it is done\n\
cairn: items/0005-the-detail-pane-scrolls-past-its-pane.md: prompt: 0002 finished without a result, so this prompt is handed nothing instead — `cairn close 0002 --result \"…\"` records one\n";

fn diagnostics(app: &mut App) -> String {
    app.diagnostics = true;
    ui::render_to_string(app, 110, 40, 0)
}

#[test]
fn prompt_findings_are_shown_under_prompts_as_cairn_said_them() {
    let mut app = testkit::app();
    app.show_check(Ok((SUMMARY.into(), ADVICE.into())));
    assert_eq!(app.prompt_findings.len(), 3);
    assert_eq!(app.prompt_findings[0].0, Some(3.into()));
    assert_eq!(
        app.prompt_findings[0].1,
        "no context: the body says nothing beyond headings and criteria"
    );
    let screen = diagnostics(&mut app);
    let prompts = screen.find("Prompts").expect("a Prompts heading");
    assert!(
        screen.find("cairn check").unwrap() < prompts,
        "after cairn's own"
    );
    assert!(screen.contains("0003 no context"), "{screen}");
    assert!(screen.contains("0005 no acceptance criteria"), "{screen}");
}

/// Without `--prompts` there is no advice, and the overlay is what it was.
#[test]
fn without_prompt_findings_the_overlay_is_unchanged_cell_for_cell() {
    let mut before = testkit::app();
    before.show_check(Ok((SUMMARY.into(), String::new())));
    let mut after = testkit::app();
    after.show_check(Ok((
        SUMMARY.into(),
        "cairn: warning: something else\n".into(),
    )));
    assert_eq!(diagnostics(&mut before), diagnostics(&mut after));
    assert!(!diagnostics(&mut after).contains("Prompts"));
}

/// cairn prints warnings, prompt findings among them, before the error that
/// failed the check. The failure is the error; the findings are still advice.
#[test]
fn a_failed_check_names_its_error_and_keeps_its_advice() {
    let mut app = testkit::app();
    app.show_check(Err(harrow::exec::ExecError::Failed {
        code: Some(1),
        stderr: format!("{ADVICE}cairn: items/0004-x.md: id 4 is used twice\n"),
    }));
    assert_eq!(
        app.checked,
        Some(Err(
            "exited 1: cairn: items/0004-x.md: id 4 is used twice".into()
        ))
    );
    assert_eq!(app.prompt_findings.len(), 3, "still shown, as advice");
}

#[test]
fn a_check_that_could_not_run_carries_no_advice_from_an_earlier_one() {
    let mut app = testkit::app();
    app.show_check(Ok((SUMMARY.into(), ADVICE.into())));
    app.show_check(Err(harrow::exec::ExecError::Spawn(std::io::Error::other(
        "no such file or directory",
    ))));
    assert!(app.prompt_findings.is_empty());
}

#[test]
fn a_long_list_says_how_many_more_there_are() {
    let mut app = testkit::app();
    let many: String = (1..=12)
        .map(|n| format!("cairn: items/000{n}-x.md: prompt: no context: nothing\n"))
        .collect();
    app.show_check(Ok((SUMMARY.into(), many)));
    assert!(diagnostics(&mut app).contains("…and 4 more"));
}
