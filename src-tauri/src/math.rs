use meval;

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

    match meval::eval_str(trimmed) {
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
