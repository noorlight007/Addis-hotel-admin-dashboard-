//! Staff and roles, backed by `/organizations/{id}/staff/` and
//! `/roles-permissions/roles/`.
//!
//! Adding a staff member creates the account outright — the API has no
//! invitation flow, so the form sets an initial password the person changes
//! after signing in. Roles are the platform's system roles plus any custom role
//! the hotel has created; changing someone's role is a PATCH on their staff row.

use crate::api;
use crate::components::{
    use_toast, Avatar, Badge, EmptyState, Icon, Modal, PageHeader, SegmentedControl, StatCard,
};
use leptos::prelude::*;

const TABS: &[&str] = &["Team", "Roles"];

/// Deterministic avatar tint so a person keeps the same colour across the app.
fn gradient_for(initials: &str) -> &'static str {
    const PALETTE: [&str; 6] = [
        "from-blue-600 to-indigo-700",
        "from-emerald-600 to-teal-700",
        "from-amber-500 to-orange-600",
        "from-purple-600 to-fuchsia-700",
        "from-rose-500 to-red-600",
        "from-cyan-600 to-sky-700",
    ];
    let sum: usize = initials.bytes().map(|b| b as usize).sum();
    PALETTE[sum % PALETTE.len()]
}

#[component]
pub fn StaffPage() -> impl IntoView {
    let toast = use_toast();
    let tab = RwSignal::new(TABS[0]);
    let search = RwSignal::new(String::new());
    let add_open = RwSignal::new(false);
    let reset_for = RwSignal::new(Option::<api::StaffMember>::None);
    let refresh = RwSignal::new(0u32);

    let staff = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_staff().await }
    });
    let roles = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_roles().await }
    });

    let team = move || staff.get().and_then(Result::ok).unwrap_or_default();
    let role_list = move || roles.get().and_then(Result::ok).unwrap_or_default();

    let visible = move || {
        let q = search.get().to_lowercase();
        team()
            .into_iter()
            .filter(|m| {
                q.is_empty()
                    || m.name().to_lowercase().contains(&q)
                    || m.role_str().to_lowercase().contains(&q)
                    || m.email.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    let bump = move || refresh.update(|n| *n += 1);
    let me_id = api::session::user().and_then(|u| u.id).unwrap_or(0);

    let change_role = move |user_id: i64, role_id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_staff_role(user_id, role_id).await {
                Ok(m) => {
                    toast.success("Role updated", format!("{} is now {}.", m.name(), m.role_str()));
                    bump();
                }
                Err(e) => toast.error("Could not change the role", e.detail()),
            }
        });
    };

    let remove = move |m: api::StaffMember| {
        if !crate::confirm(&format!(
            "Remove {} from this property? They will lose access immediately.",
            m.name()
        )) {
            return;
        }
        let id = m.id;
        wasm_bindgen_futures::spawn_local(async move {
            match api::remove_staff(id).await {
                Ok(()) => {
                    toast.success("Staff removed", "They no longer have access to this hotel.");
                    bump();
                }
                Err(e) => toast.error("Could not remove them", e.detail()),
            }
        });
    };

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Staff & roles"
                subtitle="Who can sign in to this property, and what they can do."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| add_open.set(true)
                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-3 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                >
                    <Icon name="user-plus" class="h-4 w-4" />
                    <span class="hidden sm:inline">"Add staff"</span>
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <StatCard
                    icon="users"
                    label="Team members"
                    value=Signal::derive(move || team().len().to_string())
                    hint="With access to this hotel"
                />
                <StatCard
                    icon="check-circle"
                    label="Active"
                    value=Signal::derive(move || team().iter().filter(|m| m.is_active()).count().to_string())
                    hint="Can sign in now"
                    accent="text-emerald-600 bg-emerald-50"
                />
                <StatCard
                    icon="shield-check"
                    label="Roles available"
                    value=Signal::derive(move || role_list().len().to_string())
                    hint="System and custom"
                    accent="text-purple-600 bg-purple-50"
                />
                <StatCard
                    icon="key"
                    label="Administrators"
                    value=Signal::derive(move || {
                        team().iter().filter(|m| m.role_str().contains("Admin")).count().to_string()
                    })
                    hint="Full control"
                    accent="text-amber-600 bg-amber-50"
                />
            </div>

            // ================= TEAM =================
            <Show when=move || (tab.get() == "Team")>
                <div class="animate-fade-up">
                    <div class="mb-4 relative">
                        <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                        <input
                            type="text"
                            placeholder="Search by name, email or role"
                            class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                            prop:value=search
                            on:input:target=move |ev| search.set(ev.target().value())
                        />
                    </div>

                    <Suspense fallback=|| view! {
                        <p class="py-16 text-center text-sm text-slate-400">"Loading team…"</p>
                    }>
                        {move || Suspend::new(async move {
                            if let Err(e) = staff.await {
                                return view! {
                                    <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                                }.into_any();
                            }
                            let rows = visible();
                            if rows.is_empty() {
                                return view! {
                                    <div class="rounded-xl border border-slate-200 bg-white">
                                        <EmptyState
                                            icon="users"
                                            title="Nobody matches"
                                            body="No staff member matches this search. Add someone to give them access to this property."
                                        />
                                    </div>
                                }.into_any();
                            }
                            let available = role_list();

                            view! {
                                <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                                    <table class="w-full text-left text-sm">
                                        <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                            <tr>
                                                <th class="px-4 py-3">"Name"</th>
                                                <th class="px-4 py-3">"Contact"</th>
                                                <th class="px-4 py-3">"Role"</th>
                                                <th class="px-4 py-3">"Status"</th>
                                                <th class="px-4 py-3">"Added"</th>
                                                <th class="px-4 py-3">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows.into_iter().map(|m| {
                                                let initials = m.initials();
                                                let uid = m.id;
                                                let is_me = uid == me_id;
                                                let for_reset = m.clone();
                                                let for_remove = m.clone();
                                                let current_role = m.role_id.unwrap_or(0);
                                                let opts = available.clone();
                                                view! {
                                                    <tr class="border-b border-slate-100 last:border-0 hover:bg-slate-50">
                                                        <td class="px-4 py-3">
                                                            <div class="flex items-center gap-2.5">
                                                                <Avatar initials=initials.clone() size="h-9 w-9 text-2xs" gradient=gradient_for(&initials) />
                                                                <div class="min-w-0">
                                                                    <p class="flex items-center gap-1.5 truncate font-medium text-slate-900">
                                                                        {m.name()}
                                                                        {is_me.then(|| view! {
                                                                            <span class="rounded bg-slate-100 px-1 py-0.5 text-[9px] font-bold text-slate-500">"YOU"</span>
                                                                        })}
                                                                    </p>
                                                                    <p class="truncate text-2xs text-slate-400">
                                                                        {if m.is_system_role { "System role" } else { "Custom role" }}
                                                                    </p>
                                                                </div>
                                                            </div>
                                                        </td>
                                                        <td class="px-4 py-3 text-slate-600">
                                                            {m.email.clone().unwrap_or_else(|| "—".into())}
                                                            <span class="mt-0.5 block text-2xs text-slate-400">
                                                                {m.phone.clone().unwrap_or_default()}
                                                            </span>
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            {if opts.is_empty() || is_me {
                                                                view! { <span class="font-medium text-slate-700">{m.role_str().to_string()}</span> }.into_any()
                                                            } else {
                                                                view! {
                                                                    <select
                                                                        class="rounded-lg border border-slate-300 px-2 py-1.5 text-xs"
                                                                        on:change:target=move |ev| {
                                                                            if let Ok(rid) = ev.target().value().parse::<i64>() {
                                                                                if rid != current_role {
                                                                                    change_role(uid, rid);
                                                                                }
                                                                            }
                                                                        }
                                                                    >
                                                                        {opts.into_iter().map(|r| view! {
                                                                            <option value=r.id.to_string() selected=(r.id == current_role)>{r.name}</option>
                                                                        }).collect_view()}
                                                                    </select>
                                                                }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <Badge
                                                                label=m.status_str().to_string()
                                                                tone=if m.is_active() { "green" } else { "slate" }
                                                            />
                                                        </td>
                                                        <td class="px-4 py-3 whitespace-nowrap text-slate-500">
                                                            {api::pretty_date(m.created_at.as_deref())}
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <div class="flex items-center gap-2 text-slate-400">
                                                                <button
                                                                    title="Reset password"
                                                                    class="hover:text-slate-700"
                                                                    on:click={
                                                                        let m = for_reset.clone();
                                                                        move |_| reset_for.set(Some(m.clone()))
                                                                    }
                                                                >
                                                                    <Icon name="key" class="h-4 w-4" />
                                                                </button>
                                                                <Show when=move || !is_me>
                                                                    <button
                                                                        title="Remove from property"
                                                                        class="hover:text-red-600"
                                                                        on:click={
                                                                            let m = for_remove.clone();
                                                                            move |_| remove(m.clone())
                                                                        }
                                                                    >
                                                                        <Icon name="user-x" class="h-4 w-4" />
                                                                    </button>
                                                                </Show>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </Show>

            // ================= ROLES =================
            <Show when=move || (tab.get() == "Roles")>
                <div class="animate-fade-up">
                    <Suspense fallback=|| view! {
                        <p class="py-16 text-center text-sm text-slate-400">"Loading roles…"</p>
                    }>
                        {move || Suspend::new(async move {
                            let rows = match roles.await {
                                Ok(r) => r,
                                Err(e) => return view! {
                                    <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                                }.into_any(),
                            };
                            if rows.is_empty() {
                                return view! {
                                    <div class="rounded-xl border border-slate-200 bg-white">
                                        <EmptyState icon="shield-check" title="No roles" body="The platform has not defined any roles for this property." />
                                    </div>
                                }.into_any();
                            }
                            let team_now = team();
                            view! {
                                <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                                    {rows.into_iter().map(|r| {
                                        let holders = team_now
                                            .iter()
                                            .filter(|m| m.role_id == Some(r.id))
                                            .count();
                                        view! {
                                            <div class="rounded-xl border border-slate-200 bg-white p-5">
                                                <div class="mb-2 flex items-start justify-between gap-2">
                                                    <h3 class="font-bold text-slate-900">{r.name.clone()}</h3>
                                                    <Badge
                                                        label=if r.is_system_role { "System" } else { "Custom" }
                                                        tone=if r.is_system_role { "blue" } else { "purple" }
                                                    />
                                                </div>
                                                <p class="mb-3 text-sm text-slate-500">
                                                    {r.description.clone().unwrap_or_else(|| "No description.".into())}
                                                </p>
                                                <p class="mb-3 text-2xs font-semibold uppercase tracking-wide text-slate-400">
                                                    {format!("{holders} member(s) · {} permission(s)", r.permissions.len())}
                                                </p>
                                                <div class="flex flex-wrap gap-1.5">
                                                    {r.permissions.into_iter().map(|p| view! {
                                                        <span class="rounded-md bg-slate-100 px-2 py-0.5 text-2xs font-medium text-slate-600">{p}</span>
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </Show>

            <Show when=move || add_open.get()>
                <AddStaffModal
                    roles=Signal::derive(role_list)
                    on_close=move || add_open.set(false)
                    on_added=move || {
                        toast.success("Staff added", "They can sign in with the password you set.");
                        bump();
                    }
                />
            </Show>

            <Show when=move || reset_for.get().is_some()>
                {move || reset_for.get().map(|m| view! {
                    <ResetPasswordModal
                        member=m
                        on_close=move || reset_for.set(None)
                    />
                })}
            </Show>
        </div>
    }
}

/// Creates a staff account on this property.
///
/// The API creates the user immediately with the password given — there is no
/// invitation email — so the form makes that explicit.
#[component]
fn AddStaffModal(
    roles: Signal<Vec<api::Role>>,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_added: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let role_id = RwSignal::new(0i64);
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    // Default to the least-privileged role the property has.
    Effect::new(move |_| {
        if role_id.get() == 0 {
            let list = roles.get();
            let pick = list
                .iter()
                .min_by_key(|r| r.permissions.len())
                .or_else(|| list.first());
            if let Some(r) = pick {
                role_id.set(r.id);
            }
        }
    });

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        let mail = email.get().trim().to_string();
        if mail.is_empty() && phone.get().trim().is_empty() {
            error.set(Some("Enter an email address or a phone number.".into()));
            return;
        }
        if password.get().len() < 6 {
            error.set(Some("The password must be at least 6 characters.".into()));
            return;
        }
        if role_id.get() == 0 {
            error.set(Some("Pick a role.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let payload = api::NewStaff {
            first_name: first.get().trim().to_string(),
            last_name: last.get().trim().to_string(),
            email: mail,
            phone: phone.get().trim().to_string(),
            password: password.get(),
            role_id: role_id.get(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::add_staff(&payload).await {
                Ok(_) => {
                    busy.set(false);
                    on_added();
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
        <Modal title="Add staff member" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="grid grid-cols-2 gap-4">
                    <TextField label="First name" value=first />
                    <TextField label="Last name" value=last />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <TextField label="Work email" value=email kind="email" />
                    <TextField label="Phone" value=phone kind="tel" />
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Role"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| role_id.set(ev.target().value().parse().unwrap_or(0))
                    >
                        {move || roles.get().into_iter().map(|r| view! {
                            <option value=r.id.to_string() selected=(r.id == role_id.get())>
                                {format!("{} · {} permissions", r.name, r.permissions.len())}
                            </option>
                        }).collect_view()}
                    </select>
                </div>

                <TextField label="Initial password" value=password kind="password" />

                <p class="rounded-lg bg-blue-50 px-3 py-2 text-2xs text-blue-900">
                    "The API creates the account straight away — there is no invitation email. Share this password with them privately and ask them to change it after their first sign-in."
                </p>

                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Creating…" } else { "Add staff" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn ResetPasswordModal(
    member: api::StaffMember,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = member.id;
    let name = member.name();
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if password.get().len() < 6 {
            error.set(Some("The password must be at least 6 characters.".into()));
            return;
        }
        error.set(None);
        busy.set(true);
        let pw = password.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::reset_staff_password(id, &pw).await {
                Ok(()) => {
                    busy.set(false);
                    toast.success("Password reset", "Share the new password with them privately.");
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
        <Modal title="Reset password" on_close=on_close>
            <form on:submit=submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>
                <p class="text-sm text-slate-600">{format!("Set a new password for {name}.")}</p>
                <TextField label="New password" value=password kind="password" />
                <div class="mt-1 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                        {move || if busy.get() { "Resetting…" } else { "Reset password" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn TextField(
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
