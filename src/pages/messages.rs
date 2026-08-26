use crate::components::Icon;
use crate::data::{ChatMessage, CONVERSATIONS};
use leptos::prelude::*;

#[component]
pub fn MessagesPage() -> impl IntoView {
    let conversations = RwSignal::new(
        CONVERSATIONS
            .iter()
            .map(|c| (c.guest, c.initials, c.booking_ref, c.last_message.to_string(), c.time, RwSignal::new(c.unread), RwSignal::new(c.thread.to_vec())))
            .collect::<Vec<_>>(),
    );
    let active = RwSignal::new(0usize);
    let draft = RwSignal::new(String::new());

    let do_send = move || {
        let text = draft.get();
        if text.trim().is_empty() {
            return;
        }
        conversations.update(|list| {
            if let Some((_, _, _, last, _, _, thread)) = list.get_mut(active.get()) {
                thread.update(|t| t.push(ChatMessage { from_guest: false, text: Box::leak(text.clone().into_boxed_str()), time: "Just now" }));
                *last = format!("You: {text}");
            }
        });
        draft.set(String::new());
    };

    view! {
        <div class="flex h-[calc(100vh-3.5rem)]">
            <div class="w-80 shrink-0 overflow-y-auto border-r border-slate-200 bg-white">
                <div class="border-b border-slate-200 p-4">
                    <h1 class="text-lg font-bold text-slate-900">"Messages"</h1>
                    <div class="relative mt-2">
                        <Icon name="search" class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                        <input type="text" placeholder="Search conversations" class="w-full rounded-lg border border-slate-300 py-2 pl-9 pr-3 text-sm" />
                    </div>
                </div>
                <div>
                    {move || conversations.get().into_iter().enumerate().map(|(i, (guest, initials, booking_ref, last, time, unread, _))| {
                        view! {
                            <button
                                on:click=move |_| { active.set(i); unread.set(0); }
                                class=move || format!(
                                    "flex w-full items-start gap-3 border-b border-slate-100 p-4 text-left transition-colors hover:bg-slate-50 {}",
                                    if active.get() == i { "bg-blue-50/60" } else { "" }
                                )
                            >
                                <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-blue-100 text-sm font-semibold text-blue-700">{initials}</span>
                                <span class="flex-1 overflow-hidden">
                                    <span class="flex items-center justify-between">
                                        <span class="truncate text-sm font-semibold text-slate-900">{guest}</span>
                                        <span class="shrink-0 text-xs text-slate-400">{time}</span>
                                    </span>
                                    <span class="block truncate text-xs text-slate-500">{booking_ref}</span>
                                    <span class="block truncate text-sm text-slate-500">{last}</span>
                                </span>
                                <Show when=move || (unread.get() > 0)>
                                    <span class="mt-1 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-red-500 text-xs font-semibold text-white">{unread.get()}</span>
                                </Show>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="flex flex-1 flex-col bg-slate-50">
                {move || {
                    let list = conversations.get();
                    let idx = active.get();
                    list.get(idx).cloned().map(|(guest, initials, booking_ref, _, _, _, thread)| {
                        view! {
                            <div class="flex items-center gap-3 border-b border-slate-200 bg-white p-4">
                                <span class="flex h-10 w-10 items-center justify-center rounded-full bg-blue-100 text-sm font-semibold text-blue-700">{initials}</span>
                                <div>
                                    <p class="font-semibold text-slate-900">{guest}</p>
                                    <p class="text-xs text-slate-400">{booking_ref}</p>
                                </div>
                                <div class="ml-auto flex items-center gap-3 text-slate-400">
                                    <Icon name="phone" class="h-4 w-4" />
                                    <Icon name="more-v" class="h-4 w-4" />
                                </div>
                            </div>

                            <div class="flex-1 overflow-y-auto p-4">
                                <div class="flex flex-col gap-3">
                                    {move || thread.get().into_iter().map(|m| view! {
                                        <div class=if m.from_guest { "flex justify-start" } else { "flex justify-end" }>
                                            <div class=if m.from_guest {
                                                "max-w-sm rounded-2xl rounded-tl-sm bg-white px-4 py-2 text-sm text-slate-700 shadow-sm"
                                            } else {
                                                "max-w-sm rounded-2xl rounded-tr-sm bg-blue-700 px-4 py-2 text-sm text-white shadow-sm"
                                            }>
                                                <p>{m.text}</p>
                                                <p class=if m.from_guest { "mt-1 text-[10px] text-slate-400" } else { "mt-1 text-[10px] text-blue-200" }>{m.time}</p>
                                            </div>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>

                            <div class="flex items-center gap-2 border-t border-slate-200 bg-white p-4">
                                <input
                                    type="text"
                                    placeholder="Type a message..."
                                    class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                                    prop:value=draft
                                    on:input:target=move |ev| draft.set(ev.target().value())
                                    on:keydown=move |ev| { if ev.key() == "Enter" { do_send(); } }
                                />
                                <button
                                    on:click=move |_| do_send()
                                    class="flex items-center gap-1.5 rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 active:scale-[0.98]"
                                >
                                    "Send"
                                    <Icon name="chevron-right" class="h-4 w-4" />
                                </button>
                            </div>
                        }
                    })
                }}
            </div>
        </div>
    }
}
