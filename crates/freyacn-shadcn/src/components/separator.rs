//! shadcn Separator — a visual or semantic divider.
//!
//! Mirrors shadcn's `Separator` component:
//!
//! - `orientation` — `Horizontal` (default) or `Vertical`.
//! - `decorative` — `true` (default) means purely visual, hidden from
//!   assistive technology. `false` exposes it as a semantic separator.
//!
//! Horizontal separators are 1px tall and fill the container width;
//! vertical separators are 1px wide and fill the container height.
//!
//! ## Example
//!
//! ```rust,ignore
//! // Horizontal (default) — visual divider between sections.
//! Separator()
//!
//! // Vertical — between inline items.
//! div()
//!     .style("flex-row items-center gap-2")
//!     .child(text("Left"))
//!     .child(Separator().orientation(SeparatorOrientation::Vertical))
//!     .child(text("Right"))
//!
//! // Semantic (announced by screen readers).
//! Separator().decorative(false)
//! ```
//!
//! ## Class mapping
//!
//! shadcn's base class string is `"shrink-0 bg-border"`. FreyaCN's
//! `Style` has no flex-shrink field, so `shrink-0` is dropped — the
//! observable effect is identical when the separator is used inside
//! flex layouts, since it's not given a `flex` dimension that could
//! shrink.
//!
//! The orientation variants contribute:
//!
//! - Horizontal: `h-px w-full` — matches shadcn's
//!   `data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full`.
//! - Vertical: `w-px h-full` — matches shadcn's
//!   `data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px`.

use freya::prelude::AccessibilityRole;
use freyacn_core::prelude::{AccessibilityExt, StyledExt, div};
use freyacn_macros::{component, variant};

/// Axis the separator runs along.
///
/// Maps directly to shadcn's `orientation` prop. `Horizontal` is a 1px
/// tall row spanning the full width; `Vertical` is a 1px wide column
/// spanning the full height.
#[variant(freyacn_core)]
#[derive(Default, PartialEq, Clone)]
pub enum SeparatorOrientation {
    /// 1px tall, full width. Default.
    #[default]
    #[variant("h-px w-full")]
    Horizontal,

    /// 1px wide, full height.
    #[variant("w-px h-full")]
    Vertical,
}

/// A horizontal or vertical divider, styled like shadcn's `Separator`.
///
/// ## Accessibility
///
/// - `decorative = true` (default) — no role, no announcement. The
///   element is purely visual, matching `<hr aria-hidden>`.
/// - `decorative = false` — exposes `Role::Splitter` (the closest
///   available accesskit role to WAI-ARIA's `separator`). Screen readers
///   announce the element as a structural boundary.
///
/// If your Freya build exposes `AccessibilityRole::Separator`, prefer
/// that in place of `Splitter` — see the inline comment in the render
/// body.
#[component(
    crate = freyacn_core,
    variants(
        "bg-border",
        orientation: SeparatorOrientation = SeparatorOrientation::Horizontal,
    ),
    props(
        decorative: bool = true,
    ),
    ext(styled, accessibility)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn Separator() {
    let mut el = div()
        .merge_style(style.clone())
        .accessibility(accessibility.clone());

    // Non-decorative separators are exposed to assistive technology as
    // structural boundaries. `Splitter` is the closest accesskit role
    // to WAI-ARIA's `separator` — see the struct doc's
    // "Accessibility" section for the rationale.
    //
    // Decorative separators set nothing here, matching `<hr aria-hidden>`:
    // no role, no announcement, purely visual.
    //
    // If your Freya version exposes `AccessibilityRole::Separator`,
    // swap `Splitter` for it — it's the WAI-ARIA-correct role for a
    // static separator, whereas `Splitter` is technically for
    // resizable splits.
    if !*decorative {
        el = el.a11y_role(AccessibilityRole::Splitter);
    }

    el
}
