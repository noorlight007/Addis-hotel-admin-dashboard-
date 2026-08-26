//! Shared presentational pieces used across dashboard pages.

use crate::components::Icon;
use leptos::prelude::*;

/// `(1, "guest")` -> `"1 guest"`, `(3, "guest")` -> `"3 guests"`.
///
/// Only handles nouns that pluralise with a bare `s`, which covers every count
/// noun in the dashboard (guest, room, night, arrival, departure).
pub fn pluralize(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("{n} {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// Page title block with an optional action slot on the right.
#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(into, default = String::new())] subtitle: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    // These props are fixed for the lifetime of the page, so plain Rust
    // conditionals are used rather than reactive `Show` blocks.
    let subtitle = (!subtitle.is_empty()).then_some(subtitle);
    view! {
        <div class="mb-5 flex animate-fade-up flex-wrap items-end justify-between gap-3">
            <div class="min-w-0">
                <h1 class="text-xl font-bold tracking-tight text-slate-900">{title}</h1>
                {subtitle.map(|s| view! { <p class="mt-0.5 text-sm text-slate-500">{s}</p> })}
            </div>
            {children.map(|c| view! { <div class="flex shrink-0 flex-wrap items-center gap-2">{c()}</div> })}
        </div>
    }
}

/// Bordered white panel — the base surface for nearly every block.
#[component]
pub fn Card(
    #[prop(into, default = String::new())] title: String,
    #[prop(into, default = String::new())] hint: String,
    #[prop(default = "")] class: &'static str,
    #[prop(optional)] action: Option<Children>,
    children: Children,
) -> impl IntoView {
    let hint = (!hint.is_empty()).then_some(hint);
    let header = (!title.is_empty()).then(|| {
        view! {
            <div class="flex items-start justify-between gap-3 border-b border-slate-100 px-5 py-3.5">
                <div class="min-w-0">
                    <h2 class="text-sm font-bold text-slate-900">{title}</h2>
                    {hint.map(|h| view! { <p class="mt-0.5 text-xs text-slate-400">{h}</p> })}
                </div>
                {action.map(|a| view! { <div class="flex shrink-0 items-center gap-2">{a()}</div> })}
            </div>
        }
    });

    view! {
        <section class=format!("rounded-xl border border-slate-200 bg-white shadow-card {class}")>
            {header}
            <div class="p-5">{children()}</div>
        </section>
    }
}

/// Placeholder shown when a filtered list has no rows.
#[component]
pub fn EmptyState(
    icon: &'static str,
    #[prop(into)] title: String,
    #[prop(into)] body: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex animate-fade-up flex-col items-center gap-2.5 px-6 py-14 text-center">
            <span class="relative flex h-14 w-14 items-center justify-center rounded-full bg-slate-50 text-slate-400 ring-1 ring-slate-200">
                <span class="absolute inset-0 animate-pulse-ring rounded-full bg-slate-200"></span>
                <Icon name=icon class="relative h-6 w-6" />
            </span>
            <h3 class="text-sm font-bold text-slate-800">{title}</h3>
            <p class="max-w-sm text-sm text-slate-500">{body}</p>
            {children.map(|c| view! { <div class="mt-2">{c()}</div> })}
        </div>
    }
}

/// Coloured status pill.
#[component]
pub fn Badge(
    #[prop(into)] label: String,
    #[prop(default = "slate")] tone: &'static str,
    #[prop(default = false)] dot: bool,
) -> impl IntoView {
    let classes = match tone {
        "green" => "bg-emerald-50 text-emerald-700 ring-emerald-200",
        "blue" => "bg-blue-50 text-blue-700 ring-blue-200",
        "amber" => "bg-amber-50 text-amber-700 ring-amber-200",
        "red" => "bg-red-50 text-red-700 ring-red-200",
        "purple" => "bg-purple-50 text-purple-700 ring-purple-200",
        _ => "bg-slate-100 text-slate-600 ring-slate-200",
    };
    let dot_class = match tone {
        "green" => "bg-emerald-500",
        "blue" => "bg-blue-500",
        "amber" => "bg-amber-500",
        "red" => "bg-red-500",
        "purple" => "bg-purple-500",
        _ => "bg-slate-400",
    };
    view! {
        <span class=format!("inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-semibold ring-1 {classes}")>
            <Show when=move || dot>
                <span class=format!("h-1.5 w-1.5 rounded-full {dot_class}")></span>
            </Show>
            {label}
        </span>
    }
}

/// Segmented control. Emits the selected label.
#[component]
pub fn SegmentedControl(
    options: &'static [&'static str],
    selected: RwSignal<&'static str>,
) -> impl IntoView {
    view! {
        <div class="flex gap-0.5 rounded-lg bg-slate-100 p-0.5">
            {options.iter().map(|opt| {
                let opt = *opt;
                view! {
                    <button
                        on:click=move |_| selected.set(opt)
                        class=move || format!(
                            "rounded-md px-3 py-1.5 text-xs font-semibold transition-all duration-200 {}",
                            if selected.get() == opt { "bg-white text-slate-900 shadow-sm" } else { "text-slate-500 hover:text-slate-700" }
                        )
                    >
                        {opt}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Circular monogram used for guests, staff and reviewers.
#[component]
pub fn Avatar(
    #[prop(into)] initials: String,
    #[prop(default = "h-9 w-9 text-xs")] size: &'static str,
    #[prop(default = "from-blue-600 to-indigo-700")] gradient: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!(
            "flex shrink-0 items-center justify-center rounded-full bg-gradient-to-br font-bold text-white {gradient} {size}"
        )>
            {initials}
        </span>
    }
}

/// Loading placeholder row for tables.
#[component]
pub fn SkeletonRows(#[prop(default = 5)] rows: usize, #[prop(default = 6)] columns: usize) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 p-4">
            {(0..rows).map(|_| view! {
                <div class="flex gap-3">
                    {(0..columns).map(|c| view! {
                        <div class=format!(
                            "skeleton h-8 rounded {}",
                            if c == 0 { "w-1/4" } else { "flex-1" }
                        )></div>
                    }).collect_view()}
                </div>
            }).collect_view()}
        </div>
    }
}
