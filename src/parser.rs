use std::env;

#[derive(Debug, Clone)]
pub struct Redirect {
    pub fd: u32,
    pub target: String,
    pub append: bool,
    pub input: bool,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
    pub background: bool,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

fn expand_vars(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('{') => {
                    chars.next();
                    let mut var = String::new();
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            break;
                        }
                        var.push(ch);
                    }
                    let val = env::var(&var).unwrap_or_default();
                    result.push_str(&val);
                }
                Some('$') => {
                    chars.next();
                    result.push_str(&std::process::id().to_string());
                }
                Some(c) if c.is_alphanumeric() || *c == '_' => {
                    let mut var = String::new();
                    while let Some(ch) = chars.peek() {
                        if ch.is_alphanumeric() || *ch == '_' {
                            var.push(*ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let val = env::var(&var).unwrap_or_default();
                    result.push_str(&val);
                }
                _ => result.push('$'),
            }
        } else if c == '~' && result.is_empty() {
            if let Ok(home) = env::var("HOME") {
                result.push_str(&home);
            } else {
                result.push('~');
            }
        } else if c == '\\' {
            match chars.next() {
                Some(next) => result.push(next),
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn parse_line(line: &str) -> Vec<Pipeline> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    let mut pipelines = vec![];
    let mut current_pipeline = Pipeline { commands: vec![] };
    let mut current_cmd = Command {
        args: vec![],
        redirects: vec![],
        background: false,
    };

    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut token = String::new();

    for ch in trimmed.chars() {
        if escaped {
            match ch {
                'n' => token.push('\n'),
                't' => token.push('\t'),
                'r' => token.push('\r'),
                '\\' => token.push('\\'),
                '"' => token.push('"'),
                '\'' => token.push('\''),
                '$' => token.push('$'),
                ' ' => token.push(' '),
                c => {
                    token.push('\\');
                    token.push(c);
                }
            }
            escaped = false;
            continue;
        }

        if ch == '\\' && !in_single {
            escaped = true;
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }

        if in_single {
            token.push(ch);
            continue;
        }

        if ch == '#' && token.is_empty() && current_cmd.args.is_empty()
            && current_cmd.redirects.is_empty()
        {
            break;
        }

        if ch == ';' && !in_double {
            flush_token(&mut current_cmd, &mut token);
            push_cmd(&mut current_pipeline, &mut current_cmd);
            push_pipeline(&mut pipelines, &mut current_pipeline);
            continue;
        }

        if ch == '|' && !in_double {
            flush_token(&mut current_cmd, &mut token);
            push_cmd(&mut current_pipeline, &mut current_cmd);
            continue;
        }

        if ch == '&' && !in_double {
            flush_token(&mut current_cmd, &mut token);
            if !current_cmd.args.is_empty() || !current_cmd.redirects.is_empty() {
                current_cmd.background = true;
                current_pipeline.commands.push(current_cmd.clone());
            }
            push_pipeline(&mut pipelines, &mut current_pipeline);
            current_cmd = Command {
                args: vec![],
                redirects: vec![],
                background: false,
            };
            continue;
        }

        if ch == ' ' && !in_double {
            flush_token(&mut current_cmd, &mut token);
            continue;
        }

        token.push(ch);
    }

    if escaped {
        token.push('\\');
    }

    flush_token(&mut current_cmd, &mut token);
    push_cmd(&mut current_pipeline, &mut current_cmd);
    push_pipeline(&mut pipelines, &mut current_pipeline);

    pipelines
}

fn flush_token(cmd: &mut Command, token: &mut String) {
    if token.is_empty() {
        return;
    }
    let expanded = expand_vars(token);
    token.clear();

    match expanded.as_str() {
        "<" | ">" | ">>" | "2>" | "2>>" => {
            let (fd, input, append) = match expanded.as_str() {
                "<" => (0, true, false),
                ">" => (1, false, false),
                ">>" => (1, false, true),
                "2>" => (2, false, false),
                "2>>" => (2, false, true),
                _ => unreachable!(),
            };
            cmd.redirects.push(Redirect {
                fd,
                target: String::new(),
                append,
                input,
            });
        }
        _ => {
            if let Some(last) = cmd.redirects.last_mut() {
                if last.target.is_empty() {
                    last.target = expanded;
                    return;
                }
            }
            cmd.args.push(expanded);
        }
    }
}

fn push_cmd(pipeline: &mut Pipeline, cmd: &mut Command) {
    if !cmd.args.is_empty() || !cmd.redirects.is_empty() {
        pipeline.commands.push(cmd.clone());
        *cmd = Command {
            args: vec![],
            redirects: vec![],
            background: false,
        };
    }
}

fn push_pipeline(pipelines: &mut Vec<Pipeline>, pipeline: &mut Pipeline) {
    if !pipeline.commands.is_empty() {
        pipelines.push(pipeline.clone());
        *pipeline = Pipeline { commands: vec![] };
    }
}
