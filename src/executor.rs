use crate::builtins;
use crate::job::JobControl;
use crate::parser::Command;
use std::env;
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::sync::atomic::Ordering;

pub fn lookup_cmd(cmd: &str) -> Option<String> {
    if cmd.contains('/') {
        let path = std::path::Path::new(cmd);
        if path.is_file() {
            return Some(cmd.to_string());
        }
        return None;
    }

    if let Ok(paths) = env::var("PATH") {
        for p in env::split_paths(&paths) {
            let full = p.join(cmd);
            if full.is_file() {
                return Some(full.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn path_lookup(cmd: &str) -> Option<CString> {
    if cmd.contains('/') {
        let path = std::path::Path::new(cmd);
        if path.is_file() {
            return CString::new(cmd).ok();
        }
        return None;
    }

    if let Ok(paths) = env::var("PATH") {
        for p in env::split_paths(&paths) {
            let full = p.join(cmd);
            if full.is_file() {
                return CString::new(full.to_string_lossy().as_ref()).ok();
            }
        }
    }
    None
}

fn to_cstrings(args: &[String]) -> Vec<CString> {
    args.iter()
        .filter_map(|a| CString::new(a.as_str()).ok())
        .collect()
}

pub fn execute_pipeline(
    commands: &[Command],
    pipelines: &[crate::parser::Pipeline],
    history: &[String],
    jobs: &mut JobControl,
) {
    if commands.is_empty() {
        return;
    }

    if commands.len() == 1 {
        execute_single(&commands[0], pipelines, history, jobs);
        return;
    }

    execute_piped(commands, pipelines, history, jobs);
}

fn child_setup() {
    unsafe {
        let _ = libc::setpgid(0, 0);
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGQUIT, libc::SIG_DFL);
        libc::signal(libc::SIGTSTP, libc::SIG_DFL);
        libc::signal(libc::SIGTTIN, libc::SIG_DFL);
        libc::signal(libc::SIGTTOU, libc::SIG_DFL);
        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
        let mut mask: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut mask);
        libc::pthread_sigmask(libc::SIG_SETMASK, &mask, std::ptr::null_mut());
    }
}

fn wait_and_restore_tty(child: i32) -> i32 {
    unsafe {
        let _ = libc::setpgid(child, child);
        libc::tcsetpgrp(libc::STDIN_FILENO, child);
        let mut status: i32 = 0;
        libc::waitpid(child, &mut status, 0);
        libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
        if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { 1 }
    }
}

fn execute_single(
    cmd: &Command,
    pipelines: &[crate::parser::Pipeline],
    history: &[String],
    jobs: &mut JobControl,
) {
    if cmd.args.is_empty() {
        return;
    }

    let cmd_name = &cmd.args[0];

    let is_builtin = builtins::execute_builtin(cmd_name, &cmd.args[1..], pipelines, history, jobs).is_some();
    let has_redirects = !cmd.redirects.is_empty();

    if is_builtin && !has_redirects {
        crate::prompt::LAST_EXIT_CODE.store(0, Ordering::Relaxed);
        return;
    }

    let foreground = !cmd.background;

    unsafe {
        if is_builtin && has_redirects {
            let child = libc::fork();
            match child {
                -1 => eprintln!("updsh: fork failed"),
                0 => {
                    child_setup();
                    apply_redirects(cmd);
                    builtins::execute_builtin(cmd_name, &cmd.args[1..], pipelines, history, jobs);
                    libc::_exit(0);
                }
                _ => {
                    if foreground {
                        crate::prompt::LAST_EXIT_CODE.store(wait_and_restore_tty(child) as i32, Ordering::Relaxed);
                    } else {
                        let _ = libc::setpgid(child, child);
                        let job_id = jobs.add_job(
                            crate::job::PidWrapper(child),
                            cmd.args.join(" "),
                            false,
                        );
                        println!("[{}] {}", job_id, child);
                        jobs.cleanup_done();
                    }
                }
            }
            return;
        }
        let child = libc::fork();
        match child {
            -1 => {
                eprintln!("updsh: fork failed");
            }
            0 => {
                child_setup();
                apply_redirects(cmd);
                if let Some(path) = path_lookup(cmd_name) {
                    let args = to_cstrings(&cmd.args);
                    let mut cargs: Vec<*const libc::c_char> =
                        args.iter().map(|a| a.as_ptr()).collect();
                    cargs.push(std::ptr::null());
                    libc::execvp(path.as_ptr(), cargs.as_ptr());
                }
                eprintln!("updsh: command not found: {}", cmd_name);
                libc::_exit(127);
            }
            _ => {
                if foreground {
                    crate::prompt::LAST_EXIT_CODE.store(wait_and_restore_tty(child) as i32, Ordering::Relaxed);
                } else {
                    let _ = libc::setpgid(child, child);
                    let job_id = jobs.add_job(
                        crate::job::PidWrapper(child),
                        cmd.args.join(" "),
                        false,
                    );
                    println!("[{}] {}", job_id, child);
                    jobs.cleanup_done();
                }
            }
        }
    }
}

fn execute_piped(
    commands: &[Command],
    _pipelines: &[crate::parser::Pipeline],
    _history: &[String],
    jobs: &mut JobControl,
) {
    let n = commands.len();
    let mut children: Vec<i32> = vec![];
    let mut prev_read: RawFd = -1;

    for (i, cmd) in commands.iter().enumerate() {
        let mut pipe_fds: [RawFd; 2] = [-1, -1];
        if i < n - 1 {
            if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } != 0 {
                eprintln!("updsh: pipe failed");
                return;
            }
        }

        unsafe {
            let child = libc::fork();
            match child {
                -1 => {
                    eprintln!("updsh: fork failed");
                    return;
                }
                0 => {
                    if prev_read != -1 {
                        libc::dup2(prev_read, 0);
                        libc::close(prev_read);
                    }
                    if i < n - 1 {
                        libc::close(pipe_fds[0]);
                        libc::dup2(pipe_fds[1], 1);
                        libc::close(pipe_fds[1]);
                    }

                    child_setup();
                    if i == 0 {
                        let _ = libc::setpgid(0, 0);
                    } else {
                        let _ = libc::setpgid(0, children[0]);
                    }

                    apply_redirects(cmd);

                    let cmd_name = &cmd.args[0];
                    if let Some(path) = path_lookup(cmd_name) {
                        let args = to_cstrings(&cmd.args);
                        let mut cargs: Vec<*const libc::c_char> =
                            args.iter().map(|a| a.as_ptr()).collect();
                        cargs.push(std::ptr::null());
                        libc::execvp(path.as_ptr(), cargs.as_ptr());
                    }
                    eprintln!("updsh: command not found: {}", cmd_name);
                    libc::_exit(127);
                }
                _ => {
                    if children.is_empty() {
                        let _ = libc::setpgid(child, child);
                    } else {
                        let _ = libc::setpgid(child, children[0]);
                    }
                    children.push(child);
                    if prev_read != -1 {
                        libc::close(prev_read);
                    }
                    if i < n - 1 {
                        libc::close(pipe_fds[1]);
                    }
                    prev_read = pipe_fds[0];
                }
            }
        }
    }

    if !children.is_empty() {
        if !commands.last().map_or(false, |c| c.background) {
            let pgid = children[0];
            unsafe {
                libc::tcsetpgrp(libc::STDIN_FILENO, pgid);
            }

            let mut last_status: i32 = 0;
            for (i, &child) in children.iter().enumerate() {
                let mut status: i32 = 0;
                unsafe {
                    libc::waitpid(child, &mut status, 0);
                }
                if i == children.len() - 1 {
                    last_status = status;
                }
            }

            unsafe {
                libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
            }
            let code = if libc::WIFEXITED(last_status) { libc::WEXITSTATUS(last_status) } else { 1 };
            crate::prompt::LAST_EXIT_CODE.store(code as i32, Ordering::Relaxed);
        } else {
            let cmd_str = commands
                .iter()
                .map(|c| c.args.join(" "))
                .collect::<Vec<_>>()
                .join(" | ");
            jobs.add_job(crate::job::PidWrapper(children[0]), cmd_str, false);
            jobs.cleanup_done();
        }
    }
}

fn apply_redirects(cmd: &Command) {
    for redir in &cmd.redirects {
        if redir.input {
            let fd = unsafe {
                let f = libc::open(
                    CString::new(redir.target.as_str()).unwrap().as_ptr(),
                    libc::O_RDONLY,
                );
                f
            };
            if fd >= 0 {
                unsafe {
                    libc::dup2(fd, redir.fd as i32);
                    libc::close(fd);
                }
            } else {
                eprintln!("updsh: {}: No such file or directory", redir.target);
            }
        } else {
            let flags = if redir.append {
                libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND
            } else {
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC
            };
            let mode = libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH;
            let fd = unsafe {
                libc::open(
                    CString::new(redir.target.as_str()).unwrap().as_ptr(),
                    flags,
                    mode,
                )
            };
            if fd >= 0 {
                unsafe {
                    libc::dup2(fd, redir.fd as i32);
                    libc::close(fd);
                }
            } else {
                eprintln!("updsh: {}: {}", redir.target, std::io::Error::last_os_error());
            }
        }
    }
}

pub fn execute_external(cmd: &Command, jobs: &mut JobControl) {
    execute_single(cmd, &[], &[], jobs);
}

pub fn resume_job_foreground(pid: i32) {
    unsafe {
        libc::tcsetpgrp(libc::STDIN_FILENO, pid);
        libc::kill(pid, libc::SIGCONT);
        let mut status: i32 = 0;
        libc::waitpid(pid, &mut status, 0);
        libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
    }
}

pub fn resume_job_background(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGCONT);
    }
}
