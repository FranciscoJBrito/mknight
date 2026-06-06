//! Terminal color helpers that honor `NO_COLOR` and TTY detection.
//!
//! Color is emitted only when `NO_COLOR` is unset *and* the relevant stream is
//! an interactive terminal. stdout and stderr are checked independently so a
//! redirected log never gets escape codes.

use std::io::IsTerminal;

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

/// Whether color should be used on stdout (the report card, status lines).
pub fn stdout_color() -> bool {
    !no_color() && std::io::stdout().is_terminal()
}

/// Whether color should be used on stderr (the live diagnostics).
pub fn stderr_color() -> bool {
    !no_color() && std::io::stderr().is_terminal()
}

/// Wrap `text` in an ANSI SGR `code` when `on`, otherwise return it plain.
pub fn paint(on: bool, code: &str, text: &str) -> String {
    if on {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
