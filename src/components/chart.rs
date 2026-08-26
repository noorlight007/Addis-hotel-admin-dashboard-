//! Hand-rolled SVG charts.
//!
//! The dashboard only needs a handful of shapes, so these are drawn directly
//! rather than pulling in a charting crate — it keeps the wasm bundle small and
//! makes the entrance animations trivial to control.

use crate::components::Icon;
use leptos::prelude::*;

/// Builds an SVG polyline `points` string, mapping values onto a 0..w by 0..h box.
fn points_for(values: &[u32], w: f64, h: f64, pad: f64) -> Vec<(f64, f64)> {
    if values.is_empty() {
        return Vec::new();
    }
    let max = *values.iter().max().unwrap_or(&1) as f64;
    let min = *values.iter().min().unwrap_or(&0) as f64;
    // Give the line a little headroom so peaks aren't glued to the top edge.
    let span = (max - min).max(1.0) * 1.15;
    let base = min - (max - min) * 0.075;
    let step = if values.len() > 1 { (w - pad * 2.0) / (values.len() - 1) as f64 } else { 0.0 };

    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = pad + step * i as f64;
            let y = h - pad - ((*v as f64 - base) / span) * (h - pad * 2.0);
            (x, y)
        })
        .collect()
}

fn fmt_path(pts: &[(f64, f64)]) -> String {
    pts.iter()
        .enumerate()
        .map(|(i, (x, y))| format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { " L" }))
        .collect()
}

/// Area + line chart with an animated draw-on and hoverable points.
#[component]
pub fn LineChart(
    values: Vec<u32>,
    labels: Vec<&'static str>,
    #[prop(default = "#2563eb")] stroke: &'static str,
    #[prop(default = "")] value_prefix: &'static str,
    #[prop(default = 200.0)] height: f64,
) -> impl IntoView {
    let w = 640.0;
    let h = height;
    let pad = 16.0;
    let pts = points_for(&values, w, h, pad);
    let line = fmt_path(&pts);
    let area = if pts.is_empty() {
        String::new()
    } else {
        format!(
            "{} L{:.1} {:.1} L{:.1} {:.1} Z",
            line,
            pts.last().unwrap().0,
            h - pad,
            pts[0].0,
            h - pad
        )
    };
    let gradient_id = format!("area-{}", stroke.trim_start_matches('#'));
    let hovered = RwSignal::new(Option::<usize>::None);
    let max = values.iter().copied().max().unwrap_or(0);
    // The tooltip closure runs on every hover, so the series has to be stored
    // rather than moved into it.
    let stored_pts = StoredValue::new(pts.clone());
    let stored_values = StoredValue::new(values.clone());
    let stored_labels = StoredValue::new(labels.clone());

    view! {
        <div class="relative">
            <svg
                viewBox=format!("0 0 {w} {h}")
                class="w-full"
                style=format!("height: {h}px")
                preserveAspectRatio="none"
            >
                <defs>
                    <linearGradient id=gradient_id.clone() x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color=stroke stop-opacity="0.28" />
                        <stop offset="100%" stop-color=stroke stop-opacity="0" />
                    </linearGradient>
                </defs>

                // Horizontal guide lines.
                {(0..4).map(|i| {
                    let y = pad + (h - pad * 2.0) / 3.0 * i as f64;
                    view! {
                        <line x1=pad x2=w - pad y1=y y2=y stroke="#e2e8f0" stroke-width="1" stroke-dasharray="4 4" />
                    }
                }).collect_view()}

                <path d=area fill=format!("url(#{gradient_id})") />
                <path
                    d=line
                    fill="none"
                    stroke=stroke
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="animate-draw-line"
                    style="stroke-dasharray: 1000"
                />

                {pts.iter().enumerate().map(|(i, (x, y))| {
                    let x = *x;
                    let y = *y;
                    view! {
                        <g>
                            <circle
                                cx=x
                                cy=y
                                r=move || if hovered.get() == Some(i) { 6.0 } else { 3.5 }
                                fill="#fff"
                                stroke=stroke
                                stroke-width="2.5"
                                class="transition-all duration-150"
                            />
                            // Wide invisible hit area so hovering is forgiving.
                            <rect
                                x=x - 18.0
                                y="0"
                                width="36"
                                height=h
                                fill="transparent"
                                on:mouseenter=move |_| hovered.set(Some(i))
                                on:mouseleave=move |_| hovered.set(None)
                            />
                        </g>
                    }
                }).collect_view()}
            </svg>

            // Tooltip follows the hovered point.
            <Show when=move || hovered.get().is_some()>
                {move || {
                    let i = hovered.get().unwrap_or(0);
                    let (x, y) = stored_pts.with_value(|p| p.get(i).copied().unwrap_or((0.0, 0.0)));
                    let value = stored_values.with_value(|v| v.get(i).copied().unwrap_or(0));
                    let label = stored_labels.with_value(|l| l.get(i).copied().unwrap_or(""));
                    view! {
                        <div
                            class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full animate-scale-in rounded-lg bg-slate-900 px-2.5 py-1.5 text-center text-xs text-white shadow-lg"
                            style=format!("left: {:.2}%; top: {:.2}%", x / w * 100.0, y / h * 100.0)
                        >
                            <span class="block font-bold">{format!("{value_prefix}{value}")}</span>
                            <span class="block text-[10px] text-slate-400">{label}</span>
                        </div>
                    }
                }}
            </Show>

            <div class="mt-2 flex justify-between px-1 text-2xs text-slate-400">
                {labels.into_iter().map(|l| view! { <span>{l}</span> }).collect_view()}
            </div>

            <Show when=move || (max == 0)>
                <p class="mt-2 text-center text-xs text-slate-400">"No data for this period."</p>
            </Show>
        </div>
    }
}

/// Vertical bars with a value label on hover.
#[component]
pub fn BarChart(
    values: Vec<u32>,
    labels: Vec<&'static str>,
    #[prop(default = "bg-blue-600")] bar_class: &'static str,
    #[prop(default = "")] value_prefix: &'static str,
) -> impl IntoView {
    let max = values.iter().copied().max().unwrap_or(1).max(1);

    view! {
        <div class="flex h-48 items-end gap-2">
            {values.into_iter().enumerate().map(|(i, v)| {
                let pct = (v * 100 / max).max(2);
                let label = labels.get(i).copied().unwrap_or("");
                let delay = format!("animation-delay: {}ms; height: {}%", i * 60, pct);
                view! {
                    <div class="group flex flex-1 flex-col items-center justify-end gap-1.5">
                        <span class="pointer-events-none rounded-md bg-slate-900 px-1.5 py-0.5 text-2xs font-bold text-white opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                            {format!("{value_prefix}{v}")}
                        </span>
                        <span
                            class=format!(
                                "w-full animate-bar-grow rounded-t-lg transition-all duration-200 group-hover:brightness-110 {bar_class}"
                            )
                            style=delay
                        ></span>
                        <span class="text-2xs text-slate-400">{label}</span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

/// Donut chart with a centre total and an interactive legend.
#[component]
pub fn DonutChart(
    /// `(label, value, tailwind-ish hex colour)`
    slices: Vec<(&'static str, u32, &'static str)>,
    #[prop(default = "Total")] centre_label: &'static str,
) -> impl IntoView {
    let total: u32 = slices.iter().map(|(_, v, _)| *v).sum();
    let total_safe = total.max(1) as f64;
    let stored_slices = StoredValue::new(slices.clone());
    let radius = 54.0;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let hovered = RwSignal::new(Option::<usize>::None);

    // Pre-compute each arc's dash length and rotation offset.
    let mut offset = 0.0f64;
    let arcs: Vec<(usize, &'static str, u32, &'static str, f64, f64)> = slices
        .iter()
        .enumerate()
        .map(|(i, (label, value, color))| {
            let fraction = *value as f64 / total_safe;
            let dash = fraction * circumference;
            let rotation = offset / circumference * 360.0 - 90.0;
            offset += dash;
            (i, *label, *value, *color, dash, rotation)
        })
        .collect();

    view! {
        <div class="flex flex-col items-center gap-5 sm:flex-row sm:justify-center">
            <div class="relative h-40 w-40 shrink-0">
                <svg viewBox="0 0 140 140" class="h-full w-full">
                    <circle cx="70" cy="70" r=radius fill="none" stroke="#f1f5f9" stroke-width="18" />
                    {arcs.iter().map(|(i, _, _, color, dash, rotation)| {
                        let i = *i;
                        let dash = *dash;
                        let rotation = *rotation;
                        view! {
                            <circle
                                cx="70"
                                cy="70"
                                r=radius
                                fill="none"
                                stroke=*color
                                stroke-width=move || if hovered.get() == Some(i) { "22" } else { "18" }
                                stroke-dasharray=format!("{dash:.2} {:.2}", circumference - dash)
                                transform=format!("rotate({rotation:.2} 70 70)")
                                stroke-linecap="butt"
                                class="origin-center transition-all duration-200"
                                on:mouseenter=move |_| hovered.set(Some(i))
                                on:mouseleave=move |_| hovered.set(None)
                            />
                        }
                    }).collect_view()}
                </svg>

                <div class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
                    {move || match hovered.get() {
                        Some(i) => {
                            let (label, value, _) = stored_slices.with_value(|s| s.get(i).copied().unwrap_or(("", 0, "")));
                            view! {
                                <>
                                    <span class="animate-pop-in text-2xl font-extrabold text-slate-900">{value}</span>
                                    <span class="text-2xs text-slate-500">{label}</span>
                                </>
                            }
                        }
                        None => view! {
                            <>
                                <span class="text-2xl font-extrabold text-slate-900">{total}</span>
                                <span class="text-2xs text-slate-500">{centre_label}</span>
                            </>
                        },
                    }}
                </div>
            </div>

            <ul class="flex w-full flex-col gap-1.5 sm:w-auto">
                {slices.iter().enumerate().map(|(i, (label, value, color))| {
                    let label = *label;
                    let value = *value;
                    let color = *color;
                    let pct = value * 100 / total.max(1);
                    view! {
                        <li
                            class=move || format!(
                                "flex items-center justify-between gap-4 rounded-lg px-2.5 py-1.5 text-sm transition-colors {}",
                                if hovered.get() == Some(i) { "bg-slate-100" } else { "" }
                            )
                            on:mouseenter=move |_| hovered.set(Some(i))
                            on:mouseleave=move |_| hovered.set(None)
                        >
                            <span class="flex items-center gap-2">
                                <span class="h-2.5 w-2.5 shrink-0 rounded-full" style=format!("background: {color}")></span>
                                <span class="text-slate-600">{label}</span>
                            </span>
                            <span class="shrink-0 font-semibold tabular-nums text-slate-900">
                                {value}
                                <span class="ml-1 text-xs font-normal text-slate-400">{format!("({pct}%)")}</span>
                            </span>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

/// Horizontal progress meter with an animated fill.
#[component]
pub fn ProgressBar(
    #[prop(into)] percent: Signal<u32>,
    #[prop(default = "bg-blue-600")] fill: &'static str,
    #[prop(default = "h-2")] height: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("block w-full overflow-hidden rounded-full bg-slate-100 {height}")>
            <span
                class=format!("block h-full animate-width-grow rounded-full transition-all duration-500 {fill}")
                style=move || format!("width: {}%", percent.get().min(100))
            ></span>
        </span>
    }
}

/// Compact "up/down since last period" pill.
#[component]
pub fn TrendPill(delta: f32, #[prop(default = "vs last week")] period: &'static str) -> impl IntoView {
    let up = delta >= 0.0;
    view! {
        <span class=format!(
            "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs font-bold {}",
            if up { "bg-emerald-50 text-emerald-700" } else { "bg-red-50 text-red-700" }
        )>
            <span class=if up { "" } else { "rotate-90" }>
                <Icon name="trending-up" class="h-3 w-3" />
            </span>
            {format!("{}{:.1}%", if up { "+" } else { "" }, delta)}
            <span class="font-normal opacity-70">{period}</span>
        </span>
    }
}
