//! Password reset, backed by the two OTP endpoints.
//!
//! The API does not email a reset *link* — it sends a six-digit code
//! (`/accounts/auth/password/reset/request/`) which is then exchanged for a new
//! password (`/accounts/auth/password/reset/confirm/`). So this is a two-step
//! form rather than a "check your inbox" dead end.
//!
//! The identifier can be an email address or a phone number, matching the
//! request endpoint.

use crate::api;
use crate::components::Icon;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Request,
    Confirm,
    Done,
}

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let navigate = use_navigate();
    let nav = StoredValue::new(navigate);

    let step = RwSignal::new(Step::Request);
    let identifier = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let request_code = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        let id = identifier.get().trim().to_string();
        if id.is_empty() {
            error.set(Some("Enter the email address or phone number on your account.".into()));
            return;
        }
        error.set(None);
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api::request_password_reset(&id).await {
                Ok(()) => {
                    busy.set(false);
                    step.set(Step::Confirm);
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    let confirm_reset = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if code.get().trim().len() != 6 {
            error.set(Some("The code is six digits.".into()));
            return;
        }
        if password.get().len() < 6 {
            error.set(Some("The new password must be at least 6 characters.".into()));
            return;
        }
        if password.get() != confirm.get() {
            error.set(Some("The two passwords do not match.".into()));
            return;
        }
        error.set(None);
        busy.set(true);
        let (id, otp, pw) = (identifier.get(), code.get(), password.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::confirm_password_reset(&id, &otp, &pw).await {
                Ok(()) => {
                    busy.set(false);
                    step.set(Step::Done);
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    let resend = move |_| {
        let id = identifier.get();
        busy.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = api::request_password_reset(&id).await {
                error.set(Some(e.detail()));
            }
            busy.set(false);
        });
    };

    view! {
        <div class="relative flex min-h-screen items-center justify-center overflow-hidden bg-gradient-to-br from-slate-900 via-slate-900 to-blue-950 p-4">
            <span class="pointer-events-none absolute -left-24 top-0 h-80 w-80 rounded-full bg-blue-600/20 blur-3xl"></span>
            <span class="pointer-events-none absolute -bottom-24 -right-16 h-80 w-80 rounded-full bg-cyan-500/15 blur-3xl"></span>

            <div class="relative w-full max-w-md animate-scale-in rounded-2xl bg-white p-8 shadow-2xl">
                // ---- Step 1: ask for the code ----------------------------
                <Show when=move || step.get() == Step::Request>
                    <div>
                        <div class="mb-6 flex flex-col items-center text-center">
                            <span class="mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-50 text-blue-700">
                                <Icon name="key" class="h-6 w-6" />
                            </span>
                            <h1 class="text-xl font-bold text-slate-900">"Reset your password"</h1>
                            <p class="mt-1 text-sm text-slate-500">
                                "We'll send a six-digit code to the email or phone on your account."
                            </p>
                        </div>

                        <ErrorBanner error=error />

                        <form on:submit=request_code class="flex flex-col gap-4">
                            <div>
                                <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">
                                    "Email or phone"
                                </label>
                                <div class="relative">
                                    <Icon name="mail" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                    <input
                                        type="text"
                                        placeholder="you@hotel.com or +251…"
                                        class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                                        prop:value=identifier
                                        on:input:target=move |ev| { identifier.set(ev.target().value()); error.set(None); }
                                    />
                                </div>
                            </div>

                            <button
                                type="submit"
                                disabled=move || busy.get()
                                class="sheen flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98] disabled:opacity-60"
                            >
                                <Icon name="send" class="h-4 w-4" />
                                {move || if busy.get() { "Sending…" } else { "Send code" }}
                            </button>
                        </form>
                    </div>
                </Show>

                // ---- Step 2: enter the code and a new password -----------
                <Show when=move || step.get() == Step::Confirm>
                    <div>
                        <div class="mb-6 flex flex-col items-center text-center">
                            <span class="mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-50 text-blue-700">
                                <Icon name="lock" class="h-6 w-6" />
                            </span>
                            <h1 class="text-xl font-bold text-slate-900">"Enter your code"</h1>
                            <p class="mt-1 text-sm text-slate-500">
                                "We sent a six-digit code to "
                                <span class="font-semibold text-slate-800">{move || identifier.get()}</span>
                                "."
                            </p>
                        </div>

                        <ErrorBanner error=error />

                        <form on:submit=confirm_reset class="flex flex-col gap-4">
                            <div>
                                <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">
                                    "Verification code"
                                </label>
                                <input
                                    type="text"
                                    inputmode="numeric"
                                    maxlength="6"
                                    placeholder="123456"
                                    class="w-full rounded-lg border border-slate-300 py-2.5 px-3 text-center text-lg font-bold tracking-[0.4em] tabular-nums transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                                    prop:value=code
                                    on:input:target=move |ev| { code.set(ev.target().value()); error.set(None); }
                                />
                            </div>

                            <PasswordField label="New password" value=password />
                            <PasswordField label="Confirm new password" value=confirm />

                            <button
                                type="submit"
                                disabled=move || busy.get()
                                class="sheen flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98] disabled:opacity-60"
                            >
                                <Icon name="check" class="h-4 w-4" />
                                {move || if busy.get() { "Resetting…" } else { "Set new password" }}
                            </button>
                        </form>

                        <div class="mt-4 flex items-center justify-between text-xs">
                            <button
                                class="font-semibold text-slate-500 hover:text-slate-800"
                                on:click=move |_| { step.set(Step::Request); error.set(None); }
                            >
                                "Use a different address"
                            </button>
                            <button
                                class="font-semibold text-blue-700 hover:underline disabled:opacity-50"
                                disabled=move || busy.get()
                                on:click=resend
                            >
                                "Resend code"
                            </button>
                        </div>
                    </div>
                </Show>

                // ---- Step 3: done ----------------------------------------
                <Show when=move || step.get() == Step::Done>
                    <div class="flex flex-col items-center text-center">
                        <span class="relative mb-4 flex h-16 w-16 items-center justify-center">
                            <span class="absolute inset-0 animate-pulse-ring rounded-full bg-emerald-400"></span>
                            <span class="relative flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500 text-white shadow-lg">
                                <Icon name="check" class="h-7 w-7" />
                            </span>
                        </span>
                        <h1 class="text-xl font-bold text-slate-900">"Password updated"</h1>
                        <p class="mt-2 text-sm leading-relaxed text-slate-500">
                            "You can now sign in with your new password."
                        </p>
                        <button
                            on:click=move |_| nav.with_value(|n| n("/login", Default::default()))
                            class="mt-5 w-full rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white hover:bg-blue-800"
                        >
                            "Go to sign in"
                        </button>
                    </div>
                </Show>

                <div class="mt-6 border-t border-slate-100 pt-5 text-center text-sm">
                    <A
                        href="/login"
                        attr:class="inline-flex items-center gap-1.5 font-semibold text-slate-600 transition-colors hover:text-blue-700"
                    >
                        <Icon name="arrow-left" class="h-4 w-4" />
                        "Back to sign in"
                    </A>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ErrorBanner(error: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <div class="mb-4 flex animate-fade-up items-start gap-2 rounded-lg bg-red-50 px-3 py-2.5 text-sm text-red-700">
                <Icon name="alert" class="mt-0.5 h-4 w-4 shrink-0" />
                {move || error.get().unwrap_or_default()}
            </div>
        </Show>
    }
}

#[component]
fn PasswordField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">{label}</label>
            <div class="relative">
                <Icon name="lock" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                    type="password"
                    class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    prop:value=value
                    on:input:target=move |ev| value.set(ev.target().value())
                />
            </div>
        </div>
    }
}
