use crate::api;
use crate::components::Icon;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SignupPage() -> impl IntoView {
    let navigate = use_navigate();
    let full_name = RwSignal::new(String::new());
    let hotel_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let agree = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        let name = full_name.get().trim().to_string();
        let mail = email.get().trim().to_string();
        let secret = password.get();
        if name.is_empty() || mail.is_empty() || secret.is_empty() {
            error.set(Some("Please fill in your name, email and password.".to_string()));
            return;
        }
        if secret != confirm_password.get() {
            error.set(Some("Passwords do not match.".to_string()));
            return;
        }
        if !agree.get() {
            error.set(Some("Please agree to the Terms & Conditions to continue.".to_string()));
            return;
        }
        error.set(None);
        busy.set(true);

        // The API takes given/family names separately; split on the first space
        // and let everything after it be the surname.
        let (first, last) = match name.split_once(' ') {
            Some((f, l)) => (f.to_string(), l.trim().to_string()),
            None => (name.clone(), String::new()),
        };
        let payload = api::RegisterPayload {
            email: mail.clone(),
            password: secret.clone(),
            first_name: first,
            last_name: last,
            phone: phone.get().trim().to_string(),
            // A hotel signing up needs to administer its own organization.
            user_type: "organization_admin".to_string(),
        };

        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::register(&payload).await {
                Ok(_) => {
                    // The register endpoint returns a token pair, so the account
                    // is already signed in. A fresh admin has no hotel yet, so
                    // land them on the profile page to create one.
                    busy.set(false);
                    navigate("/profile", Default::default());
                }
                Err(e) => {
                    // Surface the field the API objected to, when it names one.
                    let detail = ["email", "password", "phone", "first_name"]
                        .into_iter()
                        .find_map(|f| e.field(f).map(|m| format!("{f}: {m}")))
                        .unwrap_or_else(|| e.message.clone());
                    error.set(Some(detail));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-slate-900 to-blue-950 p-4">
            <div class="w-full max-w-lg animate-scale-in rounded-2xl bg-white p-8 shadow-2xl">
                <div class="mb-6 flex flex-col items-center text-center">
                    <span class="mb-3 flex h-14 w-14 items-center justify-center rounded-xl bg-blue-50 text-blue-700">
                        <Icon name="building" class="h-7 w-7" />
                    </span>
                    <h1 class="text-xl font-bold text-slate-900">"Create your hotel account"</h1>
                    <p class="text-sm text-slate-500">"Set up your property on Tourista in a few minutes."</p>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="mb-4 flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <form on:submit=on_submit class="flex flex-col gap-4">
                    <div class="grid gap-4 sm:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                            <div class="relative">
                                <Icon name="users" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="text"
                                    placeholder="Ahmed Hassan"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=full_name
                                    on:input:target=move |ev| full_name.set(ev.target().value())
                                />
                            </div>
                        </div>
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Hotel Name"</label>
                            <div class="relative">
                                <Icon name="building" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="text"
                                    placeholder="Golden Tulip Addis Ababa"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=hotel_name
                                    on:input:target=move |ev| hotel_name.set(ev.target().value())
                                />
                            </div>
                        </div>
                    </div>

                    <div class="grid gap-4 sm:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Email"</label>
                            <div class="relative">
                                <Icon name="mail" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="email"
                                    placeholder="ahmed.hassan@tourista.com"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=email
                                    on:input:target=move |ev| email.set(ev.target().value())
                                />
                            </div>
                        </div>
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Phone Number"</label>
                            <div class="relative">
                                <Icon name="phone" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="text"
                                    placeholder="+251 91 234 5678"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=phone
                                    on:input:target=move |ev| phone.set(ev.target().value())
                                />
                            </div>
                        </div>
                    </div>

                    <div class="grid gap-4 sm:grid-cols-2">
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Password"</label>
                            <div class="relative">
                                <Icon name="lock" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="password"
                                    placeholder="Create a password"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=password
                                    on:input:target=move |ev| password.set(ev.target().value())
                                />
                            </div>
                        </div>
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Confirm Password"</label>
                            <div class="relative">
                                <Icon name="lock" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                <input
                                    type="password"
                                    placeholder="Re-enter your password"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                    prop:value=confirm_password
                                    on:input:target=move |ev| confirm_password.set(ev.target().value())
                                />
                            </div>
                        </div>
                    </div>

                    <label class="flex items-start gap-2 text-sm text-slate-600">
                        <input type="checkbox" prop:checked=agree on:change:target=move |ev| agree.set(ev.target().checked()) class="mt-0.5 h-4 w-4 rounded border-slate-300 text-blue-600" />
                        <span>"I agree to the " <a href="#" class="font-medium text-blue-700 hover:underline">"Terms & Conditions"</a> " and " <a href="#" class="font-medium text-blue-700 hover:underline">"Privacy Policy"</a>"."</span>
                    </label>

                    <button
                        type="submit"
                        disabled=move || busy.get()
                        class="mt-1 flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-60"
                    >
                        {move || if busy.get() { "Creating account…" } else { "Create Account" }}
                        <Show when=move || !busy.get()>
                            <Icon name="chevron-right" class="h-4 w-4" />
                        </Show>
                    </button>
                </form>

                <p class="mt-6 text-center text-sm text-slate-500">
                    "Already have an account? "
                    <A href="/login" attr:class="font-semibold text-blue-700 hover:underline">"Log In"</A>
                </p>
            </div>
        </div>
    }
}
