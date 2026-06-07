mod cli;
mod config;
mod limits;
mod monitor;
mod report;
mod size;
mod term;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::size::format_size;
use crate::term::{paint, stdout_color};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run(args) => Config::from_args(args).and_then(run),
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            let label = paint(term::stderr_color(), "31", "[mknight] error:");
            eprintln!("{label} {e}");
            std::process::exit(1);
        }
    }
}

fn run(cfg: Config) -> Result<i32, String> {
    let on = stdout_color();
    println!(
        "{} guarding '{}'",
        paint(on, "36", "[mknight]"),
        cfg.program
    );
    println!(
        "  max-ram {} | velocity {}/s | sample {} ms{}",
        format_size(cfg.max_ram),
        format_size(cfg.max_velocity),
        cfg.interval.as_millis(),
        if cfg.report_only {
            " · report-only"
        } else {
            ""
        }
    );

    let sup = monitor::supervise(&cfg)?;
    Ok(report::render(&cfg, &sup))
}
