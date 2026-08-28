//! Guest reviews, backed by `/reviews/` and `/reviews/metrics/`.
//!
//! Metrics tiles and the list are live; the reply box POSTs to
//! `/reviews/{id}/reply/` and refreshes so the response appears immediately.

use crate::api::{self, ReviewRow};
use crate::components::{Icon, StatCard};
use leptos::prelude::*;

const TABS: &[&str] = &["All", "5 Star", "4 Star", "3 Star & Below", "Needs Reply"];

#[component]
pub fn ReviewsPage() -> impl IntoView {
    let filter = RwSignal::new("All");
    let search = RwSignal::new(String::new());
    let refresh = RwSignal::new(0u32);

    let reviews = LocalResource::new(move || {
        let q = search.get();
        let _ = refresh.get();
        async move { api::list_reviews(&q, None, None).await }
    });
    let metrics = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::review_metrics().await }
    });

    // Client-side tab filtering over the fetched page (the list endpoint has
    // rate/has_reply params, but filtering the loaded set keeps tab switches
    // instant and the datasets here are small).
    let visible = move |rows: Vec<ReviewRow>| -> Vec<ReviewRow> {
        rows.into_iter()
            .filter(|r| match filter.get() {
                "Needs Reply" => r.reply.as_deref().unwrap_or("").trim().is_empty(),
                "5 Star" => r.rate == 5,
                "4 Star" => r.rate == 4,
                "3 Star & Below" => r.rate <= 3,
                _ => true,
            })
            .collect()
    };

    let metric_str = move |f: fn(&api::ReviewMetrics) -> String| {
        metrics
            .get()
            .and_then(Result::ok)
            .map(|m| f(&m))
            .unwrap_or_else(|| "—".into())
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4">
                <h1 class="text-xl font-bold text-slate-900">"Reviews"</h1>
                <p class="text-sm text-slate-500">"See what guests are saying and respond to their feedback."</p>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <StatCard icon="star" label="Average Rating" value=Signal::derive(move || metric_str(|m| format!("{:.1}", m.average_rating))) hint="Out of 5.0" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="message" label="Total Reviews" value=Signal::derive(move || metric_str(|m| m.total_reviews.to_string())) hint="All time" />
                <StatCard icon="check-circle" label="Response Rate" value=Signal::derive(move || metric_str(|m| format!("{:.0}%", m.reply_rate_percentage))) hint="Reviews with a reply" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="clock" label="Pending Reply" value=Signal::derive(move || metric_str(|m| m.pending_reply_count.to_string())) hint="Awaiting a response" accent="text-purple-600 bg-purple-50" />
            </div>

            <div class="mb-4 flex flex-wrap gap-2 text-sm">
                {TABS.iter().map(|f| {
                    let f = *f;
                    view! {
                        <button
                            class=move || format!(
                                "rounded-lg px-4 py-2 font-semibold transition-colors {}",
                                if filter.get() == f { "bg-blue-700 text-white" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                            )
                            on:click=move |_| filter.set(f)
                        >
                            {f}
                        </button>
                    }
                }).collect_view()}
            </div>

            <Suspense fallback=|| view! {
                <div class="rounded-xl border border-slate-200 bg-white p-10 text-center text-sm text-slate-400">"Loading reviews…"</div>
            }>
                {move || Suspend::new(async move {
                    match reviews.await {
                        Err(e) => view! {
                            <div class="rounded-xl border border-slate-200 bg-white p-10 text-center text-sm text-red-600">{e.message}</div>
                        }.into_any(),
                        Ok(page) => {
                            let rows = visible(page.items);
                            if rows.is_empty() {
                                return view! {
                                    <div class="rounded-xl border border-slate-200 bg-white p-10 text-center text-sm text-slate-500">
                                        "No reviews yet. They appear here once guests leave feedback."
                                    </div>
                                }.into_any();
                            }
                            view! {
                                <div class="flex flex-col gap-4">
                                    {rows.into_iter().map(|r| view! {
                                        <ReviewCard review=r on_replied=move || refresh.update(|n| *n += 1) />
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ReviewCard(
    review: ReviewRow,
    on_replied: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let replying = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let id = review.id;
    let rating = review.rate.clamp(0, 5);
    let existing_reply = review.reply.clone().filter(|r| !r.trim().is_empty());
    let has_reply = existing_reply.is_some();
    let initials = review
        .guest_name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    let submit_reply = move |_| {
        let text = draft.get().trim().to_string();
        if text.is_empty() || busy.get() {
            return;
        }
        busy.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api::reply_review(id, &text).await;
            busy.set(false);
            replying.set(false);
            on_replied();
        });
    };

    view! {
        <div class="rounded-xl border border-slate-200 bg-white p-5">
            <div class="mb-2 flex items-start justify-between">
                <div class="flex items-center gap-3">
                    <span class="flex h-10 w-10 items-center justify-center rounded-full bg-blue-100 text-sm font-semibold text-blue-700">{initials}</span>
                    <div>
                        <p class="font-semibold text-slate-900">{review.guest_name.clone()}</p>
                        <p class="text-xs text-slate-400">
                            {format!(
                                "{} · {}",
                                api::pretty_datetime(review.created_at.as_deref()),
                                review.room_label(),
                            )}
                        </p>
                    </div>
                </div>
                <div class="flex items-center gap-0.5">
                    {(1..=5).map(|i| view! {
                        <Icon name="star-fill" class=if i <= rating { "h-4 w-4 text-amber-400" } else { "h-4 w-4 text-slate-200" } />
                    }).collect_view()}
                </div>
            </div>
            <p class="text-sm leading-relaxed text-slate-600">{review.comment.clone()}</p>

            {existing_reply.clone().map(|reply| view! {
                <div class="mt-3 flex gap-2 rounded-lg bg-slate-50 p-3">
                    <Icon name="building" class="mt-0.5 h-4 w-4 shrink-0 text-blue-600" />
                    <div>
                        <p class="text-xs font-semibold text-slate-700">"Hotel responded"</p>
                        <p class="text-sm text-slate-600">{reply}</p>
                        <p class="mt-1 text-2xs text-slate-400">
                            {format!(
                                "{}{}",
                                review.replied_by_name.clone().unwrap_or_else(|| "Hotel".into()),
                                review.replied_at.as_deref()
                                    .map(|t| format!(" · {}", api::pretty_datetime(Some(t))))
                                    .unwrap_or_default(),
                            )}
                        </p>
                    </div>
                </div>
            })}

            <Show when=move || !has_reply && !replying.get()>
                <button
                    on:click=move |_| replying.set(true)
                    class="mt-3 flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"
                >
                    <Icon name="message" class="h-4 w-4" />
                    "Reply"
                </button>
            </Show>

            <Show when=move || replying.get()>
                <div class="mt-3 flex flex-col gap-2">
                    <textarea
                        rows="2"
                        placeholder="Write a public reply to this review..."
                        class="w-full rounded-lg border border-slate-300 p-2 text-sm"
                        prop:value=draft
                        on:input:target=move |ev| draft.set(ev.target().value())
                    ></textarea>
                    <div class="flex justify-end gap-2">
                        <button class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium" on:click=move |_| replying.set(false)>"Cancel"</button>
                        <button
                            disabled=move || busy.get()
                            class="rounded-lg bg-blue-700 px-3 py-1.5 text-sm font-semibold text-white transition-all hover:bg-blue-800 active:scale-[0.98] disabled:opacity-60"
                            on:click=submit_reply
                        >
                            {move || if busy.get() { "Posting…" } else { "Post Reply" }}
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
