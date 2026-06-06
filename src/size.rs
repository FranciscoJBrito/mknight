//! Human-readable byte-size parsing and formatting.
//!
//! Uses binary multipliers (1 KB = 1024 B), which match how memory is reported
//! by the OS and how students tend to reason about RAM.

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
const GIB: u64 = 1024 * MIB;

/// Parse a human size string like `"500MB"`, `"1.5GB"`, `"1024"`, `"2g"` into bytes.
///
/// Accepted units (case-insensitive): `B`, `K`/`KB`/`KIB`, `M`/`MB`/`MIB`,
/// `G`/`GB`/`GIB`. A bare number is interpreted as bytes.
pub fn parse_size(input: &str) -> Result<u64, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty size value".to_string());
    }

    // Split into the leading numeric part and the trailing unit.
    let split = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (num_part, unit_part) = s.split_at(split);

    let value: f64 = num_part
        .parse()
        .map_err(|_| format!("invalid number in size '{input}'"))?;
    if value < 0.0 || !value.is_finite() {
        return Err(format!("size must be a positive number: '{input}'"));
    }

    let multiplier = match unit_part.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" | "KIB" => KIB,
        "M" | "MB" | "MIB" => MIB,
        "G" | "GB" | "GIB" => GIB,
        other => return Err(format!("unknown size unit '{other}' in '{input}'")),
    };

    Ok((value * multiplier as f64).round() as u64)
}

/// Format a byte count into a compact human string, e.g. `2.41 GB`.
pub fn format_size(bytes: u64) -> String {
    let b = bytes as f64;
    if bytes >= GIB {
        format!("{:.2} GB", b / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MB", b / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KB", b / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_bytes() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
    }

    #[test]
    fn parses_units() {
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("500MB").unwrap(), 500 * 1024 * 1024);
    }

    #[test]
    fn is_case_and_whitespace_insensitive() {
        assert_eq!(parse_size(" 2g ").unwrap(), 2 * GIB);
        assert_eq!(parse_size("2GiB").unwrap(), 2 * GIB);
    }

    #[test]
    fn parses_fractional() {
        assert_eq!(parse_size("1.5GB").unwrap(), (1.5 * GIB as f64) as u64);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("10XB").is_err());
    }

    #[test]
    fn formats_roundish() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(2 * MIB), "2.00 MB");
        assert_eq!(format_size(GIB + GIB / 2), "1.50 GB");
    }
}
