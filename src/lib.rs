// Comparisons inside `view!` attributes must be parenthesised, otherwise the
// macro treats `>` as the end of the tag. rustc then sees the parens as
// redundant in the expanded code, so the lint is switched off crate-wide.
#![allow(unused_parens)]

pub mod api;
pub mod app;
pub mod components;
pub mod date;
pub mod pages;

pub use app::App;

/// Browser confirmation dialog, for destructive actions. Returns `false` when
/// no window is available (should not happen in the CSR app).
pub fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.confirm_with_message(message).ok())
        .unwrap_or(false)
}
