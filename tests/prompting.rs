//! An item as the prompt it is: cairn compiles it, harrow shows it, and one
//! key copies it whole.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{Action, App};
use harrow::keys::{Command, Keymap};
use harrow::{testkit, ui};

const PROMPT: &str = "# 0003 Draw the list\n\n## How this project works\n\n- sample\n\n## The task\n\nDraw the list.\n";

fn app() -> App {
    let mut app = testkit::app();
    app.select_id(3.into());
    app
}

fn key(app: &mut App, c: char) -> Action {
    app.handle_key(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Against a cairn without `prompt`.
fn without_prompt() -> App {
    let mut app = app();
    app.cannot.push(Command::Prompt);
    app.set_keymap(Keymap::default());
    app
}

/// The shell runs cairn; `app` only says which item.
#[test]
fn the_key_asks_the_shell_for_the_items_prompt() {
    let mut app = app();
    assert_eq!(key(&mut app, 'P'), Action::Prompt(3.into()));
}

#[test]
fn a_prompt_is_read_in_harrow_and_copied_whole_with_one_key() {
    let mut app = app();
    app.show_prompt(3.into(), Ok(PROMPT.to_string()));
    let screen = ui::render_to_string(&mut app, 110, 30, 0);
    assert!(screen.contains("0003 · prompt"), "{screen}");
    assert!(screen.contains("The task"), "{screen}");
    assert!(screen.contains("y copies it whole"), "{screen}");

    assert_eq!(key(&mut app, 'y'), Action::Copy(PROMPT.to_string()));
    assert!(app.prompt.is_none(), "and it is put away");
}

#[test]
fn any_other_key_puts_it_away_without_copying() {
    let mut app = app();
    app.show_prompt(3.into(), Ok(PROMPT.to_string()));
    assert_eq!(key(&mut app, 'q'), Action::None);
    assert!(app.prompt.is_none());
}

/// What cairn said when it failed, in the footer — never an empty overlay.
#[test]
fn a_failing_prompt_is_reported_rather_than_shown_empty() {
    let mut app = app();
    app.show_prompt(
        3.into(),
        Err("exited 1: cairn: no item 0003 in this project".into()),
    );
    assert!(app.prompt.is_none());
    let (said, _, _) = app.toast.clone().expect("a toast");
    assert!(said.contains("no item 0003 in this project"), "{said}");

    app.show_prompt(3.into(), Ok("\n".into()));
    assert!(app.prompt.is_none(), "nothing said is not a prompt");
}

#[test]
fn without_prompt_the_key_is_not_bound_and_nothing_offers_it() {
    let mut app = without_prompt();
    assert_eq!(key(&mut app, 'P'), Action::None);
    assert!(
        app.keymap
            .help_rows()
            .iter()
            .all(|(_, d)| *d != Command::Prompt.describe()),
        "the help does not list it"
    );
    key(&mut app, ':');
    for c in "prompt".chars() {
        key(&mut app, c);
    }
    let offered: Vec<&str> = app
        .palette
        .as_ref()
        .expect("the palette is open")
        .matches
        .iter()
        .map(|(c, _)| c.name())
        .collect();
    assert!(!offered.contains(&"prompt"), "{offered:?}");
}

#[test]
fn with_prompt_the_help_lists_it() {
    let app = app();
    assert!(
        app.keymap
            .help_rows()
            .iter()
            .any(|(k, d)| *d == Command::Prompt.describe() && k.contains('P'))
    );
}

/// Reloading the config hands over a fresh keymap; what this cairn cannot do
/// stays withheld from it.
#[test]
fn a_reloaded_keymap_still_withholds_it() {
    let mut app = without_prompt();
    app.set_keymap(Keymap::default());
    assert_eq!(key(&mut app, 'P'), Action::None);
}

#[test]
fn copying_a_prompt_says_what_and_how_much() {
    let mut app = app();
    app.copied(PROMPT);
    let (said, _, _) = app.toast.clone().expect("a toast");
    assert_eq!(
        said,
        format!(
            "copied # 0003 Draw the list — {} characters",
            PROMPT.chars().count()
        )
    );
    app.copied("0003");
    assert_eq!(app.toast.clone().unwrap().0, "copied 0003");
}

/// The wheel scrolls the overlay being read, and nothing behind it moves.
#[test]
fn the_wheel_scrolls_the_prompt_and_not_the_rows_behind_it() {
    use crossterm::event::{MouseEvent, MouseEventKind};
    let mut app = app();
    let long: String = (0..80).map(|n| format!("line {n}\n\n")).collect();
    app.show_prompt(3.into(), Ok(long));
    let _ = ui::render_frame(&mut app, 110, 30, 0);
    let before = app.selected_item().map(|i| i.id);
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 55,
        row: 15,
        modifiers: KeyModifiers::NONE,
    });
    assert!(
        app.prompt.as_ref().is_some_and(|p| p.scroll > 0),
        "it scrolled"
    );
    assert_eq!(app.selected_item().map(|i| i.id), before, "the rows stayed");
}
