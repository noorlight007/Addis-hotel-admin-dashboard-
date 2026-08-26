use crate::components::{
    use_toast, Avatar, Badge, EmptyState, Icon, Modal, PageHeader, StatCard,
};
use crate::data::{Reservation, RESERVATIONS};
use leptos::prelude::*;
use leptos_router::components::A;

const FILTERS: &[&str] = &["All", "New", "Confirmed", "Cancelled"];
const PER_PAGE: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Newest,
    Guest,
    CheckIn,
    Status,
}

impl SortKey {
    fn label(self) -> &'static str {
        match self {
            SortKey::Newest => "Most recent",
            SortKey::Guest => "Guest name",
            SortKey::CheckIn => "Check-in date",
            SortKey::Status => "Status",
        }
    }
    const ALL: [SortKey; 4] = [SortKey::Newest, SortKey::Guest, SortKey::CheckIn, SortKey::Status];
}

fn tone(status: &str) -> &'static str {
    match status {
        "Confirmed" => "green",
        "New" => "amber",
        "Cancelled" => "red",
        _ => "slate",
    }
}

fn initials_of(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn ReservationsPage() -> impl IntoView {
    let toast = use_toast();
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    let sort = RwSignal::new(SortKey::Newest);
    let sort_open = RwSignal::new(false);
    let page = RwSignal::new(0usize);
    let selected = RwSignal::new(Option::<Reservation>::None);
    let cancelled = RwSignal::new(Vec::<&'static str>::new());

    let status_of = move |r: &Reservation| -> &'static str {
        if cancelled.get().contains(&r.booking_ref) { "Cancelled" } else { r.status }
    };

    let count = move |status: &'static str| {
        if status == "All" {
            RESERVATIONS.len()
        } else {
            RESERVATIONS.iter().filter(|r| status_of(r) == status).count()
        }
    };

    let results = Memo::new(move |_| {
        let q = search.get().to_lowercase();
        let f = filter.get();
        let mut out: Vec<&'static Reservation> = RESERVATIONS
            .iter()
            .filter(|r| f == "All" || status_of(r) == f)
            .filter(|r| {
                q.is_empty()
                    || r.guest.to_lowercase().contains(&q)
                    || r.booking_ref.to_lowercase().contains(&q)
                    || r.phone.contains(&q)
            })
            .collect();

        match sort.get() {
            SortKey::Newest => {}
            SortKey::Guest => out.sort_by_key(|r| r.guest),
            SortKey::CheckIn => out.sort_by_key(|r| r.check_in),
            SortKey::Status => out.sort_by_key(|r| status_of(r)),
        }
        out
    });

    let page_count = move || results.get().len().div_ceil(PER_PAGE).max(1);
    let visible = move || {
        results.get().into_iter().skip(page.get() * PER_PAGE).take(PER_PAGE).collect::<Vec<_>>()
    };

    // Keep the page index valid as filters narrow the result set.
    Effect::new(move |_| {
        let max = page_count().saturating_sub(1);
        if page.get() > max {
            page.set(max);
        }
    });

    let confirm = move |r: &'static Reservation| {
        toast.success("Reservation confirmed", format!("{} has been confirmed with the guest.", r.booking_ref));
        selected.set(None);
    };
    let cancel = move |r: &'static Reservation| {
        cancelled.update(|list| list.push(r.booking_ref));
        toast.warning("Reservation cancelled", format!("{} was cancelled and the room released.", r.booking_ref));
        selected.set(None);
    };

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Reservations"
                subtitle="Every booking made through the portal, phone or front desk."
            >
                <button
                    on:click=move |_| toast.info("Export queued", "A CSV of the current view will download shortly.")
                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                >
                    <Icon name="download" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Export"</span>
                </button>
                <A
                    href="/calendar"
                    attr:class="sheen flex items-center gap-1.5 rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 active:scale-95"
                >
                    <Icon name="calendar" class="h-4 w-4" />
                    "Calendar view"
                </A>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="calendar-check"
                        label="Total"
                        value=RESERVATIONS.len().to_string()
                        hint="All reservations"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="bell"
                        label="Awaiting action"
                        value=Signal::derive(move || count("New").to_string())
                        hint="Newly received"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="check-circle"
                        label="Confirmed"
                        value=Signal::derive(move || count("Confirmed").to_string())
                        hint="Ready for arrival"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="x-circle"
                        label="Cancelled"
                        value=Signal::derive(move || count("Cancelled").to_string())
                        hint="Rooms released"
                        accent="text-red-600 bg-red-50"
                    />
                </div>
            </div>

            // ---- Filters -------------------------------------------------
            <div class="mb-4 flex flex-wrap items-center gap-2">
                {FILTERS.iter().map(|f| {
                    let f = *f;
                    view! {
                        <button
                            class=move || format!(
                                "rounded-lg px-3.5 py-2 text-sm font-semibold transition-all duration-200 active:scale-95 {}",
                                if filter.get() == f {
                                    "bg-blue-700 text-white shadow-md shadow-blue-700/25"
                                } else {
                                    "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
                                }
                            )
                            on:click=move |_| { filter.set(f); page.set(0); }
                        >
                            {move || format!("{f} ({})", count(f))}
                        </button>
                    }
                }).collect_view()}

                <div class="relative ml-auto">
                    <button
                        on:click=move |_| sort_open.update(|v| *v = !*v)
                        class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                    >
                        <Icon name="sort" class="h-4 w-4" />
                        <span class="hidden sm:inline">{move || sort.get().label()}</span>
                        <Icon name="chevron-down" class="h-3 w-3" />
                    </button>
                    <Show when=move || sort_open.get()>
                        <div class="absolute right-0 top-full z-30 mt-1 w-48 animate-fade-down overflow-hidden rounded-xl border border-slate-200 bg-white p-1 shadow-xl">
                            {SortKey::ALL.into_iter().map(|k| view! {
                                <button
                                    on:click=move |_| { sort.set(k); sort_open.set(false); }
                                    class=move || format!(
                                        "flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors hover:bg-slate-50 {}",
                                        if sort.get() == k { "font-bold text-blue-700" } else { "text-slate-600" }
                                    )
                                >
                                    {k.label()}
                                    <Show when=move || (sort.get() == k)>
                                        <Icon name="check" class="h-3.5 w-3.5" />
                                    </Show>
                                </button>
                            }).collect_view()}
                        </div>
                    </Show>
                </div>
            </div>

            <div class="relative mb-4 max-w-md">
                <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="text"
                    placeholder="Search by name, booking reference or phone"
                    class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    prop:value=search
                    on:input:target=move |ev| { search.set(ev.target().value()); page.set(0); }
                />
            </div>

            // ---- Table ---------------------------------------------------
            <div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-card">
                <div class="overflow-x-auto">
                    <table class="w-full min-w-[48rem] text-left text-sm">
                        <thead class="border-b border-slate-200 bg-slate-50/60 text-2xs uppercase tracking-wide text-slate-500">
                            <tr>
                                <th class="px-4 py-3">"Guest"</th>
                                <th class="px-4 py-3">"Booking ref"</th>
                                <th class="px-4 py-3">"Check-in"</th>
                                <th class="px-4 py-3">"Check-out"</th>
                                <th class="px-4 py-3 text-center">"Guests"</th>
                                <th class="px-4 py-3">"Status"</th>
                                <th class="px-4 py-3">"Booked"</th>
                                <th class="px-4 py-3"></th>
                            </tr>
                        </thead>
                        <tbody class="stagger">
                            {move || visible().into_iter().map(|r| {
                                let status = status_of(r);
                                view! {
                                    <tr
                                        class="group cursor-pointer border-b border-slate-100 transition-colors last:border-0 hover:bg-blue-50/40"
                                        on:click=move |_| selected.set(Some(*r))
                                    >
                                        <td class="px-4 py-3">
                                            <div class="flex items-center gap-2.5">
                                                <Avatar initials=initials_of(r.guest) size="h-8 w-8 text-2xs" />
                                                <div class="min-w-0">
                                                    <p class="truncate font-semibold text-slate-900">{r.guest}</p>
                                                    <p class="text-2xs text-slate-400">{r.phone}</p>
                                                </div>
                                            </div>
                                        </td>
                                        <td class="px-4 py-3 font-medium text-blue-700">{r.booking_ref}</td>
                                        <td class="px-4 py-3 text-slate-600">{r.check_in}</td>
                                        <td class="px-4 py-3 text-slate-600">{r.check_out}</td>
                                        <td class="px-4 py-3 text-center tabular-nums text-slate-600">{r.guests}</td>
                                        <td class="px-4 py-3">
                                            <Badge label=status tone=tone(status) dot=(status == "New") />
                                        </td>
                                        <td class="px-4 py-3 text-slate-500">{r.time}</td>
                                        <td class="px-4 py-3 text-right">
                                            <span class="inline-flex items-center gap-1 text-2xs font-semibold text-slate-400 transition-colors group-hover:text-blue-700">
                                                "Open"
                                                <Icon name="chevron-right" class="h-3.5 w-3.5" />
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>

                <Show when=move || visible().is_empty()>
                    <EmptyState
                        icon="calendar-check"
                        title="No reservations match"
                        body="Try another status filter, or clear the search box."
                    />
                </Show>
            </div>

            <div class="mt-3 flex flex-wrap items-center justify-between gap-3">
                <p class="text-sm text-slate-500">
                    {move || format!("Showing {} of {} reservations", visible().len(), RESERVATIONS.len())}
                </p>

                <Show when=move || (page_count() > 1)>
                    <div class="flex items-center gap-1.5 text-sm">
                        <button
                            aria-label="Previous page"
                            class="flex h-8 w-8 items-center justify-center rounded-lg border border-slate-300 text-slate-600 transition-colors hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40"
                            disabled=move || (page.get() == 0)
                            on:click=move |_| page.update(|p| *p = p.saturating_sub(1))
                        >
                            <Icon name="chevron-left" class="h-4 w-4" />
                        </button>
                        {move || (0..page_count()).map(|i| view! {
                            <button
                                on:click=move |_| page.set(i)
                                class=move || format!(
                                    "flex h-8 min-w-8 items-center justify-center rounded-lg px-2.5 font-semibold transition-all duration-200 {}",
                                    if page.get() == i {
                                        "bg-blue-700 text-white shadow-md shadow-blue-700/25"
                                    } else {
                                        "border border-slate-300 text-slate-600 hover:bg-slate-50"
                                    }
                                )
                            >
                                {i + 1}
                            </button>
                        }).collect_view()}
                        <button
                            aria-label="Next page"
                            class="flex h-8 w-8 items-center justify-center rounded-lg border border-slate-300 text-slate-600 transition-colors hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40"
                            disabled=move || (page.get() + 1 >= page_count())
                            on:click=move |_| page.update(|p| *p += 1)
                        >
                            <Icon name="chevron-right" class="h-4 w-4" />
                        </button>
                    </div>
                </Show>
            </div>
        </div>

        // ---- Detail drawer --------------------------------------------
        <Show when=move || selected.get().is_some()>
            {move || {
                let r = selected.get().unwrap_or(RESERVATIONS[0]);
                // The table holds `&'static` rows, so this lookup always resolves.
                let stat = RESERVATIONS
                    .iter()
                    .find(|x| x.booking_ref == r.booking_ref)
                    .unwrap_or(&RESERVATIONS[0]);
                let status = status_of(&r);
                view! {
                    <Modal
                        title=format!("Reservation {}", r.booking_ref)
                        width="max-w-xl"
                        on_close=move || selected.set(None)
                    >
                        <div class="flex items-center gap-3.5 rounded-xl border border-slate-200 p-4">
                            <Avatar initials=initials_of(r.guest) size="h-12 w-12 text-sm" />
                            <div class="min-w-0 flex-1">
                                <p class="text-base font-bold text-slate-900">{r.guest}</p>
                                <p class="text-sm text-slate-500">{r.phone}</p>
                            </div>
                            <Badge label=status tone=tone(status) dot=true />
                        </div>

                        <dl class="mt-4 grid grid-cols-2 gap-4 rounded-xl bg-slate-50 p-4 text-sm">
                            {[
                                ("calendar", "Check-in", r.check_in),
                                ("calendar-x", "Check-out", r.check_out),
                                ("clock", "Booked at", r.time),
                            ].into_iter().map(|(icon, label, value)| view! {
                                <div>
                                    <dt class="flex items-center gap-1.5 text-2xs uppercase tracking-wide text-slate-400">
                                        <Icon name=icon class="h-3 w-3" />
                                        {label}
                                    </dt>
                                    <dd class="mt-0.5 font-bold text-slate-900">{value}</dd>
                                </div>
                            }).collect_view()}
                            <div>
                                <dt class="flex items-center gap-1.5 text-2xs uppercase tracking-wide text-slate-400">
                                    <Icon name="users" class="h-3 w-3" />
                                    "Guests"
                                </dt>
                                <dd class="mt-0.5 font-bold text-slate-900">{r.guests}</dd>
                            </div>
                        </dl>

                        <div class="mt-4 flex flex-col gap-2 text-sm">
                            <a
                                href=format!("tel:{}", r.phone)
                                class="flex items-center gap-2.5 rounded-lg border border-slate-200 px-3 py-2.5 transition-colors hover:border-blue-300 hover:bg-blue-50/40"
                            >
                                <Icon name="phone" class="h-4 w-4 text-blue-700" />
                                <span class="font-semibold text-slate-800">"Call the guest"</span>
                                <span class="ml-auto text-slate-500">{r.phone}</span>
                            </a>
                            <A
                                href=format!("/guests/{}", r.booking_ref)
                                attr:class="flex items-center gap-2.5 rounded-lg border border-slate-200 px-3 py-2.5 transition-colors hover:border-blue-300 hover:bg-blue-50/40"
                            >
                                <Icon name="id-card" class="h-4 w-4 text-blue-700" />
                                <span class="font-semibold text-slate-800">"Open the guest record"</span>
                                <Icon name="chevron-right" class="ml-auto h-4 w-4 text-slate-400" />
                            </A>
                        </div>

                        <div class="mt-5 flex gap-2.5">
                            <button
                                on:click=move |_| cancel(stat)
                                class="flex-1 rounded-lg border border-red-200 py-2.5 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50"
                            >
                                "Cancel reservation"
                            </button>
                            <button
                                on:click=move |_| confirm(stat)
                                class="flex-1 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-colors hover:bg-blue-800"
                            >
                                "Confirm with guest"
                            </button>
                        </div>
                    </Modal>
                }
            }}
        </Show>
    }
}
