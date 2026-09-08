//! Shared test helpers: snapshots, and a deterministic app to take them of.

use std::path::PathBuf;

use harrow::app::App;
use harrow::testkit;

/// An app holding the sample backlog, with everything that moves on its own
/// pinned. A snapshot of a clock is a snapshot that fails tomorrow.
pub fn app() -> App {
    let mut app = testkit::app();
    app.loading = false;
    app.last_load = None;
    app.now = 0;
    app
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.txt"))
}

/// Compare against the recorded screen, or record it.
///
/// `HARROW_UPDATE_SNAPSHOTS=1 cargo test` accepts a deliberate change. Without
/// it, a changed screen fails — which is the point: the interface is the
/// product, and it should not be able to change by accident.
pub fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    let update = std::env::var_os("HARROW_UPDATE_SNAPSHOTS").is_some();

    if update || !path.exists() {
        std::fs::create_dir_all(path.parent().expect("snapshots directory")).expect("create");
        std::fs::write(&path, actual).expect("write the snapshot");
        if !update {
            panic!(
                "recorded a new snapshot at {}; check it reads correctly and run again",
                path.display()
            );
        }
        return;
    }

    let expected = std::fs::read_to_string(&path).expect("read the snapshot");
    if expected.trim_end() != actual.trim_end() {
        panic!(
            "{name} does not match {}\n\
             run HARROW_UPDATE_SNAPSHOTS=1 cargo test to accept it\n\n\
             --- recorded ---\n{expected}\n--- now ---\n{actual}",
            path.display()
        );
    }
}
