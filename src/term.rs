use std::io::IsTerminal;

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

pub fn stdout_color() -> bool {
    !no_color() && std::io::stdout().is_terminal()
}

pub fn stderr_color() -> bool {
    !no_color() && std::io::stderr().is_terminal()
}

pub fn paint(on: bool, code: &str, text: &str) -> String {
    if on {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
