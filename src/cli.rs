//! The command line, described once.
//!
//! The usage text, the check that refuses an unknown option, and the shell
//! completions were three lists of the same flags, which is three chances for
//! them to disagree — and the way they disagree is that a flag works and
//! nothing tells you it exists. They are one list now, and everything that
//! needs to know about the command line reads it.

/// What a flag expects after it, which is what a completion has to know.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Takes {
    /// Nothing. `--all`.
    Nothing,
    /// A value with no useful set to offer: a filter expression, a size.
    Value(&'static str),
    /// A directory.
    Directory,
    /// A file.
    File,
    /// One of these, exactly.
    OneOf(&'static str, &'static [&'static str]),
    /// A theme, which the running harrow can list.
    Theme,
}

pub struct Flag {
    pub short: Option<char>,
    pub long: &'static str,
    pub takes: Takes,
    pub help: &'static str,
}

impl Flag {
    /// How the flag reads in the usage table.
    fn spelled(&self) -> String {
        let name = match self.takes {
            Takes::Nothing => self.long.to_string(),
            Takes::Directory => format!("{} <DIR>", self.long),
            Takes::File => format!("{} <PATH>", self.long),
            Takes::Theme => format!("{} <NAME>", self.long),
            Takes::Value(label) | Takes::OneOf(label, _) => format!("{} {label}", self.long),
        };
        match self.short {
            Some(c) => format!("-{c}, {name}"),
            None => format!("    {name}"),
        }
    }

    pub fn wants_value(&self) -> bool {
        self.takes != Takes::Nothing
    }
}

pub struct Subcommand {
    pub name: &'static str,
    pub usage: &'static str,
    pub help: &'static str,
}

pub const SUBCOMMANDS: &[Subcommand] = &[
    Subcommand {
        name: "config",
        usage: "harrow config [--write] [--force]",
        help: "show the configuration in effect, or write a commented default",
    },
    Subcommand {
        name: "themes",
        usage: "harrow themes [filter]",
        help: "list every theme harrow can find",
    },
    Subcommand {
        name: "completions",
        usage: "harrow completions <shell>",
        help: "print a completion script for fish, bash or zsh",
    },
    Subcommand {
        name: "man",
        usage: "harrow man",
        help: "print the man page",
    },
];

pub const SHELLS: &[&str] = &["fish", "bash", "zsh"];

pub const FLAGS: &[Flag] = &[
    Flag {
        short: Some('C'),
        long: "--directory",
        takes: Takes::Directory,
        help: "start looking for the project here",
    },
    Flag {
        short: Some('a'),
        long: "--all",
        takes: Takes::Nothing,
        help: "show everything: finished, dropped, milestones",
    },
    Flag {
        short: None,
        long: "--lens",
        takes: Takes::Value("<NAME>"),
        help: "needs, list, board, stats, or log",
    },
    Flag {
        short: Some('b'),
        long: "--board",
        takes: Takes::Nothing,
        help: "open on the board (--lens board)",
    },
    Flag {
        short: None,
        long: "--stats",
        takes: Takes::Nothing,
        help: "open on the statistics (--lens stats)",
    },
    Flag {
        short: Some('f'),
        long: "--filter",
        takes: Takes::Value("<EXPR>"),
        help: "open filtered, in cairn's grammar",
    },
    Flag {
        short: None,
        long: "--view",
        takes: Takes::Value("<NAME>"),
        help: "open in one of the project's saved views",
    },
    Flag {
        short: None,
        long: "--group-by",
        takes: Takes::Value("<FIELD>"),
        help: "milestone, status, type, or any field",
    },
    Flag {
        short: None,
        long: "--sort",
        takes: Takes::Value("<KEYS>"),
        help: "sort keys, `-` for descending",
    },
    Flag {
        short: Some('p'),
        long: "--plain",
        takes: Takes::Nothing,
        help: "print one line per item and exit",
    },
    Flag {
        short: None,
        long: "--theme",
        takes: Takes::Theme,
        help: "use a theme for this run (auto, mono, gotham, …)",
    },
    Flag {
        short: None,
        long: "--config",
        takes: Takes::File,
        help: "read this config file instead of the usual one",
    },
    Flag {
        short: None,
        long: "--color",
        takes: Takes::OneOf("<WHEN>", &["auto", "always", "never"]),
        help: "always, never, or auto",
    },
    Flag {
        short: None,
        long: "--no-color",
        takes: Takes::Nothing,
        help: "same as --color never",
    },
    Flag {
        short: None,
        long: "--doctor",
        takes: Takes::Nothing,
        help: "check everything harrow depends on",
    },
    Flag {
        short: None,
        long: "--screenshot",
        takes: Takes::Value("WxH"),
        help: "render one frame as text and exit",
    },
    Flag {
        short: None,
        long: "--fix-terminal",
        takes: Takes::Nothing,
        help: "undo a terminal left in mouse-reporting mode",
    },
    Flag {
        short: Some('h'),
        long: "--help",
        takes: Takes::Nothing,
        help: "show this help",
    },
    Flag {
        short: Some('V'),
        long: "--version",
        takes: Takes::Nothing,
        help: "show the version",
    },
];

/// The flag a token names, long or short.
pub fn flag(token: &str) -> Option<&'static Flag> {
    let token = token.split('=').next().unwrap_or(token);
    FLAGS.iter().find(|f| {
        f.long == token
            || f.short
                .is_some_and(|c| token.len() == 2 && token.starts_with('-') && token.ends_with(c))
    })
}

pub fn usage() -> String {
    let mut out = String::from("harrow — work a cairn backlog from the terminal\n\nUSAGE:\n");
    out.push_str("  harrow [options]\n");
    for command in SUBCOMMANDS {
        out.push_str(&format!("  {}\n", command.usage));
    }

    let width = FLAGS
        .iter()
        .map(|f| f.spelled().chars().count())
        .max()
        .unwrap_or(24);
    out.push_str("\nOPTIONS:\n");
    for f in FLAGS {
        out.push_str(&format!("  {:width$}  {}\n", f.spelled(), f.help));
    }
    out.push_str(
        "\nENVIRONMENT:\n  \
         HARROW_CONFIG  config file to read\n  \
         HARROW_LOG     append diagnostics to this file\n  \
         NO_COLOR       render without colour\n",
    );
    out
}

/// A completion script for one shell.
///
/// Generated from the table above rather than kept beside it, so a flag added
/// without a completion is not a thing that can happen.
pub fn completions(shell: &str) -> Option<String> {
    match shell {
        "fish" => Some(fish()),
        "bash" => Some(bash()),
        "zsh" => Some(zsh()),
        _ => None,
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn fish() -> String {
    let mut out = String::from(
        "# fish completions for harrow. Generated by `harrow completions fish`.\n\
         #\n\
         # Install:  harrow completions fish > ~/.config/fish/completions/harrow.fish\n\n\
         # No files unless a flag asks for one: harrow takes no positional\n\
         # arguments, so offering the working directory is noise.\n\
         complete -c harrow -f\n\n",
    );

    for command in SUBCOMMANDS {
        out.push_str(&format!(
            "complete -c harrow -n __fish_use_subcommand -a {} -d '{}'\n",
            command.name,
            escape(command.help)
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "complete -c harrow -n '__fish_seen_subcommand_from completions' -a '{}' -d 'shell'\n",
        SHELLS.join(" ")
    ));
    out.push_str(
        "complete -c harrow -n '__fish_seen_subcommand_from config' -l write -d 'write a commented default'\n\
         complete -c harrow -n '__fish_seen_subcommand_from config' -l force -d 'replace an existing file'\n\n",
    );

    for f in FLAGS {
        let mut line = String::from("complete -c harrow");
        // The subcommands take none of these.
        line.push_str(" -n 'not __fish_seen_subcommand_from config themes completions'");
        if let Some(c) = f.short {
            line.push_str(&format!(" -s {c}"));
        }
        line.push_str(&format!(" -l {}", f.long.trim_start_matches('-')));
        // `-x` is `-r -f`: takes a value, and no filenames behind it. `-r`
        // alone lets fish fall through to the directory listing, so `--color`
        // offered `auto`, `always`, `never` and then every file in sight.
        match f.takes {
            Takes::Nothing => {}
            Takes::Directory => line.push_str(" -x -a '(__fish_complete_directories)'"),
            Takes::File => line.push_str(" -r -F"),
            Takes::Value(_) => line.push_str(" -x"),
            Takes::OneOf(_, values) => {
                line.push_str(&format!(" -x -a '{}'", values.join(" ")));
            }
            // Asking the harrow on PATH, so the list is the themes this machine
            // actually has rather than the ones that existed when this was
            // generated.
            Takes::Theme => {
                line.push_str(
                    " -x -a '(harrow themes 2>/dev/null | string trim | string split -f1 \" \")'",
                );
            }
        }
        line.push_str(&format!(" -d '{}'\n", escape(f.help)));
        out.push_str(&line);
    }
    out
}

fn bash() -> String {
    let longs: Vec<&str> = FLAGS.iter().map(|f| f.long).collect();
    let shorts: Vec<String> = FLAGS
        .iter()
        .filter_map(|f| f.short.map(|c| format!("-{c}")))
        .collect();
    let valued: Vec<&str> = FLAGS
        .iter()
        .filter(|f| f.wants_value())
        .map(|f| f.long)
        .collect();
    let commands: Vec<&str> = SUBCOMMANDS.iter().map(|c| c.name).collect();

    format!(
        "# bash completions for harrow. Generated by `harrow completions bash`.\n\
         #\n\
         # Install:  harrow completions bash > /usr/local/etc/bash_completion.d/harrow\n\
         #      or:  source <(harrow completions bash)\n\n\
         _harrow() {{\n\
         \x20 local cur prev\n\
         \x20 cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\
         \x20 prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n\n\
         \x20 case \"$prev\" in\n\
         \x20   --directory|-C) COMPREPLY=( $(compgen -d -- \"$cur\") ); return ;;\n\
         \x20   --config) COMPREPLY=( $(compgen -f -- \"$cur\") ); return ;;\n\
         \x20   --color) COMPREPLY=( $(compgen -W 'auto always never' -- \"$cur\") ); return ;;\n\
         \x20   --theme) COMPREPLY=( $(compgen -W \"$(harrow themes 2>/dev/null | awk '{{print $1}}')\" -- \"$cur\") ); return ;;\n\
         \x20 esac\n\n\
         \x20 # Everything else that takes a value takes one we cannot guess.\n\
         \x20 for flag in {valued}; do\n\
         \x20   [ \"$prev\" = \"$flag\" ] && return\n\
         \x20 done\n\n\
         \x20 if [ \"$COMP_CWORD\" -eq 1 ]; then\n\
         \x20   COMPREPLY=( $(compgen -W '{commands} {longs} {shorts}' -- \"$cur\") )\n\
         \x20 else\n\
         \x20   COMPREPLY=( $(compgen -W '{longs} {shorts}' -- \"$cur\") )\n\
         \x20 fi\n\
         }}\n\
         complete -F _harrow harrow\n",
        valued = valued.join(" "),
        commands = commands.join(" "),
        longs = longs.join(" "),
        shorts = shorts.join(" "),
    )
}

fn zsh() -> String {
    let mut specs = String::new();
    for f in FLAGS {
        let action = match f.takes {
            Takes::Nothing => String::new(),
            Takes::Directory => ":directory:_files -/".into(),
            Takes::File => ":file:_files".into(),
            Takes::Theme => ":theme:->theme".into(),
            Takes::OneOf(_, values) => format!(":when:({})", values.join(" ")),
            Takes::Value(label) => format!(":{}:", label.trim_matches(['<', '>'])),
        };
        let help = f.help.replace('\'', "'\\''").replace(['[', ']'], "");
        match f.short {
            Some(c) => specs.push_str(&format!(
                "    '(-{c} {long})'{{-{c},{long}}}'[{help}]{action}' \\\n",
                long = f.long
            )),
            None => specs.push_str(&format!("    '{}[{help}]{action}' \\\n", f.long)),
        }
    }
    let commands: Vec<String> = SUBCOMMANDS
        .iter()
        .map(|c| format!("{}:{}", c.name, c.help.replace(':', " ")))
        .collect();

    format!(
        "#compdef harrow\n\
         # zsh completions for harrow. Generated by `harrow completions zsh`.\n\
         #\n\
         # Install:  harrow completions zsh > \"${{fpath[1]}}/_harrow\"\n\n\
         _harrow() {{\n\
         \x20 local state\n\
         \x20 _arguments -s \\\n\
         {specs}    '1:command:(({commands}))'\n\n\
         \x20 case \"$state\" in\n\
         \x20   theme) compadd $(harrow themes 2>/dev/null | awk '{{print $1}}') ;;\n\
         \x20 esac\n\
         }}\n\
         _harrow \"$@\"\n",
        commands = commands.join(" "),
    )
}

/// The man page, from the same table as everything else.
///
/// Written by hand it would be a fourth list of the flags, and the one nobody
/// looks at when adding one.
pub fn man(version: &str) -> String {
    let escape = |s: &str| s.replace('\\', "\\\\").replace('-', "\\-");
    let mut out = format!(
        ".\\\" Generated by `harrow man`. Do not edit.\n\
         .TH HARROW 1 \"\" \"harrow {version}\" \"User Commands\"\n\
         .SH NAME\n\
         harrow \\- work a cairn backlog from the terminal\n\
         .SH SYNOPSIS\n\
         .B harrow\n\
         .RI [ options ]\n\
         .br\n"
    );
    for command in SUBCOMMANDS {
        out.push_str(&format!(".B harrow {}\n.br\n", escape(command.name)));
    }
    // Every line at column zero: roff reads a leading space as text, so an
    // indented directive is printed rather than obeyed. A continuation in the
    // Rust source carries its indentation into the string, which is how the
    // whole of DESCRIPTION once came out as literal `.B harrow`.
    for line in [
        ".SH DESCRIPTION",
        ".B harrow",
        "reads a cairn backlog \\(em the Markdown items under",
        ".I cairn.toml",
        "\\(em and lets you move through it, read an item in full, and change one.",
        "It is built for a pane beside the work: a status strip says what is",
        "happening before a row is read, and a change made in another window",
        "arrives at once.",
        ".PP",
        "Reads go straight to the files, so a backlog opens with",
        ".BR cairn (1)",
        "not installed. Every change is a",
        ".B cairn",
        "invocation: it owns the locking, the identifiers and the hooks.",
        ".SH OPTIONS",
    ] {
        out.push_str(line);
        out.push('\n');
    }
    for f in FLAGS {
        let names = match f.short {
            Some(c) => format!("\\fB\\-{c}\\fR, \\fB{}\\fR", escape(f.long)),
            None => format!("\\fB{}\\fR", escape(f.long)),
        };
        let value = match f.takes {
            Takes::Nothing => String::new(),
            Takes::Directory => " \\fIDIR\\fR".into(),
            Takes::File => " \\fIPATH\\fR".into(),
            Takes::Theme => " \\fINAME\\fR".into(),
            Takes::Value(label) | Takes::OneOf(label, _) => {
                format!(" \\fI{}\\fR", escape(label.trim_matches(['<', '>'])))
            }
        };
        out.push_str(&format!(".TP\n{names}{value}\n{}\n", escape(f.help)));
    }

    out.push_str(".SH COMMANDS\n");
    for command in SUBCOMMANDS {
        out.push_str(&format!(
            ".TP\n\\fB{}\\fR\n{}\n",
            escape(command.name),
            escape(command.help)
        ));
    }

    for line in [
        ".SH ENVIRONMENT",
        ".TP",
        "\\fBHARROW_CONFIG\\fR",
        "Config file to read.",
        ".TP",
        "\\fBHARROW_LOG\\fR",
        "Append diagnostics to this file.",
        ".TP",
        "\\fBNO_COLOR\\fR",
        "Render without colour.",
        ".SH FILES",
        ".TP",
        "\\fI~/.config/harrow/config.toml\\fR",
        "Your configuration.",
        ".TP",
        "\\fI~/.config/harrow/themes/\\fR",
        "Your themes.",
        ".TP",
        "\\fIcairn.toml\\fR",
        "The project\\(aqs schema, which harrow never writes to.",
        ".SH SEE ALSO",
        ".BR cairn (1)",
        ".SH AUTHOR",
        "Oddur Sigurdsson.",
    ] {
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_flag_is_in_the_usage_and_in_every_completion() {
        let usage = usage();
        let scripts: Vec<String> = SHELLS
            .iter()
            .map(|s| completions(s).expect("a script for every shell we claim"))
            .collect();

        for f in FLAGS {
            assert!(usage.contains(f.long), "{} is not in the usage", f.long);
            for (shell, script) in SHELLS.iter().zip(&scripts) {
                let named =
                    script.contains(f.long) || script.contains(f.long.trim_start_matches('-'));
                assert!(named, "{} is not in the {shell} completions", f.long);
            }
        }
    }

    #[test]
    fn every_subcommand_is_offered() {
        for command in SUBCOMMANDS {
            assert!(usage().contains(command.name));
            for shell in SHELLS {
                let script = completions(shell).expect("a script");
                assert!(
                    script.contains(command.name),
                    "{} is not in the {shell} completions",
                    command.name
                );
            }
        }
    }

    /// The fourth list this table exists to prevent.
    #[test]
    fn every_flag_is_in_the_man_page() {
        let page = man("9.9.9");
        for f in FLAGS {
            // roff escapes the hyphens, so compare on the escaped spelling.
            let escaped = f.long.replace('-', "\\-");
            assert!(page.contains(&escaped), "{} is not in the man page", f.long);
        }
        for command in SUBCOMMANDS {
            assert!(page.contains(command.name), "{} is missing", command.name);
        }
        assert!(page.contains("harrow 9.9.9"), "the version has to be in it");
    }

    /// Rendered, not just written: an unbalanced escape is a page that looks
    /// fine in a diff and wrong on screen.
    #[test]
    fn the_man_page_renders() {
        let page = man("9.9.9");
        let file = std::env::temp_dir().join(format!("harrow-{}.1", std::process::id()));
        std::fs::write(&file, &page).expect("write");
        let rendered = std::process::Command::new("mandoc")
            .args(["-T", "ascii", "-W", "warning"])
            .arg(&file)
            .output()
            .or_else(|_| {
                std::process::Command::new("groff")
                    .args(["-man", "-T", "ascii"])
                    .arg(&file)
                    .output()
            });
        let _ = std::fs::remove_file(&file);

        let Ok(out) = rendered else {
            return; // Neither is installed; nothing to check with.
        };
        // Renderers bold by overstriking — `N\x08N` for a bold N — so the
        // text has to be flattened before anything can be found in it.
        let raw = String::from_utf8_lossy(&out.stdout);
        let mut text = String::with_capacity(raw.len());
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if chars.peek() == Some(&'\u{8}') {
                chars.next();
                continue;
            }
            text.push(c);
        }

        assert!(
            text.contains("work a cairn backlog"),
            "the page did not render: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            text.contains("--doctor") || text.contains("-doctor"),
            "{text}"
        );
        assert!(
            text.contains("pane beside the work"),
            "DESCRIPTION did not render: {text}"
        );
        // An indented directive is printed rather than obeyed and the page
        // still renders, so the only way to catch it is to look for the
        // directive in the output.
        for literal in [".B ", ".PP", ".TP", ".SH "] {
            assert!(
                !text.contains(literal),
                "roff printed {literal:?} instead of obeying it:\n{text}"
            );
        }
    }

    #[test]
    fn a_shell_we_do_not_speak_gets_nothing_rather_than_something_wrong() {
        assert!(completions("tcsh").is_none());
        assert!(completions("").is_none());
    }

    #[test]
    fn a_flag_resolves_from_either_spelling() {
        assert_eq!(flag("--all").map(|f| f.long), Some("--all"));
        assert_eq!(flag("-a").map(|f| f.long), Some("--all"));
        assert_eq!(flag("--color=never").map(|f| f.long), Some("--color"));
        assert!(flag("--nonesuch").is_none());
        assert!(flag("-z").is_none());
    }

    #[test]
    fn what_takes_a_value_is_one_answer_everywhere() {
        assert!(flag("--theme").expect("theme").wants_value());
        assert!(!flag("--all").expect("all").wants_value());
    }

    /// Quoting mistakes in a generated script are not found by reading it.
    #[test]
    fn every_script_is_syntactically_valid_where_the_shell_is_here_to_say_so() {
        // Whichever of them is installed. A shell that is not here cannot say,
        // and skipping is honest; CI has bash and zsh, and a developer on fish
        // gets the third.
        for shell in SHELLS {
            let script = completions(shell).expect("a script");
            let file = std::env::temp_dir()
                .join(format!("harrow-completions-{}-{shell}", std::process::id()));
            std::fs::write(&file, &script).expect("write");
            let checked = std::process::Command::new(shell)
                .arg("-n")
                .arg(&file)
                .output();
            let _ = std::fs::remove_file(&file);
            if let Ok(out) = checked {
                assert!(
                    out.status.success(),
                    "{shell} rejects its own completions: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    }
}
