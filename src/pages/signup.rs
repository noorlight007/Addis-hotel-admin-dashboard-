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

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if full_name.get().trim().is_empty() || email.get().trim().is_empty() || password.get().is_empty() {
            error.set(Some("Please fill in your name, email and password.".to_string()));
            return;
        }
        if password.get() != confirm_password.get() {
            error.set(Some("Passwords do not match.".to_string()));
            return;
        }
        if !agree.get() {
            error.set(Some("Please agree to the Terms & Conditions to continue.".to_string()));
            return;
        }
        error.set(None);
        navigate("/", Default::default());
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
                        class="mt-1 flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98]"
                    >
                        "Create Account"
                        <Icon name="chevron-right" class="h-4 w-4" />
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
