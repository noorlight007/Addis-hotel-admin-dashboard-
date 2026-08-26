use crate::components::{provide_layout, provide_toasts, Footer, Sidebar, ToastHost, Topbar};
use crate::pages::*;
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

/// Chrome shared by every authenticated page.
#[component]
fn Shell(children: Children) -> impl IntoView {
    view! {
        <div class="flex min-h-screen bg-slate-50">
            <Sidebar/>
            <div class="flex min-w-0 flex-1 flex-col">
                <Topbar/>
                <main class="flex-1">{children()}</main>
                <Footer/>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_layout();
    provide_toasts();

    view! {
        <Router>
            <ScrollToTopOnNavigate/>
            <Routes fallback=|| view! { <NotFoundPage/> }>
                // ---- Unauthenticated ------------------------------------
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/signup") view=SignupPage/>
                <Route path=path!("/forgot-password") view=ForgotPasswordPage/>

                // ---- Dashboard -------------------------------------------
                <Route path=path!("/") view=|| view! { <Shell><DashboardPage/></Shell> }/>
                <Route path=path!("/analytics") view=|| view! { <Shell><AnalyticsPage/></Shell> }/>
                <Route path=path!("/reservations") view=|| view! { <Shell><ReservationsPage/></Shell> }/>
                <Route path=path!("/calendar") view=|| view! { <Shell><CalendarPage/></Shell> }/>
                <Route path=path!("/rooms") view=|| view! { <Shell><RoomsPage/></Shell> }/>
                <Route path=path!("/rooms/bulk-upload") view=|| view! { <Shell><BulkUploadPage/></Shell> }/>
                <Route path=path!("/rooms/:number") view=|| view! { <Shell><RoomDetailsPage/></Shell> }/>
                <Route path=path!("/rates") view=|| view! { <Shell><RatesPage/></Shell> }/>
                <Route path=path!("/guests") view=|| view! { <Shell><GuestsPage/></Shell> }/>
                <Route path=path!("/guests/:booking_ref") view=|| view! { <Shell><GuestDetailsPage/></Shell> }/>
                <Route path=path!("/payments") view=|| view! { <Shell><PaymentsPage/></Shell> }/>
                <Route path=path!("/reviews") view=|| view! { <Shell><ReviewsPage/></Shell> }/>
                <Route path=path!("/messages") view=|| view! { <Shell><MessagesPage/></Shell> }/>
                <Route path=path!("/profile") view=|| view! { <Shell><HotelProfilePage/></Shell> }/>
                <Route path=path!("/staff") view=|| view! { <Shell><StaffPage/></Shell> }/>
                <Route path=path!("/settings") view=|| view! { <Shell><SettingsPage/></Shell> }/>
            </Routes>
            <ToastHost/>
        </Router>
    }
}

/// Resets the scroll offset on every route change.
///
/// Without this, moving from the bottom of a long table to another page lands
/// you halfway down the new one. Must live inside `<Router>` so `use_location`
/// has a router context.
#[component]
fn ScrollToTopOnNavigate() -> impl IntoView {
    let pathname = use_location().pathname;

    Effect::new(move |prev: Option<String>| {
        let path = pathname.get();
        if prev.is_some_and(|p| p != path) {
            // `scroll-behavior: smooth` is set site-wide; a route change should
            // land instantly rather than animate the whole page past the user.
            let opts = web_sys::ScrollToOptions::new();
            opts.set_top(0.0);
            opts.set_left(0.0);
            opts.set_behavior(web_sys::ScrollBehavior::Instant);
            window().scroll_to_with_scroll_to_options(&opts);
        }
        path
    });
}
