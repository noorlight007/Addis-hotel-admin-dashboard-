use crate::components::{Icon, Toggle};
use crate::pages::dashboard::status_dot;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

struct PreviewRow {
    number: &'static str,
    room_type: &'static str,
    floor: u32,
    capacity: &'static str,
    price: &'static str,
    status: &'static str,
    validation: &'static str,
    error: &'static str,
}

const PREVIEW_ROWS: &[PreviewRow] = &[
    PreviewRow { number: "101", room_type: "Standard Room", floor: 1, capacity: "2 Guests", price: "ETB 4,500", status: "Available", validation: "Valid", error: "" },
    PreviewRow { number: "102", room_type: "Deluxe Room", floor: 1, capacity: "2 Guests", price: "ETB 6,200", status: "Occupied", validation: "Valid", error: "" },
    PreviewRow { number: "301", room_type: "Executive Suite", floor: 3, capacity: "4 Guests", price: "ETB 9,800", status: "Maintenance", validation: "Warning", error: "" },
    PreviewRow { number: "202", room_type: "Deluxe Room", floor: 2, capacity: "2 Guests", price: "ETB 6,200", status: "Reserved", validation: "Error", error: "Duplicate room number: 202 already exists." },
    PreviewRow { number: "401", room_type: "Family Room", floor: 4, capacity: "4 Guests", price: "—", status: "Available", validation: "Error", error: "Price per night is required." },
];

fn validation_color(validation: &str) -> &'static str {
    match validation {
        "Valid" => "text-emerald-700",
        "Warning" => "text-amber-600",
        _ => "text-red-600",
    }
}

fn validation_icon(validation: &str) -> &'static str {
    match validation {
        "Valid" => "check-circle",
        "Warning" => "info",
        _ => "x-circle",
    }
}

fn download_template() {
    let csv = "Room Number,Room Type,Floor,Capacity,Price per Night,Status,Amenities\n101,Standard Room,1,2,4500,Available,\"Wi-Fi,TV,AC\"\n";
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(csv));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv");
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &options) {
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Ok(a) = document.create_element("a") {
                        let a: web_sys::HtmlAnchorElement = a.unchecked_into();
                        a.set_href(&url);
                        a.set_download("room_upload_template.csv");
                        a.click();
                    }
                }
            }
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UploadState {
    Idle,
    Selected,
    Validating,
    Validated,
}

#[component]
pub fn BulkUploadPage() -> impl IntoView {
    let navigate = use_navigate();
    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();
    let file_name = RwSignal::new(String::new());
    let state = RwSignal::new(UploadState::Idle);
    let skip_duplicates = RwSignal::new(true);
    let update_existing = RwSignal::new(false);
    let set_available = RwSignal::new(true);
    let validate_amenities = RwSignal::new(true);

    let trigger_browse = move |_| {
        if let Some(el) = file_input.get() {
            el.click();
        }
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let target: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                file_name.set(file.name());
                state.set(UploadState::Selected);
            }
        }
    };

    let clear_file = move |_| {
        file_name.set(String::new());
        state.set(UploadState::Idle);
        if let Some(el) = file_input.get() {
            el.set_value("");
        }
    };

    let validate = move |_| {
        if state.get() != UploadState::Selected {
            return;
        }
        state.set(UploadState::Validating);
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(900).await;
            state.set(UploadState::Validated);
        });
    };

    let import = {
        let navigate = navigate.clone();
        move |_| {
            if state.get() == UploadState::Validated {
                navigate("/rooms", Default::default());
            }
        }
    };

    view! {
        <div class="p-4 sm:p-6">
            <A href="/rooms" attr:class="mb-3 inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                <Icon name="chevron-left" class="h-4 w-4" />
                "Back to Rooms"
            </A>
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl font-bold text-slate-900">"Bulk Upload Rooms"</h1>
                    <p class="text-sm text-slate-500">"Import multiple rooms at once using a spreadsheet template."</p>
                </div>
                <div class="flex gap-2">
                    <button on:click=move |_| download_template() class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50">
                        <Icon name="download" class="h-4 w-4" />
                        "Download Template"
                    </button>
                    <button on:click=trigger_browse class="flex items-center gap-1.5 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-3 py-2 text-sm font-semibold text-white">
                        <Icon name="upload" class="h-4 w-4" />
                        "Upload File"
                    </button>
                </div>
            </div>

            <input
                type="file"
                accept=".xlsx,.csv"
                node_ref=file_input
                on:change=on_file_change
                class="hidden"
            />

            <div class="mb-6 grid gap-3 sm:grid-cols-3">
                <div class=move || format!("flex items-start gap-3 rounded-xl border p-4 {}", if state.get() == UploadState::Idle { "border-blue-300 bg-blue-50/60" } else { "border-green-200 bg-emerald-50/60" })>
                    <span class=move || format!("flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-semibold text-white {}", if state.get() == UploadState::Idle { "bg-blue-600" } else { "bg-green-600" })>
                        <Show when=move || state.get() != UploadState::Idle fallback=|| "1">
                            <Icon name="check" class="h-3.5 w-3.5" />
                        </Show>
                    </span>
                    <span>
                        <span class="block text-sm font-semibold text-slate-800">"Download Template"</span>
                        <span class="text-xs text-slate-500">"Use our sample Excel/CSV format."</span>
                    </span>
                </div>
                <div class=move || format!("flex items-start gap-3 rounded-xl border p-4 {}", if state.get() == UploadState::Validated { "border-green-200 bg-emerald-50/60" } else if state.get() == UploadState::Idle { "border-slate-200" } else { "border-blue-300 bg-blue-50/60" })>
                    <span class=move || format!("flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-semibold text-white {}", if state.get() == UploadState::Validated { "bg-green-600" } else if state.get() == UploadState::Idle { "bg-slate-300" } else { "bg-blue-600" })>
                        <Show when=move || state.get() == UploadState::Validated fallback=|| "2">
                            <Icon name="check" class="h-3.5 w-3.5" />
                        </Show>
                    </span>
                    <span>
                        <span class="block text-sm font-semibold text-slate-800">"Upload & Validate"</span>
                        <span class="text-xs text-slate-500">"Check your file for errors before import."</span>
                    </span>
                </div>
                <div class=move || format!("flex items-start gap-3 rounded-xl border p-4 {}", if state.get() == UploadState::Validated { "border-blue-300 bg-blue-50/60" } else { "border-slate-200" })>
                    <span class=move || format!("flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-semibold text-white {}", if state.get() == UploadState::Validated { "bg-blue-600" } else { "bg-slate-300" })>"3"</span>
                    <span>
                        <span class="block text-sm font-semibold text-slate-800">"Review & Import"</span>
                        <span class="text-xs text-slate-500">"Confirm the valid data and finalize import."</span>
                    </span>
                </div>
            </div>

            <div class="grid gap-6 lg:grid-cols-3">
                <div class="lg:col-span-2">
                    <h2 class="mb-2 text-sm font-semibold text-slate-900">"Upload Spreadsheet"</h2>
                    <div
                        on:click=trigger_browse
                        class="cursor-pointer rounded-xl border-2 border-dashed border-slate-300 bg-white p-10 text-center transition-colors hover:border-blue-300"
                    >
                        <Icon name="upload" class="mx-auto h-8 w-8 text-blue-600" />
                        <p class="mt-2 font-medium text-slate-800">"Drag & drop your file here"</p>
                        <p class="text-sm text-slate-500">"Supports .xlsx and .csv files up to 10 MB"</p>
                        <button type="button" class="mt-4 rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white">"Browse Files"</button>
                    </div>

                    <Show when=move || state.get() != UploadState::Idle>
                        <div class="mt-4 flex items-center justify-between rounded-xl border border-slate-200 bg-white p-3 text-sm">
                            <span class="flex items-center gap-2">
                                <Icon name="image" class="h-5 w-5 text-emerald-600" />
                                <span>
                                    <span class="font-medium text-slate-800">{move || file_name.get()}</span>
                                    <span class="ml-2 text-xs text-slate-400">
                                        {move || match state.get() {
                                            UploadState::Selected => "Ready to validate",
                                            UploadState::Validating => "Validating...",
                                            UploadState::Validated => "Validated successfully",
                                            UploadState::Idle => "",
                                        }}
                                    </span>
                                </span>
                            </span>
                            <button on:click=clear_file><Icon name="x" class="h-4 w-4 text-slate-400 hover:text-slate-600" /></button>
                        </div>
                    </Show>

                    <div class="mt-6 rounded-xl border border-slate-200 bg-white p-4">
                        <h2 class="mb-3 text-sm font-semibold text-slate-900">"Import Settings"</h2>
                        <div class="flex flex-col divide-y divide-slate-100">
                            <ImportSettingRow icon="edit" label="Skip duplicate room numbers" hint="Ignore rows with room numbers that already exist." checked=skip_duplicates />
                            <ImportSettingRow icon="edit" label="Update existing rooms if matched" hint="Update details for rooms with matching room numbers." checked=update_existing />
                            <ImportSettingRow icon="check-circle" label="Set newly imported rooms as Available by default" hint="All new rooms will be marked as Available." checked=set_available />
                            <ImportSettingRow icon="sliders" label="Validate amenities against master list" hint="Ensure amenities match the predefined master list." checked=validate_amenities />
                        </div>
                    </div>

                    <Show when=move || state.get() == UploadState::Validated>
                        <div class="mt-6">
                            <h2 class="mb-3 text-sm font-semibold text-slate-900">"Validation Summary"</h2>
                            <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
                                <div class="animate-fade-up rounded-xl border border-slate-200 bg-white p-4 text-center">
                                    <Icon name="edit" class="mx-auto mb-1 h-5 w-5 text-slate-400" />
                                    <p class="text-2xl font-bold text-slate-900">"48"</p>
                                    <p class="text-xs text-slate-500">"Rows Detected"</p>
                                </div>
                                <div class="animate-fade-up rounded-xl border border-slate-200 bg-white p-4 text-center" style="animation-delay: 60ms">
                                    <Icon name="check-circle" class="mx-auto mb-1 h-5 w-5 text-emerald-600" />
                                    <p class="text-2xl font-bold text-emerald-600">"43"</p>
                                    <p class="text-xs text-slate-500">"Valid"</p>
                                </div>
                                <div class="animate-fade-up rounded-xl border border-slate-200 bg-white p-4 text-center" style="animation-delay: 120ms">
                                    <Icon name="info" class="mx-auto mb-1 h-5 w-5 text-amber-600" />
                                    <p class="text-2xl font-bold text-amber-600">"3"</p>
                                    <p class="text-xs text-slate-500">"Warnings"</p>
                                </div>
                                <div class="animate-fade-up rounded-xl border border-slate-200 bg-white p-4 text-center" style="animation-delay: 180ms">
                                    <Icon name="x-circle" class="mx-auto mb-1 h-5 w-5 text-red-600" />
                                    <p class="text-2xl font-bold text-red-600">"2"</p>
                                    <p class="text-xs text-slate-500">"Errors"</p>
                                </div>
                            </div>
                        </div>

                        <div class="mt-6">
                            <h2 class="mb-3 text-sm font-semibold text-slate-900">"Preview of Uploaded Data"</h2>
                            <div class="overflow-x-auto rounded-xl border border-slate-200 bg-white">
                                <table class="w-full text-left text-sm">
                                    <thead class="border-b border-slate-200 text-xs uppercase text-slate-500">
                                        <tr>
                                            <th class="px-4 py-3">"Room Number"</th>
                                            <th class="px-4 py-3">"Room Type"</th>
                                            <th class="px-4 py-3">"Floor"</th>
                                            <th class="px-4 py-3">"Capacity"</th>
                                            <th class="px-4 py-3">"Price/Night"</th>
                                            <th class="px-4 py-3">"Status"</th>
                                            <th class="px-4 py-3">"Validation"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {PREVIEW_ROWS.iter().map(|row| {
                                            let (dot, text) = status_dot(row.status);
                                            view! {
                                                <tr class="border-b border-slate-100 last:border-0">
                                                    <td class="px-4 py-3 font-medium text-slate-900">{row.number}</td>
                                                    <td class="px-4 py-3">{row.room_type}</td>
                                                    <td class="px-4 py-3">{row.floor}</td>
                                                    <td class="px-4 py-3">{row.capacity}</td>
                                                    <td class="px-4 py-3">{row.price}</td>
                                                    <td class="px-4 py-3">
                                                        <span class=format!("flex items-center gap-1.5 text-xs font-semibold {text}")>
                                                            <span class=format!("h-1.5 w-1.5 rounded-full {dot}")></span>
                                                            {row.status}
                                                        </span>
                                                    </td>
                                                    <td class="px-4 py-3">
                                                        <span class=format!("flex items-center gap-1 text-xs font-semibold {}", validation_color(row.validation))>
                                                            <Icon name=validation_icon(row.validation) class="h-3.5 w-3.5" />
                                                            {row.validation}
                                                        </span>
                                                        <Show when=move || !row.error.is_empty()>
                                                            <p class="mt-0.5 text-xs text-red-600">{row.error}</p>
                                                        </Show>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="mt-3 flex items-center justify-between text-sm text-slate-500">
                                <p>"Showing 1 to 5 of 48 rows"</p>
                                <div class="flex items-center gap-1">
                                    <button class="rounded-lg border border-slate-300 bg-white px-2 py-1 text-slate-400"><Icon name="chevron-left" class="h-4 w-4" /></button>
                                    <span class="rounded-lg bg-blue-700 px-3 py-1 font-semibold text-white">"1"</span>
                                    <span class="rounded-lg px-3 py-1 hover:bg-slate-50">"2"</span>
                                    <span class="rounded-lg px-3 py-1 hover:bg-slate-50">"3"</span>
                                    <span class="px-1">"…"</span>
                                    <span class="rounded-lg px-3 py-1 hover:bg-slate-50">"10"</span>
                                    <button class="rounded-lg border border-slate-300 bg-white px-2 py-1 text-slate-400"><Icon name="chevron-right" class="h-4 w-4" /></button>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>

                <div class="flex flex-col gap-4">
                    <div class="rounded-xl border border-slate-200 bg-white p-4">
                        <h2 class="mb-2 text-sm font-semibold text-slate-900">"Template Guidelines"</h2>
                        <p class="mb-2 text-xs text-slate-500">"Your spreadsheet must include the following columns:"</p>
                        <ul class="flex flex-col gap-2 text-sm text-slate-600">
                            {[
                                ("Room Number (Required)", ""),
                                ("Room Type (Required)", ""),
                                ("Floor (Required)", ""),
                                ("Capacity (Required)", ""),
                                ("Price per Night (Required)", ""),
                                ("Status (Required)", "Available, Occupied, Reserved, Maintenance"),
                                ("Amenities (Optional)", "Comma-separated list"),
                            ].iter().map(|(t, hint)| view! {
                                <li class="flex items-start gap-2">
                                    <Icon name="check-circle" class="mt-0.5 h-4 w-4 shrink-0 text-emerald-600" />
                                    <span>
                                        <span class="block">{*t}</span>
                                        <Show when=move || !hint.is_empty()>
                                            <span class="block text-xs text-slate-400">{*hint}</span>
                                        </Show>
                                    </span>
                                </li>
                            }).collect_view()}
                        </ul>
                    </div>
                    <div class="rounded-xl border border-slate-200 bg-white p-4">
                        <h2 class="mb-2 text-sm font-semibold text-slate-900">"Accepted Room Types"</h2>
                        <div class="flex flex-wrap gap-2 text-xs">
                            {["Standard Room", "Deluxe Room", "Twin Room", "Family Room", "Executive Suite", "Single Room"].iter().map(|t| view! {
                                <span class="rounded-full bg-slate-100 px-2 py-1 text-slate-600">{*t}</span>
                            }).collect_view()}
                        </div>
                    </div>
                    <div class="rounded-xl border border-slate-200 bg-white p-4">
                        <h2 class="mb-2 text-sm font-semibold text-slate-900">"Import Tips"</h2>
                        <ul class="flex flex-col gap-2 text-sm text-slate-600">
                            {[
                                ("Use unique room numbers", "Each room number must be unique."),
                                ("Amenities can be comma-separated", "E.g., Wi-Fi, TV, AC, Breakfast"),
                                ("Status values: Available, Occupied, Reserved, Maintenance", ""),
                                ("Leave blank cells only for optional fields", ""),
                            ].iter().map(|(t, hint)| view! {
                                <li class="flex items-start gap-2">
                                    <span class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-blue-600"></span>
                                    <span>
                                        <span class="block">{*t}</span>
                                        <Show when=move || !hint.is_empty()>
                                            <span class="block text-xs text-slate-400">{*hint}</span>
                                        </Show>
                                    </span>
                                </li>
                            }).collect_view()}
                        </ul>
                    </div>
                    <div class="rounded-xl border border-slate-200 bg-white p-4">
                        <h2 class="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-900">
                            <Icon name="message" class="h-4 w-4 text-blue-600" />
                            "Need help?"
                        </h2>
                        <p class="text-sm text-slate-500">"Download the sample template or contact support if your file fails validation."</p>
                        <a href="#" class="mt-2 flex items-center gap-1 text-sm font-medium text-blue-700 hover:underline">
                            "View Import Guide"
                            <Icon name="external-link" class="h-3.5 w-3.5" />
                        </a>
                    </div>
                </div>
            </div>

            <div class="mt-6 flex justify-end gap-3">
                <A href="/rooms" attr:class="rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium transition-colors hover:bg-slate-50">"Cancel"</A>
                <button disabled=move || state.get() == UploadState::Idle class="rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium transition-colors hover:bg-slate-50 disabled:opacity-40">"Save as Draft"</button>
                {move || {
                    let import = import.clone();
                    match state.get() {
                    UploadState::Validated => view! {
                        <button on:click=import class="rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white">"Import Rooms"</button>
                    }.into_any(),
                    UploadState::Selected => view! {
                        <button on:click=validate class="rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white">"Validate & Continue"</button>
                    }.into_any(),
                    UploadState::Validating => view! {
                        <button disabled class="rounded-lg bg-blue-400 px-4 py-2 text-sm font-semibold text-white">"Validating..."</button>
                    }.into_any(),
                    UploadState::Idle => view! {
                        <button disabled class="rounded-lg bg-slate-300 px-4 py-2 text-sm font-semibold text-white">"Validate & Continue"</button>
                    }.into_any(),
                }}}
            </div>
        </div>
    }
}

#[component]
fn ImportSettingRow(icon: &'static str, label: &'static str, hint: &'static str, checked: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-3">
            <span class="flex items-start gap-3">
                <Icon name=icon class="mt-0.5 h-4 w-4 text-slate-400" />
                <span>
                    <span class="block text-sm font-medium text-slate-800">{label}</span>
                    <span class="block text-xs text-slate-400">{hint}</span>
                </span>
            </span>
            <button type="button" on:click=move |_| checked.update(|v| *v = !*v)>
                <Toggle checked=checked.get() />
            </button>
        </div>
    }
}
