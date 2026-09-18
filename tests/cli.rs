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

    // All of it: the finished item and the milestone. `--all` said
    // "finished, dropped, milestones" and returned no milestone, because the
    // default grouping draws them as headings and left them out of the rows
    // whether or not anybody had asked for them.
    let (out, _, _) = run(&["-C", &path, "--plain", "--all"]);
    assert_eq!(out.lines().count(), 6, "--all means all: {out}");
    assert!(
        out.contains("\tmilestone\t"),
        "no milestone in --all: {out}"
    );

    // And asking for the type by name, which is cairn's other way in.
    let (out, _, _) = run(&["-C", &path, "--plain", "-f", "type=milestone"]);
    assert_eq!(out.lines().count(), 1, "{out}");
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

/// What `brew install` runs to produce the completions and the man page. If the
/// binary cannot print them, the formula installs nothing and says nothing.
#[test]
fn the_binary_prints_its_own_completions_and_man_page() {
    for shell in ["fish", "bash", "zsh"] {
        let (out, _, code) = run(&["completions", shell]);
        assert_eq!(code, 0, "completions {shell}");
        assert!(out.contains("harrow"), "{shell}: {out}");
        assert!(out.len() > 400, "{shell} completions look empty:\n{out}");
    }

    let (_, err, code) = run(&["completions", "tcsh"]);
    assert_eq!(code, 2, "a shell we do not speak is an error");
    assert!(err.contains("fish"), "and says which we do: {err}");

    let (out, _, code) = run(&["man"]);
    assert_eq!(code, 0);
    assert!(
        out.starts_with(".\\\""),
        "a man page starts with a comment: {out:.40}"
    );
    assert!(out.contains(".TH HARROW 1"), "{out:.200}");
}

/// The formula the release attaches. Generated rather than transcribed, because
/// four checksums copied by hand is four chances to ship one that does not
/// match the file it names.
#[test]
fn the_homebrew_formula_is_generated_from_the_checksums() {
    let dir = testkit::project();
    let sums = dir.path().join("SHA256SUMS");
    std::fs::write(
        &sums,
        "1111111111111111111111111111111111111111111111111111111111111111  harrow-9.9.9-aarch64-apple-darwin.tar.gz\n\
         2222222222222222222222222222222222222222222222222222222222222222  harrow-9.9.9-x86_64-apple-darwin.tar.gz\n\
         3333333333333333333333333333333333333333333333333333333333333333  harrow-9.9.9-aarch64-unknown-linux-musl.tar.gz\n\
         4444444444444444444444444444444444444444444444444444444444444444  harrow-9.9.9-x86_64-unknown-linux-musl.tar.gz\n",
    )
    .expect("write the checksums");

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/formula");
    let out = Command::new(&script)
        .args(["9.9.9", &sums.display().to_string()])
        .output()
        .expect("the script runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let formula = String::from_utf8_lossy(&out.stdout);

    assert!(formula.contains("class Harrow < Formula"), "{formula}");
    assert!(formula.contains("version \"9.9.9\""), "{formula}");
    for sha in ['1', '2', '3', '4'] {
        assert!(
            formula.contains(&sha.to_string().repeat(64)),
            "every target's checksum has to be in it: {formula}"
        );
    }

    // A missing target is an error rather than a formula with a blank checksum.
    std::fs::write(&sums, "1111  harrow-9.9.9-aarch64-apple-darwin.tar.gz\n")
        .expect("write a short listing");
    let out = Command::new(&script)
        .args(["9.9.9", &sums.display().to_string()])
        .output()
        .expect("runs");
    assert!(!out.status.success(), "a missing checksum must fail loudly");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no checksum"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A screenshot has no event loop, so whatever the interface still needs has to
/// be asked for before the frame is drawn. Otherwise the log lens renders the
/// question it asks before the repository has answered.
#[test]
fn a_screenshot_of_the_log_shows_the_history_rather_than_the_asking() {
    let dir = testkit::project();
    let config = dir.path().join("harrow.toml");
    std::fs::write(&config, "pane = \"log\"\n").expect("write a config");

    let (out, _, code) = run(&[
        "-C",
        &dir.path().display().to_string(),
        "--config",
        &config.display().to_string(),
        "--screenshot",
        "100x20",
    ]);
    assert_eq!(code, 0);
    assert!(
        out.contains("What happened"),
        "the log lens is drawn:\n{out}"
    );
    assert!(
        !out.contains("Asking the repository"),
        "and it has been answered:\n{out}"
    );
    // Either answer is a real one: "no history to read" where git declines the
    // directory, "nothing has changed here yet" where it resolves a repository
    // and finds no commits touching it. Which of the two depends on the
    // environment — a git hook exports GIT_DIR, and a child git reads it — so
    // asserting one of them is asserting where the suite was run from.
    assert!(
        out.contains("no history") || out.contains("Nothing has changed"),
        "the lens has to say something it means:\n{out}"
    );
}
