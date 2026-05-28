mod alias;
mod builtins;
mod completer;
mod config;
mod executor;
mod history;
mod job;
mod parser;
mod pkg;
mod prompt;
mod signal;

use crate::completer::UpdshCompleter;
use crate::history::History;
use crate::job::JobControl;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use signal::{SIGCHLD_RECEIVED, SIGINT_RECEIVED};
use std::sync::atomic::Ordering;
use std::panic;

const MAX_INPUT_LEN: usize = 65536;

fn main() {
    signal::setup_signals();

    let mut history = History::new();
    let mut jobs = JobControl::new();
    let shell_cfg = config::load_config();

    let config_builder = Config::builder()
        .auto_add_history(false)
        .history_ignore_space(true)
        .max_history_size(10000);
    let config = config_builder.expect("valid config").build();

    let mut rl = match Editor::with_config(config) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("updsh: failed to initialize editor: {}", e);
            std::process::exit(1);
        }
    };

    std::env::set_var("UPD_SHELL", "updsh");
    alias::init();

    pkg::source_packages();
    pkg::apply_enabled_packages();

    rl.set_helper(Some(UpdshCompleter::new(history.all())));

    for entry in history.all() {
        let _ = rl.add_history_entry(entry);
    }

    show_greeting();

    loop {
        check_sigchld(&mut jobs);

        let prompt = prompt::build_prompt(&shell_cfg);
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                if line.len() > MAX_INPUT_LEN {
                    eprintln!("updsh: input too long (max {} bytes)", MAX_INPUT_LEN);
                    continue;
                }

                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                history.add(&trimmed);
                let _ = rl.add_history_entry(trimmed.as_str());

                let expanded = alias::expand(&trimmed);
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    parser::parse_line(&expanded)
                }));
                let pipelines = match result {
                    Ok(p) => p,
                    Err(_) => {
                        eprintln!("updsh: parser error — malformed input");
                        continue;
                    }
                };
                for pipeline in &pipelines {
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        executor::execute_pipeline(
                            &pipeline.commands,
                            &pipelines,
                            history.all(),
                            &mut jobs,
                        )
                    }));
                    if result.is_err() {
                        eprintln!("updsh: execution error");
                        continue;
                    }
                }

                check_sigchld(&mut jobs);

                if SIGINT_RECEIVED.swap(false, Ordering::SeqCst) {
                    println!();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!();
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(e) => {
                eprintln!("updsh: readline error: {}", e);
                break;
            }
        }
    }
}

fn show_greeting() {
    let style = |s: &str, c: &str| format!("\x1b[{}m{}\x1b[0m", c, s);
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| {
        let mut buf = vec![0u8; 256];
        unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, 256); }
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    });
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".into());

    println!();
    println!("  {} {} {}",
        style("╭───", "90"),
        style("updSH", "1;34"),
        style("───╮", "90"),
    );
    println!("  {} {}@{}  {}",
        style("│", "90"),
        style(&user, "32"),
        style(&host, "34"),
        style("│", "90"),
    );
    println!("  {} {}  {}",
        style("│", "90"),
        style(&os, "33"),
        style("│", "90"),
    );
    println!("  {} {}",
        style("╰──────╯", "90"),
        style("  Type help for commands", "2"),
    );
    println!();
}

fn check_sigchld(jobs: &mut JobControl) {
    if !SIGCHLD_RECEIVED.swap(false, Ordering::SeqCst) {
        return;
    }

    loop {
        let mut status: i32 = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };

        if pid <= 0 {
            break;
        }

        if let Some(job_id) = jobs.find_by_pid(pid) {
            let done_cmd = jobs.get_job(job_id).map(|j| j.command.clone());
            jobs.update_job(job_id, job::JobStatus::Done);
            if let Some(cmd) = done_cmd {
                println!();
                println!("[{}]  Done  {}", job_id, cmd);
            }
        }
    }

    jobs.cleanup_done();
}
