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

/// Without `--prompts` there is no advice: cairn's other findings are its
/// check's own, and go under cairn check, not under Prompts.
#[test]
fn without_prompt_findings_there_is_no_prompts_section() {
    let mut app = testkit::app();
    app.show_check(Ok((
        SUMMARY.into(),
        "cairn: warning: something else\n".into(),
    )));
    assert!(app.prompt_findings.is_empty());
    let screen = diagnostics(&mut app);
    assert!(!screen.contains("Prompts"), "{screen}");
    assert!(
        screen.contains("cairn: warning: something else"),
        "{screen}"
    );
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
        Some(Err(vec![
            "cairn: items/0004-x.md: id 4 is used twice".into()
        ]))
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

/// What cairn check prints for a project with warnings, stdout then stderr.
const WARNED: (&str, &str) = (
    "ok: 7 item(s), 2 warning(s)\n",
    "cairn: cairn/items/0098-warned.md:5: field `mystery` is not declared in cairn.toml\n\
     cairn: cairn/items/0098-warned.md: filename does not match the item (expected 0002-warned.md)\n",
);

/// The overlay used to say "2 warning(s)" and show neither.
#[test]
fn a_checks_warnings_are_shown_word_for_word() {
    let mut app = testkit::app();
    app.show_check(Ok((WARNED.0.into(), WARNED.1.into())));
    assert_eq!(
        app.checked,
        Some(Ok(vec![
            "ok: 7 item(s), 2 warning(s)".into(),
            "cairn: cairn/items/0098-warned.md:5: field `mystery` is not declared in cairn.toml".into(),
            "cairn: cairn/items/0098-warned.md: filename does not match the item (expected 0002-warned.md)".into(),
        ]))
    );
    let screen = diagnostics(&mut app);
    assert!(
        screen.contains("field `mystery` is not declared"),
        "{screen}"
    );
}

/// A failed check says everything cairn found, its summary first.
#[test]
fn a_failed_check_shows_every_finding_and_its_summary_first() {
    let mut app = testkit::app();
    app.show_check(Err(harrow::exec::ExecError::Failed {
        code: Some(1),
        stderr: "cairn: cairn/items/0099-bad.md:7: unknown status `nonsense`\n\
                 cairn: cairn/items/0099-bad.md:4: key `v0.1` is already used by 0001\n\
                 \n\
                 failed: 2 error(s), 0 warning(s) across 6 item(s)\n"
            .into(),
    }));
    let Some(Err(said)) = &app.checked else {
        panic!("a failure: {:?}", app.checked)
    };
    assert_eq!(said[0], "failed: 2 error(s), 0 warning(s) across 6 item(s)");
    assert!(
        said[1].contains("already used by 0001"),
        "the last printed, first: {said:?}"
    );
    assert_eq!(said.len(), 3, "{said:?}");
    let screen = diagnostics(&mut app);
    assert!(screen.contains("unknown status `nonsense`"), "{screen}");
    assert!(screen.contains("already used by 0001"), "{screen}");
}

#[test]
fn a_long_check_says_how_many_more_there_are() {
    let mut app = testkit::app();
    let many: String = (1..=12)
        .map(|n| format!("cairn: items/000{n}-x.md: something\n"))
        .collect();
    app.show_check(Ok(("ok: 12 item(s), 12 warning(s)\n".into(), many)));
    let screen = diagnostics(&mut app);
    assert!(
        screen.contains("…and 5 more — cairn check has them all"),
        "{screen}"
    );
    assert!(
        screen.contains("ok: 12 item(s)"),
        "the summary survives the cut"
    );
}

/// Eight warnings and one error: the error, printed last, is what failed the
/// check, and it is on screen however many warnings came before it.
#[test]
fn a_failed_check_keeps_its_errors_on_screen_under_many_warnings() {
    let mut app = testkit::app();
    let warnings: String = (1..=8)
        .map(|n| format!("cairn: items/000{n}-x.md:5: field `f{n}` is not declared\n"))
        .collect();
    app.show_check(Err(harrow::exec::ExecError::Failed {
        code: Some(1),
        stderr: format!(
            "{warnings}cairn: items/0009-x.md:7: unknown status `nonsense`\n\nfailed: 1 error(s), 8 warning(s) across 9 item(s)\n"
        ),
    }));
    let screen = diagnostics(&mut app);
    assert!(screen.contains("unknown status `nonsense`"), "{screen}");
    assert!(screen.contains("failed: 1 error(s)"), "{screen}");
}
