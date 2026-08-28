//! Notifications and announcements, backed by `/notifications/`.
//!
//! The API has no guest-messaging resource: there are no conversation threads to
//! read or reply to. What it does have is a notification feed for the property —
//! every booking, checkout and payment event lands there — plus a broadcast
//! endpoint for sending an announcement to a role.
//!
//! So this screen is the property inbox: a filterable feed on the left, the
//! selected item and its context on the right, a composer for announcements, and
//! the delivery preferences from `/notifications/preferences/`.

use crate::api;
use crate::components::{use_toast, Badge, Card, EmptyState, Icon, Modal};
use leptos::prelude::*;
use leptos_router::components::A;

const FILTERS: &[&str] = &["All", "Booking", "Payment", "System", "Alert"];

fn tone_for(kind: &str) -> &'static str {
    match kind {
        "Booking" => "blue",
        "Payment" => "green",
        "Alert" => "red",
        _ => "slate",
    }
}

fn icon_for(kind: &str) -> &'static str {
    match kind {
        "Booking" => "calendar-check",
        "Payment" => "banknote",
        "Alert" => "alert",
        _ => "settings",
    }
}

#[component]
pub fn MessagesPage() -> impl IntoView {
    let toast = use_toast();
    let filter = RwSignal::new("All");
    let unread_only = RwSignal::new(false);
    let search = RwSignal::new(String::new());
    let selected = RwSignal::new(Option::<api::Notification>::None);
    let compose_open = RwSignal::new(false);
    let prefs_open = RwSignal::new(false);
    let refresh = RwSignal::new(0u32);

    let feed = LocalResource::new(move || {
        let kind = filter.get();
        let only = unread_only.get();
        let q = search.get();
        let _ = refresh.get();
        async move {
            api::list_notifications(Some(kind), if only { Some(false) } else { None }, &q).await
        }
    });
    let counts = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::unread_counts().await }
    });

    let bump = move || refresh.update(|n| *n += 1);
    let unread = move || {
        counts
            .get()
            .and_then(Result::ok)
            .map(|c| c.unread_count)
            .unwrap_or(0)
    };

    let open = move |n: api::Notification| {
        let id = n.id;
        let was_unread = !n.is_read;
        selected.set(Some(n));
        if was_unread {
            wasm_bindgen_futures::spawn_local(async move {
                if api::mark_notification_read(id).await.is_ok() {
                    bump();
                }
            });
        }
    };

    let mark_all = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::mark_all_notifications_read().await {
                Ok(()) => {
                    toast.success("Inbox cleared", "Everything is marked as read.");
                    bump();
                }
                Err(e) => toast.error("Could not mark all read", e.detail()),
            }
        });
    };

    let dismiss = move |id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_notification(id).await {
                Ok(()) => {
                    selected.set(None);
                    bump();
                }
                Err(e) => toast.error("Could not dismiss it", e.detail()),
            }
        });
    };

    view! {
        <div class="flex h-[calc(100vh-3.5rem)] flex-col lg:flex-row">
            // ================= FEED =================
            <div class="flex w-full shrink-0 flex-col border-r border-slate-200 bg-white lg:w-96">
                <div class="border-b border-slate-200 p-4">
                    <div class="flex items-center justify-between gap-2">
                        <h1 class="text-lg font-bold text-slate-900">
                            "Inbox"
                            <Show when=move || (unread() > 0)>
                                <span class="ml-2 rounded-full bg-red-500 px-2 py-0.5 text-2xs font-bold text-white">
                                    {move || unread()}
                                </span>
                            </Show>
                        </h1>
                        <div class="flex items-center gap-1">
                            <button
                                title="Announcement"
                                class="rounded-lg p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700"
                                on:click=move |_| compose_open.set(true)
                            >
                                <Icon name="send" class="h-4 w-4" />
                            </button>
                            <button
                                title="Delivery preferences"
                                class="rounded-lg p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700"
                                on:click=move |_| prefs_open.set(true)
                            >
                                <Icon name="settings" class="h-4 w-4" />
                            </button>
                        </div>
                    </div>

                    <div class="relative mt-2">
                        <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                        <input
                            type="text"
                            placeholder="Search the feed"
                            class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                            prop:value=search
                            on:input:target=move |ev| search.set(ev.target().value())
                        />
                    </div>

                    <div class="mt-2 flex flex-wrap gap-1.5">
                        {FILTERS.iter().map(|f| {
                            let f = *f;
                            view! {
                                <button
                                    class=move || format!(
                                        "rounded-md px-2 py-1 text-2xs font-semibold transition-colors {}",
                                        if filter.get() == f { "bg-blue-700 text-white" } else { "bg-slate-100 text-slate-600 hover:bg-slate-200" }
                                    )
                                    on:click=move |_| filter.set(f)
                                >
                                    {f}
                                </button>
                            }
                        }).collect_view()}
                    </div>

                    <div class="mt-2 flex items-center justify-between text-2xs">
                        <label class="flex cursor-pointer items-center gap-1.5 text-slate-500">
                            <input type="checkbox" class="h-3.5 w-3.5" prop:checked=unread_only
                                on:change:target=move |ev| unread_only.set(ev.target().checked()) />
                            "Unread only"
                        </label>
                        <button class="font-semibold text-blue-700 hover:text-blue-800" on:click=mark_all>
                            "Mark all read"
                        </button>
                    </div>
                </div>

                <div class="min-h-0 flex-1 overflow-y-auto">
                    <Suspense fallback=|| view! {
                        <p class="p-6 text-center text-sm text-slate-400">"Loading feed…"</p>
                    }>
                        {move || Suspend::new(async move {
                            let rows = match feed.await {
                                Ok(p) => p.items,
                                Err(e) => return view! {
                                    <p class="p-6 text-center text-sm text-red-600">{e.detail()}</p>
                                }.into_any(),
                            };
                            if rows.is_empty() {
                                return view! {
                                    <EmptyState
                                        icon="bell"
                                        title="Nothing here"
                                        body="Booking, checkout and payment events land in this feed as they happen."
                                    />
                                }.into_any();
                            }
                            view! {
                                <div>
                                    {rows.into_iter().map(|n| {
                                        let id = n.id;
                                        let is_read = n.is_read;
                                        let kind = n.kind().to_string();
                                        let row = n.clone();
                                        view! {
                                            <button
                                                on:click=move |_| open(row.clone())
                                                class=move || format!(
                                                    "flex w-full items-start gap-3 border-b border-slate-100 p-4 text-left transition-colors hover:bg-slate-50 {}",
                                                    if selected.get().as_ref().is_some_and(|s| s.id == id) { "bg-blue-50/60" } else { "" }
                                                )
                                            >
                                                <span class=format!(
                                                    "flex h-9 w-9 shrink-0 items-center justify-center rounded-full {}",
                                                    match kind.as_str() {
                                                        "Booking" => "bg-blue-50 text-blue-600",
                                                        "Payment" => "bg-emerald-50 text-emerald-600",
                                                        "Alert" => "bg-red-50 text-red-600",
                                                        _ => "bg-slate-100 text-slate-500",
                                                    }
                                                )>
                                                    <Icon name=icon_for(&kind) class="h-4 w-4" />
                                                </span>
                                                <span class="min-w-0 flex-1">
                                                    <span class="flex items-baseline justify-between gap-2">
                                                        <span class=format!(
                                                            "truncate text-sm {}",
                                                            if is_read { "font-medium text-slate-700" } else { "font-bold text-slate-900" }
                                                        )>
                                                            {n.title.clone()}
                                                        </span>
                                                        <span class="shrink-0 text-2xs text-slate-400">
                                                            {api::pretty_datetime(n.created_at.as_deref())}
                                                        </span>
                                                    </span>
                                                    <span class="mt-0.5 block line-clamp-2 text-xs text-slate-500">{n.message.clone()}</span>
                                                </span>
                                                <Show when=move || !is_read>
                                                    <span class="mt-1.5 h-2 w-2 shrink-0 rounded-full bg-blue-600"></span>
                                                </Show>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </div>

            // ================= DETAIL =================
            <div class="min-h-0 flex-1 overflow-y-auto bg-slate-50 p-4 sm:p-6">
                {move || match selected.get() {
                    None => view! {
                        <div class="flex h-full flex-col items-center justify-center gap-3 text-center">
                            <span class="flex h-14 w-14 items-center justify-center rounded-full bg-white text-slate-300 ring-1 ring-slate-200">
                                <Icon name="bell" class="h-6 w-6" />
                            </span>
                            <p class="text-sm font-semibold text-slate-600">"Select an item to read it"</p>
                            <p class="max-w-sm text-xs text-slate-400">
                                "This is the property feed. The API has no guest-messaging endpoint, so replies to guests happen on the reservation itself."
                            </p>
                        </div>
                    }.into_any(),
                    Some(n) => {
                        let id = n.id;
                        let kind = n.kind().to_string();
                        let route = n.route();
                        view! {
                            <div class="mx-auto flex max-w-2xl flex-col gap-4">
                                <Card>
                                    <div class="mb-3 flex flex-wrap items-start justify-between gap-3">
                                        <div class="min-w-0">
                                            <h2 class="text-lg font-bold text-slate-900">{n.title.clone()}</h2>
                                            <p class="mt-0.5 text-xs text-slate-500">
                                                {format!(
                                                    "{} · {}",
                                                    n.sender_name(),
                                                    api::pretty_datetime(n.created_at.as_deref()),
                                                )}
                                            </p>
                                        </div>
                                        <Badge label=kind.clone() tone=tone_for(&kind) />
                                    </div>

                                    <p class="whitespace-pre-line text-sm leading-relaxed text-slate-700">
                                        {n.message.clone()}
                                    </p>

                                    <dl class="mt-4 grid grid-cols-2 gap-x-4 gap-y-2 border-t border-slate-100 pt-4 text-xs">
                                        <Fact label="Audience" value=n.target_role.clone().unwrap_or_else(|| "—".into()) />
                                        <Fact label="Related to" value=n.entity_type.clone().unwrap_or_else(|| "—".into()) />
                                        <Fact label="Reference" value=n.entity_id.clone().unwrap_or_else(|| "—".into()) />
                                        <Fact label="Status" value=(if n.is_read { "Read" } else { "Unread" }).to_string() />
                                    </dl>

                                    <div class="mt-4 flex flex-wrap gap-2 border-t border-slate-100 pt-4">
                                        {route.map(|r| view! {
                                            <A
                                                href=r
                                                attr:class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-800"
                                            >
                                                "Open related record"
                                            </A>
                                        })}
                                        <button
                                            class="rounded-lg border border-red-300 px-3 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
                                            on:click=move |_| dismiss(id)
                                        >
                                            "Dismiss"
                                        </button>
                                    </div>
                                </Card>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <Show when=move || compose_open.get()>
                <ComposeModal
                    on_close=move || compose_open.set(false)
                    on_sent=move || {
                        toast.success("Announcement sent", "It is now in the recipients' feed.");
                        bump();
                    }
                />
            </Show>

            <Show when=move || prefs_open.get()>
                <PrefsModal on_close=move || prefs_open.set(false) />
            </Show>
        </div>
    }
}

/// Broadcasts an announcement to a role inside the property.
#[component]
fn ComposeModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_sent: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let audience = RwSignal::new("STAFF".to_string());
    let kind = RwSignal::new("System".to_string());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if title.get().trim().is_empty() || body.get().trim().is_empty() {
            error.set(Some("Give the announcement a title and a message.".into()));
            return;
        }
        error.set(None);
        busy.set(true);
        let (a, t, b, k) = (audience.get(), title.get(), body.get(), kind.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::broadcast_notification(&a, &t, &b, &k).await {
                Ok(()) => {
                    busy.set(false);
                    on_sent();
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
        <Modal title="New announcement" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Audience"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| audience.set(ev.target().value())
                        >
                            {api::TARGET_ROLES.iter().map(|r| view! {
                                <option value=*r selected=(*r == "STAFF")>{*r}</option>
                            }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Type"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| kind.set(ev.target().value())
                        >
                            {api::NOTIFICATION_TYPES.iter().map(|r| view! {
                                <option value=*r selected=(*r == "System")>{*r}</option>
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Title"</label>
                    <input type="text" placeholder="e.g. Water shutdown on Friday"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=title on:input:target=move |ev| title.set(ev.target().value()) />
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Message"</label>
                    <textarea rows="5" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=body on:input:target=move |ev| body.set(ev.target().value())></textarea>
                </div>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        <Icon name="send" class="h-4 w-4" />
                        {move || if busy.get() { "Sending…" } else { "Send" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

/// Delivery preferences from `/notifications/preferences/`.
#[component]
fn PrefsModal(on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let toast = use_toast();
    let loaded = LocalResource::new(|| async move { api::get_notification_prefs().await });
    let prefs = RwSignal::new(api::NotificationPrefs::default());
    let ready = RwSignal::new(false);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(Ok(p)) = loaded.get().map(|r| r.map(|x| x.clone())) {
            if !ready.get() {
                prefs.set(p);
                ready.set(true);
            }
        }
    });

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let p = prefs.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_notification_prefs(&p).await {
                Ok(_) => {
                    busy.set(false);
                    toast.success("Preferences saved", "Delivery settings updated.");
                    on_close();
                }
                Err(e) => {
                    busy.set(false);
                    toast.error("Could not save preferences", e.detail());
                }
            }
        });
    };

    view! {
        <Modal title="Notification preferences" on_close=on_close>
            <Suspense fallback=|| view! {
                <p class="py-8 text-center text-sm text-slate-400">"Loading preferences…"</p>
            }>
                {move || Suspend::new(async move {
                    if let Err(e) = loaded.await {
                        return view! {
                            <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                        }.into_any();
                    }
                    view! {
                        <div class="flex flex-col gap-3">
                            <Switch
                                label="Booking alerts"
                                hint="New bookings, confirmations, cancellations"
                                get=Signal::derive(move || prefs.get().booking_alerts)
                                set=move |v| prefs.update(|p| p.booking_alerts = v)
                            />
                            <Switch
                                label="Payment alerts"
                                hint="Checkouts and settlements"
                                get=Signal::derive(move || prefs.get().payment_alerts)
                                set=move |v| prefs.update(|p| p.payment_alerts = v)
                            />
                            <Switch
                                label="System alerts"
                                hint="Platform notices"
                                get=Signal::derive(move || prefs.get().system_alerts)
                                set=move |v| prefs.update(|p| p.system_alerts = v)
                            />
                            <Switch
                                label="Email notifications"
                                hint="Also send these to your inbox"
                                get=Signal::derive(move || prefs.get().email_notifications)
                                set=move |v| prefs.update(|p| p.email_notifications = v)
                            />
                            <Switch
                                label="Sound"
                                hint="Play a chime for new items"
                                get=Signal::derive(move || prefs.get().sound_enabled)
                                set=move |v| prefs.update(|p| p.sound_enabled = v)
                            />

                            <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                                <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                                <button type="button" on:click=save disabled=move || busy.get()
                                    class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                                    {move || if busy.get() { "Saving…" } else { "Save" }}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </Modal>
    }
}

#[component]
fn Switch(
    label: &'static str,
    hint: &'static str,
    get: Signal<bool>,
    set: impl Fn(bool) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <label class="flex cursor-pointer items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2.5">
            <span class="min-w-0">
                <span class="block text-sm font-medium text-slate-800">{label}</span>
                <span class="block text-2xs text-slate-500">{hint}</span>
            </span>
            <input
                type="checkbox"
                class="h-4 w-4 shrink-0"
                prop:checked=move || get.get()
                on:change:target=move |ev| set(ev.target().checked())
            />
        </label>
    }
}

#[component]
fn Fact(label: &'static str, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div>
            <dt class="text-2xs uppercase tracking-wide text-slate-400">{label}</dt>
            <dd class="mt-0.5 font-medium text-slate-700">{value}</dd>
        </div>
    }
}
