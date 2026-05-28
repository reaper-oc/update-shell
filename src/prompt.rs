use crate::config::ShellConfig;
use std::env;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};

pub static LAST_EXIT_CODE: AtomicI32 = AtomicI32::new(0);

fn get_git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }
    None
}

fn get_hostname() -> String {
    let mut buf = vec![0u8; 256];
    let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, 256) };
    if ret == 0 {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        "host".into()
    }
}

fn get_cwd() -> String {
    env::current_dir()
        .map(|p| {
            let path = p.to_string_lossy().into_owned();
            if let Ok(home) = env::var("HOME") {
                if path.starts_with(&home) {
                    return path.replacen(&home, "~", 1);
                }
            }
            path
        })
        .unwrap_or_else(|_| "?".into())
}

fn style(s: &str, code: &str) -> String {
    if code.is_empty() || code == "0" {
        s.to_string()
    } else {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    }
}

pub fn build_prompt(cfg: &ShellConfig) -> String {
    let user = env::var("USER").unwrap_or_else(|_| "user".into());
    let host = get_hostname();
    let cwd = get_cwd();
    let prompt_char = match env::var("USER").as_deref() {
        Ok("root") => "#",
        _ => "$",
    };
    let exit_code = LAST_EXIT_CODE.load(Ordering::Relaxed);
    let branch = if cfg.show_git { get_git_branch() } else { None };

    match cfg.prompt_style.as_str() {
        "minimal" => build_minimal(&user, &host, &cwd, &branch, prompt_char, exit_code, cfg),
        "powerline" => build_powerline(&user, &host, &cwd, &branch, prompt_char, exit_code, cfg),
        "plain" => build_plain(&user, &host, &cwd, &branch, prompt_char, exit_code, cfg),
        _ => build_multiline(&user, &host, &cwd, &branch, prompt_char, exit_code, cfg),
    }
}

fn build_multiline(
    user: &str, host: &str, cwd: &str, branch: &Option<String>,
    prompt_char: &str, exit_code: i32, cfg: &ShellConfig,
) -> String {
    let mut top = format!(
        "\x1b[90m┌─\x1b[0m{}@{} \x1b[90m─\x1b[0m {}",
        style(user, &cfg.color_user),
        style(host, &cfg.color_host),
        style(cwd, &cfg.color_path),
    );
    if let Some(b) = branch {
        top.push_str(&format!(" \x1b[90m(\x1b[0m{})\x1b[90m", style(b, &cfg.color_git)));
    }
    let exit_str = if exit_code != 0 {
        format!(" \x1b[{}m\u{2717} {}\x1b[0m", cfg.color_exit, exit_code)
    } else {
        String::new()
    };
    top.push_str(&exit_str);

    let width = terminal_width();
    if top.len() > width.saturating_sub(4) {
        top.truncate(width.saturating_sub(7));
        top.push_str("\x1b[90m…\x1b[0m");
    }

    format!(
        "{}\n\x1b[90m└─\x1b[0m{} ",
        top,
        style(prompt_char, &cfg.color_user),
    )
}

fn build_powerline(
    user: &str, host: &str, cwd: &str, branch: &Option<String>,
    prompt_char: &str, exit_code: i32, cfg: &ShellConfig,
) -> String {
    let bg_user = code_to_bg(&cfg.color_user);
    let bg_host = code_to_bg(&cfg.color_host);
    let bg_path = code_to_bg(&cfg.color_path);
    let fg_black = "\x1b[30m";
    let reset = "\x1b[0m";

    let mut p = format!(
        "{}{} {} {}{}{} {} {}{}{} {}",
        bg_user, fg_black, user,
        "\x1b[42m\x1b[32m",  // green bg, green fg for separator
        bg_host, fg_black, host,
        "\x1b[44m\x1b[34m",
        bg_path, fg_black, cwd,
    );

    if let Some(b) = branch {
        let bg_git = code_to_bg(&cfg.color_git);
        p.push_str(&format!(
            " {}{}{} {}",
            "\x1b[41m\x1b[31m",
            bg_git, "\x1b[30m", b,
        ));
    }

    p.push_str(&format!("{} {}", reset, style(prompt_char, &cfg.color_user)));

    if exit_code != 0 {
        p = format!("{} {} {}", style(&exit_code.to_string(), &cfg.color_exit), p, reset);
    }

    p
}

fn build_minimal(
    _user: &str, _host: &str, cwd: &str, branch: &Option<String>,
    prompt_char: &str, exit_code: i32, cfg: &ShellConfig,
) -> String {
    let mut p = format!("{} ", style(cwd, &cfg.color_path));
    if let Some(b) = branch {
        p.push_str(&format!("{} ", style(&format!("({})", b), &cfg.color_git)));
    }
    if exit_code != 0 {
        p = format!("{} {}", style(&exit_code.to_string(), &cfg.color_exit), p);
    }
    p.push_str(&style(prompt_char, &cfg.color_user));
    p.push(' ');
    p
}

fn build_plain(
    _user: &str, _host: &str, cwd: &str, branch: &Option<String>,
    prompt_char: &str, _exit_code: i32, _cfg: &ShellConfig,
) -> String {
    let mut p = format!("{} {}", cwd, prompt_char);
    if let Some(b) = branch {
        p.push_str(&format!(" ({})", b));
    }
    p.push(' ');
    p
}

fn terminal_width() -> usize {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 {
            ws.ws_col as usize
        } else {
            80
        }
    }
}

fn code_to_bg(code: &str) -> String {
    let base = code.split(';').last().unwrap_or(code);
    match base {
        "30" => "\x1b[40m".into(),
        "31" => "\x1b[41m".into(),
        "32" => "\x1b[42m".into(),
        "33" => "\x1b[43m".into(),
        "34" => "\x1b[44m".into(),
        "35" => "\x1b[45m".into(),
        "36" => "\x1b[46m".into(),
        "37" => "\x1b[47m".into(),
        "1;31" => "\x1b[41;1m".into(),
        "1;32" => "\x1b[42;1m".into(),
        "1;33" => "\x1b[43;1m".into(),
        "1;34" => "\x1b[44;1m".into(),
        "1;35" => "\x1b[45;1m".into(),
        "1;36" => "\x1b[46;1m".into(),
        "1;37" => "\x1b[47;1m".into(),
        _ => format!("\x1b[{}m", base.replace("3", "4")),
    }
}
