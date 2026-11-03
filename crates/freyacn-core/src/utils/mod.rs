//! Cross-cutting helpers that don't belong to any single module.
//!
//! Keep this file small. Every function here must solve a problem that
//! shows up in **at least two** modules — otherwise it belongs next to
//! the one thing that uses it. When in doubt, don't add it.
//!
//! # What's here
//!
//! - [`ColorExt`] — `Color`-level math (`with_alpha`, `multiply_alpha`,
//!   `blend_toward`, `luminance`, `contrast_ratio`, `is_dark`). The
//!   `StyledColor` API is still the primary color interface; these are
//!   for callers who already have a plain `Color` in hand.
//! - [`InteractionState`] / [`pick_component_state`] — collapse the
//!   `hovered` / `pressed` / `focused` / … booleans into the single
//!   `ComponentState` a theme lookup should use.
//! - [`parse_hex_rgba`] — the one hex grammar.
//!
//! # What's not here
//!
//! - Anything already covered by `StyledColor` (alpha, opacity).
//! - `Style` mutations — those live on `StyledExt`.
//! - Theme lookups — those live on `Theme` / `ThemedExt`.

pub mod color;
pub mod component;
pub mod hex;

pub use color::ColorExt;
pub use component::{InteractionState, pick_component_state};
pub use hex::parse_hex_rgba;
