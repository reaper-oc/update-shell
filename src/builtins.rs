use crate::job::{JobControl, JobStatus};
use crate::parser::Pipeline;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn execute_builtin(
    cmd: &str,
    args: &[String],
    pipelines: &[Pipeline],
    history: &[String],
    jobs: &mut JobControl,
) -> Option<i32> {
    match cmd {
        "exit" => {
            let code = args.first().and_then(|a| a.parse().ok()).unwrap_or(0);
            std::process::exit(code);
        }
        "cd" => {
            let target = args.first().map(|s| s.as_str()).unwrap_or("~");
            let expanded = if target == "~" {
                env::var("HOME").unwrap_or_else(|_| "/".into())
            } else if target.starts_with("~/") {
                env::var("HOME")
                    .map(|h| h + &target[1..])
                    .unwrap_or_else(|_| target.to_string())
            } else {
                target.to_string()
            };
            if let Err(e) = env::set_current_dir(Path::new(&expanded)) {
                eprintln!("cd: {}: {}", expanded, e);
            }
            Some(0)
        }
        "pwd" => {
            if let Ok(cwd) = env::current_dir() {
                println!("{}", cwd.display());
            }
            Some(0)
        }
        "echo" => {
            let text = args.join(" ");
            println!("{}", text);
            Some(0)
        }
        "clear" => {
            use std::io::Write;
            let _ = std::io::stdout().write_all(b"\x1b[2J\x1b[3J\x1b[H");
            let _ = std::io::stdout().flush();
            Some(0)
        }
        "export" => {
            for arg in args {
                if let Some(eq) = arg.find('=') {
                    let key = &arg[..eq];
                    let val = &arg[eq + 1..];
                    env::set_var(key, val);
                }
            }
            Some(0)
        }
        "history" => {
            for (i, entry) in history.iter().enumerate() {
                println!("{:>5}  {}", i + 1, entry);
            }
            Some(0)
        }
        "help" => {
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("updsh");
            let _ = fs::create_dir_all(&data_dir);
            let help_path = data_dir.join("help.html");
            let html = include_str!("help.html");
            if fs::write(&help_path, html).is_ok() {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&help_path)
                    .spawn();
            } else {
                eprintln!("help: failed to write help file");
            }
            Some(0)
        }
        "jobs" => {
            if jobs.is_empty() {
                println!("No background jobs.");
            } else {
                for job in jobs.list_jobs() {
                    println!("{}", crate::job::format_job_status(job));
                }
            }
            Some(0)
        }
        "fg" => {
            let id = args
                .first()
                .and_then(|a| a.parse::<u32>().ok())
                .or_else(|| jobs.list_jobs().last().map(|j| j.id));
            if let Some(job_id) = id {
                if let Some(job) = jobs.remove_job(job_id) {
                    crate::executor::resume_job_foreground(job.pid.0);
                } else {
                    eprintln!("fg: job not found: {}", job_id);
                }
            } else {
                eprintln!("fg: no current job");
            }
            Some(0)
        }
        "bg" => {
            let id = args
                .first()
                .and_then(|a| a.parse::<u32>().ok())
                .or_else(|| jobs.list_jobs().last().map(|j| j.id));
            if let Some(job_id) = id {
                if let Some(job) = jobs.get_job(job_id) {
                    crate::executor::resume_job_background(job.pid.0);
                    jobs.update_job(job_id, JobStatus::Running);
                } else {
                    eprintln!("bg: job not found: {}", job_id);
                }
            } else {
                eprintln!("bg: no current job");
            }
            Some(0)
        }
        "source" => {
            if let Some(file) = args.first() {
                match std::fs::read_to_string(file) {
                    Ok(content) => {
                        for line in content.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
                                continue;
                            }
                            let parsed = crate::parser::parse_line(line);
                            for pipeline in parsed {
                                for cmd in &pipeline.commands {
                                    if let Some(_) = execute_builtin(
                                        &cmd.args[0],
                                        &cmd.args[1..],
                                        pipelines,
                                        history,
                                        jobs,
                                    ) {
                                    } else {
                                        crate::executor::execute_external(cmd, jobs);
                                    }
                                }
                            }
                        }
                        Some(0)
                    }
                    Err(e) => {
                        eprintln!("source: {}: {}", file, e);
                        Some(1)
                    }
                }
            } else {
                eprintln!("source: missing filename");
                Some(1)
            }
        }
        "alias" => {
            if args.is_empty() {
                for (name, value) in crate::alias::list() {
                    println!("alias {}='{}'", name, value);
                }
            } else {
                for arg in args {
                    if let Some(eq) = arg.find('=') {
                        let name = &arg[..eq];
                        let value = &arg[eq + 1..];
                        let value = value.trim_matches('\'').trim_matches('"');
                        crate::alias::set(name, value);
                    } else {
                        if let Some(val) = crate::alias::get(arg) {
                            println!("alias {}='{}'", arg, val);
                        }
                    }
                }
            }
            Some(0)
        }
        "unalias" => {
            for arg in args {
                crate::alias::remove(arg);
            }
            Some(0)
        }
        "pkg" => {
            crate::pkg::execute_pkg(args)
        }
        "type" => {
            if let Some(name) = args.first() {
                if is_builtin(name) {
                    println!("{} is a shell builtin", name);
                } else if let Ok(paths) = env::var("PATH") {
                    let found = env::split_paths(&paths).any(|p| {
                        let full = p.join(name);
                        full.is_file()
                    });
                    if found {
                        println!("{} is an external command", name);
                    } else {
                        println!("{}: not found", name);
                    }
                } else {
                    println!("{}: not found", name);
                }
            }
            Some(0)
        }
        _ => None,
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(
        cmd,
        "cd" | "exit" | "pwd" | "echo" | "clear" | "export" | "history" | "help" | "jobs"
            | "fg" | "bg" | "source" | "type" | "pkg" | "alias" | "unalias"
    )
}
