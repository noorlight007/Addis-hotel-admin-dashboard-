//! Bulk room import, backed by `/rooms/bulk-upload/`.
//!
//! Three real steps, each a call:
//!
//! 1. `GET  /rooms/bulk-upload/template/?format=csv` — the pre-formatted sheet.
//! 2. `POST /rooms/bulk-upload/validate/` — a dry run that parses the file and
//!    returns per-row diagnostics without writing anything.
//! 3. `POST /rooms/bulk-upload/import/` — the atomic commit.
//!
//! Both POSTs are `multipart/form-data` and take the same option switches, so
//! the preview reflects exactly what the commit will do.

use crate::api;
use crate::components::{use_toast, Card, Icon, PageHeader};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

fn validation_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "valid" => "bg-emerald-100 text-emerald-700",
        "warning" => "bg-amber-100 text-amber-700",
        _ => "bg-red-100 text-red-700",
    }
}

fn validation_icon(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "valid" => "check-circle",
        "warning" => "alert",
        _ => "x-circle",
    }
}

#[component]
pub fn BulkUploadPage() -> impl IntoView {
    let toast = use_toast();
    let navigate = use_navigate();
    let nav = StoredValue::new(navigate);

    // The picked file lives in a StoredValue: `web_sys::File` is not a signal
    // value, and it has to survive between the validate and import calls.
    let file = StoredValue::new(Option::<web_sys::File>::None);
    let file_name = RwSignal::new(Option::<String>::None);
    let preview = RwSignal::new(Option::<api::BulkValidation>::None);
    let result = RwSignal::new(Option::<api::BulkImportResult>::None);
    let error = RwSignal::new(Option::<String>::None);
    let validating = RwSignal::new(false);
    let importing = RwSignal::new(false);
    let downloading = RwSignal::new(false);

    // Import switches, shared by validate and import.
    let skip_duplicates = RwSignal::new(true);
    let update_existing = RwSignal::new(false);
    let default_available = RwSignal::new(true);
    let validate_amenities = RwSignal::new(true);

    let options = move || api::BulkOptions {
        skip_duplicates: skip_duplicates.get(),
        update_existing: update_existing.get(),
        default_available: default_available.get(),
        validate_amenities: validate_amenities.get(),
    };

    let download_template = move |_| {
        if downloading.get() {
            return;
        }
        downloading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api::download_bulk_template().await {
                Ok(()) => toast.success("Template downloaded", "rooms_upload_template.csv is in your downloads."),
                Err(e) => toast.error("Could not fetch the template", e.detail()),
            }
            downloading.set(false);
        });
    };

    // Runs the dry run as soon as a file is chosen, so the operator sees the
    // diagnostics without a second click.
    let validate_now = move || {
        let Some(f) = file.get_value() else {
            return;
        };
        validating.set(true);
        error.set(None);
        result.set(None);
        let opts = options();
        wasm_bindgen_futures::spawn_local(async move {
            match api::validate_bulk_rooms(&f, opts).await {
                Ok(v) => preview.set(Some(v)),
                Err(e) => {
                    preview.set(None);
                    error.set(Some(e.detail()));
                }
            }
            validating.set(false);
        });
    };

    let on_pick = move |ev: leptos::ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let picked = input.files().and_then(|list| list.get(0));
        match picked {
            Some(f) => {
                file_name.set(Some(f.name()));
                file.set_value(Some(f));
                validate_now();
            }
            None => {
                file_name.set(None);
                file.set_value(None);
                preview.set(None);
            }
        }
    };

    let run_import = move |_| {
        let Some(f) = file.get_value() else {
            toast.error("No file", "Choose a spreadsheet first.");
            return;
        };
        if importing.get() {
            return;
        }
        importing.set(true);
        error.set(None);
        let opts = options();
        wasm_bindgen_futures::spawn_local(async move {
            match api::import_bulk_rooms(&f, opts).await {
                Ok(r) => {
                    let imported = r.imported_count + r.updated_count;
                    if r.failed_count == 0 {
                        toast.success(
                            "Import complete",
                            format!("{imported} room(s) written to your inventory."),
                        );
                    } else {
                        toast.warning(
                            "Imported with errors",
                            format!("{imported} written, {} row(s) failed.", r.failed_count),
                        );
                    }
                    result.set(Some(r));
                }
                Err(e) => error.set(Some(e.detail())),
            }
            importing.set(false);
        });
    };

    let can_import = move || {
        preview
            .get()
            .is_some_and(|p| p.valid > 0 || (update_existing.get() && p.rows_detected > 0))
            && !importing.get()
    };

    view! {
        <div class="p-4 sm:p-6">
            <A href="/rooms" attr:class="mb-3 inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                <Icon name="chevron-left" class="h-4 w-4" />
                "Back to Rooms"
            </A>

            <PageHeader
                title="Bulk room upload"
                subtitle="Load your whole inventory from a CSV or Excel sheet."
            >
                <button
                    on:click=download_template
                    disabled=move || downloading.get()
                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition-colors hover:bg-slate-50 disabled:opacity-60"
                >
                    <Icon name="download" class="h-4 w-4" />
                    {move || if downloading.get() { "Fetching…" } else { "Download template" }}
                </button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="mb-5 flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2.5 text-sm text-red-700">
                    <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)]">
                // ---- Left: file + options -----------------------------
                <div class="flex flex-col gap-5">
                    <Card title="1. Choose your file" hint="CSV or Excel, up to 10 MB">
                        <label class="flex cursor-pointer flex-col items-center gap-2 rounded-xl border-2 border-dashed border-slate-300 px-4 py-8 text-center transition-colors hover:border-blue-400 hover:bg-blue-50/40">
                            <span class="flex h-11 w-11 items-center justify-center rounded-full bg-slate-100 text-slate-500">
                                <Icon name="upload" class="h-5 w-5" />
                            </span>
                            <span class="text-sm font-semibold text-slate-700">
                                {move || file_name.get().unwrap_or_else(|| "Click to choose a spreadsheet".into())}
                            </span>
                            <span class="text-2xs text-slate-400">".csv, .xlsx or .xls"</span>
                            <input
                                type="file"
                                accept=".csv,.xlsx,.xls"
                                class="hidden"
                                on:change=on_pick
                            />
                        </label>

                        <Show when=move || validating.get()>
                            <p class="mt-3 flex items-center justify-center gap-2 text-sm text-slate-500">
                                <Icon name="loader" class="h-4 w-4 animate-spin" />
                                "Validating…"
                            </p>
                        </Show>
                    </Card>

                    <Card title="2. Import options" hint="Applied to the preview and the commit">
                        <div class="flex flex-col gap-2">
                            <ImportSetting
                                icon="check-circle"
                                label="Skip duplicates"
                                hint="Leave existing room numbers untouched"
                                checked=skip_duplicates
                                on_change=move || validate_now()
                            />
                            <ImportSetting
                                icon="edit"
                                label="Update existing"
                                hint="Overwrite rooms that already have that number"
                                checked=update_existing
                                on_change=move || validate_now()
                            />
                            <ImportSetting
                                icon="bed"
                                label="Default to Available"
                                hint="Fill in a blank status column"
                                checked=default_available
                                on_change=move || validate_now()
                            />
                            <ImportSetting
                                icon="sparkles"
                                label="Map amenities"
                                hint="Link the amenity names to your catalogue"
                                checked=validate_amenities
                                on_change=move || validate_now()
                            />
                        </div>
                    </Card>

                    <Card title="Column format">
                        <p class="mb-2 text-sm text-slate-600">
                            "Headers are case-insensitive. Required columns:"
                        </p>
                        <ul class="flex flex-col gap-1.5 text-xs text-slate-600">
                            <Col name="Room Number" note="Unique per property" />
                            <Col name="Room Type" note="Standard, Deluxe, Twin, Family, Executive Suite, Single" />
                            <Col name="Floor" note="Whole number" />
                            <Col name="Capacity" note="Maximum guests" />
                            <Col name="Price per Night" note="Decimal, greater than zero" />
                            <Col name="Status" note="Optional — Available, Occupied, Reserved, Maintenance" />
                            <Col name="Amenities" note="Optional — comma-separated names" />
                        </ul>
                    </Card>
                </div>

                // ---- Right: preview + result --------------------------
                <div class="flex flex-col gap-5">
                    <Show when=move || result.get().is_some()>
                        {move || result.get().map(|r| {
                            let clean = r.failed_count == 0;
                            view! {
                                <div class=format!(
                                    "rounded-xl border p-5 {}",
                                    if clean { "border-emerald-200 bg-emerald-50/60" } else { "border-amber-200 bg-amber-50/60" }
                                )>
                                    <p class="mb-3 flex items-center gap-2 font-bold text-slate-900">
                                        <Icon name=if clean { "check-circle" } else { "alert" } class="h-5 w-5" />
                                        {if clean { "Import complete" } else { "Imported with errors" }}
                                    </p>
                                    <div class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
                                        <Tally label="Rows" value=r.total_rows />
                                        <Tally label="Created" value=r.imported_count />
                                        <Tally label="Updated" value=r.updated_count />
                                        <Tally label="Skipped" value=r.skipped_count />
                                    </div>
                                    {(!r.errors.is_empty()).then(|| view! {
                                        <ul class="mt-3 flex flex-col gap-1 border-t border-slate-200 pt-3 text-xs text-red-700">
                                            {r.errors.clone().into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}
                                        </ul>
                                    })}
                                    <button
                                        class="mt-4 rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800"
                                        on:click=move |_| nav.with_value(|n| n("/rooms", Default::default()))
                                    >
                                        "View room inventory"
                                    </button>
                                </div>
                            }
                        })}
                    </Show>

                    <Card
                        title="3. Preview"
                        hint=Signal::derive(move || match preview.get() {
                            Some(p) => format!(
                                "{} row(s) · {} valid · {} warning(s) · {} error(s)",
                                p.rows_detected, p.valid, p.warnings, p.errors,
                            ),
                            None => "Choose a file to see the parsed rows".into(),
                        })
                        action=Box::new(move || view! {
                            <button
                                on:click=run_import
                                disabled=move || !can_import()
                                class="rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-800 disabled:opacity-40"
                            >
                                {move || if importing.get() { "Importing…" } else { "Import rooms" }}
                            </button>
                        }.into_any())
                    >
                        {move || match preview.get() {
                            None => view! {
                                <p class="py-12 text-center text-sm text-slate-400">
                                    "Nothing to preview yet. The file is checked without writing anything, so it is safe to try."
                                </p>
                            }.into_any(),
                            Some(p) if p.preview_rows.is_empty() => view! {
                                <p class="py-12 text-center text-sm text-slate-400">
                                    "The sheet parsed but contained no data rows."
                                </p>
                            }.into_any(),
                            Some(p) => view! {
                                <div class="-mx-5 overflow-x-auto">
                                    <table class="w-full text-left text-sm">
                                        <thead class="border-b border-slate-100 text-2xs uppercase text-slate-500">
                                            <tr>
                                                <th class="px-5 py-2">"#"</th>
                                                <th class="px-5 py-2">"Room"</th>
                                                <th class="px-5 py-2">"Type"</th>
                                                <th class="px-5 py-2">"Floor"</th>
                                                <th class="px-5 py-2">"Capacity"</th>
                                                <th class="px-5 py-2">"Price"</th>
                                                <th class="px-5 py-2">"Amenities"</th>
                                                <th class="px-5 py-2">"Check"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {p.preview_rows.clone().into_iter().map(|row| {
                                                let status = row.validation_status.clone();
                                                view! {
                                                    <tr class=format!(
                                                        "border-b border-slate-50 last:border-0 {}",
                                                        if row.is_error() { "bg-red-50/40" } else { "" }
                                                    )>
                                                        <td class="px-5 py-2.5 tabular-nums text-slate-400">{row.row_number}</td>
                                                        <td class="px-5 py-2.5 font-semibold text-slate-900">
                                                            {row.room_number.clone().unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="px-5 py-2.5 text-slate-600">
                                                            {row.room_type.clone().unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="px-5 py-2.5 tabular-nums">
                                                            {row.floor.map(|f| f.to_string()).unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="px-5 py-2.5 tabular-nums">
                                                            {row.guest_capacity.map(|c| c.to_string()).unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="px-5 py-2.5 tabular-nums">
                                                            {row.price_per_night
                                                                .map(|p| api::money_round(Some(&format!("{p:.2}"))))
                                                                .unwrap_or_else(|| "—".into())}
                                                        </td>
                                                        <td class="px-5 py-2.5 max-w-[14rem] truncate text-2xs text-slate-500">
                                                            {row.amenities.join(", ")}
                                                        </td>
                                                        <td class="px-5 py-2.5">
                                                            <span
                                                                title=row.message.clone().unwrap_or_default()
                                                                class=format!(
                                                                    "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs font-semibold {}",
                                                                    validation_color(&status),
                                                                )
                                                            >
                                                                <Icon name=validation_icon(&status) class="h-3 w-3" />
                                                                {status.clone()}
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>

                                {(p.errors > 0).then(|| view! {
                                    <div class="mt-4 rounded-lg bg-red-50 px-3 py-2.5 text-xs text-red-700">
                                        <p class="mb-1 font-bold">"Rows with errors are skipped by the import."</p>
                                        <ul class="flex flex-col gap-1">
                                            {p.preview_rows.clone().into_iter().filter(|r| r.is_error()).map(|r| view! {
                                                <li>{format!("Row {}: {}", r.row_number, r.message.clone().unwrap_or_default())}</li>
                                            }).collect_view()}
                                        </ul>
                                    </div>
                                })}
                            }.into_any(),
                        }}
                    </Card>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ImportSetting(
    icon: &'static str,
    label: &'static str,
    hint: &'static str,
    checked: RwSignal<bool>,
    /// Re-runs the dry run so the preview matches the switches.
    on_change: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <label class="flex cursor-pointer items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2.5">
            <span class="flex min-w-0 items-center gap-2.5">
                <Icon name=icon class="h-4 w-4 shrink-0 text-slate-400" />
                <span class="min-w-0">
                    <span class="block text-sm font-medium text-slate-800">{label}</span>
                    <span class="block text-2xs text-slate-500">{hint}</span>
                </span>
            </span>
            <input
                type="checkbox"
                class="h-4 w-4 shrink-0"
                prop:checked=checked
                on:change:target=move |ev| {
                    checked.set(ev.target().checked());
                    on_change();
                }
            />
        </label>
    }
}

#[component]
fn Tally(label: &'static str, value: u32) -> impl IntoView {
    view! {
        <div>
            <p class="text-2xs uppercase tracking-wide text-slate-500">{label}</p>
            <p class="text-lg font-bold tabular-nums text-slate-900">{value}</p>
        </div>
    }
}

#[component]
fn Col(name: &'static str, note: &'static str) -> impl IntoView {
    view! {
        <li class="flex flex-wrap items-baseline gap-2">
            <code class="rounded bg-slate-100 px-1.5 py-0.5 font-semibold text-slate-700">{name}</code>
            <span class="text-slate-500">{note}</span>
        </li>
    }
}
