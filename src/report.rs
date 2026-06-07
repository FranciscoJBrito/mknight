use std::process::ExitStatus;

use crate::config::Config;
use crate::monitor::{Outcome, Supervision, Violation};
use crate::size::format_size;
use crate::term::{paint, stdout_color};

const BANNER: &str = r"  __  __ _          _       _     _     _____                       _
 |  \/  | |        (_)     | |   | |   |  __ \                     | |
 | \  / | | ___ __  _  __ _| |__ | |_  | |__) |___ _ __   ___  _ __| |_
 | |\/| | |/ / '_ \| |/ _` | '_ \| __| |  _  // _ \ '_ \ / _ \| '__| __|
 | |  | |   <| | | | | (_| | | | | |_  | | \ \  __/ |_) | (_) | |  | |_
 |_|  |_|_|\_\_| |_|_|\__, |_| |_|\__| |_|  \_\___| .__/ \___/|_|   \__|
                       __/ |                      | |
                      |___/                       |_|";

pub fn render(cfg: &Config, sup: &Supervision) -> i32 {
    let on = stdout_color();
    match &sup.outcome {
        Outcome::Killed(v) => {
            render_kill(cfg, sup, *v, on);
            137
        }
        Outcome::Exited(status) => render_exit(cfg, sup, status, on),
    }
}

fn render_kill(cfg: &Config, sup: &Supervision, v: Violation, on: bool) {
    let s = &sup.stats;
    let rule = "=".repeat(70);

    println!();
    println!("{}", paint(on, "1;31", BANNER));
    println!();
    println!(
        "{}",
        paint(on, "1;31", "[!] PROCESS TERMINATED TO SAVE YOUR SYSTEM")
    );
    println!();

    println!("The program `{}` was aborted because it violated", cfg.program);
    println!(
        "the safety policy: {}.",
        paint(on, "33", &format!("[{}]", v.reason()))
    );
    println!();

    println!("{}", paint(on, "1", "[*] Post-Mortem Analytics"));
    println!(
        "    - Total Execution Time : {:.2} seconds",
        s.duration.as_secs_f64()
    );
    println!(
        "    - Peak RAM Consumed    : {} (limit {})",
        format_size(s.peak_rss),
        format_size(cfg.max_ram)
    );
    println!(
        "    - Allocation Velocity  : ~{}/sec ({})",
        format_size(s.peak_velocity as u64),
        velocity_label(s.peak_velocity)
    );
    println!();

    println!("{}", paint(on, "1", "[i] System Guard Insight"));
    for line in insight_lines(v, cfg) {
        println!("    {line}");
    }
    println!("{}", paint(on, "1;31", &rule));
}

fn render_exit(cfg: &Config, sup: &Supervision, status: &ExitStatus, on: bool) -> i32 {
    match status.code() {
        Some(0) => {
            println!(
                "\n{}",
                paint(on, "1;32", "[OK] mknight - program completed cleanly.")
            );
            print_stats_line(sup, on);
            0
        }
        Some(code) => {
            println!(
                "\n{}",
                paint(
                    on,
                    "1;33",
                    &format!("[!] mknight - program exited with error code {code}.")
                )
            );
            print_stats_line(sup, on);
            println!(
                "  {} the non-zero exit code came from your program, not from mknight.",
                paint(on, "2", "note:")
            );
            code
        }
        None => render_signal(cfg, sup, status, on),
    }
}

fn render_signal(cfg: &Config, sup: &Supervision, status: &ExitStatus, on: bool) -> i32 {
    let sig = signal_of(status);

    let sig_name = sig.map(signal_name).unwrap_or("unknown");
    let sig_num = sig.map(|s| s.to_string()).unwrap_or_else(|| "?".to_string());

    println!(
        "\n{}",
        paint(
            on,
            "1;31",
            &format!("[X] mknight - program crashed (signal {sig_num} - {sig_name}).")
        )
    );
    print_stats_line(sup, on);

    if sig == Some(11) {
        println!("  {}", paint(on, "1", "[i] System Guard Insight"));
        println!("  A segmentation fault usually means your program read or wrote");
        println!("  memory it doesn't own.");
        if cfg.wall_active() {
            println!("  Because the memory wall is active, malloc/calloc start returning");
            println!(
                "  NULL once the {} cap is hit - dereferencing that NULL without",
                format_size(cfg.max_ram)
            );
            println!("  checking it causes exactly this crash. Always check malloc's result.");
        }
    }

    sig.map(|s| 128 + s).unwrap_or(1)
}

fn print_stats_line(sup: &Supervision, on: bool) {
    let s = &sup.stats;
    let line = format!(
        "peak RAM {} | peak velocity {}/s | runtime {:.2}s",
        format_size(s.peak_rss),
        format_size(s.peak_velocity as u64),
        s.duration.as_secs_f64(),
    );
    println!("  {}", paint(on, "2", &line));
}

fn velocity_label(bytes_per_sec: f64) -> &'static str {
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * MB;
    if bytes_per_sec >= GB {
        "Severe Leak"
    } else if bytes_per_sec >= 100.0 * MB {
        "Rapid Growth"
    } else {
        "Moderate"
    }
}

fn insight_lines(v: Violation, cfg: &Config) -> Vec<String> {
    match v {
        Violation::AbsoluteLimit => vec![
            format!(
                "Your program crossed the {} memory ceiling and kept climbing.",
                format_size(cfg.max_ram)
            ),
            "Look for allocations (malloc/calloc/new) inside loops that never".to_string(),
            "free() what they take, or a structure that grows without bound.".to_string(),
        ],
        Violation::Velocity => vec![
            "Your program allocated memory explosively - a near-vertical curve.".to_string(),
            "This almost always means a loop calling malloc() or calloc() with".to_string(),
            "no matching free() and no exit condition. Check the bounds of your".to_string(),
            "while/for loops and make every allocation be released.".to_string(),
        ],
    }
}

fn signal_of(status: &ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}

fn signal_name(sig: i32) -> &'static str {
    match sig {
        2 => "SIGINT",
        4 => "SIGILL",
        6 => "SIGABRT",
        8 => "SIGFPE",
        9 => "SIGKILL",
        11 => "SIGSEGV",
        15 => "SIGTERM",
        _ => "signal",
    }
}
