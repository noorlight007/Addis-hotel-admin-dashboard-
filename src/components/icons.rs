use leptos::prelude::*;

#[component]
pub fn Icon(
    name: &'static str,
    #[prop(default = "h-4 w-4")] class: &'static str,
) -> impl IntoView {
    let path = match name {
        "home" => view! { <path d="M3 10l9-7 9 7"/><path d="M5 9.5V20a1 1 0 0 0 1 1h4v-6h4v6h4a1 1 0 0 0 1-1V9.5"/> }.into_any(),
        "calendar" => view! { <rect x="3" y="4.5" width="18" height="16" rx="2"/><path d="M16 2.5v4M8 2.5v4M3 10h18"/> }.into_any(),
        "calendar-check" => view! { <rect x="3" y="4.5" width="18" height="16" rx="2"/><path d="M16 2.5v4M8 2.5v4M3 10h18"/><path d="M8.5 15l1.8 1.8L14.5 13"/> }.into_any(),
        "calendar-x" => view! { <rect x="3" y="4.5" width="18" height="16" rx="2"/><path d="M16 2.5v4M8 2.5v4M3 10h18"/><path d="M9 13l4 4M13 13l-4 4"/> }.into_any(),
        "bed" => view! { <path d="M3 18v-6.5A2 2 0 0 1 5 9.5h14a2 2 0 0 1 2 2V18"/><path d="M2.5 18h19M7 9.5V6a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v3.5"/> }.into_any(),
        "users" => view! { <path d="M2 21v-1.5A4.5 4.5 0 0 1 6.5 15h3A4.5 4.5 0 0 1 14 19.5V21"/><circle cx="8" cy="8" r="4"/><path d="M16.5 21v-1.5a4 4 0 0 0-2.4-3.7"/><path d="M14.5 3.2a4 4 0 0 1 0 7.7"/> }.into_any(),
        "user-plus" => view! { <path d="M2 21v-1.5A4.5 4.5 0 0 1 6.5 15h1A4.5 4.5 0 0 1 12 19.5V21"/><circle cx="7" cy="8" r="4"/><path d="M17 8h5M19.5 5.5v5"/> }.into_any(),
        "star" => view! { <path d="M12 2.5l2.9 6 6.6.9-4.8 4.6 1.2 6.5-5.9-3.2-5.9 3.2 1.2-6.5-4.8-4.6 6.6-.9z"/> }.into_any(),
        "star-fill" => view! { <path d="M12 2.5l2.9 6 6.6.9-4.8 4.6 1.2 6.5-5.9-3.2-5.9 3.2 1.2-6.5-4.8-4.6 6.6-.9z" fill="currentColor" stroke="none"/> }.into_any(),
        "mail" => view! { <rect x="3" y="5.5" width="18" height="13" rx="2"/><path d="M3.5 6.5l8.5 6.5 8.5-6.5"/> }.into_any(),
        "message" => view! { <path d="M21 11.6c0 4.7-4 8.4-9 8.4-1 0-1.9-.1-2.8-.4L4 21l1.5-4.3A8.6 8.6 0 0 1 3 11.6C3 7 7 3.2 12 3.2s9 3.8 9 8.4z"/> }.into_any(),
        "settings" => view! { <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9c.1.7.7 1.3 1.5 1.5H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1.5z"/> }.into_any(),
        "wrench" => view! { <path d="M14.7 6.3a4 4 0 1 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.1 2.1-2-2z"/> }.into_any(),
        "log-out" => view! { <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="M16 17l5-5-5-5"/><path d="M21 12H9"/> }.into_any(),
        "search" => view! { <circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/> }.into_any(),
        "sliders" => view! { <path d="M4 5.5h5M13 5.5h7M4 12h9M17 12h3M4 18.5h4M12 18.5h8"/><circle cx="10.5" cy="5.5" r="1.8"/><circle cx="15" cy="12" r="1.8"/><circle cx="9.5" cy="18.5" r="1.8"/> }.into_any(),
        "plus" => view! { <path d="M12 5v14M5 12h14"/> }.into_any(),
        "upload" => view! { <path d="M12 15V3"/><path d="M7 8l5-5 5 5"/><path d="M5 21h14"/> }.into_any(),
        "download" => view! { <path d="M12 3v11.5"/><path d="M7 10l5 5 5-5"/><path d="M4.5 20.5h15"/> }.into_any(),
        "eye" => view! { <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7z"/><circle cx="12" cy="12" r="3"/> }.into_any(),
        "eye-off" => view! { <path d="M3 3l18 18"/><path d="M10.6 5.1A10.4 10.4 0 0 1 12 5c6.4 0 10 7 10 7a17 17 0 0 1-3.3 4.2M6.6 6.6C4 8.3 2 12 2 12s3.6 7 10 7a9.6 9.6 0 0 0 3.9-.8"/><path d="M9.5 9.8a3 3 0 0 0 4.2 4.2"/> }.into_any(),
        "edit" => view! { <path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z"/> }.into_any(),
        "more" => view! { <circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/> }.into_any(),
        "more-v" => view! { <circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/> }.into_any(),
        "trash" => view! { <path d="M3 6.5h18"/><path d="M8 6.5V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2.5"/><path d="M19 6.5l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2l-1-14"/> }.into_any(),
        "chevron-left" => view! { <path d="M15 18l-6-6 6-6"/> }.into_any(),
        "chevron-right" => view! { <path d="M9 18l6-6-6-6"/> }.into_any(),
        "chevron-down" => view! { <path d="M6 9l6 6 6-6"/> }.into_any(),
        "check" => view! { <path d="M4 12.5l5 5 11-11"/> }.into_any(),
        "check-circle" => view! { <circle cx="12" cy="12" r="9"/><path d="M8 12.3l2.6 2.6L16 9.3"/> }.into_any(),
        "x" => view! { <path d="M18 6L6 18M6 6l12 12"/> }.into_any(),
        "x-circle" => view! { <circle cx="12" cy="12" r="9"/><path d="M9.5 9.5l5 5M14.5 9.5l-5 5"/> }.into_any(),
        "shield-check" => view! { <path d="M12 2.5l7.5 3.4v5.4c0 5-3.2 8.5-7.5 10-4.3-1.5-7.5-5-7.5-10V5.9z"/><path d="M8.5 12l2.3 2.3L15.5 9.5"/> }.into_any(),
        "map-pin" => view! { <path d="M12 21s7-6.2 7-11.3A7 7 0 0 0 5 9.7C5 14.8 12 21 12 21z"/><circle cx="12" cy="9.5" r="2.4"/> }.into_any(),
        "phone" => view! { <path d="M5 4h3.5l1.4 4.2-2 1.6a12.5 12.5 0 0 0 5.9 5.9l1.6-2 4.2 1.4V19a1 1 0 0 1-1 1C10.7 20 4 13.3 4 5a1 1 0 0 1 1-1z"/> }.into_any(),
        "building" => view! { <rect x="4" y="2.5" width="16" height="19" rx="1.2"/><path d="M8.5 7.5h1M14.5 7.5h1M8.5 11.5h1M14.5 11.5h1M8.5 15.5h1M14.5 15.5h1"/><path d="M9.5 21.5v-3.8h5v3.8"/> }.into_any(),
        "camera" => view! { <path d="M4 8.5a1.5 1.5 0 0 1 1.5-1.5H8l1.2-2h5.6l1.2 2h2.5A1.5 1.5 0 0 1 20 8.5v9A1.5 1.5 0 0 1 18.5 19h-13A1.5 1.5 0 0 1 4 17.5z"/><circle cx="12" cy="13" r="3.3"/> }.into_any(),
        "image" => view! { <rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8.5" cy="9.5" r="1.5"/><path d="M21 15.5l-5-5-4.5 4.5-2.5-2.5-6 6"/> }.into_any(),
        "clock" => view! { <circle cx="12" cy="12" r="9"/><path d="M12 7.5V12l3 2.2"/> }.into_any(),
        "credit-card" => view! { <rect x="2.5" y="5.5" width="19" height="13" rx="2"/><path d="M2.5 10h19"/><path d="M6 14.5h4"/> }.into_any(),
        "banknote" => view! { <rect x="2.5" y="6.5" width="19" height="11" rx="1.5"/><circle cx="12" cy="12" r="2.5"/><path d="M6 9v.01M18 15v.01"/> }.into_any(),
        "globe" => view! { <circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.4 2.5 3.8 5.7 3.8 9s-1.4 6.5-3.8 9c-2.4-2.5-3.8-5.7-3.8-9s1.4-6.5 3.8-9z"/> }.into_any(),
        "id-card" => view! { <rect x="2.5" y="5" width="19" height="14" rx="2"/><circle cx="8" cy="12" r="2"/><path d="M6 16.2c.4-1.2 1-1.7 2-1.7s1.6.5 2 1.7"/><path d="M13.5 9.5h6M13.5 13h6M13.5 16.2h4"/> }.into_any(),
        "briefcase" => view! { <rect x="2.5" y="7.5" width="19" height="12" rx="2"/><path d="M8 7.5V6a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v1.5"/><path d="M2.5 13h19"/> }.into_any(),
        "monitor" => view! { <rect x="2.5" y="4.5" width="19" height="12.5" rx="1.5"/><path d="M8 20.5h8M12 17v3.5"/> }.into_any(),
        "printer" => view! { <path d="M6 8V4h12v4"/><rect x="3.5" y="8" width="17" height="8" rx="1.5"/><path d="M7 16v4h10v-4"/> }.into_any(),
        "wifi" => view! { <path d="M4.5 11.5a10.7 10.7 0 0 1 15 0M7.8 14.8a6.4 6.4 0 0 1 8.4 0M11 18h.01" stroke-linecap="round"/> }.into_any(),
        "shuttle" => view! { <rect x="2.5" y="6" width="19" height="10.5" rx="2.2"/><circle cx="7.5" cy="18.5" r="1.6"/><circle cx="16.5" cy="18.5" r="1.6"/><path d="M2.5 11h19"/> }.into_any(),
        "snowflake" => view! { <path d="M12 2.5v19M4.5 7l15 10M19.5 7l-15 10"/> }.into_any(),
        "tv" => view! { <rect x="3" y="4.5" width="18" height="12" rx="1.5"/><path d="M8 20.5h8"/> }.into_any(),
        "minibar" => view! { <rect x="5" y="3" width="14" height="18" rx="1.5"/><path d="M5 10h14"/><path d="M9 6h.01M9 14h.01"/> }.into_any(),
        "safe" => view! { <rect x="3.5" y="3.5" width="17" height="17" rx="2"/><circle cx="12" cy="12" r="3.2"/><path d="M12 10.2V12l1.4.9"/> }.into_any(),
        "desk" => view! { <path d="M3 8h18M5 8v12M19 8v12M3 15h7"/> }.into_any(),
        "coffee" => view! { <path d="M5 9h13v3.5A5.5 5.5 0 0 1 12.5 18h-2A5.5 5.5 0 0 1 5 12.5z"/><path d="M18 9.5h1.5a2 2 0 0 1 0 4H18"/><path d="M8 5.5c-.6.5-.6 1 0 1.5M12 5.5c-.6.5-.6 1 0 1.5"/> }.into_any(),
        "hairdryer" => view! { <path d="M9 8a4 4 0 0 1 4-4h1a4 4 0 0 1 0 8h-1.5"/><path d="M9 8v4l-4 6"/><path d="M18 9.5h3M18 12h2.3"/> }.into_any(),
        "telephone" => view! { <path d="M8 3.5h8v6H8z"/><path d="M9.5 9.5v11a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-11"/> }.into_any(),
        "slippers" => view! { <path d="M3 16c0-3 2.5-5 6-5s6 1 6 4-2.5 3.5-6 3.5-6-.7-6-2.5z"/><path d="M9.5 11l1-4.5"/> }.into_any(),
        "wardrobe" => view! { <rect x="4" y="2.5" width="16" height="19" rx="1.2"/><path d="M12 2.5v19"/><path d="M8.5 12v1.5M15.5 12v1.5"/> }.into_any(),
        "iron" => view! { <path d="M4 17c0-4.5 3-8 9-8 4 0 7 2.5 7 5.5 0 1.5-1 2.5-2.5 2.5H6c-1 0-2-.3-2-1z"/><path d="M9 9V6.5"/> }.into_any(),
        "crib" => view! { <path d="M3 8h18M5 8v12M19 8v12"/><path d="M3 20h18"/><path d="M5 8c1.5-3 3.5-4.5 7-4.5S17.5 5 19 8"/> }.into_any(),
        "bell-concierge" => view! { <path d="M3 17h18"/><path d="M5 17a7 7 0 0 1 14 0"/><path d="M12 10V7M9.5 7h5"/> }.into_any(),
        "balcony" => view! { <rect x="4" y="3" width="16" height="18" rx="1"/><path d="M4 9h16"/><path d="M9 3v6M15 3v6"/> }.into_any(),
        "bathtub" => view! { <path d="M3 12.5h18v2A4.5 4.5 0 0 1 16.5 19h-9A4.5 4.5 0 0 1 3 14.5z"/><path d="M5 12.5V7a2 2 0 0 1 3.5-1.3"/><path d="M4 19l-.8 2M20 19l.8 2"/> }.into_any(),
        "umbrella" => view! { <path d="M3 12a9 9 0 0 1 18 0z"/><path d="M12 12v7a2 2 0 0 1-3.5 1.3"/><path d="M12 3v2"/> }.into_any(),
        "dumbbell" => view! { <path d="M4 9v6M7 7.5v9M17 7.5v9M20 9v6"/><path d="M7 12h10"/> }.into_any(),
        "sparkles" => view! { <path d="M12 3l1.3 3.7L17 8l-3.7 1.3L12 13l-1.3-3.7L7 8l3.7-1.3z"/><path d="M5 15l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7z"/> }.into_any(),
        "paw" => view! { <circle cx="7" cy="9" r="2"/><circle cx="12" cy="7" r="2"/><circle cx="17" cy="9" r="2"/><path d="M6 15.5c0-2 1.8-3 6-3s6 1 6 3-2 3.5-6 3.5-6-1.5-6-3.5z"/> }.into_any(),
        "wheelchair" => view! { <circle cx="9" cy="4.5" r="1.6"/><path d="M9 8v6h6l3 6"/><path d="M9 12h5"/><circle cx="10.5" cy="17.5" r="4"/> }.into_any(),
        "family" => view! { <circle cx="7" cy="7" r="2.3"/><circle cx="15" cy="7" r="2.3"/><circle cx="11" cy="16" r="1.8"/><path d="M3 18c0-2.5 1.8-4 4-4s4 1.5 4 4M13 18c0-2-1.4-3.3-3-3.5M13 14.5c1.6.2 3 1.5 3 3.5M11 18v2.5"/> }.into_any(),
        "moon" => view! { <path d="M20 14.5A8.5 8.5 0 1 1 9.5 4 6.5 6.5 0 0 0 20 14.5z"/> }.into_any(),
        "info" => view! { <circle cx="12" cy="12" r="9"/><path d="M12 8h.01"/><path d="M11 11.5h1v5"/><path d="M10 16.5h4"/> }.into_any(),
        "arrow-left" => view! { <path d="M19 12H5M11 5l-6 7 6 7"/> }.into_any(),
        "external-link" => view! { <path d="M14 4.5h5.5V10"/><path d="M19.2 5L11 13.2"/><path d="M18 13v6a1.5 1.5 0 0 1-1.5 1.5H5.5A1.5 1.5 0 0 1 4 19V7.5A1.5 1.5 0 0 1 5.5 6H11"/> }.into_any(),
        "package" => view! { <path d="M3.5 7.5L12 3l8.5 4.5L12 12z"/><path d="M3.5 7.5V16.5L12 21l8.5-4.5V7.5"/><path d="M12 12v9"/> }.into_any(),
        "shirt-off" => view! { <circle cx="12" cy="6.5" r="2"/><path d="M4 21c0-4 3-6 8-6s8 2 8 6"/><path d="M5 8l-2 3 2 3M19 8l2 3-2 3"/> }.into_any(),
        "tag" => view! { <path d="M20.6 12.1L12 20.7 2.8 11.5 11.4 2.9h8.2a1 1 0 0 1 1 1z"/><circle cx="16" cy="7" r="1.3"/> }.into_any(),
        "lock" => view! { <rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V7a4 4 0 1 1 8 0v4"/> }.into_any(),
        "bell" => view! { <path d="M6 8.5a6 6 0 1 1 12 0c0 3.8 1.3 4.9 1.8 5.8H4.2c.5-.9 1.8-2 1.8-5.8z"/><path d="M10 19a2 2 0 0 0 4 0"/> }.into_any(),
        "flag" => view! { <path d="M5 21V4"/><path d="M5 4h13l-3 4 3 4H5"/> }.into_any(),
        "headset" => view! { <path d="M4 13v-1a8 8 0 1 1 16 0v1"/><rect x="3" y="13" width="4" height="6" rx="1.3"/><rect x="17" y="13" width="4" height="6" rx="1.3"/><path d="M20 19a4 4 0 0 1-4 3h-2"/> }.into_any(),
        "file-text" => view! { <path d="M6 2.5h9l4 4V20a1.2 1.2 0 0 1-1.2 1.2H6A1.2 1.2 0 0 1 4.8 20V3.7A1.2 1.2 0 0 1 6 2.5z"/><path d="M14.5 2.5V7h4.5"/><path d="M7.5 12h9M7.5 15.5h9M7.5 18.5h6"/> }.into_any(),
        "trending-up" => view! { <path d="M3 17l6-6 4 4 8-8"/><path d="M15 7h6v6"/> }.into_any(),
        "trending-down" => view! { <path d="M3 7l6 6 4-4 8 8"/><path d="M15 17h6v-6"/> }.into_any(),
        "arrow-right" => view! { <path d="M5 12h14M13 5l7 7-7 7"/> }.into_any(),
        "arrow-up-right" => view! { <path d="M7 17L17 7M8 7h9v9"/> }.into_any(),
        "chevron-up" => view! { <path d="M6 15l6-6 6 6"/> }.into_any(),
        "menu" => view! { <path d="M4 6.5h16M4 12h16M4 17.5h16"/> }.into_any(),
        "grid" => view! { <rect x="3.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="3.5" y="13.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="13.5" width="7" height="7" rx="1.5"/> }.into_any(),
        "list" => view! { <path d="M9 6.5h11M9 12h11M9 17.5h11"/><circle cx="4.5" cy="6.5" r="1.3" fill="currentColor" stroke="none"/><circle cx="4.5" cy="12" r="1.3" fill="currentColor" stroke="none"/><circle cx="4.5" cy="17.5" r="1.3" fill="currentColor" stroke="none"/> }.into_any(),
        "percent" => view! { <path d="M19 5L5 19"/><circle cx="7.5" cy="7.5" r="2.5"/><circle cx="16.5" cy="16.5" r="2.5"/> }.into_any(),
        "key" => view! { <circle cx="7" cy="12" r="4"/><path d="M11 12h10"/><path d="M17.5 12v3.5M20.5 12v2.5"/> }.into_any(),
        "help-circle" => view! { <circle cx="12" cy="12" r="9"/><path d="M9.6 9.4a2.5 2.5 0 1 1 3.4 2.3c-.7.3-1 .9-1 1.6v.4"/><path d="M12 17h.01"/> }.into_any(),
        "alert" => view! { <path d="M12 3.5L21.5 20H2.5z"/><path d="M12 9.5v4.5M12 17.2h.01"/> }.into_any(),
        "wallet" => view! { <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H18a1 1 0 0 1 1 1v1.5"/><rect x="3" y="7.5" width="18" height="12" rx="2.5"/><circle cx="16.5" cy="13.5" r="1.3" fill="currentColor" stroke="none"/> }.into_any(),
        "activity" => view! { <path d="M3 12h4l3 8 4-16 3 8h4"/> }.into_any(),
        "pie-chart" => view! { <path d="M12 3v9h9a9 9 0 1 0-9-9z"/><path d="M20.5 15.5A9 9 0 0 1 12 21"/> }.into_any(),
        "bar-chart" => view! { <path d="M4 20V10M10 20V4M16 20v-7M22 20H2"/> }.into_any(),
        "loader" => view! { <path d="M12 3v3.5M12 17.5V21M4.9 4.9l2.5 2.5M16.6 16.6l2.5 2.5M3 12h3.5M17.5 12H21M4.9 19.1l2.5-2.5M16.6 7.4l2.5-2.5"/> }.into_any(),
        "minus" => view! { <path d="M5 12h14"/> }.into_any(),
        "refresh" => view! { <path d="M20.5 12a8.5 8.5 0 1 1-2.6-6.1"/><path d="M20.5 4v5h-5"/> }.into_any(),
        "send" => view! { <path d="M21 3L10.5 13.5M21 3l-6.8 18-3.7-7.5L3 9.8z"/> }.into_any(),
        "zap" => view! { <path d="M13 2.5L4 13.5h7l-1 8 9-11h-7z"/> }.into_any(),
        "award" => view! { <circle cx="12" cy="9" r="6"/><path d="M8.5 14.2L7 22l5-2.6L17 22l-1.5-7.8"/> }.into_any(),
        "target" => view! { <circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none"/> }.into_any(),
        "user-x" => view! { <path d="M2 21c0-3.6 2.7-5.5 6-5.5s6 1.9 6 5.5"/><circle cx="8" cy="8" r="4"/><path d="M17 9l4 4M21 9l-4 4"/> }.into_any(),
        "smile" => view! { <circle cx="12" cy="12" r="9"/><path d="M8.5 14a4.5 4.5 0 0 0 7 0"/><path d="M9 9.5h.01M15 9.5h.01"/> }.into_any(),
        "quote" => view! { <path d="M9.5 6C6.5 7.2 5 9.8 5 13.8V18h5.5v-5.5H8c0-2.3.6-3.7 2.4-4.6z" fill="currentColor" stroke="none"/><path d="M19 6c-3 1.2-4.5 3.8-4.5 7.8V18H20v-5.5h-2.5c0-2.3.6-3.7 2.4-4.6z" fill="currentColor" stroke="none"/> }.into_any(),
        "sun" => view! { <circle cx="12" cy="12" r="4"/><path d="M12 2.5v2.5M12 19v2.5M4.2 4.2l1.8 1.8M18 18l1.8 1.8M2.5 12H5M19 12h2.5M4.2 19.8L6 18M18 6l1.8-1.8"/> }.into_any(),
        "gift" => view! { <rect x="3" y="8.5" width="18" height="4" rx="1"/><path d="M4.5 12.5v7A1.5 1.5 0 0 0 6 21h12a1.5 1.5 0 0 0 1.5-1.5v-7M12 8.5V21"/><path d="M12 8.5S10.5 3 8 3a2.5 2.5 0 0 0 0 5.5zM12 8.5S13.5 3 16 3a2.5 2.5 0 0 1 0 5.5z"/> }.into_any(),
        "sort" => view! { <path d="M7 4.5v15M7 4.5L4 8M7 4.5l3 3.5M17 19.5v-15M17 19.5l-3-3.5M17 19.5l3-3.5"/> }.into_any(),
        "waves" => view! { <path d="M2.5 7.5c2-1.6 3.5-1.6 5.5 0s3.5 1.6 5.5 0 3.5-1.6 5.5 0"/><path d="M2.5 12.5c2-1.6 3.5-1.6 5.5 0s3.5 1.6 5.5 0 3.5-1.6 5.5 0"/><path d="M2.5 17.5c2-1.6 3.5-1.6 5.5 0s3.5 1.6 5.5 0 3.5-1.6 5.5 0"/> }.into_any(),
        "facebook" => view! { <path d="M14.5 21.5v-8h2.7l.4-3.2h-3.1V8.3c0-.9.3-1.6 1.6-1.6h1.7V3.9c-.3 0-1.3-.1-2.4-.1-2.4 0-4.1 1.5-4.1 4.2v2.3H8.6v3.2h2.7v8z" fill="currentColor" stroke="none"/> }.into_any(),
        "instagram" => view! { <rect x="3" y="3" width="18" height="18" rx="5"/><circle cx="12" cy="12" r="4"/><path d="M17.3 6.7h.01" stroke-linecap="round" stroke-width="2.4"/> }.into_any(),
        "twitter" => view! { <path d="M21 5.5c-.7.4-1.5.6-2.3.8a3.4 3.4 0 0 0-5.8 3.1A9.6 9.6 0 0 1 5.9 5.9a3.4 3.4 0 0 0 1.1 4.6c-.6 0-1.2-.2-1.7-.5v.1a3.4 3.4 0 0 0 2.7 3.4c-.5.1-1.1.2-1.6.1a3.4 3.4 0 0 0 3.2 2.4A6.9 6.9 0 0 1 3 17.4a9.7 9.7 0 0 0 5.3 1.6c6.3 0 9.8-5.4 9.8-10.1v-.5c.7-.5 1.3-1.1 1.8-1.8z" fill="currentColor" stroke="none"/> }.into_any(),
        "housekeeping" => view! { <path d="M6 10.5V8a3 3 0 0 1 6 0v2.5"/><rect x="4" y="10.5" width="10" height="7" rx="2"/><path d="M16.5 13.5c1.8-.5 3-1.4 3-3.5a2.5 2.5 0 0 0-5 0v1"/><path d="M18 13v3a1.5 1.5 0 0 1-1.5 1.5" /> }.into_any(),
        "utensils" => view! { <path d="M7 2.5v7a1.6 1.6 0 0 0 3.2 0v-7M8.6 9.5v12M15.5 2.5s-1.5 1-1.5 4 1.5 3.5 1.5 3.5V21"/> }.into_any(),
        "wine" => view! { <path d="M7 3h10l-1 6a4 4 0 0 1-8 0z"/><path d="M12 13v6M8.5 21.5h7"/> }.into_any(),
        _ => view! { <circle cx="12" cy="12" r="9"/> }.into_any(),
    };

    view! {
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            {path}
        </svg>
    }
}

#[component]
pub fn Toggle(checked: bool) -> impl IntoView {
    view! {
        <span class=format!(
            "relative inline-flex h-5 w-9 items-center rounded-full transition {}",
            if checked { "bg-blue-600" } else { "bg-slate-300" }
        )>
            <span class=format!(
                "inline-block h-4 w-4 transform rounded-full bg-white transition {}",
                if checked { "translate-x-4" } else { "translate-x-0.5" }
            )></span>
        </span>
    }
}
