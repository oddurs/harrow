//! Splitting an item into its numbered steps: cairn's dry run names every
//! child, the confirm shows them, and `y` is the only thing that writes.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App};
use harrow::keys::{Command, Keymap};
use harrow::{testkit, ui};

/// What `cairn split 3 --dry-run` prints, word for word.
const DRY_RUN: &str = "  0010 Read the file\n  0011 Parse the frontmatter  after 0010\n  0012 Check the schema  after 0011\n\ndry run: 3 item(s) would be created from 0003's `Approach`\n";

fn app() -> App {
    let mut app = testkit::app();
    app.select_id(3.into());
    app
}

/// It has no key: it creates items in bulk, and is asked for by name.
#[test]
fn split_is_asked_for_by_name_and_writes_nothing_yet() {
    let mut app = app();
    assert!(app.keymap.keys_for(Command::Split).is_empty(), "no key");
    app.handle_key(KeyCode::Char(':'), KeyModifiers::NONE);
    for c in "split".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert_eq!(
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE),
        Action::Split(3.into()),
        "the shell asks cairn for a dry run first"
    );
}

#[test]
fn the_confirm_lists_every_child_before_anything_is_written() {
    let mut app = app();
    app.show_split(3.into(), Ok(DRY_RUN.to_string()));
    let confirm = app.confirm.as_ref().expect("it asks");
    assert_eq!(confirm.prompt, "Split 0003 into 3 items?");
    let screen = ui::render_to_string(&mut app, 110, 30, 0);
    for child in [
        "0010 Read the file",
        "0011 Parse the frontmatter  after 0010",
        "0012 Check the schema  after 0011",
    ] {
        assert!(screen.contains(child), "{child:?} in\n{screen}");
    }
    assert!(
        !screen.contains("dry run:"),
        "cairn's summary is the prompt's job"
    );
}

#[test]
fn yes_writes_through_cairn_split() {
    let mut app = app();
    app.show_split(3.into(), Ok(DRY_RUN.to_string()));
    match app.handle_key(KeyCode::Char('y'), KeyModifiers::NONE) {
        Action::Write(change) => {
            assert_eq!(change.args, vec!["split", "3"]);
            assert_eq!(change.describe, "0003 split");
        }
        other => panic!("expected the split, got {other:?}"),
    }
}

#[test]
fn no_writes_nothing() {
    let mut app = app();
    app.show_split(3.into(), Ok(DRY_RUN.to_string()));
    assert_eq!(
        app.handle_key(KeyCode::Char('n'), KeyModifiers::NONE),
        Action::None
    );
    assert!(app.confirm.is_none());
}

/// Cairn's refusal, as cairn put it.
#[test]
fn cairns_refusal_reaches_the_footer_unchanged() {
    let mut app = app();
    let refusal = "cairn: 0003 has no `Approach` section to split";
    app.show_split(3.into(), Err(refusal.to_string()));
    assert!(app.confirm.is_none());
    assert_eq!(app.toast.clone().expect("a toast").0, refusal);
}

/// `exec` keeps a refusal in the program's own words rather than wrapping it
/// in an exit code.
#[test]
fn a_refusal_is_said_in_the_programs_own_words() {
    let error = harrow::exec::ExecError::Failed {
        code: Some(1),
        stderr: "cairn: 0003 has no `Approach` section to split\n".into(),
    };
    assert_eq!(
        error.said(),
        "cairn: 0003 has no `Approach` section to split"
    );
}

#[test]
fn without_split_the_palette_does_not_offer_it() {
    let mut app = app();
    app.cannot.push(Command::Split);
    app.set_keymap(Keymap::default());
    app.handle_key(KeyCode::Char(':'), KeyModifiers::NONE);
    for c in "split".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    let offered: Vec<&str> = app
        .palette
        .as_ref()
        .expect("the palette is open")
        .matches
        .iter()
        .map(|(c, _)| c.name())
        .collect();
    assert!(!offered.contains(&"split"), "{offered:?}");
    assert_eq!(app.run(Command::Split), Action::None);
}

/// The one mark is what a write acts on, wherever the cursor is.
#[test]
fn a_single_mark_is_what_is_split() {
    let mut app = app();
    app.select_id(5.into());
    app.run(Command::ToggleGroup);
    app.select_id(3.into());
    assert_eq!(app.run(Command::Split), Action::Split(5.into()));
}

/// More children than the screen holds are said to be there.
#[test]
fn a_long_split_says_how_many_more_it_makes() {
    let mut app = app();
    let many: String = (10..40).map(|n| format!("  00{n} Step {n}\n")).collect();
    app.show_split(3.into(), Ok(many));
    let screen = ui::render_to_string(&mut app, 110, 24, 0);
    assert!(screen.contains("more"), "{screen}");
    assert!(screen.contains("Split 0003 into 30 items?"), "{screen}");
}

#[test]
fn one_item_at_a_time() {
    let mut app = app();
    app.run(Command::ToggleGroup);
    app.select_id(5.into());
    app.run(Command::ToggleGroup);
    assert_eq!(app.run(Command::Split), Action::None);
    assert!(app.toast.clone().unwrap().0.contains("one item"));
}
