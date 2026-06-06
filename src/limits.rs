use std::process::Command;

#[cfg(target_os = "linux")]
pub fn install_memory_wall(cmd: &mut Command, max_bytes: u64) {
    use std::io;
    use std::os::unix::process::CommandExt;

    // Runs in the forked child before exec; setrlimit is async-signal-safe.
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

#[cfg(not(target_os = "linux"))]
pub fn install_memory_wall(_cmd: &mut Command, _max_bytes: u64) {}
