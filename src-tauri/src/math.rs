use meval;
use regex::Regex;
use std::sync::LazyLock;

static BASE_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b0x[0-9a-f_]+\b|\b0b[01_]+\b|\b0o[0-7_]+\b")
        .expect("base literal regex must be valid")
});

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
    use super::evaluate;

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
}
