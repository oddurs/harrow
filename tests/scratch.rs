//! In a binary of its own: it counts files in the temporary directory by this
//! process's id, and any other write running beside it would be counted too.

use std::time::Duration;

use harrow::app::Change;
use harrow::writer::{LIMIT, Writing};

/// Nothing a write leaves in the temporary directory is readable by name
/// while it runs, and nothing is left once it ends.
#[cfg(unix)]
#[test]
fn a_write_leaves_no_files_behind() {
    let ours = format!("harrow-write-{}-", std::process::id());
    let named = || {
        std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with(&ours))
            .count()
    };
    let write = Writing::start(
        "sh",
        Change {
            args: vec!["-c".into(), "sleep 0.5; echo hi".into()],
            describe: "slow".into(),
            undo: None,
        },
        LIMIT,
    );
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(named(), 0, "named while it runs");
    assert_eq!(write.wait().outcome.unwrap().0.trim(), "hi");
    assert_eq!(named(), 0, "left behind");
}
