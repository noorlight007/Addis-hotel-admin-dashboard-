use crate::components::{Icon, Modal, StatCard, Toggle};
use crate::data::{Room, ROOMS};
use crate::pages::dashboard::status_dot;
use leptos::prelude::*;
use leptos_router::components::A;

const ROOM_TYPES: &[&str] = &["Standard Room", "Deluxe Room", "Twin Room", "Family Room", "Executive Suite", "Single Room"];
const STATUSES: &[&str] = &["Available", "Occupied", "Reserved", "Maintenance"];

#[component]
pub fn RoomsPage() -> impl IntoView {
    let rooms = RwSignal::new(ROOMS.to_vec());
    let modal_open = RwSignal::new(false);
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());

    let available = move || rooms.get().iter().filter(|r| r.status == "Available").count();
    let occupied = move || rooms.get().iter().filter(|r| r.status == "Occupied").count();
    let reserved = move || rooms.get().iter().filter(|r| r.status == "Reserved").count();
    let maintenance = move || rooms.get().iter().filter(|r| r.status == "Maintenance").count();

    let visible_rooms = move || {
        let q = search.get().to_lowercase();
        rooms
            .get()
            .into_iter()
            .filter(|r| filter.get() == "All" || r.status == filter.get())
            .filter(|r| {
                q.is_empty()
                    || r.number.to_lowercase().contains(&q)
                    || r.room_type.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
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
                <StatCard icon="bed" label="Total Rooms" value=Signal::derive(move || rooms.get().len().to_string()) hint="All rooms in property" />
                <StatCard icon="check-circle" label="Available" value=Signal::derive(move || available().to_string()) hint="Ready for booking" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="users" label="Occupied" value=Signal::derive(move || occupied().to_string()) hint="Currently occupied" accent="text-blue-600 bg-blue-50" />
                <StatCard icon="calendar" label="Reserved" value=Signal::derive(move || reserved().to_string()) hint="Upcoming stays" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="wrench" label="Maintenance" value=Signal::derive(move || maintenance().to_string()) hint="Under maintenance" accent="text-red-600 bg-red-50" />
            </div>

            <div class="mb-4 flex flex-wrap gap-2 text-sm">
                {["All", "Available", "Occupied", "Reserved", "Maintenance"].iter().map(|f| {
                    let f = *f;
                    let n = move || if f == "All" { rooms.get().len() } else { rooms.get().iter().filter(|r| r.status == f).count() };
                    view! {
                        <button
                            class=move || format!(
                                "rounded-lg px-4 py-2 font-semibold transition-all duration-200 {}",
                                if filter.get() == f { "bg-blue-700 text-white shadow-md active:scale-[0.98]" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                            )
                            on:click=move |_| filter.set(f)
                        >
                            {move || format!("{f} ({})", n())}
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
                <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm transition-colors hover:bg-slate-50">
                    <Icon name="sliders" class="h-4 w-4" />
                    "Filters"
                </button>
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
                        {move || visible_rooms().into_iter().map(|r| {
                            let (dot, text) = status_dot(r.status);
                            view! {
                                <tr class="border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50">
                                    <td class="px-4 py-3 font-medium text-slate-900">{r.number}</td>
                                    <td class="px-4 py-3">{r.room_type}</td>
                                    <td class="px-4 py-3">{r.floor}</td>
                                    <td class="px-4 py-3">{format!("{} Guests", r.capacity)}</td>
                                    <td class="px-4 py-3">{format!("ETB {}", r.price)}</td>
                                    <td class="px-4 py-3 text-slate-500">{r.amenities.join(", ")}</td>
                                    <td class="px-4 py-3">
                                        <span class=format!("flex items-center gap-1.5 text-xs font-semibold {text}")>
                                            <span class=format!("h-1.5 w-1.5 rounded-full {dot}")></span>
                                            {r.status}
                                        </span>
                                    </td>
                                    <td class="px-4 py-3">
                                        <div class="flex items-center gap-2 text-slate-400">
                                            <A href=format!("/rooms/{}", r.number) attr:class="hover:text-slate-700"><Icon name="eye" class="h-4 w-4" /></A>
                                            <A href=format!("/rooms/{}", r.number) attr:class="hover:text-slate-700"><Icon name="edit" class="h-4 w-4" /></A>
                                            <button class="hover:text-slate-700"><Icon name="calendar" class="h-4 w-4" /></button>
                                            <button class="hover:text-slate-700"><Icon name="more-v" class="h-4 w-4" /></button>
                                        </div>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>

                <Show when=move || visible_rooms().is_empty()>
                    <div class="p-10 text-center text-sm text-slate-500">"No rooms match your search."</div>
                </Show>
            </div>
            <p class="mt-3 text-sm text-slate-500">{move || format!("Showing {} of {} rooms", visible_rooms().len(), rooms.get().len())}</p>

            <div class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-3 text-base font-semibold text-slate-900">"Quick Actions"</h2>
                <div class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
                    <button on:click=move |_| modal_open.set(true) class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 text-left transition-colors hover:bg-slate-50">
                        <Icon name="calendar" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Add Room"</span><span class="text-xs text-slate-400">"Add a new room to your inventory"</span></span>
                    </button>
                    <A href="/rooms" attr:class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 hover:bg-slate-50">
                        <Icon name="tag" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Set Special Rate"</span><span class="text-xs text-slate-400">"Create special rates for rooms"</span></span>
                    </A>
                    <A href="/rooms" attr:class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 hover:bg-slate-50">
                        <Icon name="lock" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Block Room"</span><span class="text-xs text-slate-400">"Block rooms for maintenance or events"</span></span>
                    </A>
                    <A href="/calendar" attr:class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 hover:bg-slate-50">
                        <Icon name="calendar-check" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"View Availability Calendar"</span><span class="text-xs text-slate-400">"Check room availability by date"</span></span>
                    </A>
                </div>
            </div>

            <Show when=move || modal_open.get()>
                <AddRoomModal
                    on_close=move || modal_open.set(false)
                    on_create=move |room| rooms.update(|r| r.insert(0, room))
                />
            </Show>
        </div>
    }
}

#[component]
fn AddRoomModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_create: impl Fn(Room) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let number = RwSignal::new(String::new());
    let room_type = RwSignal::new(ROOM_TYPES[0].to_string());
    let floor = RwSignal::new(String::new());
    let capacity = RwSignal::new(String::new());
    let price = RwSignal::new(String::new());
    let breakfast = RwSignal::new(false);
    let status = RwSignal::new(STATUSES[0].to_string());
    let error = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if number.get().trim().is_empty() {
            error.set(Some("Please enter a room number.".to_string()));
            return;
        }
        let mut amenities: Vec<&'static str> = vec!["Wi-Fi", "TV", "AC"];
        if breakfast.get() {
            amenities.push("Breakfast");
        }
        let room = Room {
            number: Box::leak(number.get().into_boxed_str()),
            room_type: Box::leak(room_type.get().into_boxed_str()),
            floor: floor.get().trim().parse().unwrap_or(1),
            capacity: capacity.get().trim().parse().unwrap_or(2),
            price: price.get().trim().parse().unwrap_or(0),
            amenities: Box::leak(amenities.into_boxed_slice()),
            status: Box::leak(status.get().into_boxed_str()),
            discount_percent: 0,
            breakfast_included: breakfast.get(),
            uuid: Box::leak(format!("{:0>8}-0000-0000-0000-000000000000", number.get()).into_boxed_str()),
            created_at: "Just now",
            updated_at: "Just now",
        };
        on_create(room);
        on_close();
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

                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Create Room"</button>
                </div>
            </form>
        </Modal>
    }
}
