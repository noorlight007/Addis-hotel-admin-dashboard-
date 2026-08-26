use crate::components::{Icon, Modal, Toggle};
use leptos::prelude::*;
use leptos_router::components::A;

const TABS: &[&str] = &["Profile", "Photos", "Amenities", "Policies", "Bank Details"];

const STOCK_PHOTOS: &[&str] = &[
    "https://images.unsplash.com/photo-1566073771259-6a8506099945?q=80&w=400",
    "https://images.unsplash.com/photo-1590490360182-c33d57733427?q=80&w=400",
    "https://images.unsplash.com/photo-1611892440504-42a792e24d32?q=80&w=400",
    "https://images.unsplash.com/photo-1582719478250-c89cae4dc85b?q=80&w=400",
    "https://images.unsplash.com/photo-1595576508898-0ad5c879a061?q=80&w=400",
    "https://images.unsplash.com/photo-1560185127-6ed189bf02f4?q=80&w=400",
];

#[derive(Debug, Clone, Copy, PartialEq)]
struct HotelInfo {
    name: &'static str,
    phone: &'static str,
    address: &'static str,
    whatsapp: &'static str,
    email: &'static str,
    description: &'static str,
    currency: &'static str,
    language: &'static str,
    check_in_time: &'static str,
    check_out_time: &'static str,
}

const DEFAULT_HOTEL: HotelInfo = HotelInfo {
    name: "Golden Tulip Addis Ababa",
    phone: "+251 11 123 4567",
    address: "Bole, Addis Ababa, Ethiopia",
    whatsapp: "+251 11 123 4567",
    email: "info@goldentulipaddis.com",
    description: "Golden Tulip Addis Ababa offers modern rooms, excellent service and a comfortable stay in the heart of Addis Ababa.",
    currency: "ETB",
    language: "English",
    check_in_time: "2:00 PM",
    check_out_time: "12:00 PM",
};

fn currency_label(code: &str) -> &'static str {
    match code {
        "ETB" => "ETB (Ethiopian Birr)",
        "USD" => "USD (US Dollar)",
        "EUR" => "EUR (Euro)",
        _ => "ETB (Ethiopian Birr)",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BankInfo {
    bank_name: &'static str,
    account_name: &'static str,
    account_number: &'static str,
    swift_code: &'static str,
}

const DEFAULT_BANK: BankInfo = BankInfo {
    bank_name: "Commercial Bank of Ethiopia",
    account_name: "Golden Tulip Addis Ababa PLC",
    account_number: "1000234567890",
    swift_code: "CBETETAA",
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct PolicyInfo {
    children_allowed: bool,
    extra_bed: bool,
    child_free_age: &'static str,
    pets_allowed: bool,
    smoking_allowed: bool,
    non_smoking: bool,
    govt_id_required: bool,
    min_age: &'static str,
    parties_allowed: bool,
    notes: &'static str,
}

const DEFAULT_POLICIES: PolicyInfo = PolicyInfo {
    children_allowed: true,
    extra_bed: false,
    child_free_age: "6",
    pets_allowed: false,
    smoking_allowed: false,
    non_smoking: true,
    govt_id_required: true,
    min_age: "18",
    parties_allowed: false,
    notes: "Guests must present a valid ID at check-in. Late check-out is available upon request. The property is fully non-smoking.",
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContactPerson {
    name: &'static str,
    title: &'static str,
    phone: &'static str,
    email: &'static str,
}

const DEFAULT_CONTACT: ContactPerson = ContactPerson {
    name: "Ahmed Hassan",
    title: "General Manager",
    phone: "+251 91 234 5678",
    email: "ahmed.hassan@email.com",
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct BusinessInfo {
    registration_number: &'static str,
    tin: &'static str,
    vat: &'static str,
    established_year: &'static str,
}

const DEFAULT_BUSINESS: BusinessInfo = BusinessInfo {
    registration_number: "BIRR-123456-7890",
    tin: "100-234-567",
    vat: "ET-100-23-456789",
    established_year: "2018",
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct SocialLinks {
    facebook: &'static str,
    instagram: &'static str,
    twitter: &'static str,
}

const DEFAULT_SOCIAL: SocialLinks = SocialLinks {
    facebook: "facebook.com/goldentulipaddis",
    instagram: "instagram.com/goldentulipaddis",
    twitter: "twitter.com/goldentulipaddis",
};

const AMENITY_CATEGORIES: &[(&str, &str, &[(&str, &str, bool)])] = &[
    ("Essentials", "sparkles", &[
        ("wifi", "Free Wi-Fi", true), ("package", "Parking", true),
        ("snowflake", "Air Conditioning", true), ("clock", "24/7 Front Desk", true),
    ]),
    ("Leisure & Wellness", "users", &[
        ("umbrella", "Swimming Pool", true), ("dumbbell", "Gym", true),
        ("sparkles", "Spa", false), ("paw", "Pet Friendly", false),
    ]),
    ("Dining", "coffee", &[
        ("utensils", "Restaurant", true), ("wine", "Bar", true),
        ("check-circle", "Breakfast Included", true), ("bell-concierge", "Room Service", true),
    ]),
    ("Services", "package", &[
        ("shuttle", "Airport Shuttle", true), ("package", "Laundry", true),
        ("housekeeping", "Housekeeping", true),
    ]),
    ("Rooms & Accessibility", "family", &[
        ("family", "Family Rooms", true), ("wheelchair", "Wheelchair Access", true),
        ("x-circle", "Non-smoking Rooms", false),
    ]),
    ("Business", "briefcase", &[
        ("briefcase", "Meeting Room", true), ("monitor", "Business Center", false),
        ("printer", "Printer / Fax Service", false),
    ]),
];

fn default_amenities() -> Vec<&'static str> {
    AMENITY_CATEGORIES.iter().flat_map(|(_, _, items)| items.iter()).filter(|(_, _, on)| *on).map(|(_, label, _)| *label).collect()
}

fn amenity_icon(label: &str) -> &'static str {
    AMENITY_CATEGORIES.iter().flat_map(|(_, _, items)| items.iter()).find(|(_, l, _)| *l == label).map(|(icon, _, _)| *icon).unwrap_or("check-circle")
}

#[component]
pub fn HotelProfilePage() -> impl IntoView {
    let active_tab = RwSignal::new("Profile");
    let amenities_modal_open = RwSignal::new(false);
    let policies_modal_open = RwSignal::new(false);
    let profile_modal_open = RwSignal::new(false);
    let bank_modal_open = RwSignal::new(false);
    let contact_modal_open = RwSignal::new(false);
    let business_modal_open = RwSignal::new(false);
    let social_modal_open = RwSignal::new(false);
    let deactivate_modal_open = RwSignal::new(false);
    let hotel = RwSignal::new(DEFAULT_HOTEL);
    let bank = RwSignal::new(DEFAULT_BANK);
    let policies = RwSignal::new(DEFAULT_POLICIES);
    let contact = RwSignal::new(DEFAULT_CONTACT);
    let business = RwSignal::new(DEFAULT_BUSINESS);
    let social = RwSignal::new(DEFAULT_SOCIAL);
    let amenities = RwSignal::new(default_amenities());
    let photos = RwSignal::new(STOCK_PHOTOS[..4].to_vec());
    let deactivated = RwSignal::new(false);

    view! {
        <div class="p-4 sm:p-6">
            <Show when=move || deactivated.get()>
                <div class="mb-4 flex items-center gap-2 rounded-lg bg-amber-50 px-3 py-2 text-sm text-amber-700">
                    <Icon name="info" class="h-4 w-4" />
                    "This hotel profile is deactivated and hidden from the public listings."
                    <button on:click=move |_| deactivated.set(false) class="ml-auto font-semibold hover:underline">"Reactivate"</button>
                </div>
            </Show>

            <A href="/" attr:class="mb-3 inline-flex items-center gap-1 text-sm text-slate-500 hover:underline">
                <Icon name="chevron-left" class="h-4 w-4" />
                "Back to Dashboard"
            </A>

            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <h1 class="text-xl font-bold text-slate-900">"Hotel Profile"</h1>
                <a
                    href="http://localhost:3000/hotels/golden-tulip-addis-ababa"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm font-medium transition-colors hover:bg-slate-50"
                >
                    "Preview Profile"
                    <Icon name="external-link" class="h-4 w-4" />
                </a>
            </div>

            <div class="mb-6 flex gap-6 border-b border-slate-200 text-sm font-medium text-slate-500">
                {TABS.iter().map(|t| {
                    let t = *t;
                    view! {
                        <button
                            class=move || format!(
                                "-mb-px border-b-2 pb-2 {}",
                                if active_tab.get() == t { "border-blue-700 text-blue-700" } else { "border-transparent" }
                            )
                            on:click=move |_| active_tab.set(t)
                        >
                            {t}
                        </button>
                    }
                }).collect_view()}
            </div>

            {move || match active_tab.get() {
                "Photos" => view! {
                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                        <div class="mb-3 flex items-center justify-between">
                            <h2 class="text-base font-semibold text-slate-900">"Hotel Images"</h2>
                            <span class="text-xs text-slate-400">"Hover a photo to remove it."</span>
                        </div>
                        <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
                            {move || photos.get().into_iter().enumerate().map(|(i, p)| view! {
                                <div class="group relative overflow-hidden rounded-lg">
                                    <img src=p alt="" class="h-32 w-full object-cover" />
                                    <button
                                        on:click=move |_| photos.update(|list| { list.remove(i); })
                                        class="absolute right-1.5 top-1.5 flex h-6 w-6 items-center justify-center rounded-full bg-black/60 text-white opacity-0 transition-opacity group-hover:opacity-100"
                                    >
                                        <Icon name="x" class="h-3.5 w-3.5" />
                                    </button>
                                </div>
                            }).collect_view()}
                            <button
                                on:click=move |_| photos.update(|list| {
                                    let next = STOCK_PHOTOS[list.len() % STOCK_PHOTOS.len()];
                                    list.push(next);
                                })
                                class="flex h-32 w-full flex-col items-center justify-center gap-1 rounded-lg border-2 border-dashed border-slate-300 text-sm text-slate-400 transition-colors hover:border-blue-300 hover:text-blue-600"
                            >
                                <Icon name="plus" class="h-5 w-5" />
                                "Add More"
                            </button>
                        </div>
                    </div>
                }.into_any(),

                "Amenities" => view! {
                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                        <div class="mb-3 flex items-center justify-between">
                            <h2 class="text-base font-semibold text-slate-900">"Amenities"</h2>
                            <button
                                class="flex items-center gap-1.5 rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-blue-700"
                                on:click=move |_| amenities_modal_open.set(true)
                            >
                                <Icon name="edit" class="h-4 w-4" />
                                "Edit Amenities"
                            </button>
                        </div>
                        <p class="mb-3 text-sm text-slate-500">"Manage the amenities and services available at your hotel."</p>
                        <div class="flex flex-wrap gap-2">
                            {move || amenities.get().into_iter().map(|label| view! {
                                <span class="flex items-center gap-1.5 rounded-full bg-blue-50 px-3 py-1 text-sm text-blue-700">
                                    <Icon name=amenity_icon(label) class="h-3.5 w-3.5" />
                                    {label}
                                </span>
                            }).collect_view()}
                        </div>
                    </div>

                    <Show when=move || amenities_modal_open.get()>
                        <AmenitiesModal amenities=amenities on_close=move || amenities_modal_open.set(false) />
                    </Show>
                }.into_any(),

                "Policies" => view! {
                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                        <div class="mb-3 flex items-center justify-between">
                            <h2 class="text-base font-semibold text-slate-900">"Policies"</h2>
                            <button
                                class="flex items-center gap-1.5 rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-blue-700"
                                on:click=move |_| policies_modal_open.set(true)
                            >
                                <Icon name="edit" class="h-4 w-4" />
                                "Edit Policies"
                            </button>
                        </div>
                        <p class="mb-4 text-sm text-slate-500">"Define the rules and guidelines for guests staying at your hotel."</p>
                        <dl class="grid grid-cols-2 gap-4 text-sm">
                            <div><dt class="text-slate-400">"Check-in Time"</dt><dd class="font-medium">{move || hotel.get().check_in_time}</dd></div>
                            <div><dt class="text-slate-400">"Check-out Time"</dt><dd class="font-medium">{move || hotel.get().check_out_time}</dd></div>
                            <div><dt class="text-slate-400">"Children Allowed"</dt><dd class="font-medium">{move || if policies.get().children_allowed { "Yes" } else { "No" }}</dd></div>
                            <div><dt class="text-slate-400">"Pets Allowed"</dt><dd class="font-medium">{move || if policies.get().pets_allowed { "Yes" } else { "No" }}</dd></div>
                            <div><dt class="text-slate-400">"Government ID Required"</dt><dd class="font-medium">{move || if policies.get().govt_id_required { "Yes" } else { "No" }}</dd></div>
                            <div><dt class="text-slate-400">"Minimum Check-in Age"</dt><dd class="font-medium">{move || policies.get().min_age}</dd></div>
                        </dl>
                        <p class="mt-4 rounded-lg bg-slate-50 p-3 text-sm text-slate-600">{move || policies.get().notes}</p>
                    </div>

                    <Show when=move || policies_modal_open.get()>
                        <PoliciesModal hotel=hotel policies=policies on_close=move || policies_modal_open.set(false) />
                    </Show>
                }.into_any(),

                "Bank Details" => view! {
                    <div class="rounded-xl border border-slate-200 bg-white p-5">
                        <div class="mb-3 flex items-center justify-between">
                            <h2 class="text-base font-semibold text-slate-900">"Bank Details"</h2>
                            <button on:click=move |_| bank_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                        </div>
                        <dl class="grid grid-cols-2 gap-4 text-sm">
                            <div><dt class="text-slate-400">"Bank Name"</dt><dd class="font-medium">{move || bank.get().bank_name}</dd></div>
                            <div><dt class="text-slate-400">"Account Name"</dt><dd class="font-medium">{move || bank.get().account_name}</dd></div>
                            <div><dt class="text-slate-400">"Account Number"</dt><dd class="font-medium">{move || bank.get().account_number}</dd></div>
                            <div><dt class="text-slate-400">"SWIFT Code"</dt><dd class="font-medium">{move || bank.get().swift_code}</dd></div>
                        </dl>
                    </div>

                    <Show when=move || bank_modal_open.get()>
                        <EditBankModal bank=bank on_close=move || bank_modal_open.set(false) />
                    </Show>
                }.into_any(),

                _ => view! {
                    <div class="flex flex-col gap-4">
                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Profile Information"</h2>
                                <button on:click=move |_| profile_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                            </div>
                            <p class="mb-4 text-sm text-slate-500">"Update your hotel information and contact details."</p>
                            <dl class="flex flex-col divide-y divide-slate-100 text-sm">
                                <div class="flex items-center justify-between py-2"><dt class="text-slate-400">"Hotel Name"</dt><dd class="font-medium">{move || hotel.get().name}</dd></div>
                                <div class="flex items-center justify-between py-2"><dt class="text-slate-400">"Address"</dt><dd class="font-medium">{move || hotel.get().address}</dd></div>
                                <div class="flex items-center justify-between py-2"><dt class="text-slate-400">"Phone"</dt><dd class="font-medium">{move || hotel.get().phone}</dd></div>
                                <div class="flex items-center justify-between py-2"><dt class="text-slate-400">"WhatsApp"</dt><dd class="font-medium">{move || hotel.get().whatsapp}</dd></div>
                                <div class="flex items-center justify-between py-2"><dt class="text-slate-400">"Email"</dt><dd class="font-medium">{move || hotel.get().email}</dd></div>
                                <div class="py-2"><dt class="mb-1 text-slate-400">"Description"</dt><dd class="text-slate-600">{move || hotel.get().description}</dd></div>
                            </dl>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Hotel Images"</h2>
                                <button on:click=move |_| active_tab.set("Photos") class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                            </div>
                            <div class="grid grid-cols-3 gap-3 sm:grid-cols-6">
                                {move || photos.get().into_iter().map(|p| view! {
                                    <img src=p alt="" class="h-20 w-full rounded-lg object-cover" />
                                }).collect_view()}
                                <button on:click=move |_| active_tab.set("Photos") class="flex h-20 w-full flex-col items-center justify-center gap-0.5 rounded-lg border-2 border-dashed border-slate-300 text-xs text-slate-400 transition-colors hover:border-blue-300 hover:text-blue-600">
                                    <Icon name="plus" class="h-4 w-4" />
                                    "Add More"
                                </button>
                            </div>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Hotel Settings"</h2>
                                <button on:click=move |_| profile_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit"</button>
                            </div>
                            <dl class="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
                                <div><dt class="text-slate-400">"Check-in Time"</dt><dd class="font-medium">{move || hotel.get().check_in_time}</dd></div>
                                <div><dt class="text-slate-400">"Check-out Time"</dt><dd class="font-medium">{move || hotel.get().check_out_time}</dd></div>
                                <div><dt class="text-slate-400">"Currency"</dt><dd class="font-medium">{move || currency_label(hotel.get().currency)}</dd></div>
                                <div><dt class="text-slate-400">"Language"</dt><dd class="font-medium">{move || hotel.get().language}</dd></div>
                            </dl>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Contact Person"</h2>
                                <button on:click=move |_| contact_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit Contact"</button>
                            </div>
                            <p class="mb-4 text-sm text-slate-500">"Primary contact for your hotel."</p>
                            <div class="flex items-center gap-4">
                                <span class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-purple-100 text-lg font-bold text-purple-700">
                                    {move || contact.get().name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()}
                                </span>
                                <div class="text-sm">
                                    <p class="flex items-center gap-2 font-semibold text-slate-900">
                                        {move || contact.get().name}
                                        <span class="rounded-full bg-blue-50 px-2 py-0.5 text-xs font-semibold text-blue-700">"Primary Contact"</span>
                                    </p>
                                    <p class="text-slate-500">{move || contact.get().title}</p>
                                    <p class="text-slate-500">{move || contact.get().phone}</p>
                                    <p class="text-slate-500">{move || contact.get().email}</p>
                                </div>
                            </div>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Business Information"</h2>
                                <button on:click=move |_| business_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit Information"</button>
                            </div>
                            <p class="mb-4 text-sm text-slate-500">"Additional information about your hotel."</p>
                            <dl class="grid grid-cols-2 gap-4 text-sm">
                                <div><dt class="text-slate-400">"Business Registration Number"</dt><dd class="font-medium">{move || business.get().registration_number}</dd></div>
                                <div><dt class="text-slate-400">"Established Year"</dt><dd class="font-medium">{move || business.get().established_year}</dd></div>
                                <div><dt class="text-slate-400">"Tax Identification Number (TIN)"</dt><dd class="font-medium">{move || business.get().tin}</dd></div>
                                <div><dt class="text-slate-400">"VAT Number"</dt><dd class="font-medium">{move || business.get().vat}</dd></div>
                            </dl>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-5">
                            <div class="mb-3 flex items-center justify-between">
                                <h2 class="text-base font-semibold text-slate-900">"Social Media Links"</h2>
                                <button on:click=move |_| social_modal_open.set(true) class="flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:underline"><Icon name="edit" class="h-4 w-4" />"Edit Social Links"</button>
                            </div>
                            <p class="mb-4 text-sm text-slate-500">"Your hotel's social media presence."</p>
                            <div class="flex flex-wrap gap-6 text-sm">
                                <span class="flex items-center gap-2"><Icon name="facebook" class="h-4 w-4 text-blue-600" />{move || social.get().facebook}</span>
                                <span class="flex items-center gap-2"><Icon name="instagram" class="h-4 w-4 text-pink-600" />{move || social.get().instagram}</span>
                                <span class="flex items-center gap-2"><Icon name="twitter" class="h-4 w-4 text-slate-800" />{move || social.get().twitter}</span>
                            </div>
                        </div>
                    </div>

                    <Show when=move || profile_modal_open.get()>
                        <EditProfileModal hotel=hotel on_close=move || profile_modal_open.set(false) />
                    </Show>
                    <Show when=move || contact_modal_open.get()>
                        <EditContactModal contact=contact on_close=move || contact_modal_open.set(false) />
                    </Show>
                    <Show when=move || business_modal_open.get()>
                        <EditBusinessModal business=business on_close=move || business_modal_open.set(false) />
                    </Show>
                    <Show when=move || social_modal_open.get()>
                        <EditSocialModal social=social on_close=move || social_modal_open.set(false) />
                    </Show>
                }.into_any(),
            }}

            <button
                on:click=move |_| deactivate_modal_open.set(true)
                class="mt-6 w-full rounded-xl border border-red-200 py-3 text-sm font-semibold text-red-600 transition-colors hover:bg-red-50"
            >
                "Deactivate Hotel Profile"
            </button>

            <Show when=move || deactivate_modal_open.get()>
                <Modal title="Deactivate Hotel Profile" on_close=move || deactivate_modal_open.set(false)>
                    <p class="text-sm text-slate-600">
                        "Your hotel will be hidden from public search results and guests won't be able to make new reservations. You can reactivate at any time from this page."
                    </p>
                    <div class="mt-6 flex justify-end gap-3">
                        <button class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| deactivate_modal_open.set(false)>"Cancel"</button>
                        <button
                            on:click=move |_| { deactivated.set(true); deactivate_modal_open.set(false); }
                            class="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-red-700 active:scale-[0.98]"
                        >
                            "Deactivate"
                        </button>
                    </div>
                </Modal>
            </Show>
        </div>
    }
}

#[component]
fn EditProfileModal(hotel: RwSignal<HotelInfo>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let h0 = hotel.get_untracked();
    let name = RwSignal::new(h0.name.to_string());
    let phone = RwSignal::new(h0.phone.to_string());
    let address = RwSignal::new(h0.address.to_string());
    let whatsapp = RwSignal::new(h0.whatsapp.to_string());
    let email = RwSignal::new(h0.email.to_string());
    let description = RwSignal::new(h0.description.to_string());
    let currency = RwSignal::new(h0.currency.to_string());
    let language = RwSignal::new(h0.language.to_string());
    let check_in_time = RwSignal::new(h0.check_in_time.to_string());
    let check_out_time = RwSignal::new(h0.check_out_time.to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        hotel.update(|h| {
            h.name = Box::leak(name.get().into_boxed_str());
            h.phone = Box::leak(phone.get().into_boxed_str());
            h.address = Box::leak(address.get().into_boxed_str());
            h.whatsapp = Box::leak(whatsapp.get().into_boxed_str());
            h.email = Box::leak(email.get().into_boxed_str());
            h.description = Box::leak(description.get().into_boxed_str());
            h.currency = Box::leak(currency.get().into_boxed_str());
            h.language = Box::leak(language.get().into_boxed_str());
            h.check_in_time = Box::leak(check_in_time.get().into_boxed_str());
            h.check_out_time = Box::leak(check_out_time.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <Modal title="Edit Profile Information" on_close=on_close width="max-w-2xl">
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Hotel Name"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Address"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=address on:input:target=move |ev| address.set(ev.target().value()) />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"WhatsApp"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=whatsapp on:input:target=move |ev| whatsapp.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Email"</label>
                        <input type="email" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=email on:input:target=move |ev| email.set(ev.target().value()) />
                    </div>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Description"</label>
                    <textarea rows="3" class="w-full rounded-lg border border-slate-300 p-2 text-sm" prop:value=description on:input:target=move |ev| description.set(ev.target().value())></textarea>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-in Time"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=check_in_time on:input:target=move |ev| check_in_time.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Check-out Time"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=check_out_time on:input:target=move |ev| check_out_time.set(ev.target().value()) />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Currency"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| currency.set(ev.target().value())>
                            {["ETB", "USD", "EUR"].iter().map(|c| view! { <option value=*c selected=*c == currency.get_untracked()>{*c}</option> }).collect_view()}
                        </select>
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Language"</label>
                        <select class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" on:change:target=move |ev| language.set(ev.target().value())>
                            {["English", "Amharic", "French", "Arabic"].iter().map(|l| view! { <option value=*l selected=*l == language.get_untracked()>{*l}</option> }).collect_view()}
                        </select>
                    </div>
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn EditBankModal(bank: RwSignal<BankInfo>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let b0 = bank.get_untracked();
    let bank_name = RwSignal::new(b0.bank_name.to_string());
    let account_name = RwSignal::new(b0.account_name.to_string());
    let account_number = RwSignal::new(b0.account_number.to_string());
    let swift_code = RwSignal::new(b0.swift_code.to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        bank.update(|b| {
            b.bank_name = Box::leak(bank_name.get().into_boxed_str());
            b.account_name = Box::leak(account_name.get().into_boxed_str());
            b.account_number = Box::leak(account_number.get().into_boxed_str());
            b.swift_code = Box::leak(swift_code.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <Modal title="Edit Bank Details" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Bank Name"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=bank_name on:input:target=move |ev| bank_name.set(ev.target().value()) />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Account Name"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=account_name on:input:target=move |ev| account_name.set(ev.target().value()) />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Account Number"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=account_number on:input:target=move |ev| account_number.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"SWIFT Code"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=swift_code on:input:target=move |ev| swift_code.set(ev.target().value()) />
                    </div>
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn EditContactModal(contact: RwSignal<ContactPerson>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let c0 = contact.get_untracked();
    let name = RwSignal::new(c0.name.to_string());
    let title = RwSignal::new(c0.title.to_string());
    let phone = RwSignal::new(c0.phone.to_string());
    let email = RwSignal::new(c0.email.to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        contact.update(|c| {
            c.name = Box::leak(name.get().into_boxed_str());
            c.title = Box::leak(title.get().into_boxed_str());
            c.phone = Box::leak(phone.get().into_boxed_str());
            c.email = Box::leak(email.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <Modal title="Edit Contact Person" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Full Name"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=name on:input:target=move |ev| name.set(ev.target().value()) />
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Job Title"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=title on:input:target=move |ev| title.set(ev.target().value()) />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Phone"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=phone on:input:target=move |ev| phone.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Email"</label>
                        <input type="email" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=email on:input:target=move |ev| email.set(ev.target().value()) />
                    </div>
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn EditBusinessModal(business: RwSignal<BusinessInfo>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let b0 = business.get_untracked();
    let registration_number = RwSignal::new(b0.registration_number.to_string());
    let tin = RwSignal::new(b0.tin.to_string());
    let vat = RwSignal::new(b0.vat.to_string());
    let established_year = RwSignal::new(b0.established_year.to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        business.update(|b| {
            b.registration_number = Box::leak(registration_number.get().into_boxed_str());
            b.tin = Box::leak(tin.get().into_boxed_str());
            b.vat = Box::leak(vat.get().into_boxed_str());
            b.established_year = Box::leak(established_year.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <Modal title="Edit Business Information" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Business Registration Number"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=registration_number on:input:target=move |ev| registration_number.set(ev.target().value()) />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"Tax Identification Number (TIN)"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=tin on:input:target=move |ev| tin.set(ev.target().value()) />
                    </div>
                    <div>
                        <label class="mb-1 block text-xs font-medium text-slate-500">"VAT Number"</label>
                        <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=vat on:input:target=move |ev| vat.set(ev.target().value()) />
                    </div>
                </div>
                <div>
                    <label class="mb-1 block text-xs font-medium text-slate-500">"Established Year"</label>
                    <input type="text" class="w-full max-w-[10rem] rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=established_year on:input:target=move |ev| established_year.set(ev.target().value()) />
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn EditSocialModal(social: RwSignal<SocialLinks>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let s0 = social.get_untracked();
    let facebook = RwSignal::new(s0.facebook.to_string());
    let instagram = RwSignal::new(s0.instagram.to_string());
    let twitter = RwSignal::new(s0.twitter.to_string());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        social.update(|s| {
            s.facebook = Box::leak(facebook.get().into_boxed_str());
            s.instagram = Box::leak(instagram.get().into_boxed_str());
            s.twitter = Box::leak(twitter.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <Modal title="Edit Social Media Links" on_close=on_close>
            <form on:submit=on_submit class="flex flex-col gap-4">
                <div>
                    <label class="mb-1 flex items-center gap-1.5 text-xs font-medium text-slate-500"><Icon name="facebook" class="h-3.5 w-3.5 text-blue-600" />"Facebook"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=facebook on:input:target=move |ev| facebook.set(ev.target().value()) />
                </div>
                <div>
                    <label class="mb-1 flex items-center gap-1.5 text-xs font-medium text-slate-500"><Icon name="instagram" class="h-3.5 w-3.5 text-pink-600" />"Instagram"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=instagram on:input:target=move |ev| instagram.set(ev.target().value()) />
                </div>
                <div>
                    <label class="mb-1 flex items-center gap-1.5 text-xs font-medium text-slate-500"><Icon name="twitter" class="h-3.5 w-3.5 text-slate-800" />"X / Twitter"</label>
                    <input type="text" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm" prop:value=twitter on:input:target=move |ev| twitter.set(ev.target().value()) />
                </div>
                <div class="mt-2 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button type="submit" class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-semibold text-white transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98]">"Save Changes"</button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn AmenitiesModal(amenities: RwSignal<Vec<&'static str>>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let draft = RwSignal::new(amenities.get_untracked());
    let custom_input = RwSignal::new(String::new());

    let toggle = move |label: &'static str| {
        draft.update(|list| {
            if let Some(pos) = list.iter().position(|l| *l == label) {
                list.remove(pos);
            } else {
                list.push(label);
            }
        });
    };

    let add_custom = move || {
        let value = custom_input.get();
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            let leaked: &'static str = Box::leak(trimmed.to_string().into_boxed_str());
            draft.update(|list| {
                if !list.contains(&leaked) {
                    list.push(leaked);
                }
            });
            custom_input.set(String::new());
        }
    };

    let save = move |_| {
        amenities.set(draft.get());
        on_close();
    };

    view! {
        <div class="fixed inset-0 z-30 flex items-center justify-center bg-black/40 p-4 animate-fade-in">
            <div class="max-h-[90vh] w-full max-w-3xl overflow-y-auto rounded-2xl bg-white p-6 animate-scale-in">
                <div class="mb-1 flex items-center justify-between">
                    <h2 class="text-lg font-bold text-slate-900">"Edit Amenities"</h2>
                    <button on:click=move |_| on_close()><Icon name="x" class="h-5 w-5 text-slate-400" /></button>
                </div>
                <p class="mb-5 text-sm text-slate-500">"Select the amenities available at your hotel."</p>

                <p class="mb-2 text-sm font-semibold text-slate-800">"Amenity Categories"</p>
                <div class="grid gap-6 sm:grid-cols-3">
                    {AMENITY_CATEGORIES.iter().map(|(cat, cat_icon, items)| view! {
                        <div>
                            <p class="mb-2 flex items-center gap-1.5 text-sm font-semibold text-slate-700">
                                <Icon name=*cat_icon class="h-4 w-4 text-blue-600" />
                                {*cat}
                            </p>
                            <div class="flex flex-col gap-2">
                                {items.iter().map(|(icon, label, _)| {
                                    let label = *label;
                                    let icon = *icon;
                                    view! {
                                        <div class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
                                            <span class="flex items-center gap-2 text-sm text-slate-700">
                                                <Icon name=icon class="h-4 w-4 text-slate-400" />
                                                {label}
                                            </span>
                                            <button type="button" on:click=move |_| toggle(label)>
                                                <Toggle checked=draft.get().contains(&label) />
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }).collect_view()}
                </div>

                <div class="mt-6 grid gap-6 sm:grid-cols-2">
                    <div>
                        <p class="mb-1 text-sm font-semibold text-slate-800">"Custom Amenity"</p>
                        <p class="mb-2 text-xs text-slate-500">"Add a custom amenity not listed above."</p>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                placeholder="Enter amenity name"
                                class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
                                prop:value=custom_input
                                on:input:target=move |ev| custom_input.set(ev.target().value())
                                on:keydown=move |ev| { if ev.key() == "Enter" { ev.prevent_default(); add_custom(); } }
                            />
                            <button type="button" on:click=move |_| add_custom() class="rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white">"Add"</button>
                        </div>
                    </div>
                    <div>
                        <p class="mb-2 text-sm font-semibold text-slate-800">{move || format!("Selected Amenities ({})", draft.get().len())}</p>
                        <div class="flex flex-wrap gap-2">
                            {move || draft.get().into_iter().map(|label| view! {
                                <span class="flex items-center gap-1 rounded-full bg-blue-50 px-2.5 py-1 text-xs text-blue-700">
                                    {label}
                                    <button type="button" on:click=move |_| toggle(label)><Icon name="x" class="h-3 w-3" /></button>
                                </span>
                            }).collect_view()}
                        </div>
                        <p class="mt-2 flex items-center gap-1 text-xs text-slate-400">
                            <Icon name="info" class="h-3.5 w-3.5" />
                            "Drag to reorder amenities (optional)."
                        </p>
                    </div>
                </div>

                <div class="mt-6 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button class="rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white" on:click=save>"Save Changes"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PoliciesModal(hotel: RwSignal<HotelInfo>, policies: RwSignal<PolicyInfo>, on_close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let h0 = hotel.get_untracked();
    let p0 = policies.get_untracked();
    let check_in_time = RwSignal::new(h0.check_in_time.to_string());
    let check_out_time = RwSignal::new(h0.check_out_time.to_string());
    let children_allowed = RwSignal::new(p0.children_allowed);
    let extra_bed = RwSignal::new(p0.extra_bed);
    let child_free_age = RwSignal::new(p0.child_free_age.to_string());
    let pets_allowed = RwSignal::new(p0.pets_allowed);
    let smoking_allowed = RwSignal::new(p0.smoking_allowed);
    let non_smoking = RwSignal::new(p0.non_smoking);
    let govt_id = RwSignal::new(p0.govt_id_required);
    let min_age = RwSignal::new(p0.min_age.to_string());
    let parties = RwSignal::new(p0.parties_allowed);
    let notes = RwSignal::new(p0.notes.to_string());

    let save = move |_| {
        hotel.update(|h| {
            h.check_in_time = Box::leak(check_in_time.get().into_boxed_str());
            h.check_out_time = Box::leak(check_out_time.get().into_boxed_str());
        });
        policies.update(|p| {
            p.children_allowed = children_allowed.get();
            p.extra_bed = extra_bed.get();
            p.child_free_age = Box::leak(child_free_age.get().into_boxed_str());
            p.pets_allowed = pets_allowed.get();
            p.smoking_allowed = smoking_allowed.get();
            p.non_smoking = non_smoking.get();
            p.govt_id_required = govt_id.get();
            p.min_age = Box::leak(min_age.get().into_boxed_str());
            p.parties_allowed = parties.get();
            p.notes = Box::leak(notes.get().into_boxed_str());
        });
        on_close();
    };

    view! {
        <div class="fixed inset-0 z-30 flex items-center justify-center bg-black/40 p-4 animate-fade-in">
            <div class="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-2xl bg-white p-6 animate-scale-in">
                <div class="mb-1 flex items-center justify-between">
                    <h2 class="text-lg font-bold text-slate-900">"Edit Policies"</h2>
                    <button on:click=move |_| on_close()><Icon name="x" class="h-5 w-5 text-slate-400" /></button>
                </div>
                <p class="mb-5 text-sm text-slate-500">"Configure the policies and rules for your hotel."</p>

                <div class="grid gap-4 sm:grid-cols-2">
                    <div class="rounded-xl border border-slate-200 p-4">
                        <p class="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                            <Icon name="clock" class="h-4 w-4 text-blue-600" />
                            "Check-in & Check-out"
                        </p>
                        <div class="grid grid-cols-2 gap-3 text-sm">
                            <div><label class="mb-1 block text-xs text-slate-500">"Check-in Time"</label><input type="text" class="w-full rounded-lg border border-slate-300 px-2 py-1.5" prop:value=check_in_time on:input:target=move |ev| check_in_time.set(ev.target().value()) /></div>
                            <div><label class="mb-1 block text-xs text-slate-500">"Check-out Time"</label><input type="text" class="w-full rounded-lg border border-slate-300 px-2 py-1.5" prop:value=check_out_time on:input:target=move |ev| check_out_time.set(ev.target().value()) /></div>
                        </div>
                    </div>

                    <div class="rounded-xl border border-slate-200 p-4">
                        <p class="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                            <Icon name="family" class="h-4 w-4 text-blue-600" />
                            "Children & Extra Beds"
                        </p>
                        <div class="flex flex-col gap-2 text-sm">
                            <div class="flex items-center justify-between">
                                <span>"Children Allowed"</span>
                                <button type="button" on:click=move |_| children_allowed.update(|v| *v = !*v)><Toggle checked=children_allowed.get() /></button>
                            </div>
                            <div class="flex items-center justify-between">
                                <span>"Extra Bed Available"</span>
                                <button type="button" on:click=move |_| extra_bed.update(|v| *v = !*v)><Toggle checked=extra_bed.get() /></button>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-slate-500">"Child age considered free up to:"</span>
                                <input type="text" class="w-14 rounded-lg border border-slate-300 px-2 py-1 text-center" prop:value=child_free_age on:input:target=move |ev| child_free_age.set(ev.target().value()) />
                            </div>
                        </div>
                    </div>

                    <div class="rounded-xl border border-slate-200 p-4">
                        <p class="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                            <Icon name="paw" class="h-4 w-4 text-blue-600" />
                            "Pets & Smoking"
                        </p>
                        <div class="flex flex-col gap-2 text-sm">
                            <div class="flex items-center justify-between"><span>"Pets Allowed"</span><button type="button" on:click=move |_| pets_allowed.update(|v| *v = !*v)><Toggle checked=pets_allowed.get() /></button></div>
                            <div class="flex items-center justify-between"><span>"Smoking Allowed"</span><button type="button" on:click=move |_| smoking_allowed.update(|v| *v = !*v)><Toggle checked=smoking_allowed.get() /></button></div>
                            <div class="flex items-center justify-between"><span>"Non-smoking Property"</span><button type="button" on:click=move |_| non_smoking.update(|v| *v = !*v)><Toggle checked=non_smoking.get() /></button></div>
                        </div>
                    </div>

                    <div class="rounded-xl border border-slate-200 p-4">
                        <p class="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                            <Icon name="id-card" class="h-4 w-4 text-blue-600" />
                            "House Rules / Guest Requirements"
                        </p>
                        <div class="flex flex-col gap-2 text-sm">
                            <div class="flex items-center justify-between"><span>"Government ID Required"</span><button type="button" on:click=move |_| govt_id.update(|v| *v = !*v)><Toggle checked=govt_id.get() /></button></div>
                            <div class="flex items-center justify-between">
                                <span>"Minimum Check-in Age"</span>
                                <input type="text" class="w-14 rounded-lg border border-slate-300 px-2 py-1 text-center" prop:value=min_age on:input:target=move |ev| min_age.set(ev.target().value()) />
                            </div>
                            <div class="flex items-center justify-between"><span>"Parties or Events Allowed"</span><button type="button" on:click=move |_| parties.update(|v| *v = !*v)><Toggle checked=parties.get() /></button></div>
                        </div>
                    </div>
                </div>

                <div class="mt-4 rounded-xl border border-slate-200 p-4">
                    <p class="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-800">
                        <Icon name="clock" class="h-4 w-4 text-blue-600" />
                        "Policy Notes for Guests"
                    </p>
                    <textarea class="w-full rounded-lg border border-slate-300 p-2 text-sm" rows="2" prop:value=notes on:input:target=move |ev| notes.set(ev.target().value())></textarea>
                </div>

                <div class="mt-6 flex justify-end gap-3 border-t border-slate-100 pt-4">
                    <button class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium" on:click=move |_| on_close()>"Cancel"</button>
                    <button class="rounded-lg bg-blue-700 transition-all duration-200 hover:bg-blue-800 hover:shadow-md active:scale-[0.98] px-4 py-2 text-sm font-semibold text-white" on:click=save>"Save Changes"</button>
                </div>
            </div>
        </div>
    }
}
