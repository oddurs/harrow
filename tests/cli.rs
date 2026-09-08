//! The binary, from the outside.

use std::path::PathBuf;
use std::process::Command;

use harrow::testkit;

fn harrow() -> PathBuf {
    // The binary next to this test's own executable, whichever profile it was
    // built in.
    let mut path = std::env::current_exe().expect("test binary path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("harrow")
}

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(harrow())
        .args(args)
        .output()
        .expect("harrow runs");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn version_and_help_say_what_this_is() {
    let (out, _, code) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("harrow "), "{out}");

    let (out, _, code) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("cairn backlog"), "{out}");
    assert!(out.contains("--screenshot"), "{out}");
}

#[test]
fn an_unknown_option_is_refused_rather_than_ignored() {
    let (_, err, code) = run(&["--not-an-option"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown option"), "{err}");
}

#[test]
fn a_directory_with_no_project_says_which_directory() {
    let (_, err, code) = run(&["-C", "/", "--plain"]);
    assert_eq!(code, 1);
    assert!(err.contains("cairn.toml"), "{err}");
    assert!(err.contains("cairn init"), "and what to do about it: {err}");
}

#[test]
fn plain_prints_one_line_per_item() {
    let dir = testkit::project();
    let (out, _, code) = run(&["-C", &dir.path().display().to_string(), "--plain"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "the finished one and the milestone are not rows"
    );
    assert!(lines[0].starts_with("0005\t"), "{out}");
    assert!(lines.iter().all(|l| l.matches('\t').count() == 4), "{out}");
}

#[test]
fn plain_takes_the_same_filters_the_interface_does() {
    let dir = testkit::project();
    let path = dir.path().display().to_string();
    let (out, _, _) = run(&["-C", &path, "--plain", "-f", "priority=p1"]);
    assert_eq!(out.lines().count(), 2, "{out}");

    let (out, _, _) = run(&["-C", &path, "--plain", "--all"]);
    assert_eq!(out.lines().count(), 5, "--all includes what is finished");
}

#[test]
fn a_screenshot_needs_no_terminal() {
    let dir = testkit::project();
    let (out, _, code) = run(&[
        "-C",
        &dir.path().display().to_string(),
        "--screenshot",
        "100x20",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out.lines().count(), 20, "one line per row of the screen");
    assert!(out.contains("harrow"), "{out}");
    assert!(out.contains("First usable version"), "{out}");
}

#[test]
fn the_doctor_reports_on_a_real_project() {
    let dir = testkit::project();
    let (out, _, _) = run(&["-C", &dir.path().display().to_string(), "--doctor"]);
    assert!(out.contains("project"), "{out}");
    assert!(out.contains("sample"), "{out}");
    assert!(out.contains("schema"), "{out}");
}

#[test]
fn the_written_config_is_the_config_harrow_reads() {
    let dir = testkit::project();
    let path = dir.path().join("config.toml");
    let (out, _, code) = run(&["config", "--write", "--config", &path.display().to_string()]);
    assert_eq!(code, 0, "{out}");
    assert!(path.exists());

    let (out, _, code) = run(&["config", "--config", &path.display().to_string()]);
    assert_eq!(code, 0);
    assert!(out.contains("group_by"), "{out}");

    // A second write refuses rather than clobbering.
    let (_, err, code) = run(&["config", "--write", "--config", &path.display().to_string()]);
    assert_eq!(code, 1);
    assert!(err.contains("already exists"), "{err}");
}

#[test]
fn themes_lists_what_can_actually_be_resolved() {
    let (out, _, code) = run(&["themes"]);
    assert_eq!(code, 0);
    for builtin in ["auto", "mono", "gotham", "night", "paper"] {
        assert!(out.contains(builtin), "{builtin} is missing from:\n{out}");
    }
}

#[test]
fn no_color_produces_a_screen_with_no_colour_in_it() {
    let dir = testkit::project();
    let out = Command::new(harrow())
        .args([
            "-C",
            &dir.path().display().to_string(),
            "--screenshot",
            "80x20",
            "--no-color",
        ])
        .output()
        .expect("runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains('\u{1b}'), "an escape sequence got through");
}
