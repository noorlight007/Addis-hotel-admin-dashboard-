//! One guest, backed by `GET /guests/{id}/`.
//!
//! The route is keyed on the booking reference the tables link with, so the id
//! is resolved through a search first. `/guests/id-7` is also accepted for the
//! walk-in records that have no booking yet.
//!
//! Editing splits across three endpoints: the profile fields go to
//! `PATCH /guests/{id}/`, the notes to `/notes/` and the star to `/toggle-vip/`.

use crate::api;
use crate::components::{use_toast, Avatar, Card, Icon, Modal};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

#[component]
pub fn GuestDetailsPage() -> impl IntoView {
    let toast = use_toast();
    let params = use_params_map();
    let refresh = RwSignal::new(0u32);
    let edit_open = RwSignal::new(false);

    let detail = LocalResource::new(move || {
        let key = params.get().get("booking_ref").unwrap_or_default();
        let _ = refresh.get();
        async move {
            // The guests table links by booking reference; walk-ins with no
            // booking fall back to an explicit `id-<pk>` form.
            match key.strip_prefix("id-").and_then(|n| n.parse::<i64>().ok()) {
                Some(id) => api::get_guest(id).await,
                None => api::get_guest_by_booking_ref(&key).await,
            }
        }
    });

    let bump = move || refresh.update(|n| *n += 1);

    view! {
        <div class="p-4 sm:p-6">
            <Suspense fallback=|| view! {
                <p class="py-16 text-center text-sm text-slate-400">"Loading guest…"</p>
            }>
                {move || Suspend::new(async move {
                    let d = match detail.await {
                        Ok(d) => d,
                        Err(e) => return view! {
                            <div class="py-16 text-center">
                                <p class="text-sm text-red-600">{e.detail()}</p>
                                <A href="/guests" attr:class="mt-3 inline-block text-sm text-blue-700 hover:underline">"Back to Guests"</A>
                            </div>
                        }.into_any(),
                    };
                    let g = d.guest.clone();
                    let m = d.metrics.clone();
                    let gid = g.id;
                    let vip = g.is_vip;
                    let active = d.active_booking.clone();
                    let past = d.past_bookings.clone();
                    let profile_for_edit = g.clone();
                    let profile_for_notes = g.clone();

                    view! {
                        <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
                            <A href="/guests" attr:class="inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                                <Icon name="chevron-left" class="h-4 w-4" />
                                "Back to Guests"
                            </A>
                            <div class="flex gap-2">
                                <button
                                    class=format!(
                                        "flex items-center gap-1.5 rounded-lg border px-3 py-2 text-sm font-medium transition-colors {}",
                                        if vip { "border-purple-300 bg-purple-50 text-purple-700 hover:bg-purple-100" } else { "border-slate-300 bg-white text-slate-700 hover:bg-slate-50" }
                                    )
                                    on:click=move |_| {
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match api::toggle_guest_vip(gid, !vip).await {
                                                Ok(()) => { bump(); toast.success("Guest updated", "VIP status changed."); }
                                                Err(e) => toast.error("Could not update guest", e.detail()),
                                            }
                                        });
                                    }
                                >
                                    <Icon name="star" class="h-4 w-4" />
                                    {if vip { "Remove VIP" } else { "Mark as VIP" }}
                                </button>
                                <button
                                    on:click=move |_| edit_open.set(true)
                                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                                >
                                    <Icon name="edit" class="h-4 w-4" />
                                    "Edit Guest"
                                </button>
                            </div>
                        </div>

                        // ---- Identity -------------------------------------
                        <div class="mb-5 flex flex-wrap items-center gap-3">
                            <Avatar initials=g.initials.clone() size="h-12 w-12 text-sm" />
                            <div class="min-w-0">
                                <h1 class="flex flex-wrap items-center gap-2 text-xl font-bold text-slate-900">
                                    {g.display_name.clone()}
                                    {vip.then(|| view! {
                                        <span class="rounded-full bg-purple-100 px-2 py-0.5 text-xs font-bold text-purple-700">"VIP"</span>
                                    })}
                                </h1>
                                <p class="text-sm text-slate-500">
                                    {m.guest_type.clone().unwrap_or_else(|| "Guest".into())}
                                    {g.member_since.as_deref().map(|d| format!(" · with us since {}", api::pretty_date(Some(d)))).unwrap_or_default()}
                                </p>
                            </div>
                            {active.as_ref().and_then(|a| a.stay_status.clone()).map(|s| {
                                let pill = status_pill(&s);
                                view! {
                                    <span class=format!("rounded-full px-2.5 py-1 text-xs font-semibold {pill}")>{s}</span>
                                }
                            })}
                        </div>

                        // ---- Stay metrics ---------------------------------
                        <div class="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4">
                            <Metric icon="users" tint="bg-blue-50 text-blue-600" label="Total stays"
                                value=m.total_stays.to_string() hint="All-time bookings" />
                            <Metric icon="calendar" tint="bg-emerald-50 text-emerald-600" label="Current stay"
                                value=m.current_stay_length.clone().unwrap_or_else(|| "—".into())
                                hint=m.current_stay_dates.clone().unwrap_or_else(|| "Not in house".into()) />
                            <Metric icon="moon" tint="bg-indigo-50 text-indigo-600" label="Total nights"
                                value=m.total_nights.to_string() hint="Across every stay" />
                            <Metric icon="star" tint="bg-purple-50 text-purple-600" label="Guest type"
                                value=m.guest_type.clone().unwrap_or_else(|| "—".into()) hint="Based on stay history" />
                        </div>

                        <div class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
                            // ---- Contact & preferences -------------------
                            <div class="flex flex-col gap-5">
                                <Card title="Contact details">
                                    <dl class="flex flex-col gap-3 text-sm">
                                        <Row label="Email" value=g.display_email.clone() />
                                        <Row label="Phone" value=g.display_phone.clone() />
                                        <Row label="ID / passport" value=g.id_or_passport_number.clone().unwrap_or_default() />
                                        <Row label="Nationality" value=g.nationality.clone().unwrap_or_default() />
                                        <Row label="Language" value=g.preferred_language.clone().unwrap_or_default() />
                                        <Row label="Company" value=g.company.clone().unwrap_or_default() />
                                        <Row label="Address" value=g.address.clone().unwrap_or_default() />
                                        <Row label="Date of birth" value=g.date_of_birth.as_deref().map(|d| api::pretty_date(Some(d))).unwrap_or_default() />
                                    </dl>
                                </Card>

                                <NotesCard guest=profile_for_notes on_saved=move || { bump(); } />
                            </div>

                            // ---- Bookings --------------------------------
                            <div class="flex flex-col gap-5">
                                <Card title="Current booking">
                                    {match active.clone() {
                                        None => view! {
                                            <p class="py-6 text-center text-sm text-slate-400">"This guest is not currently booked in."</p>
                                        }.into_any(),
                                        Some(a) => view! {
                                            <dl class="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
                                                <Row label="Reference" value=a.booking_reference.clone() />
                                                <Row label="Room" value=format!(
                                                    "{} · {}",
                                                    a.room_number.clone().unwrap_or_else(|| "—".into()),
                                                    a.room_type.clone().unwrap_or_else(|| "—".into()),
                                                ) />
                                                <Row label="Stay" value=a.stay_dates.clone().unwrap_or_default() />
                                                <Row label="Nights" value=a.nights.to_string() />
                                                <Row label="Guests" value=a.number_of_guests.to_string() />
                                                <Row label="Booking status" value=a.booking_status.clone().unwrap_or_default() />
                                                <Row label="Payment" value=a.payment_method.clone().unwrap_or_default() />
                                                <Row label="Amount" value=a.total_amount.as_deref().map(|t| format!("ETB {}", api::money(Some(t)))).unwrap_or_else(|| "—".into()) />
                                            </dl>
                                            {a.special_notes.clone().filter(|s| !s.is_empty()).map(|n| view! {
                                                <div class="mt-3 rounded-lg bg-blue-50 px-3 py-2 text-sm text-blue-900">
                                                    <p class="mb-0.5 text-2xs font-bold uppercase tracking-wide text-blue-700">"Guest request"</p>
                                                    {n}
                                                </div>
                                            })}
                                            <A
                                                href="/reservations"
                                                attr:class="mt-3 inline-flex items-center gap-1 text-sm font-semibold text-blue-700 hover:text-blue-800"
                                            >
                                                "Open in reservations"
                                                <Icon name="arrow-right" class="h-3.5 w-3.5" />
                                            </A>
                                        }.into_any(),
                                    }}
                                </Card>

                                <Card title="Stay history" hint=format!("{} recorded", past.len())>
                                    {if past.is_empty() {
                                        view! { <p class="py-6 text-center text-sm text-slate-400">"No past stays."</p> }.into_any()
                                    } else {
                                        view! {
                                            <div class="-mx-5 overflow-x-auto">
                                                <table class="w-full text-left text-sm">
                                                    <thead class="border-b border-slate-100 text-2xs uppercase text-slate-500">
                                                        <tr>
                                                            <th class="px-5 py-2">"Reference"</th>
                                                            <th class="px-5 py-2">"Room"</th>
                                                            <th class="px-5 py-2">"Stay"</th>
                                                            <th class="px-5 py-2">"Nights"</th>
                                                            <th class="px-5 py-2">"Amount"</th>
                                                            <th class="px-5 py-2">"Status"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {past.into_iter().map(|b| view! {
                                                            <tr class="border-b border-slate-50 last:border-0">
                                                                <td class="px-5 py-2.5 font-medium text-slate-800">{b.booking_reference.clone()}</td>
                                                                <td class="px-5 py-2.5">{b.room_display.clone().unwrap_or_else(|| "—".into())}</td>
                                                                <td class="px-5 py-2.5 text-slate-600">{b.stay_dates.clone().unwrap_or_default()}</td>
                                                                <td class="px-5 py-2.5 tabular-nums">{b.nights}</td>
                                                                <td class="px-5 py-2.5 tabular-nums">
                                                                    {b.amount.as_deref().map(|a| format!("ETB {}", api::money(Some(a)))).unwrap_or_else(|| "—".into())}
                                                                </td>
                                                                <td class="px-5 py-2.5">
                                                                    <span class=format!(
                                                                        "rounded-full px-2 py-0.5 text-2xs font-semibold {}",
                                                                        status_pill(b.status.as_deref().unwrap_or(""))
                                                                    )>
                                                                        {b.status.clone().unwrap_or_else(|| "—".into())}
                                                                    </span>
                                                                </td>
                                                            </tr>
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    }}
                                </Card>
                            </div>
                        </div>

                        <Show when=move || edit_open.get()>
                            <EditGuestModal
                                guest=profile_for_edit.clone()
                                on_close=move || edit_open.set(false)
                                on_saved=move || {
                                    toast.success("Guest saved", "The profile has been updated.");
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

/// Internal notes and stay preferences — a separate endpoint from the profile.
#[component]
fn NotesCard(
    guest: api::GuestProfile,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = guest.id;
    let notes = RwSignal::new(guest.internal_notes.clone().unwrap_or_default());
    let prefs = RwSignal::new(guest.special_preferences.clone().unwrap_or_default());
    let busy = RwSignal::new(false);

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let (n, p) = (notes.get(), prefs.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_guest_notes(id, &n, &p).await {
                Ok(()) => {
                    toast.success("Notes saved", "Only staff can see these.");
                    on_saved();
                }
                Err(e) => toast.error("Could not save notes", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <Card title="Notes & preferences" hint="Staff-only">
            <div class="flex flex-col gap-3">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Internal notes"</label>
                    <textarea rows="3" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=notes on:input:target=move |ev| notes.set(ev.target().value())></textarea>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Stay preferences"</label>
                    <textarea rows="3" placeholder="High floor, extra pillows, late breakfast…"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=prefs on:input:target=move |ev| prefs.set(ev.target().value())></textarea>
                </div>
                <button
                    on:click=save
                    disabled=move || busy.get()
                    class="self-start rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                >
                    {move || if busy.get() { "Saving…" } else { "Save notes" }}
                </button>
            </div>
        </Card>
    }
}

#[component]
fn EditGuestModal(
    guest: api::GuestProfile,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let id = guest.id;
    let name = RwSignal::new(guest.full_name.clone());
    let email = RwSignal::new(guest.email.clone().unwrap_or_default());
    let phone = RwSignal::new(guest.phone.clone().unwrap_or_default());
    let passport = RwSignal::new(guest.id_or_passport_number.clone().unwrap_or_default());
    let nationality = RwSignal::new(guest.nationality.clone().unwrap_or_default());
    let language = RwSignal::new(guest.preferred_language.clone().unwrap_or_default());
    let company = RwSignal::new(guest.company.clone().unwrap_or_default());
    let address = RwSignal::new(guest.address.clone().unwrap_or_default());
    let dob = RwSignal::new(guest.date_of_birth.clone().unwrap_or_default());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        busy.set(true);
        error.set(None);
        let payload = api::GuestProfile {
            full_name: name.get().trim().to_string(),
            email: opt(email.get()),
            phone: opt(phone.get()),
            id_or_passport_number: opt(passport.get()),
            nationality: opt(nationality.get()),
            preferred_language: opt(language.get()),
            company: opt(company.get()),
            address: opt(address.get()),
            date_of_birth: opt(dob.get()),
            is_vip: guest.is_vip,
            ..Default::default()
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_guest(id, &payload).await {
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
        <Modal title="Edit Guest" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <Input label="Full name" value=name />
                <div class="grid grid-cols-2 gap-4">
                    <Input label="Email" value=email kind="email" />
                    <Input label="Phone" value=phone kind="tel" />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <Input label="ID / passport" value=passport />
                    <Input label="Nationality" value=nationality />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <Input label="Company" value=company />
                    <Input label="Preferred language" value=language />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <Input label="Address" value=address />
                    <Input label="Date of birth" value=dob kind="date" />
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Saving…" } else { "Save Changes" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn Metric(
    icon: &'static str,
    tint: &'static str,
    label: &'static str,
    #[prop(into)] value: String,
    #[prop(into)] hint: String,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-200 bg-white p-4">
            <div class="mb-2 flex items-center gap-2">
                <span class=format!("flex h-9 w-9 items-center justify-center rounded-full {tint}")>
                    <Icon name=icon class="h-4 w-4" />
                </span>
                <p class="text-xs text-slate-500">{label}</p>
            </div>
            <p class="truncate text-xl font-bold text-slate-900">{value}</p>
            <p class="truncate text-xs text-slate-400">{hint}</p>
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
fn Input(
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

fn opt(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}
