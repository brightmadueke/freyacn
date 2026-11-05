//! shadcn Card — container with header, content, and footer sections.
//!
//! Mirrors shadcn's `Card` family: `Card` / `CardHeader` / `CardTitle` /
//! `CardDescription` / `CardAction` / `CardContent` / `CardFooter`.
//!
//! ## Example
//!
//! ```rust,ignore
//! Card()
//!     .child(
//!         CardHeader()
//!             .child(CardTitle().child(text("Title")))
//!             .child(CardDescription().child(text("A short description.")))
//!     )
//!     .child(CardContent().child(text("Body content.")))
//!     .child(
//!         CardFooter()
//!             .child(Button().child(text("Action")))
//!     )
//! ```
//!
//! ## Class mapping
//!
//! | Component | shadcn base class string |
//! |---|---|
//! | `Card` | `flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm` |
//! | `CardHeader` | `flex flex-col gap-2 px-6` (simplified from `@container/card-header grid …`) |
//! | `CardTitle` | `leading-none font-semibold` |
//! | `CardDescription` | `text-sm text-muted-foreground` |
//! | `CardAction` | *(no base — see "CardAction" below)* |
//! | `CardContent` | `px-6` |
//! | `CardFooter` | `flex-row items-center px-6` |
//!
//! ## CardAction
//!
//! shadcn places `CardAction` in the header's top-right via CSS grid:
//! `col-start-2 row-span-2 row-start-1 self-start justify-self-end`. Freya's
//! layout engine is flexbox-only — there's no grid, no per-child cross-axis
//! alignment, and no `justify-self` — so automatic placement isn't
//! expressible. Use `CardAction` as a wrapper for the action's own styling,
//! and position it by either:
//!
//! - Overriding `CardHeader` to `flex-row justify-between items-start` and
//!   wrapping `CardTitle` + `CardDescription` in their own `flex-col`.
//! - Or placing the action outside `CardHeader` in its own row.

use freyacn_core::prelude::{ChildrenExt, StyledExt, div};
use freyacn_macros::component;

// ── Card ────────────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("flex-col gap-6 rounded-xl border border-border bg-card py-6 text-card-foreground shadow-sm"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn Card() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardHeader ──────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("flex-col items-start gap-2 px-6"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardHeader() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardTitle ───────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("leading-none font-semibold"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardTitle() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardDescription ─────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("text-sm text-muted-foreground"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardDescription() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardAction ──────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardAction() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardContent ─────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("px-6"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardContent() {
    div().merge_style(style.clone()).children(children.clone())
}

// ── CardFooter ──────────────────────────────────────────────────────

#[component(
    crate = freyacn_core,
    variants("flex-row items-center px-6"),
    ext(styled, children)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn CardFooter() {
    div().merge_style(style.clone()).children(children.clone())
}
