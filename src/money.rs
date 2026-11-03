//! Money is kept in kobo (1 naira = 100 kobo) as i64 everywhere.

/// 1250000 -> "₦12,500.00"
pub fn format_naira(kobo: i64) -> String {
    let sign = if kobo < 0 { "-" } else { "" };
    let abs = kobo.unsigned_abs();
    let naira = abs / 100;
    let rest = abs % 100;
    format!("{sign}₦{}.{rest:02}", group_thousands(naira))
}

/// 180000 -> "1800.00", for form inputs.
pub fn plain_naira(kobo: i64) -> String {
    format!("{}.{:02}", kobo / 100, (kobo % 100).abs())
}

fn group_thousands(mut value: u64) -> String {
    let mut groups = Vec::new();
    loop {
        if value < 1000 {
            groups.push(value.to_string());
            break;
        }
        groups.push(format!("{:03}", value % 1000));
        value /= 1000;
    }
    groups.reverse();
    groups.join(",")
}

/// Parses an amount typed by a person: "12500", "12,500.50", "₦ 12,500". Empty means zero.
pub fn parse_naira(input: &str) -> Option<i64> {
    let cleaned: String = input
        .trim()
        .chars()
        .filter(|c| !matches!(c, ',' | ' ' | '₦'))
        .collect();
    if cleaned.is_empty() {
        return Some(0);
    }
    let (whole, fraction) = match cleaned.split_once('.') {
        Some((w, f)) => (w, f),
        None => (cleaned.as_str(), ""),
    };
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole.chars().all(|c| c.is_ascii_digit()) || !fraction.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if fraction.len() > 2 {
        return None;
    }
    let naira: i64 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let kobo: i64 = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().ok()? * 10,
        _ => fraction.parse().ok()?,
    };
    naira.checked_mul(100)?.checked_add(kobo)
}
