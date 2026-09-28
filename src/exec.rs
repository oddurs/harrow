//! Running an external command without ever hanging on it.
//!
//! harrow shells out for exactly one thing — asking `cairn` to change an item —
//! and a write that never returns would wedge the interface with no way to say
//! why. So every subprocess goes through here: piped output drained on its own
//! thread (a full pipe buffer is its own deadlock), a hard deadline, and a kill
//! on expiry.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub enum ExecError {
    /// The binary is not on PATH, or we were not allowed to run it.
    Spawn(std::io::Error),
    /// It ran past its deadline and was killed.
    Timeout(Duration),
    /// It exited non-zero with nothing useful on stdout.
    Failed { code: Option<i32>, stderr: String },
    /// A write ran past its limit and was stopped: all of it, or where the
    /// group could not be signalled, only cairn.
    Stopped { after: Duration, whole: bool },
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::Spawn(e) => write!(f, "could not run: {e}"),
            ExecError::Timeout(d) => write!(f, "timed out after {}ms", d.as_millis()),
            ExecError::Failed { code, stderr } => match code {
                Some(c) => write!(f, "exited {c}: {}", first_line(stderr)),
                None => write!(f, "killed by a signal: {}", first_line(stderr)),
            },
            ExecError::Stopped { after, .. } => write!(f, "stopped after {}s", after.as_secs()),
        }
    }
}

impl std::error::Error for ExecError {}

impl ExecError {
    /// What the program said, in its own words, where it said anything: a
    /// refusal is the program's sentence, and wrapping it in an exit code
    /// changes what the reader is told.
    pub fn said(&self) -> String {
        match self {
            ExecError::Failed { stderr, .. } if !first_line(stderr).is_empty() => {
                first_line(stderr).to_string()
            }
            other => other.to_string(),
        }
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("").trim()
}

/// Run a command, returning stdout. Non-zero exits are tolerated as long as
/// something came back on stdout: a tool that reports what it managed to do
/// before failing is more use than an error with the report thrown away.
/// Everything a git hook exports to what it runs.
///
/// A hook's child inherits `GIT_DIR`, and git reads it in preference to
/// discovering a repository — so `git -C <project> log` run from a hook
/// answers about the *hook's* repository. `-C` changes the directory, not the
/// discovery. Harrow launched from a hook showed another project's history
/// on its log lens for exactly that reason. 0077.
pub(crate) const GIT_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_CEILING_DIRECTORIES",
];

/// `git`, asked about the repository the arguments name and no other.
///
/// Separate from `run` rather than an argument to it: `run` takes an argv and
/// a deadline, and every caller having to say something about the environment
/// to get the ordinary behaviour is a worse trade than git having its own way
/// in. Nothing else harrow spawns is sensitive to where it was launched from.
pub fn git(args: &[&str], timeout: Duration) -> Result<String, ExecError> {
    run_with(("git", GIT_ENV), args, timeout).map(|(out, _)| out)
}

pub fn run(program: &str, args: &[&str], timeout: Duration) -> Result<String, ExecError> {
    run_with((program, &[]), args, timeout).map(|(out, _)| out)
}

/// The same, with what it said on stderr as well as stdout: cairn reports on
/// stdout and advises on stderr.
pub fn run_advised(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(String, String), ExecError> {
    run_with((program, &[]), args, timeout)
}

/// A write: cairn told to change something, waited on for as long as `limit`.
///
/// Its output goes to files rather than pipes. Nothing it or a hook prints can
/// then block on a full pipe, and a harrow that quits or dies does not take
/// the reading end away mid-write, which would stop cairn, or a hook, at its
/// next line of output. It runs in a process group of its own, so a Ctrl-C or
/// a hangup meant for harrow does not reach it, and so that at the limit all
/// of it — cairn and any hook it started — is stopped, not cairn alone.
pub fn run_write(
    program: &str,
    args: &[&str],
    limit: Duration,
) -> Result<(String, String), ExecError> {
    let mut out = Scratch::new().map_err(ExecError::Spawn)?;
    let mut err = Scratch::new().map_err(ExecError::Spawn)?;
    let mut command = Command::new(program);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(out.for_child().map_err(ExecError::Spawn)?)
        .stderr(err.for_child().map_err(ExecError::Spawn)?)
        .spawn()
        .map_err(ExecError::Spawn)?;
    // The child holds its own handles now; the names are no longer needed,
    // and a harrow that dies from here on leaves nothing behind.
    out.forget_name();
    err.forget_name();

    let deadline = Instant::now() + limit;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            // Not left running behind a slot that says it has ended.
            Err(e) => {
                stop(&mut child);
                return Err(ExecError::Spawn(e));
            }
        }
        if Instant::now() >= deadline {
            let whole = stop(&mut child);
            return Err(ExecError::Stopped {
                after: limit,
                whole,
            });
        }
        std::thread::sleep(POLL);
    };

    let out = out.read().map_err(ExecError::Spawn)?;
    let err = err.read().map_err(ExecError::Spawn)?;
    // A write that did not succeed failed, whatever it printed first: a bulk
    // change that stopped on its third item has said something about two.
    if !status.success() {
        return Err(ExecError::Failed {
            code: status.code(),
            stderr: err,
        });
    }
    Ok((out, err))
}

/// Stop a write and everything it started, and say whether that is what
/// happened. It leads its own process group, so the group is signalled; std
/// can only signal the process itself, which is the fallback.
fn stop(child: &mut std::process::Child) -> bool {
    #[cfg(unix)]
    let whole = {
        let group = libc::pid_t::try_from(child.id()).map_or(0, |pid| -pid);
        // SAFETY: kill(2) with a negated pid signals that process group and
        // touches no memory. The group is ours: its leader has not been
        // waited on, so its pid cannot have been reused.
        #[allow(unsafe_code)]
        let sent = group != 0 && unsafe { libc::kill(group, libc::SIGKILL) } == 0;
        sent
    };
    #[cfg(not(unix))]
    let whole = false;
    // Already gone, where the group went; the leader alone, where it did not.
    let _ = child.kill();
    let _ = child.wait();
    whole
}

/// Where a write's output goes: a file only this process can read, made
/// fresh, and named only until the child has it.
struct Scratch {
    file: std::fs::File,
    path: Option<std::path::PathBuf>,
}

/// More than anything worth showing in a footer; a hook that says more is
/// not read past this.
const SCRATCH_CAP: u64 = 1 << 20;

impl Scratch {
    fn new() -> std::io::Result<Scratch> {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dir = std::env::temp_dir();
        loop {
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos());
            let path = dir.join(format!("harrow-write-{}-{nanos}-{n}", std::process::id()));
            let mut open = std::fs::OpenOptions::new();
            // Made here or not at all: never a file somebody left, or a link
            // somebody planted, under a name that can be guessed.
            open.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                open.mode(0o600);
            }
            match open.open(&path) {
                Ok(file) => {
                    return Ok(Scratch {
                        file,
                        path: Some(path),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
        }
    }

    /// A handle of the child's own, appending: a hook left running in the
    /// background writes after the end, never over what was said, and this
    /// side reads from its own position.
    fn for_child(&self) -> std::io::Result<std::fs::File> {
        let path = self.path.as_ref().ok_or_else(|| {
            std::io::Error::other("the scratch file has already given up its name")
        })?;
        std::fs::OpenOptions::new().append(true).open(path)
    }

    /// Unix lets an open file lose its name; elsewhere it keeps it until it
    /// is dropped.
    fn forget_name(&mut self) {
        #[cfg(unix)]
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }

    fn read(&mut self) -> std::io::Result<String> {
        use std::io::{Read, Seek};
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        (&self.file).take(SCRATCH_CAP).read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Where it still has a name. In the temporary directory if this
        // fails, which the system clears.
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn run_with(
    (program, without): (&str, &[&str]),
    args: &[&str],
    timeout: Duration,
) -> Result<(String, String), ExecError> {
    let mut command = Command::new(program);
    for name in without {
        command.env_remove(name);
    }
    let mut child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ExecError::Spawn)?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (tx, rx) = mpsc::channel::<(String, String)>();
    std::thread::spawn(move || {
        let mut out = String::new();
        let mut err = String::new();
        if let Some(mut s) = stdout {
            let _ = s.read_to_string(&mut out);
        }
        if let Some(mut s) = stderr {
            let _ = s.read_to_string(&mut err);
        }
        let _ = tx.send((out, err));
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(ExecError::Spawn(e)),
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExecError::Timeout(timeout));
        }
        std::thread::sleep(POLL);
    };

    // The reader finishes as soon as both pipes close, which the exit above
    // guarantees; the timeout is belt and braces against a leaked descriptor.
    let (out, err) = rx
        .recv_timeout(Duration::from_millis(500))
        .unwrap_or_default();

    if out.trim().is_empty() && !status.success() {
        return Err(ExecError::Failed {
            code: status.code(),
            stderr: err,
        });
    }
    Ok((out, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_stdout() {
        let out = run("echo", &["hello"], Duration::from_secs(2)).expect("echo runs");
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn kills_a_command_that_overruns() {
        let started = Instant::now();
        let err = run("sleep", &["30"], Duration::from_millis(150)).unwrap_err();
        assert!(matches!(err, ExecError::Timeout(_)), "got {err:?}");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "kill was not prompt"
        );
    }

    #[test]
    fn reports_a_missing_binary() {
        let err = run("harrow-not-a-real-binary", &[], Duration::from_secs(1)).unwrap_err();
        assert!(matches!(err, ExecError::Spawn(_)), "got {err:?}");
    }

    #[test]
    fn tolerates_a_nonzero_exit_that_still_printed() {
        let out = run(
            "sh",
            &["-c", "echo partial; exit 1"],
            Duration::from_secs(2),
        )
        .expect("partial output is still useful");
        assert_eq!(out.trim(), "partial");
    }

    #[test]
    fn survives_more_output_than_a_pipe_buffer_holds() {
        let out = run(
            "sh",
            &[
                "-c",
                "for i in $(seq 1 20000); do echo aaaaaaaaaaaaaaaaaaaaaaaa; done",
            ],
            Duration::from_secs(20),
        )
        .expect("large output does not deadlock");
        assert_eq!(out.lines().count(), 20000);
    }

    /// A write's output is nobody else's to read.
    #[cfg(unix)]
    #[test]
    fn a_scratch_file_is_readable_by_its_owner_alone() {
        use std::os::unix::fs::PermissionsExt;
        let scratch = Scratch::new().unwrap();
        let mode = scratch.file.metadata().unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
