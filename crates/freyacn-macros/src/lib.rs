//! Procedural macros for the [`freyacn`] crate.
//!
//! This crate provides the attribute macros that turn plain Rust functions
//! and enums into the styled, prop-driven building blocks freyacn uses.
//!
//! * [`macro@component`] — turns a function into a component struct.
//! * [`macro@variant`]   — turns an enum into a style-resolving variant type.
//! * [`macro@extensions`] — opts a hand-written struct into a freyacn
//!   extension bundle (used standalone; the `ext(...)` group inside
//!   `#[component]` is the normal path).
//! * [`macro@struct_fields`] — stub attribute that fires when
//!   `#[struct_fields(...)]` is placed in the wrong position.
//!
//! [`freyacn`]: https://crates.io/crates/freyacn

mod component;
mod extension;
mod struct_fields;
mod variant;

use proc_macro::TokenStream;

use crate::component::parse_component;
use crate::extension::parse_extensions;
use crate::variant::parse_variant;

/// See the crate docs.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_component(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// See the crate docs.
#[proc_macro_attribute]
pub fn extensions(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_extensions(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// See the crate docs.
#[proc_macro_attribute]
pub fn variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_variant(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Stub for `#[struct_fields(...)]` when placed above `#[component(...)]`.
#[proc_macro_attribute]
pub fn struct_fields(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    struct_fields::stub().into()
}
