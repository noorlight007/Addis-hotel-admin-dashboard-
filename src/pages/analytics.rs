use crate::components::{
    BarChart, Card, DonutChart, Icon, LineChart, PageHeader, ProgressBar, SegmentedControl,
    StatCard, TrendPill,
};
use crate::data::{
    CHANNELS, DAY_LABELS, MONTH_LABELS, NATIONALITIES, OCCUPANCY_12M, OCCUPANCY_30D, OCCUPANCY_7D,
    REVENUE_12M, REVENUE_30D, REVENUE_7D, ROOM_TYPE_PERFORMANCE,
};
use leptos::prelude::*;

const RANGES: &[&str] = &["7 days", "30 days", "12 months"];

/// Labels for the 30-day series — every fifth day is named, the rest blank so
/// the axis stays readable.
fn thirty_day_labels() -> Vec<&'static str> {
    const L: [&str; 30] = [
        "1", "", "", "", "5", "", "", "", "", "10", "", "", "", "", "15", "", "", "", "", "20", "",
        "", "", "", "25", "", "", "", "", "30",
    ];
    L.to_vec()
}

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

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let range = RwSignal::new(RANGES[0]);

    let revenue_series = move || match range.get() {
        "30 days" => REVENUE_30D.to_vec(),
        "12 months" => REVENUE_12M.to_vec(),
        _ => REVENUE_7D.to_vec(),
    };
    let occupancy_series = move || match range.get() {
        "30 days" => OCCUPANCY_30D.to_vec(),
        "12 months" => OCCUPANCY_12M.to_vec(),
        _ => OCCUPANCY_7D.to_vec(),
    };
    let labels = move || match range.get() {
        "30 days" => thirty_day_labels(),
        "12 months" => MONTH_LABELS.to_vec(),
        _ => DAY_LABELS.to_vec(),
    };

    let total_revenue = move || revenue_series().iter().sum::<u32>();
    let avg_occupancy = move || {
        let s = occupancy_series();
        if s.is_empty() { 0 } else { s.iter().sum::<u32>() / s.len() as u32 }
    };

    let total_channel_bookings: u32 = CHANNELS.iter().map(|c| c.bookings).sum();
    let max_nationality = NATIONALITIES.iter().map(|n| n.guests).max().unwrap_or(1);

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Analytics"
                subtitle="Revenue, occupancy and demand for Golden Tulip Addis Ababa."
            >
                <SegmentedControl options=RANGES selected=range />
                <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50">
                    <Icon name="download" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Export"</span>
                </button>
            </PageHeader>

            // ---- Headline numbers -------------------------------------
            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="banknote"
                        label="Revenue"
                        value=Signal::derive(move || format!("ETB {}k", total_revenue()))
                        hint="Gross, before tax"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="bed"
                        label="Avg. occupancy"
                        value=Signal::derive(move || format!("{}%", avg_occupancy()))
                        hint="Across all room types"
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="tag"
                        label="Average daily rate"
                        value="ETB 5,842".to_string()
                        hint="Per occupied room"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="trending-up"
                        label="RevPAR"
                        value="ETB 4,673".to_string()
                        hint="Revenue per available room"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
            </div>

            // ---- Revenue ------------------------------------------------
            <div class="mb-5 animate-fade-up" style="animation-delay: 220ms">
                <Card title="Revenue trend" hint="Thousands of ETB">
                    <div class="mb-3 flex flex-wrap items-center gap-3">
                        <span class="text-2xl font-bold tabular-nums text-slate-900">
                            {move || format!("ETB {}k", total_revenue())}
                        </span>
                        <TrendPill delta=12.4 period="vs previous period" />
                    </div>
                    {move || view! {
                        <LineChart
                            values=revenue_series()
                            labels=labels()
                            stroke="#2563eb"
                            value_prefix="ETB "
                            height=220.0
                        />
                    }}
                </Card>
            </div>

            <div class="mb-5 grid animate-fade-up gap-5 lg:grid-cols-2" style="animation-delay: 260ms">
                <Card title="Occupancy" hint="Percentage of rooms sold">
                    <div class="mb-3 flex flex-wrap items-center gap-3">
                        <span class="text-2xl font-bold tabular-nums text-slate-900">
                            {move || format!("{}%", avg_occupancy())}
                        </span>
                        <TrendPill delta=5.8 period="vs previous period" />
                    </div>
                    {move || view! {
                        <LineChart
                            values=occupancy_series()
                            labels=labels()
                            stroke="#8b5cf6"
                            value_prefix=""
                            height=180.0
                        />
                    }}
                </Card>

                <Card title="Booking channels" hint=format!("{total_channel_bookings} bookings this period")>
                    <DonutChart
                        slices=CHANNELS.iter().map(|c| (c.name, c.bookings, c.color)).collect()
                        centre_label="Bookings"
                    />
                </Card>
            </div>

            <div class="mb-5 grid animate-fade-up gap-5 lg:grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)]" style="animation-delay: 300ms">
                // ---- Room type performance ------------------------------
                <Card title="Room type performance" hint="Rooms sold, revenue and occupancy">
                    <div class="overflow-x-auto">
                        <table class="w-full min-w-[34rem] text-left text-sm">
                            <thead class="border-b border-slate-200 text-2xs uppercase tracking-wide text-slate-500">
                                <tr>
                                    <th class="pb-2.5 pr-3">"Room type"</th>
                                    <th class="pb-2.5 px-3 text-right">"Sold"</th>
                                    <th class="pb-2.5 px-3 text-right">"ADR"</th>
                                    <th class="pb-2.5 px-3 text-right">"Revenue"</th>
                                    <th class="pb-2.5 pl-3 w-36">"Occupancy"</th>
                                </tr>
                            </thead>
                            <tbody class="stagger">
                                {ROOM_TYPE_PERFORMANCE.iter().map(|r| view! {
                                    <tr class="border-b border-slate-100 transition-colors last:border-0 hover:bg-slate-50">
                                        <td class="py-3 pr-3 font-semibold text-slate-800">{r.name}</td>
                                        <td class="px-3 py-3 text-right tabular-nums text-slate-600">{r.sold}</td>
                                        <td class="px-3 py-3 text-right tabular-nums text-slate-600">
                                            {format!("ETB {}", thousands(r.adr))}
                                        </td>
                                        <td class="px-3 py-3 text-right font-semibold tabular-nums text-slate-900">
                                            {format!("ETB {}", thousands(r.revenue))}
                                        </td>
                                        <td class="py-3 pl-3">
                                            <div class="flex items-center gap-2">
                                                <ProgressBar
                                                    percent=r.occupancy
                                                    fill={if r.occupancy >= 80 { "bg-emerald-500" } else if r.occupancy >= 65 { "bg-blue-500" } else { "bg-amber-500" }}
                                                />
                                                <span class="w-9 shrink-0 text-right text-xs font-semibold tabular-nums text-slate-600">
                                                    {format!("{}%", r.occupancy)}
                                                </span>
                                            </div>
                                        </td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                </Card>

                // ---- Guest origin ---------------------------------------
                <Card title="Guest nationality" hint="Where your guests travel from">
                    <ul class="flex flex-col gap-3">
                        {NATIONALITIES.iter().enumerate().map(|(i, n)| {
                            let pct = n.guests * 100 / max_nationality;
                            let delay = format!("animation-delay: {}ms", i * 50);
                            view! {
                                <li class="animate-fade-up" style=delay>
                                    <div class="mb-1 flex items-center justify-between text-sm">
                                        <span class="flex items-center gap-2 text-slate-700">
                                            <Icon name="globe" class="h-3.5 w-3.5 text-slate-400" />
                                            {n.country}
                                        </span>
                                        <span class="font-semibold tabular-nums text-slate-900">{n.guests}</span>
                                    </div>
                                    <ProgressBar percent=pct fill="bg-blue-500" height="h-1.5" />
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </Card>
            </div>

            // ---- Weekday demand -----------------------------------------
            <div class="animate-fade-up" style="animation-delay: 340ms">
                <Card title="Demand by weekday" hint="Average rooms sold per night over the last 90 days">
                    <BarChart
                        values=vec![29, 33, 31, 39, 45, 47, 36]
                        labels=DAY_LABELS.to_vec()
                        bar_class="bg-gradient-to-t from-blue-600 to-blue-400"
                        value_prefix=""
                    />
                    <div class="mt-4 flex flex-wrap gap-3 border-t border-slate-100 pt-4 text-xs text-slate-500">
                        <span class="flex items-center gap-1.5">
                            <Icon name="trending-up" class="h-3.5 w-3.5 text-emerald-600" />
                            "Friday and Saturday run near capacity — consider a weekend premium."
                        </span>
                    </div>
                </Card>
            </div>
        </div>
    }
}
