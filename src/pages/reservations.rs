//! Reservations, backed by `/reservations/`.
//!
//! Bookings arrive from the guest portal, the phone and the front desk. The tabs
//! map to the API's `status_tab` filter so paging stays server-side; the drawer
//! opens the detail endpoint (which adds the timeline and settlement) and drives
//! the whole lifecycle: confirm, reject, check-in, checkout, cancel and notes.
//!
//! There is no reservations-metrics endpoint, so the tiles are counted from an
//! unfiltered fetch that is kept separate from the tab query.

use crate::api;
use crate::components::{use_toast, Avatar, Badge, Icon, Modal, StatCard};
use crate::pages::dashboard::{status_pill, status_tone};
use leptos::prelude::*;

const TABS: &[&str] = &["All", "New", "Confirmed", "In-house", "Completed", "Cancelled"];

/// Which action the drawer is currently collecting input for.
#[derive(Clone, Copy, PartialEq)]
enum Action {
    None,
    Confirm,
    Reject,
    Cancel,
    CheckIn,
    CheckOut,
    Notes,
}

#[component]
pub fn ReservationsPage() -> impl IntoView {
    let toast = use_toast();
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    let open_id = RwSignal::new(Option::<i64>::None);
    let refresh = RwSignal::new(0u32);
    let new_open = RwSignal::new(false);

    let reservations = LocalResource::new(move || {
        let tab = filter.get();
        let q = search.get();
        let _ = refresh.get();
        async move {
            api::list_reservations(&api::ReservationQuery {
                status_tab: api::reservation_tab_param(tab),
                search: q,
                page_size: 100,
                ordering: "-created_at".into(),
                ..Default::default()
            })
            .await
        }
    });

    // Tile counts come from an unfiltered fetch so they stay stable while the
    // operator switches tabs.
    let all = LocalResource::new(move || {
        let _ = refresh.get();
        async move {
            api::list_reservations(&api::ReservationQuery {
                page_size: 100,
                ..Default::default()
            })
            .await
        }
    });
    let count = move |statuses: &'static [&'static str]| {
        all.get()
            .and_then(Result::ok)
            .map(|p| {
                p.items
                    .iter()
                    .filter(|r| statuses.contains(&r.status_str()))
                    .count()
                    .to_string()
            })
            .unwrap_or_else(|| "—".into())
    };
    let total = move || {
        all.get()
            .and_then(Result::ok)
            .map(|p| p.meta.count.to_string())
            .unwrap_or_else(|| "—".into())
    };

    let bump = move || refresh.update(|n| *n += 1);

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Reservations"</h1>
                    <p class="text-sm text-slate-500">"Bookings from the portal, phone and front desk."</p>
                </div>
                <button
                    on:click=move |_| new_open.set(true)
                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                >
                    <Icon name="plus" class="h-4 w-4" />
                    "New Booking"
                </button>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
                <StatCard icon="calendar-check" label="Total" value=Signal::derive(total) hint="All reservations" />
                <StatCard icon="clock" label="Awaiting action" value=Signal::derive(move || count(&["New", "Pending"])) hint="Needs confirming" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="check-circle" label="Confirmed" value=Signal::derive(move || count(&["Confirmed"])) hint="Ready to arrive" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="users" label="In house" value=Signal::derive(move || count(&["In-house"])) hint="Currently staying" accent="text-blue-600 bg-blue-50" />
                <StatCard icon="x-circle" label="Cancelled" value=Signal::derive(move || count(&["Cancelled", "Rejected", "No show"])) hint="Not proceeding" accent="text-red-600 bg-red-50" />
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
                    placeholder="Search reference, guest name, email, phone or room number"
                    class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                    prop:value=search
                    on:input:target=move |ev| search.set(ev.target().value())
                />
            </div>

            <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                <table class="w-full text-left text-sm">
                    <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                        <tr>
                            <th class="px-4 py-3">"Reference"</th>
                            <th class="px-4 py-3">"Guest"</th>
                            <th class="px-4 py-3">"Room"</th>
                            <th class="px-4 py-3">"Stay"</th>
                            <th class="px-4 py-3">"Nights"</th>
                            <th class="px-4 py-3">"Amount"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3"></th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=|| view! {
                            <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-400">"Loading reservations…"</td></tr>
                        }>
                            {move || Suspend::new(async move {
                                match reservations.await {
                                    Err(e) => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-red-600">{e.detail()}</td></tr>
                                    }.into_any(),
                                    Ok(page) if page.items.is_empty() => view! {
                                        <tr><td colspan="8" class="px-4 py-10 text-center text-sm text-slate-500">"No reservations match this view."</td></tr>
                                    }.into_any(),
                                    Ok(page) => page.items.into_iter().map(|r| {
                                        let id = r.id;
                                        view! {
                                            <tr
                                                class="cursor-pointer border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50"
                                                on:click=move |_| open_id.set(Some(id))
                                            >
                                                <td class="px-4 py-3 font-semibold text-slate-900">{r.reference()}</td>
                                                <td class="px-4 py-3">
                                                    <div class="flex items-center gap-2.5">
                                                        <Avatar initials=r.guest_initials() size="h-8 w-8 text-2xs" />
                                                        <div class="min-w-0">
                                                            <p class="flex items-center gap-1.5 truncate font-medium text-slate-900">
                                                                {r.guest_name()}
                                                                {r.guest.is_vip.then(|| view! {
                                                                    <span class="rounded bg-purple-100 px-1 py-0.5 text-[9px] font-bold text-purple-700">"VIP"</span>
                                                                })}
                                                            </p>
                                                            <p class="truncate text-2xs text-slate-500">{r.guest_phone()}</p>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td class="px-4 py-3">
                                                    <p class="font-medium text-slate-800">{r.room_label()}</p>
                                                    <p class="text-2xs text-slate-500">{r.room_type().to_string()}</p>
                                                </td>
                                                <td class="px-4 py-3 whitespace-nowrap text-slate-600">{r.stay_dates()}</td>
                                                <td class="px-4 py-3 tabular-nums">{r.number_of_nights}</td>
                                                <td class="px-4 py-3 tabular-nums font-semibold text-slate-800">
                                                    {format!("ETB {}", api::money_round(Some(&format!("{:.2}", r.amount()))))}
                                                </td>
                                                <td class="px-4 py-3">
                                                    <span class=format!(
                                                        "rounded-full px-2 py-0.5 text-xs font-semibold {}",
                                                        status_pill(r.status_str())
                                                    )>
                                                        {r.status_str().to_string()}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <Icon name="chevron-right" class="h-4 w-4 text-slate-300" />
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

            <Show when=move || open_id.get().is_some()>
                <ReservationDrawer
                    id=Signal::derive(move || open_id.get().unwrap_or(0))
                    on_close=move || open_id.set(None)
                    on_changed=move || bump()
                />
            </Show>

            <Show when=move || new_open.get()>
                <NewBookingModal
                    on_close=move || new_open.set(false)
                    on_created=move || {
                        toast.success("Booking created", "The reservation is now in the New tab.");
                        bump();
                    }
                />
            </Show>
        </div>
    }
}

/// Detail drawer: timeline, settlement and every lifecycle action.
#[component]
fn ReservationDrawer(
    id: Signal<i64>,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_changed: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let reload = RwSignal::new(0u32);
    let action = RwSignal::new(Action::None);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    // Shared inputs for whichever action is open.
    let note = RwSignal::new(String::new());
    let reason = RwSignal::new(String::new());
    let passport = RwSignal::new(String::new());
    let org_note = RwSignal::new(String::new());
    // Checkout form.
    let additional = RwSignal::new("0".to_string());
    let additional_note = RwSignal::new(String::new());
    let damage = RwSignal::new("0".to_string());
    let discount = RwSignal::new("0".to_string());
    let tax = RwSignal::new("15".to_string());
    let paid = RwSignal::new(String::new());
    let pay_status = RwSignal::new("Paid".to_string());
    let pay_method = RwSignal::new("Cash".to_string());
    let pay_ref = RwSignal::new(String::new());
    let condition = RwSignal::new("Good".to_string());

    let detail = LocalResource::new(move || {
        let id = id.get();
        let _ = reload.get();
        async move { api::get_reservation(id).await }
    });

    // Seeds the checkout and notes forms from the loaded reservation.
    Effect::new(move |_| {
        if let Some(Ok(r)) = detail.get().map(|d| d.map(|x| x.clone())) {
            org_note.set(r.organization_note.clone().unwrap_or_default());
            if paid.get().is_empty() {
                paid.set(format!("{:.2}", r.amount()));
            }
        }
    });

    let finish = move |label: &'static str| {
        busy.set(false);
        action.set(Action::None);
        error.set(None);
        note.set(String::new());
        reason.set(String::new());
        reload.update(|n| *n += 1);
        on_changed();
        toast.success(label, "The reservation has been updated.");
    };
    let fail = move |e: api::ApiError| {
        busy.set(false);
        error.set(Some(e.detail()));
    };

    let run = move |kind: Action| {
        if busy.get() {
            return;
        }
        let rid = id.get();
        busy.set(true);
        error.set(None);
        let (n, rs, pp) = (note.get(), reason.get(), passport.get());
        let payload = api::CheckoutPayload {
            damage_charge: num_or_zero(&damage.get()),
            additional_charge: num_or_zero(&additional.get()),
            additional_charge_note: additional_note.get(),
            discount_amount: num_or_zero(&discount.get()),
            tax_rate_percent: num_or_zero(&tax.get()),
            amount_paid: num_or_zero(&paid.get()),
            payment_status: pay_status.get(),
            payment_method: pay_method.get(),
            payment_reference: pay_ref.get(),
            room_condition: condition.get(),
            completion_note: note.get(),
        };
        let on = org_note.get();

        wasm_bindgen_futures::spawn_local(async move {
            let res = match kind {
                Action::Confirm => api::confirm_reservation(rid, &n).await.map(|_| "Confirmed"),
                Action::Reject => api::reject_reservation(rid, &rs, &n).await.map(|_| "Rejected"),
                Action::Cancel => api::cancel_reservation(rid, &rs).await.map(|_| "Cancelled"),
                Action::CheckIn => api::check_in(rid, &pp, &n).await.map(|_| "Checked in"),
                Action::CheckOut => api::check_out(rid, &payload).await.map(|_| "Checked out"),
                Action::Notes => api::update_reservation_notes(rid, None, Some(&on))
                    .await
                    .map(|_| "Notes saved"),
                Action::None => return,
            };
            match res {
                Ok(label) => finish(label),
                Err(e) => fail(e),
            }
        });
    };

    view! {
        <Modal title="Reservation" on_close=on_close>
            <Suspense fallback=|| view! {
                <p class="py-10 text-center text-sm text-slate-400">"Loading reservation…"</p>
            }>
                {move || Suspend::new(async move {
                    let r = match detail.await {
                        Ok(r) => r,
                        Err(e) => return view! {
                            <p class="py-10 text-center text-sm text-red-600">{e.detail()}</p>
                        }.into_any(),
                    };
                    let status = r.status_str().to_string();
                    let completion = r.completion().cloned();
                    let acts = r.activities.clone();

                    view! {
                        <div class="flex flex-col gap-5">
                            // ---- Header -------------------------------------
                            <div class="flex flex-wrap items-start justify-between gap-3">
                                <div class="flex items-center gap-3">
                                    <Avatar initials=r.guest_initials() size="h-11 w-11 text-sm" />
                                    <div>
                                        <p class="flex items-center gap-2 font-bold text-slate-900">
                                            {r.guest_name()}
                                            {r.guest.is_vip.then(|| view! {
                                                <span class="rounded bg-purple-100 px-1.5 py-0.5 text-2xs font-bold text-purple-700">"VIP"</span>
                                            })}
                                        </p>
                                        <p class="text-xs text-slate-500">{r.reference()}</p>
                                    </div>
                                </div>
                                <Badge label=status.clone() tone=status_tone(&status) />
                            </div>

                            // ---- Facts --------------------------------------
                            <dl class="grid grid-cols-2 gap-x-4 gap-y-3 rounded-xl border border-slate-200 p-4 text-sm sm:grid-cols-3">
                                <Fact label="Room" value=format!("{} · {}", r.room_label(), r.room_type()) />
                                <Fact label="Check-in" value=api::pretty_date(r.check_in_date.as_deref()) />
                                <Fact label="Check-out" value=api::pretty_date(r.check_out_date.as_deref()) />
                                <Fact label="Nights" value=r.number_of_nights.to_string() />
                                <Fact label="Guests" value=r.guest_count.to_string() />
                                <Fact label="Extra beds" value=r.extra_bed.to_string() />
                                <Fact label="Phone" value=if r.guest_phone().is_empty() { "—".into() } else { r.guest_phone() } />
                                <Fact label="Email" value=if r.guest_email().is_empty() { "—".into() } else { r.guest_email() } />
                                <Fact label="Booked" value=api::pretty_datetime(r.created_at.as_deref()) />
                            </dl>

                            {(!r.guest_note.as_deref().unwrap_or("").is_empty()).then(|| view! {
                                <div class="rounded-lg bg-blue-50 px-3 py-2 text-sm text-blue-900">
                                    <p class="mb-0.5 text-2xs font-bold uppercase tracking-wide text-blue-700">"Guest request"</p>
                                    {r.guest_note.clone().unwrap_or_default()}
                                </div>
                            })}

                            {r.cancellation_reason.clone().filter(|s| !s.is_empty()).map(|reason| view! {
                                <div class="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-900">
                                    <p class="mb-0.5 text-2xs font-bold uppercase tracking-wide text-red-700">"Reason"</p>
                                    {reason}
                                </div>
                            })}

                            // ---- Settlement ---------------------------------
                            {completion.map(|c| view! {
                                <div class="rounded-xl border border-emerald-200 bg-emerald-50/50 p-4">
                                    <p class="mb-2.5 text-2xs font-bold uppercase tracking-wide text-emerald-700">"Settlement"</p>
                                    <dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm sm:grid-cols-3">
                                        <Fact label="Room charge" value=format!("ETB {}", api::money(c.room_charge.as_deref())) />
                                        <Fact label="Extras" value=format!("ETB {}", api::money(c.additional_charge.as_deref())) />
                                        <Fact label="Damage" value=format!("ETB {}", api::money(c.damage_charge.as_deref())) />
                                        <Fact label="Tax" value=format!("ETB {}", api::money(c.tax_amount.as_deref())) />
                                        <Fact label="Total" value=format!("ETB {}", api::money(c.total_amount.as_deref())) />
                                        <Fact label="Paid" value=format!("ETB {}", api::money(c.amount_paid.as_deref())) />
                                        <Fact label="Outstanding" value=format!("ETB {}", api::money(c.outstanding_amount.as_deref())) />
                                        <Fact label="Method" value=c.method_str().to_string() />
                                        <Fact label="Reference" value=c.payment_reference.clone().unwrap_or_else(|| "—".into()) />
                                        <Fact label="Room condition" value=c.room_condition.clone().unwrap_or_else(|| "—".into()) />
                                    </dl>
                                    {c.completion_note.clone().filter(|s| !s.is_empty()).map(|n| view! {
                                        <p class="mt-2 text-xs text-emerald-900">{n}</p>
                                    })}
                                </div>
                            })}

                            // ---- Internal note ------------------------------
                            <div>
                                <label class="mb-1 block text-xs font-medium text-slate-500">"Internal note"</label>
                                <textarea
                                    rows="2"
                                    placeholder="Visible to staff only"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                    prop:value=org_note
                                    on:input:target=move |ev| org_note.set(ev.target().value())
                                ></textarea>
                                <button
                                    class="mt-1.5 text-xs font-semibold text-blue-700 hover:text-blue-800"
                                    on:click=move |_| run(Action::Notes)
                                >
                                    "Save note"
                                </button>
                            </div>

                            // ---- Error banner -------------------------------
                            <Show when=move || error.get().is_some()>
                                <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                                    <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </Show>

                            // ---- Action forms -------------------------------
                            <Show when=move || action.get() == Action::CheckIn>
                                <div class="flex flex-col gap-3 rounded-xl border border-blue-200 bg-blue-50/40 p-4">
                                    <p class="text-sm font-bold text-slate-800">"Check in guest"</p>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"ID / passport number"</label>
                                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=passport on:input:target=move |ev| passport.set(ev.target().value()) />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Note"</label>
                                        <input type="text" placeholder="Key card issued, room ready…" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=note on:input:target=move |ev| note.set(ev.target().value()) />
                                    </div>
                                    <ActionButtons busy=busy label="Check in" on_cancel=move || action.set(Action::None) on_submit=move || run(Action::CheckIn) />
                                </div>
                            </Show>

                            <Show when=move || action.get() == Action::CheckOut>
                                <div class="flex flex-col gap-3 rounded-xl border border-emerald-200 bg-emerald-50/40 p-4">
                                    <p class="text-sm font-bold text-slate-800">"Check out & settle"</p>
                                    <div class="grid grid-cols-2 gap-3">
                                        <NumField label="Additional charges" value=additional />
                                        <NumField label="Damage charge" value=damage />
                                        <NumField label="Discount" value=discount />
                                        <NumField label="Tax rate %" value=tax />
                                        <NumField label="Amount paid" value=paid />
                                        <div>
                                            <label class="mb-1 block text-xs font-medium text-slate-500">"Payment status"</label>
                                            <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                                on:change:target=move |ev| pay_status.set(ev.target().value())
                                            >
                                                {api::PAYMENT_STATUSES.iter().map(|s| view! {
                                                    <option value=*s selected=(*s == "Paid")>{*s}</option>
                                                }).collect_view()}
                                            </select>
                                        </div>
                                        <div>
                                            <label class="mb-1 block text-xs font-medium text-slate-500">"Payment method"</label>
                                            <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                                on:change:target=move |ev| pay_method.set(ev.target().value())
                                            >
                                                {api::PAYMENT_METHODS.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                                            </select>
                                        </div>
                                        <div>
                                            <label class="mb-1 block text-xs font-medium text-slate-500">"Room condition"</label>
                                            <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                                on:change:target=move |ev| condition.set(ev.target().value())
                                            >
                                                {api::ROOM_CONDITIONS.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                                            </select>
                                        </div>
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Charge description"</label>
                                        <input type="text" placeholder="Minibar, laundry…" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=additional_note on:input:target=move |ev| additional_note.set(ev.target().value()) />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Payment reference"</label>
                                        <input type="text" placeholder="POS or transfer reference" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=pay_ref on:input:target=move |ev| pay_ref.set(ev.target().value()) />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Checkout note"</label>
                                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=note on:input:target=move |ev| note.set(ev.target().value()) />
                                    </div>
                                    <p class="text-2xs text-slate-500">
                                        "A ‘Paid’ status requires the amount paid to cover the settled total, and damage charges above zero require a condition other than Good."
                                    </p>
                                    <ActionButtons busy=busy label="Complete checkout" on_cancel=move || action.set(Action::None) on_submit=move || run(Action::CheckOut) />
                                </div>
                            </Show>

                            <Show when=move || matches!(action.get(), Action::Confirm | Action::Reject | Action::Cancel)>
                                <div class="flex flex-col gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4">
                                    <p class="text-sm font-bold text-slate-800">
                                        {move || match action.get() {
                                            Action::Confirm => "Confirm reservation",
                                            Action::Reject => "Reject reservation",
                                            _ => "Cancel reservation",
                                        }}
                                    </p>
                                    <Show when=move || action.get() != Action::Confirm>
                                        <div>
                                            <label class="mb-1 block text-xs font-medium text-slate-500">"Reason"</label>
                                            <input type="text" placeholder="Shared with the guest" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                                prop:value=reason on:input:target=move |ev| reason.set(ev.target().value()) />
                                        </div>
                                    </Show>
                                    <Show when=move || action.get() != Action::Cancel>
                                        <div>
                                            <label class="mb-1 block text-xs font-medium text-slate-500">"Internal note"</label>
                                            <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                                prop:value=note on:input:target=move |ev| note.set(ev.target().value()) />
                                        </div>
                                    </Show>
                                    <ActionButtons
                                        busy=busy
                                        label="Apply"
                                        on_cancel=move || action.set(Action::None)
                                        on_submit=move || run(action.get())
                                    />
                                </div>
                            </Show>

                            // ---- Action bar ---------------------------------
                            <Show when=move || action.get() == Action::None>
                                <div class="flex flex-wrap gap-2 border-t border-slate-100 pt-4">
                                    {r.can_confirm().then(|| view! {
                                        <button class="rounded-lg bg-emerald-600 px-3 py-2 text-sm font-semibold text-white hover:bg-emerald-700"
                                            on:click=move |_| action.set(Action::Confirm)>"Confirm"</button>
                                    })}
                                    {r.can_check_in().then(|| view! {
                                        <button class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-800"
                                            on:click=move |_| action.set(Action::CheckIn)>"Check in"</button>
                                    })}
                                    {r.can_check_out().then(|| view! {
                                        <button class="rounded-lg bg-indigo-700 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-800"
                                            on:click=move |_| action.set(Action::CheckOut)>"Check out"</button>
                                    })}
                                    {r.can_reject().then(|| view! {
                                        <button class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
                                            on:click=move |_| action.set(Action::Reject)>"Reject"</button>
                                    })}
                                    {r.can_cancel().then(|| view! {
                                        <button class="rounded-lg border border-red-300 px-3 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
                                            on:click=move |_| action.set(Action::Cancel)>"Cancel booking"</button>
                                    })}
                                </div>
                            </Show>

                            // ---- Timeline -----------------------------------
                            <div class="border-t border-slate-100 pt-4">
                                <p class="mb-3 text-2xs font-bold uppercase tracking-wide text-slate-500">"Timeline"</p>
                                {if acts.is_empty() {
                                    view! { <p class="text-sm text-slate-400">"No activity recorded."</p> }.into_any()
                                } else {
                                    view! {
                                        <ol class="flex flex-col gap-3">
                                            {acts.into_iter().map(|a| view! {
                                                <li class="flex gap-3">
                                                    <span class="mt-1 h-2 w-2 shrink-0 rounded-full bg-blue-500"></span>
                                                    <div class="min-w-0">
                                                        <p class="text-xs font-semibold text-slate-800">
                                                            {format!("{} · {}", a.kind, a.actor_name.clone().unwrap_or_else(|| "System".into()))}
                                                        </p>
                                                        <p class="text-2xs text-slate-500">{a.note.clone().unwrap_or_default()}</p>
                                                        <p class="text-2xs text-slate-400">{api::pretty_datetime(a.created_at.as_deref())}</p>
                                                    </div>
                                                </li>
                                            }).collect_view()}
                                        </ol>
                                    }.into_any()
                                }}
                            </div>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </Modal>
    }
}

/// Front-desk booking form.
#[component]
fn NewBookingModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_created: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let room_id = RwSignal::new(0i64);
    let check_in = RwSignal::new(String::new());
    let check_out = RwSignal::new(String::new());
    let guests = RwSignal::new("2".to_string());
    let extra_bed = RwSignal::new("0".to_string());
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let rooms = LocalResource::new(|| async move { api::all_rooms().await });

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        let Ok(org) = api::require_org() else {
            error.set(Some("Create your hotel under Hotel Profile first.".into()));
            return;
        };
        if room_id.get() == 0 {
            error.set(Some("Pick a room.".into()));
            return;
        }
        if check_in.get().is_empty() || check_out.get().is_empty() {
            error.set(Some("Enter both stay dates.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let payload = api::NewReservation {
            organization_id: org,
            room_id: room_id.get(),
            check_in_date: check_in.get(),
            check_out_date: check_out.get(),
            guest_count: guests.get().parse().unwrap_or(1),
            extra_bed: extra_bed.get().parse().unwrap_or(0),
            pet_presence: false,
            baby_presence: false,
            guest_first_name: first.get(),
            guest_last_name: last.get(),
            guest_email: email.get(),
            guest_phone: phone.get(),
            guest_note: note.get(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::create_reservation(&payload).await {
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
        <Modal title="New Booking" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Room"</label>
                    <Suspense fallback=|| view! { <p class="text-sm text-slate-400">"Loading rooms…"</p> }>
                        {move || Suspend::new(async move {
                            let list = rooms.await.unwrap_or_default();
                            if list.is_empty() {
                                return view! { <p class="text-sm text-slate-400">"No rooms in inventory yet."</p> }.into_any();
                            }
                            view! {
                                <select
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                    on:change:target=move |ev| room_id.set(ev.target().value().parse().unwrap_or(0))
                                >
                                    <option value="0">"Select a room…"</option>
                                    {list.into_iter().map(|r| view! {
                                        <option value=r.id.to_string()>
                                            {format!("{} · {} · ETB {}", r.room_number, r.type_str(), r.price_display())}
                                        </option>
                                    }).collect_view()}
                                </select>
                            }.into_any()
                        })}
                    </Suspense>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-in"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=check_in on:input:target=move |ev| check_in.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-out"</label>
                        <input type="date" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=check_out on:input:target=move |ev| check_out.set(ev.target().value()) />
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guests"</label>
                        <input type="number" min="1" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=guests on:input:target=move |ev| guests.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Extra beds"</label>
                        <input type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=extra_bed on:input:target=move |ev| extra_bed.set(ev.target().value()) />
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest first name"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=first on:input:target=move |ev| first.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest last name"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=last on:input:target=move |ev| last.set(ev.target().value()) />
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest email"</label>
                        <input type="email" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=email on:input:target=move |ev| email.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Guest phone"</label>
                        <input type="tel" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Notes"</label>
                    <textarea rows="2" placeholder="Arrival time, special requests…"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=note on:input:target=move |ev| note.set(ev.target().value())
                    ></textarea>
                </div>

                <p class="text-2xs text-slate-500">
                    "Dashboard bookings are filed against the signed-in account, so the guest details above are stored on the reservation note rather than creating a separate guest login."
                </p>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] disabled:opacity-60">
                        {move || if busy.get() { "Creating…" } else { "Create Booking" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn Fact(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <dt class="text-2xs uppercase tracking-wide text-slate-400">{label}</dt>
            <dd class="mt-0.5 font-medium text-slate-800">{value}</dd>
        </div>
    }
}

#[component]
fn NumField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-xs font-medium text-slate-500">{label}</label>
            <input type="number" step="0.01" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                prop:value=value on:input:target=move |ev| value.set(ev.target().value()) />
        </div>
    }
}

#[component]
fn ActionButtons(
    busy: RwSignal<bool>,
    label: &'static str,
    on_cancel: impl Fn() + Copy + Send + Sync + 'static,
    on_submit: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex justify-end gap-2">
            <button type="button" class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium"
                on:click=move |_| on_cancel()>"Back"</button>
            <button type="button" disabled=move || busy.get()
                class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                on:click=move |_| on_submit()>
                {move || if busy.get() { "Working…".to_string() } else { label.to_string() }}
            </button>
        </div>
    }
}

/// The checkout endpoint wants decimal strings; a blank box means zero.
fn num_or_zero(raw: &str) -> String {
    match raw.trim().parse::<f64>() {
        Ok(n) => format!("{n:.2}"),
        Err(_) => "0.00".into(),
    }
}
