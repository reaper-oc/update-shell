use std::sync::atomic::{AtomicBool, Ordering};

pub static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);
pub static SIGCHLD_RECEIVED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigint(_: i32) {
    SIGINT_RECEIVED.store(true, Ordering::SeqCst);
}

extern "C" fn handle_sigchld(_: i32) {
    SIGCHLD_RECEIVED.store(true, Ordering::SeqCst);
}

pub fn setup_signals() {
    unsafe {
        let mut sa_int: libc::sigaction = std::mem::zeroed();
        sa_int.sa_sigaction = handle_sigint as *const () as usize;
        sa_int.sa_flags = libc::SA_RESTART;
        libc::sigaction(libc::SIGINT, &sa_int, std::ptr::null_mut());

        let mut sa_chld: libc::sigaction = std::mem::zeroed();
        sa_chld.sa_sigaction = handle_sigchld as *const () as usize;
        sa_chld.sa_flags = libc::SA_RESTART | libc::SA_NOCLDSTOP;
        libc::sigaction(libc::SIGCHLD, &sa_chld, std::ptr::null_mut());

        let mut mask: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut mask);
        libc::sigaddset(&mut mask, libc::SIGQUIT);
        libc::sigaddset(&mut mask, libc::SIGTSTP);
        libc::sigaddset(&mut mask, libc::SIGTTIN);
        libc::sigaddset(&mut mask, libc::SIGTTOU);
        libc::pthread_sigmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut());
    }
}
