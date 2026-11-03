//! Input normalisation shared by the order and rider forms.

/// Normalises a Nigerian mobile number to the local 11-digit form ("08031234567").
/// Accepts spaces and dashes, and the +234 / 234 international prefix.
pub fn normalize_phone(input: &str) -> Option<String> {
    let digits: String = input
        .trim()
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')'))
        .collect();
    let local = if let Some(rest) = digits.strip_prefix("+234") {
        format!("0{rest}")
    } else if let Some(rest) = digits.strip_prefix("234").filter(|r| r.len() == 10) {
        format!("0{rest}")
    } else {
        digits
    };
    let bytes = local.as_bytes();
    let valid = bytes.len() == 11
        && local.chars().all(|c| c.is_ascii_digit())
        && bytes[0] == b'0'
        && matches!(bytes[1], b'7' | b'8' | b'9')
        && matches!(bytes[2], b'0' | b'1');
    valid.then_some(local)
}

/// Trims and collapses inner whitespace; returns None for blank input.
pub fn clean_text(input: &str) -> Option<String> {
    let text = input.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

pub fn parse_distance(input: &str) -> Option<f64> {
    let km: f64 = input.trim().replace(',', ".").parse().ok()?;
    (km > 0.0 && km <= 80.0).then_some(km)
}
