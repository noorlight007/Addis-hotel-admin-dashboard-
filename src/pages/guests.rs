use crate::components::{Icon, Modal, StatCard};
use crate::data::{Guest, GUESTS};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_GUEST_REF: AtomicU32 = AtomicU32::new(251);

fn initials_of(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn GuestsPage() -> impl IntoView {
    let guests = RwSignal::new(GUESTS.to_vec());
    let modal_open = RwSignal::new(false);
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());

    let in_house = move || guests.get().iter().filter(|g| g.status == "In House").count();
    let arriving = move || guests.get().iter().filter(|g| g.status == "Arriving Today").count();
    let checking_out = move || guests.get().iter().filter(|g| g.status == "Checking Out").count();
    let vip = move || guests.get().iter().filter(|g| g.vip).count();

    let visible_guests = move || {
        let q = search.get().to_lowercase();
        guests
            .get()
            .into_iter()
            .filter(|g| match filter.get() {
                "In House" => g.status == "In House",
                "Arriving Today" => g.status == "Arriving Today",
                "Checking Out" => g.status == "Checking Out",
                "VIP" => g.vip,
                _ => true,
            })
            .filter(|g| {
                q.is_empty()
                    || g.name.to_lowercase().contains(&q)
                    || g.booking_ref.to_lowercase().contains(&q)
                    || g.contact.contains(&q)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Guests"</h1>
                    <p class="text-sm text-slate-500">"Manage guest records, stay details, and communication."</p>
                </div>
                <div class="flex gap-2">
                    <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50">
                        <Icon name="download" class="h-4 w-4" />
                        "Export Guests"
                    </button>
                    <button
                        on:click=move |_| modal_open.set(true)
                        class="flex items-center gap-1.5 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-3 py-2 text-sm font-semibold text-white"
                    >
                        <Icon name="plus" class="h-4 w-4" />
                        "Add New Guest"
                    </button>
                </div>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
                <StatCard icon="users" label="Total Guests" value=Signal::derive(move || guests.get().len().to_string()) hint="All guest records" />
                <StatCard icon="bed" label="Currently Staying" value=Signal::derive(move || in_house().to_string()) hint="In-house guests" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="calendar-check" label="Arriving Today" value=Signal::derive(move || arriving().to_string()) hint="Expected check-ins" accent="text-blue-600 bg-blue-50" />
                <StatCard icon="calendar-x" label="Checking Out Today" value=Signal::derive(move || checking_out().to_string()) hint="Expected departures" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="star" label="VIP Guests" value=Signal::derive(move || vip().to_string()) hint="Priority guests" accent="text-purple-600 bg-purple-50" />
            </div>

            <div class="mb-4 flex flex-wrap gap-2 text-sm">
                {["All", "In House", "Arriving Today", "Checking Out", "VIP"].iter().map(|f| {
                    let f = *f;
                    let n = move || match f {
                        "In House" => in_house(),
                        "Arriving Today" => arriving(),
                        "Checking Out" => checking_out(),
                        "VIP" => vip(),
                        _ => guests.get().len(),
                    };
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
                        placeholder="Search by guest name, booking ref, phone or email"
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
                            <th class="px-4 py-3">"Guest"</th>
                            <th class="px-4 py-3">"Booking Ref"</th>
                            <th class="px-4 py-3">"Room"</th>
                            <th class="px-4 py-3">"Stay Dates"</th>
                            <th class="px-4 py-3">"Contact"</th>
                            <th class="px-4 py-3">"Guests"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || visible_guests().into_iter().map(|g| view! {
                            <tr class="border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50">
                                <td class="px-4 py-3">
                                    <div class="flex items-center gap-2">
                                        <span class="flex h-8 w-8 items-center justify-center rounded-full bg-blue-100 text-xs font-semibold text-blue-700">{g.initials}</span>
                                        <div>
                                            <p class="font-medium text-slate-900">{g.name}</p>
                                            <p class="text-xs text-slate-400">{g.nationality}</p>
                                            <Show when=move || g.vip>
                                                <span class="mt-0.5 inline-block rounded-full bg-purple-100 px-1.5 py-0.5 text-[10px] font-semibold text-purple-700">"VIP Guest"</span>
                                            </Show>
                                        </div>
                                    </div>
                                </td>
                                <td class="px-4 py-3 text-blue-700">{g.booking_ref}</td>
                                <td class="px-4 py-3">{g.room}<br/><span class="text-xs text-slate-400">{g.room_type}</span></td>
                                <td class="px-4 py-3">{g.stay_dates}<br/><span class="text-xs text-slate-400">{g.nights}</span></td>
                                <td class="px-4 py-3 text-slate-500">{g.contact}</td>
                                <td class="px-4 py-3">{g.guests}</td>
                                <td class="px-4 py-3"><span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(g.status))>{g.status}</span></td>
                                <td class="px-4 py-3">
                                    <div class="flex items-center gap-2 text-slate-400">
                                        <A href=format!("/guests/{}", g.booking_ref) attr:class="hover:text-slate-700"><Icon name="eye" class="h-4 w-4" /></A>
                                        <button class="hover:text-slate-700"><Icon name="message" class="h-4 w-4" /></button>
                                        <A href=format!("/guests/{}", g.booking_ref) attr:class="hover:text-slate-700"><Icon name="edit" class="h-4 w-4" /></A>
                                        <button class="hover:text-slate-700"><Icon name="more-v" class="h-4 w-4" /></button>
                                    </div>
                                </td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>

                <Show when=move || visible_guests().is_empty()>
                    <div class="p-10 text-center text-sm text-slate-500">"No guests match your search."</div>
                </Show>
            </div>
            <p class="mt-3 text-sm text-slate-500">{move || format!("Showing {} of {} guests", visible_guests().len(), guests.get().len())}</p>

            <div class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-3 text-base font-semibold text-slate-900">"Quick Actions"</h2>
                <div class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
                    <button on:click=move |_| modal_open.set(true) class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 text-left transition-colors hover:bg-slate-50">
                        <Icon name="user-plus" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Add Walk-in Guest"</span><span class="text-xs text-slate-400">"Create a guest record instantly"</span></span>
                    </button>
                    <button class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 text-left hover:bg-slate-50">
                        <Icon name="message" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Send Message"</span><span class="text-xs text-slate-400">"Message a guest directly"</span></span>
                    </button>
                    <A href="/reservations" attr:class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 hover:bg-slate-50">
                        <Icon name="calendar-check" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"View Reservations"</span><span class="text-xs text-slate-400">"Open related bookings"</span></span>
                    </A>
                    <button class="flex items-start gap-2 rounded-lg border border-slate-200 p-3 text-left hover:bg-slate-50">
                        <Icon name="clock" class="mt-0.5 h-4 w-4 text-slate-500" />
                        <span><span class="block font-medium text-slate-800">"Guest History"</span><span class="text-xs text-slate-400">"Review previous stays"</span></span>
                    </button>
                </div>
            </div>

            <Show when=move || modal_open.get()>
                <AddGuestModal
                    on_close=move || modal_open.set(false)
                    on_create=move |guest| guests.update(|g| g.insert(0, guest))
                />
            </Show>
        </div>
    }
}

#[component]
fn AddGuestModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_create: impl Fn(Guest) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let nationality = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let room = RwSignal::new(String::new());
    let room_type = RwSignal::new(String::new());
    let party_size = RwSignal::new("1".to_string());
    let vip = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if name.get().trim().is_empty() || phone.get().trim().is_empty() {
            error.set(Some("Please enter the guest's name and phone number.".to_string()));
            return;
        }
        let guest = Guest {
            initials: Box::leak(initials_of(&name.get()).into_boxed_str()),
            name: Box::leak(name.get().into_boxed_str()),
            nationality: Box::leak(
                (if nationality.get().trim().is_empty() { "Ethiopian".to_string() } else { nationality.get() }).into_boxed_str(),
            ),
            booking_ref: Box::leak(format!("HA-2502{}", NEXT_GUEST_REF.fetch_add(1, Ordering::Relaxed)).into_boxed_str()),
            room: Box::leak((if room.get().trim().is_empty() { "—".to_string() } else { room.get() }).into_boxed_str()),
            room_type: Box::leak((if room_type.get().trim().is_empty() { "Unassigned".to_string() } else { room_type.get() }).into_boxed_str()),
            stay_dates: "Today",
            nights: "New",
            contact: Box::leak(phone.get().into_boxed_str()),
            guests: Box::leak(format!("{} Guest{}", party_size.get(), if party_size.get() == "1" { "" } else { "s" }).into_boxed_str()),
            status: "Arriving Today",
            vip: vip.get(),
            id_passport: "—",
            date_of_birth: "—",
            preferred_language: "English",
            address: "—",
            member_since: "New guest",
            company: "—",
            notes: "—",
            check_in_date: "Today",
            check_out_date: "—",
            payment_method: "—",
            special_notes: "—",
            total_stays: 1,
            total_nights: 0,
        };
        on_create(guest);
        on_close();
    };

    view! {
        <Modal title="Add New Guest" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                    <input type="text" placeholder="e.g. Sara Getachew" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Nationality"</label>
                        <input type="text" placeholder="Ethiopian" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=nationality on:input:target=move |ev| nationality.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone Number"</label>
                        <input type="text" placeholder="+251 91 234 5678" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                </div>

                <div class="grid grid-cols-3 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Number"</label>
                        <input type="text" placeholder="e.g. 301" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=room on:input:target=move |ev| room.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Type"</label>
                        <input type="text" placeholder="e.g. Deluxe Room" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=room_type on:input:target=move |ev| room_type.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guests"</label>
                        <input type="number" min="1" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=party_size on:input:target=move |ev| party_size.set(ev.target().value()) />
                    </div>
                </div>

                <label class="flex items-center gap-2 text-sm text-slate-600">
                    <input type="checkbox" prop:checked=vip on:change:target=move |ev| vip.set(ev.target().checked()) class="h-4 w-4 rounded border-slate-300 text-blue-600" />
                    "Mark as VIP guest"
                </label>

                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Add Guest"</button>
                </div>
            </form>
        </Modal>
    }
}
