//! Layer 1 — "The Wall": a hard, kernel-enforced memory cap installed on the
//! child *before* it execs.
//!
//! On Linux we set `RLIMIT_AS` (total virtual address space) via a `pre_exec`
//! hook. Once the child crosses the cap, the kernel makes `malloc`/`calloc`
//! return `NULL` — the memory is never actually backed, so the machine cannot
//! thrash, and there is no monitoring race to lose.
//!
//! On platforms that do not reliably enforce `RLIMIT_AS` (macOS, Windows) this
//! is a no-op; Layer 2 (the monitor) provides protection there.

use std::process::Command;

/// Install the memory wall on `cmd` so that the spawned child cannot allocate
/// more than `max_bytes` of address space.
#[cfg(target_os = "linux")]
pub fn install_memory_wall(cmd: &mut Command, max_bytes: u64) {
    use std::io;
    use std::os::unix::process::CommandExt;

    // `pre_exec` runs in the forked child, after `fork()` and before `execvp()`.
    // The closure must be async-signal-safe: a single `setrlimit` syscall is.
    unsafe {
        cmd.pre_exec(move || {
            let limit = libc::rlimit {
                rlim_cur: max_bytes as libc::rlim_t,
                rlim_max: max_bytes as libc::rlim_t,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

/// No-op fallback: `RLIMIT_AS` is not reliably enforced here, so Layer 2 guards
/// the process instead.
#[cfg(not(target_os = "linux"))]
pub fn install_memory_wall(_cmd: &mut Command, _max_bytes: u64) {}
