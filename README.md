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
  api.rs          The API client: session, transport, and one fn per endpoint
  app.rs          Router and the authenticated Shell (sidebar + topbar + footer)
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

One row per room, one column per day of the viewed month, read from
`/rooms/calendar-matrix/`. Click a night to close or reopen it: an open night is
blocked through `/rooms/block-dates/`, a blocked one is released through
`/rooms/special-events/{id}/end/`. Nights held by a reservation are read-only —
those are released from the reservation itself.

## API

Every screen is backed by the Addis Hotel Booking API at
`https://addisapi.pastaromatour.com/api/v1` — there are no fixtures left in the
crate. `src/api.rs` is the whole client: it owns the JWT session
(`localStorage`, with a transparent refresh on a 401), unwraps the service's
`{success, message, data, meta}` envelope, and exposes one typed function per
endpoint.

Two things the published OpenAPI schema gets wrong, and which the client
corrects:

* The schema documents plain DRF pagination (`{count, next, previous, results}`)
  and types most response bodies as `{}`. The service actually wraps everything
  in the envelope above. Types here follow the running service.
* `/rooms/calendar-matrix/` returns each blocking entry once per blocked night
  but never says *which* night. `CalendarRoom::blocked_days` reconstructs the
  grid from each entry's own check-in/check-out dates instead.

Three screens are composed rather than fetched, because the API has no endpoint
for them:

| Screen | Built from |
| --- | --- |
| Payments | the `completions` settlement written onto a reservation at checkout — there is no `/payments/` resource |
| Analytics | the reservation ledger, expanded into room-nights and bucketed by day, month, weekday and room type |
| Messages | the notification feed plus `/notifications/broadcast/` — the API has no guest-messaging resource |

## Status

Complete and live. The login screen gates every dashboard route, and a fresh
`organization_admin` with no property is sent to Hotel Profile to create one —
until it exists, the API refuses every org-scoped endpoint.
