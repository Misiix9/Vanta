use chrono::Utc;
use chrono_tz::Tz;
use meval;
use regex::Regex;
use std::sync::LazyLock;

static BASE_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b0x[0-9a-f_]+\b|\b0b[01_]+\b|\b0o[0-7_]+\b")
        .expect("base literal regex must be valid")
});

static UNIT_CONVERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*([-+]?[0-9]*\.?[0-9]+)\s*(km|mi|kg|lb)\s*(?:to|in)?\s*(km|mi|kg|lb)\s*$")
        .expect("unit conversion regex must be valid")
});

static TIMEZONE_QUERY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(?:time(?:zone)?(?:\s+in)?|tz)\s+([a-zA-Z0-9_./\- ]+)\s*$")
        .expect("timezone regex must be valid")
});

fn convert_units(value: f64, from: &str, to: &str) -> Option<f64> {
    match (from, to) {
        ("km", "mi") => Some(value * 0.621_371),
        ("mi", "km") => Some(value / 0.621_371),
        ("kg", "lb") => Some(value * 2.204_622_621_8),
        ("lb", "kg") => Some(value / 2.204_622_621_8),
        _ => None,
    }
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        format!("{:.6}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn resolve_timezone_alias(raw: &str) -> Option<Tz> {
    let key = raw.trim().to_ascii_lowercase();
    let alias = match key.as_str() {
        "utc" | "gmt" => "UTC",
        "london" => "Europe/London",
        "new york" | "nyc" => "America/New_York",
        "los angeles" | "la" => "America/Los_Angeles",
        "san francisco" | "sf" => "America/Los_Angeles",
        "chicago" => "America/Chicago",
        "tokyo" => "Asia/Tokyo",
        "seoul" => "Asia/Seoul",
        "singapore" => "Asia/Singapore",
        "shanghai" => "Asia/Shanghai",
        "hong kong" => "Asia/Hong_Kong",
        "delhi" | "india" | "ist" => "Asia/Kolkata",
        "dubai" => "Asia/Dubai",
        "berlin" => "Europe/Berlin",
        "paris" => "Europe/Paris",
        "sydney" => "Australia/Sydney",
        _ => return None,
    };
    alias.parse::<Tz>().ok()
}

fn resolve_timezone(raw: &str) -> Option<Tz> {
    if let Some(tz) = resolve_timezone_alias(raw) {
        return Some(tz);
    }

    let cleaned = raw.trim().replace(' ', "_");
    cleaned.parse::<Tz>().ok()
}

pub fn evaluate_timezone_display(query: &str) -> Option<(String, String)> {
    let caps = TIMEZONE_QUERY_RE.captures(query.trim())?;
    let target = caps.get(1)?.as_str().trim();
    let tz = resolve_timezone(target)?;

    let now = Utc::now().with_timezone(&tz);
    let value = now.format("%Y-%m-%d %H:%M %Z").to_string();
    let display = format!("{} ({})", value, tz.name());
    Some((display.clone(), value))
}

pub fn evaluate_display(expression: &str) -> Option<(String, String)> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(caps) = UNIT_CONVERSION_RE.captures(trimmed) {
        let raw_value = caps.get(1)?.as_str().parse::<f64>().ok()?;
        let from = caps.get(2)?.as_str().to_ascii_lowercase();
        let to = caps.get(3)?.as_str().to_ascii_lowercase();
        let converted = convert_units(raw_value, &from, &to)?;
        let display = format!("{} {}", format_number(converted), to);
        return Some((display.clone(), display));
    }

    let val = evaluate(trimmed)?;
    let out = format_number(val);
    Some((out.clone(), out))
}

fn parse_base_literal(token: &str) -> Option<u64> {
    let lower = token.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("0x") {
        let cleaned = rest.replace('_', "");
        return u64::from_str_radix(&cleaned, 16).ok();
    }
    if let Some(rest) = lower.strip_prefix("0b") {
        let cleaned = rest.replace('_', "");
        return u64::from_str_radix(&cleaned, 2).ok();
    }
    if let Some(rest) = lower.strip_prefix("0o") {
        let cleaned = rest.replace('_', "");
        return u64::from_str_radix(&cleaned, 8).ok();
    }
    None
}

fn normalize_base_literals(expression: &str) -> Option<String> {
    let mut normalized = String::with_capacity(expression.len());
    let mut last = 0usize;

    for m in BASE_LITERAL_RE.find_iter(expression) {
        normalized.push_str(&expression[last..m.start()]);
        let value = parse_base_literal(m.as_str())?;
        normalized.push_str(&value.to_string());
        last = m.end();
    }

    normalized.push_str(&expression[last..]);
    Some(normalized)
}

/// Evaluates a mathematical expression string.
/// Returns Some(result) if the expression is valid and calculation succeeds.
/// Returns None if the expression is invalid or empty.
///
/// This function performs a pre-check to ensure the query looks like math
/// to avoid false positives on normal text searches.
pub fn evaluate(expression: &str) -> Option<f64> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return None;
    }

    // specific check: must contain at least one operator/number?
    // or just rely on meval's parser.
    // To avoid "vanta" being parsed as variables, we can restrict allowed chars
    // to typical math symbols.
    // Allowed: 0-9, +, -, *, /, %, ^, (, ), ., and spaces.
    // Also e, pi (constants) but maybe we want to be strict.

    // Simple heuristic: if it contains letters other than e/pi/sin/cos/etc, ignore.
    // But meval handles functions.
    // Let's just try to evaluate it. If it errors, return None.

    // However, meval treats words as variables. "test" might error "unknown variable".
    // "pi" works.

    // Critical: Filter out queries that are obviously not math.
    // If the query has no digits, it's probably not a calculation we want to show.
    if !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let normalized = normalize_base_literals(trimmed)?;

    match meval::eval_str(&normalized) {
        Ok(val) => {
            // Filter out infinite or NaN results
            if val.is_infinite() || val.is_nan() {
                None
            } else {
                Some(val)
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, evaluate_display, evaluate_timezone_display, resolve_timezone};

    #[test]
    fn evaluates_hex_literal() {
        assert_eq!(evaluate("0xff"), Some(255.0));
    }

    #[test]
    fn evaluates_mixed_base_expression() {
        assert_eq!(evaluate("0b1010 + 0o7"), Some(17.0));
    }

    #[test]
    fn evaluates_base_literals_with_separators() {
        assert_eq!(evaluate("0xFF_FF - 1"), Some(65534.0));
    }

    #[test]
    fn rejects_invalid_base_literals() {
        assert_eq!(evaluate("0b102"), None);
    }

    #[test]
    fn converts_km_to_mi() {
        let out = evaluate_display("10 km to mi").expect("conversion result");
        assert_eq!(out.0, "6.21371 mi");
    }

    #[test]
    fn converts_lb_to_kg_without_to_keyword() {
        let out = evaluate_display("10lb kg").expect("conversion result");
        assert_eq!(out.0, "4.535924 kg");
    }

    #[test]
    fn evaluate_display_falls_back_to_math() {
        let out = evaluate_display("2 + 2").expect("math result");
        assert_eq!(out.0, "4");
    }

    #[test]
    fn resolves_timezone_alias() {
        let tz = resolve_timezone("tokyo").expect("timezone alias");
        assert_eq!(tz.name(), "Asia/Tokyo");
    }

    #[test]
    fn resolves_timezone_iana_with_spaces() {
        let tz = resolve_timezone("America/New York").expect("timezone id");
        assert_eq!(tz.name(), "America/New_York");
    }

    #[test]
    fn evaluates_timezone_display_query() {
        let out = evaluate_timezone_display("time in tokyo").expect("timezone output");
        assert!(out.0.contains("Asia/Tokyo"));
        assert!(!out.1.is_empty());
    }
}
