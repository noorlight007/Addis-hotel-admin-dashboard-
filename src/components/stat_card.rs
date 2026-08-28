use crate::components::Icon;
use leptos::prelude::*;

fn split_numeric(s: &str) -> (String, Option<i64>, String) {
    let chars: Vec<char> = s.chars().collect();
    let Some(start) = chars.iter().position(|c| c.is_ascii_digit()) else {
        return (s.to_string(), None, String::new());
    };
    let mut end = start;
    while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == ',') {
        end += 1;
    }
    let prefix: String = chars[..start].iter().collect();
    let digits: String = chars[start..end].iter().filter(|c| c.is_ascii_digit()).collect();
    let suffix: String = chars[end..].iter().collect();
    match digits.parse::<i64>() {
        Ok(n) => (prefix, Some(n), suffix),
        Err(_) => (s.to_string(), None, String::new()),
    }
}

fn format_thousands(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

#[component]
pub fn StatCard(
    icon: &'static str,
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(into, default = Signal::derive(String::new))] hint: Signal<String>,
    #[prop(default = "text-blue-600 bg-blue-50")] accent: &'static str,
) -> impl IntoView {
    let display = RwSignal::new(0i64);
    let prefix = RwSignal::new(String::new());
    let suffix = RwSignal::new(String::new());
    let is_numeric = RwSignal::new(false);
    let static_text = RwSignal::new(String::new());

    Effect::new(move |_| {
        let raw = value.get();
        let (p, number, s) = split_numeric(&raw);
        match number {
            Some(target) => {
                is_numeric.set(true);
                prefix.set(p);
                suffix.set(s);
                wasm_bindgen_futures::spawn_local(async move {
                    let steps = 20;
                    for i in 1..=steps {
                        let v = ((target as f64) * (i as f64 / steps as f64)).round() as i64;
                        display.set(v);
                        gloo_timers::future::TimeoutFuture::new(16).await;
                    }
                    display.set(target);
                });
            }
            None => {
                is_numeric.set(false);
                static_text.set(raw);
            }
        }
    });

    view! {
        <div class="group rounded-xl border border-slate-200 bg-white p-4 transition-all duration-300 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-slate-200/60">
            <div class="mb-2 flex items-center gap-2">
                <span class=format!(
                    "flex h-9 w-9 items-center justify-center rounded-full transition-transform duration-300 group-hover:scale-110 group-hover:rotate-3 {accent}"
                )>
                    <Icon name=icon class="h-4 w-4" />
                </span>
                <p class="text-xs text-slate-500">{label}</p>
            </div>
            <p class="text-2xl font-bold tabular-nums text-slate-900">
                {move || if is_numeric.get() {
                    format!("{}{}{}", prefix.get(), format_thousands(display.get()), suffix.get())
                } else {
                    static_text.get()
                }}
            </p>
            <p class="text-xs text-slate-400">{move || hint.get()}</p>
        </div>
    }
}
