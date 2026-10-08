//! Keeps other libraries from writing over the game. Sound libraries (ALSA in
//! particular) print warnings such as buffer underruns straight to stderr, which is the
//! terminal the game draws on. While the game runs, stderr goes to a log file instead.

use std::path::Path;

#[cfg(unix)]
mod imp {
    use std::fs::{self, File};
    use std::os::fd::AsRawFd;
    use std::path::Path;
    use std::sync::atomic::{AtomicI32, Ordering};

    /// The real stderr, kept to put back; -1 when not redirected.
    static SAVED: AtomicI32 = AtomicI32::new(-1);

    pub fn stderr_to_log(path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let Ok(log) = File::create(path).or_else(|_| File::create("/dev/null")) else { return };
        // SAFETY: dup and dup2 only duplicate file descriptors; both are valid here.
        unsafe {
            let saved = libc::dup(2);
            if saved >= 0 && libc::dup2(log.as_raw_fd(), 2) >= 0 {
                SAVED.store(saved, Ordering::SeqCst);
            } else if saved >= 0 {
                libc::close(saved);
            }
        }
    }

    pub fn restore_stderr() {
        let saved = SAVED.swap(-1, Ordering::SeqCst);
        if saved >= 0 {
            // SAFETY: `saved` is the descriptor dup'd above and not yet closed.
            unsafe {
                libc::dup2(saved, 2);
                libc::close(saved);
            }
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::Path;

    pub fn stderr_to_log(_path: &Path) {}

    pub fn restore_stderr() {}
}

/// From now on, stderr goes to the log at `path`.
pub fn stderr_to_log(path: &Path) {
    imp::stderr_to_log(path);
}

/// Puts the terminal's stderr back. Safe to call more than once.
pub fn restore_stderr() {
    imp::restore_stderr();
}
