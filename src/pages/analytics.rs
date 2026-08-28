//! Analytics, computed from `/reservations/`, `/rooms/statistics/`,
//! `/guests/` and `/reviews/metrics/`.
//!
//! The API exposes no time-series or analytics endpoint, so every series here is
//! derived in the browser from the reservation ledger: each booking is expanded
//! into its individual nights, and those room-nights are then bucketed by day,
//! month, weekday or room type. Cancelled and rejected bookings are excluded.
//!
//! Occupancy is room-nights sold over room-nights available (`total_rooms` ×
//! days in the bucket), which is the same definition the availability matrix
//! uses, so the two screens agree.

use crate::api;
use crate::components::{
    BarChart, Card, DonutChart, Icon, LineChart, PageHeader, ProgressBar, SegmentedControl,
    StatCard, TrendPill,
};
use crate::date::{
    add_days, days_from_epoch, days_in_month, parse_iso, shift_month, today, Date, MONTH_NAMES,
    WEEKDAY_ABBR,
};
use leptos::prelude::*;

const RANGES: &[&str] = &["7 days", "30 days", "12 months"];

/// One night sold: which date, at what rate, in which room type.
#[derive(Clone)]
struct Night {
    date: Date,
    rate: f64,
    room_type: String,
}

/// Expands every live reservation into the nights it occupies.
///
/// The nightly rate is the stay's total divided by its nights, so a settled stay
/// contributes its actual money and an upcoming one its expected value.
fn nights_sold(rows: &[api::Reservation]) -> Vec<Night> {
    let mut out = Vec::new();
    for r in rows {
        if matches!(r.status_str(), "Cancelled" | "Rejected" | "No show") {
            continue;
        }
        let Some(start) = r.check_in_date.as_deref().and_then(parse_iso) else {
            continue;
        };
        let nights = r.number_of_nights.max(1) as i64;
        let rate = r.amount() / nights as f64;
        let room_type = r.room_type().to_string();
        for i in 0..nights {
            out.push(Night {
                date: add_days(start, i),
                rate,
                room_type: room_type.clone(),
            });
        }
    }
    out
}

/// The buckets a range is charted over: a label plus the dates it covers.
struct Buckets {
    labels: Vec<String>,
    /// Inclusive epoch-day range per bucket, and how many days it spans.
    spans: Vec<(i64, i64, u32)>,
}

fn buckets_for(range: &str) -> Buckets {
    let t = today();
    match range {
        "30 days" => {
            let mut labels = Vec::new();
            let mut spans = Vec::new();
            for i in (0..30).rev() {
                let d = add_days(t, -i);
                let e = days_from_epoch(d);
                // Naming every day crowds the axis; every fifth is enough.
                labels.push(if i % 5 == 0 { d.2.to_string() } else { String::new() });
                spans.push((e, e, 1));
            }
            Buckets { labels, spans }
        }
        "12 months" => {
            let mut labels = Vec::new();
            let mut spans = Vec::new();
            for i in (0..12).rev() {
                let (y, m) = shift_month(t.0, t.1, -i);
                let len = days_in_month(y, m);
                labels.push(MONTH_NAMES[(m - 1) as usize][..3].to_string());
                spans.push((
                    days_from_epoch((y, m, 1)),
                    days_from_epoch((y, m, len)),
                    len,
                ));
            }
            Buckets { labels, spans }
        }
        _ => {
            let mut labels = Vec::new();
            let mut spans = Vec::new();
            for i in (0..7).rev() {
                let d = add_days(t, -i);
                let e = days_from_epoch(d);
                labels.push(WEEKDAY_ABBR[(e.rem_euclid(7) as usize + 4) % 7].to_string());
                spans.push((e, e, 1));
            }
            Buckets { labels, spans }
        }
    }
}

/// Revenue (in thousands) and occupancy percentage per bucket.
fn series(nights: &[Night], b: &Buckets, total_rooms: u32) -> (Vec<u32>, Vec<u32>) {
    let mut revenue = Vec::with_capacity(b.spans.len());
    let mut occupancy = Vec::with_capacity(b.spans.len());
    for (from, to, days) in &b.spans {
        let picked: Vec<&Night> = nights
            .iter()
            .filter(|n| {
                let e = days_from_epoch(n.date);
                e >= *from && e <= *to
            })
            .collect();
        let money: f64 = picked.iter().map(|n| n.rate).sum();
        revenue.push((money / 1000.0).round() as u32);
        let capacity = total_rooms as u64 * *days as u64;
        occupancy.push(if capacity == 0 {
            0
        } else {
            (picked.len() as u64 * 100 / capacity).min(100) as u32
        });
    }
    (revenue, occupancy)
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let range = RwSignal::new(RANGES[0]);

    let reservations = LocalResource::new(|| async move {
        api::list_reservations(&api::ReservationQuery {
            page_size: 100,
            ordering: "-check_in_date".into(),
            ..Default::default()
        })
        .await
        .map(|p| p.items)
        .unwrap_or_default()
    });
    let stats = LocalResource::new(|| async move { api::room_stats().await });
    let guests = LocalResource::new(|| async move { api::list_guests("All", "").await });
    let reviews = LocalResource::new(|| async move { api::review_metrics().await });
    let hotel = LocalResource::new(|| async move { api::dashboard_summary(None).await });

    let total_rooms = move || {
        stats
            .get()
            .and_then(Result::ok)
            .map(|s| s.total_rooms)
            .unwrap_or(0)
    };
    let currency = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .and_then(|s| s.hotel.currency)
            .unwrap_or_else(|| "ETB".into())
    };
    let hotel_name = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .map(|s| s.hotel.name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "your property".into())
    };

    // All derived numbers in one reader so the whole page shares one pass.
    let computed = move || {
        let rows = reservations.get().unwrap_or_default();
        let nights = nights_sold(&rows);
        let b = buckets_for(range.get());
        let rooms = total_rooms();
        let (revenue, occupancy) = series(&nights, &b, rooms);

        // Range totals, over the whole window rather than per bucket.
        let from = b.spans.first().map(|(f, _, _)| *f).unwrap_or(0);
        let to = b.spans.last().map(|(_, t, _)| *t).unwrap_or(0);
        let days: u32 = b.spans.iter().map(|(_, _, d)| *d).sum();
        let in_range: Vec<&Night> = nights
            .iter()
            .filter(|n| {
                let e = days_from_epoch(n.date);
                e >= from && e <= to
            })
            .collect();
        let money: f64 = in_range.iter().map(|n| n.rate).sum();
        let sold = in_range.len() as u32;
        let adr = if sold == 0 { 0.0 } else { money / sold as f64 };
        let capacity = rooms as u64 * days as u64;
        let revpar = if capacity == 0 { 0.0 } else { money / capacity as f64 };
        let avg_occ = if capacity == 0 {
            0
        } else {
            (sold as u64 * 100 / capacity).min(100) as u32
        };

        (b.labels, revenue, occupancy, money, sold, adr, revpar, avg_occ, nights)
    };

    let labels = move || computed().0;
    let revenue_series = move || computed().1;
    let occupancy_series = move || computed().2;
    let total_revenue = move || computed().3;
    let nights_count = move || computed().4;
    let adr = move || computed().5;
    let revpar = move || computed().6;
    let avg_occupancy = move || computed().7;
    let all_nights = move || computed().8;

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Analytics"
                subtitle=Signal::derive(move || format!("Revenue, occupancy and demand for {}.", hotel_name()))
            >
                <SegmentedControl options=RANGES selected=range />
            </PageHeader>

            // ---- Headline numbers -------------------------------------
            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="banknote"
                        label="Revenue"
                        value=Signal::derive(move || format!("{} {}", currency(), api::money_round(Some(&format!("{:.2}", total_revenue())))))
                        hint=Signal::derive(move || format!("{} room-nights sold", nights_count()))
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="bed"
                        label="Avg. occupancy"
                        value=Signal::derive(move || format!("{}%", avg_occupancy()))
                        hint=Signal::derive(move || format!("Across {} rooms", total_rooms()))
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="tag"
                        label="Average daily rate"
                        value=Signal::derive(move || format!("{} {}", currency(), api::money_round(Some(&format!("{:.2}", adr())))))
                        hint="Per occupied room"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="trending-up"
                        label="RevPAR"
                        value=Signal::derive(move || format!("{} {}", currency(), api::money_round(Some(&format!("{:.2}", revpar())))))
                        hint="Revenue per available room"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
            </div>

            // ---- Revenue ------------------------------------------------
            <div class="mb-5 animate-fade-up" style="animation-delay: 220ms">
                <Card title="Revenue trend" hint=Signal::derive(move || format!("Thousands of {}", currency()))>
                    <div class="mb-3 flex flex-wrap items-center gap-3">
                        <span class="text-2xl font-bold tabular-nums text-slate-900">
                            {move || format!("{} {}", currency(), api::money_round(Some(&format!("{:.2}", total_revenue()))))}
                        </span>
                        <TrendPill delta=Signal::derive(move || delta_of(revenue_series())) period="vs earlier in range" />
                    </div>
                    {move || view! {
                        <LineChart values=revenue_series() labels=labels() stroke="#10b981" value_prefix="" height=220.0 />
                    }}
                </Card>
            </div>

            // ---- Occupancy ----------------------------------------------
            <div class="mb-5 animate-fade-up" style="animation-delay: 260ms">
                <Card title="Occupancy" hint="Room-nights sold over rooms available">
                    <div class="mb-3 flex flex-wrap items-center gap-3">
                        <span class="text-2xl font-bold tabular-nums text-slate-900">
                            {move || format!("{}%", avg_occupancy())}
                        </span>
                        <TrendPill delta=Signal::derive(move || delta_of(occupancy_series())) period="vs earlier in range" />
                    </div>
                    {move || view! {
                        <LineChart values=occupancy_series() labels=labels() stroke="#8b5cf6" value_prefix="" height=220.0 />
                    }}
                </Card>
            </div>

            <div class="mb-5 grid animate-fade-up gap-5 lg:grid-cols-2" style="animation-delay: 300ms">
                // ---- Guest ratings --------------------------------------
                // The API has no channel attribution, so the closest real
                // distribution is the rating histogram.
                <Card title="Guest ratings" hint="From /reviews/metrics/">
                    <Suspense fallback=|| view! { <p class="py-6 text-center text-sm text-slate-400">"Loading…"</p> }>
                        {move || Suspend::new(async move {
                            let m = reviews.await.unwrap_or_default();
                            if m.total_reviews == 0 {
                                return view! {
                                    <p class="py-6 text-center text-sm text-slate-400">"No reviews yet."</p>
                                }.into_any();
                            }
                            view! {
                                <div class="mb-4 flex items-end gap-2">
                                    <span class="text-3xl font-bold tabular-nums text-slate-900">{format!("{:.1}", m.average_rating)}</span>
                                    <span class="pb-1 text-sm text-slate-500">{format!("from {} reviews", m.total_reviews)}</span>
                                </div>
                                <ul class="flex flex-col gap-2.5">
                                    {m.rating_breakdown.rows().into_iter().map(|(stars, bucket)| {
                                        let pct = bucket.percentage.round() as u32;
                                        let count = bucket.count;
                                        view! {
                                            <li>
                                                <div class="mb-1 flex items-center justify-between text-xs">
                                                    <span class="flex items-center gap-1 text-slate-600">
                                                        <Icon name="star" class="h-3 w-3 text-amber-500" />
                                                        {format!("{stars} star")}
                                                    </span>
                                                    <span class="font-bold tabular-nums text-slate-900">{format!("{count} · {pct}%")}</span>
                                                </div>
                                                <ProgressBar percent=pct fill="bg-amber-500" height="h-1.5" />
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        })}
                    </Suspense>
                </Card>

                // ---- Room mix -------------------------------------------
                <Card title="Room-nights by type" hint="Share of nights sold in range">
                    {move || {
                        let by_type = group_room_types(&all_nights());
                        if by_type.is_empty() {
                            return view! {
                                <p class="py-6 text-center text-sm text-slate-400">"No nights sold in this range."</p>
                            }.into_any();
                        }
                        let palette = ["#2563eb", "#10b981", "#f59e0b", "#8b5cf6", "#ef4444", "#0ea5e9"];
                        let slices: Vec<(String, u32, &'static str)> = by_type
                            .iter()
                            .enumerate()
                            .map(|(i, (name, nights, _))| {
                                (name.clone(), *nights, palette[i % palette.len()])
                            })
                            .collect();
                        view! { <DonutChart slices=slices centre_label="Nights" /> }.into_any()
                    }}
                </Card>
            </div>

            // ---- Room type performance ----------------------------------
            <div class="mb-5 animate-fade-up" style="animation-delay: 340ms">
                <Card title="Room type performance" hint="Nights sold, revenue and share">
                    {move || {
                        let by_type = group_room_types(&all_nights());
                        let total_nights: u32 = by_type.iter().map(|(_, n, _)| *n).sum::<u32>().max(1);
                        if by_type.is_empty() {
                            return view! {
                                <p class="py-6 text-center text-sm text-slate-400">"No nights sold in this range."</p>
                            }.into_any();
                        }
                        let cur = currency();
                        view! {
                            <div class="-mx-5 overflow-x-auto">
                                <table class="w-full text-left text-sm">
                                    <thead class="border-b border-slate-100 text-2xs uppercase text-slate-500">
                                        <tr>
                                            <th class="px-5 py-2">"Room type"</th>
                                            <th class="px-5 py-2">"Nights sold"</th>
                                            <th class="px-5 py-2">"Revenue"</th>
                                            <th class="px-5 py-2">"Avg. rate"</th>
                                            <th class="px-5 py-2 w-40">"Share"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {by_type.into_iter().map(|(name, nights, money)| {
                                            let pct = nights * 100 / total_nights;
                                            let avg = if nights == 0 { 0.0 } else { money / nights as f64 };
                                            let cur = cur.clone();
                                            view! {
                                                <tr class="border-b border-slate-50 last:border-0">
                                                    <td class="px-5 py-2.5 font-medium text-slate-800">{name}</td>
                                                    <td class="px-5 py-2.5 tabular-nums">{nights}</td>
                                                    <td class="px-5 py-2.5 tabular-nums">
                                                        {format!("{cur} {}", api::money_round(Some(&format!("{money:.2}"))))}
                                                    </td>
                                                    <td class="px-5 py-2.5 tabular-nums">
                                                        {format!("{cur} {}", api::money_round(Some(&format!("{avg:.2}"))))}
                                                    </td>
                                                    <td class="px-5 py-2.5">
                                                        <div class="flex items-center gap-2">
                                                            <ProgressBar percent=pct fill="bg-blue-600" height="h-1.5" />
                                                            <span class="w-9 shrink-0 text-right text-2xs font-bold tabular-nums text-slate-600">
                                                                {format!("{pct}%")}
                                                            </span>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }}
                </Card>
            </div>

            <div class="grid animate-fade-up gap-5 lg:grid-cols-2" style="animation-delay: 380ms">
                // ---- Nationality ----------------------------------------
                <Card title="Guest nationality" hint="Where your guests travel from">
                    <Suspense fallback=|| view! { <p class="py-6 text-center text-sm text-slate-400">"Loading…"</p> }>
                        {move || Suspend::new(async move {
                            let rows = guests.await.map(|p| p.items).unwrap_or_default();
                            let groups = group_nationalities(&rows);
                            if groups.is_empty() {
                                return view! {
                                    <p class="py-6 text-center text-sm text-slate-400">"No guest records yet."</p>
                                }.into_any();
                            }
                            let max = groups.iter().map(|(_, n)| *n).max().unwrap_or(1);
                            view! {
                                <ul class="flex flex-col gap-3">
                                    {groups.into_iter().map(|(name, count)| {
                                        let pct = count * 100 / max.max(1);
                                        view! {
                                            <li>
                                                <div class="mb-1 flex items-center justify-between text-xs">
                                                    <span class="text-slate-600">{name}</span>
                                                    <span class="font-bold tabular-nums text-slate-900">{count}</span>
                                                </div>
                                                <ProgressBar percent=pct fill="bg-indigo-500" height="h-1.5" />
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        })}
                    </Suspense>
                </Card>

                // ---- Weekday demand -------------------------------------
                <Card title="Demand by weekday" hint="Room-nights sold per weekday in range">
                    {move || {
                        let by_day = group_weekdays(&all_nights());
                        view! {
                            <BarChart
                                values=by_day
                                labels={WEEKDAY_ABBR.iter().map(|s| s.to_string()).collect::<Vec<_>>()}
                                value_prefix=""
                            />
                        }
                    }}
                </Card>
            </div>
        </div>
    }
}

/// `(room type, nights sold, revenue)`, busiest first.
fn group_room_types(nights: &[Night]) -> Vec<(String, u32, f64)> {
    let mut out: Vec<(String, u32, f64)> = Vec::new();
    for n in nights {
        let key = if n.room_type.is_empty() || n.room_type == "—" {
            "Unspecified".to_string()
        } else {
            n.room_type.clone()
        };
        match out.iter_mut().find(|(k, _, _)| *k == key) {
            Some(e) => {
                e.1 += 1;
                e.2 += n.rate;
            }
            None => out.push((key, 1, n.rate)),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

/// Top nationalities across the guest list, most guests first.
fn group_nationalities(rows: &[api::GuestRow]) -> Vec<(String, u32)> {
    let mut out: Vec<(String, u32)> = Vec::new();
    for g in rows {
        let key = g.nationality_str();
        if key == "—" {
            continue;
        }
        match out.iter_mut().find(|(k, _)| k == key) {
            Some(e) => e.1 += 1,
            None => out.push((key.to_string(), 1)),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out.truncate(8);
    out
}

/// Nights sold per weekday, Sunday first to match [`WEEKDAY_ABBR`].
fn group_weekdays(nights: &[Night]) -> Vec<u32> {
    let mut out = vec![0u32; 7];
    for n in nights {
        let idx = (days_from_epoch(n.date).rem_euclid(7) as usize + 4) % 7;
        out[idx] += 1;
    }
    out
}

/// Last bucket against the mean of the earlier ones, as a percentage.
fn delta_of(values: Vec<u32>) -> f32 {
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
