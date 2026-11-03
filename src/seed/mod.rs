//! Demo data for a working day in Lagos. Times are relative to "now" so the board looks
//! the same whenever the app starts.

use chrono::Duration;
use rusqlite::{Connection, params};

use crate::{auth::hash_password, clock, services::pricing::Tariff};

mod catalog;
mod orders;
use catalog::{BANDS, RIDERS, USERS, ZONES};
use orders::{Spec, insert};

pub const DEMO_PASSWORD: &str = "dispatch2026";

pub fn is_empty(conn: &Connection) -> rusqlite::Result<bool> {
    conn.query_row("SELECT COUNT(*) = 0 FROM users", [], |r| r.get(0))
}

pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "DELETE FROM order_events; DELETE FROM remittances; DELETE FROM orders; DELETE FROM users;
         DELETE FROM riders; DELETE FROM areas; DELETE FROM distance_bands; DELETE FROM zones;",
    )?;
    for (code, name, side, fee, areas) in ZONES {
        conn.execute("INSERT INTO zones (code, name, side, base_fee_kobo) VALUES (?1, ?2, ?3, ?4)", params![code, name, side, fee])?;
        let zone_id = conn.last_insert_rowid();
        for area in *areas {
            conn.execute("INSERT INTO areas (name, zone_id) VALUES (?1, ?2)", params![area, zone_id])?;
        }
    }
    for (label, min, max, fee) in BANDS {
        conn.execute("INSERT INTO distance_bands (label, min_km, max_km, fee_kobo) VALUES (?1, ?2, ?3, ?4)", params![label, min, max, fee])?;
    }
    for (name, phone, plate, capacity, active) in RIDERS {
        conn.execute(
            "INSERT INTO riders (name, phone, bike_plate, capacity, active) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, phone, plate, capacity, active],
        )?;
    }
    for (username, name, role, rider_id) in USERS {
        conn.execute(
            "INSERT INTO users (username, display_name, role, password_hash, rider_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![username, name, role, hash_password(DEMO_PASSWORD, username), rider_id],
        )?;
    }

    let tariff = Tariff::load(conn)?;
    let now = clock::now();
    let today = clock::today();
    let ago = |minutes: i64| now - Duration::minutes(minutes);
    let on = |days_back: i64, h: u32, m: u32| clock::at_local(today - Duration::days(days_back), h, m);

    #[rustfmt::skip]
    let specs = vec![
        // Waiting for a rider
        Spec { merchant: 0, customer: "Bisi Adewale", phone: "08031234567", area: "Surulere", address: "22 Bode Thomas St", km: 6.2, express: false, cod: 1_850_000, created: ago(95), rider: None, status: "pending" },
        Spec { merchant: 1, customer: "Adaeze Nwankwo", phone: "08164420981", area: "Victoria Island", address: "Plot 1412 Ahmadu Bello Way", km: 21.5, express: true, cod: 0, created: ago(62), rider: None, status: "pending" },
        Spec { merchant: 2, customer: "Ifeanyi Uche", phone: "07038812290", area: "Festac", address: "4th Avenue, 41 Road, House 12", km: 12.0, express: false, cod: 720_000, created: ago(38), rider: None, status: "pending" },
        Spec { merchant: 3, customer: "Halima Bello", phone: "09012239876", area: "Ajah", address: "Abraham Adesanya Estate, Block C", km: 14.3, express: false, cod: 2_500_000, created: ago(21), rider: None, status: "pending" },
        Spec { merchant: 4, customer: "Segun Ojo", phone: "08051192233", area: "Maryland", address: "7 Mobolaji Bank Anthony Way", km: 4.1, express: false, cod: 450_000, created: ago(6), rider: None, status: "pending" },
        // On the road
        Spec { merchant: 5, customer: "Ngozi Okafor", phone: "08029981100", area: "Lekki Phase 1", address: "18 Fola Osibo Rd", km: 7.5, express: false, cod: 1_200_000, created: ago(80), rider: Some(1), status: "picked_up" },
        Spec { merchant: 0, customer: "Temi Bakare", phone: "08133345521", area: "Ikeja", address: "9 Allen Avenue", km: 9.8, express: false, cod: 0, created: ago(48), rider: Some(1), status: "assigned" },
        Spec { merchant: 1, customer: "Kunle Afolabi", phone: "08076654410", area: "Agege", address: "31 Old Abeokuta Rd", km: 8.0, express: false, cod: 6_500_000, created: ago(70), rider: Some(2), status: "picked_up" },
        Spec { merchant: 6, customer: "Zainab Musa", phone: "07062234418", area: "Victoria Island", address: "Eko Hotel Rd, Block 4", km: 3.2, express: false, cod: 380_000, created: ago(75), rider: Some(4), status: "picked_up" },
        Spec { merchant: 6, customer: "Dapo Oyebanji", phone: "08184432217", area: "Ikoyi", address: "2 Glover Rd", km: 4.4, express: false, cod: 0, created: ago(58), rider: Some(4), status: "assigned" },
        Spec { merchant: 6, customer: "Yemisi Lawal", phone: "09033321184", area: "Oniru", address: "Oniru Estate, Plot 9", km: 5.6, express: false, cod: 650_000, created: ago(44), rider: Some(4), status: "assigned" },
        Spec { merchant: 3, customer: "Chioma Eze", phone: "08127790034", area: "Sangotedo", address: "Monastery Rd, Shop 3", km: 18.0, express: false, cod: 1_550_000, created: ago(55), rider: Some(5), status: "assigned" },
        // Delivered today (prepaid, so they do not change anyone's cash)
        Spec { merchant: 4, customer: "Gbenga Coker", phone: "08024451123", area: "Yaba", address: "UNILAG Staff Quarters, Akoka", km: 5.9, express: false, cod: 0, created: ago(200), rider: Some(1), status: "delivered" },
        Spec { merchant: 2, customer: "Esther Okon", phone: "07055512200", area: "Ebute Metta", address: "11 Borno Way", km: 6.8, express: false, cod: 0, created: ago(150), rider: Some(5), status: "delivered" },
        // Yesterday
        Spec { merchant: 4, customer: "Funmi Adeyemi", phone: "08033327781", area: "Surulere", address: "15 Ogunlana Drive", km: 9.1, express: true, cod: 1_850_000, created: on(2, 23, 30), rider: Some(2), status: "delivered" },
        Spec { merchant: 1, customer: "Obinna Eze", phone: "08164439902", area: "Maryland", address: "Mende Villa Estate", km: 6.0, express: false, cod: 4_500_000, created: on(1, 13, 10), rider: Some(2), status: "delivered" },
        Spec { merchant: 0, customer: "Kelechi Nnaji", phone: "08099911234", area: "Gbagada", address: "Ifako Gbagada, 6 Oladipo St", km: 7.2, express: false, cod: 1_500_000, created: on(1, 10, 30), rider: Some(1), status: "delivered" },
        Spec { merchant: 2, customer: "Aisha Garba", phone: "07084456630", area: "Yaba", address: "33 Commercial Avenue", km: 4.0, express: false, cod: 800_000, created: on(1, 15, 45), rider: Some(1), status: "delivered" },
        Spec { merchant: 1, customer: "Tolani Martins", phone: "08178823341", area: "Lekki Phase 1", address: "Chisco Bus Stop, 1 Durosinmi Etti", km: 24.5, express: false, cod: 11_200_000, created: on(1, 12, 5), rider: Some(5), status: "delivered" },
        Spec { merchant: 5, customer: "Uche Mbah", phone: "08067732210", area: "Victoria Island", address: "Bishop Aboyade Cole St", km: 3.0, express: false, cod: 0, created: on(1, 9, 20), rider: Some(3), status: "delivered" },
        Spec { merchant: 3, customer: "Rita Alabi", phone: "09045561287", area: "Chevron", address: "Chevron Drive, Block 7", km: 11.4, express: false, cod: 0, created: on(1, 11, 40), rider: Some(4), status: "delivered" },
        // Two days ago
        Spec { merchant: 2, customer: "Bola Shittu", phone: "08025567781", area: "Ikeja", address: "GRA, 4 Isaac John St", km: 11.0, express: false, cod: 3_150_000, created: on(2, 11, 0), rider: Some(1), status: "delivered" },
        Spec { merchant: 1, customer: "Nnamdi Ike", phone: "08139956671", area: "Egbeda", address: "Idimu Rd, 120", km: 16.5, express: false, cod: 5_200_000, created: on(2, 14, 30), rider: Some(2), status: "delivered" },
        Spec { merchant: 6, customer: "Ronke Akande", phone: "07012238890", area: "Ikoyi", address: "Bourdillon Rd, 21", km: 4.8, express: false, cod: 980_000, created: on(2, 16, 15), rider: Some(4), status: "delivered" },
        Spec { merchant: 0, customer: "Femi Kuti", phone: "08022244456", area: "Ojo", address: "LASU Rd, 3", km: 27.0, express: false, cod: 2_100_000, created: on(2, 10, 0), rider: Some(3), status: "failed" },
        Spec { merchant: 4, customer: "Peace Okoro", phone: "08188876543", area: "Agege", address: "Pen Cinema, 14 Ajayi St", km: 10.5, express: false, cod: 600_000, created: on(2, 18, 0), rider: None, status: "cancelled" },
    ];

    for spec in &specs {
        insert(conn, &tariff, spec)?;
    }

    // Cash handed in. Musa came back short yesterday.
    let remit = |rider: i64, days_back: i64, amount: i64, h: u32| {
        let day = today - Duration::days(days_back);
        conn.execute(
            "INSERT INTO remittances (rider_id, day, amount_kobo, received_by, note, at) VALUES (?1, ?2, ?3, 'Femi Ajayi', '', ?4)",
            params![rider, day.to_string(), amount, clock::to_db(clock::at_local(day, h, 0))],
        )
    };
    remit(1, 2, 3_150_000, 19)?;
    remit(2, 2, 5_200_000, 19)?;
    remit(1, 1, 2_000_000, 19)?;
    Ok(())
}
