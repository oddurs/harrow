//! What a group has already finished.
//!
//! A heading that says `53%` and `6 left` is making a claim about seven items
//! that are nowhere on screen. 0088 fixed that at the endpoint — a milestone
//! with *nothing* left shows its work — and left the middle alone, so a
//! finished milestone showed everything and a half-finished one showed none of
//! it. Both were visible on one screen, which is worse than either.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::{App, Row};
use harrow::{testkit, ui};

fn rows(app: &App) -> Vec<String> {
    app.rows
        .iter()
        .map(|r| match r {
            Row::Group(g) => format!("group {}", app.groups[*g].key),
            Row::Finished(g) => format!("fold {}", app.groups[*g].key),
            Row::Item(i) => format!(
                "{} {}",
                if app.items[*i].category.is_closed() {
                    "done"
                } else {
                    "open"
                },
                app.items[*i].id
            ),
        })
        .collect()
}

/// Put the cursor on the fold for a group.
fn on_fold(app: &mut App, key: &str) {
    let at = app
        .rows
        .iter()
        .position(|r| matches!(r, Row::Finished(g) if app.groups[*g].key == key))
        .unwrap_or_else(|| panic!("no fold for {key}: {:?}", rows(app)));
    app.selected = at;
}

#[test]
fn a_group_says_how_much_it_has_finished() {
    let app = testkit::app();
    let said = rows(&app);
    assert!(said.contains(&"fold v0.1".to_string()), "{said:?}");
    assert_eq!(app.folded_away.get("v0.1").copied(), Some(1));
}

/// A group with nothing finished is unchanged, so a fresh backlog looks
/// exactly as it did.
#[test]
fn a_group_with_nothing_finished_gets_no_row() {
    let app = testkit::app();
    let said = rows(&app);
    assert!(said.contains(&"group ".to_string()), "{said:?}");
    assert!(
        !said.contains(&"fold ".to_string()),
        "the group with no finished work got a fold: {said:?}"
    );
}

/// The fold is a container: its contents belong under it, not sorted in among
/// the open work.
#[test]
fn unfolding_lists_the_finished_work_above_the_open_work() {
    let mut app = testkit::app();
    on_fold(&mut app, "v0.1");
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    let said = rows(&app);
    let fold = said.iter().position(|r| r == "fold v0.1").unwrap();
    let done = said.iter().position(|r| r.starts_with("done")).unwrap();
    let open = said.iter().position(|r| r.starts_with("open")).unwrap();
    assert!(
        fold < done,
        "the finished work is not under the fold: {said:?}"
    );
    assert!(
        done < open,
        "the finished work is not above the open: {said:?}"
    );
}

#[test]
fn folding_it_again_puts_it_away() {
    let mut app = testkit::app();
    let before = rows(&app);
    on_fold(&mut app, "v0.1");
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    on_fold(&mut app, "v0.1");
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(rows(&app), before);
}

/// The same gesture as collapsing, on the same key.
#[test]
fn space_folds_it_too() {
    let mut app = testkit::app();
    on_fold(&mut app, "v0.1");
    app.handle_key(KeyCode::Char(' '), KeyModifiers::NONE);
    assert!(app.unfolded.contains("v0.1"));
}

/// A rule that revealed finished work when the cursor entered a group would
/// make rows appear and disappear as you moved, and take the scroll with them.
#[test]
fn nothing_appears_or_disappears_as_the_cursor_moves() {
    let mut app = testkit::app();
    let before = rows(&app);
    for _ in 0..app.rows.len() + 2 {
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(rows(&app), before, "the rows moved under the cursor");
    }
}

/// 0088's rule stays: there the fold would be the only thing under the
/// heading, and folding it would leave a hole.
#[test]
fn a_milestone_with_nothing_left_still_shows_its_work_unasked() {
    let app = testkit::finished();
    let said = rows(&app);
    assert!(
        said.iter().any(|r| r.starts_with("done")),
        "the finished milestone hid its work: {said:?}"
    );
    assert!(
        !said.iter().any(|r| r.starts_with("fold")),
        "it was offered a fold it does not need: {said:?}"
    );
}

/// What the fold offers is what matches — not the group's own tally, which
/// ignores the filter on purpose so a heading's progress does not move when
/// you narrow the list.
#[test]
fn the_count_is_of_what_the_filter_would_show() {
    let mut app = testkit::app();
    assert_eq!(app.folded_away.get("v0.1").copied(), Some(1));
    app.filter = "priority=p1".into();
    app.ingest(testkit::report());
    assert_eq!(
        app.folded_away.get("v0.1").copied().unwrap_or(0),
        0,
        "it is offering to show work the filter excludes"
    );
}

/// The row has to say which way it goes, and the screen has to show it.
#[test]
fn the_row_says_what_pressing_it_does() {
    let mut app = testkit::app();
    let screen = ui::render_to_string(&mut app, 110, 26, 0);
    assert!(screen.contains("1 done"), "{screen}");
    assert!(screen.contains("↵ shows"), "{screen}");

    on_fold(&mut app, "v0.1");
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    let screen = ui::render_to_string(&mut app, 110, 26, 0);
    assert!(screen.contains("↵ folds"), "{screen}");
}

/// A collapsed group is folded away entirely; offering to unfold part of it
/// would be two controls disagreeing about whether it is open.
#[test]
fn a_collapsed_group_offers_nothing_to_unfold() {
    let mut app = testkit::app();
    app.collapsed.insert("v0.1".into());
    app.rebuild();
    let said = rows(&app);
    assert!(!said.contains(&"fold v0.1".to_string()), "{said:?}");
}
