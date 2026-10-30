use crate::styling::colors::StyledColor;
use crate::styling::style::Style;
use freya_core::prelude::{
    FontFeature, FontSize, FontSlant, FontWeight, FontWidth, LetterSpacing, TextAlign, TextOverflow,
};
use std::borrow::Cow;

/// Fluent styling extensions for FreyaCN components.
///
/// Every method consumes and returns `Self`, so calls can be chained:
///
/// ```ignore
/// text("Hello")
///     .text_color(neutral_900())
///     .text_lg()
///     .font_semibold()
///     .text_center()
/// ```
///
/// The trait is organized into the following groups:
///
/// - **Colors** — [`bg`](Self::bg), [`text_color`](Self::text_color)
/// - **Text alignment** — [`text_align`](Self::text_align) and its shortcuts
/// - **Text overflow** — [`text_overflow`](Self::text_overflow) and its shortcuts
/// - **Font size** — [`font_size`](Self::font_size) and the `text_*` shortcuts
/// - **Font weight** — [`font_weight`](Self::font_weight) and the `font_*` shortcuts
/// - **Font style (slant)** — [`font_style`](Self::font_style) and its shortcuts
/// - **Font stretch (width)** — [`font_stretch`](Self::font_stretch) and its shortcuts
/// - **Letter spacing** — [`tracking`](Self::tracking) and its shortcuts
/// - **OpenType features** — [`font_feature`](Self::font_feature) and its shortcuts
/// - **Font families** — [`font_family`](Self::font_family),
///   [`font_families`](Self::font_families), and the `font_sans`/`font_serif`/
///   `font_mono` stacks
pub trait StyledExt: Sized {
    /// Returns a mutable reference to the underlying [`Style`].
    ///
    /// This is the single accessor every other method builds on.
    fn get_style(&mut self) -> &mut Style;

    // ---- colors ----

    /// Sets the background fill.
    ///
    /// Accepts any [`StyledColor`] — solid colors, gradients, shaders, etc.
    /// A solid color's `.opacity(..)` is applied on conversion.
    fn bg(mut self, color: StyledColor) -> Self {
        self.get_style().background = color.into();
        self
    }

    /// Sets the text (foreground) color.
    ///
    /// Accepts any [`StyledColor`] — solid colors, gradients, shaders, etc.
    fn text_color(mut self, color: StyledColor) -> Self {
        self.get_style().text_color = color.into();
        self
    }

    // ---- text alignment ----

    /// Sets the horizontal text alignment.
    ///
    /// Accepts anything convertible into [`TextAlign`]; the shortcut methods
    /// below (`text_left`, `text_center`, …) all delegate to this.
    fn text_align(mut self, text_align: impl Into<TextAlign>) -> Self {
        self.get_style().text_align = text_align.into();
        self
    }

    /// Align text to the left edge. This is the default.
    fn text_left(self) -> Self {
        self.text_align(TextAlign::Left)
    }

    /// Align text to the right edge.
    fn text_right(self) -> Self {
        self.text_align(TextAlign::Right)
    }

    /// Center the text horizontally.
    fn text_center(self) -> Self {
        self.text_align(TextAlign::Center)
    }

    /// Stretch each line to fill the width, except the last line.
    fn text_justify(self) -> Self {
        self.text_align(TextAlign::Justify)
    }

    /// Align text to the end edge, following the text direction.
    fn text_end(self) -> Self {
        self.text_align(TextAlign::End)
    }

    /// Align text to the start edge, following the text direction.
    fn text_start(self) -> Self {
        self.text_align(TextAlign::Start)
    }

    // ---- text overflow ----

    /// Sets how overflowing text is rendered.
    ///
    /// Accepts anything convertible into [`TextOverflow`]; the shortcut
    /// methods below (`text_clip`, `text_ellipsis`, `text_custom`) all
    /// delegate to this.
    fn text_overflow(mut self, overflow: TextOverflow) -> Self {
        self.get_style().text_overflow = overflow.into();
        self
    }

    /// Cut the text off at the edge. This is the default.
    fn text_clip(self) -> Self {
        self.text_overflow(TextOverflow::Clip)
    }

    /// Replace the cut-off text with an ellipsis (...).
    fn text_ellipsis(self) -> Self {
        self.text_overflow(TextOverflow::Ellipsis)
    }

    /// Replace the cut-off text with a custom string.
    ///
    /// Shorthand for [`text_overflow`](Self::text_overflow) with
    /// [`TextOverflow::Custom`].
    fn text_custom(self, custom: impl Into<String>) -> Self {
        self.text_overflow(TextOverflow::Custom(custom.into()))
    }

    // ---- font size ----

    /// Sets the font size.
    ///
    /// Accepts anything convertible into [`FontSize`] (e.g. `16.0`).
    /// The `text_*` shortcuts below all delegate to this method.
    fn font_size(mut self, font_size: impl Into<FontSize>) -> Self {
        self.get_style().font_size = font_size.into();
        self
    }

    /// Extra-extra-small text — `12px`.
    fn text_xs(self) -> Self {
        self.font_size(12.0)
    }
    /// Small text — `14px`.
    fn text_sm(self) -> Self {
        self.font_size(14.0)
    }
    /// Base text — `16px`.
    fn text_md(self) -> Self {
        self.font_size(16.0)
    }
    /// Large text — `18px`.
    fn text_lg(self) -> Self {
        self.font_size(18.0)
    }
    /// Extra-large text — `20px`.
    fn text_xl(self) -> Self {
        self.font_size(20.0)
    }
    /// 2× extra-large text — `24px`.
    fn text_2xl(self) -> Self {
        self.font_size(24.0)
    }
    /// 3× extra-large text — `30px`.
    fn text_3xl(self) -> Self {
        self.font_size(30.0)
    }
    /// 4× extra-large text — `36px`.
    fn text_4xl(self) -> Self {
        self.font_size(36.0)
    }
    /// 5× extra-large text — `48px`.
    fn text_5xl(self) -> Self {
        self.font_size(48.0)
    }
    /// 6× extra-large text — `60px`.
    fn text_6xl(self) -> Self {
        self.font_size(60.0)
    }
    /// 7× extra-large text — `72px`.
    fn text_7xl(self) -> Self {
        self.font_size(72.0)
    }
    /// 8× extra-large text — `96px`.
    fn text_8xl(self) -> Self {
        self.font_size(96.0)
    }
    /// 9× extra-large text — `128px`.
    fn text_9xl(self) -> Self {
        self.font_size(128.0)
    }

    // ---- font weight ----

    /// Sets the font weight.
    ///
    /// Accepts anything convertible into [`FontWeight`]; the `font_*`
    /// shortcuts below all delegate to this method.
    fn font_weight(mut self, font_weight: impl Into<FontWeight>) -> Self {
        self.get_style().font_weight = font_weight.into();
        self
    }

    /// Hairline weight (`100`).
    fn font_thin(self) -> Self {
        self.font_weight(FontWeight::THIN)
    }
    /// Extra-light weight (`200`).
    fn font_extralight(self) -> Self {
        self.font_weight(FontWeight::EXTRA_LIGHT)
    }
    /// Light weight (`300`).
    fn font_light(self) -> Self {
        self.font_weight(FontWeight::LIGHT)
    }
    /// Regular weight (`400`). This is the default.
    fn font_normal(self) -> Self {
        self.font_weight(FontWeight::NORMAL)
    }
    /// Medium weight (`500`).
    fn font_medium(self) -> Self {
        self.font_weight(FontWeight::MEDIUM)
    }
    /// Semi-bold weight (`600`).
    fn font_semibold(self) -> Self {
        self.font_weight(FontWeight::SEMI_BOLD)
    }
    /// Bold weight (`700`).
    fn font_bold(self) -> Self {
        self.font_weight(FontWeight::BOLD)
    }
    /// Extra-bold weight (`800`).
    fn font_extrabold(self) -> Self {
        self.font_weight(FontWeight::EXTRA_BOLD)
    }
    /// Black/heavy weight (`900`).
    fn font_black(self) -> Self {
        self.font_weight(FontWeight::BLACK)
    }

    // ---- letter spacing ----

    /// Sets letter spacing, expressed in `em` (a multiple of the current font size).
    ///
    /// Because the value is relative to the font size, the resulting pixel
    /// spacing scales automatically with `.font_size(..)` / `.text_lg()` etc.
    /// Call this **after** the font-size setter to get the expected result:
    ///
    /// ```ignore
    /// text("Hi").text_lg().tracking(0.05) // 18px * 0.05 = 0.9px
    /// ```
    ///
    /// Does nothing meaningful if the font size resolves to `0.0`.
    /// The `tracking_*` shortcuts below all delegate to this method.
    fn tracking(mut self, em: impl Into<LetterSpacing>) -> Self {
        let font_size: f32 = self.get_style().font_size.into();
        let letter_spacing: f32 = em.into().into();
        let result = font_size * letter_spacing;

        self.get_style().letter_spacing = result.into();
        self
    }

    /// Tighter tracking — `-0.05em`.
    fn tracking_tighter(self) -> Self {
        self.tracking(-0.05)
    }

    /// Tighten tracking slightly — `-0.025em`.
    fn tracking_tighten(self) -> Self {
        self.tracking(-0.025)
    }

    /// Normal tracking — `0em`.
    fn tracking_normal(self) -> Self {
        self.tracking(0.)
    }

    /// Widen tracking slightly — `0.025em`.
    fn tracking_widen(self) -> Self {
        self.tracking(0.025)
    }

    /// Wider tracking — `0.05em`.
    fn tracking_wider(self) -> Self {
        self.tracking(0.05)
    }

    /// Widest tracking — `0.1em`.
    fn tracking_widest(self) -> Self {
        self.tracking(0.1)
    }

    // ---- OpenType font features ----

    /// Add an OpenType font feature.
    ///
    /// Accepts anything convertible into [`FontFeature`]. Features
    /// **accumulate**, so calling this multiple times (or using the
    /// `font_*` shortcuts below) stacks them:
    ///
    /// ```ignore
    /// text("0x1234")
    ///     .font_feature(FontFeature::TABULAR_NUMBERS)
    ///     .font_feature(FontFeature::SLASHED_ZERO)
    /// ```
    ///
    /// The shortcuts below delegate to this method. Use the raw method
    /// when you need a feature that has no dedicated shortcut.
    fn font_feature(mut self, font_feature: impl Into<FontFeature>) -> Self {
        self.get_style().font_features.push(font_feature.into());
        self
    }

    /// Every digit shares the same width, so changing numbers do not shift the layout.
    fn font_tabular(self) -> Self {
        self.font_feature(FontFeature::TABULAR_NUMBERS)
    }

    /// Lowercase letters render as small capitals.
    fn font_small_caps(self) -> Self {
        self.font_feature(FontFeature::SMALL_CAPS)
    }

    /// Zero renders with a slash or dot to tell it apart from the letter O.
    fn font_slashed_zero(self) -> Self {
        self.font_feature(FontFeature::SLASHED_ZERO)
    }

    /// Sequences like `1/2` render as fractions.
    fn font_fractions(self) -> Self {
        self.font_feature(FontFeature::FRACTIONS)
    }

    /// Disable standard ligatures like `fi` or the joined `=>` of coding fonts.
    fn font_no_ligatures(self) -> Self {
        self.font_feature(FontFeature::NO_LIGATURES)
    }

    // ---- font families ----

    /// Append a font family to try, in order of preference.
    ///
    /// The renderer uses the first family that has the required glyphs, so
    /// call this multiple times (or use [`Self::font_families`]) to build a
    /// fallback stack:
    ///
    /// ```ignore
    /// text("Hi").font_family("Inter").font_family("system-ui")
    /// ```
    ///
    /// This method **appends**. To replace the whole stack, call
    /// [`Self::clear_font_families`] first — or use [`Self::font_sans`],
    /// [`Self::font_serif`], or [`Self::font_mono`], which reset the stack.
    fn font_family(mut self, font_family: impl Into<Cow<'static, str>>) -> Self {
        self.get_style().font_families.push(font_family.into());
        self
    }

    /// Append several font families at once, in order of preference.
    ///
    /// Accepts any iterable of `&'static str` / `String` / `Cow<'static, str>`
    /// — arrays, `Vec`s, iterators, etc.:
    ///
    /// ```ignore
    /// text("Hi").font_families(["Inter", "system-ui", "sans-serif"]);
    /// text("Hi").font_families(vec![font_family.clone(), "sans-serif"]);
    /// ```
    fn font_families<I, S>(mut self, font_families: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'static, str>>,
    {
        self.get_style()
            .font_families
            .extend(font_families.into_iter().map(Into::into));
        self
    }

    /// Remove every font family previously set on this style.
    fn clear_font_families(mut self) -> Self {
        self.get_style().font_families.clear();
        self
    }

    /// Replace the font stack with the default sans-serif stack.
    ///
    /// Matches Tailwind's `font-sans`:
    /// `ui-sans-serif, system-ui, sans-serif, …emoji fallbacks`.
    fn font_sans(self) -> Self {
        self.clear_font_families().font_families([
            "ui-sans-serif",
            "system-ui",
            "sans-serif",
            "Apple Color Emoji",
            "Segoe UI Emoji",
            "Segoe UI Symbol",
            "Noto Color Emoji",
        ])
    }

    /// Replace the font stack with the default serif stack.
    ///
    /// Matches Tailwind's `font-serif`:
    /// `ui-serif, Georgia, Cambria, "Times New Roman", Times, serif`.
    fn font_serif(self) -> Self {
        self.clear_font_families().font_families([
            "ui-serif",
            "Georgia",
            "Cambria",
            "Times New Roman",
            "Times",
            "serif",
        ])
    }

    /// Replace the font stack with the default monospace stack.
    ///
    /// Matches Tailwind's `font-mono`:
    /// `ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, …`.
    fn font_mono(self) -> Self {
        self.clear_font_families().font_families([
            "ui-monospace",
            "SFMono-Regular",
            "Menlo",
            "Monaco",
            "Consolas",
            "Liberation Mono",
            "Courier New",
            "monospace",
        ])
    }

    // ---- font style (slant) ----

    /// Sets the font slant.
    ///
    /// Accepts anything convertible into [`FontSlant`]; the shortcut
    /// methods below (`italic`, `non_italic`, `oblique`) all delegate
    /// to this.
    ///
    /// - [`FontSlant::Upright`] — normal, upright text (**default**).
    /// - [`FontSlant::Italic`] — true italic glyphs when the family has
    ///   them, otherwise synthesized by slanting the upright face.
    /// - [`FontSlant::Oblique`] — always synthesized by slanting the
    ///   upright face (no dedicated glyphs used). Not a Tailwind class,
    ///   but available through Freya.
    fn font_style(mut self, font_style: impl Into<FontSlant>) -> Self {
        self.get_style().font_style = font_style.into();
        self
    }

    /// Italic text — Tailwind's `italic` (`font-style: italic`).
    fn italic(self) -> Self {
        self.font_style(FontSlant::Italic)
    }

    /// Upright text — Tailwind's `not-italic` (`font-style: normal`).
    /// This is the default.
    fn non_italic(self) -> Self {
        self.font_style(FontSlant::Upright)
    }

    /// Oblique text — slanted upright glyphs, without using italic
    /// faces. No Tailwind equivalent; exposed because Freya supports it.
    fn oblique(self) -> Self {
        self.font_style(FontSlant::Oblique)
    }

    // ---- font stretch (width) ----

    /// Sets the font stretch (width) axis.
    ///
    /// Accepts anything convertible into [`FontWidth`]; the
    /// `font_stretch_*` shortcuts below all delegate to this. Matching
    /// only works with variable fonts or a family that ships multiple
    /// widths — otherwise the renderer falls back to the nearest match
    /// or the upright face.
    fn font_stretch(mut self, font_width: impl Into<FontWidth>) -> Self {
        self.get_style().font_stretch = font_width.into();
        self
    }

    /// Ultra-condensed — `50%` width.
    fn font_stretch_ultra_condensed(self) -> Self {
        self.font_stretch(FontWidth::ULTRA_CONDENSED)
    }

    /// Extra-condensed — `62.5%` width.
    fn font_stretch_extra_condensed(self) -> Self {
        self.font_stretch(FontWidth::EXTRA_CONDENSED)
    }

    /// Condensed — `75%` width.
    fn font_stretch_condensed(self) -> Self {
        self.font_stretch(FontWidth::CONDENSED)
    }

    /// Semi-condensed — `87.5%` width.
    fn font_stretch_semi_condensed(self) -> Self {
        self.font_stretch(FontWidth::SEMI_CONDENSED)
    }

    /// Normal width — `100%`. This is the default.
    fn font_stretch_normal(self) -> Self {
        self.font_stretch(FontWidth::NORMAL)
    }

    /// Semi-expanded — `112.5%` width.
    fn font_stretch_semi_expanded(self) -> Self {
        self.font_stretch(FontWidth::SEMI_EXPANDED)
    }

    /// Extra-expanded — `125%` width.
    fn font_stretch_extra_expanded(self) -> Self {
        self.font_stretch(FontWidth::EXTRA_EXPANDED)
    }

    /// Ultra-expanded — `150%` width.
    fn font_stretch_ultra_expanded(self) -> Self {
        self.font_stretch(FontWidth::ULTRA_EXPANDED)
    }
}
