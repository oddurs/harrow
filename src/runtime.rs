//! The supervised background loop.
//!
//! Reading the backlog happens on one thread that outlives individual reads. The
//! UI never blocks on it: it receives messages, and if the thread is slow,
//! wedged or dead, the UI keeps drawing and says so.
//!
//! A backlog is a directory somebody else is also editing — an editor, a `cairn
//! set`, a `git checkout` — so the loop re-reads on a timer and emits only when
//! something actually changed. A screen that flickered every three seconds
//! whether or not anything happened would be a screen you stop trusting.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TrySendError};
use std::time::{Duration, Instant, SystemTime};

use crate::diag;
use crate::engine::{Report, Source};

/// How long between checks for a backlog that has changed underneath you.
pub const AUTO_REFRESH: Duration = Duration::from_secs(3);
/// A read slower than this is worth complaining about.
pub const SLOW_READ: Duration = Duration::from_millis(500);
/// Bounded so a stalled UI cannot make the loader grow memory without limit.
pub const QUEUE_DEPTH: usize = 64;

pub enum Msg {
    Loading,
    Loaded(Box<Report>),
    LoadFailed { detail: String, transient: bool },
}

enum Command {
    Refresh,
    Shutdown,
}

pub struct Settings {
    pub auto_refresh: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_refresh: AUTO_REFRESH,
        }
    }
}

impl Settings {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            auto_refresh: config.refresh(),
        }
    }
}

/// The UI's handle on the background thread.
pub struct Handle {
    commands: Sender<Command>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Handle {
    /// Ask for an immediate re-read. Silently ignored if the thread has died —
    /// the caller finds out from [`Handle::is_alive`], not from here.
    pub fn refresh(&self) {
        let _ = self.commands.send(Command::Refresh);
    }

    pub fn is_alive(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    /// Stop the loop and wait for it. Called on quit, and on drop.
    pub fn shutdown(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Start the loop. Returns the handle and the receiver the UI drains.
pub fn spawn(source: Box<dyn Source>, settings: Settings) -> (Handle, Receiver<Msg>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (msg_tx, msg_rx) = mpsc::sync_channel::<Msg>(QUEUE_DEPTH);

    let thread = std::thread::Builder::new()
        .name("harrow-loader".into())
        .spawn(move || run(source, settings, cmd_rx, msg_tx))
        .expect("spawn loader thread");

    (
        Handle {
            commands: cmd_tx,
            thread: Some(thread),
        },
        msg_rx,
    )
}

/// Send without ever blocking the loader on a busy UI. A dropped update is
/// recoverable — the next read sends it again — but a blocked loader is not.
fn emit(tx: &SyncSender<Msg>, msg: Msg) -> bool {
    match tx.try_send(msg) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            diag::warn("runtime", "message queue full; dropped an update");
            true
        }
        Err(TrySendError::Disconnected(_)) => false,
    }
}

/// Enough of a reading to tell whether it is worth sending: what was there, and
/// when it last changed. Deliberately not a hash of the contents — a change
/// harrow cannot see this way is one `r` will pick up regardless.
fn fingerprint(report: &Report) -> (usize, Option<SystemTime>) {
    (report.items.len(), report.stamp)
}

fn run(
    mut source: Box<dyn Source>,
    settings: Settings,
    commands: Receiver<Command>,
    out: SyncSender<Msg>,
) {
    let mut consecutive_failures = 0u32;
    let mut last: Option<(usize, Option<SystemTime>)> = None;
    // Set by an explicit `r`: the answer goes to the screen whether or not it
    // differs, because the user asked and silence would read as a broken key.
    let mut forced = true;

    loop {
        if forced && !emit(&out, Msg::Loading) {
            return;
        }

        let started = Instant::now();
        let result = source.load();
        let elapsed = started.elapsed();
        if elapsed > SLOW_READ {
            diag::warn("runtime", format!("read took {}ms", elapsed.as_millis()));
        }

        match result {
            Ok(report) => {
                consecutive_failures = 0;
                let print = fingerprint(&report);
                let changed = last != Some(print);
                last = Some(print);
                if (changed || forced) && !emit(&out, Msg::Loaded(Box::new(report))) {
                    return;
                }
            }
            Err(e) => {
                consecutive_failures += 1;
                diag::error("runtime", format!("read failed: {e}"));
                last = None;
                if !emit(
                    &out,
                    Msg::LoadFailed {
                        detail: e.detail,
                        transient: e.transient,
                    },
                ) {
                    return;
                }
            }
        }
        forced = false;

        // Back off after repeated failures rather than re-reading a directory
        // that is not there every three seconds forever.
        let wait = if consecutive_failures > 0 {
            let factor = 2u32.saturating_pow(consecutive_failures.min(4));
            settings
                .auto_refresh
                .saturating_mul(factor)
                .min(Duration::from_secs(60))
        } else {
            settings.auto_refresh
        };

        let deadline = Instant::now() + wait;
        loop {
            match commands.recv_timeout(Duration::from_millis(100)) {
                Ok(Command::Refresh) => {
                    forced = true;
                    break;
                }
                Ok(Command::Shutdown) => return,
                Err(RecvTimeoutError::Timeout) => {
                    if Instant::now() >= deadline {
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Failing, Static};
    use crate::testkit;

    fn static_source() -> Box<dyn Source> {
        Box::new(Static {
            schema: testkit::schema(),
            items: testkit::report().items,
        })
    }

    fn drain_until<F>(rx: &Receiver<Msg>, mut done: F) -> Vec<Msg>
    where
        F: FnMut(&[Msg]) -> bool,
    {
        let mut got = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let left = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(left.min(Duration::from_millis(200))) {
                Ok(m) => got.push(m),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if done(&got) {
                break;
            }
        }
        got
    }

    #[test]
    fn the_backlog_arrives_without_being_asked_for() {
        let (mut handle, rx) = spawn(static_source(), Settings::default());
        let msgs = drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));
        assert!(msgs.iter().any(|m| matches!(m, Msg::Loading)));
        assert!(
            msgs.iter()
                .any(|m| matches!(m, Msg::Loaded(r) if r.items.len() == 6)),
            "expected the sample backlog"
        );
        handle.shutdown();
        assert!(!handle.is_alive());
    }

    #[test]
    fn an_unchanged_backlog_is_not_sent_again() {
        // The whole reason for the fingerprint: a screen that redrew every
        // three seconds whether or not anything happened is one you stop
        // trusting.
        let (mut handle, rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_millis(60),
            },
        );
        let first = drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));
        assert_eq!(
            first.iter().filter(|m| matches!(m, Msg::Loaded(_))).count(),
            1
        );

        std::thread::sleep(Duration::from_millis(400));
        let more: Vec<Msg> = rx.try_iter().collect();
        assert!(
            !more.iter().any(|m| matches!(m, Msg::Loaded(_))),
            "nothing changed, so nothing should have been sent"
        );
        handle.shutdown();
    }

    #[test]
    fn asking_for_a_refresh_answers_even_when_nothing_changed() {
        let (mut handle, rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_secs(300),
            },
        );
        drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));
        handle.refresh();
        let msgs = drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));
        assert!(
            msgs.iter().any(|m| matches!(m, Msg::Loaded(_))),
            "an explicit refresh has to produce something on screen"
        );
        handle.shutdown();
    }

    #[test]
    fn a_failing_source_reports_and_keeps_running() {
        let (mut handle, rx) = spawn(
            Box::new(Failing("no such directory".into())),
            Settings {
                auto_refresh: Duration::from_millis(50),
            },
        );
        let msgs = drain_until(&rx, |m| {
            m.iter().any(|m| matches!(m, Msg::LoadFailed { .. }))
        });
        assert!(msgs.iter().any(|m| matches!(m, Msg::LoadFailed { .. })));
        assert!(handle.is_alive(), "the loop must survive a failed read");
        handle.shutdown();
    }

    #[test]
    fn shutdown_is_prompt_even_mid_wait() {
        let (mut handle, _rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_secs(300),
            },
        );
        std::thread::sleep(Duration::from_millis(150));
        let started = Instant::now();
        handle.shutdown();
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "shutdown waited on the refresh timer"
        );
    }

    #[test]
    fn dropping_the_handle_stops_the_thread() {
        let (handle, _rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_secs(300),
            },
        );
        drop(handle); // Hangs here if shutdown-on-drop regresses.
    }

    #[test]
    fn a_dropped_receiver_ends_the_loop() {
        let (handle, rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_millis(50),
            },
        );
        drop(rx);
        let deadline = Instant::now() + Duration::from_secs(5);
        while handle.is_alive() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!handle.is_alive(), "the loop outlived its receiver");
    }
}
