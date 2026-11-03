mod common;

use dispatchly::services::pricing::{BRIDGE_FEE_KOBO, EXPRESS_FEE_KOBO, QuoteError, TariffCache, update_zone_fee};

#[test]
fn same_side_short_trip_is_zone_fee_only() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let q = t.quote(common::area(&t, "Yaba"), common::area(&t, "Surulere"), 3.4, false).unwrap();
    assert_eq!(q.zone_fee_kobo, 180_000);
    assert_eq!(q.band_fee_kobo, 0);
    assert_eq!(q.bridge_fee_kobo, 0);
    assert_eq!(q.total_kobo, 180_000);
}

#[test]
fn crossing_to_the_island_adds_the_bridge_fee() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let q = t.quote(common::area(&t, "Yaba"), common::area(&t, "Ikoyi"), 12.0, false).unwrap();
    assert_eq!(q.zone_name, "Island");
    assert_eq!(q.band_label, "10 to 20 km");
    assert_eq!(q.bridge_fee_kobo, BRIDGE_FEE_KOBO);
    assert_eq!(q.total_kobo, 250_000 + 120_000 + BRIDGE_FEE_KOBO);
}

#[test]
fn express_and_long_distance() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let q = t.quote(common::area(&t, "Ikorodu"), common::area(&t, "Ojo"), 41.0, true).unwrap();
    assert_eq!(q.band_label, "Over 35 km");
    assert_eq!(q.total_kobo, 220_000 + 350_000 + EXPRESS_FEE_KOBO);
}

#[test]
fn band_lower_bound_is_inclusive() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    assert_eq!(t.band_for(4.9).unwrap().label, "Up to 5 km");
    assert_eq!(t.band_for(5.0).unwrap().label, "5 to 10 km");
    assert_eq!(t.band_for(10.0).unwrap().label, "10 to 20 km");
}

#[test]
fn unknown_area_is_rejected() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    assert_eq!(t.quote(9_999, 1, 3.0, false), Err(QuoteError::UnknownArea));
}

#[test]
fn cache_reloads_after_invalidate() {
    let conn = common::seeded();
    let cache = TariffCache::default();
    let before = cache.get(&conn).unwrap();
    let zone = before.zones.iter().find(|z| z.code == "A").unwrap().id;
    update_zone_fee(&conn, zone, 200_000).unwrap();
    cache.invalidate();
    let after = cache.get(&conn).unwrap();
    assert_eq!(after.zones.iter().find(|z| z.id == zone).unwrap().base_fee_kobo, 200_000);
}
