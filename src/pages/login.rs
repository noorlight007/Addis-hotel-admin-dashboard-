use crate::api;
use crate::components::Icon;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember = RwSignal::new(true);
    let show_password = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    // Already signed in? Skip the form.
    Effect::new(move |_| {
        if api::session::is_authenticated() {
            navigate("/", Default::default());
        }
    });

    let submit_navigate = use_navigate();
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        let identifier = email.get().trim().to_string();
        let secret = password.get();
        if identifier.is_empty() || secret.is_empty() {
            error.set(Some("Please enter your email/phone and password.".to_string()));
            return;
        }
        error.set(None);
        busy.set(true);

        let navigate = submit_navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::login(&identifier, &secret).await {
                Ok(_) => {
                    // "Remember me" off means the session should not outlive the
                    // tab, so drop the refresh token and keep only the access one.
                    if !remember.get_untracked() {
                        if let Some(access) = api::session::access_token() {
                            api::session::clear();
                            api::session::set_tokens(&access, None);
                        }
                    }
                    busy.set(false);
                    navigate("/", Default::default());
                }
                Err(e) => {
                    error.set(Some(if e.is_unauthorized() {
                        "Those credentials were not recognised.".to_string()
                    } else {
                        e.message.clone()
                    }));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <div class="flex min-h-screen items-center justify-center bg-gradient-to-br from-slate-900 via-slate-900 to-blue-950 p-4">
            <div class="w-full max-w-md animate-scale-in rounded-2xl bg-white p-8 shadow-2xl">
                <div class="mb-6 flex flex-col items-center text-center">
                    <img
                        src="https://images.unsplash.com/photo-1566073771259-6a8506099945?q=80&w=200"
                        alt="Golden Tulip Addis Ababa"
                        class="mb-3 h-16 w-16 rounded-xl object-cover shadow-md"
                    />
                    <h1 class="text-xl font-bold text-slate-900">"Welcome back"</h1>
                    <p class="text-sm text-slate-500">"Sign in to manage Golden Tulip Addis Ababa"</p>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="mb-4 flex items-center gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        <Icon name="info" class="h-4 w-4 shrink-0" />
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <form on:submit=on_submit class="flex flex-col gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Email or Phone"</label>
                        <div class="relative">
                            <Icon name="mail" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                            <input
                                type="text"
                                placeholder="ahmed.hassan@tourista.com"
                                class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                prop:value=email
                                on:input:target=move |ev| email.set(ev.target().value())
                            />
                        </div>
                    </div>

                    <div>
                        <div class="mb-1 flex items-center justify-between">
                            <label class="block text-xs font-medium text-slate-500">"Password"</label>
                            <A href="/forgot-password" attr:class="text-xs font-medium text-blue-700 hover:underline">"Forgot password?"</A>
                        </div>
                        <div class="relative">
                            <Icon name="lock" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                placeholder="Enter your password"
                                class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-9 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100"
                                prop:value=password
                                on:input:target=move |ev| password.set(ev.target().value())
                            />
                            <button
                                type="button"
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600"
                                on:click=move |_| show_password.update(|v| *v = !*v)
                            >
                                {move || view! { <Icon name=if show_password.get() { "eye-off" } else { "eye" } class="h-4 w-4" /> }}
                            </button>
                        </div>
                    </div>

                    <label class="flex items-center gap-2 text-sm text-slate-600">
                        <input type="checkbox" prop:checked=remember on:change:target=move |ev| remember.set(ev.target().checked()) class="h-4 w-4 rounded border-slate-300 text-blue-600" />
                        "Remember me on this device"
                    </label>

                    <button
                        type="submit"
                        disabled=move || busy.get()
                        class="mt-1 flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-60"
                    >
                        {move || if busy.get() { "Signing in…" } else { "Sign In" }}
                        <Show when=move || !busy.get()>
                            <Icon name="chevron-right" class="h-4 w-4" />
                        </Show>
                    </button>
                </form>

                <p class="mt-6 text-center text-sm text-slate-500">
                    "Don't have an account? "
                    <A href="/signup" attr:class="font-semibold text-blue-700 hover:underline">"Sign Up"</A>
                </p>
            </div>
        </div>
    }
}
