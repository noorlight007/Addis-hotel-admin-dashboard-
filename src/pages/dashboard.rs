use crate::components::{
    pluralize, Avatar, Badge, Card, DonutChart, Icon, LineChart, ProgressBar, SegmentedControl,
    StatCard, TrendPill,
};
use crate::data::{
    Guest, ACTIVITY, BOOKINGS_7D, CHANNELS, DAY_LABELS, GUESTS, OCCUPANCY_7D, RESERVATIONS,
    REVENUE_7D, ROOMS,
};
use crate::date::{format_long, today};
use leptos::prelude::*;
use leptos_router::components::A;

const CHART_TABS: &[&str] = &["Revenue", "Occupancy", "Bookings"];

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
pub fn DashboardPage() -> impl IntoView {
    let chart_tab = RwSignal::new(CHART_TABS[0]);

    let room_status = |status: &str| ROOMS.iter().filter(|r| r.status == status).count() as u32;
    let occupied = room_status("Occupied");
    let available = room_status("Available");
    let reserved = room_status("Reserved");
    let maintenance = room_status("Maintenance");
    let total_rooms = ROOMS.len() as u32;
    let occupancy_pct = (occupied + reserved) * 100 / total_rooms.max(1);

    let arrivals: Vec<&'static Guest> =
        GUESTS.iter().filter(|g| g.status == "Arriving Today").collect();
    let departures: Vec<&'static Guest> =
        GUESTS.iter().filter(|g| g.status == "Checking Out").collect();
    let in_house = GUESTS.iter().filter(|g| g.status == "In House").count();
    let arrivals_count = arrivals.len();
    let departures_count = departures.len();

    let series = move || match chart_tab.get() {
        "Occupancy" => OCCUPANCY_7D.to_vec(),
        "Bookings" => BOOKINGS_7D.to_vec(),
        _ => REVENUE_7D.to_vec(),
    };
    let series_total = move || series().iter().sum::<u32>();
    let series_label = move || match chart_tab.get() {
        "Occupancy" => format!("{}% average", series_total() / 7),
        "Bookings" => format!("{} bookings", series_total()),
        _ => format!("ETB {}k", series_total()),
    };
    let series_colour = move || match chart_tab.get() {
        "Occupancy" => "#8b5cf6",
        "Bookings" => "#0ea5e9",
        _ => "#2563eb",
    };

    view! {
        <div class="p-4 sm:p-6">
            // ---- Welcome banner ---------------------------------------
            <div class="mb-5 flex animate-fade-up flex-wrap items-center justify-between gap-4 overflow-hidden rounded-2xl bg-gradient-to-r from-blue-700 via-blue-700 to-indigo-800 p-5 text-white shadow-lg shadow-blue-900/20 sm:p-6">
                <div class="relative min-w-0">
                    <span class="pointer-events-none absolute -left-16 -top-16 h-40 w-40 rounded-full bg-white/10 blur-2xl"></span>
                    <h2 class="relative text-xl font-bold sm:text-2xl">"Good morning, Ahmed"</h2>
                    <p class="relative mt-1 text-sm text-blue-100">
                        {format!(
                            "{}, {} and {} in house today.",
                            pluralize(arrivals.len(), "arrival"),
                            pluralize(departures.len(), "departure"),
                            pluralize(in_house, "guest")
                        )}
                    </p>
                </div>
                <div class="flex shrink-0 flex-wrap items-center gap-2">
                    <span class="flex items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-sm font-medium ring-1 ring-white/20 backdrop-blur-sm">
                        <Icon name="calendar" class="h-4 w-4" />
                        {format_long(today())}
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

            // ---- Stat row ----------------------------------------------
            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="calendar-check"
                        label="New reservations"
                        value="7".to_string()
                        hint="Received today"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="check-circle"
                        label="Check-ins today"
                        value=arrivals_count.to_string()
                        hint="Guests arriving"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="bed"
                        label="Rooms available"
                        value=available.to_string()
                        hint="Ready to sell"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="trending-up"
                        label="Occupancy"
                        value=format!("{occupancy_pct}%")
                        hint="Right now"
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
                                        <TrendPill delta=12.4 />
                                    </div>
                                </div>
                                <SegmentedControl options=CHART_TABS selected=chart_tab />
                            </div>

                            {move || view! {
                                <LineChart
                                    values=series()
                                    labels=DAY_LABELS.to_vec()
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
                            hint=format!("{} expected", pluralize(arrivals_count, "guest"))
                        >
                            {if arrivals.is_empty() {
                                view! {
                                    <p class="py-6 text-center text-sm text-slate-400">"No arrivals scheduled."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="stagger flex flex-col gap-2.5">
                                        {arrivals.into_iter().map(|g| view! {
                                            <A
                                                href=format!("/guests/{}", g.booking_ref)
                                                attr:class="group flex items-center gap-3 rounded-lg border border-slate-100 p-2.5 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:bg-blue-50/40 hover:shadow-sm"
                                            >
                                                <Avatar initials=g.initials size="h-9 w-9 text-2xs" />
                                                <span class="min-w-0 flex-1">
                                                    <span class="flex items-center gap-1.5">
                                                        <span class="truncate text-sm font-semibold text-slate-900">{g.name}</span>
                                                        <Show when=move || g.vip>
                                                            <span class="rounded bg-purple-100 px-1 py-0.5 text-[9px] font-bold text-purple-700">"VIP"</span>
                                                        </Show>
                                                    </span>
                                                    <span class="block truncate text-2xs text-slate-500">
                                                        {format!("Room {} · {} · {}", g.room, g.room_type, g.guests)}
                                                    </span>
                                                </span>
                                                <Icon name="chevron-right" class="h-4 w-4 shrink-0 text-slate-300 transition-transform duration-200 group-hover:translate-x-0.5" />
                                            </A>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </Card>

                        <Card
                            title="Checking out today"
                            hint=format!("{} turning over", pluralize(departures_count, "room"))
                        >
                            {if departures.is_empty() {
                                view! {
                                    <p class="py-6 text-center text-sm text-slate-400">"No departures scheduled."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="stagger flex flex-col gap-2.5">
                                        {departures.into_iter().map(|g| view! {
                                            <A
                                                href=format!("/guests/{}", g.booking_ref)
                                                attr:class="group flex items-center gap-3 rounded-lg border border-slate-100 p-2.5 transition-all duration-200 hover:-translate-y-0.5 hover:border-amber-200 hover:bg-amber-50/40 hover:shadow-sm"
                                            >
                                                <Avatar initials=g.initials size="h-9 w-9 text-2xs" gradient="from-amber-500 to-orange-600" />
                                                <span class="min-w-0 flex-1">
                                                    <span class="block truncate text-sm font-semibold text-slate-900">{g.name}</span>
                                                    <span class="block truncate text-2xs text-slate-500">
                                                        {format!("Room {} · out by {}", g.room, g.check_out_date)}
                                                    </span>
                                                </span>
                                                <Icon name="chevron-right" class="h-4 w-4 shrink-0 text-slate-300 transition-transform duration-200 group-hover:translate-x-0.5" />
                                            </A>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
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
                            <div class="stagger flex flex-col gap-2.5">
                                {RESERVATIONS.iter().take(5).map(|r| view! {
                                    <div class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-slate-100 p-3 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:bg-blue-50/40 hover:shadow-sm">
                                        <div class="min-w-0">
                                            <p class="flex items-center gap-2 text-sm font-semibold text-slate-900">
                                                {r.booking_ref}
                                                <Badge
                                                    label=r.status
                                                    tone=status_tone(r.status)
                                                    dot=(r.status == "New")
                                                />
                                            </p>
                                            <p class="mt-0.5 truncate text-xs text-slate-500">
                                                {format!("{} · {} · {}", r.guest, pluralize(r.guests as usize, "guest"), r.phone)}
                                            </p>
                                        </div>
                                        <div class="shrink-0 text-right">
                                            <p class="text-xs font-semibold text-slate-700">
                                                {format!("{} – {}", r.check_in, r.check_out)}
                                            </p>
                                            <p class="text-2xs text-slate-400">{format!("Booked {}", r.time)}</p>
                                        </div>
                                    </div>
                                }).collect_view()}
                            </div>
                        </Card>
                    </div>
                </div>

                // ================= RIGHT RAIL =================
                <div class="flex min-w-0 flex-col gap-5">
                    // ---- Room status ------------------------------------
                    <div class="animate-fade-up" style="animation-delay: 240ms">
                        <Card title="Room status" hint=format!("{total_rooms} rooms in inventory")>
                            <DonutChart
                                slices=vec![
                                    ("Available", available, "#10b981"),
                                    ("Occupied", occupied, "#2563eb"),
                                    ("Reserved", reserved, "#f59e0b"),
                                    ("Maintenance", maintenance, "#ef4444"),
                                ]
                                centre_label="Rooms"
                            />
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
                                    ("calendar", "Block dates", "/rates"),
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
                                {[
                                    ("Total bookings", "7", "calendar-check"),
                                    ("Room revenue", "ETB 43,200", "banknote"),
                                    ("Check-ins", "18", "check-circle"),
                                    ("Check-outs", "6", "log-out"),
                                    ("Cancellations", "1", "x-circle"),
                                ].into_iter().map(|(label, value, icon)| view! {
                                    <div class="flex items-center justify-between gap-3">
                                        <dt class="flex items-center gap-2 text-slate-500">
                                            <Icon name=icon class="h-3.5 w-3.5 text-slate-400" />
                                            {label}
                                        </dt>
                                        <dd class="font-bold tabular-nums text-slate-900">{value}</dd>
                                    </div>
                                }).collect_view()}
                            </dl>

                            <div class="mt-4 border-t border-slate-100 pt-4">
                                <div class="mb-1.5 flex items-center justify-between text-xs">
                                    <span class="font-semibold text-slate-600">"Monthly revenue target"</span>
                                    <span class="font-bold tabular-nums text-slate-900">"78%"</span>
                                </div>
                                <ProgressBar percent=78 fill="bg-gradient-to-r from-blue-600 to-indigo-600" />
                                <p class="mt-1.5 text-2xs text-slate-400">
                                    {format!("ETB {} of ETB 1,500,000", thousands(1_170_000))}
                                </p>
                            </div>
                        </Card>
                    </div>

                    // ---- Channels ----------------------------------------
                    <div class="animate-fade-up" style="animation-delay: 360ms">
                        <Card title="Booking channels" hint="Last 30 days">
                            <ul class="flex flex-col gap-3">
                                {CHANNELS.iter().map(|c| {
                                    let total: u32 = CHANNELS.iter().map(|x| x.bookings).sum();
                                    let pct = c.bookings * 100 / total.max(1);
                                    view! {
                                        <li>
                                            <div class="mb-1 flex items-center justify-between text-xs">
                                                <span class="flex items-center gap-1.5 text-slate-600">
                                                    <span class="h-2 w-2 rounded-full" style=format!("background: {}", c.color)></span>
                                                    {c.name}
                                                </span>
                                                <span class="font-bold tabular-nums text-slate-900">{format!("{pct}%")}</span>
                                            </div>
                                            <ProgressBar percent=pct fill="bg-blue-500" height="h-1.5" />
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </Card>
                    </div>

                    // ---- Activity feed -----------------------------------
                    <div class="animate-fade-up" style="animation-delay: 400ms">
                        <Card title="Recent activity">
                            <ol class="flex flex-col">
                                {ACTIVITY.iter().enumerate().map(|(i, a)| {
                                    let is_last = i == ACTIVITY.len() - 1;
                                    view! {
                                        <li class="relative flex gap-3 pb-4 last:pb-0">
                                            <Show when=move || !is_last>
                                                <span class="absolute left-[15px] top-9 h-[calc(100%-1.75rem)] w-px bg-slate-200"></span>
                                            </Show>
                                            <span class=format!(
                                                "relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full ring-4 ring-white {}",
                                                a.tint
                                            )>
                                                <Icon name=a.icon class="h-3.5 w-3.5" />
                                            </span>
                                            <span class="min-w-0 pt-0.5">
                                                <span class="block text-xs font-semibold text-slate-800">{a.title}</span>
                                                <span class="mt-0.5 block text-2xs leading-relaxed text-slate-500">{a.detail}</span>
                                                <span class="mt-0.5 block text-2xs text-slate-400">{a.time}</span>
                                            </span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ol>
                        </Card>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn status_tone(status: &str) -> &'static str {
    match status {
        "Confirmed" | "Available" | "Active" | "In House" | "Completed" => "green",
        "New" | "Reserved" | "Checking Out" => "amber",
        "Arriving Today" | "Occupied" => "blue",
        "Cancelled" | "No-show" | "Maintenance" => "red",
        "VIP" => "purple",
        _ => "slate",
    }
}

/// Filled pill classes shared with the guests, reservations and rooms tables.
pub(crate) fn status_pill(status: &str) -> &'static str {
    match status {
        "Confirmed" | "Available" | "Active" | "In House" | "Completed" | "Success" => "bg-emerald-100 text-emerald-700",
        "New" | "Reserved" | "Checking Out" => "bg-amber-100 text-amber-700",
        "Arriving Today" | "Occupied" => "bg-blue-100 text-blue-700",
        "Cancelled" | "No-show" | "Maintenance" => "bg-red-100 text-red-700",
        "VIP" => "bg-purple-100 text-purple-700",
        "Inactive" => "bg-slate-200 text-slate-600",
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
