//! Resolved, validated runtime configuration derived from CLI args.

use std::time::Duration;

use crate::cli::RunArgs;
use crate::size::parse_size;

/// Default explosive-growth threshold: 500 MB per 100 ms window == 5000 MB/s.
pub const DEFAULT_MAX_VELOCITY_BYTES_PER_SEC: u64 = 500 * 1024 * 1024 * 10;

/// Fully resolved configuration for a single `run` invocation.
#[derive(Debug, Clone)]
pub struct Config {
    pub program: String,
    pub args: Vec<String>,
    /// Absolute RSS ceiling, in bytes.
    pub max_ram: u64,
    /// Explosive-growth threshold, in bytes per second.
    pub max_velocity: u64,
    /// Memory sampling interval.
    pub interval: Duration,
    /// Monitor only; never kill the child.
    pub report_only: bool,
    /// Whether the user asked for the Layer 1 setrlimit wall (default true).
    pub rlimit_requested: bool,
}

impl Config {
    /// Validate and resolve raw CLI args into a usable [`Config`].
    pub fn from_args(args: RunArgs) -> Result<Self, String> {
        let max_ram = parse_size(&args.max_ram)?;
        if max_ram == 0 {
            return Err("--max-ram must be greater than zero".to_string());
        }

        let max_velocity = match args.max_velocity {
            Some(ref s) => parse_size(s)?,
            None => DEFAULT_MAX_VELOCITY_BYTES_PER_SEC,
        };

        if args.interval == 0 {
            return Err("--interval must be at least 1 ms".to_string());
        }

        Ok(Self {
            program: args.program,
            args: args.args,
            max_ram,
            max_velocity,
            interval: Duration::from_millis(args.interval),
            report_only: args.report_only,
            rlimit_requested: !args.no_rlimit,
        })
    }

    /// Whether Layer 1 (the `setrlimit` wall) will actually be active here.
    /// Requested by the user *and* on a platform that enforces it (Linux).
    pub fn wall_active(&self) -> bool {
        self.rlimit_requested && cfg!(target_os = "linux")
    }
}
