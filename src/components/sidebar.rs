use crate::api;
use crate::components::Icon;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// Shared chrome state so the topbar can drive the sidebar.
#[derive(Copy, Clone)]
pub struct LayoutCtx {
    /// Narrow icon-only rail on desktop.
    pub collapsed: RwSignal<bool>,
    /// Overlay drawer on small screens.
    pub mobile_open: RwSignal<bool>,
}

pub fn provide_layout() {
    provide_context(LayoutCtx {
        collapsed: RwSignal::new(false),
        mobile_open: RwSignal::new(false),
    });
}

pub fn use_layout() -> LayoutCtx {
    expect_context::<LayoutCtx>()
}

/// Which live counter a nav item shows in its badge, if any.
#[derive(Clone, Copy, PartialEq)]
enum Badge {
    None,
    /// Reservations awaiting a decision.
    PendingReservations,
    /// Completed stays still carrying a balance.
    Outstanding,
    /// Unread notifications.
    Unread,
}

struct NavItem {
    label: &'static str,
    href: &'static str,
    icon: &'static str,
    badge: Badge,
}

struct NavGroup {
    heading: &'static str,
    items: &'static [NavItem],
}

const NAV: &[NavGroup] = &[
    NavGroup {
        heading: "Overview",
        items: &[
            NavItem { label: "Dashboard", href: "/", icon: "home", badge: Badge::None },
            NavItem { label: "Analytics", href: "/analytics", icon: "bar-chart", badge: Badge::None },
        ],
    },
    NavGroup {
        heading: "Operations",
        items: &[
            NavItem { label: "Reservations", href: "/reservations", icon: "calendar-check", badge: Badge::PendingReservations },
            NavItem { label: "Calendar", href: "/calendar", icon: "calendar", badge: Badge::None },
            NavItem { label: "Rooms", href: "/rooms", icon: "bed", badge: Badge::None },
            NavItem { label: "Rates", href: "/rates", icon: "tag", badge: Badge::None },
            NavItem { label: "Guests", href: "/guests", icon: "users", badge: Badge::None },
        ],
    },
    NavGroup {
        heading: "Revenue",
        items: &[
            NavItem { label: "Payments", href: "/payments", icon: "wallet", badge: Badge::Outstanding },
        ],
    },
    NavGroup {
        heading: "Engagement",
        items: &[
            NavItem { label: "Messages", href: "/messages", icon: "mail", badge: Badge::Unread },
            NavItem { label: "Reviews", href: "/reviews", icon: "star", badge: Badge::None },
        ],
    },
    NavGroup {
        heading: "Administration",
        items: &[
            NavItem { label: "Hotel Profile", href: "/profile", icon: "building", badge: Badge::None },
            NavItem { label: "Staff & Roles", href: "/staff", icon: "user-plus", badge: Badge::None },
            NavItem { label: "Settings", href: "/settings", icon: "settings", badge: Badge::None },
        ],
    },
];

#[component]
pub fn Sidebar() -> impl IntoView {
    let layout = use_layout();
    let location = use_location();
    let logout_nav = leptos_router::hooks::use_navigate();

    // The property header and the nav badges come from the API. They are read
    // once per mount rather than per navigation — the numbers move slowly and
    // each screen refreshes its own data anyway.
    let hotel = LocalResource::new(|| async move { api::dashboard_summary(None).await });
    let unread = LocalResource::new(|| async move { api::unread_counts().await });
    let pending = LocalResource::new(|| async move {
        api::list_reservations(&api::ReservationQuery {
            status_tab: "new".into(),
            page_size: 100,
            ..Default::default()
        })
        .await
    });
    let ledger = LocalResource::new(|| async move { api::payments_ledger("").await });

    let badge_count = move |kind: Badge| -> u32 {
        match kind {
            Badge::None => 0,
            Badge::Unread => unread.get().and_then(Result::ok).map(|c| c.unread_count).unwrap_or(0),
            Badge::PendingReservations => pending
                .get()
                .and_then(Result::ok)
                .map(|p| p.meta.count)
                .unwrap_or(0),
            Badge::Outstanding => ledger
                .get()
                .and_then(Result::ok)
                .map(|l| l.transactions.iter().filter(|t| t.outstanding > 0.0).count() as u32)
                .unwrap_or(0),
        }
    };

    let hotel_name = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .map(|s| s.hotel.name)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Your property".into())
    };
    let hotel_location = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .map(|s| s.hotel.location())
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| "No address set".into())
    };
    let hotel_logo = move || {
        hotel
            .get()
            .and_then(Result::ok)
            .and_then(|s| s.hotel.logo)
            .filter(|l| l.starts_with("http"))
    };
    let is_verified = move || {
        hotel.get().and_then(Result::ok).map(|s| s.hotel.is_verified).unwrap_or(false)
    };

    // Any navigation closes the mobile drawer.
    Effect::new(move |_| {
        let _ = location.pathname.get();
        layout.mobile_open.set(false);
    });

    view! {
        // ---- Mobile backdrop ------------------------------------------
        <Show when=move || layout.mobile_open.get()>
            <div
                class="fixed inset-0 z-40 animate-fade-in bg-slate-900/50 backdrop-blur-sm lg:hidden"
                on:click=move |_| layout.mobile_open.set(false)
            ></div>
        </Show>

        <aside class=move || format!(
            "fixed inset-y-0 left-0 z-50 flex h-screen shrink-0 flex-col overflow-hidden bg-gradient-to-b from-slate-900 via-slate-900 to-slate-950 text-slate-200 transition-all duration-300 ease-soft lg:sticky lg:top-0 lg:z-auto lg:translate-x-0 {} {}",
            if layout.collapsed.get() { "lg:w-[4.75rem]" } else { "lg:w-64" },
            if layout.mobile_open.get() { "w-72 translate-x-0 shadow-2xl" } else { "w-72 -translate-x-full" },
        )>
            // ---- Property header --------------------------------------
            <div class=move || format!(
                "flex items-center gap-3 border-b border-white/5 transition-all duration-300 {}",
                if layout.collapsed.get() { "justify-center p-3" } else { "p-4" }
            )>
                {move || match hotel_logo() {
                    Some(url) => view! {
                        <img
                            src=url
                            alt=hotel_name()
                            class=move || format!(
                                "shrink-0 rounded-xl object-cover shadow-lg ring-1 ring-white/10 transition-all duration-300 hover:scale-105 {}",
                                if layout.collapsed.get() { "h-11 w-11" } else { "h-12 w-12" }
                            )
                        />
                    }.into_any(),
                    // No logo uploaded yet — fall back to a monogram rather than
                    // a stock photo of somebody else's hotel.
                    None => view! {
                        <span class=move || format!(
                            "flex shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-blue-600 to-indigo-700 font-bold text-white shadow-lg ring-1 ring-white/10 {}",
                            if layout.collapsed.get() { "h-11 w-11 text-sm" } else { "h-12 w-12 text-base" }
                        )>
                            {hotel_name().chars().next().unwrap_or('H').to_uppercase().to_string()}
                        </span>
                    }.into_any(),
                }}
                <Show when=move || !layout.collapsed.get()>
                    <div class="min-w-0 animate-fade-in">
                        <p class="truncate text-sm font-bold text-white">{hotel_name}</p>
                        <p class="truncate text-2xs text-slate-400">{hotel_location}</p>
                        <Show when=move || is_verified()>
                            <span class="mt-1 inline-flex items-center gap-1 rounded-full bg-emerald-500/10 px-1.5 py-0.5 text-2xs font-semibold text-emerald-400 ring-1 ring-emerald-500/20">
                                <Icon name="check-circle" class="h-3 w-3" />
                                "Verified"
                            </span>
                        </Show>
                    </div>
                </Show>
                <button
                    aria-label="Close menu"
                    class="ml-auto flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/10 hover:text-white lg:hidden"
                    on:click=move |_| layout.mobile_open.set(false)
                >
                    <Icon name="x" class="h-4 w-4" />
                </button>
            </div>

            // ---- Navigation -------------------------------------------
            <nav class="thin-scrollbar flex-1 overflow-y-auto px-2.5 py-3">
                {NAV.iter().map(|group| view! {
                    <div class="mb-3">
                        <Show when=move || !layout.collapsed.get()>
                            <p class="mb-1 px-2.5 text-2xs font-bold uppercase tracking-wider text-slate-500">
                                {group.heading}
                            </p>
                        </Show>
                        <Show when=move || layout.collapsed.get()>
                            <div class="mx-3 mb-2 h-px bg-white/5"></div>
                        </Show>

                        {group.items.iter().map(|item| {
                            let is_active = move || {
                                let path = location.pathname.get();
                                if item.href == "/" { path == "/" } else { path.starts_with(item.href) }
                            };
                            view! {
                                <A
                                    href=item.href
                                    attr:title=item.label
                                    attr:class=move || format!(
                                        "group relative mb-0.5 flex items-center overflow-hidden rounded-lg text-sm font-medium transition-all duration-200 {} {}",
                                        if layout.collapsed.get() { "justify-center px-2 py-2.5" } else { "justify-between px-2.5 py-2.5" },
                                        if is_active() {
                                            "bg-blue-600 text-white shadow-lg shadow-blue-950/50"
                                        } else {
                                            "text-slate-300 hover:bg-white/5 hover:text-white"
                                        }
                                    )
                                >
                                    <span class=move || format!(
                                        "absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r bg-sky-300 transition-transform duration-200 {}",
                                        if is_active() { "scale-y-100" } else { "scale-y-0" }
                                    )></span>

                                    <span class="flex min-w-0 items-center gap-3">
                                        <span class="relative shrink-0">
                                            <Icon name=item.icon class="h-4 w-4 transition-transform duration-200 group-hover:scale-110" />
                                            <Show when=move || (layout.collapsed.get() && badge_count(item.badge) > 0)>
                                                <span class="absolute -right-1.5 -top-1.5 h-2 w-2 rounded-full bg-red-500 ring-2 ring-slate-900"></span>
                                            </Show>
                                        </span>
                                        <Show when=move || !layout.collapsed.get()>
                                            <span class="truncate">{item.label}</span>
                                        </Show>
                                    </span>

                                    <Show when=move || (!layout.collapsed.get() && badge_count(item.badge) > 0)>
                                        <span class="shrink-0 rounded-full bg-red-500 px-1.5 py-0.5 text-2xs font-bold text-white">
                                            {move || badge_count(item.badge)}
                                        </span>
                                    </Show>
                                </A>
                            }
                        }).collect_view()}
                    </div>
                }).collect_view()}
            </nav>

            // ---- Footer ------------------------------------------------
            <div class="border-t border-white/5 p-2.5">
                <button
                    aria-label="Toggle sidebar width"
                    class="mb-1 hidden w-full items-center gap-3 rounded-lg px-2.5 py-2.5 text-sm font-medium text-slate-400 transition-colors hover:bg-white/5 hover:text-white lg:flex"
                    on:click=move |_| layout.collapsed.update(|v| *v = !*v)
                >
                    <span class=move || format!(
                        "shrink-0 transition-transform duration-300 {}",
                        if layout.collapsed.get() { "rotate-180" } else { "" }
                    )>
                        <Icon name="chevron-left" class="h-4 w-4" />
                    </span>
                    <Show when=move || !layout.collapsed.get()>
                        <span>"Collapse"</span>
                    </Show>
                </button>

                <button
                    title="Logout"
                    on:click=move |_| {
                        // Drop the tokens before navigating, or the guard on
                        // /login bounces straight back into the dashboard.
                        crate::api::logout();
                        logout_nav("/login", Default::default());
                    }
                    class=move || format!(
                        "flex w-full items-center gap-3 rounded-lg py-2.5 text-sm font-medium text-slate-300 transition-colors hover:bg-red-500/10 hover:text-red-300 {}",
                        if layout.collapsed.get() { "justify-center px-2" } else { "px-2.5" }
                    )
                >
                    <Icon name="log-out" class="h-4 w-4 shrink-0" />
                    <Show when=move || !layout.collapsed.get()>
                        <span>"Logout"</span>
                    </Show>
                </button>
            </div>
        </aside>
    }
}
