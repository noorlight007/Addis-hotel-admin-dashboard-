//! Payments, derived from reservation settlements.
//!
//! The API has no `/payments/` resource. Money is recorded once, on the
//! `completions` entry the checkout endpoint writes onto a reservation, so a
//! transaction here *is* a completed stay's settlement — see
//! [`api::payments_ledger`], which walks the reservation list and reads the
//! settlement off each completed booking.
//!
//! Two tabs:
//!
//! * **Transactions** — every settled stay, with what was charged, collected and
//!   still owed.
//! * **Outstanding** — confirmed and in-house stays with no settlement yet, i.e.
//!   the money still to be taken at checkout. This replaces the payout view: the
//!   platform never holds guest money, so there is nothing to pay out.

use crate::api;
use crate::components::{
    use_toast, Avatar, Badge, Card, EmptyState, Icon, PageHeader, SegmentedControl, StatCard,
};
use leptos::prelude::*;

const TABS: &[&str] = &["Transactions", "Outstanding"];
const FILTERS: &[&str] = &["All", "Paid", "Partially paid", "Unpaid", "Refunded"];

fn tone_for(status: &str) -> &'static str {
    match status {
        "Paid" => "green",
        "Partially paid" => "amber",
        "Refunded" => "purple",
        "Unpaid" => "red",
        _ => "slate",
    }
}

fn method_icon(method: &str) -> &'static str {
    match method {
        "Cash" => "banknote",
        "Mobile payment" => "phone",
        "Bank transfer" => "building",
        _ => "credit-card",
    }
}

#[component]
pub fn PaymentsPage() -> impl IntoView {
    let toast = use_toast();
    let tab = RwSignal::new(TABS[0]);
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    let refresh = RwSignal::new(0u32);

    let ledger = LocalResource::new(move || {
        let q = search.get();
        let _ = refresh.get();
        async move { api::payments_ledger(&q).await }
    });
    let hotel = LocalResource::new(|| async move { api::dashboard_summary(None).await });

    let currency = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .and_then(|s| s.hotel.currency)
            .unwrap_or_else(|| "ETB".into())
    };
    let money = move |n: f64| format!("{} {}", currency(), api::money(Some(&format!("{n:.2}"))));

    let data = move || ledger.get().and_then(Result::ok).unwrap_or_default();

    let filtered = move || {
        let f = filter.get();
        data()
            .transactions
            .into_iter()
            .filter(|t| f == "All" || t.status == f)
            .collect::<Vec<_>>()
    };

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Payments"
                subtitle="Settlements recorded at checkout, and what is still owed."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| {
                        refresh.update(|n| *n += 1);
                        toast.info("Refreshing", "Re-reading settlements from the API.");
                    }
                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                >
                    <Icon name="refresh" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Refresh"</span>
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="banknote"
                        label="Collected"
                        value=Signal::derive(move || money(data().total_collected))
                        hint="Taken at checkout"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="alert"
                        label="Unsettled balance"
                        value=Signal::derive(move || money(data().total_outstanding))
                        hint="Owed on completed stays"
                        accent="text-red-600 bg-red-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="clock"
                        label="Expected"
                        value=Signal::derive(move || money(data().expected_from_pending))
                        hint="From stays yet to check out"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="file-text"
                        label="Transactions"
                        value=Signal::derive(move || data().transactions.len().to_string())
                        hint="Settled stays"
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
            </div>

            // ================= TRANSACTIONS =================
            <Show when=move || (tab.get() == "Transactions")>
                <div class="animate-fade-up">
                    <div class="mb-4 flex flex-wrap items-center gap-3">
                        <div class="flex flex-wrap gap-2 text-sm">
                            {FILTERS.iter().map(|f| {
                                let f = *f;
                                view! {
                                    <button
                                        class=move || format!(
                                            "rounded-lg px-3 py-1.5 font-semibold transition-all duration-200 {}",
                                            if filter.get() == f { "bg-blue-700 text-white shadow-sm" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                                        )
                                        on:click=move |_| filter.set(f)
                                    >
                                        {f}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="relative min-w-[14rem] flex-1">
                            <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                            <input
                                type="text"
                                placeholder="Search reference, guest or room"
                                class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                                prop:value=search
                                on:input:target=move |ev| search.set(ev.target().value())
                            />
                        </div>
                    </div>

                    <Suspense fallback=|| view! {
                        <p class="py-16 text-center text-sm text-slate-400">"Reading settlements…"</p>
                    }>
                        {move || Suspend::new(async move {
                            if let Err(e) = ledger.await {
                                return view! {
                                    <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                                }.into_any();
                            }
                            let rows = filtered();
                            if rows.is_empty() {
                                return view! {
                                    <div class="rounded-xl border border-slate-200 bg-white">
                                        <EmptyState
                                            icon="file-text"
                                            title="No transactions yet"
                                            body="A transaction is written when a stay is checked out and settled. Complete a checkout from Reservations to see it here."
                                        />
                                    </div>
                                }.into_any();
                            }
                            let total = data().transactions.len();
                            let shown = rows.len();

                            view! {
                                <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                                    <table class="w-full text-left text-sm">
                                        <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                            <tr>
                                                <th class="px-4 py-3">"Reference"</th>
                                                <th class="px-4 py-3">"Guest"</th>
                                                <th class="px-4 py-3">"Room"</th>
                                                <th class="px-4 py-3">"Settled"</th>
                                                <th class="px-4 py-3">"Total"</th>
                                                <th class="px-4 py-3">"Paid"</th>
                                                <th class="px-4 py-3">"Due"</th>
                                                <th class="px-4 py-3">"Method"</th>
                                                <th class="px-4 py-3">"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows.into_iter().map(|t| {
                                                let initials = t
                                                    .guest_name
                                                    .split_whitespace()
                                                    .filter_map(|w| w.chars().next())
                                                    .take(2)
                                                    .collect::<String>()
                                                    .to_uppercase();
                                                let due = t.outstanding;
                                                view! {
                                                    <tr class="border-b border-slate-100 last:border-0 hover:bg-slate-50">
                                                        <td class="px-4 py-3 font-semibold text-slate-900">
                                                            {t.reference.clone()}
                                                            <span class="mt-0.5 block text-2xs font-normal text-slate-400">
                                                                {if t.payment_reference.is_empty() { String::new() } else { t.payment_reference.clone() }}
                                                            </span>
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <div class="flex items-center gap-2.5">
                                                                <Avatar initials=initials size="h-8 w-8 text-2xs" />
                                                                <span class="truncate font-medium text-slate-800">{t.guest_name.clone()}</span>
                                                            </div>
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            {t.room_number.clone()}
                                                            <span class="mt-0.5 block text-2xs text-slate-400">
                                                                {format!("{} · {} nights", t.room_type.clone(), t.nights)}
                                                            </span>
                                                        </td>
                                                        <td class="px-4 py-3 whitespace-nowrap text-slate-600">
                                                            {api::pretty_datetime(t.settled_at.as_deref())}
                                                        </td>
                                                        <td class="px-4 py-3 tabular-nums font-semibold text-slate-900">{money(t.total)}</td>
                                                        <td class="px-4 py-3 tabular-nums text-emerald-700">{money(t.paid)}</td>
                                                        <td class=format!(
                                                            "px-4 py-3 tabular-nums {}",
                                                            if due > 0.0 { "font-semibold text-red-700" } else { "text-slate-400" }
                                                        )>
                                                            {money(due)}
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <span class="flex items-center gap-1.5 text-slate-600">
                                                                <Icon name=method_icon(&t.method) class="h-3.5 w-3.5 text-slate-400" />
                                                                {t.method.clone()}
                                                            </span>
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <Badge label=t.status.clone() tone=tone_for(&t.status) />
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                                <p class="mt-3 text-sm text-slate-500">
                                    {format!("Showing {shown} of {total} transactions")}
                                </p>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </Show>

            // ================= OUTSTANDING =================
            <Show when=move || (tab.get() == "Outstanding")>
                <div class="animate-fade-up">
                    <div class="mb-5 grid gap-5 lg:grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)]">
                        <Card
                            title="Awaiting settlement"
                            hint="Stays that have not been checked out yet"
                        >
                            <Suspense fallback=|| view! {
                                <p class="py-8 text-center text-sm text-slate-400">"Loading…"</p>
                            }>
                                {move || Suspend::new(async move {
                                    let l = match ledger.await {
                                        Ok(l) => l,
                                        Err(e) => return view! {
                                            <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                                        }.into_any(),
                                    };
                                    if l.pending.is_empty() {
                                        return view! {
                                            <p class="py-8 text-center text-sm text-slate-400">
                                                "Nothing outstanding — every stay has been settled."
                                            </p>
                                        }.into_any();
                                    }
                                    view! {
                                        <div class="flex flex-col gap-2.5">
                                            {l.pending.into_iter().enumerate().map(|(i, r)| {
                                                let delay = format!("animation-delay: {}ms", i * 60);
                                                let status = r.status_str().to_string();
                                                view! {
                                                    <div
                                                        class="flex animate-fade-up flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:shadow-sm"
                                                        style=delay
                                                    >
                                                        <div class="min-w-0">
                                                            <p class="flex flex-wrap items-center gap-2 font-semibold text-slate-900">
                                                                {r.reference()}
                                                                <Badge label=status.clone() tone=crate::pages::dashboard::status_tone(&status) />
                                                            </p>
                                                            <p class="mt-0.5 truncate text-xs text-slate-500">
                                                                {format!(
                                                                    "{} · Room {} · {}",
                                                                    r.guest_name(),
                                                                    r.room_label(),
                                                                    r.stay_dates(),
                                                                )}
                                                            </p>
                                                        </div>
                                                        <span class="text-right">
                                                            <span class="block text-lg font-bold tabular-nums text-slate-900">
                                                                {money(r.amount())}
                                                            </span>
                                                            <span class="block text-2xs text-slate-400">"Expected at checkout"</span>
                                                        </span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                })}
                            </Suspense>
                        </Card>

                        <div class="flex flex-col gap-5">
                            <Card title="How settlement works">
                                <ol class="flex flex-col gap-3 text-sm text-slate-600">
                                    <Step n="1" text="A booking is confirmed, then the guest is checked in." />
                                    <Step n="2" text="At checkout the front desk records extras, tax, the amount collected and the method." />
                                    <Step n="3" text="That settlement becomes the transaction row on the previous tab." />
                                </ol>
                                <a
                                    href="/reservations"
                                    class="mt-4 flex w-full items-center justify-center gap-2 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                                >
                                    <Icon name="calendar-check" class="h-4 w-4" />
                                    "Go to reservations"
                                </a>
                            </Card>

                            <div class="flex items-start gap-3 rounded-xl bg-blue-50 p-4 text-sm text-blue-900 ring-1 ring-blue-100">
                                <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                                <span>
                                    <span class="font-bold">"Guests pay you directly. "</span>
                                    "The platform never holds guest money and takes no commission, so there are no payouts to reconcile — only what you have collected."
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn Step(n: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <li class="flex gap-3">
            <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-blue-100 text-2xs font-bold text-blue-700">
                {n}
            </span>
            <span>{text}</span>
        </li>
    }
}
