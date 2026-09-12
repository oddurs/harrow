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

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TrySendError};
use std::time::{Duration, Instant, SystemTime};

use notify::{RecursiveMode, Watcher};

use crate::diag;
use crate::engine::{Report, Source};

/// How long between checks for a backlog that has changed underneath you.
///
/// The watcher makes this a backstop rather than the mechanism: it is what
/// notices a change on a network mount, in a container bind, or anywhere else
/// the operating system declines to tell anybody anything.
pub const AUTO_REFRESH: Duration = Duration::from_secs(3);
/// How long to wait for a burst of filesystem events to finish.
///
/// One `cairn set` writes an item, renames it into place and rewrites the
/// roadmap; an editor does its own dance. Reading after the first event would
/// read a directory mid-write.
pub const SETTLE: Duration = Duration::from_millis(120);
/// The floor between two reads, however much noise the directory is making.
pub const MIN_INTERVAL: Duration = Duration::from_millis(250);
/// A read slower than this is worth complaining about.
pub const SLOW_READ: Duration = Duration::from_millis(500);
/// Bounded so a stalled UI cannot make the loader grow memory without limit.
pub const QUEUE_DEPTH: usize = 64;

pub enum Msg {
    Loading,
    Loaded(Box<Report>),
    LoadFailed {
        detail: String,
        transient: bool,
    },
    /// Read it, nothing had changed. The UI does nothing with this, and that is
    /// the point: sending *something* every cycle is how the loop finds out its
    /// receiver has gone away. Without it a backlog nobody is editing keeps a
    /// thread alive after the interface it was feeding has stopped listening.
    Idle,
}

enum Command {
    Refresh,
    Shutdown,
}

pub struct Settings {
    pub auto_refresh: Duration,
    /// Watch the filesystem as well as polling it.
    pub watch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_refresh: AUTO_REFRESH,
            watch: true,
        }
    }
}

impl Settings {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            auto_refresh: config.refresh(),
            watch: config.watch,
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

/// What is being watched, and the watcher doing it.
///
/// The two have to change together — a watcher replaced without its paths
/// would be re-created on every read — so they are one thing. Dropping it is
/// what stops the watching.
#[derive(Default)]
struct Watching {
    watcher: Option<notify::RecommendedWatcher>,
    paths: Vec<PathBuf>,
}

impl Watching {
    fn follow(&mut self, wanted: Vec<PathBuf>, wake: &Sender<()>) {
        if wanted == self.paths && self.watcher.is_some() {
            return;
        }
        self.watcher = watch(&wanted, wake.clone());
        self.paths = if self.watcher.is_some() {
            wanted
        } else {
            Vec::new()
        };
    }
}

/// Watch the directories a project lives in, waking the loop when they change.
///
/// Returns `None` where the platform cannot: a watcher that could not be
/// created is a reason to keep polling, not a reason to stop.
fn watch(paths: &[PathBuf], wake: Sender<()>) -> Option<notify::RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        // Anything that changed the directory is worth a look. Deciding which
        // events matter means encoding one platform's idea of a write, and the
        // read that follows is a directory listing.
        if event.is_ok() {
            let _ = wake.send(());
        }
    })
    .map_err(|e| diag::warn("watch", format!("no filesystem watcher: {e}")))
    .ok()?;

    let mut watching = 0;
    for path in paths {
        // Recursive, because items may be filed in subdirectories and a
        // change below the top level would otherwise wait for the poll — the
        // watcher exists precisely so that it does not.
        let mode = if path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        match watcher.watch(path, mode) {
            Ok(()) => watching += 1,
            Err(e) => diag::warn("watch", format!("{}: {e}", path.display())),
        }
    }
    if watching == 0 {
        return None;
    }
    diag::info(
        "watch",
        format!("watching {watching} path(s); the poll stays as a backstop"),
    );
    Some(watcher)
}

fn run(
    mut source: Box<dyn Source>,
    settings: Settings,
    commands: Receiver<Command>,
    out: SyncSender<Msg>,
) {
    let mut consecutive_failures = 0u32;
    let (wake_tx, wake_rx) = mpsc::channel::<()>();
    let mut watching = Watching::default();
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

                // The paths are only known once something has been read, and
                // they change if the project's `dir` does.
                if settings.watch {
                    watching.follow(
                        vec![report.schema.items_dir(), source.config_path()],
                        &wake_tx,
                    );
                }

                let print = fingerprint(&report);
                let changed = last != Some(print);
                last = Some(print);
                let sent = if changed || forced {
                    emit(&out, Msg::Loaded(Box::new(report)))
                } else {
                    emit(&out, Msg::Idle)
                };
                if !sent {
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

        let read_at = Instant::now();
        let deadline = read_at + wait;
        loop {
            // A command is answered before anything else. Without this, a
            // directory producing events faster than they are drained — which
            // inotify will, one per write, where FSEvents coalesces — means the
            // loop breaks on a wake every time and never reaches the channel
            // carrying Shutdown. The thread then outlives the interface and
            // `join` waits for it forever: a busy backlog made harrow
            // unquittable.
            match commands.try_recv() {
                Ok(Command::Shutdown) => return,
                Ok(Command::Refresh) => {
                    forced = true;
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => return,
                Err(mpsc::TryRecvError::Empty) => {}
            }

            // A change on disk is the reason to look again; the timer is what
            // catches the changes the filesystem never mentioned.
            if wake_rx.try_recv().is_ok() {
                // Let the burst finish, then take everything it sent.
                std::thread::sleep(SETTLE);
                while wake_rx.try_recv().is_ok() {}
                // And never read faster than this, however much noise
                // something else is making in the directory.
                let since = read_at.elapsed();
                if since < MIN_INTERVAL {
                    std::thread::sleep(MIN_INTERVAL - since);
                    while wake_rx.try_recv().is_ok() {}
                }
                break;
            }
            match commands.recv_timeout(Duration::from_millis(50)) {
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

/// Whether this platform will watch that directory, without keeping a watcher.
///
/// The doctor asks so it can say which mechanism a project is actually running
/// on, rather than leaving somebody to wonder why a change took three seconds.
pub fn can_watch(path: &Path) -> Result<(), String> {
    let (tx, _rx) = mpsc::channel::<()>();
    let mut watcher = notify::recommended_watcher(move |_| {
        let _ = tx.send(());
    })
    .map_err(|e| e.to_string())?;
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())
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
                watch: false,
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
                watch: false,
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

    /// The criterion this exists for: a change made in another window shows up
    /// without pressing anything. The poll is set to a minute, so nothing but
    /// the watcher can explain the answer arriving.
    #[test]
    fn a_change_on_disk_arrives_without_being_asked_for() {
        let dir = testkit::project();
        let source = Box::new(crate::engine::Project::discover(dir.path()).expect("found"));
        let (mut handle, rx) = spawn(
            source,
            Settings {
                auto_refresh: Duration::from_secs(60),
                watch: true,
            },
        );
        drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));

        std::fs::write(
            dir.path().join("items/0099-written-by-somebody-else.md"),
            "---\nid: 99\ntitle: Written by somebody else\ntype: feature\nstatus: backlog\n---\n",
        )
        .expect("write an item");

        let started = Instant::now();
        let msgs = drain_until(&rx, |m| {
            m.iter()
                .any(|m| matches!(m, Msg::Loaded(r) if r.items.len() == 7))
        });
        assert!(
            msgs.iter()
                .any(|m| matches!(m, Msg::Loaded(r) if r.items.len() == 7)),
            "the new item never arrived"
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "took {:?}, which means the poll found it rather than the watcher",
            started.elapsed()
        );
        handle.shutdown();
    }

    /// And where there is no watcher — a network mount, a platform that will
    /// not — the poll is still what it always was.
    #[test]
    fn the_poll_still_works_with_the_watcher_turned_off() {
        let dir = testkit::project();
        let source = Box::new(crate::engine::Project::discover(dir.path()).expect("found"));
        let (mut handle, rx) = spawn(
            source,
            Settings {
                auto_refresh: Duration::from_millis(100),
                watch: false,
            },
        );
        drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));

        std::fs::write(
            dir.path().join("items/0099-later.md"),
            "---\nid: 99\ntitle: Later\ntype: feature\nstatus: backlog\n---\n",
        )
        .expect("write an item");

        let msgs = drain_until(&rx, |m| {
            m.iter()
                .any(|m| matches!(m, Msg::Loaded(r) if r.items.len() == 7))
        });
        assert!(
            msgs.iter()
                .any(|m| matches!(m, Msg::Loaded(r) if r.items.len() == 7)),
            "the poll has to keep working on its own"
        );
        handle.shutdown();
    }

    /// Something rewriting the directory in a loop must not turn into a loop
    /// here. `MIN_INTERVAL` is the floor, and this is what holds it.
    #[test]
    fn a_directory_being_rewritten_continuously_is_not_a_busy_loop() {
        let dir = testkit::project();
        let source = Box::new(crate::engine::Project::discover(dir.path()).expect("found"));
        let (mut handle, rx) = spawn(
            source,
            Settings {
                auto_refresh: Duration::from_secs(60),
                watch: true,
            },
        );
        drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));

        let noisy = dir.path().join("items/0099-noisy.md");
        let started = Instant::now();
        let mut n = 0;
        while started.elapsed() < Duration::from_secs(1) {
            n += 1;
            std::fs::write(
                &noisy,
                format!("---\nid: 99\ntitle: Noisy {n}\ntype: feature\nstatus: backlog\n---\n"),
            )
            .expect("write");
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(600));

        let reads = rx
            .try_iter()
            .filter(|m| matches!(m, Msg::Loaded(_)))
            .count();
        assert!(
            reads <= 8,
            "{n} writes in a second produced {reads} reads; the floor is not holding"
        );
        assert!(reads >= 1, "and it still has to notice at all");

        // And it still answers. A stream of events must never starve the one
        // message that stops the thread — the first version of this hung here
        // on Linux, where inotify sends an event per write.
        let started = Instant::now();
        handle.shutdown();
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "shutdown took {:?} with the directory still noisy",
            started.elapsed()
        );
    }

    #[test]
    fn a_failing_source_reports_and_keeps_running() {
        let (mut handle, rx) = spawn(
            Box::new(Failing("no such directory".into())),
            Settings {
                auto_refresh: Duration::from_millis(50),
                watch: false,
            },
        );
        let msgs = drain_until(&rx, |m| {
            m.iter().any(|m| matches!(m, Msg::LoadFailed { .. }))
        });
        assert!(msgs.iter().any(|m| matches!(m, Msg::LoadFailed { .. })));
        assert!(handle.is_alive(), "the loop must survive a failed read");
        handle.shutdown();
    }

    /// The failure this was written for: the first read succeeds, the interface
    /// goes away, and nothing about the backlog ever changes again — so there is
    /// no update to fail to send, and without `Msg::Idle` the loop spins on
    /// forever with nobody listening.
    #[test]
    fn a_receiver_dropped_after_the_first_read_still_ends_the_loop() {
        let (handle, rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_millis(50),
                watch: false,
            },
        );
        drain_until(&rx, |m| m.iter().any(|m| matches!(m, Msg::Loaded(_))));
        drop(rx);

        let deadline = Instant::now() + Duration::from_secs(5);
        while handle.is_alive() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            !handle.is_alive(),
            "the loop outlived the interface it was feeding"
        );
    }

    #[test]
    fn shutdown_is_prompt_even_mid_wait() {
        let (mut handle, _rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_secs(300),
                watch: false,
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
                watch: false,
            },
        );
        drop(handle); // Hangs here if shutdown-on-drop regresses.
    }

    /// The loop must not outlive the interface it is feeding, and the case that
    /// gets this wrong is the quiet one: a backlog nobody is editing produces
    /// no update to fail to send.
    #[test]
    fn a_dropped_receiver_ends_the_loop() {
        let (handle, rx) = spawn(
            static_source(),
            Settings {
                auto_refresh: Duration::from_millis(50),
                watch: false,
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
