//! Shared `Style` → primitive helpers.

use crate::styling::Style;
use freya::prelude::Paragraph as ParagraphPrimitive;
use freya::prelude::*;
use freya_core::elements::rect::Rect;
use std::borrow::Cow;

// ─────────────────────────────────────────────────────────────────────
// Public entry points
// ─────────────────────────────────────────────────────────────────────

/// Applies every `Style` field that `Rect` understands: layout, paint,
/// effects, border, inheritable text props, cursor.
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

    // inheritable text style — shared with `text.rs` / `paragraph.rs`
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
/// `font_feature` / `text_shadow` adders.
///
/// Works on any primitive that implements both [`TextStyleExt`] and
/// [`HasFontLists`] — currently `Label` and `Paragraph` (the primitive).
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
