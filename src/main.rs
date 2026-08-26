use addis_crm_frontend::App;
use leptos::prelude::*;

fn main() {
    _ = console_log::init_with_level(log::Level::Warn);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <App/> }
    })
}
