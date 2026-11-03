//! Reference data: zones, bands, riders, staff logins and regular merchants.

pub const ZONES: &[(&str, &str, &str, i64, &[&str])] = &[
    ("A", "Mainland Central", "mainland", 180_000, &["Yaba", "Surulere", "Ikeja", "Maryland", "Ebute Metta", "Gbagada"]),
    ("B", "Island", "island", 250_000, &["Victoria Island", "Ikoyi", "Lagos Island", "Oniru", "Lekki Phase 1"]),
    ("C", "Mainland Outer", "mainland", 220_000, &["Ikorodu", "Festac", "Agege", "Egbeda", "Ojo"]),
    ("D", "Lekki-Ajah Corridor", "island", 300_000, &["Ajah", "Sangotedo", "Chevron", "Ikota"]),
];

pub const BANDS: &[(&str, f64, Option<f64>, i64)] = &[
    ("Up to 5 km", 0.0, Some(5.0), 0),
    ("5 to 10 km", 5.0, Some(10.0), 50_000),
    ("10 to 20 km", 10.0, Some(20.0), 120_000),
    ("20 to 35 km", 20.0, Some(35.0), 200_000),
    ("Over 35 km", 35.0, None, 350_000),
];

pub const RIDERS: &[(&str, &str, &str, i64, bool)] = &[
    ("Musa Abdullahi", "08034561201", "LSR 214 KJA", 3, true),
    ("Chidi Okeke", "08127734410", "KJA 552 AB", 3, true),
    ("Sule Ibrahim", "07065512398", "EKY 118 XA", 2, true),
    ("Tunde Bello", "09021145670", "LND 730 KT", 3, true),
    ("Emeka Obi", "08153390021", "AGL 402 BC", 4, true),
    ("Yusuf Lawal", "08098812234", "FKJ 991 AA", 3, false),
];

pub const USERS: &[(&str, &str, &str, Option<i64>)] = &[
    ("tolu", "Tolulope Adebayo", "dispatcher", None),
    ("ngozi", "Ngozi Eze", "dispatcher", None),
    ("kemi", "Kemi Ogunleye", "ops_lead", None),
    ("femi", "Femi Ajayi", "accountant", None),
    ("musa", "Musa Abdullahi", "rider", Some(1)),
    ("chidi", "Chidi Okeke", "rider", Some(2)),
];

pub const MERCHANTS: &[(&str, &str, &str, &str)] = &[
    ("Ronke's Closet", "08023341190", "Yaba", "14 Herbert Macaulay Way"),
    ("GadgetHub NG", "08187720045", "Ikeja", "Shop 22, Computer Village, Otigba St"),
    ("Mama Tee Foods", "07031128876", "Surulere", "5 Adeniran Ogunsanya St"),
    ("Glow by Ada", "09065543321", "Lekki Phase 1", "3 Admiralty Way"),
    ("Tasty Chops", "08142219087", "Gbagada", "40 Diya St, Gbagada Phase 2"),
    ("Paperback Lagos", "08055567712", "Ikoyi", "12 Awolowo Rd"),
    ("Bukka Express", "07019984432", "Lagos Island", "8 Broad St, Marina"),
];
