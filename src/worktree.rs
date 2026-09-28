//! The same backlog, checked out more than once.
//!
//! An agent works on a branch in a worktree of its own, and `cairn claim`
//! there writes the claim into that worktree's copy of the item. That is
//! right for cairn, whose claims coordinate the writers of one directory, and
//! it cannot be seen from any other: nothing reaches this checkout until the
//! branch merges. So a board with four agents at work looked like a board
//! nobody was touching.
//!
//! Immutable ids make the question answerable, because an id names the same
//! item in every checkout. So this asks git which item files each other
//! worktree has changed since it diverged from this one, and reads those.
//! What was read is attached beside the record by `engine::sight`, never in
//! place of it.
//!
//! And which it has *added*, committed or not. An agent files the item it is
//! about to do and starts on it a second later, so an item new on a branch is
//! usually the most visible thing an agent is doing. Those are handed back
//! as items of their own, for `engine::filed` to show as provisional rows.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::diag;
use crate::exec;
use crate::item::Item;
use crate::schema::Schema;

/// Longer than git takes on any worktree it can read at all, and short enough
/// that one wedged on a dead mount does not hold up the reading for long.
const GIT: Duration = Duration::from_secs(2);

/// Every other worktree of the repository a project lives in.
#[derive(Default)]
pub struct Survey {
    /// Where git registers worktrees. Watched so a new one is noticed when it
    /// is made, rather than on the poll after its first write.
    pub registry: Option<PathBuf>,
    pub others: Vec<Other>,
}

pub struct Other {
    pub branch: String,
    /// Its copy of the items directory.
    pub items: PathBuf,
    /// Its copies of the items it has changed since it diverged from here.
    pub copies: Vec<Item>,
    /// Items it has filed that this checkout does not have.
    pub filed: Vec<Item>,
}

/// One entry of `git worktree list --porcelain`.
#[derive(Debug, PartialEq)]
struct Checkout {
    path: PathBuf,
    head: String,
    /// The branch, or for a detached worktree the commit, as `git worktree
    /// list` itself names it.
    name: String,
}

/// Look, and give up quietly where there is nothing to look at.
///
/// A project outside git, a repository with no commits and a machine with no
/// git are all ordinary, and each simply has no other worktrees.
pub fn survey(schema: &Schema) -> Survey {
    // Canonical, because git reports canonical paths and one has to be
    // found inside another. `/tmp` is `/private/tmp` on macOS.
    let Ok(root) = schema.root.canonicalize() else {
        return Survey::default();
    };
    // One process for all of it: the listing names every checkout's commit,
    // this one's included, so nothing else has to be asked before the diffs.
    let Ok(listing) = exec::git(
        &[
            "-C",
            &root.to_string_lossy(),
            "worktree",
            "list",
            "--porcelain",
        ],
        GIT,
    ) else {
        return Survey::default();
    };
    let all = checkouts(&listing);
    // The innermost, because a worktree may be kept inside another.
    let Some(here) = all
        .iter()
        .filter(|c| root.starts_with(&c.path))
        .max_by_key(|c| c.path.components().count())
    else {
        return Survey::default();
    };
    let Ok(within) = root.strip_prefix(&here.path) else {
        return Survey::default();
    };
    let items = within.join(&schema.dir);

    // In parallel: each is a git process that mostly waits on the disk, and a
    // reading waits on the slowest of them rather than on their sum. Only the
    // asking — the schema is not shared across threads, and the reading is a
    // few files.
    let format = schema.format;
    let changed: Vec<(&Checkout, Changed)> = std::thread::scope(|scope| {
        let handles: Vec<_> = all
            .iter()
            .filter(|c| c.path != here.path && c.path.is_dir())
            .map(|there| {
                let items = &items;
                scope.spawn(move || {
                    let files = changed(format, &here.head, there, within, items)?;
                    Some((there, files))
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| h.join().ok().flatten())
            .collect()
    });

    let read = |there: &Checkout, files: &[PathBuf]| -> Vec<Item> {
        files
            .iter()
            .filter_map(|file| {
                let text = std::fs::read_to_string(file).ok()?;
                crate::item::parse_for_schema(&text, file, schema)
                    .map_err(|e| diag::info("worktree", format!("{}: {e}", there.name)))
                    .ok()
            })
            .collect()
    };
    let others = changed
        .into_iter()
        .map(|(there, files)| Other {
            branch: there.name.clone(),
            items: there.path.join(&items),
            copies: read(there, &files.modified),
            filed: read(there, &files.added),
        })
        .collect();

    Survey {
        registry: registry(&here.path),
        others,
    }
}

/// Where git keeps its list of worktrees, read from the checkout rather than
/// asked for. `.git` is the repository in the main worktree, and in any other
/// a file naming `<common>/worktrees/<name>`.
fn registry(top: &Path) -> Option<PathBuf> {
    let dot = top.join(".git");
    if dot.is_dir() {
        return Some(dot.join("worktrees"));
    }
    let text = std::fs::read_to_string(&dot).ok()?;
    let gitdir = PathBuf::from(text.strip_prefix("gitdir:")?.trim());
    let gitdir = if gitdir.is_relative() {
        top.join(gitdir)
    } else {
        gitdir
    };
    gitdir.parent().map(Path::to_path_buf)
}

/// Every checkout in `git worktree list --porcelain` that has files to read.
fn checkouts(porcelain: &str) -> Vec<Checkout> {
    let mut out = Vec::new();
    for block in porcelain.split("\n\n") {
        let (mut path, mut head, mut branch) = (None, None, None);
        let mut readable = true;
        for line in block.lines() {
            let (key, value) = line.split_once(' ').unwrap_or((line, ""));
            match key {
                "worktree" => path = Some(PathBuf::from(value)),
                "HEAD" => head = Some(value.to_string()),
                "branch" => {
                    branch = Some(
                        value
                            .strip_prefix("refs/heads/")
                            .unwrap_or(value)
                            .to_string(),
                    )
                }
                "bare" | "prunable" => readable = false,
                _ => {}
            }
        }
        // A worktree with no HEAD has no commit yet, and so nothing it could
        // have diverged from.
        if let (Some(path), Some(head), true) = (path, head, readable) {
            let name = branch.unwrap_or_else(|| head.chars().take(8).collect());
            out.push(Checkout {
                path: path.canonicalize().unwrap_or(path),
                head,
                name,
            });
        }
    }
    out
}

/// What one worktree has done to the item files since it diverged.
#[derive(Default)]
struct Changed {
    modified: Vec<PathBuf>,
    added: Vec<PathBuf>,
}

/// The item files one other worktree has changed since it diverged from
/// this one, or `None` where its ids cannot be read as ours.
fn changed(
    format: u32,
    head: &str,
    there: &Checkout,
    within: &Path,
    items: &Path,
) -> Option<Changed> {
    // A worktree on another format spells ids another way, so an id there
    // does not name the item it would name here.
    let theirs = Schema::load(&there.path.join(within).join("cairn.toml")).ok()?;
    if theirs.format != format {
        diag::info(
            "worktree",
            format!(
                "{}: format {}, not {format}; skipped",
                there.name, theirs.format
            ),
        );
        return None;
    }

    // Against the merge-base, not against our copy: a branch cut before this
    // one moved holds old copies of items it never touched, and those are
    // not work. A branch already merged here has its tip as the merge-base,
    // so all it can show is what is still uncommitted in it.
    //
    // Modified or renamed are copies: a file that existed at the
    // merge-base is an item both checkouts have, so its id names the same
    // item in both — even in a format whose ids are counted, where two
    // branches can each file a different item as 7. Added is the other kind
    // of thing, an item only that branch has, and is kept apart.
    let dir = there.path.to_string_lossy().into_owned();
    let git = |args: &[&str]| -> Option<Vec<PathBuf>> {
        let args: Vec<&str> = ["-C", dir.as_str()]
            .into_iter()
            .chain(args.iter().copied())
            .collect();
        let out = exec::git(&args, GIT)
            .map_err(|e| diag::warn("worktree", format!("{}: {e}", there.name)))
            .ok()?;
        Some(
            out.split('\0')
                .filter(|name| !name.is_empty())
                .map(|name| there.path.join(name))
                .filter(|file| crate::engine::is_item_file(file) && file.is_file())
                .collect(),
        )
    };
    let items = items.to_string_lossy();
    let diff = |filter: &str| {
        git(&[
            "diff",
            "--name-only",
            "-z",
            filter,
            "--merge-base",
            head,
            "--",
            &items,
        ])
    };
    let modified = diff("--diff-filter=MR")?;
    let mut added = diff("--diff-filter=A")?;
    // Not yet committed, which is how `cairn new` leaves an item: the diff
    // cannot see an untracked file at all.
    added.extend(git(&[
        "ls-files",
        "--others",
        "--exclude-standard",
        "-z",
        "--",
        &items,
    ])?);
    Some(Changed { modified, added })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_worktree_listing_names_each_checkout_by_its_branch() {
        let listing = "worktree /r/main\nHEAD 0123456789abcdef\nbranch refs/heads/main\n\n\
                       worktree /r/agent\nHEAD fedcba9876543210\nbranch refs/heads/perf/96d2-chunks\n\n\
                       worktree /r/loose\nHEAD aaaaaaaabbbbbbbb\ndetached\n\n\
                       worktree /r/bare.git\nbare\n\n\
                       worktree /r/gone\nHEAD cccccccc\nbranch refs/heads/old\nprunable gitdir file points to non-existent location\n";
        let names: Vec<(String, String)> = checkouts(listing)
            .into_iter()
            .map(|c| (c.path.display().to_string(), c.name))
            .collect();
        assert_eq!(
            names,
            vec![
                ("/r/main".into(), "main".into()),
                ("/r/agent".into(), "perf/96d2-chunks".into()),
                ("/r/loose".into(), "aaaaaaaa".into()),
            ]
        );
    }
}
