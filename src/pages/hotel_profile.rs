//! The hotel itself, backed by `/organizations/`.
//!
//! This is the one screen that works before a property exists: a fresh
//! `organization_admin` has no hotel, so every other scoped endpoint returns
//! 403 until one is created here. When `session::active_org()` is empty the page
//! shows the onboarding form and `POST /organizations/` instead of the tabs.
//!
//! Once a hotel exists the tabs map to its sub-resources:
//!
//! | Tab       | Endpoint |
//! |-----------|----------|
//! | Profile   | `PATCH /organizations/{id}/` |
//! | Photos    | `/organizations/{id}/gallery/` |
//! | Amenities | `/amenities/`, `/amenities/create/`, `/amenities/organization/sync/` |
//! | Policies  | `/organizations/{id}/policies/` |
//! | Contacts  | `/organizations/{id}/contacts/` |

use crate::api;
use crate::components::{use_toast, Badge, Card, EmptyState, Icon, PageHeader};
use leptos::prelude::*;

const TABS: &[&str] = &["Profile", "Photos", "Amenities", "Policies", "Contacts"];

#[component]
pub fn HotelProfilePage() -> impl IntoView {
    let tab = RwSignal::new(TABS[0]);
    let refresh = RwSignal::new(0u32);

    let org = LocalResource::new(move || {
        let _ = refresh.get();
        async move {
            match api::require_org() {
                Ok(_) => api::active_organization().await.map(Some),
                // No property yet — not an error, just the onboarding state.
                Err(_) => Ok(None),
            }
        }
    });

    let bump = move || refresh.update(|n| *n += 1);

    view! {
        <div class="p-4 sm:p-6">
            <Suspense fallback=|| view! {
                <p class="py-16 text-center text-sm text-slate-400">"Loading your property…"</p>
            }>
                {move || Suspend::new(async move {
                    let loaded = match org.await {
                        Ok(o) => o,
                        Err(e) => return view! {
                            <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                        }.into_any(),
                    };

                    let Some(hotel) = loaded else {
                        return view! { <CreateHotel on_created=move || bump() /> }.into_any();
                    };

                    let h_profile = hotel.clone();
                    let h_photos = hotel.clone();
                    let h_contacts = hotel.clone();

                    view! {
                        <PageHeader
                            title=hotel.name.clone()
                            subtitle=hotel.location()
                        >
                            <Badge label=format!("{:.1} stars", hotel.stars()) tone="amber" />
                            <Badge label=hotel.currency_code().to_string() tone="slate" />
                        </PageHeader>

                        <div class="mb-5 flex flex-wrap gap-2 text-sm">
                            {TABS.iter().map(|t| {
                                let t = *t;
                                view! {
                                    <button
                                        class=move || format!(
                                            "rounded-lg px-4 py-2 font-semibold transition-all duration-200 {}",
                                            if tab.get() == t { "bg-blue-700 text-white shadow-md active:scale-[0.98]" } else { "border border-slate-300 bg-white text-slate-600 hover:bg-slate-50" }
                                        )
                                        on:click=move |_| tab.set(t)
                                    >
                                        {t}
                                    </button>
                                }
                            }).collect_view()}
                        </div>

                        <Show when=move || (tab.get() == "Profile")>
                            <ProfileTab hotel=h_profile.clone() on_saved=move || bump() />
                        </Show>
                        <Show when=move || (tab.get() == "Photos")>
                            <PhotosTab hotel=h_photos.clone() on_changed=move || bump() />
                        </Show>
                        <Show when=move || (tab.get() == "Amenities")>
                            <AmenitiesTab/>
                        </Show>
                        <Show when=move || (tab.get() == "Policies")>
                            <PoliciesTab/>
                        </Show>
                        <Show when=move || (tab.get() == "Contacts")>
                            <ContactsTab hotel=h_contacts.clone() />
                        </Show>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Onboarding
// ---------------------------------------------------------------------------

/// Shown when the signed-in admin has no property yet.
///
/// `POST /organizations/` also seeds default policies and makes the caller the
/// hotel admin, which is what unlocks every other scoped endpoint.
#[component]
fn CreateHotel(on_created: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let toast = use_toast();
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let stars = RwSignal::new("3".to_string());
    let currency = RwSignal::new("ETB".to_string());
    let address = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let country = RwSignal::new("Ethiopia".to_string());
    let postal = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if name.get().trim().is_empty() {
            error.set(Some("Give your property a name.".into()));
            return;
        }
        if !email.get().contains('@') {
            error.set(Some("Enter the hotel's contact email address.".into()));
            return;
        }
        if phone.get().trim().is_empty() {
            error.set(Some("Enter the hotel's phone number.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let payload = api::Organization {
            name: name.get().trim().to_string(),
            description: opt(description.get()),
            currency: opt(currency.get()),
            address_line: opt(address.get()),
            city: opt(city.get()),
            country: opt(country.get()),
            postal_code: opt(postal.get()),
            email: opt(email.get()),
            phone_number: opt(phone.get()),
            ..Default::default()
        };
        let rating = stars.get().parse::<f32>().unwrap_or(3.0).clamp(1.0, 5.0);

        wasm_bindgen_futures::spawn_local(async move {
            match api::create_organization(&payload, rating).await {
                Ok(o) => {
                    busy.set(false);
                    toast.success("Property created", format!("{} is ready. Add your rooms next.", o.name));
                    on_created();
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <div class="mx-auto max-w-2xl">
            <div class="mb-6 text-center">
                <span class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-blue-50 text-blue-600 ring-1 ring-blue-100">
                    <Icon name="building" class="h-6 w-6" />
                </span>
                <h1 class="text-xl font-bold text-slate-900">"Set up your property"</h1>
                <p class="mx-auto mt-1 max-w-md text-sm text-slate-500">
                    "The dashboard is scoped to one hotel. Create yours and the rooms, reservations, guests and analytics screens all unlock."
                </p>
            </div>

            <Card>
                <form on:submit=submit class="flex flex-col gap-4">
                    <Show when=move || error.get().is_some()>
                        <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                            <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <Field label="Hotel name" value=name placeholder="e.g. Golden Tulip Addis Ababa" />

                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Description"</label>
                        <textarea rows="3" placeholder="What makes the property worth staying at"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=description on:input:target=move |ev| description.set(ev.target().value())></textarea>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Star rating"</label>
                            <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                on:change:target=move |ev| stars.set(ev.target().value())
                            >
                                {(1..=5).map(|n| view! {
                                    <option value=n.to_string() selected=(n == 3)>{format!("{n} star")}</option>
                                }).collect_view()}
                            </select>
                        </div>
                        <Field label="Currency" value=currency placeholder="ETB" />
                    </div>

                    <Field label="Street address" value=address placeholder="e.g. Bole, Africa Avenue" />

                    <div class="grid grid-cols-3 gap-4">
                        <Field label="City" value=city placeholder="Addis Ababa" />
                        <Field label="Country" value=country />
                        <Field label="Postal code" value=postal />
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <Field label="Contact email" value=email kind="email" />
                        <Field label="Phone number" value=phone kind="tel" placeholder="+251 …" />
                    </div>

                    <div class="mt-1 flex justify-end border-t border-slate-100 pt-4">
                        <button type="submit" disabled=move || busy.get()
                            class="rounded-lg bg-blue-700 px-5 py-2.5 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                            {move || if busy.get() { "Creating…" } else { "Create property" }}
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

#[component]
fn ProfileTab(
    hotel: api::Organization,
    on_saved: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let id = hotel.id;
    let name = RwSignal::new(hotel.name.clone());
    let description = RwSignal::new(hotel.description.clone().unwrap_or_default());
    let stars = RwSignal::new(format!("{:.0}", hotel.stars().max(1.0)));
    let currency = RwSignal::new(hotel.currency_code().to_string());
    let address = RwSignal::new(hotel.address_line.clone().unwrap_or_default());
    let city = RwSignal::new(hotel.city.clone().unwrap_or_default());
    let country = RwSignal::new(hotel.country.clone().unwrap_or_default());
    let postal = RwSignal::new(hotel.postal_code.clone().unwrap_or_default());
    let email = RwSignal::new(hotel.email.clone().unwrap_or_default());
    let phone = RwSignal::new(hotel.phone_number.clone().unwrap_or_default());
    let whatsapp = RwSignal::new(hotel.whatsapp.clone().unwrap_or_default());
    let website = RwSignal::new(hotel.website_url.clone().unwrap_or_default());
    let facebook = RwSignal::new(hotel.fb_link.clone().unwrap_or_default());
    let instagram = RwSignal::new(hotel.instagram_link.clone().unwrap_or_default());
    let established = RwSignal::new(
        hotel
            .established_year
            .map(|y| y.to_string())
            .unwrap_or_default(),
    );
    let reg_no = RwSignal::new(hotel.business_registration_number.clone().unwrap_or_default());
    let tin = RwSignal::new(hotel.tin_number.clone().unwrap_or_default());
    let vat = RwSignal::new(hotel.vat_number.clone().unwrap_or_default());
    let error = RwSignal::new(Option::<String>::None);
    let busy = RwSignal::new(false);

    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get() {
            return;
        }
        if name.get().trim().is_empty() {
            error.set(Some("The property needs a name.".into()));
            return;
        }
        error.set(None);
        busy.set(true);

        let payload = api::Organization {
            name: name.get().trim().to_string(),
            description: opt(description.get()),
            currency: opt(currency.get()),
            address_line: opt(address.get()),
            city: opt(city.get()),
            country: opt(country.get()),
            postal_code: opt(postal.get()),
            email: opt(email.get()),
            phone_number: opt(phone.get()),
            whatsapp: opt(whatsapp.get()),
            website_url: opt(website.get()),
            fb_link: opt(facebook.get()),
            instagram_link: opt(instagram.get()),
            established_year: established.get().trim().parse().ok(),
            business_registration_number: opt(reg_no.get()),
            tin_number: opt(tin.get()),
            vat_number: opt(vat.get()),
            ..Default::default()
        };
        let rating = stars.get().parse::<f32>().ok().map(|r| r.clamp(1.0, 5.0));

        wasm_bindgen_futures::spawn_local(async move {
            match api::update_organization(id, &payload, rating).await {
                Ok(_) => {
                    busy.set(false);
                    toast.success("Property saved", "The public listing has been updated.");
                    on_saved();
                }
                Err(e) => {
                    error.set(Some(e.detail()));
                    busy.set(false);
                }
            }
        });
    };

    view! {
        <form on:submit=save class="flex animate-fade-up flex-col gap-5">
            <Show when=move || error.get().is_some()>
                <div class="flex items-start gap-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                    <Icon name="info" class="mt-0.5 h-4 w-4 shrink-0" />
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="grid gap-5 lg:grid-cols-2">
                <Card title="Identity" hint="What guests see first">
                    <div class="flex flex-col gap-4">
                        <Field label="Hotel name" value=name />
                        <div>
                            <label class="mb-1 block text-xs font-medium text-slate-500">"Description"</label>
                            <textarea rows="4" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                prop:value=description on:input:target=move |ev| description.set(ev.target().value())></textarea>
                        </div>
                        <div class="grid grid-cols-3 gap-4">
                            <div>
                                <label class="mb-1 block text-xs font-medium text-slate-500">"Stars"</label>
                                <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                    on:change:target=move |ev| stars.set(ev.target().value())
                                >
                                    {(1..=5).map(|n| view! {
                                        <option value=n.to_string() selected=move || stars.get() == n.to_string()>{n}</option>
                                    }).collect_view()}
                                </select>
                            </div>
                            <Field label="Currency" value=currency />
                            <Field label="Established" value=established kind="number" />
                        </div>
                    </div>
                </Card>

                <Card title="Location">
                    <div class="flex flex-col gap-4">
                        <Field label="Street address" value=address />
                        <div class="grid grid-cols-2 gap-4">
                            <Field label="City" value=city />
                            <Field label="Country" value=country />
                        </div>
                        <Field label="Postal code" value=postal />
                    </div>
                </Card>

                <Card title="Contact">
                    <div class="flex flex-col gap-4">
                        <Field label="Email" value=email kind="email" />
                        <div class="grid grid-cols-2 gap-4">
                            <Field label="Phone" value=phone kind="tel" />
                            <Field label="WhatsApp" value=whatsapp kind="tel" />
                        </div>
                        <Field label="Website" value=website kind="url" placeholder="https://" />
                        <div class="grid grid-cols-2 gap-4">
                            <Field label="Facebook" value=facebook kind="url" placeholder="https://" />
                            <Field label="Instagram" value=instagram kind="url" placeholder="https://" />
                        </div>
                    </div>
                </Card>

                <Card title="Legal & tax" hint="Kept private, used on invoices">
                    <div class="flex flex-col gap-4">
                        <Field label="Business registration number" value=reg_no />
                        <Field label="TIN number" value=tin />
                        <Field label="VAT number" value=vat />
                    </div>
                </Card>
            </div>

            <div class="flex justify-end">
                <button type="submit" disabled=move || busy.get()
                    class="rounded-lg bg-blue-700 px-5 py-2.5 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                    {move || if busy.get() { "Saving…" } else { "Save property" }}
                </button>
            </div>
        </form>
    }
}

// ---------------------------------------------------------------------------
// Photos
// ---------------------------------------------------------------------------

/// Hotel gallery. The endpoint is multipart and accepts an `image_url`, so the
/// dashboard links hosted images rather than uploading binaries.
#[component]
fn PhotosTab(
    hotel: api::Organization,
    on_changed: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let photos = hotel.photos.clone();
    let url = RwSignal::new(String::new());
    let caption = RwSignal::new(String::new());
    let is_logo = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let add = move |_| {
        if busy.get() {
            return;
        }
        let u = url.get().trim().to_string();
        if !u.starts_with("http") {
            toast.error("Check the link", "Paste a full image URL starting with http.");
            return;
        }
        busy.set(true);
        let (c, logo) = (caption.get(), is_logo.get());
        wasm_bindgen_futures::spawn_local(async move {
            match api::add_gallery_photo(&u, &c, logo).await {
                Ok(_) => {
                    url.set(String::new());
                    caption.set(String::new());
                    is_logo.set(false);
                    toast.success("Photo added", "It now appears on your public listing.");
                    on_changed();
                }
                Err(e) => toast.error("Could not add the photo", e.detail()),
            }
            busy.set(false);
        });
    };

    let remove = move |photo_id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_gallery_photo(photo_id).await {
                Ok(()) => {
                    toast.success("Photo removed", "It is no longer shown.");
                    on_changed();
                }
                Err(e) => toast.error("Could not remove the photo", e.detail()),
            }
        });
    };

    view! {
        <div class="animate-fade-up">
            <Card title="Gallery" hint=format!("{} photo(s)", photos.len())>
                {if photos.is_empty() {
                    view! {
                        <EmptyState
                            icon="image"
                            title="No photos yet"
                            body="Hotels with photos convert far better. Add at least an exterior shot, the lobby and a room."
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
                            {photos.into_iter().map(|p| {
                                let pid = p.id;
                                let logo = p.is_logo;
                                view! {
                                    <div class="group relative overflow-hidden rounded-xl border border-slate-200">
                                        <img src=p.image_url.clone() alt=p.caption.clone().unwrap_or_default()
                                            class="h-32 w-full object-cover" />
                                        {logo.then(|| view! {
                                            <span class="absolute left-1.5 top-1.5 rounded bg-blue-700 px-1.5 py-0.5 text-[9px] font-bold text-white">
                                                "LOGO"
                                            </span>
                                        })}
                                        <button
                                            title="Remove"
                                            class="absolute right-1.5 top-1.5 rounded-lg bg-white/90 p-1.5 text-slate-500 opacity-0 shadow transition-opacity hover:text-red-600 group-hover:opacity-100"
                                            on:click=move |_| remove(pid)
                                        >
                                            <Icon name="trash" class="h-3.5 w-3.5" />
                                        </button>
                                        {p.caption.clone().filter(|c| !c.is_empty()).map(|c| view! {
                                            <p class="truncate px-2 py-1.5 text-2xs text-slate-500">{c}</p>
                                        })}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}

                <div class="mt-4 flex flex-col gap-2 border-t border-slate-100 pt-4">
                    <div class="flex flex-col gap-2 sm:flex-row">
                        <input
                            type="url"
                            placeholder="https://images.example.com/lobby.jpg"
                            class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            prop:value=url
                            on:input:target=move |ev| url.set(ev.target().value())
                        />
                        <input
                            type="text"
                            placeholder="Caption (optional)"
                            class="rounded-lg border border-slate-300 px-3 py-2 text-sm sm:w-48"
                            prop:value=caption
                            on:input:target=move |ev| caption.set(ev.target().value())
                        />
                        <button
                            on:click=add
                            disabled=move || busy.get()
                            class="rounded-lg bg-blue-700 px-3.5 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                        >
                            {move || if busy.get() { "Adding…" } else { "Add photo" }}
                        </button>
                    </div>
                    <label class="flex cursor-pointer items-center gap-2 text-xs text-slate-500">
                        <input type="checkbox" class="h-3.5 w-3.5" prop:checked=is_logo
                            on:change:target=move |ev| is_logo.set(ev.target().checked()) />
                        "Use this image as the property logo"
                    </label>
                </div>
            </Card>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Amenities
// ---------------------------------------------------------------------------

/// The property's amenity catalogue. Every amenity a room can offer has to exist
/// here first, so this doubles as the place to create them.
#[component]
fn AmenitiesTab() -> impl IntoView {
    let toast = use_toast();
    let refresh = RwSignal::new(0u32);
    let catalogue = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_amenities().await }
    });
    // Which of the catalogue this property actually offers. This is the set the
    // public listing shows — `/amenities/` alone is just the vocabulary.
    let active = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_org_amenities().await }
    });
    let picked = RwSignal::new(Vec::<i64>::new());
    let seeded = RwSignal::new(false);
    let saving = RwSignal::new(false);

    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        if let Some(Ok(rows)) = active.get().map(|r| r.map(|x| x.clone())) {
            picked.set(
                rows.iter()
                    .filter(|r| r.is_active)
                    .map(|r| r.amenity.id)
                    .collect(),
            );
            seeded.set(true);
        }
    });

    let save = move |_| {
        if saving.get() {
            return;
        }
        saving.set(true);
        let ids = picked.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::sync_org_amenities(&ids).await {
                Ok(()) => toast.success(
                    "Amenities updated",
                    "These now appear on your public listing.",
                ),
                Err(e) => toast.error("Could not save the amenities", e.detail()),
            }
            saving.set(false);
        });
    };

    let new_name = RwSignal::new(String::new());
    let new_category = RwSignal::new(api::AMENITY_CATEGORIES[0].to_string());
    let busy = RwSignal::new(false);

    let create = move |_| {
        if busy.get() {
            return;
        }
        let name = new_name.get().trim().to_string();
        if name.is_empty() {
            toast.error("Name required", "Give the amenity a name, e.g. Rooftop Bar.");
            return;
        }
        busy.set(true);
        let cat = new_category.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_amenity(&name, &cat).await {
                Ok(a) => {
                    new_name.set(String::new());
                    toast.success("Amenity added", format!("{} is now available to your rooms.", a.name));
                    refresh.update(|n| *n += 1);
                }
                Err(e) => toast.error("Could not add the amenity", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <div class="grid animate-fade-up gap-5 lg:grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)]">
            <Card
                title="What this property offers"
                hint=Signal::derive(move || format!(
                    "{} selected · shown on your public listing",
                    picked.get().len(),
                ))
                action=Box::new(move || view! {
                    <button
                        on:click=save
                        disabled=move || saving.get()
                        class="rounded-lg bg-blue-700 px-3 py-1.5 text-xs font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                    >
                        {move || if saving.get() { "Saving…" } else { "Save selection" }}
                    </button>
                }.into_any())
            >
                <Suspense fallback=|| view! {
                    <p class="py-8 text-center text-sm text-slate-400">"Loading catalogue…"</p>
                }>
                    {move || Suspend::new(async move {
                        let all = match catalogue.await {
                            Ok(a) => a,
                            Err(e) => return view! {
                                <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                            }.into_any(),
                        };
                        if all.is_empty() {
                            return view! {
                                <EmptyState
                                    icon="sparkles"
                                    title="Nothing in the catalogue"
                                    body="Add the facilities your property offers — Wi-Fi, air conditioning, parking — then select the ones this hotel provides."
                                />
                            }.into_any();
                        }
                        // Group by the catalogue's own category values.
                        let mut groups: Vec<(String, Vec<api::Amenity>)> = Vec::new();
                        for a in all {
                            let cat = a.category_str().to_string();
                            match groups.iter_mut().find(|(c, _)| *c == cat) {
                                Some(g) => g.1.push(a),
                                None => groups.push((cat, vec![a])),
                            }
                        }
                        view! {
                            <div class="flex flex-col gap-5">
                                {groups.into_iter().map(|(cat, items)| view! {
                                    <div>
                                        <p class="mb-2 text-2xs font-bold uppercase tracking-wide text-slate-500">
                                            {format!("{cat} · {}", items.len())}
                                        </p>
                                        <div class="flex flex-wrap gap-1.5">
                                            {items.into_iter().map(|a| {
                                                let id = a.id;
                                                view! {
                                                    <button
                                                        type="button"
                                                        on:click=move |_| picked.update(|p| match p.iter().position(|x| *x == id) {
                                                            Some(i) => { p.remove(i); }
                                                            None => p.push(id),
                                                        })
                                                        class=move || format!(
                                                            "flex items-center gap-1.5 rounded-lg border px-2.5 py-1 text-xs font-medium transition-colors {}",
                                                            if picked.get().contains(&id) {
                                                                "border-blue-600 bg-blue-50 text-blue-700"
                                                            } else {
                                                                "border-slate-200 bg-slate-50 text-slate-600 hover:bg-white"
                                                            }
                                                        )
                                                    >
                                                        <Show when=move || picked.get().contains(&id)>
                                                            <Icon name="check" class="h-3 w-3" />
                                                        </Show>
                                                        {a.name.clone()}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }).collect_view()}
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </Card>

            <Card title="Add an amenity">
                <div class="flex flex-col gap-4">
                    <Field label="Name" value=new_name placeholder="e.g. Rooftop Bar" />
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Category"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                            on:change:target=move |ev| new_category.set(ev.target().value())
                        >
                            {api::AMENITY_CATEGORIES.iter().map(|c| view! {
                                <option value=*c>{*c}</option>
                            }).collect_view()}
                        </select>
                    </div>
                    <button
                        on:click=create
                        disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                    >
                        {move || if busy.get() { "Adding…" } else { "Add amenity" }}
                    </button>
                    <p class="text-2xs text-slate-500">
                        "Creating an amenity only adds it to the vocabulary. Tick it on the left and save to advertise it on your listing, and assign it to individual rooms from the room detail page."
                    </p>
                </div>
            </Card>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Policies
// ---------------------------------------------------------------------------

#[component]
fn PoliciesTab() -> impl IntoView {
    let toast = use_toast();
    let loaded = LocalResource::new(|| async move { api::get_policies().await });
    let policy = RwSignal::new(api::OrgPolicies::default());
    let seeded = RwSignal::new(false);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        if seeded.get() {
            return;
        }
        if let Some(Ok(p)) = loaded.get().map(|r| r.map(|x| x.clone())) {
            policy.set(p);
            seeded.set(true);
        }
    });

    let save = move |_| {
        if busy.get() {
            return;
        }
        busy.set(true);
        let body = policy.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::update_policies(&body).await {
                Ok(_) => toast.success("Policies saved", "These rules now show on your listing."),
                Err(e) => toast.error("Could not save the policies", e.detail()),
            }
            busy.set(false);
        });
    };

    view! {
        <div class="animate-fade-up">
            <Suspense fallback=|| view! {
                <p class="py-16 text-center text-sm text-slate-400">"Loading policies…"</p>
            }>
                {move || Suspend::new(async move {
                    if let Err(e) = loaded.await {
                        return view! {
                            <p class="py-16 text-center text-sm text-red-600">{e.detail()}</p>
                        }.into_any();
                    }
                    view! {
                        <div class="grid gap-5 lg:grid-cols-2">
                            <Card title="Check-in & check-out">
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-in from"</label>
                                        <input type="time" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=move || trim_seconds(policy.get().checkin_time)
                                            on:input:target=move |ev| {
                                                let v = ev.target().value();
                                                policy.update(|p| p.checkin_time = Some(v));
                                            } />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-out by"</label>
                                        <input type="time" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=move || trim_seconds(policy.get().checkout_time)
                                            on:input:target=move |ev| {
                                                let v = ev.target().value();
                                                policy.update(|p| p.checkout_time = Some(v));
                                            } />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Minimum check-in age"</label>
                                        <input type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=move || policy.get().minimum_checkin_age.map(|n| n.to_string()).unwrap_or_default()
                                            on:input:target=move |ev| {
                                                let v = ev.target().value().parse().ok();
                                                policy.update(|p| p.minimum_checkin_age = v);
                                            } />
                                    </div>
                                    <div>
                                        <label class="mb-1 block text-xs font-medium text-slate-500">"Children age limit"</label>
                                        <input type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                            prop:value=move || policy.get().children_age.map(|n| n.to_string()).unwrap_or_default()
                                            on:input:target=move |ev| {
                                                let v = ev.target().value().parse().ok();
                                                policy.update(|p| p.children_age = v);
                                            } />
                                    </div>
                                </div>
                            </Card>

                            <Card title="House rules">
                                <div class="flex flex-col gap-2">
                                    <Toggle label="Children allowed"
                                        get=Signal::derive(move || policy.get().children_allowed.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.children_allowed = Some(v)) />
                                    <Toggle label="Extra beds available"
                                        get=Signal::derive(move || policy.get().extrabed_available.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.extrabed_available = Some(v)) />
                                    <Toggle label="Pets allowed"
                                        get=Signal::derive(move || policy.get().pet_allowed.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.pet_allowed = Some(v)) />
                                    <Toggle label="Smoking allowed"
                                        get=Signal::derive(move || policy.get().smoking_allowed.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.smoking_allowed = Some(v)) />
                                    <Toggle label="Entirely non-smoking property"
                                        get=Signal::derive(move || policy.get().non_smoking_property.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.non_smoking_property = Some(v)) />
                                    <Toggle label="Parties or events allowed"
                                        get=Signal::derive(move || policy.get().parties_or_event_allowed.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.parties_or_event_allowed = Some(v)) />
                                    <Toggle label="Government ID required at check-in"
                                        get=Signal::derive(move || policy.get().government_id_required.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.government_id_required = Some(v)) />
                                </div>
                            </Card>

                            <Card title="Accepted payment">
                                <div class="flex flex-col gap-2">
                                    <Toggle label="Bank cards"
                                        get=Signal::derive(move || policy.get().bank_card_allow.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.bank_card_allow = Some(v)) />
                                    <Toggle label="Online transactions"
                                        get=Signal::derive(move || policy.get().online_transaction_allow.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.online_transaction_allow = Some(v)) />
                                    <Toggle label="Bank transfer"
                                        get=Signal::derive(move || policy.get().bank_payment_allow.unwrap_or(false))
                                        set=move |v| policy.update(|p| p.bank_payment_allow = Some(v)) />
                                </div>
                                <p class="mt-3 text-2xs text-slate-500">
                                    "Guests pay the property directly — the platform never holds their money."
                                </p>
                            </Card>

                            <Card title="Public note" hint="Shown on your listing">
                                <textarea rows="6" placeholder="Anything a guest should know before booking"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                    prop:value=move || policy.get().public_note.unwrap_or_default()
                                    on:input:target=move |ev| {
                                        let v = ev.target().value();
                                        policy.update(|p| p.public_note = Some(v));
                                    }
                                ></textarea>
                            </Card>
                        </div>

                        <div class="mt-5 flex justify-end">
                            <button on:click=save disabled=move || busy.get()
                                class="rounded-lg bg-blue-700 px-5 py-2.5 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60">
                                {move || if busy.get() { "Saving…" } else { "Save policies" }}
                            </button>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Contacts
// ---------------------------------------------------------------------------

#[component]
fn ContactsTab(hotel: api::Organization) -> impl IntoView {
    let toast = use_toast();
    let refresh = RwSignal::new(0u32);
    let contacts = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_contacts().await }
    });
    // The organization payload already carries contacts; the dedicated endpoint
    // is used so the list stays live after an add or delete.
    let _ = hotel;

    let name = RwSignal::new(String::new());
    let title = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let primary = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let add = move |_| {
        if busy.get() {
            return;
        }
        if name.get().trim().is_empty() {
            toast.error("Name required", "Enter the contact person's name.");
            return;
        }
        busy.set(true);
        let payload = api::OrgContact {
            id: 0,
            name: name.get().trim().to_string(),
            title: opt(title.get()),
            email: opt(email.get()),
            phone_number: opt(phone.get()),
            is_primary: primary.get(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_contact(&payload).await {
                Ok(_) => {
                    name.set(String::new());
                    title.set(String::new());
                    email.set(String::new());
                    phone.set(String::new());
                    primary.set(false);
                    toast.success("Contact added", "They are now listed for this property.");
                    refresh.update(|n| *n += 1);
                }
                Err(e) => toast.error("Could not add the contact", e.detail()),
            }
            busy.set(false);
        });
    };

    let remove = move |id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            match api::delete_contact(id).await {
                Ok(()) => {
                    toast.success("Contact removed", "They are no longer listed.");
                    refresh.update(|n| *n += 1);
                }
                Err(e) => toast.error("Could not remove the contact", e.detail()),
            }
        });
    };

    view! {
        <div class="grid animate-fade-up gap-5 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
            <Card title="Contact people" hint="Who the platform reaches about this property">
                <Suspense fallback=|| view! {
                    <p class="py-8 text-center text-sm text-slate-400">"Loading contacts…"</p>
                }>
                    {move || Suspend::new(async move {
                        let rows = match contacts.await {
                            Ok(c) => c,
                            Err(e) => return view! {
                                <p class="py-8 text-center text-sm text-red-600">{e.detail()}</p>
                            }.into_any(),
                        };
                        if rows.is_empty() {
                            return view! {
                                <EmptyState
                                    icon="users"
                                    title="No contacts yet"
                                    body="Add the general manager or front-office lead so the platform knows who to reach."
                                />
                            }.into_any();
                        }
                        view! {
                            <div class="flex flex-col gap-2.5">
                                {rows.into_iter().map(|c| {
                                    let cid = c.id;
                                    view! {
                                        <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 p-3">
                                            <div class="min-w-0">
                                                <p class="flex flex-wrap items-center gap-2 font-semibold text-slate-900">
                                                    {c.name.clone()}
                                                    {c.is_primary.then(|| view! { <Badge label="Primary" tone="blue" /> })}
                                                </p>
                                                <p class="mt-0.5 text-xs text-slate-500">
                                                    {c.title.clone().unwrap_or_default()}
                                                </p>
                                                <p class="mt-0.5 text-xs text-slate-400">
                                                    {[c.email.clone(), c.phone_number.clone()]
                                                        .into_iter()
                                                        .flatten()
                                                        .filter(|s| !s.is_empty())
                                                        .collect::<Vec<_>>()
                                                        .join(" · ")}
                                                </p>
                                            </div>
                                            <button
                                                title="Remove"
                                                class="rounded-lg p-2 text-slate-400 hover:text-red-600"
                                                on:click=move |_| remove(cid)
                                            >
                                                <Icon name="trash" class="h-4 w-4" />
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </Card>

            <Card title="Add a contact">
                <div class="flex flex-col gap-4">
                    <Field label="Name" value=name placeholder="e.g. Yonas Desta" />
                    <Field label="Title" value=title placeholder="General Manager" />
                    <Field label="Email" value=email kind="email" />
                    <Field label="Phone" value=phone kind="tel" />
                    <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-600">
                        <input type="checkbox" class="h-4 w-4" prop:checked=primary
                            on:change:target=move |ev| primary.set(ev.target().checked()) />
                        "Primary contact"
                    </label>
                    <button
                        on:click=add
                        disabled=move || busy.get()
                        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-800 disabled:opacity-60"
                    >
                        {move || if busy.get() { "Adding…" } else { "Add contact" }}
                    </button>
                </div>
            </Card>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

/// Blank inputs are dropped rather than sent as `""`.
fn opt(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// The API returns `"14:00:00"`; `<input type="time">` wants `"14:00"`.
fn trim_seconds(raw: Option<String>) -> String {
    raw.map(|s| s.chars().take(5).collect()).unwrap_or_default()
}

#[component]
fn Field(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = "")] placeholder: &'static str,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-xs font-medium text-slate-500">{label}</label>
            <input type=kind placeholder=placeholder class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
                prop:value=value on:input:target=move |ev| value.set(ev.target().value()) />
        </div>
    }
}

#[component]
fn Toggle(
    label: &'static str,
    get: Signal<bool>,
    set: impl Fn(bool) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <label class="flex cursor-pointer items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2.5">
            <span class="text-sm text-slate-700">{label}</span>
            <input type="checkbox" class="h-4 w-4 shrink-0" prop:checked=move || get.get()
                on:change:target=move |ev| set(ev.target().checked()) />
        </label>
    }
}
