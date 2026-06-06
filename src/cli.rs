use clap::{Args, Parser, Subcommand};

/// MemoryKnight — a user-space sandbox watcher that guards programs against
/// runaway memory allocations (e.g. infinite `malloc` loops in C/C++).
#[derive(Parser, Debug)]
#[command(name = "mknight", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a program under MemoryKnight's protection.
    Run(RunArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Absolute memory ceiling. Also sets RLIMIT_AS on Linux. e.g. 1GB, 500MB.
    #[arg(long, value_name = "SIZE", default_value = "1GB")]
    pub max_ram: String,

    /// Explosive-growth threshold per second, e.g. 5GB. Defaults to ~500MB/100ms.
    #[arg(long, value_name = "SIZE")]
    pub max_velocity: Option<String>,

    /// Memory sampling interval, in milliseconds.
    #[arg(long, value_name = "MS", default_value_t = 50)]
    pub interval: u64,

    /// Monitor and report, but never kill the child process.
    #[arg(long)]
    pub report_only: bool,

    /// Skip the Linux setrlimit wall (Layer 1); rely on monitoring only.
    #[arg(long)]
    pub no_rlimit: bool,

    /// The program to run.
    #[arg(value_name = "PROGRAM")]
    pub program: String,

    /// Arguments forwarded to the program (everything after PROGRAM).
    #[arg(value_name = "ARGS", trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}
