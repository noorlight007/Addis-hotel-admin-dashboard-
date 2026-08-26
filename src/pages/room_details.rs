use crate::components::{Icon, Modal, Toggle};
use crate::data::{find_room, Room};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

const ALL_FACILITIES: &[(&str, &str)] = &[
    ("wifi", "Free Wi-Fi"), ("snowflake", "Air Conditioning"), ("tv", "TV"), ("minibar", "Minibar"),
    ("safe", "Safe"), ("desk", "Desk"), ("coffee", "Tea/Coffee Maker"), ("hairdryer", "Hair Dryer"),
    ("telephone", "Telephone"), ("slippers", "Slippers"), ("wardrobe", "Wardrobe"), ("iron", "Iron & Ironing Board"),
    ("crib", "Baby Cot"), ("bell-concierge", "Room Service"), ("balcony", "Balcony"), ("bathtub", "Bathtub"),
    ("breakfast", "Breakfast"),
];

/// Room.amenities uses short codes ("Wi-Fi", "AC") from the rooms list/table view;
/// the facilities checklist here uses full descriptive labels. Map short to full so
/// a room's stored amenities show as checked against the right checklist entry.
fn normalize_amenity(label: &'static str) -> &'static str {
    match label {
        "Wi-Fi" => "Free Wi-Fi",
        "AC" => "Air Conditioning",
        "Coffee" => "Tea/Coffee Maker",
        other => other,
    }
}

fn amenities_match(current: &[&'static str], label: &str) -> bool {
    current.iter().any(|c| *c == label || normalize_amenity(c) == label)
}

const ROOM_TYPES: &[&str] = &["Standard Room", "Deluxe Room", "Twin Room", "Family Room", "Executive Suite", "Single Room"];
const STATUSES: &[&str] = &["Available", "Occupied", "Reserved", "Maintenance"];
const BED_TYPES: &[&str] = &["Single Bed", "Twin Bed", "Double Bed", "Queen Bed", "King Bed", "Sofa Bed", "Bunk Bed", "Crib / Baby Cot"];

const STOCK_PHOTOS: &[&str] = &[
    "https://images.unsplash.com/photo-1611892440504-42a792e24d32?q=80&w=300",
    "https://images.unsplash.com/photo-1595576508898-0ad5c879a061?q=80&w=300",
    "https://images.unsplash.com/photo-1584132967334-10e028bd69f7?q=80&w=300",
    "https://images.unsplash.com/photo-1620626011761-996317b8d101?q=80&w=300",
    "https://images.unsplash.com/photo-1560185127-6ed189bf02f4?q=80&w=300",
    "https://images.unsplash.com/photo-1522771739844-6a9f6d5f14af?q=80&w=300",
];

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bed {
    bed_type: &'static str,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpecialEvent {
    event_type: &'static str,
    start: &'static str,
    end: &'static str,
}

#[component]
pub fn RoomDetailsPage() -> impl IntoView {
    let params = use_params_map();
    let active_tab = RwSignal::new("Overview");
    let initial = move || {
        let number = params.get().get("number").unwrap_or_default();
        find_room(&number).copied()
    };

    view! {
        <Show
            when=move || initial().is_some()
            fallback=|| view! { <div class="p-6 text-center text-slate-500">"Room not found."</div> }
        >
            {move || {
                let room = RwSignal::new(initial().unwrap());
                let photos = RwSignal::new(STOCK_PHOTOS[..3].to_vec());
                let beds = RwSignal::new(vec![Bed { bed_type: "King Bed", count: 1 }, Bed { bed_type: "Sofa Bed", count: 1 }]);
                let events = RwSignal::new(vec![
                    SpecialEvent { event_type: "Blocked", start: "Jun 10, 2024", end: "Jun 15, 2024" },
                    SpecialEvent { event_type: "Unavailable", start: "Jul 1, 2024", end: "Jul 3, 2024" },
                ]);

                let info_modal = RwSignal::new(false);
                let facilities_modal = RwSignal::new(false);
                let beds_modal = RwSignal::new(false);
                let event_modal = RwSignal::new(false);
                let edit_event = RwSignal::new(Option::<usize>::None);
                let photos_edit = RwSignal::new(false);
                let facility_search = RwSignal::new(String::new());
                let actions_open = RwSignal::new(false);
                let navigate = use_navigate();

                view! {
                    <div class="p-4 sm:p-6">
                        <A href="/rooms" attr:class="mb-3 inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                            <Icon name="chevron-left" class="h-4 w-4" />
                            "Back to Rooms"
                        </A>
                        <div class="mb-2 flex items-center justify-between">
                            <div class="flex items-center gap-3">
                                <h1 class="text-xl font-bold text-slate-900">{move || format!("Room {}", room.get().number)}</h1>
                                <span class=move || format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(room.get().status))>{move || room.get().status}</span>
                            </div>
                            <div class="relative">
                                <button
                                    on:click=move |_| actions_open.update(|v| *v = !*v)
                                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50"
                                >
                                    "Actions"
                                    <Icon name="chevron-down" class="h-4 w-4" />
                                </button>
                                <Show when=move || actions_open.get()>
                                    <div class="absolute right-0 top-full z-10 mt-1 w-52 animate-scale-in rounded-lg border border-slate-200 bg-white p-1.5 text-sm shadow-lg">
                                        <button
                                            on:click=move |_| { room.update(|r| r.status = "Maintenance"); actions_open.set(false); }
                                            class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-slate-700 hover:bg-slate-50"
                                        >
                                            <Icon name="wrench" class="h-4 w-4" />
                                            "Mark Under Maintenance"
                                        </button>
                                        <button
                                            on:click=move |_| { room.update(|r| r.status = "Available"); actions_open.set(false); }
                                            class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-slate-700 hover:bg-slate-50"
                                        >
                                            <Icon name="check-circle" class="h-4 w-4" />
                                            "Mark Available"
                                        </button>
                                        <button
                                            on:click={
                                                let navigate = navigate.clone();
                                                move |_| navigate("/rooms", Default::default())
                                            }
                                            class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-red-600 hover:bg-red-50"
                                        >
                                            <Icon name="trash" class="h-4 w-4" />
                                            "Delete Room"
                                        </button>
                                    </div>
                                </Show>
                            </div>
                        </div>

                        <div class="mb-6 flex gap-6 border-b border-slate-200 text-sm font-medium text-slate-500">
                            {["Overview", "Photos", "Special Events"].iter().map(|t| {
                                let t = *t;
                                view! {
                                    <button
                                        class=move || format!("-mb-px border-b-2 pb-2 {}", if active_tab.get() == t { "border-blue-700 text-blue-700" } else { "border-transparent" })
                                        on:click=move |_| active_tab.set(t)
                                    >
                                        {t}
                                    </button>
                                }
                            }).collect_view()}
                        </div>

                        {move || match active_tab.get() {
                            "Photos" => view! {
                                <div class="rounded-xl border border-slate-200 bg-white p-5">
                                    <div class="mb-3 flex items-center justify-between">
                                        <h2 class="flex items-center gap-2 text-base font-semibold text-slate-900">
                                            <Icon name="image" class="h-4 w-4 text-blue-600" />
                                            "Photos"
                                        </h2>
                                        <button on:click=move |_| photos_edit.update(|v| *v = !*v) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline">
                                            <Icon name="edit" class="h-4 w-4" />
                                            {move || if photos_edit.get() { "Done" } else { "Edit" }}
                                        </button>
                                    </div>
                                    <div class="grid grid-cols-2 gap-3 sm:grid-cols-5">
                                        {move || photos.get().into_iter().enumerate().map(|(i, p)| view! {
                                            <div class="group relative overflow-hidden rounded-lg">
                                                <img src=p alt="" class="h-28 w-full object-cover" />
                                                <button
                                                    on:click=move |_| photos.update(|list| { list.remove(i); })
                                                    class=move || format!(
                                                        "absolute right-1.5 top-1.5 flex h-6 w-6 items-center justify-center rounded-full bg-black/60 text-white transition-opacity group-hover:opacity-100 {}",
                                                        if photos_edit.get() { "opacity-100" } else { "opacity-0" }
                                                    )
                                                >
                                                    <Icon name="x" class="h-3.5 w-3.5" />
                                                </button>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                    <button
                                        on:click=move |_| photos.update(|list| {
                                            let next = STOCK_PHOTOS[list.len() % STOCK_PHOTOS.len()];
                                            list.push(next);
                                        })
                                        class="mt-3 flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"
                                    >
                                        <Icon name="plus" class="h-4 w-4" />
                                        "Add Photos"
                                    </button>
                                </div>
                            }.into_any(),
                            "Special Events" => view! {
                                <div class="rounded-xl border border-slate-200 bg-white p-5">
                                    <div class="mb-1 flex items-center justify-between">
                                        <h2 class="flex items-center gap-2 text-base font-semibold text-slate-900">
                                            <Icon name="flag" class="h-4 w-4 text-blue-600" />
                                            "Special Events"
                                        </h2>
                                        <button
                                            on:click=move |_| { edit_event.set(None); event_modal.set(true); }
                                            class="flex items-center gap-1.5 rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 transition-colors hover:bg-slate-50"
                                        >
                                            <Icon name="plus" class="h-4 w-4" />
                                            "Add Event"
                                        </button>
                                    </div>
                                    <p class="mb-4 text-sm text-slate-500">"In these events, the room cannot take bookings."</p>
                                    <table class="w-full text-left text-sm">
                                        <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                            <tr><th class="py-2">"Event Type"</th><th class="py-2">"Start Date"</th><th class="py-2">"End Date"</th><th class="py-2">"Status"</th><th class="py-2">"Actions"</th></tr>
                                        </thead>
                                        <tbody>
                                            {move || events.get().into_iter().enumerate().map(|(i, ev)| view! {
                                                <tr class="border-b border-slate-100 last:border-0">
                                                    <td class="py-2"><span class="rounded-full bg-red-100 px-2 py-0.5 text-xs font-semibold text-red-700">{ev.event_type}</span></td>
                                                    <td class="py-2">{ev.start}</td>
                                                    <td class="py-2">{ev.end}</td>
                                                    <td class="py-2"><span class="rounded-full bg-amber-100 px-2 py-0.5 text-xs font-semibold text-amber-700">"Upcoming"</span></td>
                                                    <td class="py-2">
                                                        <div class="flex items-center gap-2 text-slate-400">
                                                            <button on:click=move |_| { edit_event.set(Some(i)); event_modal.set(true); } class="hover:text-blue-700">
                                                                <Icon name="edit" class="h-4 w-4" />
                                                            </button>
                                                            <button on:click=move |_| events.update(|list| { list.remove(i); }) class="hover:text-red-600">
                                                                <Icon name="trash" class="h-4 w-4" />
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    <Show when=move || events.get().is_empty()>
                                        <p class="py-6 text-center text-sm text-slate-400">"No special events scheduled."</p>
                                    </Show>
                                </div>

                                <Show when=move || event_modal.get()>
                                    <AddEventModal
                                        initial=edit_event.get().and_then(|i| events.get_untracked().get(i).copied())
                                        on_close=move || { event_modal.set(false); edit_event.set(None); }
                                        on_create=move |e| {
                                            events.update(|list| {
                                                match edit_event.get_untracked() {
                                                    Some(i) if i < list.len() => list[i] = e,
                                                    _ => list.push(e),
                                                }
                                            });
                                            edit_event.set(None);
                                        }
                                    />
                                </Show>
                            }.into_any(),
                            _ => view! {
                                <div class="flex flex-col gap-4">
                                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                                        <div class="mb-3 flex items-center justify-between">
                                            <h2 class="flex items-center gap-2 text-base font-semibold text-slate-900">
                                                <Icon name="bed" class="h-4 w-4 text-blue-600" />
                                                "Room Information"
                                            </h2>
                                            <button on:click=move |_| info_modal.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                                        </div>
                                        <div class="grid grid-cols-3 gap-4 text-sm">
                                            <div><p class="text-slate-400">"Room Number"</p><p class="font-medium">{move || room.get().number}</p></div>
                                            <div><p class="text-slate-400">"Price per Night"</p><p class="font-medium">{move || format!("ETB {}", room.get().price)}</p></div>
                                            <div><p class="text-slate-400">"Status"</p><p class="font-medium">{move || room.get().status}</p></div>
                                            <div><p class="text-slate-400">"Room Type"</p><p class="flex items-center gap-1.5 font-medium"><Icon name="bed" class="h-3.5 w-3.5 text-slate-400" />{move || room.get().room_type}</p></div>
                                            <div><p class="text-slate-400">"Discount per Night"</p><p class="font-medium">{move || format!("{}%", room.get().discount_percent)}</p></div>
                                            <div><p class="text-slate-400">"UUID"</p><p class="truncate font-medium" title=move || room.get().uuid>{move || room.get().uuid}</p></div>
                                            <div><p class="text-slate-400">"Floor"</p><p class="font-medium">{move || room.get().floor}</p></div>
                                            <div>
                                                <p class="text-slate-400">"Price after Discount"</p>
                                                <p class="font-medium">{move || {
                                                    let r = room.get();
                                                    let discounted = r.price.saturating_sub(r.price * r.discount_percent / 100);
                                                    format!("ETB {} / night", discounted)
                                                }}</p>
                                            </div>
                                            <div><p class="text-slate-400">"Created At"</p><p class="font-medium">{move || room.get().created_at}</p></div>
                                            <div><p class="text-slate-400">"Guest Capacity"</p><p class="font-medium">{move || format!("{} Guests", room.get().capacity)}</p></div>
                                            <div>
                                                <p class="text-slate-400">"Breakfast Included"</p>
                                                <p class="flex items-center gap-1 font-medium">
                                                    <Show when=move || room.get().breakfast_included fallback=|| view! { <Icon name="x-circle" class="h-4 w-4 text-slate-400" /> }>
                                                        <Icon name="check-circle" class="h-4 w-4 text-emerald-600" />
                                                    </Show>
                                                    {move || if room.get().breakfast_included { "Yes" } else { "No" }}
                                                </p>
                                            </div>
                                            <div><p class="text-slate-400">"Last Updated"</p><p class="font-medium">{move || room.get().updated_at}</p></div>
                                        </div>
                                    </div>

                                    <div class="grid gap-4 lg:grid-cols-3">
                                        <div class="rounded-xl border border-slate-200 bg-white p-5 lg:col-span-2">
                                            <div class="mb-1 flex items-center justify-between">
                                                <h2 class="flex items-center gap-2 text-base font-semibold text-slate-900">
                                                    <Icon name="bed" class="h-4 w-4 text-blue-600" />
                                                    "Beds"
                                                </h2>
                                            </div>
                                            <p class="mb-3 text-sm text-slate-500">"Multiple beds can be in a single room."</p>
                                            <table class="w-full text-left text-sm">
                                                <thead class="text-xs uppercase text-slate-500"><tr><th class="py-1">"#"</th><th class="py-1">"Bed Type"</th><th class="py-1">"Number of Beds"</th><th class="py-1"></th></tr></thead>
                                                <tbody>
                                                    {move || beds.get().into_iter().enumerate().map(|(i, b)| view! {
                                                        <tr>
                                                            <td class="py-1">{i + 1}</td>
                                                            <td class="py-1 font-medium">{b.bed_type}</td>
                                                            <td class="py-1">{b.count}</td>
                                                            <td class="py-1 text-right">
                                                                <button on:click=move |_| beds.update(|list| { list.remove(i); }) class="text-slate-400 hover:text-red-600">
                                                                    <Icon name="trash" class="h-3.5 w-3.5" />
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                            <button on:click=move |_| beds_modal.set(true) class="mt-3 flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="plus" class="h-4 w-4" />"Add Bed"</button>
                                        </div>
                                        <div class="rounded-xl border border-slate-200 bg-slate-50 p-5">
                                            <p class="mb-2 text-sm font-semibold text-slate-800">"Examples"</p>
                                            <ul class="flex flex-col gap-1.5 text-sm text-slate-500">
                                                <li>"• 1 King Bed"</li>
                                                <li>"• 2 Double Beds"</li>
                                                <li>"• 2 Queen Beds"</li>
                                                <li>"• 1 King Bed and 1 Sofa Bed"</li>
                                                <li>"• 1 Double Bed and 1 Single Bed"</li>
                                            </ul>
                                        </div>
                                    </div>

                                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                                        <div class="mb-1 flex items-center justify-between">
                                            <h2 class="flex items-center gap-2 text-base font-semibold text-slate-900">
                                                <Icon name="users" class="h-4 w-4 text-blue-600" />
                                                "Room Facilities"
                                            </h2>
                                            <button on:click=move |_| facilities_modal.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                                        </div>
                                        <p class="mb-3 text-sm text-slate-500">"Search and select the facilities available in this room."</p>
                                        <div class="relative mb-3 max-w-sm">
                                            <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                            <input
                                                type="text"
                                                placeholder="Search facilities..."
                                                class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                                                prop:value=facility_search
                                                on:input:target=move |ev| facility_search.set(ev.target().value())
                                            />
                                        </div>
                                        <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
                                            {move || {
                                                let current = room.get().amenities;
                                                let q = facility_search.get().to_lowercase();
                                                ALL_FACILITIES.iter().filter(|(_, label)| q.is_empty() || label.to_lowercase().contains(&q)).map(|(icon, label)| {
                                                    let checked = amenities_match(current, label);
                                                    view! {
                                                        <span class=format!(
                                                            "flex items-center gap-2 rounded-lg border px-3 py-2 text-sm {}",
                                                            if checked { "border-blue-200 bg-blue-50/50 text-slate-800" } else { "border-slate-200 text-slate-400" }
                                                        )>
                                                            <span class=format!(
                                                                "flex h-4 w-4 shrink-0 items-center justify-center rounded border {}",
                                                                if checked { "border-blue-600 bg-blue-600 text-white" } else { "border-slate-300" }
                                                            )>
                                                                <Show when=move || checked>
                                                                    <Icon name="check" class="h-3 w-3" />
                                                                </Show>
                                                            </span>
                                                            <Icon name=*icon class="h-4 w-4" />
                                                            {*label}
                                                        </span>
                                                    }
                                                }).collect_view()
                                            }}
                                        </div>
                                        <p class="mt-2 text-right text-xs text-slate-400">{move || format!("Selected {} of {} facilities", room.get().amenities.len(), ALL_FACILITIES.len())}</p>
                                    </div>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    <Show when=move || info_modal.get()>
                        <EditRoomInfoModal room=room on_close=move || info_modal.set(false) />
                    </Show>
                    <Show when=move || facilities_modal.get()>
                        <EditFacilitiesModal room=room on_close=move || facilities_modal.set(false) />
                    </Show>
                    <Show when=move || beds_modal.get()>
                        <AddBedModal on_close=move || beds_modal.set(false) on_create=move |b| beds.update(|list| list.push(b)) />
                    </Show>
                }
            }}
        </Show>
    }
}

#[component]
fn EditRoomInfoModal(room: RwSignal<Room>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let r0 = room.get_untracked();
    let room_type = RwSignal::new(r0.room_type.to_string());
    let floor = RwSignal::new(r0.floor.to_string());
    let capacity = RwSignal::new(r0.capacity.to_string());
    let price = RwSignal::new(r0.price.to_string());
    let status = RwSignal::new(r0.status.to_string());
    let discount_percent = RwSignal::new(r0.discount_percent.to_string());
    let breakfast_included = RwSignal::new(r0.breakfast_included);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        room.update(|r| {
            r.room_type = Box::leak(room_type.get().into_boxed_str());
            r.floor = floor.get().trim().parse().unwrap_or(r.floor);
            r.capacity = capacity.get().trim().parse().unwrap_or(r.capacity);
            r.price = price.get().trim().parse().unwrap_or(r.price);
            r.status = Box::leak(status.get().into_boxed_str());
            r.discount_percent = discount_percent.get().trim().parse().unwrap_or(r.discount_percent);
            r.breakfast_included = breakfast_included.get();
        });
        on_close();
    };

    view! {
        <Modal title="Edit Room Information" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| room_type.set(ev.target().value())>
                            {ROOM_TYPES.iter().map(|t| view! { <option value=*t selected=*t == room_type.get_untracked()>{*t}</option> }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Status"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| status.set(ev.target().value())>
                            {STATUSES.iter().map(|s| view! { <option value=*s selected=*s == status.get_untracked()>{*s}</option> }).collect_view()}
                        </select>
                    </div>
                </div>
                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Floor"</label>
                        <input type="number" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=floor on:input:target=move |ev| floor.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest Capacity"</label>
                        <input type="number" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=capacity on:input:target=move |ev| capacity.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Price / Night"</label>
                        <input type="number" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=price on:input:target=move |ev| price.set(ev.target().value()) />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Discount per Night (%)"</label>
                        <input type="number" min="0" max="100" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=discount_percent on:input:target=move |ev| discount_percent.set(ev.target().value()) />
                    </div>
                    <div class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
                        <span class="flex items-center gap-2 text-sm text-slate-700">
                            <Icon name="coffee" class="h-4 w-4 text-slate-400" />
                            "Breakfast Included"
                        </span>
                        <button type="button" on:click=move |_| breakfast_included.update(|v| *v = !*v)>
                            <Toggle checked=breakfast_included.get() />
                        </button>
                    </div>
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn EditFacilitiesModal(room: RwSignal<Room>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let selected = RwSignal::new(room.get_untracked().amenities.iter().map(|a| normalize_amenity(a)).collect::<Vec<_>>());

    let toggle = move |label: &'static str| {
        selected.update(|list| {
            if let Some(pos) = list.iter().position(|l| *l == label) {
                list.remove(pos);
            } else {
                list.push(label);
            }
        });
    };

    let save = move |_| {
        room.update(|r| r.amenities = Box::leak(selected.get().into_boxed_slice()));
        on_close();
    };

    view! {
        <Modal title="Edit Room Facilities" on_close=on_close width="max-w-2xl">
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
                {ALL_FACILITIES.iter().map(|(icon, label)| {
                    let label = *label;
                    let icon = *icon;
                    view! {
                        <button
                            type="button"
                            on:click=move |_| toggle(label)
                            class=move || format!(
                                "flex items-center gap-2 rounded-lg border px-3 py-2 text-left text-sm transition-colors {}",
                                if selected.get().contains(&label) { "border-blue-300 bg-blue-50 text-slate-800" } else { "border-slate-200 text-slate-500 hover:bg-slate-50" }
                            )
                        >
                            <span class=move || format!(
                                "flex h-4 w-4 shrink-0 items-center justify-center rounded border {}",
                                if selected.get().contains(&label) { "border-blue-600 bg-blue-600 text-white" } else { "border-slate-300" }
                            )>
                                <Show when=move || selected.get().contains(&label)>
                                    <Icon name="check" class="h-3 w-3" />
                                </Show>
                            </span>
                            <Icon name=icon class="h-4 w-4" />
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>
            <p class="mt-3 text-right text-xs text-slate-400">{move || format!("Selected {} of {} facilities", selected.get().len(), ALL_FACILITIES.len())}</p>
            <div class="mt-4 flex justify-end gap-3 border-t border-slate-100 pt-4">
                <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                <button type="button" on:click=save class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
            </div>
        </Modal>
    }
}

#[component]
fn AddBedModal(on_close: impl Fn() + Copy + Send + Sync + 'static, on_create: impl Fn(Bed) + Copy + Send + Sync + 'static) -> impl IntoView {
    let bed_type = RwSignal::new(BED_TYPES[0].to_string());
    let count = RwSignal::new("1".to_string());

    let submit = move |_| {
        on_create(Bed {
            bed_type: Box::leak(bed_type.get().into_boxed_str()),
            count: count.get().trim().parse().unwrap_or(1),
        });
        on_close();
    };

    view! {
        <Modal title="Add Bed" on_close=on_close>
            <div class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Bed Type"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| bed_type.set(ev.target().value())>
                        {BED_TYPES.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                    </select>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Number of Beds"</label>
                    <input type="number" min="1" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=count on:input:target=move |ev| count.set(ev.target().value()) />
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="button" on:click=submit class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Add Bed"</button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn AddEventModal(
    initial: Option<SpecialEvent>,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_create: impl Fn(SpecialEvent) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let is_edit = initial.is_some();
    let event_type = RwSignal::new(initial.map(|e| e.event_type).unwrap_or("Blocked").to_string());
    let start = RwSignal::new(initial.map(|e| e.start).unwrap_or_default().to_string());
    let end = RwSignal::new(initial.map(|e| e.end).unwrap_or_default().to_string());
    let error = RwSignal::new(Option::<String>::None);

    let submit = move |_| {
        if start.get().trim().is_empty() || end.get().trim().is_empty() {
            error.set(Some("Please provide both a start and end date.".to_string()));
            return;
        }
        on_create(SpecialEvent {
            event_type: Box::leak(event_type.get().into_boxed_str()),
            start: Box::leak(start.get().into_boxed_str()),
            end: Box::leak(end.get().into_boxed_str()),
        });
        on_close();
    };

    view! {
        <Modal title=if is_edit { "Edit Special Event" } else { "Add Special Event" } on_close=on_close>
            <div class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Event Type"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| event_type.set(ev.target().value())>
                        <option value="Blocked" selected=move || event_type.get_untracked() == "Blocked">"Blocked"</option>
                        <option value="Unavailable" selected=move || event_type.get_untracked() == "Unavailable">"Unavailable"</option>
                    </select>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Start Date"</label>
                        <input type="text" placeholder="e.g. Jun 10, 2024" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=start on:input:target=move |ev| start.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"End Date"</label>
                        <input type="text" placeholder="e.g. Jun 15, 2024" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=end on:input:target=move |ev| end.set(ev.target().value()) />
                    </div>
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="button" on:click=submit class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">{if is_edit { "Save Changes" } else { "Add Event" }}</button>
                </div>
            </div>
        </Modal>
    }
}
