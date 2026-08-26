use crate::components::{Icon, StatCard};
use crate::data::{Review, REVIEWS};
use leptos::prelude::*;

fn avg_rating(reviews: &[Review]) -> f32 {
    if reviews.is_empty() {
        return 0.0;
    }
    reviews.iter().map(|r| r.rating as f32).sum::<f32>() / reviews.len() as f32
}

#[component]
pub fn ReviewsPage() -> impl IntoView {
    let reviews = RwSignal::new(REVIEWS.to_vec());
    let filter = RwSignal::new("All");

    let five_star_pct = move || {
        let list = reviews.get();
        if list.is_empty() {
            return 0;
        }
        (list.iter().filter(|r| r.rating == 5).count() * 100 / list.len()) as u32
    };
    let response_rate = move || {
        let list = reviews.get();
        if list.is_empty() {
            return 0;
        }
        (list.iter().filter(|r| r.reply.is_some()).count() * 100 / list.len()) as u32
    };

    let filtered = move || {
        let list = reviews.get();
        match filter.get() {
            "Needs Reply" => list.into_iter().filter(|r| r.reply.is_none()).collect::<Vec<_>>(),
            "5 Star" => list.into_iter().filter(|r| r.rating == 5).collect(),
            "4 Star" => list.into_iter().filter(|r| r.rating == 4).collect(),
            "3 Star & Below" => list.into_iter().filter(|r| r.rating <= 3).collect(),
            _ => list,
        }
    };

    view! {
        <div class="p-4 sm:p-6">
            <div class="mb-4">
                <h1 class="text-xl font-bold text-slate-900">"Reviews"</h1>
                <p class="text-sm text-slate-500">"See what guests are saying and respond to their feedback."</p>
            </div>

            <div class="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4">
                <StatCard icon="star" label="Average Rating" value=Signal::derive(move || format!("{:.1}", avg_rating(&reviews.get()))) hint="Out of 5.0" accent="text-amber-600 bg-amber-50" />
                <StatCard icon="message" label="Total Reviews" value=Signal::derive(move || reviews.get().len().to_string()) hint="All time" />
                <StatCard icon="check-circle" label="Response Rate" value=Signal::derive(move || format!("{}%", response_rate())) hint="Reviews with a reply" accent="text-emerald-600 bg-emerald-50" />
                <StatCard icon="star-fill" label="5-Star Reviews" value=Signal::derive(move || format!("{}%", five_star_pct())) hint="Of total reviews" accent="text-purple-600 bg-purple-50" />
            </div>

            <div class="mb-4 flex flex-wrap gap-2 text-sm">
                {["All", "5 Star", "4 Star", "3 Star & Below", "Needs Reply"].iter().map(|f| {
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

            <div class="flex flex-col gap-4">
                {move || filtered().into_iter().map(|r| {
                    let guest = r.guest;
                    let date = r.date;
                    view! {
                        <ReviewCard review=r on_reply=move |text: String| {
                            reviews.update(|list| {
                                if let Some(item) = list.iter_mut().find(|x| x.guest == guest && x.date == date) {
                                    item.reply = Some(Box::leak(text.into_boxed_str()));
                                }
                            });
                        } />
                    }
                }).collect_view()}
            </div>

            <Show when=move || filtered().is_empty()>
                <div class="rounded-xl border border-slate-200 bg-white p-10 text-center text-sm text-slate-500">
                    "No reviews match this filter."
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ReviewCard(review: Review, on_reply: impl Fn(String) + Copy + Send + Sync + 'static) -> impl IntoView {
    let replying = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let reply = RwSignal::new(review.reply.map(|r| r.to_string()));
    let rating = review.rating;

    let submit_reply = move |_| {
        if !draft.get().trim().is_empty() {
            reply.set(Some(draft.get()));
            on_reply(draft.get());
            replying.set(false);
        }
    };

    view! {
        <div class="rounded-xl border border-slate-200 bg-white p-5">
            <div class="mb-2 flex items-start justify-between">
                <div class="flex items-center gap-3">
                    <span class="flex h-10 w-10 items-center justify-center rounded-full bg-blue-100 text-sm font-semibold text-blue-700">{review.initials}</span>
                    <div>
                        <p class="font-semibold text-slate-900">{review.guest}</p>
                        <p class="text-xs text-slate-400">{format!("{} · {}", review.room_type, review.date)}</p>
                    </div>
                </div>
                <div class="flex items-center gap-0.5">
                    {(1..=5).map(|i| view! {
                        <Icon name="star-fill" class=if i <= rating { "h-4 w-4 text-amber-400" } else { "h-4 w-4 text-slate-200" } />
                    }).collect_view()}
                </div>
            </div>
            <p class="text-sm leading-relaxed text-slate-600">{review.comment}</p>

            <Show when=move || reply.get().is_some()>
                <div class="mt-3 flex gap-2 rounded-lg bg-slate-50 p-3">
                    <Icon name="building" class="mt-0.5 h-4 w-4 shrink-0 text-blue-600" />
                    <div>
                        <p class="text-xs font-semibold text-slate-700">"Golden Tulip Addis Ababa responded"</p>
                        <p class="text-sm text-slate-600">{move || reply.get().unwrap_or_default()}</p>
                    </div>
                </div>
            </Show>

            <Show when=move || reply.get().is_none() && !replying.get()>
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
                        <button class="rounded-lg bg-blue-700 px-3 py-1.5 text-sm font-semibold text-white transition-all hover:bg-blue-800 active:scale-[0.98]" on:click=submit_reply>"Post Reply"</button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
