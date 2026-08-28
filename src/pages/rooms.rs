//! Room inventory, backed by the live API.
//!
//! The table, the stat tiles and the status tabs all read `/rooms/` and
//! `/rooms/statistics/`; adding a room POSTs to `/rooms/` and refreshes the
//! list. A `refresh` counter is the single source the resources depend on, so
//! any mutation (create, delete, status change) re-fetches by bumping it.

use crate::api;
use crate::components::{Icon, Modal, StatCard, Toggle};
use crate::pages::dashboard::status_dot;
use leptos::prelude::*;
use leptos_router::components::A;

const ROOM_TYPES: &[&str] = &[
    "Standard Room",
    "Deluxe Room",
    "Twin Room",
    "Family Room",
    "Executive Suite",
    "Single Room",
];
const STATUSES: &[&str] = &["Available", "Occupied", "Reserved", "Maintenance"];
const TABS: &[&str] = &["All", "Available", "Occupied", "Reserved", "Maintenance"];

#[component]
pub fn RoomsPage() -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    // Bumped after any mutation to force the resources to re-fetch.
    let refresh = RwSignal::new(0u32);

    // LocalResource: this is a CSR-only app, so resources need not be
    // serializable. Signals read before the await make the fetch reactive.
    let rooms = LocalResource::new(move || {
        let status = filter.get();
        let q = search.get();
        let _ = refresh.get();
        async move { api::list_rooms(Some(status), &q).await }
    });
    let stats = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::room_stats().await }
    });

    let stat = move |pick: fn(&api::RoomStats) -> u32| {
        stats
            .get()
            .and_then(Result::ok)
            .map(|s| pick(&s).to_string())
            .unwrap_or_else(|| "—".into())
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Rooms"</h1>
                    <p class="text-sm text-slate-500">"Manage your room inventory, pricing, and status."</p>
                </div>
                <div class="flex gap-2">
                    <A href="/rooms/bulk-upload" attr:class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50">
                        <Icon name="upload" class="h-4 w-4" />
                        "Bulk Update"
                    </A>
                    <button
                        on:click=move |_| modal_open.set(true)
                        class="flex items-center gap-1.5 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-3 py-2 text-sm font-semibold text-white"
                    >
                        <Icon name="plus" class="h-4 w-4" />
                        "Add New Room"
                    </button>
                </div>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
                <StatCard icon="bed" label="Total Rooms" value=Signal::derive(move || stat(|s| s.total_rooms)) hint="All rooms in property" />
                <StatCard icon="check-circle" label="Available" value=Signal::derive(move || stat(|s| s.available)) hint="Ready for booking" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="users" label="Occupied" value=Signal::derive(move || stat(|s| s.occupied)) hint="Currently occupied" accent="text-blue-600 bg-blue-50" />
                <StatCard icon="calendar" label="Reserved" value=Signal::derive(move || stat(|s| s.reserved)) hint="Upcoming stays" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="wrench" label="Maintenance" value=Signal::derive(move || stat(|s| s.maintenance)) hint="Under maintenance" accent="text-red-600 bg-red-50" />
            </div>

            <div class="mb-4 flex flex-wrap gap-2 text-sm">
                {TABS.iter().map(|f| {
                    let f = *f;
                    view! {
                        <button
                            class=move || format!(
                                "rounded-lg px-4 py-2 font-semibold transition-all duration-200 {}",
                                if filter.get() == f { "bg-blue-700 text-white shadow-md active:scale-[0.98]" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                            )
                            on:click=move |_| filter.set(f)
                        >
                            {f}
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="mb-4 flex flex-wrap items-center gap-3">
                <div class="relative flex-1">
                    <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                    <input
                        type="text"
                        placeholder="Search by room name, number or type"
                        class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                        prop:value=search
                        on:input:target=move |ev| search.set(ev.target().value())
                    />
                </div>
            </div>

            <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                <table class="w-full text-left text-sm">
                    <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                        <tr>
                            <th class="px-4 py-3">"Room"</th>
                            <th class="px-4 py-3">"Type"</th>
                            <th class="px-4 py-3">"Floor"</th>
                            <th class="px-4 py-3">"Capacity"</th>
                            <th class="px-4 py-3">"Price/Night"</th>
                            <th class="px-4 py-3">"Amenities"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=|| view! {
                            <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-400">"Loading rooms…"</td></tr>
                        }>
                            {move || Suspend::new(async move {
                                match rooms.await {
                                    Err(e) => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-red-600">{e.message}</td></tr>
                                    }.into_any(),
                                    Ok(page) if page.items.is_empty() => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-500">"No rooms match your search."</td></tr>
                                    }.into_any(),
                                    Ok(page) => page.items.into_iter().map(|r| {
                                        let (dot, text) = status_dot(r.status_str());
                                        let id = r.id;
                                        let href = format!("/rooms/{}", r.room_number);
                                        let href2 = href.clone();
                                        view! {
                                            <tr class="border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50">
                                                <td class="px-4 py-3 font-medium text-slate-900">{r.room_number.clone()}</td>
                                                <td class="px-4 py-3">{r.room_type.clone().unwrap_or_default()}</td>
                                                <td class="px-4 py-3">{r.floor.unwrap_or(0)}</td>
                                                <td class="px-4 py-3">{format!("{} Guests", r.guest_capacity)}</td>
                                                <td class="px-4 py-3">{format!("ETB {}", r.price_display())}</td>
                                                <td class="px-4 py-3 text-slate-500">{r.amenities_preview.join(", ")}</td>
                                                <td class="px-4 py-3">
                                                    <span class=format!("flex items-center gap-1.5 text-xs font-semibold {text}")>
                                                        <span class=format!("h-1.5 w-1.5 rounded-full {dot}")></span>
                                                        {r.status_str().to_string()}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3">
                                                    <div class="flex items-center gap-2 text-slate-400">
                                                        <A href=href attr:class="hover:text-slate-700"><Icon name="eye" class="h-4 w-4" /></A>
                                                        <A href=href2 attr:class="hover:text-slate-700"><Icon name="edit" class="h-4 w-4" /></A>
                                                        <button
                                                            title="Delete room"
                                                            class="hover:text-red-600"
                                                            on:click=move |_| {
                                                                if crate::confirm(&format!("Delete room {id}? This cannot be undone.")) {
                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        let _ = api::delete_room(id).await;
                                                                        refresh.update(|n| *n += 1);
                                                                    });
                                                                }
                                                            }
                                                        >
                                                            <Icon name="trash" class="h-4 w-4" />
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any(),
                                }
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>

            <Show when=move || modal_open.get()>
                <AddRoomModal
                    on_close=move || modal_open.set(false)
                    on_created=move || refresh.update(|n| *n += 1)
                />
            </Show>
        </div>
    }
}

#[component]
fn AddRoomModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_created: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let number = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let room_type = RwSignal::new(ROOM_TYPES[0].to_string());
    let floor = RwSignal::new(String::new());
    let capacity = RwSignal::new(String::new());
    let price = RwSignal::new(String::new());
    let discount = RwSignal::new("0".to_string());
    let bed_type = RwSignal::new(api::BED_TYPES[3].to_string());
    let bed_count = RwSignal::new("1".to_string());
    let breakfast = RwSignal::new(false);
    let status = RwSignal::new(STATUSES[0].to_string());
    let picked = RwSignal::new(Vec::<i64>::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let amenities = LocalResource::new(|| async move { api::list_amenities().await });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if number.get().trim().is_empty() {
            error.set(Some("Please enter a room number.".to_string()));
            return;
        }
        if price.get().trim().parse::<f64>().is_err() {
            error.set(Some("Please enter a nightly price.".to_string()));
            return;
        }
        error.set(None);
        busy.set(true);

        let display_name = {
            let n = name.get();
            if n.trim().is_empty() { room_type.get() } else { n }
        };
        let payload = api::NewRoom {
            room_number: number.get().trim().to_string(),
            name: display_name,
            room_type: room_type.get(),
            floor: floor.get().trim().parse().unwrap_or(1),
            guest_capacity: capacity.get().trim().parse().unwrap_or(2),
            price_per_night: price.get().trim().to_string(),
            discount_percent_per_night: discount.get().trim().parse().unwrap_or(0),
            breakfast_included: breakfast.get(),
            status: status.get(),
            amenity_ids: picked.get(),
            beds: bed_type_of(&bed_type.get(), bed_count.get().trim().parse().unwrap_or(1)),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::create_room(&payload).await {
                Ok(_) => {
                    busy.set(false);
                    on_created();
                    on_close();
                }
                Err(e) => {
                    let detail = ["room_number", "price_per_night", "guest_capacity"]
                        .into_iter()
                        .find_map(|f| e.field(f).map(|m| format!("{f}: {m}")))
                        .unwrap_or_else(|| e.message.clone());
                    error.set(Some(detail));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <Modal title="Add New Room" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Number"</label>
                        <input type="text" placeholder="e.g. 501" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=number on:input:target=move |ev| number.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| room_type.set(ev.target().value())
                        >
                            {ROOM_TYPES.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                        </select>
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Display Name (optional)"</label>
                    <input type="text" placeholder="Defaults to the room type" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                </div>

                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Floor"</label>
                        <input type="number" placeholder="1" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=floor on:input:target=move |ev| floor.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest Capacity"</label>
                        <input type="number" placeholder="2" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=capacity on:input:target=move |ev| capacity.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Price / Night"</label>
                        <input type="number" placeholder="4500" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=price on:input:target=move |ev| price.set(ev.target().value()) />
                    </div>
                </div>

                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Discount %"</label>
                        <input type="number" min="0" max="100" placeholder="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=discount on:input:target=move |ev| discount.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Bed Type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| bed_type.set(ev.target().value())
                        >
                            {api::BED_TYPES.iter().map(|t| view! {
                                <option value=*t selected=(*t == api::BED_TYPES[3])>{*t}</option>
                            }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Number of Beds"</label>
                        <input type="number" min="1" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=bed_count on:input:target=move |ev| bed_count.set(ev.target().value()) />
                    </div>
                </div>

                <div class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
                    <span class="flex items-center gap-2 text-sm text-slate-700">
                        <Icon name="coffee" class="h-4 w-4 text-slate-400" />
                        "Breakfast Included"
                    </span>
                    <button type="button" on:click=move |_| breakfast.update(|v| *v = !*v)>
                        <Toggle checked=breakfast.get() />
                    </button>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Status"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| status.set(ev.target().value())
                    >
                        {STATUSES.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                    </select>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Amenities"</label>
                    <Suspense fallback=|| view! { <p class="text-sm text-slate-400">"Loading amenities…"</p> }>
                        {move || Suspend::new(async move {
                            let list = amenities.await.unwrap_or_default();
                            if list.is_empty() {
                                return view! { <p class="text-sm text-slate-400">"No amenities in the catalogue yet."</p> }.into_any();
                            }
                            view! {
                                <div class="flex flex-wrap gap-1.5">
                                    {list.into_iter().map(|a| {
                                        let id = a.id;
                                        view! {
                                            <button type="button"
                                                on:click=move |_| picked.update(|p| match p.iter().position(|x| *x==id) {
                                                    Some(i) => { p.remove(i); }, None => p.push(id),
                                                })
                                                class=move || format!(
                                                    "rounded-lg border px-2.5 py-1 text-xs font-medium transition-colors {}",
                                                    if picked.get().contains(&id) { "border-blue-600 bg-blue-50 text-blue-700" } else { "border-slate-300 text-slate-600 hover:bg-slate-50" }
                                                )
                                            >
                                                {a.name}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </div>

                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get() class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] disabled:opacity-60">
                        {move || if busy.get() { "Creating…" } else { "Create Room" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

/// Wraps the modal's single bed selection in the array shape `POST /rooms/`
/// expects. A count of zero is treated as one bed rather than an empty room.
fn bed_type_of(bed_type: &str, count: u32) -> Vec<api::Bed> {
    vec![api::Bed {
        id: 0,
        bed_type: bed_type.to_string(),
        number_of_beds: count.max(1),
    }]
}
