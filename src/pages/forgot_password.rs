use crate::components::Icon;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let sent = RwSignal::new(false);
    let error = RwSignal::new(Option::<&'static str>::None);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let value = email.get();
        if !value.contains('@') || !value.contains('.') {
            error.set(Some("Enter the email address linked to your account."));
            return;
        }
        error.set(None);
        sent.set(true);
    };

    view! {
        <div class="relative flex min-h-screen items-center justify-center overflow-hidden bg-gradient-to-br from-slate-900 via-slate-900 to-blue-950 p-4">
            <span class="pointer-events-none absolute -left-24 top-0 h-80 w-80 rounded-full bg-blue-600/20 blur-3xl"></span>
            <span class="pointer-events-none absolute -bottom-24 -right-16 h-80 w-80 rounded-full bg-cyan-500/15 blur-3xl"></span>

            <div class="relative w-full max-w-md animate-scale-in rounded-2xl bg-white p-8 shadow-2xl">
                <Show
                    when=move || sent.get()
                    fallback=move || view! {
                        <div>
                            <div class="mb-6 flex flex-col items-center text-center">
                                <span class="mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-50 text-blue-700">
                                    <Icon name="key" class="h-6 w-6" />
                                </span>
                                <h1 class="text-xl font-bold text-slate-900">"Reset your password"</h1>
                                <p class="mt-1 text-sm text-slate-500">
                                    "We'll email you a link to set a new one."
                                </p>
                            </div>

                            <Show when=move || error.get().is_some()>
                                <div class="mb-4 flex animate-fade-up items-center gap-2 rounded-lg bg-red-50 px-3 py-2.5 text-sm text-red-700">
                                    <Icon name="alert" class="h-4 w-4 shrink-0" />
                                    {move || error.get().unwrap_or("")}
                                </div>
                            </Show>

                            <form on:submit=submit class="flex flex-col gap-4">
                                <div>
                                    <label class="mb-1.5 block text-xs font-bold uppercase tracking-wide text-slate-500">
                                        "Email address"
                                    </label>
                                    <div class="relative">
                                        <Icon name="mail" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                                        <input
                                            type="email"
                                            placeholder="ahmed.hassan@tourista.com"
                                            class="w-full rounded-lg border border-slate-300 py-2.5 pl-9 pr-3 text-sm transition-colors focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                                            prop:value=email
                                            on:input:target=move |ev| { email.set(ev.target().value()); error.set(None); }
                                        />
                                    </div>
                                </div>

                                <button
                                    type="submit"
                                    class="sheen flex items-center justify-center gap-2 rounded-lg bg-blue-700 py-2.5 text-sm font-semibold text-white shadow-md shadow-blue-700/30 transition-all duration-200 hover:-translate-y-0.5 hover:bg-blue-800 hover:shadow-lg active:scale-[0.98]"
                                >
                                    <Icon name="send" class="h-4 w-4" />
                                    "Send reset link"
                                </button>
                            </form>
                        </div>
                    }
                >
                    <div class="flex flex-col items-center text-center">
                        <span class="relative mb-4 flex h-16 w-16 items-center justify-center">
                            <span class="absolute inset-0 animate-pulse-ring rounded-full bg-emerald-400"></span>
                            <span class="relative flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500 text-white shadow-lg">
                                <Icon name="mail" class="h-7 w-7" />
                            </span>
                        </span>
                        <h1 class="text-xl font-bold text-slate-900">"Check your inbox"</h1>
                        <p class="mt-2 text-sm leading-relaxed text-slate-500">
                            "If an account exists for "
                            <span class="font-semibold text-slate-800">{move || email.get()}</span>
                            ", a reset link is on its way. It expires in 60 minutes."
                        </p>
                        <button
                            on:click=move |_| sent.set(false)
                            class="mt-5 text-sm font-semibold text-blue-700 hover:underline"
                        >
                            "Use a different email"
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
