//! Account settings, backed by the `/accounts/` endpoints.
//!
//! Four tabs, each mapped to real endpoints:
//!
//! * **Profile** — `GET`/`PATCH /accounts/me/`, plus notification delivery from
//!   `/notifications/preferences/`.
//! * **Security** — `POST /accounts/auth/password/change/`.
//! * **Account** — the hotels this login is attached to (`/organizations/`) and
//!   the permission codes the API granted, plus account deactivation
//!   (`DELETE /accounts/me/`).
//! * **Activity** — the audit trail from `/accounts/audit-logs/`.
//!
//! There is no two-factor or session-management endpoint, so those panels are
//! not shown; roles and team membership live on the Staff screen instead.

use crate::api;
use crate::components::{use_toast, Avatar, Badge, Card, EmptyState, Icon, PageHeader};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

const TABS: &[&str] = &["Profile", "Security", "Account", "Activity"];

#[component]
pub fn SettingsPage() -> impl IntoView {
    let tab = RwSignal::new(TABS[0]);

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Settings"
                subtitle="Your profile, password and what this login can reach."
            />

            <div class="mb-5 flex flex-wrap gap-2 text-sm">
                {TABS.iter().map(|t| {
                    let t = *t;
                    view! {
                        <button
                            class=move || format!(
                                "rounded-lg px-4 py-2 font-semibold transition-all duration-200 {}",
                                if tab.get() == t { "bg-blue-700 text-white shadow-md active:scale-[0.98]" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                            )
                            on:click=move |_| tab.set(t)
                        >
                            {t}
                        </button>
                    }
                }).collect_view()}
            </div>

            <Show when=move || (tab.get() == "Profile")><ProfileTab/></Show>
            <Show when=move || (tab.get() == "Security")><SecurityTab/></Show>
            <Show when=move || (tab.get() == "Account")><AccountTab/></Show>
            <Show when=move || (tab.get() == "Activity")><ActivityTab/></Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

#[component]
fn ProfileTab() -> impl IntoView {
    let toast = use_toast();
    let refresh = RwSignal::new(0u32);
    let me = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::me().await }
    });

    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let postal = RwSignal::new(String::new());
    let seeded = RwSignal::new(false);
    let busy = RwSignal::new(false);

    // Seed the form once the profile lands, but never overwrite edits in flight.
    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        if let Some(Ok(u)) = me.get().map(|r| r.map(|x| x.clone())) {
            first.set(u.first_name.clone().unwrap_or_default());
            last.set(u.last_name.clone().unwrap_or_default());
            phone.set(u.phone.clone().unwrap_or_default());
            address.set(u.address_1.clone().unwrap_or_default());
            city.set(u.city.clone().unwrap_or_default());
            postal.set(u.postal_code.clone().unwrap_or_default());
            seeded.set(true);
        }
    });

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let payload = api::ProfileUpdate {
            first_name: first.get().trim().to_string(),
            last_name: last.get().trim().to_string(),
            phone: phone.get().trim().to_string(),
            address_1: address.get().trim().to_string(),
            city: city.get().trim().to_string(),
            postal_code: postal.get().trim().to_string(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_profile(&payload).await {
                Ok(_) => {
                    toast.success("Profile saved", "Your details have been updated.");
                    refresh.update(|n| *n += 1);
                }
                Err(e) => toast.error("Could not save your profile", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <div class="grid animate-fade-up gap-5 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
            <Card title="Your profile" hint="Shown to your team and on activity records">
                <Suspense fallback=|| view! {
                    <p class="py-8 text-center text-sm text-slate-400">"Loading profile…"</p>
                }>
                    {move || Suspend::new(async move {
                        let u = match me.await {
                            Ok(u) => u,
                            Err(e) => return view! {
                                <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                            }.into_any(),
                        };
                        view! {
                            <div class="mb-5 flex items-center gap-3">
                                <Avatar initials=u.initials() size="h-14 w-14 text-base" />
                                <div class="min-w-0">
                                    <p class="font-bold text-slate-900">{u.display_name()}</p>
                                    <p class="text-sm text-slate-500">{u.email.clone().unwrap_or_default()}</p>
                                    <Badge label=u.role_label() tone="blue" />
                                </div>
                            </div>

                            <div class="flex flex-col gap-4">
                                <div class="grid grid-cols-2 gap-4">
                                    <Field label="First name" value=first />
                                    <Field label="Last name" value=last />
                                </div>
                                <Field label="Phone" value=phone kind="tel" />
                                <Field label="Address" value=address />
                                <div class="grid grid-cols-2 gap-4">
                                    <Field label="City" value=city />
                                    <Field label="Postal code" value=postal />
                                </div>
                                <p class="text-2xs text-slate-500">
                                    "Your email address is your sign-in identifier and cannot be changed here."
                                </p>
                                <div class="flex justify-end border-t border-slate-100 pt-4">
                                    <button on:click=save disabled=move || busy.get()
                                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                                        {move || if busy.get() { "Saving…" } else { "Save profile" }}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </Card>

            <NotificationPrefsCard/>
        </div>
    }
}

/// Delivery preferences, from `/notifications/preferences/`.
#[component]
fn NotificationPrefsCard() -> impl IntoView {
    let toast = use_toast();
    let loaded = LocalResource::new(|| async move { api::get_notification_prefs().await });
    let prefs = RwSignal::new(api::NotificationPrefs::default());
    let seeded = RwSignal::new(false);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        if let Some(Ok(p)) = loaded.get().map(|r| r.map(|x| x.clone())) {
            prefs.set(p);
            seeded.set(true);
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
                Ok(_) => toast.success("Preferences saved", "Delivery settings updated."),
                Err(e) => toast.error("Could not save preferences", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <Card title="Notifications" hint="What reaches you, and how">
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
                        <div class="flex flex-col gap-2">
                            <NotifRow icon="calendar-check" label="Booking alerts" hint="New bookings and cancellations"
                                get=Signal::derive(move || prefs.get().booking_alerts)
                                set=move |v| prefs.update(|p| p.booking_alerts = v) />
                            <NotifRow icon="banknote" label="Payment alerts" hint="Checkouts and settlements"
                                get=Signal::derive(move || prefs.get().payment_alerts)
                                set=move |v| prefs.update(|p| p.payment_alerts = v) />
                            <NotifRow icon="settings" label="System alerts" hint="Platform notices"
                                get=Signal::derive(move || prefs.get().system_alerts)
                                set=move |v| prefs.update(|p| p.system_alerts = v) />
                            <NotifRow icon="mail" label="Email copies" hint="Also send these to your inbox"
                                get=Signal::derive(move || prefs.get().email_notifications)
                                set=move |v| prefs.update(|p| p.email_notifications = v) />
                            <NotifRow icon="bell" label="Sound" hint="Play a chime for new items"
                                get=Signal::derive(move || prefs.get().sound_enabled)
                                set=move |v| prefs.update(|p| p.sound_enabled = v) />
                        </div>
                        <div class="mt-4 flex justify-end border-t border-slate-100 pt-4">
                            <button on:click=save disabled=move || busy.get()
                                class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                                {move || if busy.get() { "Saving…" } else { "Save preferences" }}
                            </button>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </Card>
    }
}

// ---------------------------------------------------------------------------
// Security
// ---------------------------------------------------------------------------

#[component]
fn SecurityTab() -> impl IntoView {
    let toast = use_toast();
    let navigate = use_navigate();
    let nav = StoredValue::new(navigate);
    let current = RwSignal::new(String::new());
    let next = RwSignal::new(String::new());
    let confirm_pw = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if next.get().len() < 6 {
            error.set(Some("The new password must be at least 6 characters.".into()));
            return;
        }
        if next.get() != confirm_pw.get() {
            error.set(Some("The two new passwords do not match.".into()));
            return;
        }
        error.set(None);
        busy.set(true);
        let (old, new) = (current.get(), next.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::change_password(&old, &new).await {
                Ok(()) => {
                    busy.set(false);
                    current.set(String::new());
                    next.set(String::new());
                    confirm_pw.set(String::new());
                    toast.success("Password changed", "Use your new password next time you sign in.");
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    let sign_out = move |_| {
        api::logout();
        nav.with_value(|n| n("/login", Default::default()));
    };

    view! {
        <div class="grid animate-fade-up gap-5 lg:grid-cols-2">
            <Card title="Change password" hint="At least 6 characters">
                <form on:submit=submit class="flex flex-col gap-4">
                    <Show when=move || error.get().is_some()>
                        <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                            <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Field label="Current password" value=current kind="password" />
                    <Field label="New password" value=next kind="password" />
                    <Field label="Confirm new password" value=confirm_pw kind="password" />
                    <div class="flex justify-end border-t border-slate-100 pt-4">
                        <button type="submit" disabled=move || busy.get()
                            class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                            {move || if busy.get() { "Changing…" } else { "Change password" }}
                        </button>
                    </div>
                </form>
            </Card>

            <div class="flex flex-col gap-5">
                <Card title="Session">
                    <p class="text-sm text-slate-600">
                        "You are signed in with a JSON Web Token stored in this browser. The token is refreshed automatically while you work and expires an hour after it was issued."
                    </p>
                    <p class="mt-2 text-2xs text-slate-500">
                        "The API keeps no server-side session list, so there is nothing to revoke remotely — signing out clears the token from this browser."
                    </p>
                    <button
                        on:click=sign_out
                        class="mt-4 flex w-full items-center justify-center gap-2 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                    >
                        <Icon name="log-out" class="h-4 w-4" />
                        "Sign out of this browser"
                    </button>
                </Card>

                <div class="flex items-start gap-3 rounded-xl bg-amber-50 p-4 text-sm text-amber-900 ring-1 ring-amber-100">
                    <Icon name="shield-check" class="mt-0.5 h-4 w-4 shrink-0" />
                    <span>
                        <span class="font-bold">"Forgotten passwords "</span>
                        "are reset with a one-time code sent to the account's email or phone — use the link on the sign-in page."
                    </span>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

#[component]
fn AccountTab() -> impl IntoView {
    let toast = use_toast();
    let navigate = use_navigate();
    let nav = StoredValue::new(navigate);
    let me = LocalResource::new(|| async move { api::me().await });
    let orgs = LocalResource::new(|| async move { api::list_organizations().await });

    let switch_to = move |id: i64, name: String| {
        api::session::set_active_org(id);
        toast.success("Property switched", format!("Now working on {name}."));
        // Everything downstream is scoped to the active hotel, so land on the
        // dashboard rather than leaving stale data on screen.
        nav.with_value(|n| n("/", Default::default()));
    };

    let deactivate = move |_| {
        if !crate::confirm(
            "Deactivate your account? You will be signed out and will lose access to this property.",
        ) {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            match api::request_value(
                api::Method::Delete,
                "/accounts/me/",
                None,
                api::Auth::Required,
            )
            .await
            {
                Ok(_) => {
                    api::logout();
                    nav.with_value(|n| n("/login", Default::default()));
                }
                Err(e) => toast.error("Could not deactivate the account", e.detail()),
            }
        });
    };

    view! {
        <div class="grid animate-fade-up gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <Card title="Properties" hint="Hotels this login can administer">
                <Suspense fallback=|| view! {
                    <p class="py-8 text-center text-sm text-slate-400">"Loading properties…"</p>
                }>
                    {move || Suspend::new(async move {
                        let rows = match orgs.await {
                            Ok(r) => r,
                            Err(e) => return view! {
                                <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                            }.into_any(),
                        };
                        if rows.is_empty() {
                            return view! {
                                <div>
                                    <p class="py-6 text-center text-sm text-slate-400">
                                        "This account is not attached to a hotel yet."
                                    </p>
                                    <A href="/profile" attr:class="flex w-full items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white hover:bg-blue-800">
                                        <Icon name="plus" class="h-4 w-4" />
                                        "Create your property"
                                    </A>
                                </div>
                            }.into_any();
                        }
                        let active = api::session::active_org();
                        view! {
                            <div class="flex flex-col gap-2.5">
                                {rows.into_iter().map(|o| {
                                    let is_active = active == Some(o.id);
                                    let oid = o.id;
                                    let name = o.name.clone();
                                    view! {
                                        <div class=format!(
                                            "flex flex-wrap items-center justify-between gap-3 rounded-xl border p-3 {}",
                                            if is_active { "border-blue-300 bg-blue-50/50" } else { "border-slate-200" }
                                        )>
                                            <div class="min-w-0">
                                                <p class="flex flex-wrap items-center gap-2 font-semibold text-slate-900">
                                                    {o.name.clone()}
                                                    {is_active.then(|| view! { <Badge label="Active" tone="blue" /> })}
                                                </p>
                                                <p class="mt-0.5 text-xs text-slate-500">
                                                    {format!(
                                                        "{} · {}",
                                                        o.role_name.clone(),
                                                        [o.city.clone(), o.country.clone()]
                                                            .into_iter()
                                                            .flatten()
                                                            .filter(|s| !s.is_empty())
                                                            .collect::<Vec<_>>()
                                                            .join(", "),
                                                    )}
                                                </p>
                                            </div>
                                            <Show when=move || !is_active>
                                                <button
                                                    class="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-semibold hover:bg-white"
                                                    on:click={
                                                        let n = name.clone();
                                                        move |_| switch_to(oid, n.clone())
                                                    }
                                                >
                                                    "Switch to"
                                                </button>
                                            </Show>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </Card>

            <div class="flex flex-col gap-5">
                <Card title="What this login can do" hint="Permission codes granted by the API">
                    <Suspense fallback=|| view! {
                        <p class="py-8 text-center text-sm text-slate-400">"Loading permissions…"</p>
                    }>
                        {move || Suspend::new(async move {
                            let u = match me.await {
                                Ok(u) => u,
                                Err(e) => return view! {
                                    <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                                }.into_any(),
                            };
                            if u.permissions.is_empty() {
                                return view! {
                                    <p class="py-6 text-center text-sm text-slate-400">
                                        "No permissions yet — they are granted once the account joins a hotel."
                                    </p>
                                }.into_any();
                            }
                            view! {
                                <div>
                                    <dl class="mb-4 flex flex-col gap-2 text-sm">
                                        <Row label="Account type" value=u.user_type.clone().unwrap_or_default() />
                                        <Row label="Role" value=u.role_label() />
                                        <Row label="Status" value=u.status.clone().unwrap_or_default() />
                                        <Row label="Member since" value=api::pretty_date(u.date_joined.as_deref()) />
                                    </dl>
                                    <div class="flex flex-wrap gap-1.5 border-t border-slate-100 pt-4">
                                        {u.permissions.clone().into_iter().map(|p| view! {
                                            <span class="rounded-md bg-slate-100 px-2 py-0.5 text-2xs font-medium text-slate-600">{p}</span>
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </Card>

                <Card title="Danger zone">
                    <p class="text-sm text-slate-600">
                        "Deactivating your account signs you out and removes your access to this property. Reservation and guest records stay intact."
                    </p>
                    <button
                        on:click=deactivate
                        class="mt-4 flex w-full items-center justify-center gap-2 rounded-lg border border-red-300 py-2.5 text-sm font-semibold text-red-700 transition-colors hover:bg-red-50"
                    >
                        <Icon name="user-x" class="h-4 w-4" />
                        "Deactivate my account"
                    </button>
                </Card>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Activity
// ---------------------------------------------------------------------------

#[component]
fn ActivityTab() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let logs = LocalResource::new(move || {
        let q = search.get();
        async move { api::list_audit_logs(&q, "").await }
    });

    view! {
        <div class="animate-fade-up">
            <div class="mb-4 relative">
                <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="text"
                    placeholder="Search by action, description, user or IP address"
                    class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                    prop:value=search
                    on:input:target=move |ev| search.set(ev.target().value())
                />
            </div>

            <Suspense fallback=|| view! {
                <p class="py-16 text-center text-sm text-slate-400">"Loading activity…"</p>
            }>
                {move || Suspend::new(async move {
                    let rows = match logs.await {
                        Ok(p) => p.items,
                        Err(e) => return view! {
                            <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                        }.into_any(),
                    };
                    if rows.is_empty() {
                        return view! {
                            <div class="rounded-xl border border-slate-200 bg-white">
                                <EmptyState
                                    icon="activity"
                                    title="No activity recorded"
                                    body="Signing in, changing rates, adding staff and processing checkouts all leave an entry here."
                                />
                            </div>
                        }.into_any();
                    }
                    view! {
                        <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                            <table class="w-full text-left text-sm">
                                <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                    <tr>
                                        <th class="px-4 py-3">"Action"</th>
                                        <th class="px-4 py-3">"Detail"</th>
                                        <th class="px-4 py-3">"By"</th>
                                        <th class="px-4 py-3">"IP"</th>
                                        <th class="px-4 py-3">"When"</th>
                                        <th class="px-4 py-3">"Result"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|l| view! {
                                        <tr class="border-b border-slate-100 last:border-0 hover:bg-slate-50">
                                            <td class="px-4 py-3 font-medium text-slate-900">{l.action_label()}</td>
                                            <td class="px-4 py-3 max-w-md text-slate-600">{l.description()}</td>
                                            <td class="px-4 py-3 text-slate-600">{l.actor()}</td>
                                            <td class="px-4 py-3 text-2xs tabular-nums text-slate-400">
                                                {l.ip_address.clone().unwrap_or_default()}
                                            </td>
                                            <td class="px-4 py-3 whitespace-nowrap text-slate-500">
                                                {api::pretty_datetime(l.created_at.as_deref())}
                                            </td>
                                            <td class="px-4 py-3">
                                                <Badge
                                                    label=l.status.clone()
                                                    tone=if l.succeeded() { "green" } else { "red" }
                                                />
                                            </td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

#[component]
fn Field(
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
fn NotifRow(
    icon: &'static str,
    label: &'static str,
    hint: &'static str,
    get: Signal<bool>,
    set: impl Fn(bool) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <label class="flex cursor-pointer items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2.5">
            <span class="flex min-w-0 items-center gap-2.5">
                <Icon name=icon class="h-4 w-4 shrink-0 text-slate-400" />
                <span class="min-w-0">
                    <span class="block text-sm font-medium text-slate-800">{label}</span>
                    <span class="block text-2xs text-slate-500">{hint}</span>
                </span>
            </span>
            <input type="checkbox" class="h-4 w-4 shrink-0" prop:checked=move || get.get()
                on:change:target=move |ev| set(ev.target().checked()) />
        </label>
    }
}
