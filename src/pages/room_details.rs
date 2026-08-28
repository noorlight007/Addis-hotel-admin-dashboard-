//! One room, backed by `GET /rooms/{id}/`.
//!
//! The route is keyed on the human room number, so the id is resolved through a
//! search first (see [`api::get_room_by_number`]).
//!
//! Editing is spread across the endpoints the API actually provides:
//!
//! | Section    | Endpoint |
//! |------------|----------|
//! | Details    | `PATCH /rooms/{id}/` |
//! | Status     | `PATCH /rooms/{id}/status/` |
//! | Amenities  | `POST /rooms/{id}/amenities/sync/` |
//! | Beds       | `POST /rooms/{id}/beds/` |
//! | Photos     | `POST /rooms/{id}/photos/`, `DELETE …/{photo_id}/` |
//! | Blocks     | `POST /rooms/{id}/special-events/`, `DELETE|end …` |
//! | Policy     | `PATCH /rooms/{id}/policies/` |

use crate::api;
use crate::components::{use_toast, Badge, Card, Icon, Modal, Toggle};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

const TABS: &[&str] = &["Overview", "Amenities", "Photos", "Availability", "Policy"];

#[component]
pub fn RoomDetailsPage() -> impl IntoView {
    let toast = use_toast();
    let params = use_params_map();
    let navigate = use_navigate();
    let tab = RwSignal::new(TABS[0]);
    let refresh = RwSignal::new(0u32);
    let edit_open = RwSignal::new(false);
    let block_open = RwSignal::new(false);

    let room = LocalResource::new(move || {
        let number = params.get().get("number").unwrap_or_default();
        let _ = refresh.get();
        async move { api::get_room_by_number(&number).await }
    });

    let bump = move || refresh.update(|n| *n += 1);

    let set_status = move |id: i64, status: String| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::set_room_status(id, &status).await {
                Ok(()) => {
                    toast.success("Status updated", format!("The room is now {status}."));
                    bump();
                }
                Err(e) => toast.error("Could not change the status", e.detail()),
            }
        });
    };

    let nav = StoredValue::new(navigate);
    let delete_room = move |id: i64, number: String| {
        if !crate::confirm(&format!(
            "Delete room {number}? The API refuses this if the room has reservations."
        )) {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_room(id).await {
                Ok(()) => {
                    toast.success("Room deleted", "It has been removed from your inventory.");
                    nav.with_value(|n| n("/rooms", Default::default()));
                }
                Err(e) => toast.error("Could not delete the room", e.detail()),
            }
        });
    };

    view! {
        <div class="p-4 sm:p-6">
            <A href="/rooms" attr:class="mb-3 inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                <Icon name="chevron-left" class="h-4 w-4" />
                "Back to Rooms"
            </A>

            <Suspense fallback=|| view! {
                <p class="py-16 text-center text-sm text-slate-400">"Loading room…"</p>
            }>
                {move || Suspend::new(async move {
                    let r = match room.await {
                        Ok(r) => r,
                        Err(e) => return view! {
                            <div class="py-16 text-center">
                                <p class="text-sm text-red-600">{e.detail()}</p>
                                <A href="/rooms" attr:class="mt-3 inline-block text-sm text-blue-700 hover:underline">"Back to Rooms"</A>
                            </div>
                        }.into_any(),
                    };
                    let id = r.id;
                    let number = r.room_number.clone();
                    let status = r.status_str().to_string();
                    let detail_for_edit = r.clone();
                    let r_overview = r.clone();
                    let r_beds = r.clone();
                    let r_amenities = r.clone();
                    let r_photos = r.clone();
                    let r_avail = r.clone();
                    let r_policy = r.clone();

                    view! {
                        // ---- Header ------------------------------------
                        <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
                            <div>
                                <div class="flex flex-wrap items-center gap-3">
                                    <h1 class="text-xl font-bold text-slate-900">{format!("Room {number}")}</h1>
                                    <span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(&status))>
                                        {status.clone()}
                                    </span>
                                    {r.breakfast_included.then(|| view! {
                                        <Badge label="Breakfast included" tone="green" />
                                    })}
                                </div>
                                <p class="mt-1 text-sm text-slate-500">
                                    {format!(
                                        "{} · {} · floor {} · sleeps {}",
                                        r.name,
                                        r.type_str(),
                                        r.floor.map(|f| f.to_string()).unwrap_or_else(|| "—".into()),
                                        r.guest_capacity,
                                    )}
                                </p>
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <select
                                    class="rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium"
                                    on:change:target={
                                        let current = status.clone();
                                        move |ev| {
                                            let next = ev.target().value();
                                            if next != current {
                                                set_status(id, next);
                                            }
                                        }
                                    }
                                >
                                    {api::ROOM_STATUSES.iter().map(|s| view! {
                                        <option value=*s selected=(*s == status)>{*s}</option>
                                    }).collect_view()}
                                </select>
                                <button
                                    on:click=move |_| edit_open.set(true)
                                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-800"
                                >
                                    <Icon name="edit" class="h-4 w-4" />
                                    "Edit room"
                                </button>
                                <button
                                    title="Delete room"
                                    class="rounded-lg border border-red-300 px-3 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
                                    on:click={
                                        let num = number.clone();
                                        move |_| delete_room(id, num.clone())
                                    }
                                >
                                    <Icon name="trash" class="h-4 w-4" />
                                </button>
                            </div>
                        </div>

                        // ---- Price band --------------------------------
                        <div class="mb-5 grid grid-cols-2 gap-3 rounded-xl border border-slate-200 bg-white p-4 sm:grid-cols-4">
                            <Metric label="Base rate" value=format!("ETB {}", api::money_round(r.price_per_night.as_deref())) />
                            <Metric label="Discount" value=format!("{}%", r.discount_percent_per_night.unwrap_or(0)) />
                            <Metric label="Sells at" value=format!("ETB {}", api::money_round(r.price_after_discount.as_deref())) />
                            <Metric label="Beds" value=r.bed_summary() />
                        </div>

                        // ---- Tabs --------------------------------------
                        <div class="mb-4 flex flex-wrap gap-2 text-sm">
                            {TABS.iter().map(|t| {
                                let t = *t;
                                view! {
                                    <button
                                        class=move || format!(
                                            "rounded-lg px-3.5 py-2 font-semibold transition-all duration-200 {}",
                                            if tab.get() == t { "bg-blue-700 text-white shadow-sm" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                                        )
                                        on:click=move |_| tab.set(t)
                                    >
                                        {t}
                                    </button>
                                }
                            }).collect_view()}
                        </div>

                        // ================= OVERVIEW =================
                        <Show when=move || (tab.get() == "Overview")>
                            <div class="grid gap-5 lg:grid-cols-2">
                                <OverviewCard room=r_overview.clone() />
                                <BedsCard room=r_beds.clone() on_saved=move || bump() />
                            </div>
                        </Show>

                        // ================= AMENITIES =================
                        <Show when=move || (tab.get() == "Amenities")>
                            <AmenitiesCard room=r_amenities.clone() on_saved=move || bump() />
                        </Show>

                        // ================= PHOTOS =================
                        <Show when=move || (tab.get() == "Photos")>
                            <PhotosCard room=r_photos.clone() on_saved=move || bump() />
                        </Show>

                        // ================= AVAILABILITY =================
                        <Show when=move || (tab.get() == "Availability")>
                            <AvailabilityCard
                                room=r_avail.clone()
                                on_block=move || block_open.set(true)
                                on_changed=move || bump()
                            />
                        </Show>

                        // ================= POLICY =================
                        <Show when=move || (tab.get() == "Policy")>
                            <PolicyCard room=r_policy.clone() on_saved=move || bump() />
                        </Show>

                        <Show when=move || edit_open.get()>
                            <EditRoomModal
                                room=detail_for_edit.clone()
                                on_close=move || edit_open.set(false)
                                on_saved=move || {
                                    toast.success("Room saved", "The changes are live.");
                                    bump();
                                }
                            />
                        </Show>

                        <Show when=move || block_open.get()>
                            <BlockRoomModal
                                room_id=id
                                on_close=move || block_open.set(false)
                                on_done=move || {
                                    toast.success("Dates blocked", "The range is no longer sellable.");
                                    bump();
                                }
                            />
                        </Show>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

/// Read-only summary of the room record.
#[component]
fn OverviewCard(room: api::RoomDetail) -> impl IntoView {
    view! {
        <Card title="Room details">
            <dl class="flex flex-col gap-3 text-sm">
                <Row label="Room number" value=room.room_number.clone() />
                <Row label="Display name" value=room.name.clone() />
                <Row label="Type" value=room.type_str().to_string() />
                <Row label="Floor" value=room.floor.map(|f| f.to_string()).unwrap_or_default() />
                <Row label="Guest capacity" value=room.guest_capacity.to_string() />
                <Row label="Breakfast" value=(if room.breakfast_included { "Included" } else { "Not included" }).to_string() />
                <Row label="Beds" value=room.bed_summary() />
                <Row label="Created" value=api::pretty_datetime(room.created_at.as_deref()) />
                <Row label="Last updated" value=api::pretty_datetime(room.updated_at.as_deref()) />
            </dl>
        </Card>
    }
}

/// Blocked ranges on this room, with the two ways the API lets you clear one:
/// end it now (keeps the record) or delete it outright.
#[component]
fn AvailabilityCard(
    room: api::RoomDetail,
    on_block: impl Fn() + Copy + Send + Sync + 'static,
    on_changed: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let events = room.special_events.clone();

    view! {
        <Card
            title="Blocked ranges"
            hint="Dates this room is not sellable"
            action=Box::new(move || view! {
                <button
                    on:click=move |_| on_block()
                    class="rounded-lg bg-blue-700 px-3 py-1.5 text-xs font-semibold text-white hover:bg-blue-800"
                >
                    "Block dates"
                </button>
            }.into_any())
        >
            {if events.is_empty() {
                view! {
                    <p class="py-8 text-center text-sm text-slate-400">
                        "Nothing blocked. This room is open on every date it is not booked."
                    </p>
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-2.5">
                        {events.into_iter().map(|e| {
                            let eid = e.id;
                            let active = e.is_active;
                            view! {
                                <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 p-3">
                                    <div class="min-w-0">
                                        <p class="flex flex-wrap items-center gap-2 text-sm font-semibold text-slate-900">
                                            {e.event_type.clone().unwrap_or_else(|| "Blocked".into())}
                                            <Badge
                                                label=if active { "Active" } else { "Ended" }
                                                tone=if active { "red" } else { "slate" }
                                            />
                                        </p>
                                        <p class="mt-0.5 text-xs text-slate-500">
                                            {format!(
                                                "{} → {}",
                                                api::pretty_date(e.event_start_date.as_deref()),
                                                api::pretty_date(e.event_finished_date.as_deref()),
                                            )}
                                        </p>
                                        {e.note.clone().filter(|n| !n.is_empty()).map(|n| view! {
                                            <p class="mt-0.5 text-xs text-slate-400">{n}</p>
                                        })}
                                    </div>
                                    <div class="flex gap-2">
                                        <Show when=move || active>
                                            <button
                                                class="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium hover:bg-slate-50"
                                                on:click=move |_| {
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        match api::end_special_event(eid).await {
                                                            Ok(()) => { toast.success("Block ended", "The room is sellable again."); on_changed(); }
                                                            Err(err) => toast.error("Could not end the block", err.detail()),
                                                        }
                                                    });
                                                }
                                            >
                                                "End now"
                                            </button>
                                        </Show>
                                        <button
                                            class="rounded-lg border border-red-300 px-3 py-1.5 text-xs font-medium text-red-700 hover:bg-red-50"
                                            on:click=move |_| {
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    match api::delete_special_event(eid).await {
                                                        Ok(()) => { toast.success("Block removed", "The range has been deleted."); on_changed(); }
                                                        Err(err) => toast.error("Could not remove the block", err.detail()),
                                                    }
                                                });
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </Card>
    }
}

/// Bed configuration — `POST /rooms/{id}/beds/` replaces the whole set.
#[component]
fn BedsCard(
    room: api::RoomDetail,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = room.id;
    let beds = RwSignal::new(if room.beds.is_empty() {
        vec![api::Bed {
            id: 0,
            bed_type: api::BED_TYPES[3].to_string(),
            number_of_beds: 1,
        }]
    } else {
        room.beds.clone()
    });
    let busy = RwSignal::new(false);

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let list = beds.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::sync_room_beds(id, &list).await {
                Ok(()) => {
                    toast.success("Beds updated", "The room's sleeping arrangement is saved.");
                    on_saved();
                }
                Err(e) => toast.error("Could not save the beds", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <Card title="Sleeping arrangement" hint="Replaces the whole configuration when saved">
            <div class="flex flex-col gap-2.5">
                {move || beds.get().into_iter().enumerate().map(|(i, b)| view! {
                    <div class="flex items-center gap-2">
                        <select
                            class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| {
                                let v = ev.target().value();
                                beds.update(|list| if let Some(x) = list.get_mut(i) { x.bed_type = v; });
                            }
                        >
                            {api::BED_TYPES.iter().map(|t| view! {
                                <option value=*t selected=(*t == b.bed_type)>{*t}</option>
                            }).collect_view()}
                        </select>
                        <input
                            type="number"
                            min="1"
                            class="w-20 rounded-lg border border-slate-300 px-3 py-2 text-sm tabular-nums"
                            prop:value=b.number_of_beds.to_string()
                            on:input:target=move |ev| {
                                let n = ev.target().value().parse().unwrap_or(1u32);
                                beds.update(|list| if let Some(x) = list.get_mut(i) { x.number_of_beds = n.max(1); });
                            }
                        />
                        <button
                            title="Remove"
                            class="rounded-lg p-2 text-slate-400 hover:text-red-600"
                            on:click=move |_| beds.update(|list| { if list.len() > 1 { list.remove(i); } })
                        >
                            <Icon name="trash" class="h-4 w-4" />
                        </button>
                    </div>
                }).collect_view()}
            </div>

            <div class="mt-3 flex items-center justify-between gap-2 border-t border-slate-100 pt-3">
                <button
                    class="flex items-center gap-1.5 text-sm font-semibold text-blue-700 hover:text-blue-800"
                    on:click=move |_| beds.update(|list| list.push(api::Bed {
                        id: 0,
                        bed_type: api::BED_TYPES[0].to_string(),
                        number_of_beds: 1,
                    }))
                >
                    <Icon name="plus" class="h-4 w-4" />
                    "Add bed"
                </button>
                <button
                    on:click=save
                    disabled=move || busy.get()
                    class="rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                >
                    {move || if busy.get() { "Saving…" } else { "Save beds" }}
                </button>
            </div>
        </Card>
    }
}

/// Amenity checklist, grouped by the catalogue's own categories.
#[component]
fn AmenitiesCard(
    room: api::RoomDetail,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = room.id;
    let picked = RwSignal::new(room.amenity_ids());
    let search = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let catalogue = LocalResource::new(|| async move { api::list_amenities().await });

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let ids = picked.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::sync_room_amenities(id, &ids).await {
                Ok(()) => {
                    toast.success("Amenities updated", "The room's facilities are saved.");
                    on_saved();
                }
                Err(e) => toast.error("Could not save the amenities", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <Card
            title="Amenities"
            hint=Signal::derive(move || format!("{} selected", picked.get().len()))
            action=Box::new(move || view! {
                <button
                    on:click=save
                    disabled=move || busy.get()
                    class="rounded-lg bg-blue-700 px-3 py-1.5 text-xs font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                >
                    {move || if busy.get() { "Saving…" } else { "Save amenities" }}
                </button>
            }.into_any())
        >
            <div class="mb-4 relative">
                <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="text"
                    placeholder="Search amenities"
                    class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                    prop:value=search
                    on:input:target=move |ev| search.set(ev.target().value())
                />
            </div>

            <Suspense fallback=|| view! { <p class="py-6 text-center text-sm text-slate-400">"Loading catalogue…"</p> }>
                {move || Suspend::new(async move {
                    let all = catalogue.await.unwrap_or_default();
                    if all.is_empty() {
                        return view! {
                            <p class="py-6 text-center text-sm text-slate-400">
                                "The amenity catalogue is empty. Add amenities under Hotel Profile first."
                            </p>
                        }.into_any();
                    }
                    let q = search.get().to_lowercase();
                    let mut groups: Vec<(String, Vec<api::Amenity>)> = Vec::new();
                    for a in all.into_iter().filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q)) {
                        let cat = a.category_str().to_string();
                        match groups.iter_mut().find(|(c, _)| *c == cat) {
                            Some(g) => g.1.push(a),
                            None => groups.push((cat, vec![a])),
                        }
                    }
                    if groups.is_empty() {
                        return view! {
                            <p class="py-6 text-center text-sm text-slate-400">"Nothing matches that search."</p>
                        }.into_any();
                    }
                    view! {
                        <div class="flex flex-col gap-5">
                            {groups.into_iter().map(|(cat, items)| view! {
                                <div>
                                    <p class="mb-2 text-2xs font-bold uppercase tracking-wide text-slate-500">{cat}</p>
                                    <div class="flex flex-wrap gap-1.5">
                                        {items.into_iter().map(|a| {
                                            let aid = a.id;
                                            view! {
                                                <button
                                                    type="button"
                                                    on:click=move |_| picked.update(|p| match p.iter().position(|x| *x == aid) {
                                                        Some(i) => { p.remove(i); }
                                                        None => p.push(aid),
                                                    })
                                                    class=move || format!(
                                                        "rounded-lg border px-2.5 py-1 text-xs font-medium transition-colors {}",
                                                        if picked.get().contains(&aid) { "border-blue-600 bg-blue-50 text-blue-700" } else { "border-slate-300 text-slate-600 hover:bg-slate-50" }
                                                    )
                                                >
                                                    {a.name}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }).collect_view()}
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </Card>
    }
}

/// Room gallery. The endpoint is multipart and accepts an `image_url`, which is
/// what the dashboard uses — hosted images rather than direct uploads.
#[component]
fn PhotosCard(
    room: api::RoomDetail,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = room.id;
    let photos = room.photos.clone();
    let url = RwSignal::new(String::new());
    let caption = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let add = move |_| {
        if busy.get() {
            return;
        }
        let u = url.get().trim().to_string();
        if !u.starts_with("http") {
            toast.error("Check the link", "Paste a full image URL starting with http.");
            return;
        }
        busy.set(true);
        let c = caption.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::add_room_photo(id, &u, &c).await {
                Ok(_) => {
                    url.set(String::new());
                    caption.set(String::new());
                    toast.success("Photo added", "It now appears on the public listing.");
                    on_saved();
                }
                Err(e) => toast.error("Could not add the photo", e.detail()),
            }
            busy.set(false);
        });
    };

    let remove = move |photo_id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_room_photo(id, photo_id).await {
                Ok(()) => {
                    toast.success("Photo removed", "It is no longer shown.");
                    on_saved();
                }
                Err(e) => toast.error("Could not remove the photo", e.detail()),
            }
        });
    };

    view! {
        <Card title="Photos" hint=format!("{} in the gallery", photos.len())>
            {if photos.is_empty() {
                view! {
                    <p class="py-8 text-center text-sm text-slate-400">
                        "No photos yet. Guests are far more likely to book a room they can see."
                    </p>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
                        {photos.into_iter().map(|p| {
                            let pid = p.id;
                            view! {
                                <div class="group relative overflow-hidden rounded-xl border border-slate-200">
                                    <img src=p.image_url.clone() alt=p.caption.clone().unwrap_or_default()
                                        class="h-32 w-full object-cover" />
                                    <button
                                        title="Remove"
                                        class="absolute right-1.5 top-1.5 rounded-lg bg-white/90 p-1.5 text-slate-500 opacity-0 shadow transition-opacity hover:text-red-600 group-hover:opacity-100"
                                        on:click=move |_| remove(pid)
                                    >
                                        <Icon name="trash" class="h-3.5 w-3.5" />
                                    </button>
                                    {p.caption.clone().filter(|c| !c.is_empty()).map(|c| view! {
                                        <p class="truncate px-2 py-1.5 text-2xs text-slate-500">{c}</p>
                                    })}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}

            <div class="mt-4 flex flex-col gap-2 border-t border-slate-100 pt-4 sm:flex-row">
                <input
                    type="url"
                    placeholder="https://images.example.com/room.jpg"
                    class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
                    prop:value=url
                    on:input:target=move |ev| url.set(ev.target().value())
                />
                <input
                    type="text"
                    placeholder="Caption (optional)"
                    class="rounded-lg border border-slate-300 px-3 py-2 text-sm sm:w-48"
                    prop:value=caption
                    on:input:target=move |ev| caption.set(ev.target().value())
                />
                <button
                    on:click=add
                    disabled=move || busy.get()
                    class="rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                >
                    {move || if busy.get() { "Adding…" } else { "Add photo" }}
                </button>
            </div>
        </Card>
    }
}

/// Room-level policy overrides.
#[component]
fn PolicyCard(
    room: api::RoomDetail,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = room.id;
    let p = room.policy.clone().unwrap_or_default();
    let policy = RwSignal::new(p);
    let busy = RwSignal::new(false);

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let body = policy.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_room_policy(id, &body).await {
                Ok(_) => {
                    toast.success("Policy saved", "These rules now apply to this room.");
                    on_saved();
                }
                Err(e) => toast.error("Could not save the policy", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <Card title="Room policy" hint="Overrides the property defaults for this room">
            <div class="grid gap-4 sm:grid-cols-2">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Check-in time"</label>
                    <input type="time" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=move || trim_seconds(policy.get().checkin_time)
                        on:input:target=move |ev| {
                            let v = ev.target().value();
                            policy.update(|p| p.checkin_time = Some(v));
                        } />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Check-out time"</label>
                    <input type="time" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=move || trim_seconds(policy.get().checkout_time)
                        on:input:target=move |ev| {
                            let v = ev.target().value();
                            policy.update(|p| p.checkout_time = Some(v));
                        } />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Minimum check-in age"</label>
                    <input type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=move || policy.get().minimum_checkin_age.map(|n| n.to_string()).unwrap_or_default()
                        on:input:target=move |ev| {
                            let v = ev.target().value().parse().ok();
                            policy.update(|p| p.minimum_checkin_age = v);
                        } />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Children age limit"</label>
                    <input type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=move || policy.get().children_age.map(|n| n.to_string()).unwrap_or_default()
                        on:input:target=move |ev| {
                            let v = ev.target().value().parse().ok();
                            policy.update(|p| p.children_age = v);
                        } />
                </div>
            </div>

            <div class="mt-4 flex flex-col gap-2">
                <PolicyToggle label="Children allowed" get=Signal::derive(move || policy.get().children_allowed.unwrap_or(false))
                    set=move |v| policy.update(|p| p.children_allowed = Some(v)) />
                <PolicyToggle label="Extra bed available" get=Signal::derive(move || policy.get().extrabed_available.unwrap_or(false))
                    set=move |v| policy.update(|p| p.extrabed_available = Some(v)) />
                <PolicyToggle label="Pets allowed" get=Signal::derive(move || policy.get().pet_allowed.unwrap_or(false))
                    set=move |v| policy.update(|p| p.pet_allowed = Some(v)) />
                <PolicyToggle label="Smoking allowed" get=Signal::derive(move || policy.get().smoking_allowed.unwrap_or(false))
                    set=move |v| policy.update(|p| p.smoking_allowed = Some(v)) />
                <PolicyToggle label="Government ID required" get=Signal::derive(move || policy.get().government_id_required.unwrap_or(false))
                    set=move |v| policy.update(|p| p.government_id_required = Some(v)) />
                <PolicyToggle label="Parties or events allowed" get=Signal::derive(move || policy.get().parties_or_event_allowed.unwrap_or(false))
                    set=move |v| policy.update(|p| p.parties_or_event_allowed = Some(v)) />
            </div>

            <div class="mt-4">
                <label class="mb-1 block text-xs font-medium text-slate-500">"Public note"</label>
                <textarea rows="3" placeholder="Shown to guests on the listing"
                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                    prop:value=move || policy.get().public_note.unwrap_or_default()
                    on:input:target=move |ev| {
                        let v = ev.target().value();
                        policy.update(|p| p.public_note = Some(v));
                    }
                ></textarea>
            </div>

            <div class="mt-4 flex justify-end border-t border-slate-100 pt-4">
                <button on:click=save disabled=move || busy.get()
                    class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                    {move || if busy.get() { "Saving…" } else { "Save policy" }}
                </button>
            </div>
        </Card>
    }
}

#[component]
fn EditRoomModal(
    room: api::RoomDetail,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let id = room.id;
    let number = RwSignal::new(room.room_number.clone());
    let name = RwSignal::new(room.name.clone());
    let room_type = RwSignal::new(room.room_type.clone().unwrap_or_else(|| api::ROOM_TYPES[0].into()));
    let floor = RwSignal::new(room.floor.unwrap_or(1).to_string());
    let capacity = RwSignal::new(room.guest_capacity.to_string());
    let price = RwSignal::new(room.price_per_night.clone().unwrap_or_default());
    let discount = RwSignal::new(room.discount_percent_per_night.unwrap_or(0).to_string());
    let breakfast = RwSignal::new(room.breakfast_included);
    let status = RwSignal::new(room.status_str().to_string());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if number.get().trim().is_empty() {
            error.set(Some("Enter a room number.".into()));
            return;
        }
        if price.get().trim().parse::<f64>().is_err() {
            error.set(Some("Enter a nightly price.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let body = serde_json::json!({
            "room_number": number.get().trim(),
            "name": name.get().trim(),
            "room_type": room_type.get(),
            "floor": floor.get().trim().parse::<i32>().unwrap_or(1),
            "guest_capacity": capacity.get().trim().parse::<u32>().unwrap_or(1),
            "price_per_night": price.get().trim(),
            "discount_percent_per_night": discount.get().trim().parse::<i32>().unwrap_or(0),
            "breakfast_included": breakfast.get(),
            "status": status.get(),
        });
        wasm_bindgen_futures::spawn_local(async move {
            match api::patch_room(id, body).await {
                Ok(_) => {
                    busy.set(false);
                    on_saved();
                    on_close();
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <Modal title="Edit room" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="grid grid-cols-2 gap-4">
                    <TextField label="Room number" value=number />
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| room_type.set(ev.target().value())
                        >
                            {api::ROOM_TYPES.iter().map(|t| view! {
                                <option value=*t selected=(*t == room_type.get())>{*t}</option>
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                <TextField label="Display name" value=name />

                <div class="grid grid-cols-3 gap-4">
                    <TextField label="Floor" value=floor kind="number" />
                    <TextField label="Guest capacity" value=capacity kind="number" />
                    <TextField label="Price / night" value=price kind="number" />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <TextField label="Discount %" value=discount kind="number" />
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Status"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| status.set(ev.target().value())
                        >
                            {api::ROOM_STATUSES.iter().map(|s| view! {
                                <option value=*s selected=(*s == status.get())>{*s}</option>
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                <div class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
                    <span class="flex items-center gap-2 text-sm text-slate-700">
                        <Icon name="coffee" class="h-4 w-4 text-slate-400" />
                        "Breakfast included"
                    </span>
                    <button type="button" on:click=move |_| breakfast.update(|v| *v = !*v)>
                        <Toggle checked=breakfast.get() />
                    </button>
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Saving…" } else { "Save changes" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn BlockRoomModal(
    room_id: i64,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_done: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
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
        error.set(None);
        busy.set(true);
        let (ty, s, e, n) = (event_type.get(), start.get(), end.get(), note.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_special_event(room_id, &ty, &s, &e, &n).await {
                Ok(_) => {
                    busy.set(false);
                    on_done();
                    on_close();
                }
                Err(err) => {
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
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Reason"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| event_type.set(ev.target().value())
                    >
                        {api::EVENT_TYPES.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                    </select>
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

                <TextField label="Note" value=note />

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

// ---------------------------------------------------------------------------
// Small shared pieces
// ---------------------------------------------------------------------------

/// The API returns `"14:00:00"`; `<input type="time">` wants `"14:00"`.
fn trim_seconds(raw: Option<String>) -> String {
    raw.map(|s| s.chars().take(5).collect()).unwrap_or_default()
}

#[component]
fn Metric(label: &'static str, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <p class="text-2xs uppercase tracking-wide text-slate-400">{label}</p>
            <p class="mt-0.5 truncate text-lg font-bold text-slate-900">{value}</p>
        </div>
    }
}

#[component]
fn Row(label: &'static str, #[prop(into)] value: String) -> impl IntoView {
    let value = if value.trim().is_empty() {
        "—".to_string()
    } else {
        value
    };
    view! {
        <div class="flex items-start justify-between gap-3">
            <dt class="shrink-0 text-slate-500">{label}</dt>
            <dd class="min-w-0 break-words text-right font-medium text-slate-800">{value}</dd>
        </div>
    }
}

#[component]
fn TextField(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-xs font-medium text-slate-500">{label}</label>
            <input type=kind class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                prop:value=value on:input:target=move |ev| value.set(ev.target().value()) />
        </div>
    }
}

#[component]
fn PolicyToggle(
    label: &'static str,
    get: Signal<bool>,
    set: impl Fn(bool) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <label class="flex cursor-pointer items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2">
            <span class="text-sm text-slate-700">{label}</span>
            <input type="checkbox" class="h-4 w-4" prop:checked=move || get.get()
                on:change:target=move |ev| set(ev.target().checked()) />
        </label>
    }
}
