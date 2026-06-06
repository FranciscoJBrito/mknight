use std::time::Duration;

use crate::cli::RunArgs;
use crate::size::parse_size;

pub const DEFAULT_MAX_VELOCITY_BYTES_PER_SEC: u64 = 500 * 1024 * 1024 * 10;

#[derive(Debug, Clone)]
pub struct Config {
    pub program: String,
    pub args: Vec<String>,
    pub max_ram: u64,
    pub max_velocity: u64,
    pub interval: Duration,
    pub report_only: bool,
    pub rlimit_requested: bool,
}

impl Config {
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

    pub fn wall_active(&self) -> bool {
        self.rlimit_requested && cfg!(target_os = "linux")
    }
}
