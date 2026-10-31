//! The [`Variant`] trait.
//!
//! Implemented automatically by the `#[variant]` proc-macro on enums. Used
//! by the component macro to merge variant-derived stylesheet into a component's
//! effective style before rendering.
//!
//! # Overview
//!
//! A type implementing [`Variant`] can be resolved to a [`Style`] using the
//! active FreyaCN theme. The `#[variant]` macro generates the impl from the
//! per-variant `#[variant("…")]` style strings on your enum.
//!
//! # Contract
//!
//! [`resolve`](Self::resolve) reads the active theme via `use_cn_theme()`,
//! so it must be called **inside a Freya render context**. The generated
//! implementation panics otherwise.
//!
//! # Example
//!
//! ```ignore
//! use freyacn::{variant, Variant};
//!
//! #[variant]
//! #[derive(Clone, Copy, PartialEq, Eq, Default)]
//! pub enum ButtonVariant {
//!     #[variant("bg-primary text-primary-foreground")]
//!     #[default]
//!     Default,
//!     #[variant("bg-destructive")]
//!     Destructive,
//! }
//!
//! let style = ButtonVariant::Destructive.resolve();
//! ```

use crate::styling::style::Style;

/// A type that resolves to a [`Style`].
///
/// Implemented by the `#[variant]` proc-macro on enums. You should not need
/// to implement it manually.
pub trait Variant {
    /// Resolve the current variant into a [`Style`].
    ///
    /// Must be called inside a Freya render context.
    fn resolve(&self) -> Style;
}
