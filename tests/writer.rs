//! A write runs in the background and is waited on, not killed, until well
//! past cairn's own bounds. `sh` stands in for cairn: what is under test is
//! the waiting, not cairn.

use std::time::{Duration, Instant};

use harrow::app::Change;
use harrow::exec::ExecError;
use harrow::writer::{LIMIT, Writing};

fn script(describe: &str, sh: &str) -> Change {
    Change {
        args: vec!["-c".into(), sh.into()],
        describe: describe.into(),
        undo: None,
    }
}

/// The interface hands a write over and goes straight back to its keys.
#[test]
fn starting_a_write_does_not_wait_for_it() {
    let started = Instant::now();
    let mut write = Writing::start("sh", script("slow", "sleep 1; echo landed"), LIMIT);
    assert!(
        started.elapsed() < Duration::from_millis(200),
        "start waited {:?}",
        started.elapsed()
    );
    assert!(write.poll().is_none(), "not yet");
    let done = write.wait();
    assert_eq!(done.outcome.unwrap().0.trim(), "landed");
}

/// Past the point where harrow used to kill it, a write is said to be still
/// going — once — and it still ends.
#[test]
fn a_slow_write_is_said_to_be_still_going_once_and_then_ends() {
    let mut write = Writing::start("sh", script("0003 closed", "sleep 0.6"), LIMIT);
    assert!(!write.overdue(Duration::from_millis(200)), "not yet");
    std::thread::sleep(Duration::from_millis(300));
    assert!(write.overdue(Duration::from_millis(200)));
    assert!(!write.overdue(Duration::from_millis(200)), "said once");
    assert!(write.wait().outcome.is_ok());
}

/// cairn breaks a lock held for five minutes as abandoned, so a write is not
/// stopped before then: a claim can hold the project's lock for two minutes
/// of waiting on top of two.
#[test]
fn the_limit_is_cairns_own_age_for_an_abandoned_lock() {
    assert!(LIMIT >= Duration::from_secs(300));
}

/// A write that did not succeed failed, whatever it printed first: a bulk
/// change that stopped on its third item has said something about two.
#[test]
fn a_write_that_fails_after_printing_is_a_failure() {
    let write = Writing::start(
        "sh",
        script(
            "three items",
            "echo updated 0003; echo 'cairn: 0009: no' >&2; exit 1",
        ),
        LIMIT,
    );
    match write.wait().outcome {
        Err(ExecError::Failed { code, stderr }) => {
            assert_eq!(code, Some(1));
            assert_eq!(stderr.trim(), "cairn: 0009: no");
        }
        other => panic!("expected a failure, got {:?}", other.map(|_| ())),
    }
}

/// At the limit the write is stopped, and so is anything it started: the hook
/// that ran past it is the reason there is a limit at all.
#[cfg(unix)]
#[test]
fn a_write_past_the_limit_is_stopped_with_everything_it_started() {
    let dir = tempfile::tempdir().unwrap();
    let pid = dir.path().join("hook.pid");
    let write = Writing::start(
        "sh",
        script(
            "hung hook",
            &format!("sleep 30 & echo $! > '{}'; wait", pid.display()),
        ),
        Duration::from_millis(500),
    );
    let done = write.wait();
    assert!(
        matches!(done.outcome, Err(ExecError::Stopped { whole: true, .. })),
        "stopped, all of it"
    );
    let hook = std::fs::read_to_string(&pid).unwrap();
    let alive = |pid: &str| {
        std::process::Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    let until = Instant::now() + Duration::from_secs(5);
    while alive(&hook) && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!alive(&hook), "the hook it started outlived it");
}

/// A hook that says a great deal on stderr cannot fill a pipe and stall the
/// write behind it: the output goes to a file.
#[test]
fn a_write_that_says_a_great_deal_is_not_stalled_by_it() {
    let write = Writing::start(
        "sh",
        script(
            "chatty hook",
            "head -c 1000000 /dev/zero | tr '\\0' x >&2; echo done",
        ),
        Duration::from_secs(30),
    );
    let (out, err) = write.wait().outcome.unwrap();
    assert_eq!(out.trim(), "done");
    assert_eq!(err.len(), 1_000_000);
}

/// A refusal is cairn's, with what it said.
#[test]
fn a_refusal_comes_back_with_what_was_said() {
    let write = Writing::start("sh", script("refused", "echo no >&2; exit 1"), LIMIT);
    match write.wait().outcome {
        Err(ExecError::Failed { code, stderr }) => {
            assert_eq!(code, Some(1));
            assert_eq!(stderr.trim(), "no");
        }
        other => panic!("expected a refusal, got {:?}", other.map(|_| ())),
    }
}
