//! Randomised input, held to the invariants.
//!
//! The interface is a state machine with modal layers, several views and a
//! filter that rebuilds everything underneath the cursor. Enumerating the ways
//! those interact by hand is hopeless; asserting what must be true after *any*
//! sequence of them is not.
//!
//! `HARROW_FUZZ_SEEDS=100000 cargo test --release --test invariants` runs it
//! harder. The default is small enough to belong in every test run.

use crossterm::event::{KeyCode, KeyModifiers};

use harrow::app::App;
use harrow::testkit;
use harrow::ui;

/// A named, seeded generator. `Math.random` in a test is a bug you cannot
/// reproduce.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*, which is plenty for choosing between forty keys.
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Every key a user could press, including the ones that make no sense where
/// they are pressed.
const KEYS: &[KeyCode] = &[
    KeyCode::Char('j'),
    KeyCode::Char('k'),
    KeyCode::Char('J'),
    KeyCode::Char('K'),
    KeyCode::Char('g'),
    KeyCode::Char('G'),
    KeyCode::Char(' '),
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Tab,
    KeyCode::Char('v'),
    KeyCode::Enter,
    KeyCode::Char('c'),
    KeyCode::Char('C'),
    KeyCode::Char('x'),
    KeyCode::Char('u'),
    KeyCode::Char('n'),
    KeyCode::Char('s'),
    KeyCode::Char('p'),
    KeyCode::Char('M'),
    KeyCode::Char('l'),
    KeyCode::Char('h'),
    KeyCode::Char('y'),
    KeyCode::Char('/'),
    KeyCode::Char('a'),
    KeyCode::Char('D'),
    KeyCode::Char('?'),
    KeyCode::Char('m'),
    KeyCode::Char('1'),
    KeyCode::Char('d'),
    KeyCode::Char('y'),
    KeyCode::Char('z'),
    KeyCode::Backspace,
    KeyCode::Esc,
    KeyCode::PageDown,
    KeyCode::PageUp,
    KeyCode::Home,
    KeyCode::End,
];

fn seeds() -> u64 {
    std::env::var("HARROW_FUZZ_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400)
}

#[test]
fn no_sequence_of_keys_leaves_the_state_inconsistent() {
    let mut rng = Rng(0x5eed_1234_9abc_def1);
    for seed in 0..seeds() {
        let mut app = testkit::app();
        let mut history: Vec<KeyCode> = Vec::new();
        for _ in 0..40 {
            let key = KEYS[rng.below(KEYS.len())];
            history.push(key);
            // Quit is the one key that ends the session rather than changing it.
            let _ = app.handle_key(key, KeyModifiers::NONE);
            if let Err(problem) = app.check_invariants() {
                panic!("seed {seed}: {problem}\nafter {history:?}");
            }
        }
    }
}

#[test]
fn no_sequence_of_keys_makes_a_frame_that_cannot_be_drawn() {
    let mut rng = Rng(0xf00d_4321_1111_2222);
    for seed in 0..seeds().min(80) {
        let mut app = testkit::app();
        for _ in 0..25 {
            let _ = app.handle_key(KEYS[rng.below(KEYS.len())], KeyModifiers::NONE);
            // Two sizes: the one people use, and one small enough to expose an
            // arithmetic mistake in the layout.
            let _ = ui::render_frame(&mut app, 100, 30, 0);
            let _ = ui::render_frame(&mut app, 24, 8, 0);
        }
        assert!(app.check_invariants().is_ok(), "seed {seed}");
    }
}

#[test]
fn an_empty_backlog_survives_the_same_treatment() {
    // Every "the selected item" path, with nothing selected and nothing to
    // select. This is where an unwrap would be found.
    let mut rng = Rng(0xabcd_0000_ffff_0001);
    let mut app = App::new();
    for _ in 0..2000 {
        let _ = app.handle_key(KEYS[rng.below(KEYS.len())], KeyModifiers::NONE);
        assert!(app.check_invariants().is_ok());
    }
    let _ = ui::render_frame(&mut app, 80, 24, 0);
}

#[test]
fn a_filter_that_hides_everything_still_leaves_a_usable_screen() {
    let mut app = testkit::app();
    app.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
    for c in "nothing matches this".chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
    }
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.rows.is_empty());
    assert!(app.check_invariants().is_ok());

    // And every command still has to be safe with nothing under the cursor.
    for command in harrow::keys::Command::ALL {
        let _ = app.run(command);
        assert!(app.check_invariants().is_ok(), "{command:?}");
    }
    let _ = ui::render_frame(&mut app, 90, 24, 0);
}
