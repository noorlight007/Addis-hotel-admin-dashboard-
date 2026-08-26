use crate::components::{Icon, Modal, StatCard, Toggle};
use crate::data::{Admin, ADMINS};
use crate::pages::dashboard::status_pill;
use leptos::prelude::*;

const TABS: &[&str] = &["Profile", "Security", "Account", "Admin Management"];

const ROLES: &[(&str, &str, &str, &str, &str)] = &[
    ("shield-check", "System Administrator", "Full system access and configuration control.", "3 users", "text-purple-700 bg-purple-50"),
    ("building", "Hotel Manager", "Manage hotel operations, rooms, and staff.", "5 users", "text-blue-700 bg-blue-50"),
    ("calendar-check", "Reservation Manager", "Manage reservations, bookings, and guests.", "4 users", "text-emerald-700 bg-emerald-50"),
    ("banknote", "Finance Admin", "Manage payments, invoices, and financial reports.", "2 users", "text-amber-700 bg-amber-50"),
    ("headset", "Support Staff", "Handle guest support and inquiries.", "2 users", "text-cyan-700 bg-cyan-50"),
    ("file-text", "Content Manager", "Manage content, pages, and media.", "2 users", "text-fuchsia-700 bg-fuchsia-50"),
];

const ROLE_NAMES: &[&str] = &[
    "System Administrator", "Hotel Manager", "Reservation Manager",
    "Finance Admin", "Support Staff", "Content Manager",
];

#[derive(Debug, Clone, Copy, PartialEq)]
struct ActivityLog {
    admin: &'static str,
    action: &'static str,
    module: &'static str,
    time: &'static str,
    result: &'static str,
}

const ACTIVITY_LOGS: &[ActivityLog] = &[
    ActivityLog { admin: "Ahmed Hassan", action: "Updated role for Mekdes Alemu", module: "Admin Management", time: "31 May 2024, 10:42 AM", result: "Success" },
    ActivityLog { admin: "Bereket Tesfaye", action: "Edited reservation #BK-2024-1045", module: "Reservations", time: "31 May 2024, 09:18 AM", result: "Success" },
    ActivityLog { admin: "Mekdes Alemu", action: "Added new admin: Samuel Assefa", module: "Admin Management", time: "30 May 2024, 04:33 PM", result: "Success" },
    ActivityLog { admin: "Samuel Assefa", action: "Logged in to the system", module: "Authentication", time: "30 May 2024, 02:11 AM", result: "Success" },
    ActivityLog { admin: "Yordanos Kebede", action: "Reset password for Helen Bekele", module: "Security", time: "29 May 2024, 11:27 AM", result: "Success" },
];

#[component]
pub fn SettingsPage() -> impl IntoView {
    let active_tab = RwSignal::new("Admin Management");
    let admins = RwSignal::new(ADMINS.to_vec());
    let modal_open = RwSignal::new(false);
    let admin_search = RwSignal::new(String::new());
    let admin_role_filter = RwSignal::new("All Roles".to_string());
    let admin_status_filter = RwSignal::new("All Statuses".to_string());
    let log_search = RwSignal::new(String::new());
    let log_module_filter = RwSignal::new("All Modules".to_string());
    let log_result_filter = RwSignal::new("All Results".to_string());

    let visible_admins = move || {
        let q = admin_search.get().to_lowercase();
        admins
            .get()
            .into_iter()
            .filter(|a| admin_role_filter.get() == "All Roles" || a.role == admin_role_filter.get())
            .filter(|a| admin_status_filter.get() == "All Statuses" || a.status == admin_status_filter.get())
            .filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q) || a.email.to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    };

    let visible_logs = move || {
        let q = log_search.get().to_lowercase();
        ACTIVITY_LOGS
            .iter()
            .copied()
            .filter(|l| log_module_filter.get() == "All Modules" || l.module == log_module_filter.get())
            .filter(|l| log_result_filter.get() == "All Results" || l.result == log_result_filter.get())
            .filter(|l| q.is_empty() || l.admin.to_lowercase().contains(&q) || l.action.to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <h1 class="text-xl font-bold text-slate-900">"Settings"</h1>
                <div class="flex gap-2">
                    <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50">
                        <Icon name="shield-check" class="h-4 w-4" />
                        "Manage Roles"
                    </button>
                    <button
                        on:click=move |_| modal_open.set(true)
                        class="flex items-center gap-1.5 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-3 py-2 text-sm font-semibold text-white"
                    >
                        <Icon name="plus" class="h-4 w-4" />
                        "Add New Admin"
                    </button>
                </div>
            </div>

            <div class="mb-6 flex gap-6 border-b border-slate-200 text-sm font-medium text-slate-500">
                {TABS.iter().map(|t| {
                    let t = *t;
                    view! {
                        <button
                            class=move || format!(
                                "-mb-px border-b-2 pb-2 {}",
                                if active_tab.get() == t { "border-blue-700 text-blue-700" } else { "border-transparent" }
                            )
                            on:click=move |_| active_tab.set(t)
                        >
                            {t}
                        </button>
                    }
                }).collect_view()}
            </div>

            {move || match active_tab.get() {
                "Admin Management" => view! {
                    <div>
                        <div class="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4">
                            <StatCard icon="users" label="Total Admins" value=Signal::derive(move || admins.get().len().to_string()) hint="All administrators" />
                            <StatCard icon="check-circle" label="Active Admins" value=Signal::derive(move || admins.get().iter().filter(|a| a.status == "Active").count().to_string()) hint="Currently active" accent="text-emerald-600 bg-emerald-50" />
                            <StatCard icon="shield-check" label="Roles" value="6".to_string() hint="System roles" accent="text-amber-600 bg-amber-50" />
                            <StatCard icon="mail" label="Pending Invites" value="2".to_string() hint="Invitations sent" accent="text-purple-600 bg-purple-50" />
                        </div>

                        <div class="mb-6 rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-1 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Roles & Permissions"</h2>
                                <button class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit Roles"</button>
                            </div>
                            <p class="mb-4 text-sm text-slate-500">"Manage admin roles and their permissions across the platform."</p>
                            <div class="grid gap-3 sm:grid-cols-3">
                                {ROLES.iter().map(|(icon, name, desc, count, accent)| view! {
                                    <div class="rounded-lg border border-slate-200 p-3">
                                        <span class=format!("mb-2 flex h-8 w-8 items-center justify-center rounded-full {accent}")><Icon name=*icon class="h-4 w-4" /></span>
                                        <p class="text-sm font-semibold text-slate-800">{*name}</p>
                                        <p class="text-xs text-slate-500">{*desc}</p>
                                        <p class="mt-2 text-xs font-medium text-slate-400">{*count}</p>
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>

                        <div class="mb-6 rounded-xl border border-slate-200 bg-white p-5">
                            <h2 class="mb-1 text-base font-semibold text-slate-900">"Admins List"</h2>
                            <p class="mb-4 text-sm text-slate-500">"Manage all platform administrators."</p>
                            <div class="mb-4 flex flex-wrap items-center gap-3">
                                <div class="relative flex-1">
                                    <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                    <input
                                        type="text"
                                        placeholder="Search admins..."
                                        class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                                        prop:value=admin_search
                                        on:input:target=move |ev| admin_search.set(ev.target().value())
                                    />
                                </div>
                                <select class="rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| admin_role_filter.set(ev.target().value())>
                                    <option value="All Roles">"All Roles"</option>
                                    {ROLE_NAMES.iter().map(|r| view! { <option value=*r>{*r}</option> }).collect_view()}
                                </select>
                                <select class="rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| admin_status_filter.set(ev.target().value())>
                                    <option value="All Statuses">"All Statuses"</option>
                                    <option value="Active">"Active"</option>
                                    <option value="Inactive">"Inactive"</option>
                                </select>
                                <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm transition-colors hover:bg-slate-50">
                                    <Icon name="sliders" class="h-4 w-4" />
                                    "Filters"
                                </button>
                            </div>

                        <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                            <table class="w-full text-left text-sm">
                                <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                    <tr>
                                        <th class="px-4 py-3">"Admin"</th>
                                        <th class="px-4 py-3">"Role"</th>
                                        <th class="px-4 py-3">"Email"</th>
                                        <th class="px-4 py-3">"Phone"</th>
                                        <th class="px-4 py-3">"Last Active"</th>
                                        <th class="px-4 py-3">"Status"</th>
                                        <th class="px-4 py-3">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || visible_admins().into_iter().map(|a| view! {
                                        <tr class="border-b border-slate-100 transition-colors duration-150 last:border-0 hover:bg-slate-50">
                                            <td class="px-4 py-3">
                                                <div class="flex items-center gap-2">
                                                    <span class="flex h-8 w-8 items-center justify-center rounded-full bg-blue-100 text-xs font-semibold text-blue-700">{a.initials}</span>
                                                    <span class="font-medium text-slate-900">{a.name}</span>
                                                </div>
                                            </td>
                                            <td class="px-4 py-3"><span class="rounded-full bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-600">{a.role}</span></td>
                                            <td class="px-4 py-3 text-slate-500">{a.email}</td>
                                            <td class="px-4 py-3 text-slate-500">{a.phone}</td>
                                            <td class="px-4 py-3 text-slate-500">{a.last_active}</td>
                                            <td class="px-4 py-3"><span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(a.status))>{a.status}</span></td>
                                            <td class="px-4 py-3">
                                                <div class="flex items-center gap-2 text-slate-400">
                                                    <button class="hover:text-slate-700"><Icon name="edit" class="h-4 w-4" /></button>
                                                    <button class="hover:text-slate-700"><Icon name="image" class="h-4 w-4" /></button>
                                                    <button class="hover:text-slate-700"><Icon name="more-v" class="h-4 w-4" /></button>
                                                </div>
                                            </td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                        <p class="mt-3 text-sm text-slate-500">{move || format!("Showing 1 to {} of {} admins", visible_admins().len(), admins.get().len())}</p>
                        </div>

                        <div class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-1 flex items-center justify-between">
                                <div>
                                    <h2 class="text-base font-semibold text-slate-900">"Admin Activity Logs"</h2>
                                    <p class="text-sm text-slate-500">"Track recent admin activities and system changes."</p>
                                </div>
                            </div>
                            <div class="mb-4 mt-3 flex flex-wrap items-center gap-3">
                                <div class="relative flex-1">
                                    <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                    <input
                                        type="text"
                                        placeholder="Search activity logs..."
                                        class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm"
                                        prop:value=log_search
                                        on:input:target=move |ev| log_search.set(ev.target().value())
                                    />
                                </div>
                                <select class="rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| log_module_filter.set(ev.target().value())>
                                    <option value="All Modules">"All Modules"</option>
                                    {["Admin Management", "Reservations", "Authentication", "Security"].iter().map(|m| view! { <option value=*m>{*m}</option> }).collect_view()}
                                </select>
                                <select class="rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| log_result_filter.set(ev.target().value())>
                                    <option value="All Results">"All Results"</option>
                                    <option value="Success">"Success"</option>
                                    <option value="Failed">"Failed"</option>
                                </select>
                                <span class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm text-slate-500">
                                    <Icon name="calendar" class="h-4 w-4" />
                                    "May 24 – May 31, 2024"
                                </span>
                            </div>
                            <table class="w-full text-left text-sm">
                                <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                    <tr><th class="py-2">"Admin"</th><th class="py-2">"Action"</th><th class="py-2">"Module"</th><th class="py-2">"Date & Time"</th><th class="py-2">"Result"</th></tr>
                                </thead>
                                <tbody class="text-slate-600">
                                    {move || visible_logs().into_iter().map(|log| view! {
                                        <tr class="border-b border-slate-100 last:border-0">
                                            <td class="py-2 font-medium text-slate-800">{log.admin}</td>
                                            <td class="py-2">{log.action}</td>
                                            <td class="py-2">{log.module}</td>
                                            <td class="py-2">{log.time}</td>
                                            <td class="py-2"><span class=format!("rounded-full px-2 py-0.5 text-xs font-semibold {}", status_pill(log.result))>{log.result}</span></td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                            <p class="mt-3 text-sm text-slate-500">{move || format!("Showing 1 to {} of {} logs", visible_logs().len(), ACTIVITY_LOGS.len())}</p>
                        </div>
                    </div>
                }.into_any(),
                "Security" => view! { <SecurityTab/> }.into_any(),
                "Account" => view! { <AccountTab/> }.into_any(),
                _ => view! { <ProfileTab/> }.into_any(),
            }}

            <Show when=move || modal_open.get()>
                <AddAdminModal
                    on_close=move || modal_open.set(false)
                    on_create=move |admin| admins.update(|a| a.insert(0, admin))
                />
            </Show>
        </div>
    }
}

#[component]
fn SavedBanner(saved: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=move || saved.get()>
            <div class="mb-4 flex animate-fade-in items-center gap-2 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                <Icon name="check-circle" class="h-4 w-4" />
                "Changes saved successfully."
            </div>
        </Show>
    }
}

fn flash_saved(saved: RwSignal<bool>) {
    saved.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(2500).await;
        saved.set(false);
    });
}

#[component]
fn ProfileTab() -> impl IntoView {
    let full_name = RwSignal::new("Ahmed Hassan".to_string());
    let email = RwSignal::new("ahmed.hassan@tourista.com".to_string());
    let phone = RwSignal::new("+251 91 234 5678".to_string());
    let job_title = RwSignal::new("System Administrator".to_string());
    let language = RwSignal::new("English".to_string());
    let timezone = RwSignal::new("(GMT+03:00) Addis Ababa".to_string());
    let bio = RwSignal::new("System Administrator responsible for managing the hotel management system, users, and system configuration.".to_string());
    let email_notif = RwSignal::new(true);
    let app_notif = RwSignal::new(true);
    let booking_notif = RwSignal::new(false);
    let system_notif = RwSignal::new(true);
    let saved = RwSignal::new(false);

    view! {
        <div>
            <SavedBanner saved=saved />
            <div class="rounded-xl border border-slate-200 bg-white p-5">
                <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                    <div>
                        <h2 class="text-base font-semibold text-slate-900">"Admin Profile"</h2>
                        <p class="text-sm text-slate-500">"Update your personal information and how it will appear in the system."</p>
                    </div>
                    <button class="flex items-center gap-1.5 rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium transition-colors hover:bg-slate-50">
                        <Icon name="camera" class="h-4 w-4" />
                        "Change Photo"
                    </button>
                </div>

                <div class="grid gap-4 sm:grid-cols-2">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=full_name on:input:target=move |ev| full_name.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Email Address"</label>
                        <input type="email" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=email on:input:target=move |ev| email.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone Number"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Job Title"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=job_title on:input:target=move |ev| job_title.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Preferred Language"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| language.set(ev.target().value())>
                            {["English", "Amharic", "French", "Arabic"].iter().map(|l| view! { <option value=*l selected=*l == language.get_untracked()>{*l}</option> }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Timezone"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| timezone.set(ev.target().value())>
                            {["(GMT+03:00) Addis Ababa", "(GMT+00:00) UTC", "(GMT+01:00) Lagos"].iter().map(|t| view! { <option value=*t selected=*t == timezone.get_untracked()>{*t}</option> }).collect_view()}
                        </select>
                    </div>
                </div>

                <div class="mt-4">
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Bio"</label>
                    <textarea rows="2" class="w-full rounded-lg border border-slate-300 p-2 text-sm" prop:value=bio on:input:target=move |ev| bio.set(ev.target().value())></textarea>
                    <p class="mt-1 text-right text-xs text-slate-400">{move || format!("{} / 250", bio.get().len())}</p>
                </div>
            </div>

            <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-3 text-base font-semibold text-slate-900">"Notification Preferences"</h2>
                <div class="flex flex-col divide-y divide-slate-100">
                    <NotifRow icon="mail" label="Email Notifications" hint="Receive important updates and alerts via email." checked=email_notif />
                    <NotifRow icon="bell" label="In-app Notifications" hint="Show notifications in the system." checked=app_notif />
                    <NotifRow icon="calendar" label="Booking Alerts" hint="Receive alerts for new bookings and changes." checked=booking_notif />
                    <NotifRow icon="shield-check" label="System Updates" hint="Get notified about system updates and maintenance." checked=system_notif />
                </div>
            </div>

            <div class="mt-4 flex justify-end gap-3">
                <button class="rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium transition-colors hover:bg-slate-50">"Reset Changes"</button>
                <button
                    on:click=move |_| flash_saved(saved)
                    class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]"
                >
                    "Save Changes"
                </button>
            </div>
        </div>
    }
}

#[component]
fn NotifRow(icon: &'static str, label: &'static str, hint: &'static str, checked: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-3">
            <label class="flex items-start gap-3">
                <input type="checkbox" prop:checked=checked on:change:target=move |ev| checked.set(ev.target().checked()) class="mt-0.5 h-4 w-4 rounded border-slate-300 text-blue-600" />
                <span>
                    <span class="flex items-center gap-1.5 text-sm font-medium text-slate-800"><Icon name=icon class="h-3.5 w-3.5 text-slate-400" />{label}</span>
                    <span class="block text-xs text-slate-400">{hint}</span>
                </span>
            </label>
        </div>
    }
}

#[component]
fn SecurityTab() -> impl IntoView {
    let current_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let two_factor = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let saved = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if current_password.get().is_empty() || new_password.get().is_empty() {
            error.set(Some("Please fill in your current and new password.".to_string()));
            return;
        }
        if new_password.get() != confirm_password.get() {
            error.set(Some("New passwords do not match.".to_string()));
            return;
        }
        error.set(None);
        current_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
        flash_saved(saved);
    };

    view! {
        <div>
            <SavedBanner saved=saved />
            <div class="rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-1 text-base font-semibold text-slate-900">"Change Password"</h2>
                <p class="mb-4 text-sm text-slate-500">"Use a strong password you don't use elsewhere."</p>

                <Show when=move || error.get().is_some()>
                    <div class="mb-4 flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <form on:submit=on_submit class="flex flex-col gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Current Password"</label>
                        <input type="password" class="w-full max-w-sm rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=current_password on:input:target=move |ev| current_password.set(ev.target().value()) />
                    </div>
                    <div class="grid max-w-lg gap-4 sm:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"New Password"</label>
                            <input type="password" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=new_password on:input:target=move |ev| new_password.set(ev.target().value()) />
                        </div>
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Confirm New Password"</label>
                            <input type="password" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=confirm_password on:input:target=move |ev| confirm_password.set(ev.target().value()) />
                        </div>
                    </div>
                    <div>
                        <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Update Password"</button>
                    </div>
                </form>
            </div>

            <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="text-base font-semibold text-slate-900">"Two-Factor Authentication"</h2>
                        <p class="text-sm text-slate-500">"Add an extra layer of security to your account using an authenticator app."</p>
                    </div>
                    <button on:click=move |_| two_factor.update(|v| *v = !*v)>
                        <Toggle checked=two_factor.get() />
                    </button>
                </div>
                <Show when=move || two_factor.get()>
                    <div class="mt-3 flex items-center gap-2 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                        <Icon name="check-circle" class="h-4 w-4" />
                        "Two-factor authentication is enabled."
                    </div>
                </Show>
            </div>

            <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-1 text-base font-semibold text-slate-900">"Active Sessions"</h2>
                <p class="mb-4 text-sm text-slate-500">"Devices currently signed in to your account."</p>
                <div class="flex flex-col divide-y divide-slate-100">
                    {[
                        ("MacBook Pro · Addis Ababa, Ethiopia", "Active now", true),
                        ("iPhone 15 · Bole, Addis Ababa", "2 hours ago", false),
                        ("Chrome on Windows · Nairobi, Kenya", "3 days ago", false),
                    ].iter().map(|(device, last, is_current)| view! {
                        <div class="flex items-center justify-between py-3">
                            <div class="flex items-center gap-3">
                                <span class="flex h-9 w-9 items-center justify-center rounded-full bg-slate-100 text-slate-500"><Icon name="monitor" class="h-4 w-4" /></span>
                                <div>
                                    <p class="text-sm font-medium text-slate-800">{*device}</p>
                                    <p class="text-xs text-slate-400">{*last}</p>
                                </div>
                            </div>
                            {if *is_current {
                                view! { <span class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-semibold text-emerald-700">"This device"</span> }.into_any()
                            } else {
                                view! { <button class="text-sm font-medium text-red-600 hover:underline">"Sign out"</button> }.into_any()
                            }}
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn AccountTab() -> impl IntoView {
    let saved = RwSignal::new(false);
    let two_factor_reminder = RwSignal::new(true);

    view! {
        <div>
            <SavedBanner saved=saved />
            <div class="rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-1 text-base font-semibold text-slate-900">"Account Overview"</h2>
                <p class="mb-4 text-sm text-slate-500">"General information about your Tourista account."</p>
                <dl class="grid gap-4 text-sm sm:grid-cols-2">
                    <div><dt class="text-slate-400">"Account Type"</dt><dd class="font-medium">"Hotel Administrator"</dd></div>
                    <div><dt class="text-slate-400">"Member Since"</dt><dd class="font-medium">"05 Mar 2023"</dd></div>
                    <div><dt class="text-slate-400">"Linked Hotel"</dt><dd class="font-medium">"Golden Tulip Addis Ababa"</dd></div>
                    <div><dt class="text-slate-400">"Login Method"</dt><dd class="font-medium">"Email & Password"</dd></div>
                </dl>
            </div>

            <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                <h2 class="mb-1 text-base font-semibold text-slate-900">"Verification"</h2>
                <p class="mb-4 text-sm text-slate-500">"Keep your contact details verified for account recovery."</p>
                <div class="flex flex-col divide-y divide-slate-100">
                    <div class="flex items-center justify-between py-3">
                        <span class="flex items-center gap-2 text-sm text-slate-700"><Icon name="mail" class="h-4 w-4 text-slate-400" />"ahmed.hassan@tourista.com"</span>
                        <span class="flex items-center gap-1 rounded-full bg-green-100 px-2 py-0.5 text-xs font-semibold text-emerald-700"><Icon name="check-circle" class="h-3 w-3" />"Verified"</span>
                    </div>
                    <div class="flex items-center justify-between py-3">
                        <span class="flex items-center gap-2 text-sm text-slate-700"><Icon name="phone" class="h-4 w-4 text-slate-400" />"+251 91 234 5678"</span>
                        <button on:click=move |_| flash_saved(saved) class="text-sm font-medium text-blue-700 hover:underline">"Verify Now"</button>
                    </div>
                </div>
            </div>

            <div class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
                <label class="flex items-center justify-between">
                    <span>
                        <span class="block text-sm font-medium text-slate-800">"Security reminders"</span>
                        <span class="block text-xs text-slate-400">"Remind me to enable two-factor authentication."</span>
                    </span>
                    <button type="button" on:click=move |_| two_factor_reminder.update(|v| *v = !*v)>
                        <Toggle checked=two_factor_reminder.get() />
                    </button>
                </label>
            </div>

            <div class="mt-4 rounded-xl border border-red-200 bg-red-50/40 p-5">
                <h2 class="mb-1 text-base font-semibold text-red-700">"Danger Zone"</h2>
                <p class="mb-4 text-sm text-red-600/80">"These actions are irreversible. Please proceed with caution."</p>
                <div class="flex flex-wrap gap-3">
                    <button class="rounded-lg border border-red-300 bg-white px-4 py-2 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50">"Deactivate My Account"</button>
                    <button class="rounded-lg border border-red-300 bg-white px-4 py-2 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50">"Request Data Export"</button>
                </div>
            </div>
        </div>
    }
}

fn initials_of(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
fn AddAdminModal(
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    on_create: impl Fn(Admin) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let role = RwSignal::new(ROLE_NAMES[1].to_string());
    let send_invite = RwSignal::new(true);
    let error = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if name.get().trim().is_empty() || email.get().trim().is_empty() {
            error.set(Some("Please enter the admin's name and email.".to_string()));
            return;
        }
        let admin = Admin {
            initials: Box::leak(initials_of(&name.get()).into_boxed_str()),
            name: Box::leak(name.get().into_boxed_str()),
            role: Box::leak(role.get().into_boxed_str()),
            email: Box::leak(email.get().into_boxed_str()),
            phone: Box::leak((if phone.get().trim().is_empty() { "—".to_string() } else { phone.get() }).into_boxed_str()),
            last_active: if send_invite.get() { "Pending invite" } else { "Never" },
            status: if send_invite.get() { "Inactive" } else { "Active" },
        };
        on_create(admin);
        on_close();
    };

    view! {
        <Modal title="Add New Admin" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <Show when=move || error.get().is_some()>
                    <div class="flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                    <input type="text" placeholder="e.g. Selam Girma" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Email"</label>
                        <input type="email" placeholder="selam.girma@tourista.com" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=email on:input:target=move |ev| email.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone Number"</label>
                        <input type="text" placeholder="+251 91 234 5678" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                </div>

                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Role"</label>
                    <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                        on:change:target=move |ev| role.set(ev.target().value())
                    >
                        {ROLE_NAMES.iter().map(|r| view! { <option value=*r selected=*r == role.get_untracked()>{*r}</option> }).collect_view()}
                    </select>
                </div>

                <label class="flex items-center gap-2 text-sm text-slate-600">
                    <input type="checkbox" prop:checked=send_invite on:change:target=move |ev| send_invite.set(ev.target().checked()) class="h-4 w-4 rounded border-slate-300 text-blue-600" />
                    "Send email invitation to set up their password"
                </label>

                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Add Admin"</button>
                </div>
            </form>
        </Modal>
    }
}
