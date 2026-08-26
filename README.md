# Addis-hotel-admin-dashboard-

Property-management dashboard ("Tourista Admin") for hotels listed on the Horn
of Africa portal: reservations, an availability calendar, room inventory, rates,
guests, payments, reviews, messaging, staff and settings.

Built with [Leptos](https://leptos.dev) 0.8 in client-side rendering mode and
Tailwind CSS, bundled by [Trunk](https://trunkrs.dev).

## Requirements

- Rust (stable) with the `wasm32-unknown-unknown` target
- [`trunk`](https://trunkrs.dev)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Running locally

```bash
trunk serve
```

Serves on <http://127.0.0.1:8080> with rebuild-on-change.

## Building for release

```bash
trunk build --release
```

Emits a static bundle to `dist/` that can be served from any static host. The
app is a SPA, so the host must rewrite unknown paths to `index.html`.

## Layout

```
src/
  app.rs          Router and the authenticated Shell (sidebar + topbar + footer)
  data.rs         Reservation, room, guest, payment and review fixtures
  date.rs         Calendar arithmetic: month lengths, weekdays, formatting
  components/     Sidebar, topbar, stat cards, charts, modals, toasts
  pages/          One module per route
style/
  tailwind.css    Design tokens, keyframes and hand-written utilities
```

## Routes

| Path | Page |
| --- | --- |
| `/login`, `/signup`, `/forgot-password` | Unauthenticated |
| `/` | Dashboard |
| `/analytics` | Revenue, occupancy and demand |
| `/reservations` | Bookings from the portal, phone and front desk |
| `/calendar` | Availability calendar |
| `/rooms`, `/rooms/:number` | Room inventory and details |
| `/rooms/bulk-upload` | Spreadsheet import |
| `/rates` | Rates and availability |
| `/guests`, `/guests/:booking_ref` | Guest records |
| `/payments` | Transactions and payouts |
| `/reviews` | Guest feedback and replies |
| `/messages` | Guest conversations |
| `/profile`, `/staff`, `/settings` | Administration |

## Availability calendar

Days come from the browser clock, so month navigation, weekday alignment and
the "today" marker are real. Click a cell to cycle its status; shift-click a
second cell in the same row to select a range and set them all at once. Room and
date pairs with no explicit edit fall back to a deterministic seeded status, so
the grid looks populated and is stable across reloads.

## Status

The UI is complete and driven entirely by the fixtures in `src/data.rs`; there
is no API wired up yet, and the login screen does not gate the dashboard routes.
Both are the next steps.
