use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-slate-200 bg-white py-4 text-center text-xs text-slate-400">
            <p class="mb-1 font-semibold text-slate-500">"tourista"</p>
            <p>
                "© 2026 Tourista. All rights reserved. "
                <a href="#" class="hover:underline">"Terms & Conditions"</a>
                " · "
                <a href="#" class="hover:underline">"Privacy Policy"</a>
                " · "
                <a href="#" class="hover:underline">"Support"</a>
            </p>
        </footer>
    }
}
