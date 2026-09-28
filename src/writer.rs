//! A write to the backlog, made off the thread the interface runs on.
//!
//! A write used to run on the interface thread and be killed at a deadline.
//! The screen froze while it ran, and a cairn killed while it held the
//! project's lock left the lock behind, so the next writer failed for minutes.
//! Cairn now ends every wait for its lock on its own, so waiting for it costs
//! nothing and killing it risks a stranded lock. This waits.
//!
//! One at a time. A change is worked out from what is on screen, and what is
//! on screen has not caught up with a change still being made, so a second one
//! is refused until the first lands (`App::accept_write`) rather than queued.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crate::app::Change;
use crate::exec::{self, ExecError};

/// How long a write may run before it is stopped.
///
/// cairn's own age for a stale lock: a lock held longer than this is broken
/// by the next writer as abandoned, so stopping cairn past it strands nothing
/// that is not already treated as stranded. Shorter would not do — a claim
/// holds the project's lock while it waits its turn for the lock every
/// worktree shares, two minutes of waiting on top of two minutes.
pub const LIMIT: Duration = Duration::from_secs(300);

/// A write that has ended, and what cairn said.
pub struct Done {
    pub change: Change,
    /// stdout and stderr, or why it did not succeed.
    pub outcome: Result<(String, String), ExecError>,
}

/// A write in flight.
pub struct Writing {
    change: Change,
    started: Instant,
    said: bool,
    done: Receiver<Done>,
}

impl Writing {
    /// Start a write, and return at once.
    pub fn start(program: &str, change: Change, limit: Duration) -> Writing {
        let (report, done) = mpsc::channel();
        let program = program.to_string();
        let job = change.clone();
        std::thread::spawn(move || {
            let args: Vec<&str> = job.args.iter().map(String::as_str).collect();
            let outcome = exec::run_write(&program, &args, limit);
            // Nobody listening means harrow has gone; there is no one to tell.
            let _ = report.send(Done {
                change: job,
                outcome,
            });
        });
        Writing {
            change,
            started: Instant::now(),
            said: false,
            done,
        }
    }

    /// The write, once it has ended. A writer that died without saying so
    /// ends it too, with that as the reason, so nothing waits on it forever.
    pub fn poll(&mut self) -> Option<Done> {
        match self.done.try_recv() {
            Ok(done) => Some(done),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(self.lost()),
        }
    }

    /// Wait for it to end. Quitting does this, so nothing is cut off partway.
    pub fn wait(self) -> Done {
        match self.done.recv() {
            Ok(done) => done,
            Err(_) => self.lost(),
        }
    }

    /// Whether it has been going for longer than `after`: true once, so the
    /// footer is told and not nagged.
    pub fn overdue(&mut self, after: Duration) -> bool {
        if self.said || self.started.elapsed() < after {
            return false;
        }
        self.said = true;
        true
    }

    pub fn describe(&self) -> &str {
        &self.change.describe
    }

    fn lost(&self) -> Done {
        Done {
            change: self.change.clone(),
            outcome: Err(ExecError::Spawn(std::io::Error::other(
                "the writer stopped before cairn answered",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A writer that died without a word still ends the write, so the slot
    /// it holds is freed.
    #[test]
    fn a_writer_that_died_still_ends_the_write() {
        let (report, done) = mpsc::channel::<Done>();
        drop(report);
        let mut write = Writing {
            change: Change {
                args: vec!["close".into(), "3".into()],
                describe: "0003 closed".into(),
                undo: None,
            },
            started: Instant::now(),
            said: false,
            done,
        };
        let ended = write.poll().expect("it ends");
        assert_eq!(ended.change.describe, "0003 closed");
        assert!(matches!(ended.outcome, Err(ExecError::Spawn(_))));
    }
}
