use crate::components::{Icon, Modal};
use crate::data::{find_guest, past_bookings, Guest};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

#[component]
pub fn GuestDetailsPage() -> impl IntoView {
    let params = use_params_map();
    let initial = move || {
        let booking_ref = params.get().get("booking_ref").unwrap_or_default();
        find_guest(&booking_ref).copied()
    };

    view! {
        <Show
            when=move || initial().is_some()
            fallback=|| view! { <div class="p-6 text-center text-slate-500">"Guest not found."</div> }
        >
            {move || {
                let guest = RwSignal::new(initial().unwrap());
                let modal_open = RwSignal::new(false);
                view! {
                    <div class="p-4 sm:p-6">
                        <div class="mb-3 flex items-center justify-between">
                            <A href="/guests" attr:class="inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                                <Icon name="chevron-left" class="h-4 w-4" />
                                "Back to Guests"
                            </A>
                            <button
                                on:click=move |_| modal_open.set(true)
                                class="flex items-center gap-1.5 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-3 py-2 text-sm font-semibold text-white"
                            >
                                <Icon name="edit" class="h-4 w-4" />
                                "Edit Guest"
                            </button>
                        </div>
                        <div class="mb-4 flex items-center gap-3">
                            <h1 class="text-xl font-bold text-slate-900">{move || guest.get().name}</h1>
                            <span class=move || format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(guest.get().status))>{move || guest.get().status}</span>
                        </div>

                        <div class="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4">
                            <div class="rounded-xl border border-slate-200 bg-white p-4">
                                <div class="mb-2 flex items-center gap-2">
                                    <span class="flex h-9 w-9 items-center justify-center rounded-full bg-blue-50 text-blue-600"><Icon name="users" class="h-4 w-4" /></span>
                                    <p class="text-xs text-slate-500">"Total Stays"</p>
                                </div>
                                <p class="text-2xl font-bold text-slate-900">{move || guest.get().total_stays}</p>
                                <p class="text-xs text-slate-400">"All-time stays"</p>
                            </div>
                            <div class="rounded-xl border border-slate-200 bg-white p-4">
                                <div class="mb-2 flex items-center gap-2">
                                    <span class="flex h-9 w-9 items-center justify-center rounded-full bg-emerald-50 text-emerald-600"><Icon name="calendar" class="h-4 w-4" /></span>
                                    <p class="text-xs text-slate-500">"Current Stay Length"</p>
                                </div>
                                <p class="text-2xl font-bold text-slate-900">{move || guest.get().nights}</p>
                                <p class="text-xs text-slate-400">{move || guest.get().stay_dates}</p>
                            </div>
                            <div class="rounded-xl border border-slate-200 bg-white p-4">
                                <div class="mb-2 flex items-center gap-2">
                                    <span class="flex h-9 w-9 items-center justify-center rounded-full bg-purple-50 text-purple-600"><Icon name="moon" class="h-4 w-4" /></span>
                                    <p class="text-xs text-slate-500">"Total Nights"</p>
                                </div>
                                <p class="text-2xl font-bold text-slate-900">{move || guest.get().total_nights}</p>
                                <p class="text-xs text-slate-400">"Across all stays"</p>
                            </div>
                            <div class="rounded-xl border border-slate-200 bg-white p-4">
                                <div class="mb-2 flex items-center gap-2">
                                    <span class="flex h-9 w-9 items-center justify-center rounded-full bg-amber-50 text-amber-600"><Icon name="star" class="h-4 w-4" /></span>
                                    <p class="text-xs text-slate-500">"Guest Type"</p>
                                </div>
                                <p class="text-2xl font-bold text-slate-900">{move || if guest.get().vip { "VIP Guest" } else { "Regular" }}</p>
                                <p class="text-xs text-slate-400">{move || if guest.get().vip { "Priority guest" } else { "Standard guest" }}</p>
                            </div>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <h2 class="mb-3 text-base font-semibold text-slate-900">"Guest Information"</h2>
                            <div class="flex gap-5">
                                <span class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-purple-100 text-lg font-bold text-purple-700">{move || guest.get().initials}</span>
                                <div class="grid flex-1 grid-cols-3 gap-4 text-sm">
                                    <div><p class="text-slate-400">"Full Name"</p><p class="font-medium">{move || guest.get().name}</p></div>
                                    <div><p class="text-slate-400">"ID / Passport No."</p><p class="font-medium">{move || guest.get().id_passport}</p></div>
                                    <div><p class="text-slate-400">"Loyalty / VIP Status"</p><p class="font-medium">
                                        <Show when=move || guest.get().vip fallback=|| view! { <span class="text-slate-500">"Regular Guest"</span> }>
                                            <span class="rounded-full bg-purple-100 px-2 py-0.5 text-xs font-semibold text-purple-700">"VIP Guest"</span>
                                        </Show>
                                    </p></div>
                                    <div><p class="text-slate-400">"Nationality"</p><p class="font-medium">{move || guest.get().nationality}</p></div>
                                    <div><p class="text-slate-400">"Date of Birth"</p><p class="font-medium">{move || guest.get().date_of_birth}</p></div>
                                    <div><p class="text-slate-400">"Member Since"</p><p class="font-medium">{move || guest.get().member_since}</p></div>
                                    <div>
                                        <p class="text-slate-400">"Phone"</p>
                                        <p class="flex items-center gap-1.5 font-medium">
                                            {move || guest.get().contact}
                                            <Icon name="message" class="h-3.5 w-3.5 text-emerald-600" />
                                        </p>
                                    </div>
                                    <div><p class="text-slate-400">"Preferred Language"</p><p class="font-medium">{move || guest.get().preferred_language}</p></div>
                                    <div><p class="text-slate-400">"Company (Optional)"</p><p class="font-medium">{move || guest.get().company}</p></div>
                                    <div><p class="text-slate-400">"Email"</p><p class="font-medium">{move || format!("{}@email.com", guest.get().name.split_whitespace().next().unwrap_or("guest").to_lowercase())}</p></div>
                                    <div><p class="text-slate-400">"Address"</p><p class="font-medium">{move || guest.get().address}</p></div>
                                    <div><p class="text-slate-400">"Guest Notes"</p><p class="font-medium">{move || guest.get().notes}</p></div>
                                </div>
                            </div>
                        </div>

                        <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                            <h2 class="mb-3 flex items-center justify-between text-base font-semibold text-slate-900">
                                <span>"Current Booking / Stay Details"</span>
                                <A href=move || format!("/reservations") attr:class="flex items-center gap-1 text-sm font-medium text-blue-700 hover:underline">
                                    "View Reservation"
                                    <Icon name="external-link" class="h-3.5 w-3.5" />
                                </A>
                            </h2>
                            <div class="grid grid-cols-4 gap-4 text-sm">
                                <div><p class="text-slate-400">"Booking Reference"</p><p class="font-medium text-blue-700">{move || guest.get().booking_ref}</p></div>
                                <div><p class="text-slate-400">"Room Number"</p><p class="font-medium">{move || guest.get().room}</p></div>
                                <div><p class="text-slate-400">"Room Type"</p><p class="font-medium">{move || guest.get().room_type}</p></div>
                                <div><p class="text-slate-400">"Check-in Date"</p><p class="font-medium">{move || guest.get().check_in_date}</p></div>
                                <div><p class="text-slate-400">"Check-out Date"</p><p class="font-medium">{move || guest.get().check_out_date}</p></div>
                                <div><p class="text-slate-400">"Number of Guests"</p><p class="font-medium">{move || guest.get().guests}</p></div>
                                <div><p class="text-slate-400">"Payment Method"</p><p class="font-medium">{move || guest.get().payment_method}</p></div>
                                <div><p class="text-slate-400">"Stay Status"</p><p class="font-medium">{move || guest.get().status}</p></div>
                            </div>
                            <div class="mt-4 border-t border-slate-100 pt-3">
                                <p class="text-slate-400">"Special Notes"</p>
                                <p class="text-sm text-slate-600">{move || guest.get().special_notes}</p>
                            </div>
                        </div>

                        <PastBookingsCard booking_ref=move || guest.get().booking_ref />
                    </div>

                    <Show when=move || modal_open.get()>
                        <EditGuestModal guest=guest on_close=move || modal_open.set(false) />
                    </Show>
                }
            }}
        </Show>
    }
}

#[component]
fn PastBookingsCard(booking_ref: impl Fn() -> &'static str + Copy + Send + Sync + 'static) -> impl IntoView {
    let show_all = RwSignal::new(false);

    view! {
        <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
            <h2 class="mb-3 text-base font-semibold text-slate-900">"Past Bookings"</h2>
            {move || {
                let bookings = past_bookings(booking_ref());
                if bookings.is_empty() {
                    view! { <p class="py-6 text-center text-sm text-slate-400">"No past bookings yet."</p> }.into_any()
                } else {
                    let visible_count = if show_all.get() { bookings.len() } else { bookings.len().min(3) };
                    view! {
                        <table class="w-full text-left text-sm">
                            <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                <tr><th class="py-2">"Booking Ref"</th><th class="py-2">"Room"</th><th class="py-2">"Stay Dates"</th><th class="py-2">"Nights"</th><th class="py-2">"Amount"</th><th class="py-2">"Status"</th><th class="py-2"></th></tr>
                            </thead>
                            <tbody>
                                {bookings.iter().take(visible_count).map(|b| view! {
                                    <tr class="border-b border-slate-100 last:border-0">
                                        <td class="py-2 font-medium text-blue-700">{b.booking_ref}</td>
                                        <td class="py-2">{b.room}<br/><span class="text-xs text-slate-400">{b.room_type}</span></td>
                                        <td class="py-2">{b.stay_dates}</td>
                                        <td class="py-2">{b.nights}</td>
                                        <td class="py-2">{format!("ETB {}", b.amount)}</td>
                                        <td class="py-2"><span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(b.status))>{b.status}</span></td>
                                        <td class="py-2 text-right"><Icon name="chevron-right" class="h-4 w-4 text-slate-300" /></td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }
            }}
            <Show when=move || { past_bookings(booking_ref()).len() > 3 && !show_all.get() }>
                <button on:click=move |_| show_all.set(true) class="mx-auto mt-3 flex items-center gap-1 text-sm font-medium text-blue-700 hover:underline">
                    "View all past bookings"
                    <Icon name="chevron-down" class="h-4 w-4" />
                </button>
            </Show>
        </div>
    }
}

const STATUSES: &[&str] = &["In House", "Arriving Today", "Checking Out", "No-show"];

#[component]
fn EditGuestModal(guest: RwSignal<Guest>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let g0 = guest.get_untracked();
    let name = RwSignal::new(g0.name.to_string());
    let nationality = RwSignal::new(g0.nationality.to_string());
    let phone = RwSignal::new(g0.contact.to_string());
    let room = RwSignal::new(g0.room.to_string());
    let status = RwSignal::new(g0.status.to_string());
    let vip = RwSignal::new(g0.vip);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        guest.update(|g| {
            g.name = Box::leak(name.get().into_boxed_str());
            g.nationality = Box::leak(nationality.get().into_boxed_str());
            g.contact = Box::leak(phone.get().into_boxed_str());
            g.room = Box::leak(room.get().into_boxed_str());
            g.status = Box::leak(status.get().into_boxed_str());
            g.vip = vip.get();
        });
        on_close();
    };

    view! {
        <Modal title="Edit Guest" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Nationality"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=nationality on:input:target=move |ev| nationality.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Room Number"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=room on:input:target=move |ev| room.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Status"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| status.set(ev.target().value())>
                            {STATUSES.iter().map(|s| view! { <option value=*s selected=*s == status.get_untracked()>{*s}</option> }).collect_view()}
                        </select>
                    </div>
                </div>
                <label class="flex items-center gap-2 text-sm text-slate-600">
                    <input type="checkbox" prop:checked=vip on:change:target=move |ev| vip.set(ev.target().checked()) class="h-4 w-4 rounded border-slate-300 text-blue-600" />
                    "VIP guest"
                </label>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}
