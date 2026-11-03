use dispatchly::{money, validate};

#[test]
fn formats_naira_with_grouping_and_kobo() {
    assert_eq!(money::format_naira(0), "₦0.00");
    assert_eq!(money::format_naira(1_250_000), "₦12,500.00");
    assert_eq!(money::format_naira(123_456_789), "₦1,234,567.89");
    assert_eq!(money::format_naira(-50_000), "-₦500.00");
    assert_eq!(money::plain_naira(180_050), "1800.50");
}

#[test]
fn parses_amounts_people_type() {
    assert_eq!(money::parse_naira("12500"), Some(1_250_000));
    assert_eq!(money::parse_naira("12,500.5"), Some(1_250_050));
    assert_eq!(money::parse_naira("₦ 1,000.50"), Some(100_050));
    assert_eq!(money::parse_naira(""), Some(0));
    assert_eq!(money::parse_naira("12.345"), None);
    assert_eq!(money::parse_naira("abc"), None);
}

#[test]
fn normalises_nigerian_mobile_numbers() {
    assert_eq!(validate::normalize_phone("0803 123 4567").as_deref(), Some("08031234567"));
    assert_eq!(validate::normalize_phone("+234 816 442 0981").as_deref(), Some("08164420981"));
    assert_eq!(validate::normalize_phone("2349012239876").as_deref(), Some("09012239876"));
    assert_eq!(validate::normalize_phone("0706-551-2398").as_deref(), Some("07065512398"));
    assert_eq!(validate::normalize_phone("0123456789"), None);
    assert_eq!(validate::normalize_phone("08031234"), None);
    assert_eq!(validate::normalize_phone("06031234567"), None);
}

#[test]
fn distance_must_be_sensible() {
    assert_eq!(validate::parse_distance("12.5"), Some(12.5));
    assert_eq!(validate::parse_distance("7,2"), Some(7.2));
    assert_eq!(validate::parse_distance("0"), None);
    assert_eq!(validate::parse_distance("120"), None);
}
