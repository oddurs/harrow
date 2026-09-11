//! `harrow --doctor`: check everything harrow depends on, and say which of them
//! is broken before the user has to guess from an empty screen.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::engine::{Project, Source};
use crate::exec;
use crate::theme::Theme;

pub struct Check {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
    /// False when a failure only costs a feature, not the whole tool.
    pub fatal: bool,
}

pub fn run(config: &Config, config_path: Option<&Path>, theme: &Theme, start: &Path) -> Vec<Check> {
    let mut checks = Vec::new();

    let project = match Project::discover(start) {
        Ok(p) => Some(p),
        Err(e) => {
            checks.push(Check {
                name: "project",
                ok: false,
                detail: e.to_string(),
                fatal: true,
            });
            None
        }
    };

    let mut items = None;
    if let Some(mut project) = project {
        let path = project.config_path();
        let started = Instant::now();
        match project.load() {
            Ok(report) => {
                items = Some(report.items.len());
                checks.push(Check {
                    name: "project",
                    ok: true,
                    detail: format!(
                        "{} — {} items from {} in {}ms",
                        report.schema.name,
                        report.items.len(),
                        report.schema.items_dir().display(),
                        started.elapsed().as_millis()
                    ),
                    fatal: false,
                });
                // Whether the operating system will tell us about a change, or
                // whether this project is one the poll has to carry.
                let dir = report.schema.items_dir();
                checks.push(match crate::runtime::can_watch(&dir) {
                    Ok(()) => Check {
                        name: "watch",
                        ok: true,
                        detail: format!("{} — changes arrive at once", dir.display()),
                        fatal: false,
                    },
                    Err(e) => Check {
                        name: "watch",
                        ok: false,
                        detail: format!("{e}; falling back to the {}s poll", config.refresh_secs),
                        fatal: false,
                    },
                });

                let problems = report.schema.problems();
                checks.push(Check {
                    name: "schema",
                    ok: problems.is_empty(),
                    detail: if problems.is_empty() {
                        format!(
                            "format {}, {} types, {} statuses, {} fields",
                            report.schema.format,
                            report.schema.types.len(),
                            report.schema.statuses.len(),
                            report.schema.fields.len()
                        )
                    } else {
                        problems.join("; ")
                    },
                    fatal: false,
                });
                if !report.warnings.is_empty() {
                    checks.push(Check {
                        name: "items",
                        ok: false,
                        detail: report.warnings.join("; "),
                        fatal: false,
                    });
                }
            }
            Err(e) => checks.push(Check {
                name: "project",
                ok: false,
                detail: format!("{}: {e}", path.display()),
                fatal: true,
            }),
        }
    }

    // The write path, and a second opinion on the read path. harrow parses the
    // item files itself; if cairn is here, it is worth asking whether the two
    // agree, because a disagreement is a bug in this program.
    let version = exec::run(&config.cairn, &["--version"], Duration::from_secs(5));
    match version {
        Ok(out) => {
            let line = out.lines().next().unwrap_or("cairn").trim().to_string();
            let mut detail = line;
            if let Some(mine) = items
                && let Ok(count) = exec::run(
                    &config.cairn,
                    &["-C", &start.display().to_string(), "list", "-A", "--count"],
                    Duration::from_secs(10),
                )
                && let Ok(theirs) = count.trim().parse::<usize>()
            {
                if theirs == mine {
                    detail.push_str(&format!(" — agrees on {mine} items"));
                } else {
                    detail.push_str(&format!(
                        " — cairn counts {theirs} items where harrow reads {mine}"
                    ));
                }
            }
            let agrees = !detail.contains("where harrow reads");
            checks.push(Check {
                name: "cairn",
                ok: agrees,
                detail,
                fatal: false,
            });
        }
        Err(e) => checks.push(Check {
            name: "cairn",
            ok: false,
            // Not fatal: reading a backlog needs nothing but the files.
            detail: format!("{e} — harrow can read this backlog but not change it"),
            fatal: false,
        }),
    }

    checks.push(Check {
        name: "config",
        ok: true,
        detail: match config_path {
            Some(p) => {
                let set = config.overridden();
                if set.is_empty() {
                    format!("{} (nothing overridden)", p.display())
                } else {
                    format!("{} — {}", p.display(), set.join(", "))
                }
            }
            None => "no config file; using defaults".to_string(),
        },
        fatal: false,
    });

    checks.push(Check {
        name: "theme",
        ok: true,
        detail: format!("{} ({})", theme.name, theme.source.label()),
        fatal: false,
    });

    checks.push(Check {
        name: "editor",
        ok: true,
        detail: config.editor(),
        fatal: false,
    });
    checks.push(tool_check("clipboard", clipboard_tool()));

    checks.push(Check {
        name: "terminal",
        ok: true,
        detail: match crossterm::terminal::size() {
            Ok((w, h)) => {
                let cramped = w < 60 || h < 12;
                format!(
                    "{w}×{h}{}",
                    if cramped {
                        " (cramped; 80×24 or more is better)"
                    } else {
                        ""
                    }
                )
            }
            Err(e) => format!("size unknown: {e}"),
        },
        fatal: false,
    });

    checks.push(Check {
        name: "log",
        ok: true,
        detail: match std::env::var("HARROW_LOG") {
            Ok(p) if !p.is_empty() => format!("writing to {p}"),
            _ => "off (set HARROW_LOG=/path/to/file)".to_string(),
        },
        fatal: false,
    });

    checks
}

fn tool_check(name: &'static str, found: Option<&'static str>) -> Check {
    match found {
        Some(tool) => Check {
            name,
            ok: true,
            detail: format!("using {tool}"),
            fatal: false,
        },
        None => Check {
            name,
            ok: false,
            detail: "no supported helper found".to_string(),
            fatal: false,
        },
    }
}

fn clipboard_tool() -> Option<&'static str> {
    ["pbcopy", "wl-copy", "xclip"].into_iter().find(on_path)
}

fn on_path(tool: &&'static str) -> bool {
    exec::run(
        "sh",
        &["-c", &format!("command -v {tool}")],
        Duration::from_secs(2),
    )
    .map(|o| !o.trim().is_empty())
    .unwrap_or(false)
}

/// Exit code: 0 if nothing fatal is broken.
pub fn report(checks: &[Check]) -> i32 {
    let mut fatal = 0;
    for c in checks {
        let mark = if c.ok {
            "ok  "
        } else if c.fatal {
            "FAIL"
        } else {
            "warn"
        };
        println!("{mark}  {:<9} {}", c.name, c.detail);
        if !c.ok && c.fatal {
            fatal += 1;
        }
    }
    println!();
    if fatal == 0 {
        println!("harrow can read this backlog.");
        0
    } else {
        println!("{fatal} fatal problem(s) — harrow cannot read this backlog.");
        1
    }
}
