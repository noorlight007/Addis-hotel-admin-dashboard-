use crate::components::{
    use_toast, Badge, Card, Icon, Modal, PageHeader, ProgressBar, SegmentedControl, StatCard,
};
use crate::data::{AVAILABILITY, RATE_PLANS, ROOMS};
use leptos::prelude::*;

const TABS: &[&str] = &["Availability", "Rate plans"];

fn thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

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

#[component]
pub fn RatesPage() -> impl IntoView {
    let toast = use_toast();
    let tab = RwSignal::new(TABS[0]);
    let room_type = RwSignal::new("All room types");
    let editing = RwSignal::new(Option::<&'static str>::None);
    let bulk_open = RwSignal::new(false);
    let bulk_adjust = RwSignal::new(0i32);
    let closed_days = RwSignal::new(
        AVAILABILITY.iter().filter(|d| d.closed).map(|d| d.date).collect::<Vec<_>>(),
    );

    let toggle_closed = move |date: &'static str| {
        closed_days.update(|list| {
            if let Some(i) = list.iter().position(|d| *d == date) {
                list.remove(i);
            } else {
                list.push(date);
            }
        });
    };

    let total_rooms: u32 = AVAILABILITY.first().map(|d| d.total).unwrap_or(48);
    let sold_this_week: u32 = AVAILABILITY.iter().take(7).map(|d| d.sold).sum();
    let capacity_this_week = total_rooms * 7;
    let week_occupancy = sold_this_week * 100 / capacity_this_week.max(1);
    let avg_rate: u32 =
        AVAILABILITY.iter().map(|d| d.rate).sum::<u32>() / AVAILABILITY.len().max(1) as u32;

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Rates & availability"
                subtitle="Open and close dates, adjust nightly rates and manage rate plans."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| bulk_open.set(true)
                    class="sheen flex items-center gap-1.5 rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 active:scale-95"
                >
                    <Icon name="zap" class="h-4 w-4" />
                    "Bulk update"
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="bed"
                        label="Rooms in inventory"
                        value=total_rooms.to_string()
                        hint=Box::leak(format!("{} room types", ROOMS.len()).into_boxed_str())
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="trending-up"
                        label="Occupancy this week"
                        value=format!("{week_occupancy}%")
                        hint="Next 7 nights"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="tag"
                        label="Average rate"
                        value=format!("ETB {}", thousands(avg_rate))
                        hint="Across open dates"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="calendar-x"
                        label="Closed dates"
                        value=Signal::derive(move || closed_days.get().len().to_string())
                        hint="Not sellable"
                        accent="text-red-600 bg-red-50"
                    />
                </div>
            </div>

            // ================= AVAILABILITY =================
            <Show when=move || (tab.get() == "Availability")>
                <div class="flex animate-fade-up flex-col gap-5">
                    <Card
                        title="Next 14 nights"
                        hint="Click a date to open or close it for sale"
                    >
                        <div class="mb-4 flex flex-wrap items-center gap-2">
                            <select
                                class="rounded-lg border border-slate-300 px-3 py-2 text-sm transition-colors focus:border-blue-500 focus:outline-none"
                                on:change:target=move |ev| room_type.set(Box::leak(ev.target().value().into_boxed_str()))
                            >
                                <option>"All room types"</option>
                                {ROOMS.iter().map(|r| view! { <option>{r.room_type}</option> }).collect_view()}
                            </select>
                            <span class="ml-auto flex flex-wrap items-center gap-3 text-2xs text-slate-500">
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-emerald-200"></span>"Low"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-amber-400"></span>"Busy"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-red-500"></span>"Near full"</span>
                                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-slate-300"></span>"Closed"</span>
                            </span>
                        </div>

                        <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-7">
                            {AVAILABILITY.iter().enumerate().map(|(i, d)| {
                                let pct = d.sold * 100 / d.total.max(1);
                                let left = d.total.saturating_sub(d.sold);
                                let date = d.date;
                                let is_closed = move || closed_days.get().contains(&date);
                                let delay = format!("animation-delay: {}ms", i * 35);
                                view! {
                                    <button
                                        on:click=move |_| {
                                            toggle_closed(date);
                                            toast.info(
                                                if is_closed() { "Date closed" } else { "Date opened" },
                                                format!("{date} is now {} for sale.", if is_closed() { "closed" } else { "open" }),
                                            );
                                        }
                                        class=move || format!(
                                            "group animate-fade-up overflow-hidden rounded-xl border text-left transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md {}",
                                            if is_closed() { "border-slate-300 bg-slate-50 opacity-70" } else { "border-slate-200 bg-white" }
                                        )
                                        style=delay
                                    >
                                        <span class=move || format!(
                                            "flex items-center justify-between px-3 py-1.5 text-2xs font-bold {}",
                                            if is_closed() { "bg-slate-300 text-slate-700" } else { heat(pct) }
                                        )>
                                            <span>{d.label}</span>
                                            <span>{move || if is_closed() { "CLOSED".to_string() } else { format!("{pct}%") }}</span>
                                        </span>
                                        <span class="block p-3">
                                            <span class="block text-sm font-bold text-slate-900">{d.date}</span>
                                            <span class="mt-1 block text-2xs text-slate-500">
                                                {move || if is_closed() { "Not sellable".to_string() } else { format!("{left} of {} left", d.total) }}
                                            </span>
                                            <span class="mt-2 block text-sm font-bold text-blue-700">
                                                {format!("ETB {}", thousands(d.rate))}
                                            </span>
                                        </span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </Card>

                    <Card title="Inventory by room" hint="Rooms currently loaded into the calendar">
                        <div class="flex flex-col gap-3">
                            {ROOMS.iter().enumerate().map(|(i, r)| {
                                // Rooms not available today count as sold for this view.
                                let sold = if r.status == "Available" { 0 } else { 1 };
                                let pct = 40 + (i as u32 * 9) % 55;
                                view! {
                                    <div class="flex items-center gap-4">
                                        <span class="w-32 shrink-0 truncate text-sm font-semibold text-slate-800">
                                            {r.room_type}
                                        </span>
                                        <ProgressBar
                                            percent=pct
                                            fill={if pct >= 80 { "bg-red-500" } else if pct >= 60 { "bg-amber-500" } else { "bg-emerald-500" }}
                                        />
                                        <span class="w-24 shrink-0 text-right text-xs tabular-nums text-slate-500">
                                            {format!("{pct}% · ETB {}", thousands(r.price))}
                                        </span>
                                        <Badge
                                            label=if sold == 0 { "Open" } else { r.status }
                                            tone=if sold == 0 { "green" } else { "amber" }
                                        />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </Card>
                </div>
            </Show>

            // ================= RATE PLANS =================
            <Show when=move || (tab.get() == "Rate plans")>
                <div class="grid animate-fade-up gap-4 md:grid-cols-2">
                    {RATE_PLANS.iter().enumerate().map(|(i, p)| {
                        let delay = format!("animation-delay: {}ms", i * 60);
                        let active = RwSignal::new(p.active);
                        view! {
                            <div
                                class="card-hover animate-fade-up rounded-xl border border-slate-200 bg-white p-5 shadow-card"
                                style=delay
                            >
                                <div class="flex items-start justify-between gap-3">
                                    <div class="min-w-0">
                                        <div class="flex flex-wrap items-center gap-2">
                                            <h3 class="text-base font-bold text-slate-900">{p.name}</h3>
                                            <span class="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-2xs font-bold text-slate-600">
                                                {p.code}
                                            </span>
                                        </div>
                                        <p class="mt-1 text-sm leading-relaxed text-slate-500">{p.description}</p>
                                    </div>
                                    <label class="flex shrink-0 cursor-pointer items-center">
                                        <input
                                            type="checkbox"
                                            class="peer sr-only"
                                            prop:checked=active
                                            on:change:target=move |ev| {
                                                active.set(ev.target().checked());
                                                toast.success(
                                                    if ev.target().checked() { "Rate plan enabled" } else { "Rate plan paused" },
                                                    format!("{} is now {}.", p.name, if ev.target().checked() { "live" } else { "paused" }),
                                                );
                                            }
                                        />
                                        <span class="relative h-6 w-11 rounded-full bg-slate-300 transition-colors duration-200 after:absolute after:left-0.5 after:top-0.5 after:h-5 after:w-5 after:rounded-full after:bg-white after:shadow after:transition-transform after:duration-200 peer-checked:bg-blue-600 peer-checked:after:translate-x-5"></span>
                                    </label>
                                </div>

                                <dl class="mt-4 grid grid-cols-3 gap-3 border-t border-slate-100 pt-4 text-sm">
                                    <div>
                                        <dt class="text-2xs uppercase tracking-wide text-slate-400">"Adjustment"</dt>
                                        <dd class=format!(
                                            "mt-0.5 font-bold tabular-nums {}",
                                            if p.adjustment < 0 { "text-emerald-600" } else if p.adjustment > 0 { "text-amber-600" } else { "text-slate-800" }
                                        )>
                                            {if p.adjustment == 0 { "Base rate".to_string() } else { format!("{:+}%", p.adjustment) }}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-2xs uppercase tracking-wide text-slate-400">"Min. stay"</dt>
                                        <dd class="mt-0.5 font-bold text-slate-800">
                                            {format!("{} night{}", p.min_stay, if p.min_stay == 1 { "" } else { "s" })}
                                        </dd>
                                    </div>
                                    <div>
                                        <dt class="text-2xs uppercase tracking-wide text-slate-400">"Bookings"</dt>
                                        <dd class="mt-0.5 font-bold tabular-nums text-slate-800">{p.bookings}</dd>
                                    </div>
                                </dl>

                                <div class="mt-4 flex items-center justify-between gap-3 border-t border-slate-100 pt-4">
                                    <Badge
                                        label=if p.refundable { "Free cancellation" } else { "Non-refundable" }
                                        tone=if p.refundable { "green" } else { "amber" }
                                        dot=true
                                    />
                                    <button
                                        on:click=move |_| editing.set(Some(p.name))
                                        class="flex items-center gap-1.5 rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-semibold text-slate-700 transition-colors hover:border-blue-300 hover:text-blue-700"
                                    >
                                        <Icon name="edit" class="h-3.5 w-3.5" />
                                        "Edit"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect_view()}

                    <button
                        on:click=move |_| toast.info("New rate plan", "Rate plan creation opens in the next release.")
                        class="flex min-h-[13rem] animate-fade-up flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed border-slate-300 p-5 text-slate-500 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-400 hover:bg-blue-50/40 hover:text-blue-700"
                    >
                        <Icon name="plus" class="h-7 w-7" />
                        <span class="text-sm font-bold">"Create a rate plan"</span>
                        <span class="max-w-xs text-center text-xs">
                            "Seasonal pricing, package deals or a negotiated corporate rate."
                        </span>
                    </button>
                </div>
            </Show>
        </div>

        // ---- Bulk update ---------------------------------------------
        <Show when=move || bulk_open.get()>
            <Modal title="Bulk update rates" on_close=move || bulk_open.set(false)>
                <div class="flex flex-col gap-4">
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">"From"</label>
                            <input type="date" value="2026-09-01" class="w-full rounded-lg border border-slate-300 px-3 py-2.5 text-sm" />
                        </div>
                        <div>
                            <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">"To"</label>
                            <input type="date" value="2026-09-14" class="w-full rounded-lg border border-slate-300 px-3 py-2.5 text-sm" />
                        </div>
                    </div>

                    <div>
                        <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">"Room type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2.5 text-sm">
                            <option>"All room types"</option>
                            {ROOMS.iter().map(|r| view! { <option>{r.room_type}</option> }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">
                            "Rate adjustment"
                        </label>
                        <input
                            type="range"
                            min="-40"
                            max="40"
                            step="1"
                            class="w-full accent-blue-700"
                            prop:value=move || bulk_adjust.get().to_string()
                            on:input:target=move |ev| bulk_adjust.set(ev.target().value().parse().unwrap_or(0))
                        />
                        <div class="mt-1.5 flex items-center justify-between text-xs">
                            <span class="text-slate-400">"-40%"</span>
                            <span class=move || format!(
                                "rounded-md px-2 py-0.5 font-bold tabular-nums {}",
                                if bulk_adjust.get() < 0 { "bg-emerald-50 text-emerald-700" }
                                else if bulk_adjust.get() > 0 { "bg-amber-50 text-amber-700" }
                                else { "bg-slate-100 text-slate-600" }
                            )>
                                {move || format!("{:+}%", bulk_adjust.get())}
                            </span>
                            <span class="text-slate-400">"+40%"</span>
                        </div>
                    </div>

                    <div class="rounded-xl bg-slate-50 p-4 text-sm">
                        <p class="text-slate-500">"Example: a Standard Room currently at ETB 4,500 becomes"</p>
                        <p class="mt-1 text-lg font-bold text-slate-900">
                            {move || format!(
                                "ETB {}",
                                thousands(((4500i32 * (100 + bulk_adjust.get())) / 100).max(0) as u32)
                            )}
                        </p>
                    </div>

                    <div class="flex gap-2.5">
                        <button
                            on:click=move |_| bulk_open.set(false)
                            class="flex-1 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=move |_| {
                                bulk_open.set(false);
                                toast.success(
                                    "Rates updated",
                                    format!("Applied a {:+}% adjustment to 1–14 September.", bulk_adjust.get()),
                                );
                            }
                            class="flex-1 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-colors hover:bg-blue-800"
                        >
                            "Apply update"
                        </button>
                    </div>
                </div>
            </Modal>
        </Show>

        // ---- Edit rate plan ------------------------------------------
        <Show when=move || editing.get().is_some()>
            <Modal title="Edit rate plan" on_close=move || editing.set(None)>
                <p class="text-sm text-slate-600">
                    "Editing "
                    <span class="font-bold text-slate-900">{move || editing.get().unwrap_or("")}</span>
                    ". Changes apply to new reservations only — existing bookings keep the rate they were made at."
                </p>
                <div class="mt-4 flex gap-2.5">
                    <button
                        on:click=move |_| editing.set(None)
                        class="flex-1 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                    >
                        "Close"
                    </button>
                    <button
                        on:click=move |_| {
                            let name = editing.get().unwrap_or("");
                            editing.set(None);
                            toast.success("Rate plan saved", format!("{name} has been updated."));
                        }
                        class="flex-1 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-800"
                    >
                        "Save changes"
                    </button>
                </div>
            </Modal>
        </Show>
    }
}
