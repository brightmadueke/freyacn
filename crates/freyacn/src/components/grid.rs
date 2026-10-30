//! # Grid — fixed-column layout primitives
//!
//! Freya 0.4.3's layout engine (torin) does not expose a CSS Grid
//! primitive. `Grid` approximates one on top of `flex-wrap`:
//!
//!   1. `Grid` publishes its column count through context.
//!   2. Every `GridItem` reads that count and computes its own width
//!      as `(100 / cols) * span %`.
//!   3. `flex-wrap` on the container breaks to a new row once the
//!      running total exceeds 100%.
//!
//! ## Usage
//!
//! ```ignore
//! Grid()
//!     .cols(3)
//!     .gap_4()
//!     .child(GridItem().child(Label().text("One")))
//!     .child(GridItem().child(Label().text("Two")))
//!     .child(GridItem().child(Label().text("Three")))
//!     .child(GridItem().span(2).child(Label().text("Wide")))
//!     .child(GridItem().child(Label().text("Normal")));
//! ```
//!
//! ## Caveats
//!
//! - **Every child should be a `GridItem`.** Plain children get
//!   `Size::auto()` and are content-sized — they don't participate in
//!   the column arithmetic.
//! - **Spans are honoured per item.** A `span(2)` in a 3-column grid
//!   takes `2/3` of the row width. If a span overflows the remaining
//!   space in the current row, `flex-wrap` places it on the next row.
//! - **Row spans are stored but not enforced.** torin has no row-track
//!   primitive; `row_span` is captured on `Style` for future use but
//!   doesn't affect layout today.
//! - **Gaps come from `spacing`.** `gap-4` on the container produces
//!   both horizontal and vertical gaps. Per-axis gaps
//!   (`gap-x-*` / `gap-y-*`) are stored but torin only reads one
//!   spacing value.

use crate::prelude::*;

/// Column count published by `Grid` for its descendants.
///
/// `GridItem` reads this to size itself. Shared as a `State` so
/// changing `cols` on the parent re-renders the items with the new
/// width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridColumns(pub i32);

impl Default for GridColumns {
    fn default() -> Self {
        Self(2)
    }
}

#[component(
    props(
        cols: i32 = 2,
    ),
    ext(style, children, key, event_handlers, accessibility),
)]
#[derive(PartialEq, Clone)]
pub fn Grid() {
    let cols = (*cols).max(1);

    // Publish the column count for any `GridItem` in the subtree.
    // `use_hook` runs once on mount; the `set` afterwards keeps the
    // value in sync when the caller changes `cols` between renders.
    let mut cols_state = use_hook(|| {
        let state = State::create(GridColumns(cols));
        provide_context(state);
        state
    });
    cols_state.set(GridColumns(cols));

    // ── State folding ──────────────────────────────────────────────
    let theme = use_cn_theme();
    let mut active = StateFlags::NONE;
    if theme.read().is_dark {
        active = active.or(StateFlags::DARK);
    }

    let style = style.clone().with_active_states(active);

    // ── Container ──────────────────────────────────────────────────
    let mut container = rect()
        .content(Content::Wrap { wrap_spacing: None })
        .direction(Direction::Horizontal)
        .spacing(style.spacing.unwrap_or(0.0))
        .background(style.background.clone().unwrap_or_default())
        .corner_radius(style.corner_radius.unwrap_or_default())
        .border(style.to_border())
        .padding(style.padding.unwrap_or_default())
        .margin(style.margin.unwrap_or_default())
        .opacity(style.opacity.unwrap_or(1.0))
        .interactive(style.pointer_events.unwrap_or(true));

    if let Some(w) = style.width.clone() {
        container = container.width(w);
    }
    if let Some(h) = style.height.clone() {
        container = container.height(h);
    }
    if let Some(w) = style.min_width.clone() {
        container = container.min_width(w);
    }
    if let Some(h) = style.min_height.clone() {
        container = container.min_height(h);
    }
    if let Some(w) = style.max_width.clone() {
        container = container.max_width(w);
    }
    if let Some(h) = style.max_height.clone() {
        container = container.max_height(h);
    }

    // Inheritable text stylesheet
    if let Some(c) = style.text_color {
        container = container.color(c);
    }
    if let Some(s) = style.font_size {
        container = container.font_size(s);
    }
    if let Some(a) = style.text_align {
        container = container.text_align(a);
    }

    // Scrolling: if the grid is marked scrollable, apply the flag to
    // the same rect.
    if style.scroll_axis.is_some() {
        container = container.scrollable(true);
    }

    // ── Children ───────────────────────────────────────────────────
    // `GridItem` sizes itself from the published column count. Plain
    // children are dropped in as-is and size to their content.
    for child in children.iter() {
        container = container.child(child.clone());
    }

    container
        .accessibility(accessibility.clone())
        .event_handlers(event_handlers.clone())
        .into_element()
}

#[component(
    props(
        span: i32 = 1,
        row_span: i32 = 1,
    ),
    ext(style, children, key, accessibility),
)]
#[derive(PartialEq, Clone)]
pub fn GridItem() {
    // Read the parent grid's column count. Falls back to 2 columns if
    // the item is somehow rendered outside a `Grid`.
    let cols = try_consume_context::<State<GridColumns>>()
        .map(|s| s.read().0)
        .unwrap_or(2)
        .max(1);

    let span = (*span).max(1).min(cols);
    let width_pct = (100.0 / cols as f32) * span as f32;

    // ── State folding ──────────────────────────────────────────────
    let theme = use_cn_theme();
    let mut active = StateFlags::NONE;
    if theme.read().is_dark {
        active = active.or(StateFlags::DARK);
    }

    let style = style.clone().with_active_states(active);

    let mut cell = rect()
        .width(Size::percent(width_pct))
        .content(Content::Fit)
        .background(style.background.clone().unwrap_or_default())
        .corner_radius(style.corner_radius.unwrap_or_default())
        .border(style.to_border())
        .padding(style.padding.unwrap_or_default())
        .margin(style.margin.unwrap_or_default())
        .opacity(style.opacity.unwrap_or(1.0))
        .interactive(style.pointer_events.unwrap_or(true));

    if let Some(c) = style.text_color {
        cell = cell.color(c);
    }
    if let Some(s) = style.font_size {
        cell = cell.font_size(s);
    }
    if let Some(a) = style.text_align {
        cell = cell.text_align(a);
    }

    for child in children.iter() {
        cell = cell.child(child.clone());
    }

    // `row_span` is captured on `Style` for future engine support —
    // torin has no row-track primitive today, so the cell wraps
    // naturally after the span width is applied.
    let _ = *row_span;

    cell.accessibility(accessibility.clone()).into_element()
}
