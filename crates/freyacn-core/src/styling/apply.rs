//! Shared `Style` → primitive helpers.
//!
//! Two entry points, one for each primitive family:
//!
//! * [`apply_to_rect`] — everything a `Rect` understands: layout, paint,
//!   **box shadows**, border, effects, cursor, plus the inheritable text
//!   slice (font families, text shadows) that Freya's `Rect` forwards to
//!   any text content inside it.
//! * [`apply_text_style`] — the text-only slice for `Label` / `Paragraph`.
//!   Colour, `font_*`, `text_*`, letter-spacing, text-decoration, and the
//!   `font_family` / `font_feature` / **text_shadow** adders.
//!
//! # Shadows
//!
//! `Style` carries two shadow vectors and they route differently:
//!
//! * `Style::shadows: Vec<Shadow>` — **box shadows**. Rendered by
//!   `apply_to_rect` via `EffectExt::shadow(..)`, one call per layer.
//!   Text primitives (`Label` / `Paragraph`) don't render box shadows,
//!   so a `.shadow_md()` on `text(..)` is silently dropped — wrap in a
//!   `div()` if you need one.
//! * `Style::text_shadows: Vec<TextShadow>` — **glyph shadows**. Routed
//!   through [`HasFontLists::add_text_shadow`] in [`apply_font_lists`],
//!   which both `Rect` and the text primitives implement. A `.text_shadow_lg()`
//!   on a `Div` cascades to descendants through `InheritedStyle` and gets
//!   painted on the descendants' text primitives, not on the `Div` itself.

use crate::styling::Style;
use freya::prelude::Paragraph as ParagraphPrimitive;
use freya::prelude::*;
use freya_core::elements::rect::Rect;
use std::borrow::Cow;

// ─────────────────────────────────────────────────────────────────────
// Public entry points
// ─────────────────────────────────────────────────────────────────────

/// Applies every `Style` field that `Rect` understands: layout, paint,
/// effects, **box shadows**, border, inheritable text props, cursor.
///
/// Text shadows in `s.text_shadows` are also forwarded — `Rect`
/// implements `HasFontLists`, so the glyph shadows ride along and paint
/// behind any text content the `Rect` renders. On a pure container
/// (no text), the text-shadow field is inert.
pub fn apply_to_rect(el: Rect, s: &Style) -> Rect {
    let mut el = el
        // layout
        .width(s.width.clone())
        .height(s.height.clone())
        .min_width(s.min_width.clone())
        .min_height(s.min_height.clone())
        .max_width(s.max_width.clone())
        .max_height(s.max_height.clone())
        .padding(s.padding)
        .margin(s.margin)
        .direction(s.direction)
        .main_align(s.main_align.clone())
        .cross_align(s.cross_align.clone())
        .position(s.position.clone())
        .content(s.content.clone())
        .spacing(s.spacing.get())
        .offset_x(s.offset_x.get())
        .offset_y(s.offset_y.get())
        // paint / effects
        .background(s.background.clone())
        .corner_radius(s.radius)
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotate(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        .layer(s.layer);

    // border — `Rect` implements `DecorationExt`. `Label` and `Paragraph`
    // do not, so `text()` / `p()` ignore borders (wrap in a `div()` if
    // you need one).
    if let Some(border) = &s.border {
        el = el.border(border.clone());
    }

    // box shadows — one call per layer, matching Freya's single-shadow
    // `EffectExt::shadow(..)` builder. The size presets on `StyledExt`
    // write multiple layers (`shadow_sm` is two), so iterating here is
    // what produces the correct CSS-equivalent stack.
    for shadow in &s.shadows {
        el = el.shadow(shadow.clone());
    }

    // inheritable text style — shared with `text.rs` / `paragraph.rs`.
    // Also forwards `s.text_shadows` to the `Rect`'s text layer (see
    // the module-level `# Shadows` section).
    el = apply_text_style(el, s);

    if s.cursor != CursorIcon::default() {
        el = el.cursor(s.cursor);
    }
    if !s.cursor_visible {
        el = el.cursor_visible(false);
    }
    el
}

/// Applies the inheritable text-style slice (color, `font_*`, `text_*`,
/// letter-spacing, text-decoration) plus the `font_family` /
/// `font_feature` / **`text_shadow`** adders.
///
/// Works on any primitive that implements both [`TextStyleExt`] and
/// [`HasFontLists`] — currently `Rect`, `Label`, and `Paragraph` (the
/// primitive).
pub fn apply_text_style<E>(el: E, s: &Style) -> E
where
    E: TextStyleExt + HasFontLists + Sized,
{
    let el = el
        .color(s.text_color.clone())
        .font_size(s.font_size)
        .font_weight(s.font_weight)
        .font_slant(s.font_style)
        .font_width(s.font_stretch)
        .letter_spacing(s.letter_spacing)
        .text_align(s.text_align)
        .text_overflow(s.text_overflow.clone())
        .text_decoration(s.text_decoration);
    apply_font_lists(el, s)
}

// ─────────────────────────────────────────────────────────────────────
// Trait + generic helper for the `font_*` / `text_shadow` adders
// ─────────────────────────────────────────────────────────────────────

pub trait HasFontLists: Sized {
    fn add_font_family(self, family: Cow<'static, str>) -> Self;
    fn add_font_feature(self, feature: FontFeature) -> Self;
    fn add_text_shadow(self, shadow: TextShadow) -> Self;
}

impl HasFontLists for Rect {
    fn add_font_family(self, family: Cow<'static, str>) -> Self {
        self.font_family(family)
    }
    fn add_font_feature(self, feature: FontFeature) -> Self {
        self.font_feature(feature)
    }
    fn add_text_shadow(self, shadow: TextShadow) -> Self {
        self.text_shadow(shadow)
    }
}

impl HasFontLists for Label {
    fn add_font_family(self, family: Cow<'static, str>) -> Self {
        self.font_family(family)
    }
    fn add_font_feature(self, feature: FontFeature) -> Self {
        self.font_feature(feature)
    }
    fn add_text_shadow(self, shadow: TextShadow) -> Self {
        self.text_shadow(shadow)
    }
}

impl HasFontLists for ParagraphPrimitive {
    fn add_font_family(self, family: Cow<'static, str>) -> Self {
        self.font_family(family)
    }
    fn add_font_feature(self, feature: FontFeature) -> Self {
        self.font_feature(feature)
    }
    fn add_text_shadow(self, shadow: TextShadow) -> Self {
        self.text_shadow(shadow)
    }
}

/// Applies the `font_family` / `font_feature` / `text_shadow` loops.
///
/// All three accumulate — every entry in the vector produces one call on
/// the primitive. That's what makes `shadow_md()` (three box-shadow
/// layers on the `StyledExt` side) and `text_shadow_lg()` (four text
/// shadow layers) render as CSS-equivalent stacks rather than a single
/// flattened shadow.
pub fn apply_font_lists<E: HasFontLists>(mut el: E, s: &Style) -> E {
    for family in &s.font_families {
        el = el.add_font_family(family.clone());
    }
    for feature in &s.font_features {
        el = el.add_font_feature(feature.clone());
    }
    for shadow in &s.text_shadows {
        el = el.add_text_shadow(shadow.clone());
    }
    el
}
