//! Process supervision: spawn the target as a child and guard it.
//!
//! Step 4 makes Layer 2 active: the loop evaluates the two safety heuristics
//! every tick and terminates the child the instant one is violated (unless
//! `--report-only`), returning a structured [`Outcome`]. The pretty post-mortem
//! card is built from this in Step 5.

use std::io;
use std::process::{Child, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::config::Config;
use crate::limits;
use crate::size::format_size;
use crate::term::{paint, stderr_color, stdout_color};

/// Which safety heuristic a process violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    /// Heuristic A: RSS crossed the absolute ceiling.
    AbsoluteLimit,
    /// Heuristic B: memory grew faster than the allowed velocity.
    Velocity,
}

impl Violation {
    /// Short human label for reports.
    pub fn reason(self) -> &'static str {
        match self {
            Violation::AbsoluteLimit => "Absolute Memory Limit Exceeded",
            Violation::Velocity => "Explosive Growth Velocity Detected",
        }
    }
}

/// How the supervised child ended.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// The child finished on its own.
    Exited(ExitStatus),
    /// mknight terminated it for violating a policy.
    Killed(Violation),
}

/// Live + final measurements gathered while supervising a child.
#[derive(Debug, Default, Clone)]
pub struct RunStats {
    /// Highest RSS observed, in bytes.
    pub peak_rss: u64,
    /// Highest growth rate observed, in bytes per second.
    pub peak_velocity: f64,
    /// Wall-clock time the child ran.
    pub duration: Duration,
}

/// Full result of a supervised run.
#[derive(Debug, Clone)]
pub struct Supervision {
    pub outcome: Outcome,
    pub stats: RunStats,
}

/// Build the child [`Command`] from config, attaching the Layer 1 memory wall
/// (a no-op on platforms that don't enforce it).
pub fn build_command(cfg: &Config) -> Command {
    let mut cmd = Command::new(&cfg.program);
    cmd.args(&cfg.args);
    if cfg.rlimit_requested {
        limits::install_memory_wall(&mut cmd, cfg.max_ram);
    }
    cmd
}

/// Spawn the child and supervise it until exit or policy violation.
pub fn supervise(cfg: &Config) -> Result<Supervision, String> {
    let mut cmd = build_command(cfg);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to start '{}': {e}", cfg.program))?;
    let pid_raw = child.id();
    let pid = Pid::from_u32(pid_raw);

    let out_on = stdout_color();
    let err_on = stderr_color();
    let tag_out = paint(out_on, "36", "[mknight]");

    println!("{tag_out} spawned '{}' (pid {pid_raw})", cfg.program);
    if cfg.wall_active() {
        println!(
            "{tag_out} memory wall active: RLIMIT_AS = {}",
            format_size(cfg.max_ram)
        );
    }

    let mut sys = System::new();
    let start = Instant::now();
    let mut stats = RunStats::default();
    let mut last_sample: Option<(Instant, u64)> = None;
    let mut last_status = start;
    let mut warned = false;

    let outcome = loop {
        // Did the child exit on its own since the last tick?
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("failed waiting on child (pid {pid_raw}): {e}"))?
        {
            break Outcome::Exited(status);
        }

        // Sample this PID's resident memory.
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );

        if let Some(proc) = sys.process(pid) {
            let now = Instant::now();
            let rss = proc.memory();
            stats.peak_rss = stats.peak_rss.max(rss);

            let mut velocity = 0.0;
            if let Some((t_prev, rss_prev)) = last_sample {
                let dt = now.duration_since(t_prev).as_secs_f64();
                if dt > 0.0 && rss > rss_prev {
                    velocity = (rss - rss_prev) as f64 / dt;
                    if velocity > stats.peak_velocity {
                        stats.peak_velocity = velocity;
                    }
                }
            }
            last_sample = Some((now, rss));

            // Evaluate the two heuristics. Absolute limit takes priority.
            let violation = if rss > cfg.max_ram {
                Some(Violation::AbsoluteLimit)
            } else if velocity > cfg.max_velocity as f64 {
                Some(Violation::Velocity)
            } else {
                None
            };

            if let Some(v) = violation {
                if cfg.report_only {
                    if !warned {
                        eprintln!(
                            "{} ⚠ would terminate: {} (report-only)",
                            paint(err_on, "33", "[mknight]"),
                            v.reason()
                        );
                        warned = true;
                    }
                } else {
                    terminate(&mut child).map_err(|e| {
                        format!("failed to terminate child (pid {pid_raw}): {e}")
                    })?;
                    break Outcome::Killed(v);
                }
            }

            // Throttled live status (~4 Hz) so we don't flood the terminal.
            if now.duration_since(last_status) >= Duration::from_millis(250) {
                eprintln!(
                    "{} live · rss {} · peak {} · vel {}/s",
                    paint(err_on, "36", "[mknight]"),
                    format_size(rss),
                    format_size(stats.peak_rss),
                    format_size(stats.peak_velocity as u64),
                );
                last_status = now;
            }
        }

        thread::sleep(cfg.interval);
    };

    stats.duration = start.elapsed();
    Ok(Supervision { outcome, stats })
}

/// Terminate the child and reap it, tolerating the race where it exits on its
/// own between our last check and the kill.
fn terminate(child: &mut Child) -> io::Result<()> {
    // `kill` errors with InvalidInput if the child already exited — harmless.
    match child.kill() {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::InvalidInput => {}
        Err(e) => return Err(e),
    }
    child.wait()?; // reap the zombie regardless
    Ok(())
}
