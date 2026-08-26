//! Availability calendar.
//!
//! One row per room type, one column per day of the viewed month. Days are real
//! dates derived from the browser clock, so month navigation, weekday headers
//! and the "today" marker all behave like a calendar rather than a picture of
//! one. With no backend behind it, a room/date pair with no explicit edit falls
//! back to a deterministic status derived from the date (see [`seeded_status`])
//! so every month looks populated and looks the same on every reload.

use crate::components::{use_toast, Icon, Modal};
use crate::date::{
    days_from_epoch, days_in_month, format_month, format_short, is_weekend, shift_month, today,
    weekday, Date, WEEKDAY_ABBR,
};
use leptos::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Status {
    Available,
    Reserved,
    Blocked,
    Unavailable,
}

impl Status {
    const ALL: [Status; 4] = [
        Status::Available,
        Status::Reserved,
        Status::Blocked,
        Status::Unavailable,
    ];

    fn label(self) -> &'static str {
        match self {
            Status::Available => "Available",
            Status::Reserved => "Reserved",
            Status::Blocked => "Blocked",
            Status::Unavailable => "Unavailable",
        }
    }

    /// Cell fill.
    fn cell_class(self) -> &'static str {
        match self {
            Status::Available => "bg-green-200 hover:bg-green-300",
            Status::Reserved => "bg-blue-300 hover:bg-blue-400",
            Status::Blocked => "bg-red-300 hover:bg-red-400",
            Status::Unavailable => "bg-slate-200 hover:bg-slate-300",
        }
    }

    /// Swatch fill for the legend and the selection toolbar.
    fn swatch_class(self) -> &'static str {
        match self {
            Status::Available => "bg-green-200",
            Status::Reserved => "bg-blue-300",
            Status::Blocked => "bg-red-300",
            Status::Unavailable => "bg-slate-200",
        }
    }

    fn next(self) -> Status {
        match self {
            Status::Available => Status::Reserved,
            Status::Reserved => Status::Blocked,
            Status::Blocked => Status::Unavailable,
            Status::Unavailable => Status::Available,
        }
    }
}

struct RoomRow {
    room_type: &'static str,
    rooms: u32,
}

const ROWS: &[RoomRow] = &[
    RoomRow { room_type: "Standard Room", rooms: 6 },
    RoomRow { room_type: "Deluxe Room", rooms: 4 },
    RoomRow { room_type: "Twin Room", rooms: 3 },
    RoomRow { room_type: "Executive Suite", rooms: 2 },
    RoomRow { room_type: "Family Room", rooms: 2 },
    RoomRow { room_type: "Single Room", rooms: 3 },
];

/// Stand-in for the availability a real API would return.
///
/// Deterministic in `(row, date)` so the grid is stable across reloads and
/// consistent when you page back to a month you already looked at.
fn seeded_status(row: usize, date: Date) -> Status {
    let seed = (days_from_epoch(date) * 31 + row as i64 * 7).rem_euclid(16);
    match seed {
        0..=8 => Status::Available,
        9..=12 => Status::Reserved,
        13..=14 => Status::Blocked,
        _ => Status::Unavailable,
    }
}

/// A contiguous run of days in one row, inclusive at both ends.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Selection {
    row: usize,
    from: u32,
    to: u32,
}

impl Selection {
    fn contains(&self, row: usize, day: u32) -> bool {
        self.row == row && day >= self.from.min(self.to) && day <= self.from.max(self.to)
    }

    fn days(&self) -> u32 {
        self.from.abs_diff(self.to) + 1
    }
}

/// Parses the `YYYY-MM-DD` an `<input type="date">` produces.
fn parse_input_date(raw: &str) -> Option<Date> {
    let mut parts = raw.split('-');
    let y = parts.next()?.parse::<i32>().ok()?;
    let m = parts.next()?.parse::<u32>().ok()?;
    let d = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
        return None;
    }
    Some((y, m, d))
}

#[component]
pub fn CalendarPage() -> impl IntoView {
    let toast = use_toast();
    let now = today();

    let (view_year, view_month) = (RwSignal::new(now.0), RwSignal::new(now.1));
    // Explicit edits only; anything absent falls back to `seeded_status`.
    let edits = RwSignal::new(HashMap::<(usize, Date), Status>::new());
    let selection = RwSignal::new(None::<Selection>);
    let room_filter = RwSignal::new("All Room Types");
    let filter_open = RwSignal::new(false);
    let block_modal = RwSignal::new(false);
    let rate_modal = RwSignal::new(false);

    let days = move || {
        let (y, m) = (view_year.get(), view_month.get());
        (1..=days_in_month(y, m)).map(move |d| (y, m, d)).collect::<Vec<Date>>()
    };

    let status_of = move |row: usize, date: Date| {
        edits
            .get()
            .get(&(row, date))
            .copied()
            .unwrap_or_else(|| seeded_status(row, date))
    };

    let set_status = move |row: usize, date: Date, status: Status| {
        edits.update(|map| {
            map.insert((row, date), status);
        });
    };

    let visible_rows = move || {
        let filter = room_filter.get();
        ROWS.iter()
            .enumerate()
            .filter(move |(_, r)| filter == "All Room Types" || r.room_type == filter)
            .collect::<Vec<_>>()
    };

    // Paging months invalidates any in-progress selection.
    let go_month = move |delta: i32| {
        let (y, m) = shift_month(view_year.get(), view_month.get(), delta);
        view_year.set(y);
        view_month.set(m);
        selection.set(None);
    };

    let apply_to_selection = move |status: Status| {
        let Some(sel) = selection.get() else { return };
        let (y, m) = (view_year.get(), view_month.get());
        for day in sel.from.min(sel.to)..=sel.from.max(sel.to) {
            set_status(sel.row, (y, m, day), status);
        }
        toast.success(
            "Availability updated",
            format!(
                "{} · {} night{} marked {}.",
                ROWS[sel.row].room_type,
                sel.days(),
                if sel.days() == 1 { "" } else { "s" },
                status.label().to_lowercase()
            ),
        );
        selection.set(None);
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <h1 class="text-xl font-bold text-slate-900">"Availability Calendar"</h1>
                <div class="flex items-center gap-2 text-sm">
                    <button
                        aria-label="Previous month"
                        on:click=move |_| go_month(-1)
                        class="rounded-lg border border-slate-300 bg-white p-1.5 transition-colors hover:bg-slate-50"
                    >
                        <Icon name="chevron-left" class="h-4 w-4" />
                    </button>
                    <span class="w-36 text-center font-medium">
                        {move || format_month(view_year.get(), view_month.get())}
                    </span>
                    <button
                        aria-label="Next month"
                        on:click=move |_| go_month(1)
                        class="rounded-lg border border-slate-300 bg-white p-1.5 transition-colors hover:bg-slate-50"
                    >
                        <Icon name="chevron-right" class="h-4 w-4" />
                    </button>
                    <button
                        on:click=move |_| {
                            view_year.set(now.0);
                            view_month.set(now.1);
                            selection.set(None);
                        }
                        class="rounded-lg bg-blue-50 px-3 py-1.5 font-medium text-blue-700 transition-colors hover:bg-blue-100"
                    >
                        "Today"
                    </button>
                </div>
            </div>

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div class="relative">
                    <button
                        on:click=move |_| filter_open.update(|v| *v = !*v)
                        class="flex items-center gap-2 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm transition-colors hover:bg-slate-50"
                    >
                        {move || room_filter.get()}
                        <Icon name="chevron-down" class="h-4 w-4 text-slate-400" />
                    </button>
                    <Show when=move || filter_open.get()>
                        <div class="absolute left-0 top-full z-10 mt-1 w-48 animate-scale-in rounded-lg border border-slate-200 bg-white p-1.5 text-sm shadow-lg">
                            {std::iter::once("All Room Types")
                                .chain(ROWS.iter().map(|r| r.room_type))
                                .map(|t| view! {
                                    <button
                                        on:click=move |_| {
                                            room_filter.set(t);
                                            filter_open.set(false);
                                            selection.set(None);
                                        }
                                        class="flex w-full items-center rounded-md px-3 py-2 text-left text-slate-700 hover:bg-slate-50"
                                    >
                                        {t}
                                    </button>
                                })
                                .collect_view()}
                        </div>
                    </Show>
                </div>
                <div class="flex items-center gap-4 text-xs text-slate-600">
                    {Status::ALL.iter().map(|s| view! {
                        <span class="flex items-center gap-1">
                            <span class=format!("h-3 w-3 rounded {}", s.swatch_class())></span>
                            {s.label()}
                        </span>
                    }).collect_view()}
                </div>
            </div>

            // ---- Selection toolbar ----------------------------------------
            // Floats over the page rather than sitting in the flow: inserting it
            // above the grid would shift every cell down under the cursor,
            // so the shift-click that closes a range would miss its row.
            <Show when=move || selection.get().is_some()>
                {move || {
                    let sel = selection.get().expect("guarded by Show");
                    let (y, m) = (view_year.get(), view_month.get());
                    let (lo, hi) = (sel.from.min(sel.to), sel.from.max(sel.to));
                    view! {
                        <div class="fixed bottom-5 left-1/2 z-30 flex w-[min(46rem,calc(100vw-2rem))] -translate-x-1/2 flex-wrap items-center gap-3 rounded-xl border border-blue-200 bg-white/95 px-4 py-3 text-sm shadow-xl shadow-slate-900/10 backdrop-blur animate-fade-in">
                            <span class="font-medium text-blue-900">
                                {format!(
                                    "{} · {} – {} ({} night{})",
                                    ROWS[sel.row].room_type,
                                    format_short((y, m, lo)),
                                    format_short((y, m, hi)),
                                    sel.days(),
                                    if sel.days() == 1 { "" } else { "s" },
                                )}
                            </span>
                            <span class="ml-auto flex flex-wrap items-center gap-2">
                                {Status::ALL.iter().map(|s| {
                                    let s = *s;
                                    view! {
                                        <button
                                            on:click=move |_| apply_to_selection(s)
                                            class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 transition-colors hover:bg-slate-50"
                                        >
                                            <span class=format!("h-2.5 w-2.5 rounded-sm {}", s.swatch_class())></span>
                                            {s.label()}
                                        </button>
                                    }
                                }).collect_view()}
                                <button
                                    on:click=move |_| selection.set(None)
                                    class="rounded-lg px-2.5 py-1.5 text-xs font-medium text-slate-500 transition-colors hover:text-slate-800"
                                >
                                    "Clear"
                                </button>
                            </span>
                        </div>
                    }
                }}
            </Show>

            // ---- Grid ------------------------------------------------------
            // The scroll container carries no left padding: the sticky room-name
            // column has to sit flush against its edge, or day cells slide
            // through the gap beside it.
            <div class="rounded-xl border border-slate-200 bg-white">
                <div class="overflow-x-auto py-4 pr-4">
                <div
                    class="grid min-w-max gap-1 text-xs"
                    style=move || format!(
                        "grid-template-columns: 11rem repeat({}, minmax(2.25rem, 1fr))",
                        days().len()
                    )
                >
                    <div class="sticky left-0 z-10 bg-white pl-4"></div>
                    {move || days().into_iter().map(|d| {
                        let is_today = d == now;
                        view! {
                            <div class=format!(
                                "pb-2 text-center font-medium {}",
                                if is_today {
                                    "rounded-t bg-blue-50 text-blue-700"
                                } else if is_weekend(d) {
                                    "text-slate-400"
                                } else {
                                    "text-slate-500"
                                }
                            )>
                                <span class="block text-[10px] uppercase tracking-wide">
                                    {WEEKDAY_ABBR[weekday(d)]}
                                </span>
                                <span class="block">{d.2}</span>
                            </div>
                        }
                    }).collect_view()}

                    {move || visible_rows().into_iter().map(|(ri, row)| {
                        view! {
                            <>
                                <div class="sticky left-0 z-10 flex flex-col justify-center bg-white py-2 pl-4 pr-2">
                                    <span class="font-medium text-slate-800">{row.room_type}</span>
                                    <span class="text-slate-400">
                                        {format!("{} room{}", row.rooms, if row.rooms == 1 { "" } else { "s" })}
                                    </span>
                                </div>
                                {move || days().into_iter().map(|date| {
                                    let day = date.2;
                                    let status = status_of(ri, date);
                                    let selected = selection
                                        .get()
                                        .is_some_and(|s| s.contains(ri, day));
                                    view! {
                                        <button
                                            title=format!(
                                                "{} · {} — {} (click to cycle, shift-click to select a range)",
                                                row.room_type,
                                                format_short(date),
                                                status.label(),
                                            )
                                            aria-label=format!(
                                                "{} on {}: {}",
                                                row.room_type, format_short(date), status.label()
                                            )
                                            on:click=move |ev| {
                                                if ev.shift_key() {
                                                    selection.update(|current| {
                                                        *current = Some(match *current {
                                                            // Extend an existing run in the same row.
                                                            Some(s) if s.row == ri => {
                                                                Selection { row: ri, from: s.from, to: day }
                                                            }
                                                            _ => Selection { row: ri, from: day, to: day },
                                                        });
                                                    });
                                                } else {
                                                    selection.set(None);
                                                    set_status(ri, date, status.next());
                                                }
                                            }
                                            class=format!(
                                                "h-10 rounded transition-colors {} {}",
                                                status.cell_class(),
                                                if selected {
                                                    "ring-2 ring-blue-600 ring-offset-1"
                                                } else {
                                                    ""
                                                },
                                            )
                                        ></button>
                                    }
                                }).collect_view()}
                            </>
                        }
                    }).collect_view()}
                </div>
                </div>
            </div>

            <div class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-3 text-base font-semibold text-slate-900">"Quick Actions"</h2>
                <div class="grid gap-3 sm:grid-cols-2">
                    <button on:click=move |_| block_modal.set(true) class="flex items-center justify-between rounded-lg border border-slate-200 p-3 text-left transition-colors hover:bg-slate-50">
                        <span class="flex items-center gap-2">
                            <Icon name="lock" class="h-4 w-4 text-slate-500" />
                            <span><span class="block text-sm font-medium text-slate-800">"Block Dates"</span><span class="text-xs text-slate-400">"Block rooms for maintenance or private use"</span></span>
                        </span>
                        <Icon name="chevron-right" class="h-4 w-4 text-slate-400" />
                    </button>
                    <button on:click=move |_| rate_modal.set(true) class="flex items-center justify-between rounded-lg border border-slate-200 p-3 text-left transition-colors hover:bg-slate-50">
                        <span class="flex items-center gap-2">
                            <Icon name="tag" class="h-4 w-4 text-slate-500" />
                            <span><span class="block text-sm font-medium text-slate-800">"Add Special Rate"</span><span class="text-xs text-slate-400">"Add special rate for selected dates"</span></span>
                        </span>
                        <Icon name="chevron-right" class="h-4 w-4 text-slate-400" />
                    </button>
                </div>
            </div>

            <div class="mt-4 flex items-start gap-2 rounded-lg bg-blue-50 p-3 text-sm text-blue-700">
                <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                <span>"Click a date cell to cycle its status. Shift-click a second cell in the same row to select a range and set them all at once."</span>
            </div>

            <Show when=move || block_modal.get()>
                <BlockDatesModal
                    on_close=move || block_modal.set(false)
                    on_apply=move |room_type: &'static str, from: Date, to: Date| {
                        let row = ROWS.iter().position(|r| r.room_type == room_type);
                        let Some(row) = row else { return };
                        let mut nights = 0;
                        let mut cursor = from;
                        while days_from_epoch(cursor) <= days_from_epoch(to) {
                            set_status(row, cursor, Status::Blocked);
                            nights += 1;
                            cursor = next_day(cursor);
                        }
                        // Jump the grid to the month the block starts in so the
                        // change is visible rather than silently off-screen.
                        view_year.set(from.0);
                        view_month.set(from.1);
                        selection.set(None);
                        toast.success(
                            "Dates blocked",
                            format!(
                                "{room_type} · {nights} night{} from {}.",
                                if nights == 1 { "" } else { "s" },
                                format_short(from),
                            ),
                        );
                    }
                />
            </Show>

            <Show when=move || rate_modal.get()>
                <SpecialRateModal on_close=move || rate_modal.set(false) />
            </Show>
        </div>
    }
}

fn next_day((y, m, d): Date) -> Date {
    if d < days_in_month(y, m) {
        (y, m, d + 1)
    } else {
        let (y, m) = shift_month(y, m, 1);
        (y, m, 1)
    }
}

#[component]
fn BlockDatesModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_apply: impl Fn(&'static str, Date, Date) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let room_type = RwSignal::new(ROWS[0].room_type);
    let start = RwSignal::new(String::new());
    let end = RwSignal::new(String::new());
    let reason = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());

    let apply = move |_| {
        let (Some(from), Some(to)) =
            (parse_input_date(&start.get()), parse_input_date(&end.get()))
        else {
            error.set("Pick both a start and an end date.".to_string());
            return;
        };
        if days_from_epoch(to) < days_from_epoch(from) {
            error.set("The end date must be on or after the start date.".to_string());
            return;
        }
        error.set(String::new());
        on_apply(room_type.get(), from, to);
        if !reason.get().is_empty() {
            toast.info("Reason recorded", reason.get());
        }
        on_close();
    };

    view! {
        <Modal title="Block Dates" on_close=on_close>
            <div class="flex flex-col gap-4">
                <Show when=move || !error.get().is_empty()>
                    <p class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="alert" class="h-4 w-4 shrink-0" />
                        {move || error.get()}
                    </p>
                </Show>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Room Type"</label>
                    <select
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| {
                            let picked = ev.target().value();
                            if let Some(r) = ROWS.iter().find(|r| r.room_type == picked) {
                                room_type.set(r.room_type);
                            }
                        }
                    >
                        {ROWS.iter().map(|r| view! { <option value=r.room_type>{r.room_type}</option> }).collect_view()}
                    </select>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Start Date"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=start on:input:target=move |ev| start.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"End Date"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=end on:input:target=move |ev| end.set(ev.target().value()) />
                    </div>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Reason (optional)"</label>
                    <input type="text" placeholder="e.g. Deep cleaning" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:input:target=move |ev| reason.set(ev.target().value()) />
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="button" on:click=apply class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Block Dates"</button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn SpecialRateModal(on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let toast = use_toast();
    let room_type = RwSignal::new(ROWS[0].room_type);
    let start = RwSignal::new(String::new());
    let end = RwSignal::new(String::new());
    let rate = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());

    let save = move |_| {
        let (Some(from), Some(to)) =
            (parse_input_date(&start.get()), parse_input_date(&end.get()))
        else {
            error.set("Pick both a start and an end date.".to_string());
            return;
        };
        if days_from_epoch(to) < days_from_epoch(from) {
            error.set("The end date must be on or after the start date.".to_string());
            return;
        }
        let Ok(amount) = rate.get().trim().parse::<u32>() else {
            error.set("Enter the nightly rate as a whole number of ETB.".to_string());
            return;
        };
        if amount == 0 {
            error.set("The nightly rate must be greater than zero.".to_string());
            return;
        }
        error.set(String::new());
        toast.success(
            "Special rate saved",
            format!(
                "{} · ETB {amount} / night from {} to {}.",
                room_type.get(),
                format_short(from),
                format_short(to),
            ),
        );
        on_close();
    };

    view! {
        <Modal title="Add Special Rate" on_close=on_close>
            <div class="flex flex-col gap-4">
                <Show when=move || !error.get().is_empty()>
                    <p class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="alert" class="h-4 w-4 shrink-0" />
                        {move || error.get()}
                    </p>
                </Show>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Room Type"</label>
                    <select
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| {
                            let picked = ev.target().value();
                            if let Some(r) = ROWS.iter().find(|r| r.room_type == picked) {
                                room_type.set(r.room_type);
                            }
                        }
                    >
                        {ROWS.iter().map(|r| view! { <option value=r.room_type>{r.room_type}</option> }).collect_view()}
                    </select>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Start Date"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=start on:input:target=move |ev| start.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"End Date"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=end on:input:target=move |ev| end.set(ev.target().value()) />
                    </div>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Special Rate (ETB / night)"</label>
                    <input type="number" min="1" placeholder="e.g. 3800" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=rate on:input:target=move |ev| rate.set(ev.target().value()) />
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="button" on:click=save class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Rate"</button>
                </div>
            </div>
        </Modal>
    }
}
