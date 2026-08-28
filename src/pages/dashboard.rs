//! Dashboard, backed by `/dashboard/summary/`.
//!
//! The banner, the four tiles, today's financial summary and the recent-bookings
//! feed all come from that one endpoint. The right rail adds `/rooms/statistics/`
//! for the inventory donut, `/guests/` (arriving / checking out tabs) for the
//! movement lists and `/notifications/` for the activity feed.
//!
//! The week chart has no endpoint behind it: the API exposes no time series, so
//! the three series are bucketed here from the reservations list (bookings by
//! creation date, revenue by stay value) and from the availability matrix
//! (occupancy per night).

use crate::api;
use crate::components::{
    pluralize, Avatar, Badge, Card, DonutChart, Icon, LineChart, ProgressBar, SegmentedControl,
    StatCard, TrendPill,
};
use crate::date::{add_days, days_from_epoch, format_long, parse_iso, today, WEEKDAY_ABBR};
use leptos::prelude::*;
use leptos_router::components::A;

const CHART_TABS: &[&str] = &["Revenue", "Occupancy", "Bookings"];

/// The seven days ending today, oldest first.
fn last_7_days() -> Vec<crate::date::Date> {
    let t = today();
    (0..7).rev().map(|i| add_days(t, -(i as i64))).collect()
}

/// Bookings created per day and stay value per day, over the last week.
fn weekly_from_reservations(rows: &[api::Reservation]) -> (Vec<u32>, Vec<u32>) {
    let days = last_7_days();
    let mut bookings = vec![0u32; 7];
    let mut revenue = vec![0u32; 7];

    for r in rows {
        if let Some(created) = r.created_at.as_deref().and_then(parse_iso) {
            if let Some(i) = days.iter().position(|d| *d == created) {
                bookings[i] += 1;
            }
        }
        // Revenue is attributed to the arrival day: it is the day the stay
        // starts earning, and it is the only date every status carries.
        if matches!(r.status_str(), "Cancelled" | "Rejected") {
            continue;
        }
        if let Some(ci) = r.check_in_date.as_deref().and_then(parse_iso) {
            if let Some(i) = days.iter().position(|d| *d == ci) {
                revenue[i] += (r.amount() / 1000.0).round() as u32;
            }
        }
    }
    (bookings, revenue)
}

/// Occupied-room percentage per day, over the last week.
fn weekly_occupancy(matrix: &api::CalendarMatrix, total_rooms: u32) -> Vec<u32> {
    let days = last_7_days();
    let mut out = vec![0u32; 7];
    if total_rooms == 0 {
        return out;
    }
    for (i, (y, m, d)) in days.iter().enumerate() {
        let occupied = matrix
            .matrix
            .iter()
            .filter(|room| {
                room.blocked_days(*y, *m)
                    .iter()
                    .any(|(day, entry)| day == d && entry.reservation_id.is_some())
            })
            .count() as u32;
        out[i] = occupied * 100 / total_rooms;
    }
    out
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let chart_tab = RwSignal::new(CHART_TABS[0]);
    let t = today();

    let summary = LocalResource::new(|| async move { api::dashboard_summary(None).await });
    let stats = LocalResource::new(|| async move { api::room_stats().await });
    let arrivals = LocalResource::new(|| async move { api::list_guests("Arriving Today", "").await });
    let departures = LocalResource::new(|| async move { api::list_guests("Checking Out", "").await });
    let in_house = LocalResource::new(|| async move { api::list_guests("In House", "").await });
    let activity = LocalResource::new(|| async move { api::list_notifications(None, None, "").await });
    let pipeline = LocalResource::new(|| async move {
        api::list_reservations(&api::ReservationQuery {
            page_size: 100,
            ..Default::default()
        })
        .await
    });
    let week = LocalResource::new(move || async move {
        let rows = api::list_reservations(&api::ReservationQuery {
            page_size: 100,
            ordering: "-created_at".into(),
            ..Default::default()
        })
        .await
        .map(|p| p.items)
        .unwrap_or_default();
        let (bookings, revenue) = weekly_from_reservations(&rows);
        let matrix = api::calendar_matrix(t.0, t.1, "All", "").await.ok();
        let occupancy = match &matrix {
            Some(m) => weekly_occupancy(m, m.total_rooms),
            None => vec![0; 7],
        };
        (revenue, occupancy, bookings)
    });

    // ---- Derived readers ---------------------------------------------------

    let metrics = move || summary.get().and_then(Result::ok).map(|s| s.metrics);
    let hotel = move || summary.get().and_then(Result::ok).map(|s| s.hotel);
    let today_sum = move || summary.get().and_then(Result::ok).map(|s| s.today_summary);
    let currency = move || {
        hotel()
            .and_then(|h| h.currency)
            .unwrap_or_else(|| "ETB".into())
    };

    let tile = move |pick: fn(&api::DashMetrics) -> String| {
        metrics().map(|m| pick(&m)).unwrap_or_else(|| "—".into())
    };

    let guest_count = move |res: LocalResource<Result<api::Page<api::GuestRow>, api::ApiError>>| {
        res.get()
            .and_then(Result::ok)
            .map(|p| p.items.len())
            .unwrap_or(0)
    };

    let series = move || {
        let (revenue, occupancy, bookings) = week.get().unwrap_or((vec![], vec![], vec![]));
        let picked = match chart_tab.get() {
            "Occupancy" => occupancy,
            "Bookings" => bookings,
            _ => revenue,
        };
        if picked.is_empty() { vec![0; 7] } else { picked }
    };
    let series_label = move || {
        let vals = series();
        let total: u32 = vals.iter().sum();
        match chart_tab.get() {
            "Occupancy" => format!("{}% average", total / vals.len().max(1) as u32),
            "Bookings" => pluralize(total as usize, "booking"),
            _ => format!("{} {total}k", currency()),
        }
    };
    let series_colour = move || match chart_tab.get() {
        "Occupancy" => "#8b5cf6",
        "Bookings" => "#0ea5e9",
        _ => "#2563eb",
    };
    let day_labels = move || {
        last_7_days()
            .into_iter()
            .map(|d| WEEKDAY_ABBR[(days_from_epoch(d).rem_euclid(7) as usize + 4) % 7].to_string())
            .collect::<Vec<_>>()
    };

    view! {
        <div class="p-4 sm:p-6">
            // ---- Welcome banner ---------------------------------------
            <div class="mb-5 flex animate-fade-up flex-wrap items-center justify-between gap-4 overflow-hidden rounded-2xl bg-gradient-to-r from-blue-700 via-blue-700 to-indigo-800 p-5 text-white shadow-lg shadow-blue-900/20 sm:p-6">
                <div class="relative min-w-0">
                    <span class="pointer-events-none absolute -left-16 -top-16 h-40 w-40 rounded-full bg-white/10 blur-2xl"></span>
                    <h2 class="relative text-xl font-bold sm:text-2xl">
                        {move || match hotel() {
                            Some(h) if !h.name.is_empty() => h.name,
                            _ => api::session::user()
                                .map(|u| format!("Welcome back, {}", u.display_name()))
                                .unwrap_or_else(|| "Dashboard".into()),
                        }}
                    </h2>
                    <p class="relative mt-1 text-sm text-blue-100">
                        {move || format!(
                            "{}, {} and {} in house today.",
                            pluralize(guest_count(arrivals), "arrival"),
                            pluralize(guest_count(departures), "departure"),
                            pluralize(guest_count(in_house), "guest"),
                        )}
                    </p>
                    <p class="relative mt-1 text-xs text-blue-200">
                        {move || hotel().map(|h| h.location()).unwrap_or_default()}
                    </p>
                </div>
                <div class="flex shrink-0 flex-wrap items-center gap-2">
                    <span class="flex items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-sm font-medium ring-1 ring-white/20 backdrop-blur-sm">
                        <Icon name="calendar" class="h-4 w-4" />
                        {format_long(t)}
                    </span>
                    <A
                        href="/reservations"
                        attr:class="sheen flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-2 text-sm font-bold text-blue-800 shadow-md transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg active:scale-95"
                    >
                        <Icon name="calendar-check" class="h-4 w-4" />
                        "Reservations"
                    </A>
                </div>
            </div>

            // The API refuses every scoped endpoint until a hotel exists, so say
            // so plainly rather than showing a grid of dashes.
            <Show when=move || {
                summary.get().is_some_and(|r| r.as_ref().err().is_some_and(|e| e.is_forbidden() || e.status == 0))
            }>
                <div class="mb-5 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
                    <span class="flex items-center gap-2">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        "No property is set up on this account yet. Create your hotel to unlock the dashboard."
                    </span>
                    <A href="/profile" attr:class="rounded-lg bg-amber-600 px-3 py-1.5 text-xs font-bold text-white hover:bg-amber-700">
                        "Set up hotel"
                    </A>
                </div>
            </Show>

            // ---- Stat row ----------------------------------------------
            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="calendar-check"
                        label="New reservations"
                        value=Signal::derive(move || tile(|m| m.new_reservations_today.to_string()))
                        hint="Received today"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="check-circle"
                        label="Check-ins today"
                        value=Signal::derive(move || tile(|m| m.check_ins_today.to_string()))
                        hint="Guests arriving"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="bed"
                        label="Rooms available"
                        value=Signal::derive(move || tile(|m| m.rooms_available_today.to_string()))
                        hint=Signal::derive(move || tile(|m| format!("of {} rooms", m.total_rooms)))
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="trending-up"
                        label="Occupancy"
                        value=Signal::derive(move || tile(|m| format!("{:.0}%", m.occupancy_rate_today)))
                        hint=Signal::derive(move || tile(|m| format!("{} occupied", m.occupied_rooms_today)))
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
            </div>

            <div class="grid gap-5 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
                <div class="flex min-w-0 flex-col gap-5">
                    // ---- Performance chart -----------------------------
                    <div class="animate-fade-up" style="animation-delay: 220ms">
                        <Card>
                            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                                <div>
                                    <h2 class="text-sm font-bold text-slate-900">"This week"</h2>
                                    <div class="mt-1 flex flex-wrap items-center gap-2.5">
                                        <span class="text-2xl font-bold tabular-nums text-slate-900">
                                            {series_label}
                                        </span>
                                        <TrendPill delta=trend_delta(series()) />
                                    </div>
                                </div>
                                <SegmentedControl options=CHART_TABS selected=chart_tab />
                            </div>

                            {move || view! {
                                <LineChart
                                    values=series()
                                    labels=day_labels()
                                    stroke=series_colour()
                                    value_prefix=""
                                    height=200.0
                                />
                            }}
                        </Card>
                    </div>

                    // ---- Arrivals / departures --------------------------
                    <div class="grid animate-fade-up gap-5 md:grid-cols-2" style="animation-delay: 260ms">
                        <Card
                            title="Arriving today"
                            hint=Signal::derive(move || format!("{} expected", pluralize(guest_count(arrivals), "guest")))
                        >
                            <GuestMovementList
                                resource=arrivals
                                empty="No arrivals scheduled."
                                gradient="from-blue-500 to-indigo-600"
                            />
                        </Card>

                        <Card
                            title="Checking out today"
                            hint=Signal::derive(move || format!("{} turning over", pluralize(guest_count(departures), "room")))
                        >
                            <GuestMovementList
                                resource=departures
                                empty="No departures scheduled."
                                gradient="from-amber-500 to-orange-600"
                            />
                        </Card>
                    </div>

                    // ---- Recent reservations ----------------------------
                    <div class="animate-fade-up" style="animation-delay: 300ms">
                        <Card
                            title="Recent reservations"
                            hint="Newest first"
                            action=Box::new(|| view! {
                                <A
                                    href="/reservations"
                                    attr:class="group flex items-center gap-1 text-sm font-semibold text-blue-700 transition-colors hover:text-blue-800"
                                >
                                    "View all"
                                    <Icon name="arrow-right" class="h-3.5 w-3.5 transition-transform duration-200 group-hover:translate-x-0.5" />
                                </A>
                            }.into_any())
                        >
                            <Suspense fallback=|| view! {
                                <p class="py-6 text-center text-sm text-slate-400">"Loading reservations…"</p>
                            }>
                                {move || Suspend::new(async move {
                                    let Ok(s) = summary.await else {
                                        return view! {
                                            <p class="py-6 text-center text-sm text-slate-400">"Reservations unavailable."</p>
                                        }.into_any();
                                    };
                                    if s.recent_reservations.is_empty() {
                                        return view! {
                                            <p class="py-6 text-center text-sm text-slate-400">"No reservations yet."</p>
                                        }.into_any();
                                    }
                                    let cur = s.hotel.currency.clone().unwrap_or_else(|| "ETB".into());
                                    view! {
                                        <div class="stagger flex flex-col gap-2.5">
                                            {s.recent_reservations.into_iter().take(6).map(|r| {
                                                let cur = cur.clone();
                                                view! {
                                                    <div class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-slate-100 p-3 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:bg-blue-50/40 hover:shadow-sm">
                                                        <div class="min-w-0">
                                                            <p class="flex flex-wrap items-center gap-2 text-sm font-semibold text-slate-900">
                                                                {r.booking_reference.clone()}
                                                                <Badge
                                                                    label=r.booking_status.clone()
                                                                    tone=status_tone(&r.booking_status)
                                                                    dot=(r.booking_status == "New")
                                                                />
                                                            </p>
                                                            <p class="mt-0.5 truncate text-xs text-slate-500">
                                                                {format!(
                                                                    "{} · {} · Room {}",
                                                                    r.guest_name,
                                                                    pluralize(r.guest_count as usize, "guest"),
                                                                    r.room_number.clone().unwrap_or_else(|| "—".into()),
                                                                )}
                                                            </p>
                                                        </div>
                                                        <div class="shrink-0 text-right">
                                                            <p class="text-xs font-semibold text-slate-700">
                                                                {format!(
                                                                    "{} – {}",
                                                                    api::pretty_date(r.check_in_date.as_deref()),
                                                                    api::pretty_date(r.check_out_date.as_deref()),
                                                                )}
                                                            </p>
                                                            <p class="text-2xs text-slate-400">
                                                                {format!("{cur} {}", api::money_round(r.total_amount.as_deref()))}
                                                            </p>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                })}
                            </Suspense>
                        </Card>
                    </div>
                </div>

                // ================= RIGHT RAIL =================
                <div class="flex min-w-0 flex-col gap-5">
                    // ---- Room status ------------------------------------
                    <div class="animate-fade-up" style="animation-delay: 240ms">
                        <Card
                            title="Room status"
                            hint=Signal::derive(move || {
                                stats.get().and_then(Result::ok)
                                    .map(|s| format!("{} rooms in inventory", s.total_rooms))
                                    .unwrap_or_default()
                            })
                        >
                            <Suspense fallback=|| view! {
                                <p class="py-6 text-center text-sm text-slate-400">"Loading inventory…"</p>
                            }>
                                {move || Suspend::new(async move {
                                    let s = stats.await.unwrap_or_default();
                                    view! {
                                        <DonutChart
                                            slices=vec![
                                                ("Available".into(), s.available, "#10b981"),
                                                ("Occupied".into(), s.occupied, "#2563eb"),
                                                ("Reserved".into(), s.reserved, "#f59e0b"),
                                                ("Maintenance".into(), s.maintenance, "#ef4444"),
                                            ]
                                            centre_label="Rooms"
                                        />
                                    }
                                })}
                            </Suspense>
                            <A
                                href="/rooms"
                                attr:class="mt-4 flex w-full items-center justify-center gap-1.5 rounded-lg border border-slate-300 py-2 text-xs font-semibold text-slate-700 transition-colors hover:border-blue-300 hover:text-blue-700"
                            >
                                <Icon name="bed" class="h-3.5 w-3.5" />
                                "Manage rooms"
                            </A>
                        </Card>
                    </div>

                    // ---- Quick actions ----------------------------------
                    <div class="animate-fade-up" style="animation-delay: 280ms">
                        <Card title="Quick actions">
                            <div class="grid grid-cols-2 gap-2.5">
                                {[
                                    ("plus", "Add room", "/rooms"),
                                    ("calendar", "Block dates", "/calendar"),
                                    ("tag", "Update rates", "/rates"),
                                    ("upload", "Bulk upload", "/rooms/bulk-upload"),
                                    ("bar-chart", "Analytics", "/analytics"),
                                    ("star", "Reviews", "/reviews"),
                                ].into_iter().map(|(icon, label, href)| view! {
                                    <A
                                        href=href
                                        attr:class="group flex flex-col items-center gap-1.5 rounded-lg border border-slate-200 p-3 text-center text-xs font-semibold text-slate-700 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:bg-blue-50/60 hover:text-blue-700 hover:shadow-sm"
                                    >
                                        <Icon name=icon class="h-4 w-4 text-slate-500 transition-transform duration-200 group-hover:scale-110 group-hover:text-blue-700" />
                                        {label}
                                    </A>
                                }).collect_view()}
                            </div>
                        </Card>
                    </div>

                    // ---- Today's summary --------------------------------
                    <div class="animate-fade-up" style="animation-delay: 320ms">
                        <Card title="Today's summary">
                            <dl class="flex flex-col gap-3 text-sm">
                                {move || {
                                    let s = today_sum().unwrap_or_default();
                                    let m = metrics().unwrap_or_default();
                                    let revenue = s
                                        .total_revenue_formatted
                                        .clone()
                                        .unwrap_or_else(|| format!(
                                            "{} {}",
                                            currency(),
                                            api::money(s.total_revenue.as_deref()),
                                        ));
                                    vec![
                                        ("Total bookings".to_string(), s.total_bookings.to_string(), "calendar-check"),
                                        ("Room revenue".to_string(), revenue, "banknote"),
                                        ("Check-ins".to_string(), s.check_ins.to_string(), "check-circle"),
                                        ("Check-outs".to_string(), s.check_outs.to_string(), "log-out"),
                                        ("Rooms occupied".to_string(), m.occupied_rooms_today.to_string(), "bed"),
                                    ]
                                    .into_iter()
                                    .map(|(label, value, icon)| view! {
                                        <div class="flex items-center justify-between gap-3">
                                            <dt class="flex items-center gap-2 text-slate-500">
                                                <Icon name=icon class="h-3.5 w-3.5 text-slate-400" />
                                                {label}
                                            </dt>
                                            <dd class="font-bold tabular-nums text-slate-900">{value}</dd>
                                        </div>
                                    })
                                    .collect_view()
                                }}
                            </dl>

                            <div class="mt-4 border-t border-slate-100 pt-4">
                                <div class="mb-1.5 flex items-center justify-between text-xs">
                                    <span class="font-semibold text-slate-600">"Occupancy today"</span>
                                    <span class="font-bold tabular-nums text-slate-900">
                                        {move || tile(|m| format!("{:.0}%", m.occupancy_rate_today))}
                                    </span>
                                </div>
                                <ProgressBar
                                    percent=Signal::derive(move || {
                                        metrics().map(|m| m.occupancy_rate_today.round() as u32).unwrap_or(0)
                                    })
                                    fill="bg-gradient-to-r from-blue-600 to-indigo-600"
                                />
                                <p class="mt-1.5 text-2xs text-slate-400">
                                    {move || tile(|m| format!(
                                        "{} of {} rooms sold tonight",
                                        m.occupied_rooms_today, m.total_rooms,
                                    ))}
                                </p>
                            </div>
                        </Card>
                    </div>

                    // ---- Reservation mix ---------------------------------
                    // The API has no channel attribution, so the closest real
                    // breakdown is the status mix of live bookings.
                    <div class="animate-fade-up" style="animation-delay: 360ms">
                        <Card title="Reservation mix" hint="Current pipeline">
                            <Suspense fallback=|| view! {
                                <p class="py-4 text-center text-sm text-slate-400">"Loading…"</p>
                            }>
                                {move || Suspend::new(async move {
                                    let rows = pipeline.await.map(|p| p.items).unwrap_or_default();
                                    let total = rows.len().max(1);
                                    let buckets: Vec<(&str, &str, usize)> = ["New", "Confirmed", "In-house", "Completed", "Cancelled"]
                                        .into_iter()
                                        .map(|s| {
                                            let n = rows.iter().filter(|r| r.status_str() == s).count();
                                            (s, mix_colour(s), n)
                                        })
                                        .filter(|(_, _, n)| *n > 0)
                                        .collect();
                                    if buckets.is_empty() {
                                        return view! {
                                            <p class="py-4 text-center text-sm text-slate-400">"No reservations yet."</p>
                                        }.into_any();
                                    }
                                    view! {
                                        <ul class="flex flex-col gap-3">
                                            {buckets.into_iter().map(|(name, colour, n)| {
                                                let pct = (n * 100 / total) as u32;
                                                view! {
                                                    <li>
                                                        <div class="mb-1 flex items-center justify-between text-xs">
                                                            <span class="flex items-center gap-1.5 text-slate-600">
                                                                <span class="h-2 w-2 rounded-full" style=format!("background: {colour}")></span>
                                                                {name}
                                                            </span>
                                                            <span class="font-bold tabular-nums text-slate-900">
                                                                {format!("{n} · {pct}%")}
                                                            </span>
                                                        </div>
                                                        <ProgressBar percent=pct fill="bg-blue-500" height="h-1.5" />
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any()
                                })}
                            </Suspense>
                        </Card>
                    </div>

                    // ---- Activity feed -----------------------------------
                    <div class="animate-fade-up" style="animation-delay: 400ms">
                        <Card
                            title="Recent activity"
                            action=Box::new(|| view! {
                                <A href="/messages" attr:class="text-xs font-semibold text-blue-700 hover:text-blue-800">"All"</A>
                            }.into_any())
                        >
                            <Suspense fallback=|| view! {
                                <p class="py-4 text-center text-sm text-slate-400">"Loading activity…"</p>
                            }>
                                {move || Suspend::new(async move {
                                    let items = activity.await.map(|p| p.items).unwrap_or_default();
                                    let items: Vec<_> = items.into_iter().take(6).collect();
                                    if items.is_empty() {
                                        return view! {
                                            <p class="py-4 text-center text-sm text-slate-400">"Nothing has happened yet."</p>
                                        }.into_any();
                                    }
                                    let last = items.len() - 1;
                                    view! {
                                        <ol class="flex flex-col">
                                            {items.into_iter().enumerate().map(|(i, n)| {
                                                let is_last = i == last;
                                                let (icon, tint) = activity_style(n.kind());
                                                view! {
                                                    <li class="relative flex gap-3 pb-4 last:pb-0">
                                                        <Show when=move || !is_last>
                                                            <span class="absolute left-[15px] top-9 h-[calc(100%-1.75rem)] w-px bg-slate-200"></span>
                                                        </Show>
                                                        <span class=format!(
                                                            "relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full ring-4 ring-white {tint}"
                                                        )>
                                                            <Icon name=icon class="h-3.5 w-3.5" />
                                                        </span>
                                                        <span class="min-w-0 pt-0.5">
                                                            <span class="block text-xs font-semibold text-slate-800">{n.title.clone()}</span>
                                                            <span class="mt-0.5 block text-2xs leading-relaxed text-slate-500">{n.message.clone()}</span>
                                                            <span class="mt-0.5 block text-2xs text-slate-400">
                                                                {api::pretty_datetime(n.created_at.as_deref())}
                                                            </span>
                                                        </span>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ol>
                                    }.into_any()
                                })}
                            </Suspense>
                        </Card>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Arrivals / departures list — same shape for both tabs of `/guests/`.
#[component]
fn GuestMovementList(
    resource: LocalResource<Result<api::Page<api::GuestRow>, api::ApiError>>,
    empty: &'static str,
    gradient: &'static str,
) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! {
            <p class="py-6 text-center text-sm text-slate-400">"Loading…"</p>
        }>
            {move || Suspend::new(async move {
                let rows = resource.await.map(|p| p.items).unwrap_or_default();
                if rows.is_empty() {
                    return view! { <p class="py-6 text-center text-sm text-slate-400">{empty}</p> }.into_any();
                }
                view! {
                    <div class="stagger flex flex-col gap-2.5">
                        {rows.into_iter().map(|g| {
                            let vip = g.is_vip;
                            view! {
                                <A
                                    href=format!("/guests/{}", g.booking_ref)
                                    attr:class="group flex items-center gap-3 rounded-lg border border-slate-100 p-2.5 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:bg-blue-50/40 hover:shadow-sm"
                                >
                                    <Avatar initials=g.initials.clone() size="h-9 w-9 text-2xs" gradient=gradient />
                                    <span class="min-w-0 flex-1">
                                        <span class="flex items-center gap-1.5">
                                            <span class="truncate text-sm font-semibold text-slate-900">{g.display_name.clone()}</span>
                                            <Show when=move || vip>
                                                <span class="rounded bg-purple-100 px-1 py-0.5 text-[9px] font-bold text-purple-700">"VIP"</span>
                                            </Show>
                                        </span>
                                        <span class="block truncate text-2xs text-slate-500">
                                            {format!(
                                                "Room {} · {} · {}",
                                                if g.room_number.is_empty() { "—".into() } else { g.room_number.clone() },
                                                if g.room_type.is_empty() { "—".into() } else { g.room_type.clone() },
                                                pluralize(g.guests_count as usize, "guest"),
                                            )}
                                        </span>
                                    </span>
                                    <Icon name="chevron-right" class="h-4 w-4 shrink-0 text-slate-300 transition-transform duration-200 group-hover:translate-x-0.5" />
                                </A>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            })}
        </Suspense>
    }
}

/// Last day against the six before it, as a percentage.
fn trend_delta(values: Vec<u32>) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    let last = *values.last().unwrap() as f32;
    let earlier: u32 = values[..values.len() - 1].iter().sum();
    let mean = earlier as f32 / (values.len() - 1) as f32;
    if mean <= 0.0 {
        return 0.0;
    }
    ((last - mean) / mean * 100.0).clamp(-999.0, 999.0)
}

fn mix_colour(status: &str) -> &'static str {
    match status {
        "New" => "#f59e0b",
        "Confirmed" => "#10b981",
        "In-house" => "#2563eb",
        "Completed" => "#6366f1",
        _ => "#ef4444",
    }
}

/// Icon and tint for a notification type.
fn activity_style(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "Booking" => ("calendar-check", "bg-blue-50 text-blue-600"),
        "Payment" => ("banknote", "bg-emerald-50 text-emerald-600"),
        "Alert" => ("info", "bg-red-50 text-red-600"),
        _ => ("settings", "bg-slate-100 text-slate-500"),
    }
}

pub(crate) fn status_tone(status: &str) -> &'static str {
    match status {
        "Confirmed" | "Available" | "Active" | "In House" | "In-house" | "Completed" => "green",
        "New" | "Pending" | "Reserved" | "Checking Out" => "amber",
        "Arriving Today" | "Occupied" => "blue",
        "Cancelled" | "Rejected" | "No show" | "No-show" | "Maintenance" => "red",
        "VIP" => "purple",
        _ => "slate",
    }
}

/// Filled pill classes shared with the guests, reservations and rooms tables.
pub(crate) fn status_pill(status: &str) -> &'static str {
    match status {
        "Confirmed" | "Available" | "Active" | "In House" | "In-house" | "Completed" | "Success"
        | "Paid" => "bg-emerald-100 text-emerald-700",
        "New" | "Pending" | "Reserved" | "Checking Out" | "Partially paid" => {
            "bg-amber-100 text-amber-700"
        }
        "Arriving Today" | "Occupied" => "bg-blue-100 text-blue-700",
        "Cancelled" | "Rejected" | "No show" | "No-show" | "Maintenance" | "Failed" | "Unpaid" => {
            "bg-red-100 text-red-700"
        }
        "VIP" => "bg-purple-100 text-purple-700",
        "Inactive" | "Deactivated" => "bg-slate-200 text-slate-600",
        _ => "bg-slate-100 text-slate-600",
    }
}

/// Dot + text colour pair used by the rooms table.
pub(crate) fn status_dot(status: &str) -> (&'static str, &'static str) {
    match status {
        "Available" => ("bg-emerald-500", "text-emerald-700"),
        "Occupied" => ("bg-blue-500", "text-blue-700"),
        "Reserved" => ("bg-amber-500", "text-amber-700"),
        "Maintenance" => ("bg-red-500", "text-red-700"),
        _ => ("bg-slate-400", "text-slate-600"),
    }
}
