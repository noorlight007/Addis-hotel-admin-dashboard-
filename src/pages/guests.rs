//! Guest records, backed by `/guests/` and `/guests/metrics/`.
//!
//! Guests are normally created by bookings, but the front desk can also enter a
//! walk-in directly. The tabs map to the API's `status_tab` filter and search is
//! applied server-side. The VIP star and the CSV export are live calls.

use crate::api;
use crate::components::{use_toast, Icon, Modal, StatCard};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;

const TABS: &[&str] = &["All", "In House", "Arriving Today", "Checking Out", "VIP"];

#[component]
pub fn GuestsPage() -> impl IntoView {
    let toast = use_toast();
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    let refresh = RwSignal::new(0u32);
    let add_open = RwSignal::new(false);
    let exporting = RwSignal::new(false);

    let guests = LocalResource::new(move || {
        let tab = filter.get();
        let q = search.get();
        let _ = refresh.get();
        async move { api::list_guests(tab, &q).await }
    });
    let metrics = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::guest_metrics().await }
    });

    let metric = move |pick: fn(&api::GuestMetrics) -> u32| {
        metrics
            .get()
            .and_then(Result::ok)
            .map(|m| pick(&m).to_string())
            .unwrap_or_else(|| "—".into())
    };

    let export = move |_| {
        if exporting.get() {
            return;
        }
        exporting.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api::export_guests().await {
                Ok(()) => toast.success("Export ready", "guests.csv has been downloaded."),
                Err(e) => toast.error("Export failed", e.detail()),
            }
            exporting.set(false);
        });
    };

    let toggle_vip = move |id: i64, next: bool| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::toggle_guest_vip(id, next).await {
                Ok(()) => {
                    refresh.update(|n| *n += 1);
                    toast.success(
                        if next { "Marked VIP" } else { "VIP removed" },
                        "The guest record has been updated.",
                    );
                }
                Err(e) => toast.error("Could not update guest", e.detail()),
            }
        });
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Guests"</h1>
                    <p class="text-sm text-slate-500">"Guest records, stay details, and preferences."</p>
                </div>
                <div class="flex gap-2">
                    <button
                        on:click=export
                        disabled=move || exporting.get()
                        class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50 disabled:opacity-60"
                    >
                        <Icon name="download" class="h-4 w-4" />
                        {move || if exporting.get() { "Exporting…" } else { "Export Guests" }}
                    </button>
                    <button
                        on:click=move |_| add_open.set(true)
                        class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                    >
                        <Icon name="plus" class="h-4 w-4" />
                        "Add Guest"
                    </button>
                </div>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
                <StatCard icon="users" label="Total Guests" value=Signal::derive(move || metric(|m| m.total_guests)) hint="All guest records" />
                <StatCard icon="bed" label="Currently Staying" value=Signal::derive(move || metric(|m| m.currently_staying)) hint="In-house guests" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="calendar-check" label="Arriving Today" value=Signal::derive(move || metric(|m| m.arriving_today)) hint="Expected check-ins" accent="text-blue-600 bg-blue-50" />
                <StatCard icon="calendar-x" label="Checking Out Today" value=Signal::derive(move || metric(|m| m.checking_out_today)) hint="Expected departures" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="star" label="VIP Guests" value=Signal::derive(move || metric(|m| m.vip_guests)) hint="Priority guests" accent="text-purple-600 bg-purple-50" />
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

            <div class="mb-4 relative">
                <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="text"
                    placeholder="Search by guest name, booking ref, passport, phone or email"
                    class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                    prop:value=search
                    on:input:target=move |ev| search.set(ev.target().value())
                />
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
                        <Suspense fallback=|| view! {
                            <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-400">"Loading guests…"</td></tr>
                        }>
                            {move || Suspend::new(async move {
                                match guests.await {
                                    Err(e) => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-red-600">{e.detail()}</td></tr>
                                    }.into_any(),
                                    Ok(page) if page.items.is_empty() => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-500">"No guests in this view yet. Guests appear here once bookings are made."</td></tr>
                                    }.into_any(),
                                    Ok(page) => page.items.into_iter().map(|g| {
                                        let vip = g.is_vip;
                                        let id = g.id;
                                        let has_booking = !g.booking_ref.is_empty();
                                        let href = if has_booking {
                                            format!("/guests/{}", g.booking_ref)
                                        } else {
                                            format!("/guests/id-{id}")
                                        };
                                        view! {
                                            <tr class="border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50">
                                                <td class="px-4 py-3">
                                                    <div class="flex items-center gap-2">
                                                        <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-100 text-xs font-semibold text-blue-700">{g.initials.clone()}</span>
                                                        <div class="min-w-0">
                                                            <p class="truncate font-medium text-slate-900">{g.display_name.clone()}</p>
                                                            <p class="text-xs text-slate-400">{g.nationality_str().to_string()}</p>
                                                            <Show when=move || vip>
                                                                <span class="mt-0.5 inline-block rounded-full bg-purple-100 px-1.5 py-0.5 text-[10px] font-semibold text-purple-700">"VIP Guest"</span>
                                                            </Show>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td class="px-4 py-3 text-blue-700">
                                                    {if has_booking { g.booking_ref.clone() } else { "—".into() }}
                                                </td>
                                                <td class="px-4 py-3">
                                                    {if g.room_number.is_empty() { "—".to_string() } else { g.room_number.clone() }}
                                                    <br/><span class="text-xs text-slate-400">{g.room_type.clone()}</span>
                                                </td>
                                                <td class="px-4 py-3">
                                                    {if g.stay_dates.is_empty() { "—".to_string() } else { g.stay_dates.clone() }}
                                                    <br/><span class="text-xs text-slate-400">
                                                        {if g.nights > 0 { format!("{} nights", g.nights) } else { String::new() }}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-slate-500">
                                                    {g.display_phone.clone()}
                                                    <br/><span class="text-xs text-slate-400">{g.display_email.clone()}</span>
                                                </td>
                                                <td class="px-4 py-3 tabular-nums">{g.guests_count}</td>
                                                <td class="px-4 py-3">
                                                    <span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(&g.stay_status))>
                                                        {if g.stay_status.is_empty() { "—".to_string() } else { g.stay_status.clone() }}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3">
                                                    <div class="flex items-center gap-2 text-slate-400">
                                                        <A href=href attr:class="hover:text-slate-700" attr:title="Open profile">
                                                            <Icon name="eye" class="h-4 w-4" />
                                                        </A>
                                                        <button
                                                            title=if vip { "Remove VIP" } else { "Mark as VIP" }
                                                            class=if vip { "text-purple-600 hover:text-purple-800" } else { "hover:text-purple-600" }
                                                            on:click=move |_| toggle_vip(id, !vip)
                                                        >
                                                            <Icon name="star" class="h-4 w-4" />
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

            <Show when=move || add_open.get()>
                <AddGuestModal
                    on_close=move || add_open.set(false)
                    on_created=move || {
                        toast.success("Guest added", "The record is now in your guest list.");
                        refresh.update(|n| *n += 1);
                    }
                />
            </Show>
        </div>
    }
}

/// Walk-in / phone guest entry. `POST /guests/` requires nothing, but a name is
/// the least that makes the record useful.
#[component]
fn AddGuestModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_created: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let passport = RwSignal::new(String::new());
    let nationality = RwSignal::new(String::new());
    let company = RwSignal::new(String::new());
    let language = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let vip = RwSignal::new(false);
    let notes = RwSignal::new(String::new());
    let prefs = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if name.get().trim().is_empty() {
            error.set(Some("Please enter the guest's name.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let payload = api::GuestProfile {
            full_name: name.get().trim().to_string(),
            email: non_empty(email.get()),
            phone: non_empty(phone.get()),
            id_or_passport_number: non_empty(passport.get()),
            nationality: non_empty(nationality.get()),
            preferred_language: non_empty(language.get()),
            address: non_empty(address.get()),
            company: non_empty(company.get()),
            internal_notes: non_empty(notes.get()),
            special_preferences: non_empty(prefs.get()),
            is_vip: vip.get(),
            ..Default::default()
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::create_guest(&payload).await {
                Ok(_) => {
                    busy.set(false);
                    on_created();
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
        <Modal title="Add Guest" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <Field label="Full name" value=name placeholder="e.g. Marta Bekele" />

                <div class="grid grid-cols-2 gap-4">
                    <Field label="Email" value=email kind="email" />
                    <Field label="Phone" value=phone kind="tel" />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <Field label="ID / passport" value=passport />
                    <Field label="Nationality" value=nationality placeholder="e.g. Ethiopian" />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <Field label="Company" value=company />
                    <Field label="Preferred language" value=language placeholder="e.g. English" />
                </div>
                <Field label="Address" value=address />

                <div class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
                    <span class="flex items-center gap-2 text-sm text-slate-700">
                        <Icon name="star" class="h-4 w-4 text-slate-400" />
                        "VIP guest"
                    </span>
                    <input type="checkbox" class="h-4 w-4" prop:checked=vip
                        on:change:target=move |ev| vip.set(ev.target().checked()) />
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Internal notes"</label>
                    <textarea rows="2" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=notes on:input:target=move |ev| notes.set(ev.target().value())></textarea>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Stay preferences"</label>
                    <textarea rows="2" placeholder="High floor, quiet room, late breakfast…"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=prefs on:input:target=move |ev| prefs.set(ev.target().value())></textarea>
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] disabled:opacity-60">
                        {move || if busy.get() { "Saving…" } else { "Add Guest" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn Field(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = "")] placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-xs font-medium text-slate-500">{label}</label>
            <input type=kind placeholder=placeholder class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                prop:value=value on:input:target=move |ev| value.set(ev.target().value()) />
        </div>
    }
}

/// Blank inputs are dropped rather than sent as `""`, which the API would store.
fn non_empty(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}
