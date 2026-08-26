use crate::components::{
    use_toast, Avatar, Badge, EmptyState, Icon, Modal, PageHeader, SegmentedControl, StatCard,
};
use crate::data::{ADMINS, ROLES};
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
    let invite_open = RwSignal::new(false);
    let removing = RwSignal::new(Option::<&'static str>::None);
    let removed = RwSignal::new(Vec::<&'static str>::new());

    let invite_email = RwSignal::new(String::new());
    let invite_role = RwSignal::new(ROLES[4].name.to_string());

    let visible = move || {
        let q = search.get().to_lowercase();
        ADMINS
            .iter()
            .filter(|a| !removed.get().contains(&a.email))
            .filter(|a| {
                q.is_empty()
                    || a.name.to_lowercase().contains(&q)
                    || a.role.to_lowercase().contains(&q)
                    || a.email.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    let active_count =
        move || ADMINS.iter().filter(|a| a.status == "Active" && !removed.get().contains(&a.email)).count();

    let send_invite = move |_| {
        let email = invite_email.get();
        if !email.contains('@') || !email.contains('.') {
            toast.error("Check the email", "Enter a valid work email address for the invitation.");
            return;
        }
        invite_open.set(false);
        toast.success("Invitation sent", format!("{email} has been invited as {}.", invite_role.get()));
        invite_email.set(String::new());
    };

    let confirm_remove = move |_| {
        if let Some(email) = removing.get() {
            removed.update(|list| list.push(email));
            removing.set(None);
            toast.success("Access revoked", format!("{email} can no longer sign in to this property."));
        }
    };

    view! {
        <div class="p-4 sm:p-6">
            <PageHeader
                title="Staff & roles"
                subtitle="Who can sign in to this property's dashboard, and what they can reach."
            >
                <SegmentedControl options=TABS selected=tab />
                <button
                    on:click=move |_| invite_open.set(true)
                    class="sheen flex items-center gap-1.5 rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 active:scale-95"
                >
                    <Icon name="user-plus" class="h-4 w-4" />
                    "Invite"
                </button>
            </PageHeader>

            <div class="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="animate-fade-up">
                    <StatCard
                        icon="users"
                        label="Team members"
                        value=Signal::derive(move || visible().len().to_string())
                        hint="With dashboard access"
                        accent="text-blue-600 bg-blue-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 60ms">
                    <StatCard
                        icon="check-circle"
                        label="Active"
                        value=Signal::derive(move || active_count().to_string())
                        hint="Signed in recently"
                        accent="text-emerald-600 bg-emerald-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 120ms">
                    <StatCard
                        icon="shield-check"
                        label="Roles defined"
                        value=ROLES.len().to_string()
                        hint="Permission templates"
                        accent="text-purple-600 bg-purple-50"
                    />
                </div>
                <div class="animate-fade-up" style="animation-delay: 180ms">
                    <StatCard
                        icon="lock"
                        label="2FA enrolled"
                        value="4".to_string()
                        hint="Of 6 members"
                        accent="text-amber-600 bg-amber-50"
                    />
                </div>
            </div>

            // ================= TEAM =================
            <Show when=move || (tab.get() == "Team")>
                <div class="animate-fade-up">
                    <div class="relative mb-4 max-w-md">
                        <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                        <input
                            type="text"
                            placeholder="Search by name, role or email"
                            class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                            prop:value=search
                            on:input:target=move |ev| search.set(ev.target().value())
                        />
                    </div>

                    <div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-card">
                        <div class="overflow-x-auto">
                            <table class="w-full min-w-[46rem] text-left text-sm">
                                <thead class="border-b border-slate-200 bg-slate-50/60 text-2xs uppercase tracking-wide text-slate-500">
                                    <tr>
                                        <th class="px-4 py-3">"Member"</th>
                                        <th class="px-4 py-3">"Role"</th>
                                        <th class="px-4 py-3">"Contact"</th>
                                        <th class="px-4 py-3">"Last active"</th>
                                        <th class="px-4 py-3">"Status"</th>
                                        <th class="px-4 py-3"></th>
                                    </tr>
                                </thead>
                                <tbody class="stagger">
                                    {move || visible().into_iter().map(|a| view! {
                                        <tr class="group border-b border-slate-100 transition-colors last:border-0 hover:bg-blue-50/40">
                                            <td class="px-4 py-3">
                                                <div class="flex items-center gap-3">
                                                    <Avatar initials=a.initials gradient=gradient_for(a.initials) />
                                                    <div class="min-w-0">
                                                        <p class="truncate font-semibold text-slate-900">{a.name}</p>
                                                        <p class="truncate text-2xs text-slate-400">{a.email}</p>
                                                    </div>
                                                </div>
                                            </td>
                                            <td class="px-4 py-3">
                                                <span class="flex items-center gap-1.5 text-slate-600">
                                                    <Icon name="shield-check" class="h-3.5 w-3.5 text-slate-400" />
                                                    {a.role}
                                                </span>
                                            </td>
                                            <td class="px-4 py-3 text-slate-600">{a.phone}</td>
                                            <td class="px-4 py-3 text-slate-500">{a.last_active}</td>
                                            <td class="px-4 py-3">
                                                <Badge
                                                    label=a.status
                                                    tone=if a.status == "Active" { "green" } else { "slate" }
                                                    dot=true
                                                />
                                            </td>
                                            <td class="px-4 py-3">
                                                <div class="flex items-center justify-end gap-1 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                                                    <button
                                                        aria-label="Edit member"
                                                        on:click=move |_| toast.info("Edit member", format!("Editing {} opens in the next release.", a.name))
                                                        class="rounded-lg p-1.5 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700"
                                                    >
                                                        <Icon name="edit" class="h-4 w-4" />
                                                    </button>
                                                    <button
                                                        aria-label="Revoke access"
                                                        on:click=move |_| removing.set(Some(a.email))
                                                        class="rounded-lg p-1.5 text-slate-400 transition-colors hover:bg-red-50 hover:text-red-600"
                                                    >
                                                        <Icon name="user-x" class="h-4 w-4" />
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        <Show when=move || visible().is_empty()>
                            <EmptyState
                                icon="users"
                                title="Nobody matches that search"
                                body="Try a different name, or invite a new member to the team."
                            >
                                <button
                                    on:click=move |_| invite_open.set(true)
                                    class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-800"
                                >
                                    "Invite a member"
                                </button>
                            </EmptyState>
                        </Show>
                    </div>
                </div>
            </Show>

            // ================= ROLES =================
            <Show when=move || (tab.get() == "Roles")>
                <div class="grid animate-fade-up gap-4 md:grid-cols-2 xl:grid-cols-3">
                    {ROLES.iter().enumerate().map(|(i, r)| {
                        let delay = format!("animation-delay: {}ms", i * 60);
                        view! {
                            <div
                                class="card-hover flex animate-fade-up flex-col rounded-xl border border-slate-200 bg-white p-5 shadow-card"
                                style=delay
                            >
                                <div class="flex items-start justify-between gap-3">
                                    <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-blue-50 text-blue-700">
                                        <Icon name="shield-check" class="h-5 w-5" />
                                    </span>
                                    <Badge
                                        label=format!("{} member{}", r.members, if r.members == 1 { "" } else { "s" })
                                        tone="blue"
                                    />
                                </div>

                                <h3 class="mt-3 text-base font-bold text-slate-900">{r.name}</h3>
                                <p class="mt-1 text-sm leading-relaxed text-slate-500">{r.description}</p>

                                <div class="mt-4 flex flex-1 flex-col justify-end">
                                    <p class="mb-2 text-2xs font-bold uppercase tracking-wide text-slate-400">
                                        "Can access"
                                    </p>
                                    <div class="flex flex-wrap gap-1.5">
                                        {r.permissions.iter().map(|p| view! {
                                            <span class="rounded-md bg-slate-100 px-2 py-1 text-2xs font-medium text-slate-600">
                                                {*p}
                                            </span>
                                        }).collect_view()}
                                    </div>
                                </div>

                                <button
                                    on:click=move |_| toast.info("Role permissions", format!("Editing the {} role opens in the next release.", r.name))
                                    class="mt-4 flex w-full items-center justify-center gap-1.5 rounded-lg border border-slate-300 py-2 text-xs font-semibold text-slate-700 transition-colors hover:border-blue-300 hover:text-blue-700"
                                >
                                    <Icon name="settings" class="h-3.5 w-3.5" />
                                    "Edit permissions"
                                </button>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>

        // ---- Invite ---------------------------------------------------
        <Show when=move || invite_open.get()>
            <Modal title="Invite a team member" on_close=move || invite_open.set(false)>
                <div class="flex flex-col gap-4">
                    <div>
                        <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">
                            "Work email"
                        </label>
                        <div class="relative">
                            <Icon name="mail" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                            <input
                                type="email"
                                placeholder="name@tourista.com"
                                class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                                prop:value=invite_email
                                on:input:target=move |ev| invite_email.set(ev.target().value())
                            />
                        </div>
                    </div>

                    <div>
                        <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">"Role"</label>
                        <select
                            class="w-full rounded-lg border border-slate-300 px-3 py-2.5 text-sm transition-colors focus:border-blue-500 focus:outline-none"
                            on:change:target=move |ev| invite_role.set(ev.target().value())
                        >
                            {ROLES.iter().map(|r| view! {
                                <option selected=move || (invite_role.get() == r.name)>{r.name}</option>
                            }).collect_view()}
                        </select>
                        <p class="mt-2 rounded-lg bg-slate-50 p-3 text-xs leading-relaxed text-slate-500">
                            {move || ROLES
                                .iter()
                                .find(|r| r.name == invite_role.get())
                                .map(|r| r.description)
                                .unwrap_or("")}
                        </p>
                    </div>

                    <div class="flex items-start gap-2.5 rounded-lg bg-blue-50 p-3 text-xs text-blue-900">
                        <Icon name="info" class="mt-0.5 h-3.5 w-3.5 shrink-0" />
                        <span>"They receive an email with a sign-in link valid for 48 hours. Two-factor authentication is required on first sign-in."</span>
                    </div>

                    <div class="flex gap-2.5">
                        <button
                            on:click=move |_| invite_open.set(false)
                            class="flex-1 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=send_invite
                            class="flex-1 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/25 transition-colors hover:bg-blue-800"
                        >
                            "Send invitation"
                        </button>
                    </div>
                </div>
            </Modal>
        </Show>

        // ---- Revoke ---------------------------------------------------
        <Show when=move || removing.get().is_some()>
            <Modal title="Revoke dashboard access?" width="max-w-md" on_close=move || removing.set(None)>
                <div class="flex items-start gap-3 rounded-xl bg-amber-50 p-4 text-sm text-amber-900 ring-1 ring-amber-100">
                    <Icon name="alert" class="mt-0.5 h-4 w-4 shrink-0" />
                    <span>
                        <span class="font-bold">{move || removing.get().unwrap_or("")}</span>
                        " will be signed out immediately and will no longer be able to reach this property. Their past actions stay in the audit log."
                    </span>
                </div>
                <div class="mt-5 flex gap-2.5">
                    <button
                        on:click=move |_| removing.set(None)
                        class="flex-1 rounded-lg border border-slate-300 py-2.5 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50"
                    >
                        "Keep access"
                    </button>
                    <button
                        on:click=confirm_remove
                        class="flex-1 rounded-lg bg-red-600 py-2.5 text-sm font-semibold text-white shadow-md shadow-red-600/25 transition-colors hover:bg-red-700"
                    >
                        "Revoke access"
                    </button>
                </div>
            </Modal>
        </Show>
    }
}
