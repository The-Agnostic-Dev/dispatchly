# Dispatchly

Dispatchly is the dispatch board for our same-day delivery fleet in Lagos. Merchants (fashion
stores, food vendors, gadget shops) call or WhatsApp in a pickup; a dispatcher logs it as an
order, the board shows it in the waiting queue, and the dispatcher gives it to a free rider.
Riders update the job from their phone (picked up, delivered, could not deliver) and collect
cash from the customer on cash-on-delivery orders. At the end of the day the accountant checks
what each rider collected against what they handed in.

All money is in naira and all times are Lagos time (WAT).

## Who uses it

| Role | What they do | Demo login |
| --- | --- | --- |
| Dispatcher | Log orders, assign riders, follow the board | `tolu`, `ngozi` |
| Operations lead | Everything a dispatcher does, plus the tariff (fees) | `kemi` |
| Accountant | Riders' cash and end-of-day remittance | `femi` |
| Rider | "My jobs" on the phone: picked up / delivered / could not deliver | `musa`, `chidi` |

Every demo account uses the password `dispatch2026`.

## How things work

**The board.** Three columns: *Waiting for a rider* (oldest order first, so the order that has
waited longest goes out next), *On the road* (assigned or picked up) and *Delivered today*.
The search box matches order number, customer name, phone, merchant and drop-off area. The
board reloads itself every minute.

**Delivery fee.** Worked out when the order is created and stored on the order:

```
fee = drop-off zone fee + distance band fee
    + ₦500 bridge crossing   (pickup and drop-off on different sides: Mainland vs Island)
    + ₦1,000 express         (if the merchant asks for express)
```

Zones: A Mainland Central (Yaba, Surulere, Ikeja, Maryland, Ebute Metta, Gbagada),
B Island (Victoria Island, Ikoyi, Lagos Island, Oniru, Lekki Phase 1), C Mainland Outer
(Ikorodu, Festac, Agege, Egbeda, Ojo) and D Lekki-Ajah Corridor (Ajah, Sangotedo, Chevron,
Ikota). A and C are Mainland; B and D are Island. Distance bands: up to 5 km, 5 to 10 km,
10 to 20 km, 20 to 35 km, over 35 km (a trip of exactly 10 km is in the 10 to 20 km band).
The operations lead edits zone and band fees under **Tariff**; new orders use the new fees,
existing orders keep the fee they were created with. Editing an order's details (distance,
areas, express) works out its fee again. The fee is invoiced to the merchant; it is not
collected by the rider.

**Riders and assignment.** Each rider has a job limit (usually 3) for parcels carried at the same
time. Assigning texts the rider (in development the text is only logged) and is refused if the
rider is at their limit or off shift.

**Order status.** Waiting for rider → Assigned → Picked up → Delivered. A rider can also mark
"Could not deliver", and a dispatcher can cancel an order until it is picked up. Every change
is written to the order's timeline.

**Cash and remittance.** On cash-on-delivery orders the rider collects the goods value from the
customer. *Cash on hand* (Riders page) is everything collected and not yet handed in.
The *Remittance* page shows, for a chosen day (Lagos time, midnight to midnight), the cash
each rider collected on orders delivered that day, what they handed in and what is outstanding.
Delivery fees are invoiced to merchants every Friday and are never part of the rider's cash.

## Running it

Native (Rust 1.90 or newer, tested on 1.97):

```sh
cargo run                      # http://localhost:5115, seeds demo data on first start
DISPATCHLY_RESEED=1 cargo run  # wipe and reload the demo data
```

Docker:

```sh
docker compose up --build      # http://localhost:5115, fresh demo data on every start
```

Settings are environment variables; see `.env.example`.

## Tests

```sh
cargo test
```

Tests run against an in-memory SQLite database loaded with the same demo data.

## Project layout

```
src/
  main.rs            start-up: config, database, seed, router
  config.rs          environment settings
  db.rs              SQLite schema
  auth.rs            logins, sessions, roles, flash messages
  clock.rs           Lagos time helpers
  money.rs           kobo <-> "₦12,500.00"
  validate.rs        phone numbers, text, distance
  forms.rs           form payloads and validation
  notify.rs          rider SMS (logged in development)
  views.rs           template environment and filters
  routes/            HTTP handlers, one file per area of the app
  services/          business rules: pricing, orders, board, dispatch, riders, remittance
  seed/              demo data (dates relative to today)
templates/           MiniJinja templates
static/              CSS and a little vanilla JS (fee preview, table sorting, board refresh)
tests/               cargo integration tests
```

Demo data is relative to today: five orders waiting, seven on the road, a couple delivered
today, and two previous days of deliveries and remittances for the accountant.
