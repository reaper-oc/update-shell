use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::highlight::CmdKind;
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::Path;

pub struct UpdshCompleter {
    pub history: Vec<String>,
}

impl UpdshCompleter {
    pub fn new(history: &[String]) -> Self {
        UpdshCompleter {
            history: history.to_vec(),
        }
    }

    fn complete_path(input: &str) -> Vec<Pair> {
        let (dir, partial) = if input.ends_with('/') {
            (input.to_string(), String::new())
        } else {
            let path = Path::new(input);
            let parent = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let d = if !parent.is_empty() {
                format!("{}/", parent)
            } else {
                "./".to_string()
            };
            (d, file_name.to_string())
        };

        let dir_path = if dir.starts_with('~') {
            if let Ok(home) = env::var("HOME") {
                dir.replacen('~', &home, 1)
            } else {
                dir.clone()
            }
        } else {
            dir.clone()
        };

        let entries = match fs::read_dir(&dir_path) {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        let mut pairs = vec![];
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(&partial) && !name_str.starts_with('.') && partial != name_str {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let replacement = if is_dir {
                    format!("{}/", name_str)
                } else {
                    format!("{} ", name_str)
                };
                pairs.push(Pair {
                    display: format!("{}{}", dir, name_str),
                    replacement,
                });
            }
        }
        pairs
    }

    fn complete_command(input: &str) -> Vec<Pair> {
        let mut seen = std::collections::HashSet::new();
        let mut pairs = vec![];

        if let Ok(paths) = env::var("PATH") {
            for p in env::split_paths(&paths) {
                let entries = match fs::read_dir(&p) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy().to_string();
                    if name_str.starts_with(input) && seen.insert(name_str.clone()) {
                        pairs.push(Pair {
                            display: name_str.clone(),
                            replacement: name_str,
                        });
                    }
                }
            }
        }

        let builtins = [
            "cd", "exit", "history", "help", "jobs", "fg", "bg", "export", "source", "type",
            "echo", "pwd", "clear",
        ];
        for cmd in builtins {
            if cmd.starts_with(input) && seen.insert(cmd.to_string()) {
                pairs.push(Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                });
            }
        }

        pairs
    }

    fn complete_pkg(input: &str, word_count: usize) -> Vec<Pair> {
        if word_count == 2 {
            let subcmds = [
                "install", "remove", "list", "search", "info", "update", "help",
            ];
            let mut pairs = vec![];
            for cmd in subcmds {
                if cmd.starts_with(input) {
                    pairs.push(Pair {
                        display: cmd.to_string(),
                        replacement: format!("{} ", cmd),
                    });
                }
            }
            return pairs;
        }

        if word_count == 3 {
            let subcmd = input; // input here is the 3rd word
            if subcmd == "install" || subcmd == "info" {
                return Self::complete_pkg_name("");
            }
            if subcmd == "remove" {
                return Self::complete_installed("");
            }
        }

        if word_count >= 3 {
            return vec![];
        }

        vec![]
    }

    fn complete_pkg_name(partial: &str) -> Vec<Pair> {
        let pkgs = crate::pkg::list_available_names();
        pkgs.iter()
            .filter(|n| n.starts_with(partial))
            .map(|n| Pair {
                display: n.clone(),
                replacement: format!("{} ", n),
            })
            .collect()
    }

    fn complete_installed(partial: &str) -> Vec<Pair> {
        let pkgs = crate::pkg::list_installed_names();
        pkgs.iter()
            .filter(|n| n.starts_with(partial))
            .map(|n| Pair {
                display: n.clone(),
                replacement: format!("{} ", n),
            })
            .collect()
    }
}

impl Completer for UpdshCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let line_before = &line[..pos];

        if let Some(last_word) = line_before.split_whitespace().last() {
            if last_word.is_empty() {
                return Ok((pos, vec![]));
            }

            let words: Vec<&str> = line_before.split_whitespace().collect();
            let word_count = words.len();

            if words[0] == "pkg" {
                if word_count == 2 {
                    let pairs = Self::complete_pkg(last_word, word_count);
                    let start = pos - last_word.len();
                    return Ok((start, pairs));
                }
                if word_count >= 3 {
                    let subcmd = words[1];
                    let pairs = match subcmd {
                        "install" | "info" | "show" => Self::complete_pkg_name(last_word),
                        "remove" | "r" => Self::complete_installed(last_word),
                        _ => vec![],
                    };
                    let start = pos - last_word.len();
                    return Ok((start, pairs));
                }
                return Ok((pos, vec![]));
            }

            if word_count == 1 && !last_word.contains('/') {
                let pairs = Self::complete_command(last_word);
                let start = pos - last_word.len();
                return Ok((start, pairs));
            }

            let pairs = Self::complete_path(last_word);
            let start = pos - last_word.len();
            return Ok((start, pairs));
        }

        Ok((pos, vec![]))
    }
}

impl Hinter for UpdshCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let line_before = &line[..pos];
        let trimmed = line_before.trim();
        if trimmed.is_empty() || line_before.contains(' ') {
            return None;
        }

        for entry in self.history.iter().rev() {
            if entry.len() > trimmed.len()
                && entry.starts_with(trimmed)
                && entry.as_str() != trimmed
            {
                return Some(entry[trimmed.len()..].to_string());
            }
        }
        None
    }
}

impl Highlighter for UpdshCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char<'l>(&self, _line: &'l str, _pos: usize, _kind: CmdKind) -> bool {
        false
    }
}

impl Validator for UpdshCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for UpdshCompleter {}
