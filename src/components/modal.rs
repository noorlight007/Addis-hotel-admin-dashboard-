use crate::components::Icon;
use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)] title: String,
    on_close: impl Fn() + Copy + Send + Sync + 'static,
    #[prop(default = "max-w-lg")] width: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-30 flex items-center justify-center bg-black/40 p-4 animate-fade-in">
            <div class=format!("max-h-[90vh] w-full {width} overflow-y-auto rounded-2xl bg-white p-6 animate-scale-in")>
                <div class="mb-5 flex items-center justify-between">
                    <h2 class="text-lg font-bold text-slate-900">{title}</h2>
                    <button on:click=move |_| on_close()>
                        <Icon name="x" class="h-5 w-5 text-slate-400" />
                    </button>
                </div>
                {children()}
            </div>
        </div>
    }
}
