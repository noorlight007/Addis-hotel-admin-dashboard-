//! Rates and availability, backed by `/rooms/` and `/rooms/calendar-matrix/`.
//!
//! Two tabs over the same inventory:
//!
//! * **Availability** — the next 14 nights, with sold/left counts read off the
//!   availability matrix and a click to block or unblock a night through
//!   `/rooms/block-dates/` and `/rooms/special-events/{id}/`.
//! * **Rate plans** — the nightly price and discount of every room, editable
//!   inline through `PATCH /rooms/{id}/`, plus a bulk percentage adjustment
//!   applied room by room.
//!
//! There is no rate-plan resource in the API: a room's price *is* its rate, so
//! "rate plans" here means the per-room pricing table.

use crate::api;
use crate::components::{
    use_toast, Card, Icon, Modal, PageHeader, ProgressBar, SegmentedControl, StatCard,
};
use crate::date::{add_days, format_short, to_iso, today, WEEKDAY_ABBR, days_from_epoch};
use leptos::prelude::*;

const TABS: &[&str] = &["Availability", "Rate plans"];
const HORIZON: i64 = 14;

/// Colour band for an occupancy percentage.
fn heat(pct: u32) -> &'static str {
    match pct {
        p if p >= 95 => "bg-red-500 text-white",
        p if p >= 85 => "bg-orange-500 text-white",
        p if p >= 70 => "bg-amber-400 text-amber-950",
        p if p >= 50 => "bg-emerald-400 text-emerald-950",
        _ => "bg-emerald-200 text-emerald-900",
    }
}

/// What the matrix says about one night across the whole property.
#[derive(Clone, Default)]
struct NightSummary {
    iso: String,
    label: String,
    weekday: String,
    total: u32,
    sold: u32,
    blocked: u32,
    /// Blocks covering this night, so the cell can offer to lift them.
    event_ids: Vec<i64>,
}

impl NightSummary {
    fn pct(&self) -> u32 {
        let used = self.sold + self.blocked;
        if self.total == 0 {
            0
        } else {
            (used * 100 / self.total).min(100)
        }
    }
    fn left(&self) -> u32 {
        self.total.saturating_sub(self.sold + self.blocked)
    }
    fn is_closed(&self) -> bool {
        self.total > 0 && self.left() == 0 && self.blocked > 0
    }
}

/// Folds the matrix into a per-night property-wide view for the next 14 nights.
///
/// The matrix repeats each blocking entry once per night without saying which,
/// so [`api::CalendarRoom::blocked_days`] is used to place them on the grid.
fn summarise(matrix: &api::CalendarMatrix, extra: Option<&api::CalendarMatrix>) -> Vec<NightSummary> {
    let t = today();
    let total = matrix.total_rooms;
    (0..HORIZON)
        .map(|i| {
            let d = add_days(t, i);
            let mut s = NightSummary {
                iso: to_iso(d),
                label: format_short(d),
                weekday: WEEKDAY_ABBR[(days_from_epoch(d).rem_euclid(7) as usize + 4) % 7].to_string(),
                total,
                ..Default::default()
            };
            // A 14-night horizon can cross a month boundary, so both months are
            // consulted and the one matching the cell's month wins.
            let source = if d.1 == matrix.month { matrix } else { extra.unwrap_or(matrix) };
            for room in &source.matrix {
                for (day, entry) in room.blocked_days(d.0, d.1) {
                    if day != d.2 {
                        continue;
                    }
                    if entry.reservation_id.is_some() {
                        s.sold += 1;
                    } else {
                        s.blocked += 1;
                        if let Some(id) = entry.event_id {
                            if !s.event_ids.contains(&id) {
                                s.event_ids.push(id);
                            }
                        }
                    }
                }
            }
            s
        })
        .collect()
}

#[component]
pub fn RatesPage() -> impl IntoView {
    let toast = use_toast();
    let tab = RwSignal::new(TABS[0]);
    let type_filter = RwSignal::new("All".to_string());
    let bulk_open = RwSignal::new(false);
    let refresh = RwSignal::new(0u32);
    let busy_night = RwSignal::new(Option::<String>::None);

    let t = today();
    let next_month = crate::date::shift_month(t.0, t.1, 1);

    let rooms = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::all_rooms().await }
    });
    let matrix = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::calendar_matrix(t.0, t.1, "All", "").await }
    });
    let matrix_next = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::calendar_matrix(next_month.0, next_month.1, "All", "").await }
    });

    let nights = move || {
        let (Some(Ok(m)), n) = (matrix.get(), matrix_next.get()) else {
            return Vec::new();
        };
        let next = n.and_then(Result::ok);
        summarise(&m, next.as_ref())
    };

    let room_rows = move || rooms.get().and_then(Result::ok).unwrap_or_default();
    let room_types = move || {
        let mut list: Vec<String> = room_rows()
            .iter()
            .filter_map(|r| r.room_type.clone())
            .collect();
        list.sort();
        list.dedup();
        list
    };
    let filtered_rooms = move || {
        let f = type_filter.get();
        room_rows()
            .into_iter()
            .filter(|r| f == "All" || r.room_type.as_deref() == Some(f.as_str()))
            .collect::<Vec<_>>()
    };

    // ---- Tiles -------------------------------------------------------------
    let week = move || {
        let all = nights();
        all.into_iter().take(7).collect::<Vec<_>>()
    };
    let week_occupancy = move || {
        let w = week();
        let capacity: u32 = w.iter().map(|n| n.total).sum();
        let used: u32 = w.iter().map(|n| n.sold).sum();
        if capacity == 0 { 0 } else { used * 100 / capacity }
    };
    let avg_rate = move || {
        let rows = room_rows();
        if rows.is_empty() {
            return 0.0;
        }
        let sum: f64 = rows
            .iter()
            .filter_map(|r| {
                r.price_after_discount
                    .as_deref()
                    .or(r.price_per_night.as_deref())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .sum();
        sum / rows.len() as f64
    };
    let closed_count = move || nights().iter().filter(|n| n.is_closed()).count();

    // Blocks or unblocks a whole night across the property.
    let toggle_night = move |n: NightSummary| {
        if busy_night.get().is_some() {
            return;
        }
        busy_night.set(Some(n.iso.clone()));
        let ids: Vec<i64> = room_rows().iter().map(|r| r.id).collect();
        let iso = n.iso.clone();
        let events = n.event_ids.clone();
        let closing = !n.is_closed();

        wasm_bindgen_futures::spawn_local(async move {
            let result = if closing {
                api::block_dates(&ids, "Blocked", &iso, &iso, "Closed for sale from Rates").await
            } else {
                // Unblocking means ending every block that covers the night.
                let mut last = Ok(());
                for id in events {
                    last = api::end_special_event(id).await;
                }
                last
            };
            match result {
                Ok(()) => {
                    toast.success(
                        if closing { "Date closed" } else { "Date opened" },
                        format!("{iso} is now {} for sale.", if closing { "closed" } else { "open" }),
                    );
                    refresh.update(|x| *x += 1);
                }
                Err(e) => toast.error("Could not update the date", e.detail()),
            }
            busy_night.set(None);
        });
    };

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Rates & availability"
                subtitle="Nightly pricing and what is open for sale."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| bulk_open.set(true)
                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                >
                    <Icon name="tag" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Bulk adjust"</span>
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="bed"
                        label="Rooms loaded"
                        value=Signal::derive(move || room_rows().len().to_string())
                        hint="In the rate table"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="trending-up"
                        label="Occupancy this week"
                        value=Signal::derive(move || format!("{}%", week_occupancy()))
                        hint="Next 7 nights"
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="tag"
                        label="Average rate"
                        value=Signal::derive(move || format!("ETB {}", api::money_round(Some(&format!("{:.2}", avg_rate())))))
                        hint="Across all rooms"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="calendar-x"
                        label="Closed nights"
                        value=Signal::derive(move || closed_count().to_string())
                        hint="In the next 14"
                        accent="text-red-600 bg-red-50"
                    />
                </div>
            </div>

            // ================= AVAILABILITY =================
            <Show when=move || (tab.get() == "Availability")>
                <div class="flex animate-fade-up flex-col gap-5">
                    <Card title="Next 14 nights" hint="Click a night to close or reopen it across every room">
                        <div class="mb-4 flex flex-wrap items-center gap-2">
                            <span class="ml-auto flex flex-wrap items-center gap-3 text-2xs text-slate-500">
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-emerald-200"></span>"Low"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-amber-400"></span>"Busy"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-red-500"></span>"Near full"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-slate-300"></span>"Closed"</span>
                            </span>
                        </div>

                        <Suspense fallback=|| view! {
                            <p class="py-10 text-center text-sm text-slate-400">"Loading availability…"</p>
                        }>
                            {move || Suspend::new(async move {
                                if let Err(e) = matrix.await {
                                    return view! {
                                        <p class="py-10 text-center text-sm text-red-600">{e.detail()}</p>
                                    }.into_any();
                                }
                                let list = nights();
                                if list.is_empty() {
                                    return view! {
                                        <p class="py-10 text-center text-sm text-slate-400">"No rooms in inventory yet."</p>
                                    }.into_any();
                                }
                                view! {
                                    <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-7">
                                        {list.into_iter().enumerate().map(|(i, n)| {
                                            let closed = n.is_closed();
                                            let pct = n.pct();
                                            let left = n.left();
                                            let total = n.total;
                                            let iso = n.iso.clone();
                                            let cell = n.clone();
                                            let delay = format!("animation-delay: {}ms", i * 35);
                                            view! {
                                                <button
                                                    on:click=move |_| toggle_night(cell.clone())
                                                    disabled=move || busy_night.get().is_some()
                                                    class=format!(
                                                        "group animate-fade-up overflow-hidden rounded-xl border text-left transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md disabled:opacity-60 {}",
                                                        if closed { "border-slate-300 bg-slate-50 opacity-70" } else { "border-slate-200 bg-white" }
                                                    )
                                                    style=delay
                                                >
                                                    <span class=format!(
                                                        "flex items-center justify-between px-3 py-1.5 text-2xs font-bold {}",
                                                        if closed { "bg-slate-300 text-slate-700" } else { heat(pct) }
                                                    )>
                                                        <span>{n.weekday.clone()}</span>
                                                        <span>{if closed { "CLOSED".to_string() } else { format!("{pct}%") }}</span>
                                                    </span>
                                                    <span class="block p-3">
                                                        <span class="block text-sm font-bold text-slate-900">{n.label.clone()}</span>
                                                        <span class="mt-1 block text-2xs text-slate-500">
                                                            {if closed {
                                                                "Not sellable".to_string()
                                                            } else {
                                                                format!("{left} of {total} left")
                                                            }}
                                                        </span>
                                                        <span class="mt-2 block text-2xs text-slate-400">{iso}</span>
                                                    </span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            })}
                        </Suspense>
                    </Card>

                    <Card title="Inventory by room" hint="Status and nightly rate">
                        <Suspense fallback=|| view! {
                            <p class="py-6 text-center text-sm text-slate-400">"Loading rooms…"</p>
                        }>
                            {move || Suspend::new(async move {
                                let rows = rooms.await.unwrap_or_default();
                                if rows.is_empty() {
                                    return view! {
                                        <p class="py-6 text-center text-sm text-slate-400">"No rooms yet."</p>
                                    }.into_any();
                                }
                                let max = rows
                                    .iter()
                                    .filter_map(|r| r.price_per_night.as_deref().and_then(|s| s.parse::<f64>().ok()))
                                    .fold(1.0f64, f64::max);
                                view! {
                                    <div class="flex flex-col gap-3">
                                        {rows.into_iter().map(|r| {
                                            let price = r
                                                .price_after_discount
                                                .as_deref()
                                                .or(r.price_per_night.as_deref())
                                                .and_then(|s| s.parse::<f64>().ok())
                                                .unwrap_or(0.0);
                                            let pct = ((price / max) * 100.0).round().min(100.0) as u32;
                                            view! {
                                                <div class="flex items-center gap-4">
                                                    <span class="w-36 shrink-0 truncate text-sm font-semibold text-slate-800">
                                                        {format!("{} · {}", r.room_number, r.type_str())}
                                                    </span>
                                                    <ProgressBar
                                                        percent=pct
                                                        fill={if r.status_str() == "Available" { "bg-emerald-500" } else if r.status_str() == "Maintenance" { "bg-red-500" } else { "bg-amber-500" }}
                                                    />
                                                    <span class="w-40 shrink-0 text-right text-xs tabular-nums text-slate-500">
                                                        {format!("{} · ETB {}", r.status_str(), r.price_display())}
                                                    </span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            })}
                        </Suspense>
                    </Card>
                </div>
            </Show>

            // ================= RATE PLANS =================
            <Show when=move || (tab.get() == "Rate plans")>
                <div class="animate-fade-up">
                    <Card title="Nightly rates" hint="Edit a rate or discount to publish it immediately">
                        <div class="mb-4 flex flex-wrap items-center gap-2">
                            <select
                                class="rounded-lg border border-slate-300 px-3 py-2 text-sm transition-colors focus:border-blue-500 focus:outline-none"
                                on:change:target=move |ev| type_filter.set(ev.target().value())
                            >
                                <option value="All">"All room types"</option>
                                {move || room_types().into_iter().map(|t| {
                                    let value = t.clone();
                                    view! { <option value=value>{t}</option> }
                                }).collect_view()}
                            </select>
                        </div>

                        <Suspense fallback=|| view! {
                            <p class="py-10 text-center text-sm text-slate-400">"Loading rates…"</p>
                        }>
                            {move || Suspend::new(async move {
                                if let Err(e) = rooms.await {
                                    return view! {
                                        <p class="py-10 text-center text-sm text-red-600">{e.detail()}</p>
                                    }.into_any();
                                }
                                let rows = filtered_rooms();
                                if rows.is_empty() {
                                    return view! {
                                        <p class="py-10 text-center text-sm text-slate-400">"No rooms match this filter."</p>
                                    }.into_any();
                                }
                                view! {
                                    <div class="-mx-5 overflow-x-auto">
                                        <table class="w-full text-left text-sm">
                                            <thead class="border-b border-slate-100 text-2xs uppercase text-slate-500">
                                                <tr>
                                                    <th class="px-5 py-2">"Room"</th>
                                                    <th class="px-5 py-2">"Type"</th>
                                                    <th class="px-5 py-2">"Capacity"</th>
                                                    <th class="px-5 py-2">"Base rate"</th>
                                                    <th class="px-5 py-2">"Discount %"</th>
                                                    <th class="px-5 py-2">"Sells at"</th>
                                                    <th class="px-5 py-2"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {rows.into_iter().map(|r| view! {
                                                    <RateRow room=r on_saved=move || refresh.update(|n| *n += 1) />
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            })}
                        </Suspense>
                    </Card>
                </div>
            </Show>

            <Show when=move || bulk_open.get()>
                <BulkRateModal
                    rooms=Signal::derive(filtered_rooms)
                    on_close=move || bulk_open.set(false)
                    on_applied=move || refresh.update(|n| *n += 1)
                />
            </Show>
        </div>
    }
}

/// One editable rate row. Saving PATCHes just the two pricing fields.
#[component]
fn RateRow(
    room: api::RoomRow,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = room.id;
    let price = RwSignal::new(room.price_per_night.clone().unwrap_or_default());
    let discount = RwSignal::new(
        room.discount_percent_per_night
            .unwrap_or(0)
            .to_string(),
    );
    let busy = RwSignal::new(false);
    let original = (
        room.price_per_night.clone().unwrap_or_default(),
        room.discount_percent_per_night.unwrap_or(0).to_string(),
    );
    let dirty = move || price.get() != original.0 || discount.get() != original.1;

    // What the guest will actually pay, recomputed as the inputs change.
    let sells_at = move || {
        let base = price.get().trim().parse::<f64>().unwrap_or(0.0);
        let pct = discount.get().trim().parse::<f64>().unwrap_or(0.0).clamp(0.0, 100.0);
        base * (1.0 - pct / 100.0)
    };

    let save = move |_| {
        if busy.get() {
            return;
        }
        let base = match price.get().trim().parse::<f64>() {
            Ok(n) if n > 0.0 => n,
            _ => {
                toast.error("Invalid rate", "Enter a nightly price greater than zero.");
                return;
            }
        };
        let pct: i32 = discount.get().trim().parse().unwrap_or(0);
        if !(0..=100).contains(&pct) {
            toast.error("Invalid discount", "The discount must be between 0 and 100.");
            return;
        }
        busy.set(true);
        let body = serde_json::json!({
            "price_per_night": format!("{base:.2}"),
            "discount_percent_per_night": pct,
        });
        wasm_bindgen_futures::spawn_local(async move {
            match api::patch_room(id, body).await {
                Ok(_) => {
                    toast.success("Rate updated", "The new price is live on the portal.");
                    on_saved();
                }
                Err(e) => toast.error("Could not update the rate", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <tr class="border-b border-slate-50 last:border-0">
            <td class="px-5 py-2.5 font-semibold text-slate-900">{room.room_number.clone()}</td>
            <td class="px-5 py-2.5 text-slate-600">{room.type_str().to_string()}</td>
            <td class="px-5 py-2.5 tabular-nums">{room.guest_capacity}</td>
            <td class="px-5 py-2.5">
                <input type="number" step="0.01" min="0"
                    class="w-28 rounded-lg border border-slate-300 px-2 py-1.5 text-sm tabular-nums"
                    prop:value=price on:input:target=move |ev| price.set(ev.target().value()) />
            </td>
            <td class="px-5 py-2.5">
                <input type="number" min="0" max="100"
                    class="w-20 rounded-lg border border-slate-300 px-2 py-1.5 text-sm tabular-nums"
                    prop:value=discount on:input:target=move |ev| discount.set(ev.target().value()) />
            </td>
            <td class="px-5 py-2.5 font-bold tabular-nums text-blue-700">
                {move || format!("ETB {}", api::money_round(Some(&format!("{:.2}", sells_at()))))}
            </td>
            <td class="px-5 py-2.5 text-right">
                <button
                    on:click=save
                    disabled=move || busy.get() || !dirty()
                    class="rounded-lg bg-blue-700 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-blue-800 disabled:opacity-40"
                >
                    {move || if busy.get() { "Saving…" } else { "Save" }}
                </button>
            </td>
        </tr>
    }
}

/// Applies a percentage change to every room in the current filter.
///
/// The API has no bulk-pricing endpoint, so this is a sequence of
/// `PATCH /rooms/{id}/` calls; the result reports how many succeeded.
#[component]
fn BulkRateModal(
    rooms: Signal<Vec<api::RoomRow>>,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_applied: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let percent = RwSignal::new("0".to_string());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let apply = move |_| {
        if busy.get() {
            return;
        }
        let pct: f64 = match percent.get().trim().parse() {
            Ok(n) => n,
            Err(_) => {
                error.set(Some("Enter a percentage, e.g. 10 or -5.".into()));
                return;
            }
        };
        if pct == 0.0 {
            error.set(Some("Enter a non-zero percentage.".into()));
            return;
        }
        let targets = rooms.get();
        if targets.is_empty() {
            error.set(Some("No rooms in the current filter.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            let mut ok = 0u32;
            let mut failed = 0u32;
            for r in targets {
                let base = r
                    .price_per_night
                    .as_deref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                if base <= 0.0 {
                    failed += 1;
                    continue;
                }
                let next = (base * (1.0 + pct / 100.0)).max(1.0);
                let body = serde_json::json!({ "price_per_night": format!("{next:.2}") });
                match api::patch_room(r.id, body).await {
                    Ok(_) => ok += 1,
                    Err(_) => failed += 1,
                }
            }
            busy.set(false);
            if failed == 0 {
                toast.success("Rates adjusted", format!("{ok} rooms updated."));
            } else {
                toast.warning(
                    "Partly applied",
                    format!("{ok} rooms updated, {failed} could not be changed."),
                );
            }
            on_applied();
            on_close();
        });
    };

    view! {
        <Modal title="Bulk rate adjustment" on_close=on_close>
            <div class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <p class="text-sm text-slate-600">
                    {move || format!(
                        "This changes the base rate of {} room(s) in the current filter. Use a negative value to reduce prices.",
                        rooms.get().len(),
                    )}
                </p>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Adjustment %"</label>
                    <input type="number" step="0.5" placeholder="e.g. 10"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=percent on:input:target=move |ev| percent.set(ev.target().value()) />
                </div>

                <div class="rounded-lg border border-slate-200 bg-slate-50 p-3 text-xs text-slate-600">
                    <p class="mb-1.5 font-semibold text-slate-700">"Preview"</p>
                    {move || {
                        let pct = percent.get().trim().parse::<f64>().unwrap_or(0.0);
                        rooms.get().into_iter().take(4).map(|r| {
                            let base = r.price_per_night.as_deref().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                            let next = (base * (1.0 + pct / 100.0)).max(1.0);
                            view! {
                                <p class="tabular-nums">
                                    {format!(
                                        "{} · ETB {} → ETB {}",
                                        r.room_number,
                                        api::money_round(Some(&format!("{base:.2}"))),
                                        api::money_round(Some(&format!("{next:.2}"))),
                                    )}
                                </p>
                            }
                        }).collect_view()
                    }}
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="button" on:click=apply disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Applying…" } else { "Apply to all" }}
                    </button>
                </div>
            </div>
        </Modal>
    }
}
