//! Application top bar: page title, global search, notifications and profile.

use crate::components::{use_layout, Icon};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_location, use_navigate};

struct Notification {
    icon: &'static str,
    tint: &'static str,
    title: &'static str,
    body: &'static str,
    time: &'static str,
    unread: bool,
}

const NOTIFICATIONS: &[Notification] = &[
    Notification { icon: "calendar-check", tint: "bg-blue-50 text-blue-600", title: "New reservation", body: "Fatima Ali booked a Deluxe Room for 16–17 May.", time: "4 min ago", unread: true },
    Notification { icon: "star", tint: "bg-amber-50 text-amber-600", title: "New 5-star review", body: "Mohamed Nur reviewed the Executive Suite.", time: "1 hour ago", unread: true },
    Notification { icon: "mail", tint: "bg-purple-50 text-purple-600", title: "Guest message", body: "Ahmed Hassan asked about a late checkout.", time: "2 hours ago", unread: true },
    Notification { icon: "wrench", tint: "bg-red-50 text-red-600", title: "Room out of service", body: "Room 301 flagged for maintenance by housekeeping.", time: "Yesterday", unread: false },
    Notification { icon: "wallet", tint: "bg-emerald-50 text-emerald-600", title: "Payout processed", body: "ETB 184,300 settled for the week ending 10 May.", time: "2 days ago", unread: false },
];

/// Quick-jump targets for the global search box.
const SEARCH_TARGETS: &[(&str, &str, &str)] = &[
    ("Dashboard", "/", "home"),
    ("Analytics", "/analytics", "bar-chart"),
    ("Reservations", "/reservations", "calendar-check"),
    ("Calendar", "/calendar", "calendar"),
    ("Rooms", "/rooms", "bed"),
    ("Rates & availability", "/rates", "tag"),
    ("Guests", "/guests", "users"),
    ("Payments", "/payments", "wallet"),
    ("Messages", "/messages", "mail"),
    ("Reviews", "/reviews", "star"),
    ("Hotel profile", "/profile", "building"),
    ("Staff & roles", "/staff", "user-plus"),
    ("Settings", "/settings", "settings"),
    ("Bulk upload rooms", "/rooms/bulk-upload", "upload"),
];

/// Human-readable title for the current route.
fn title_for(path: &str) -> &'static str {
    match path {
        "/" => "Dashboard",
        "/analytics" => "Analytics",
        "/reservations" => "Reservations",
        "/calendar" => "Calendar",
        "/rooms" => "Rooms",
        "/rooms/bulk-upload" => "Bulk Upload",
        "/rates" => "Rates & Availability",
        "/guests" => "Guests",
        "/payments" => "Payments",
        "/messages" => "Messages",
        "/reviews" => "Reviews",
        "/profile" => "Hotel Profile",
        "/staff" => "Staff & Roles",
        "/settings" => "Settings",
        p if p.starts_with("/rooms/") => "Room Details",
        p if p.starts_with("/guests/") => "Guest Details",
        _ => "Tourista Admin",
    }
}

#[component]
pub fn Topbar() -> impl IntoView {
    let layout = use_layout();
    let location = use_location();
    let navigate = use_navigate();

    let search = RwSignal::new(String::new());
    let search_open = RwSignal::new(false);
    let notifications_open = RwSignal::new(false);
    let profile_open = RwSignal::new(false);
    let dismissed = RwSignal::new(Vec::<usize>::new());

    let unread = move || {
        NOTIFICATIONS
            .iter()
            .enumerate()
            .filter(|(i, n)| n.unread && !dismissed.get().contains(i))
            .count()
    };

    let matches = move || {
        let q = search.get().to_lowercase();
        SEARCH_TARGETS
            .iter()
            .filter(|(label, _, _)| q.is_empty() || label.to_lowercase().contains(&q))
            .take(6)
            .collect::<Vec<_>>()
    };

    let go = {
        let navigate = navigate.clone();
        move |href: &'static str| {
            search.set(String::new());
            search_open.set(false);
            navigate(href, Default::default());
        }
    };
    let go = StoredValue::new(go);

    // Close every popover when the route changes.
    Effect::new(move |_| {
        let _ = location.pathname.get();
        search_open.set(false);
        notifications_open.set(false);
        profile_open.set(false);
    });

    view! {
        <header class="sticky top-0 z-30 flex h-14 shrink-0 items-center gap-3 border-b border-slate-200 bg-white/90 px-3 backdrop-blur-lg sm:px-5">
            <button
                aria-label="Open menu"
                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-slate-600 transition-colors hover:bg-slate-100 lg:hidden"
                on:click=move |_| layout.mobile_open.set(true)
            >
                <Icon name="menu" class="h-5 w-5" />
            </button>

            <div class="min-w-0 shrink-0">
                <h1 class="truncate text-base font-bold text-slate-900">
                    {move || title_for(&location.pathname.get())}
                </h1>
            </div>

            // ---- Global search ------------------------------------------
            <div class="relative ml-auto w-full max-w-xs sm:ml-4 sm:mr-auto">
                <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="text"
                    placeholder="Jump to…"
                    class="w-full rounded-lg border border-slate-200 bg-slate-50 py-2 pl-9 pr-3 text-sm transition-all duration-200 focus:border-blue-400 focus:bg-white focus:outline-none focus:ring-4 focus:ring-blue-100"
                    prop:value=search
                    on:focus=move |_| search_open.set(true)
                    on:input:target=move |ev| { search.set(ev.target().value()); search_open.set(true); }
                />
                <Show when=move || (search_open.get() && !matches().is_empty())>
                    <div class="absolute left-0 right-0 top-full z-40 mt-1.5 animate-fade-down overflow-hidden rounded-xl border border-slate-200 bg-white p-1.5 shadow-2xl shadow-slate-900/10">
                        <p class="px-2.5 py-1.5 text-2xs font-bold uppercase tracking-wider text-slate-400">"Pages"</p>
                        {move || matches().into_iter().map(|(label, href, icon)| {
                            let href = *href;
                            view! {
                                <button
                                    class="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm text-slate-600 transition-colors hover:bg-blue-50 hover:text-blue-700"
                                    on:click=move |_| go.with_value(|f| f(href))
                                >
                                    <Icon name=*icon class="h-4 w-4 shrink-0 text-slate-400" />
                                    <span class="truncate">{*label}</span>
                                    <Icon name="arrow-right" class="ml-auto h-3 w-3 shrink-0 text-slate-300" />
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </div>

            <div class="flex shrink-0 items-center gap-1">
                // ---- Notifications --------------------------------------
                <div class="relative">
                    <button
                        aria-label="Notifications"
                        class="relative flex h-9 w-9 items-center justify-center rounded-lg text-slate-600 transition-colors hover:bg-slate-100"
                        on:click=move |_| { notifications_open.update(|v| *v = !*v); profile_open.set(false); }
                    >
                        <Icon name="bell" class="h-[18px] w-[18px]" />
                        <Show when=move || (unread() > 0)>
                            <span class="absolute right-1 top-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-red-500 px-1 text-[9px] font-bold text-white ring-2 ring-white">
                                {unread}
                            </span>
                        </Show>
                    </button>

                    <Show when=move || notifications_open.get()>
                        <div class="fixed inset-x-3 top-16 z-40 animate-fade-down rounded-xl border border-slate-200 bg-white shadow-2xl shadow-slate-900/10 sm:absolute sm:inset-x-auto sm:right-0 sm:top-full sm:mt-1.5 sm:w-96">
                            <div class="flex items-center justify-between border-b border-slate-100 px-4 py-3">
                                <h2 class="text-sm font-bold text-slate-900">"Notifications"</h2>
                                <button
                                    class="text-xs font-semibold text-blue-700 hover:underline"
                                    on:click=move |_| dismissed.set((0..NOTIFICATIONS.len()).collect())
                                >
                                    "Mark all read"
                                </button>
                            </div>

                            <div class="thin-scrollbar max-h-96 overflow-y-auto">
                                {NOTIFICATIONS.iter().enumerate().map(|(i, n)| view! {
                                    <button
                                        class=move || format!(
                                            "flex w-full gap-3 border-b border-slate-50 px-4 py-3 text-left transition-colors last:border-0 hover:bg-slate-50 {}",
                                            if n.unread && !dismissed.get().contains(&i) { "bg-blue-50/40" } else { "" }
                                        )
                                        on:click=move |_| dismissed.update(|d| { if !d.contains(&i) { d.push(i) } })
                                    >
                                        <span class=format!("flex h-9 w-9 shrink-0 items-center justify-center rounded-lg {}", n.tint)>
                                            <Icon name=n.icon class="h-4 w-4" />
                                        </span>
                                        <span class="min-w-0 flex-1">
                                            <span class="flex items-center gap-2">
                                                <span class="truncate text-sm font-semibold text-slate-800">{n.title}</span>
                                                <Show when=move || (n.unread && !dismissed.get().contains(&i))>
                                                    <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-blue-600"></span>
                                                </Show>
                                            </span>
                                            <span class="mt-0.5 block text-xs leading-relaxed text-slate-500">{n.body}</span>
                                            <span class="mt-1 block text-2xs text-slate-400">{n.time}</span>
                                        </span>
                                    </button>
                                }).collect_view()}
                            </div>

                            <div class="border-t border-slate-100 p-2">
                                <A
                                    href="/messages"
                                    attr:class="block rounded-lg py-2 text-center text-sm font-semibold text-blue-700 transition-colors hover:bg-blue-50"
                                >
                                    "Open messages"
                                </A>
                            </div>
                        </div>
                    </Show>
                </div>

                <A
                    href="/settings"
                    attr:aria-label="Settings"
                    attr:class="hidden h-9 w-9 items-center justify-center rounded-lg text-slate-600 transition-colors hover:bg-slate-100 sm:flex"
                >
                    <Icon name="settings" class="h-[18px] w-[18px]" />
                </A>

                // ---- Profile ---------------------------------------------
                <div class="relative">
                    <button
                        class="flex items-center gap-2 rounded-lg py-1 pl-1 pr-1.5 transition-colors hover:bg-slate-100"
                        on:click=move |_| { profile_open.update(|v| *v = !*v); notifications_open.set(false); }
                    >
                        <span class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-blue-600 to-indigo-700 text-xs font-bold text-white">
                            "AH"
                        </span>
                        <span class="hidden text-left sm:block">
                            <span class="block text-xs font-bold leading-tight text-slate-800">"Ahmed Hassan"</span>
                            <span class="block text-2xs leading-tight text-slate-400">"Administrator"</span>
                        </span>
                        <Icon name="chevron-down" class="hidden h-3 w-3 text-slate-400 sm:block" />
                    </button>

                    <Show when=move || profile_open.get()>
                        <div class="absolute right-0 top-full z-40 mt-1.5 w-56 animate-fade-down overflow-hidden rounded-xl border border-slate-200 bg-white p-1.5 shadow-2xl shadow-slate-900/10">
                            <div class="border-b border-slate-100 px-3 py-2.5">
                                <p class="text-sm font-bold text-slate-900">"Ahmed Hassan"</p>
                                <p class="truncate text-xs text-slate-500">"ahmed.hassan@tourista.com"</p>
                            </div>
                            {[
                                ("building", "Hotel profile", "/profile"),
                                ("user-plus", "Staff & roles", "/staff"),
                                ("settings", "Settings", "/settings"),
                            ].into_iter().map(|(icon, label, href)| view! {
                                <A
                                    href=href
                                    attr:class="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-slate-600 transition-colors hover:bg-slate-50"
                                >
                                    <Icon name=icon class="h-4 w-4 text-slate-400" />
                                    {label}
                                </A>
                            }).collect_view()}
                            <div class="my-1 h-px bg-slate-100"></div>
                            <A
                                href="/login"
                                attr:class="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50"
                            >
                                <Icon name="log-out" class="h-4 w-4" />
                                "Log out"
                            </A>
                        </div>
                    </Show>
                </div>
            </div>
        </header>
    }
}
