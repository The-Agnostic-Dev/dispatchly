mod common;

use dispatchly::{
    clock,
    services::{
        board, dispatch,
        orders::{self, Action, OrderInput},
        remittance, riders,
    },
};

fn input(t: &dispatchly::services::pricing::Tariff) -> OrderInput {
    OrderInput {
        merchant_name: "Ronke's Closet".into(),
        merchant_phone: "08023341190".into(),
        pickup_area_id: common::area(t, "Yaba"),
        pickup_address: "14 Herbert Macaulay Way".into(),
        customer_name: "Bisi Adewale".into(),
        customer_phone: "08031234567".into(),
        dropoff_area_id: common::area(t, "Victoria Island"),
        dropoff_address: "Plot 5, Adeola Odeku".into(),
        distance_km: 13.0,
        express: false,
        cod_kobo: 2_000_000,
        notes: String::new(),
    }
}

#[test]
fn new_order_gets_reference_fee_and_timeline() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let id = orders::create(&conn, &t, &input(&t), "Tolulope Adebayo").unwrap();
    let order = orders::get(&conn, id).unwrap();
    assert_eq!(order.reference, orders::reference_for(id));
    assert_eq!(order.status, "pending");
    assert_eq!(order.fee_kobo, 250_000 + 120_000 + 50_000);
    let events = orders::timeline(&conn, id).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].note, "Order received");
}

#[test]
fn rider_flow_records_cash_collected() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let id = orders::create(&conn, &t, &input(&t), "Tolulope Adebayo").unwrap();
    let musa = common::rider_id(&conn, "Musa Abdullahi");
    conn.execute("UPDATE orders SET rider_id = ?1, status = 'assigned' WHERE id = ?2", (musa, id)).unwrap();

    assert!(orders::transition(&conn, id, Action::PickedUp, None, "Musa Abdullahi").unwrap());
    assert!(orders::transition(&conn, id, Action::Delivered, Some(1_950_000), "Musa Abdullahi").unwrap());
    let order = orders::get(&conn, id).unwrap();
    assert_eq!(order.status, "delivered");
    assert_eq!(order.cash_collected_kobo, 1_950_000);
    assert_eq!(orders::timeline(&conn, id).unwrap().len(), 3);
}

#[test]
fn cancelled_orders_cannot_be_picked_up() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let id = orders::create(&conn, &t, &input(&t), "Ngozi Eze").unwrap();
    assert!(orders::transition(&conn, id, Action::Cancel, None, "Ngozi Eze").unwrap());
    assert!(!orders::transition(&conn, id, Action::PickedUp, None, "Ngozi Eze").unwrap());
    assert_eq!(orders::get(&conn, id).unwrap().status, "cancelled");
}

#[test]
fn editing_order_recomputes_fees_from_pricing() {
    let conn = common::seeded();
    let t = common::tariff(&conn);
    let id = orders::create(&conn, &t, &input(&t), "Tolulope Adebayo").unwrap();

    let mut edited = input(&t);
    edited.dropoff_area_id = common::area(&t, "Ajah");
    edited.distance_km = 23.0;
    edited.express = true;

    orders::update(&conn, &t, id, &edited, "Tolulope Adebayo").unwrap();

    let order = orders::get(&conn, id).unwrap();
    let expected = t.quote(order.pickup_area_id, order.dropoff_area_id, order.distance_km, order.express).unwrap();
    assert_eq!(order.zone_fee_kobo, expected.zone_fee_kobo);
    assert_eq!(order.band_fee_kobo, expected.band_fee_kobo);
    assert_eq!(order.bridge_fee_kobo, expected.bridge_fee_kobo);
    assert_eq!(order.express_fee_kobo, expected.express_fee_kobo);
    assert_eq!(order.fee_kobo, expected.total_kobo);
}

#[test]
fn order_detail_shows_customers_own_phone() {
    let conn = common::seeded();
    let id: i64 = conn
        .query_row("SELECT id FROM orders WHERE customer_name = 'Adaeze Nwankwo'", [], |r| r.get(0))
        .unwrap();
    let order = orders::get(&conn, id).unwrap();
    assert_eq!(order.reference, "DL-24002");
    let user = dispatchly::auth::authenticate(&conn, "tolu", dispatchly::seed::DEMO_PASSWORD)
        .unwrap()
        .expect("seeded dispatcher");
    let env = dispatchly::views::environment(std::path::Path::new("templates"));
    let template = env.get_template("order_detail.html").unwrap();
    let html = template
        .render(minijinja::context! { order => &order, user => &user, flash => Option::<dispatchly::auth::Flash>::None })
        .unwrap();
    assert!(html.contains(&order.customer_phone));
    let customer_card = html
        .split("data-testid=\"customer\"")
        .nth(1)
        .expect("customer card")
        .split("data-testid=")
        .nth(1)
        .map(|_| html.split("data-testid=\"customer\"").nth(1).unwrap())
        .unwrap_or_default();
    let customer_card = customer_card.split("data-testid=").next().unwrap_or(customer_card);
    assert!(customer_card.contains(&order.customer_phone));
    assert!(!customer_card.contains(&order.merchant_phone));
}

#[test]
fn board_lists_each_column() {
    let conn = common::seeded();
    let b = board::load(&conn, clock::today(), "").unwrap();
    assert_eq!(b.waiting.len(), 5);
    assert_eq!(b.on_the_road.len(), 7);
    let found = board::load(&conn, clock::today(), "Adaeze").unwrap();
    assert_eq!(found.waiting.len(), 1);
    assert_eq!(found.waiting[0].customer_name, "Adaeze Nwankwo");
}

#[tokio::test]
async fn assigning_respects_rider_limit() {
    let state = common::state();
    let (first, second, sule) = {
        let conn = state.db();
        let t = common::tariff(&conn);
        let a = orders::create(&conn, &t, &input(&t), "Tolulope Adebayo").unwrap();
        let b = orders::create(&conn, &t, &input(&t), "Tolulope Adebayo").unwrap();
        (a, b, common::rider_id(&conn, "Sule Ibrahim"))
    };
    let tunde = common::rider_id(&state.db(), "Tunde Bello");

    assert_eq!(dispatch::assign(&state, first, sule, "Tolulope Adebayo").await.unwrap(), "Sule Ibrahim");
    assert!(matches!(
        dispatch::assign(&state, second, tunde, "Tolulope Adebayo").await,
        Err(dispatch::AssignError::RiderFull { .. })
    ));
    assert!(matches!(
        dispatch::assign(&state, first, sule, "Tolulope Adebayo").await,
        Err(dispatch::AssignError::NotWaiting(_))
    ));
    assert_eq!(riders::active_jobs(&state.db(), sule).unwrap(), 1);
}

#[test]
fn remittance_sheet_tracks_what_was_handed_in() {
    let conn = common::seeded();
    let yesterday = clock::today() - chrono::Duration::days(1);
    let musa = common::rider_id(&conn, "Musa Abdullahi");
    let row = |conn: &rusqlite::Connection| {
        remittance::day_sheet(conn, yesterday).unwrap().into_iter().find(|r| r.rider_id == musa).unwrap()
    };
    let before = row(&conn);
    assert_eq!(before.expected_kobo, 2_300_000);
    assert_eq!(before.received_kobo, 2_000_000);
    assert_eq!(before.outstanding_kobo, 300_000);

    remittance::record(&conn, musa, yesterday, 300_000, "Femi Ajayi", "balance").unwrap();
    assert_eq!(row(&conn).outstanding_kobo, 0);
    assert_eq!(riders::cash_on_hand(&conn, musa).unwrap(), 0);
}
