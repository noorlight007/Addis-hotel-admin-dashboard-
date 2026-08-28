//! Availability calendar, backed by `/rooms/calendar-matrix/`.
//!
//! One row per room, one column per day of the viewed month. The matrix returns
//! a sparse, *undated* list of blocking entries per room — one copy per blocked
//! night, with only the reservation's own check-in/check-out dates to place it —
//! so [`api::CalendarRoom::blocked_days`] expands those spans onto the grid.
//!
//! Editing goes back through the two endpoints that actually exist:
//! `/rooms/block-dates/` to close a range and `/rooms/special-events/{id}/end/`
//! to reopen one. Nights held by a reservation are read-only here; they are
//! released from the reservation itself.

use crate::api;
use crate::components::{use_toast, Icon, Modal};
use crate::date::{
    days_in_month, format_month, format_short, is_weekend, shift_month, to_iso, today, weekday,
    Date, WEEKDAY_ABBR,
};
use leptos::prelude::*;

/// What a single room/day cell shows.
#[derive(Clone, PartialEq)]
enum Cell {
    Available,
    /// Held by a booking — guest name and reservation id.
    Reserved(String, i64),
    /// Closed by a special event; carries the event id so it can be lifted.
    Blocked(String, Option<i64>),
}

impl Cell {
    fn class(&self) -> &'static str {
        match self {
            Cell::Available => "bg-emerald-200 hover:bg-emerald-300",
            Cell::Reserved(_, _) => "bg-blue-300 hover:bg-blue-400",
            Cell::Blocked(_, _) => "bg-red-300 hover:bg-red-400",
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Cell::Available => "Available",
            Cell::Reserved(_, _) => "Reserved",
            Cell::Blocked(_, _) => "Blocked",
        }
    }
    fn detail(&self) -> String {
        match self {
            Cell::Available => "Open for sale".into(),
            Cell::Reserved(name, id) => format!("{name} · reservation #{id}"),
            Cell::Blocked(note, _) => {
                if note.is_empty() {
                    "Closed".into()
                } else {
                    note.clone()
                }
            }
        }
    }
}

/// One room's row: the room itself plus a cell per day of the month.
struct Row {
    room_id: i64,
    number: String,
    name: String,
    room_type: String,
    cells: Vec<Cell>,
}

/// Turns the API matrix into a dense day-by-day grid.
fn build_rows(matrix: &api::CalendarMatrix, year: i32, month: u32) -> Vec<Row> {
    let days = days_in_month(year, month) as usize;
    matrix
        .matrix
        .iter()
        .map(|room| {
            let mut cells = vec![Cell::Available; days];
            for (day, entry) in room.blocked_days(year, month) {
                let idx = day as usize - 1;
                if idx >= days {
                    continue;
                }
                cells[idx] = match entry.reservation_id {
                    Some(id) => Cell::Reserved(
                        entry.guest_name.clone().unwrap_or_else(|| "Guest".into()),
                        id,
                    ),
                    None => Cell::Blocked(
                        entry
                            .note
                            .clone()
                            .or(entry.block_reason.clone())
                            .unwrap_or_default(),
                        entry.event_id,
                    ),
                };
            }
            Row {
                room_id: room.room_id,
                number: room.room_number.clone(),
                name: room.room_name.clone(),
                room_type: room.room_type.clone().unwrap_or_default(),
                cells,
            }
        })
        .collect()
}

#[component]
pub fn CalendarPage() -> impl IntoView {
    let toast = use_toast();
    let t = today();
    let year = RwSignal::new(t.0);
    let month = RwSignal::new(t.1);
    let type_filter = RwSignal::new("All".to_string());
    let search = RwSignal::new(String::new());
    let refresh = RwSignal::new(0u32);
    let block_open = RwSignal::new(false);
    // The cell the operator clicked, so the popover knows what to offer.
    let picked = RwSignal::new(Option::<(i64, String, u32, Cell)>::None);

    let matrix = LocalResource::new(move || {
        let (y, m) = (year.get(), month.get());
        let ty = type_filter.get();
        let q = search.get();
        let _ = refresh.get();
        async move { api::calendar_matrix(y, m, &ty, &q).await }
    });
    let rooms = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::all_rooms().await }
    });

    let room_types = move || {
        let mut list: Vec<String> = rooms
            .get()
            .and_then(Result::ok)
            .unwrap_or_default()
            .iter()
            .filter_map(|r| r.room_type.clone())
            .collect();
        list.sort();
        list.dedup();
        list
    };

    let step = move |delta: i32| {
        let (y, m) = shift_month(year.get(), month.get(), delta);
        year.set(y);
        month.set(m);
    };

    let bump = move || refresh.update(|n| *n += 1);

    // Closes or reopens a single night on one room.
    let toggle_cell = move |room_id: i64, day: u32, cell: Cell| {
        let iso = to_iso((year.get(), month.get(), day));
        match cell {
            Cell::Reserved(_, _) => {
                toast.info(
                    "Held by a booking",
                    "Release this night from the reservation instead.",
                );
            }
            Cell::Blocked(_, Some(event_id)) => {
                wasm_bindgen_futures::spawn_local(async move {
                    match api::end_special_event(event_id).await {
                        Ok(()) => {
                            toast.success("Night reopened", format!("{iso} is open for sale."));
                            bump();
                        }
                        Err(e) => toast.error("Could not reopen the night", e.detail()),
                    }
                });
            }
            Cell::Blocked(_, None) => {
                toast.info(
                    "Cannot be lifted here",
                    "This block has no event record to end.",
                );
            }
            Cell::Available => {
                wasm_bindgen_futures::spawn_local(async move {
                    match api::block_dates(&[room_id], "Blocked", &iso, &iso, "Blocked from calendar")
                        .await
                    {
                        Ok(()) => {
                            toast.success("Night closed", format!("{iso} is no longer sellable."));
                            bump();
                        }
                        Err(e) => toast.error("Could not close the night", e.detail()),
                    }
                });
            }
        }
        picked.set(None);
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Availability calendar"</h1>
                    <p class="text-sm text-slate-500">"Click a night to close or reopen it for sale."</p>
                </div>
                <button
                    on:click=move |_| block_open.set(true)
                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                >
                    <Icon name="calendar-x" class="h-4 w-4" />
                    "Block dates"
                </button>
            </div>

            // ---- Month navigation + filters ---------------------------
            <div class="mb-4 flex flex-wrap items-center gap-3">
                <div class="flex items-center gap-1 rounded-lg border border-slate-300 bg-white">
                    <button class="px-2.5 py-2 text-slate-500 hover:text-slate-800" on:click=move |_| step(-1)>
                        <Icon name="chevron-left" class="h-4 w-4" />
                    </button>
                    <span class="min-w-[9rem] text-center text-sm font-bold text-slate-800">
                        {move || format_month(year.get(), month.get())}
                    </span>
                    <button class="px-2.5 py-2 text-slate-500 hover:text-slate-800" on:click=move |_| step(1)>
                        <Icon name="chevron-right" class="h-4 w-4" />
                    </button>
                </div>
                <button
                    class="rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
                    on:click=move |_| { year.set(t.0); month.set(t.1); }
                >
                    "Today"
                </button>

                <select
                    class="rounded-lg border border-slate-300 px-3 py-2 text-sm"
                    on:change:target=move |ev| type_filter.set(ev.target().value())
                >
                    <option value="All">"All room types"</option>
                    {move || room_types().into_iter().map(|ty| {
                        let value = ty.clone();
                        view! { <option value=value>{ty}</option> }
                    }).collect_view()}
                </select>

                <div class="relative min-w-[12rem] flex-1">
                    <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                    <input
                        type="text"
                        placeholder="Search room number or name"
                        class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                        prop:value=search
                        on:input:target=move |ev| search.set(ev.target().value())
                    />
                </div>
            </div>

            // ---- Month summary ----------------------------------------
            <Suspense fallback=|| view! {
                <div class="mb-4 h-16 animate-pulse rounded-xl bg-slate-100"></div>
            }>
                {move || Suspend::new(async move {
                    let Ok(m) = matrix.await else {
                        return ().into_any();
                    };
                    view! {
                        <div class="mb-4 grid grid-cols-2 gap-3 rounded-xl border border-slate-200 bg-white p-4 text-sm sm:grid-cols-5">
                            <Stat label="Occupancy" value=format!("{:.0}%", m.occupancy_rate) />
                            <Stat label="Rooms" value=m.total_rooms.to_string() />
                            <Stat label="Nights available" value=m.available_slots.to_string() />
                            <Stat label="Nights reserved" value=m.reserved_slots.to_string() />
                            <Stat label="Nights blocked" value=(m.blocked_slots + m.unavailable_slots).to_string() />
                        </div>
                    }.into_any()
                })}
            </Suspense>

            // ---- Legend ------------------------------------------------
            <div class="mb-3 flex flex-wrap items-center gap-4 text-2xs text-slate-500">
                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-emerald-200"></span>"Available"</span>
                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-blue-300"></span>"Reserved"</span>
                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded bg-red-300"></span>"Blocked"</span>
                <span class="flex items-center gap-1.5"><span class="h-3 w-3 rounded ring-2 ring-blue-600"></span>"Today"</span>
            </div>

            // ---- Grid --------------------------------------------------
            <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                <Suspense fallback=|| view! {
                    <p class="py-16 text-center text-sm text-slate-400">"Loading calendar…"</p>
                }>
                    {move || Suspend::new(async move {
                        let m = match matrix.await {
                            Ok(m) => m,
                            Err(e) => return view! {
                                <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                            }.into_any(),
                        };
                        let (y, mo) = (year.get(), month.get());
                        let rows = build_rows(&m, y, mo);
                        if rows.is_empty() {
                            return view! {
                                <p class="py-16 text-center text-sm text-slate-400">
                                    "No rooms match this view. Add rooms to your inventory to see the calendar."
                                </p>
                            }.into_any();
                        }
                        let days = days_in_month(y, mo);

                        view! {
                            <table class="w-full border-collapse text-xs">
                                <thead>
                                    <tr>
                                        <th class="sticky left-0 z-10 min-w-[10rem] border-b border-r border-slate-200 bg-slate-50 px-3 py-2 text-left text-2xs uppercase text-slate-500">
                                            "Room"
                                        </th>
                                        {(1..=days).map(|d| {
                                            let date: Date = (y, mo, d);
                                            let is_today = date == t;
                                            view! {
                                                <th class=format!(
                                                    "border-b border-slate-200 px-0 py-1 text-center font-semibold {}",
                                                    if is_today { "bg-blue-50 text-blue-700" }
                                                    else if is_weekend(date) { "bg-slate-50 text-slate-500" }
                                                    else { "text-slate-500" }
                                                )>
                                                    <span class="block text-[9px] font-normal text-slate-400">
                                                        {WEEKDAY_ABBR[weekday(date)].chars().next().unwrap_or(' ').to_string()}
                                                    </span>
                                                    {d}
                                                </th>
                                            }
                                        }).collect_view()}
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|row| {
                                        let room_id = row.room_id;
                                        let number = row.number.clone();
                                        view! {
                                            <tr>
                                                <td class="sticky left-0 z-10 border-b border-r border-slate-200 bg-white px-3 py-1.5">
                                                    <p class="font-bold text-slate-900">{row.number.clone()}</p>
                                                    <p class="truncate text-[10px] text-slate-500">
                                                        {if row.room_type.is_empty() { row.name.clone() } else { row.room_type.clone() }}
                                                    </p>
                                                </td>
                                                {row.cells.into_iter().enumerate().map(|(i, cell)| {
                                                    let day = i as u32 + 1;
                                                    let date: Date = (y, mo, day);
                                                    let is_today = date == t;
                                                    let title = format!(
                                                        "{} · {} · {}",
                                                        format_short(date),
                                                        cell.label(),
                                                        cell.detail(),
                                                    );
                                                    let for_click = cell.clone();
                                                    let num = number.clone();
                                                    view! {
                                                        <td class="border-b border-slate-100 p-0.5">
                                                            <button
                                                                title=title
                                                                class=format!(
                                                                    "h-7 w-full rounded transition-colors {} {}",
                                                                    cell.class(),
                                                                    if is_today { "ring-2 ring-blue-600" } else { "" },
                                                                )
                                                                on:click=move |_| picked.set(Some((
                                                                    room_id,
                                                                    num.clone(),
                                                                    day,
                                                                    for_click.clone(),
                                                                )))
                                                            ></button>
                                                        </td>
                                                    }
                                                }).collect_view()}
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any()
                    })}
                </Suspense>
            </div>

            // ---- Cell action sheet -------------------------------------
            <Show when=move || picked.get().is_some()>
                {move || {
                    let Some((room_id, number, day, cell)) = picked.get() else {
                        return ().into_any();
                    };
                    let date = (year.get(), month.get(), day);
                    let cell_for_action = cell.clone();
                    view! {
                        <Modal title="Night" on_close=move || picked.set(None)>
                            <div class="flex flex-col gap-4">
                                <div class="rounded-xl border border-slate-200 p-4">
                                    <p class="text-sm font-bold text-slate-900">
                                        {format!("Room {number} · {}", format_short(date))}
                                    </p>
                                    <p class="mt-1 text-sm text-slate-600">
                                        {format!("{} — {}", cell.label(), cell.detail())}
                                    </p>
                                </div>
                                <div class="flex justify-end gap-2">
                                    <button
                                        class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium"
                                        on:click=move |_| picked.set(None)
                                    >
                                        "Close"
                                    </button>
                                    {match &cell {
                                        Cell::Reserved(_, id) => {
                                            let _ = id;
                                            view! {
                                                <a
                                                    href="/reservations"
                                                    class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-800"
                                                >
                                                    "Open reservation"
                                                </a>
                                            }.into_any()
                                        }
                                        Cell::Blocked(_, _) => view! {
                                            <button
                                                class="rounded-lg bg-emerald-600 px-3 py-2 text-sm font-semibold text-white hover:bg-emerald-700"
                                                on:click=move |_| toggle_cell(room_id, day, cell_for_action.clone())
                                            >
                                                "Reopen for sale"
                                            </button>
                                        }.into_any(),
                                        Cell::Available => view! {
                                            <button
                                                class="rounded-lg bg-red-600 px-3 py-2 text-sm font-semibold text-white hover:bg-red-700"
                                                on:click=move |_| toggle_cell(room_id, day, cell_for_action.clone())
                                            >
                                                "Close this night"
                                            </button>
                                        }.into_any(),
                                    }}
                                </div>
                            </div>
                        </Modal>
                    }.into_any()
                }}
            </Show>

            <Show when=move || block_open.get()>
                <BlockRangeModal
                    rooms=Signal::derive(move || rooms.get().and_then(Result::ok).unwrap_or_default())
                    on_close=move || block_open.set(false)
                    on_done=move || bump()
                />
            </Show>
        </div>
    }
}

/// Blocks a date range across one room or the whole property.
#[component]
fn BlockRangeModal(
    rooms: Signal<Vec<api::RoomRow>>,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_done: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let scope = RwSignal::new("all".to_string());
    let room_id = RwSignal::new(0i64);
    let event_type = RwSignal::new(api::EVENT_TYPES[0].to_string());
    let start = RwSignal::new(String::new());
    let end = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if start.get().is_empty() || end.get().is_empty() {
            error.set(Some("Pick a start and end date.".into()));
            return;
        }
        let targets: Vec<i64> = if scope.get() == "all" {
            rooms.get().iter().map(|r| r.id).collect()
        } else if room_id.get() > 0 {
            vec![room_id.get()]
        } else {
            error.set(Some("Pick a room.".into()));
            return;
        };
        if targets.is_empty() {
            error.set(Some("There are no rooms to block.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let (ty, s, e, n) = (event_type.get(), start.get(), end.get(), note.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::block_dates(&targets, &ty, &s, &e, &n).await {
                Ok(()) => {
                    busy.set(false);
                    toast.success("Dates blocked", format!("{s} → {e} is closed for sale."));
                    on_done();
                    on_close();
                }
                Err(err) => {
                    // The API rejects a range that overlaps an existing block.
                    error.set(Some(err.detail()));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <Modal title="Block dates" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Apply to"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| scope.set(ev.target().value())
                    >
                        <option value="all">"Every room"</option>
                        <option value="one">"A single room"</option>
                    </select>
                </div>

                <Show when=move || scope.get() == "one">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| room_id.set(ev.target().value().parse().unwrap_or(0))
                        >
                            <option value="0">"Select a room…"</option>
                            {move || rooms.get().into_iter().map(|r| view! {
                                <option value=r.id.to_string()>
                                    {format!("{} · {}", r.room_number, r.type_str())}
                                </option>
                            }).collect_view()}
                        </select>
                    </div>
                </Show>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Reason"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| event_type.set(ev.target().value())
                    >
                        {api::EVENT_TYPES.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                    </select>
                    <p class="mt-1 text-2xs text-slate-500">
                        "The API offers Blocked, Unavailable and Upcoming — use the note for the specifics, such as maintenance."
                    </p>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"From"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=start on:input:target=move |ev| start.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"To"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=end on:input:target=move |ev| end.set(ev.target().value()) />
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Note"</label>
                    <input type="text" placeholder="Bathroom refit, staff training…"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=note on:input:target=move |ev| note.set(ev.target().value()) />
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Blocking…" } else { "Block dates" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn Stat(label: &'static str, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <p class="text-2xs uppercase tracking-wide text-slate-400">{label}</p>
            <p class="mt-0.5 text-lg font-bold tabular-nums text-slate-900">{value}</p>
        </div>
    }
}
