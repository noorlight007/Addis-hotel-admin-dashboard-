use crate::components::{
    use_toast, Avatar, Badge, Card, EmptyState, Icon, PageHeader, SegmentedControl, StatCard,
};
use crate::data::{PAYMENTS, PAYOUTS};
use leptos::prelude::*;

const TABS: &[&str] = &["Transactions", "Payouts"];
const FILTERS: &[&str] = &["All", "Paid", "Pending", "Refunded", "Failed"];

fn thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn tone_for(status: &str) -> &'static str {
    match status {
        "Paid" | "Settled" => "green",
        "Pending" | "Scheduled" => "amber",
        "Refunded" => "purple",
        "Failed" => "red",
        _ => "slate",
    }
}

fn method_icon(method: &str) -> &'static str {
    if method.starts_with("Cash") {
        "banknote"
    } else if method.starts_with("Telebirr") {
        "phone"
    } else {
        "credit-card"
    }
}

#[component]
pub fn PaymentsPage() -> impl IntoView {
    let toast = use_toast();
    let tab = RwSignal::new(TABS[0]);
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());

    let filtered = move || {
        let q = search.get().to_lowercase();
        let f = filter.get();
        PAYMENTS
            .iter()
            .filter(|p| f == "All" || p.status == f)
            .filter(|p| {
                q.is_empty()
                    || p.guest.to_lowercase().contains(&q)
                    || p.invoice.to_lowercase().contains(&q)
                    || p.booking_ref.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    let count = move |status: &'static str| {
        if status == "All" {
            PAYMENTS.len()
        } else {
            PAYMENTS.iter().filter(|p| p.status == status).count()
        }
    };

    let collected: u32 = PAYMENTS.iter().filter(|p| p.status == "Paid").map(|p| p.amount).sum();
    let outstanding: u32 =
        PAYMENTS.iter().filter(|p| p.status == "Pending").map(|p| p.amount).sum();
    let refunded: u32 = PAYMENTS.iter().filter(|p| p.status == "Refunded").map(|p| p.amount).sum();

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Payments"
                subtitle="Transactions taken at the property and weekly payouts."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| toast.success("Export queued", "A CSV of this period will download shortly.")
                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                >
                    <Icon name="download" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Export CSV"</span>
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="check-circle"
                        label="Collected"
                        value=format!("ETB {}", thousands(collected))
                        hint="Settled at the desk"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="clock"
                        label="Outstanding"
                        value=format!("ETB {}", thousands(outstanding))
                        hint="Awaiting check-out"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="refresh"
                        label="Refunded"
                        value=format!("ETB {}", thousands(refunded))
                        hint="This period"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="wallet"
                        label="Next payout"
                        value="ETB 214,800".to_string()
                        hint="Scheduled 24 May"
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
            </div>

            // ================= TRANSACTIONS =================
            <Show when=move || (tab.get() == "Transactions")>
                <div class="animate-fade-up">
                    <div class="mb-4 flex flex-wrap items-center gap-2">
                        {FILTERS.iter().map(|f| {
                            let f = *f;
                            view! {
                                <button
                                    on:click=move |_| filter.set(f)
                                    class=move || format!(
                                        "rounded-lg px-3.5 py-2 text-sm font-semibold transition-all duration-200 active:scale-95 {}",
                                        if filter.get() == f {
                                            "bg-blue-700 text-white shadow-md shadow-blue-700/25"
                                        } else {
                                            "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
                                        }
                                    )
                                >
                                    {format!("{f} ({})", count(f))}
                                </button>
                            }
                        }).collect_view()}

                        <div class="relative ml-auto w-full max-w-xs">
                            <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                            <input
                                type="text"
                                placeholder="Guest, invoice or booking ref"
                                class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                                prop:value=search
                                on:input:target=move |ev| search.set(ev.target().value())
                            />
                        </div>
                    </div>

                    <div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-card">
                        <div class="overflow-x-auto">
                            <table class="w-full min-w-[52rem] text-left text-sm">
                                <thead class="border-b border-slate-200 bg-slate-50/60 text-2xs uppercase tracking-wide text-slate-500">
                                    <tr>
                                        <th class="px-4 py-3">"Guest"</th>
                                        <th class="px-4 py-3">"Invoice"</th>
                                        <th class="px-4 py-3">"Booking"</th>
                                        <th class="px-4 py-3">"Date"</th>
                                        <th class="px-4 py-3">"Method"</th>
                                        <th class="px-4 py-3 text-right">"Amount"</th>
                                        <th class="px-4 py-3">"Status"</th>
                                        <th class="px-4 py-3"></th>
                                    </tr>
                                </thead>
                                <tbody class="stagger">
                                    {move || filtered().into_iter().map(|p| view! {
                                        <tr class="border-b border-slate-100 transition-colors last:border-0 hover:bg-blue-50/40">
                                            <td class="px-4 py-3">
                                                <div class="flex items-center gap-2.5">
                                                    <Avatar initials=p.initials size="h-8 w-8 text-2xs" />
                                                    <div class="min-w-0">
                                                        <p class="truncate font-semibold text-slate-900">{p.guest}</p>
                                                        <p class="text-2xs text-slate-400">{format!("Room {}", p.room)}</p>
                                                    </div>
                                                </div>
                                            </td>
                                            <td class="px-4 py-3 font-mono text-xs text-slate-600">{p.invoice}</td>
                                            <td class="px-4 py-3 font-medium text-blue-700">{p.booking_ref}</td>
                                            <td class="px-4 py-3 text-slate-600">{p.date}</td>
                                            <td class="px-4 py-3">
                                                <span class="flex items-center gap-1.5 text-slate-600">
                                                    <Icon name=method_icon(p.method) class="h-3.5 w-3.5 text-slate-400" />
                                                    {p.method}
                                                </span>
                                            </td>
                                            <td class="px-4 py-3 text-right font-bold tabular-nums text-slate-900">
                                                {format!("ETB {}", thousands(p.amount))}
                                            </td>
                                            <td class="px-4 py-3">
                                                <Badge label=p.status tone=tone_for(p.status) dot=true />
                                            </td>
                                            <td class="px-4 py-3 text-right">
                                                <button
                                                    aria-label="Download invoice"
                                                    on:click=move |_| toast.info("Invoice", format!("{} is being prepared for download.", p.invoice))
                                                    class="rounded-lg p-1.5 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700"
                                                >
                                                    <Icon name="download" class="h-4 w-4" />
                                                </button>
                                            </td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        <Show when=move || filtered().is_empty()>
                            <EmptyState
                                icon="wallet"
                                title="No transactions match"
                                body="Try a different status filter, or clear the search box."
                            />
                        </Show>
                    </div>

                    <p class="mt-3 text-sm text-slate-500">
                        {move || format!("Showing {} of {} transactions", filtered().len(), PAYMENTS.len())}
                    </p>
                </div>
            </Show>

            // ================= PAYOUTS =================
            <Show when=move || (tab.get() == "Payouts")>
                <div class="animate-fade-up">
                    <div class="mb-5 grid gap-5 lg:grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)]">
                        <Card title="Payout history" hint="Settled to your registered bank account">
                            <div class="flex flex-col gap-2.5">
                                {PAYOUTS.iter().enumerate().map(|(i, p)| {
                                    let delay = format!("animation-delay: {}ms", i * 60);
                                    view! {
                                        <div
                                            class="flex animate-fade-up flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-200 hover:shadow-sm"
                                            style=delay
                                        >
                                            <div class="min-w-0">
                                                <p class="flex items-center gap-2 font-semibold text-slate-900">
                                                    {p.period}
                                                    <Badge label=p.status tone=tone_for(p.status) />
                                                </p>
                                                <p class="mt-0.5 text-xs text-slate-500">
                                                    {format!("{} · {} bookings · settles {}", p.reference, p.bookings, p.settled_on)}
                                                </p>
                                            </div>
                                            <span class="text-right">
                                                <span class="block text-lg font-bold tabular-nums text-slate-900">
                                                    {format!("ETB {}", thousands(p.gross))}
                                                </span>
                                                <span class="block text-2xs text-slate-400">"Gross, no commission"</span>
                                            </span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </Card>

                        <div class="flex flex-col gap-5">
                            <Card title="Payout account">
                                <dl class="flex flex-col gap-3 text-sm">
                                    {[
                                        ("Bank", "Commercial Bank of Ethiopia"),
                                        ("Account name", "Golden Tulip Addis Ababa PLC"),
                                        ("Account number", "•••• •••• 4821"),
                                        ("Schedule", "Weekly, every Friday"),
                                    ].into_iter().map(|(label, value)| view! {
                                        <div class="flex justify-between gap-4">
                                            <dt class="text-slate-500">{label}</dt>
                                            <dd class="text-right font-semibold text-slate-800">{value}</dd>
                                        </div>
                                    }).collect_view()}
                                </dl>
                                <button
                                    on:click=move |_| toast.info("Bank details", "Changing payout details requires verification by our finance team.")
                                    class="mt-4 flex w-full items-center justify-center gap-2 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                                >
                                    <Icon name="edit" class="h-4 w-4" />
                                    "Update bank details"
                                </button>
                            </Card>

                            <div class="flex items-start gap-3 rounded-xl bg-blue-50 p-4 text-sm text-blue-900 ring-1 ring-blue-100">
                                <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                                <span>
                                    <span class="font-bold">"Guests pay you directly. "</span>
                                    "Payout rows are for reconciliation only — the portal never holds guest money and takes no commission."
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
