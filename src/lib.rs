// Comparisons inside `view!` attributes must be parenthesised, otherwise the
// macro treats `>` as the end of the tag. rustc then sees the parens as
// redundant in the expanded code, so the lint is switched off crate-wide.
#![allow(unused_parens)]

pub mod app;
pub mod components;
pub mod data;
pub mod date;
pub mod pages;

pub use app::App;
