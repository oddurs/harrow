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
use harrow::keys::Keymap;
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

/// Every key a user could press: everything the program binds, and a few it
/// does not.
///
/// Asked of the keymap rather than listed here. The list that used to live
/// here fell behind — `t`, `N`, `A`, `C`, `H`, `e`, `o`, `a`, `r`, `h`, and
/// every lens key but the first were bound and never pressed, and no
/// modified key had ever been sent at all — so the suite reported four
/// hundred passing seeds while never touching the most recently written
/// code. A test that can fall behind the thing it tests will.
fn keys() -> Vec<(KeyCode, KeyModifiers)> {
    let mut keys = Keymap::default().every_key();
    // Keys nothing is bound to are a case too, and the point of this suite
    // is the sequences nobody would think to write.
    keys.extend([
        (KeyCode::Char('z'), KeyModifiers::NONE),
        (KeyCode::Char('9'), KeyModifiers::NONE),
        (KeyCode::Char('!'), KeyModifiers::SHIFT),
        (KeyCode::Backspace, KeyModifiers::NONE),
        (KeyCode::Delete, KeyModifiers::NONE),
        (KeyCode::F(1), KeyModifiers::NONE),
    ]);
    keys
}

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
        let keys = keys();
        let mut history: Vec<(KeyCode, KeyModifiers)> = Vec::new();
        for _ in 0..40 {
            let (code, mods) = keys[rng.below(keys.len())];
            history.push((code, mods));
            // Quit is the one key that ends the session rather than changing it.
            let _ = app.handle_key(code, mods);
            if let Err(problem) = app.check_invariants() {
                panic!("seed {seed}: {problem}\nafter {history:?}");
            }
        }
    }
}

#[test]
fn no_sequence_of_keys_makes_a_frame_that_cannot_be_drawn() {
    let keys = keys();
    let mut rng = Rng(0xf00d_4321_1111_2222);
    for seed in 0..seeds().min(80) {
        let mut app = testkit::app();
        for _ in 0..25 {
            let (code, mods) = keys[rng.below(keys.len())];
            let _ = app.handle_key(code, mods);
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
    let keys = keys();
    // Every "the selected item" path, with nothing selected and nothing to
    // select. This is where an unwrap would be found.
    let mut rng = Rng(0xabcd_0000_ffff_0001);
    let mut app = App::new();
    for _ in 0..2000 {
        let (code, mods) = keys[rng.below(keys.len())];
        let _ = app.handle_key(code, mods);
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

/// The list used to be written by hand and fell behind the keymap without
/// anybody noticing, which is the failure a test cannot report about itself.
/// This is the line that would have caught it.
#[test]
fn the_suite_presses_every_key_the_program_binds() {
    let map = Keymap::default();
    let pressed = keys();
    for (code, mods) in map.every_key() {
        assert!(
            pressed.contains(&(code, mods)),
            "{code:?} with {mods:?} is bound and never pressed"
        );
    }

    // Every command reachable, not merely every key — the same thing said
    // from the other end, so a command bound to nothing is caught too.
    for command in harrow::keys::Command::ALL {
        assert!(
            !map.keys_for(command).is_empty(),
            "{command:?} has no key, so nothing can reach it"
        );
    }

    // And a modified key really is in there. It never was.
    assert!(
        pressed
            .iter()
            .any(|(_, m)| m.contains(KeyModifiers::CONTROL)),
        "no modified key is ever sent"
    );
    assert!(
        pressed.len() > map.every_key().len(),
        "keys nothing is bound to are a case too"
    );
}
