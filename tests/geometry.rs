//! Every screen, at every size it can be given.
//!
//! The recorded screens are taken at a handful of sizes and the randomised
//! suite draws at two. Between those lies every size a terminal actually is,
//! and every overlay at every one of them.
//!
//! The overlays are the risk. Each computes a centred rectangle from the
//! available area and then subtracts borders and padding from it, and every
//! one of those subtractions is a place where a width of three and a height
//! of two produce either a panic or a box drawn outside itself.
//!
//! Not snapshots: five lenses times seven overlays times forty sizes is a
//! thousand recorded files nobody will read, and a snapshot is worth having
//! only because somebody reads the diff. This asserts a property — it
//! renders, and it stays inside the lines.

use harrow::app::{App, Pane};
use harrow::{testkit, ui};

/// One screen harrow can show: a name for the failure message, and what
/// puts harrow into it.
struct Screen {
    name: String,
    show: Box<dyn Fn(&mut App)>,
}

/// Every screen harrow can show.
///
/// The lenses are asked of `Pane::ALL` rather than listed, so a lens added
/// without a thought for narrow terminals is caught here rather than by
/// somebody on a small one.
fn screens() -> Vec<Screen> {
    let mut out: Vec<Screen> = Vec::new();
    for lens in Pane::ALL {
        out.push(Screen {
            name: format!("the {} lens", lens.name()),
            show: Box::new(move |app: &mut App| app.pane = lens),
        });
    }
    let overlay = |name: &str, show: Box<dyn Fn(&mut App)>| Screen {
        name: name.to_string(),
        show,
    };
    out.push(overlay(
        "the reader",
        Box::new(|a: &mut App| a.reading = true),
    ));
    out.push(overlay("the help", Box::new(|a: &mut App| a.help = true)));
    out.push(overlay(
        "the diagnostics",
        Box::new(|a: &mut App| a.diagnostics = true),
    ));
    out.push(overlay(
        "the picker",
        Box::new(|a: &mut App| a.open_picker("status")),
    ));
    out.push(overlay(
        "the confirmation",
        Box::new(|a: &mut App| a.ask_close()),
    ));
    out.push(overlay(
        "the filter box",
        Box::new(|a: &mut App| {
            use crossterm::event::{KeyCode, KeyModifiers};
            a.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
            for c in "priority=".chars() {
                a.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
            }
        }),
    ));
    out.push(overlay(
        "the history",
        Box::new(|a: &mut App| {
            a.show_history(3, Ok("2026-09-11  somebody  status doing → done\n".into()))
        }),
    ));
    out.push(overlay(
        "a toast",
        Box::new(|a: &mut App| a.toast("something happened", harrow::app::ToastKind::Good)),
    ));
    out
}

/// Down to the smallest a terminal can be, and out past the widest anybody
/// uses. The awkward ones are deliberate: the thresholds at which the detail
/// pane and the header change their minds, and one either side of each.
const SIZES: &[(u16, u16)] = &[
    (1, 1),
    (2, 2),
    (3, 1),
    (8, 3),
    (12, 5),
    (20, 6),
    (24, 8),
    (40, 10),
    (60, 12),
    (73, 14),
    (74, 14),
    (80, 24),
    (95, 20),
    (96, 20),
    (110, 26),
    (121, 22),
    (122, 22),
    (131, 30),
    (132, 30),
    (200, 50),
    (400, 12),
];

fn ready(setup: &dyn Fn(&mut App)) -> App {
    let mut app = testkit::app();
    app.loading = false;
    app.now = 1_789_084_800;
    app.me = "oddur".into();
    app.show_activity(Ok(
        "abc\u{1f}oddur\u{1f}2026-09-10T09:00:00Z\u{1f}a change\n\
         items/0003-draw-the-list.md\n"
            .into(),
    ));
    app.select_id(3);
    setup(&mut app);
    app
}

#[test]
fn every_screen_renders_at_every_size() {
    for Screen { name, show } in screens() {
        for (w, h) in SIZES {
            let mut app = ready(show.as_ref());
            // A panic here is the test: the backend cannot fail to draw, so
            // anything that goes wrong is arithmetic in the layout.
            let buffer = ui::render_frame(&mut app, *w, *h, 0);
            assert_eq!(
                buffer.area.width, *w,
                "{name} at {w}x{h} drew the wrong width"
            );
            assert_eq!(
                buffer.area.height, *h,
                "{name} at {w}x{h} drew the wrong height"
            );
        }
    }
}

/// Nothing is registered as clickable outside the screen it was drawn on. A
/// hit region beyond the edge is a click that lands on something invisible.
#[test]
fn nothing_is_clickable_outside_the_screen() {
    let mut outside = Vec::new();
    for Screen { name, show } in screens() {
        for (w, h) in SIZES {
            let mut app = ready(show.as_ref());
            let _ = ui::render_frame(&mut app, *w, *h, 0);
            for (area, hit) in &app.hits {
                if area.x + area.width > *w || area.y + area.height > *h {
                    outside.push(format!("{name} at {w}x{h}: {hit:?} at {area:?}"));
                }
            }
        }
    }
    assert!(
        outside.is_empty(),
        "{} hit regions off the screen:\n{}",
        outside.len(),
        outside.join("\n")
    );
}

/// Every size, driven rather than merely drawn: a screen that renders once
/// and falls over when somebody presses a key on it is not much use.
#[test]
fn every_screen_survives_being_used_at_every_size() {
    use crossterm::event::{KeyCode, KeyModifiers};
    let keys = [
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::End,
        KeyCode::Enter,
        KeyCode::Tab,
        KeyCode::Esc,
    ];
    for Screen { name, show } in screens() {
        for (w, h) in [(1u16, 1u16), (20, 6), (40, 10), (96, 20), (200, 50)] {
            let mut app = ready(show.as_ref());
            for key in keys {
                let _ = ui::render_frame(&mut app, w, h, 0);
                let _ = app.handle_key(key, KeyModifiers::NONE);
                assert!(
                    app.check_invariants().is_ok(),
                    "{name} at {w}x{h} after {key:?}: {:?}",
                    app.check_invariants()
                );
            }
        }
    }
}
