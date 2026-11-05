use crate::base::CursorIcon;
use crate::styling::colors::{StyledColor, black};
use crate::styling::style::Style;
use crate::styling::style_variants::StyleVariant;
use crate::styling::styled_string::StyledString;
use crate::theme::{ComponentState, Theme, active_theme};
use freya_core::data::Overflow;
use freya_core::layers::Layer;
use freya_core::prelude::{
    AspectRatio, Border, BorderAlignment, BorderWidth, CornerRadius, FontFeature, FontSize,
    FontSlant, FontWeight, FontWidth, ImageCover, LetterSpacing, SamplingMode, Scale, Shadow,
    TextAlign, TextDecoration, TextOverflow, TextShadow, TransformOrigin,
};
use std::borrow::Cow;
use std::sync::Arc;
use torin::prelude::{Alignment, Direction, Gaps, Length, Position, Size};

/// Fluent styling extensions for FreyaCN components.
///
/// Every method consumes and returns `Self`, so calls can be chained:
///
/// ```
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
/// - **Line height** — [`line_height`](Self::line_height) and the `leading_*` shortcuts
/// - **Text shadow** — [`text_shadow`](Self::text_shadow) and the `text_shadow_*` shortcuts
/// - **Text decoration** — [`text_decoration`](Self::text_decoration) and the
///   `underline`/`overline`/`line_through`/`no_underline` shortcuts
/// - **Cursor** — [`cursor`](Self::cursor) and the `cursor_*` shortcuts
/// - **Overflow** — [`overflow`](Self::overflow) and the
///   `overflow_none`/`overflow_clip` shortcuts
/// - **Backdrop blur** — [`backdrop_blur`](Self::backdrop_blur) and the
///   `backdrop_blur_*` shortcuts
/// - **Rotation** — [`rotate`](Self::rotate) and the
///   `rotate_*` / `rotate_neg_*` shortcuts
/// - **Opacity** — [`opacity`](Self::opacity) and the `opacity_*` shortcuts
/// - **Scale** — [`scale`](Self::scale), [`scale_x`](Self::scale_x),
///   [`scale_y`](Self::scale_y), and the `scale_*` / `scale_neg_*` shortcuts
/// - **Radius** — [`radius`](Self::radius) and the `rounded_*` shortcuts
/// - **Padding** — [`p`](Self::p), [`px`](Self::px), … and the `p_*`/`px_*`/… shortcuts
///   (`*_p5` = Tailwind's `.5` half-step, e.g. `p_1_p5` = `p-1.5` = 6px)
/// - **Margin** — [`m`](Self::m), [`mx`](Self::mx), … and the `m_*`/`mx_*`/… shortcuts
/// - **Width / height** — the `w_*` and `h_*` shortcuts
///   (`*_p5` = `.5` half-step; `*_N_5` = `N/5` fraction, e.g. `w_2_5` = `w-2/5` = 40%)
/// - **Min / max dimensions** — the `min_w_*` / `max_w_*` / `min_h_*` / `max_h_*` shortcuts
/// - **Gap** — [`gap`](Self::gap) and the `gap_*` shortcuts
/// - **Variants** — [`hover`](Self::hover), [`focus`](Self::focus),
///   [`focus_visible`](Self::focus_visible), [`focus_within`](Self::focus_within),
///   [`active`](Self::active), [`disabled`](Self::disabled),
///   [`checked`](Self::checked), [`selected`](Self::selected)
pub trait StyledExt: Sized {
    /// Returns a mutable reference to the underlying [`Style`].
    ///
    /// This is the single accessor every other method builds on.
    fn get_style(&mut self) -> &mut Style;

    // ---- bulk style operations ----

    /// Merge `other` into `self`'s style, with `other`'s non-default
    /// fields taking precedence.
    ///
    /// Mirrors [`Style::merge_style`], but works on any [`StyledExt`]
    /// implementor (component builders, extension structs, `Style`
    /// itself). Fields that `other` leaves at their [`Style::bare`]
    /// value are left untouched, so `other` behaves as a *sparse patch*
    /// over the receiver:
    ///
    /// ```ignore
    /// // Whatever the user set on `card` is kept for fields the preset
    /// // leaves at default; the preset overrides the ones it sets.
    /// card.merge_style(preset)
    /// ```
    fn merge_style(mut self, other: Style) -> Self {
        let style = self.get_style();
        let current = style.clone();
        *style = current.merge_style(&other);
        self
    }

    /// Merge `self`'s style **on top of** `base`, with `self`'s
    /// non-default fields taking precedence.
    ///
    /// The inverse of [`merge_style`](Self::merge_style). Use it to apply
    /// a base / preset *beneath* what the receiver has already set — the
    /// receiver wins for every field it makes non-default:
    ///
    /// ```ignore
    /// // The card's own bg/padding/etc. win; the preset only fills
    /// // in fields the card left untouched.
    /// card.merge_style_under(theme_preset)
    /// ```
    fn merge_style_under(mut self, base: Style) -> Self {
        let style = self.get_style();
        let current = style.clone();
        *style = base.merge_style(&current);
        self
    }

    /// Replace `self`'s style with `style`, discarding everything
    /// previously set on the receiver.
    ///
    /// Unlike [`merge_style`](Self::merge_style), every field is
    /// overwritten — including fields `style` leaves at their bare
    /// defaults.
    ///
    /// ```ignore
    /// component.override_style(reset)
    /// ```
    fn override_style(mut self, style: Style) -> Self {
        *self.get_style() = style;
        self
    }

    // ---- colors ----

    /// Sets the background fill.
    ///
    /// Accepts anything convertible into [`StyledColor`] — `StyledColor`
    /// itself, [`Color`], [`Fill`], gradients (`LinearGradient`,
    /// `RadialGradient`, `ConicGradient`), shaders (`ShaderFill`), render
    /// callbacks (`RenderCallback`), and RGB/RGBA tuples. See
    /// [`StyledColor`] for the full list.
    ///
    /// A solid color's `.opacity(..)` is applied on conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// // StyledColor — the standard shape
    /// div().bg(blue_500())
    ///
    /// // Color — no `.into()` needed
    /// div().bg(Color::from_rgb(255, 0, 0))
    ///
    /// // RGB tuple
    /// div().bg((59, 130, 246))
    ///
    /// // Gradient
    /// div().bg(
    ///     LinearGradient::new()
    ///         .stop((blue_500().into(), 0.0))
    ///         .stop((purple_500().into(), 100.0)),
    /// )
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use the explicit type:
    ///
    /// ```
    /// div().bg(StyledColor::default())
    /// ```
    fn bg(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        self.get_style().background = sc.to_fill();
        self
    }

    /// Sets the text (foreground) color.
    ///
    /// Accepts anything convertible into [`StyledColor`]. See [`bg`] for
    /// the list of accepted types and examples.
    ///
    /// # Examples
    ///
    /// ```
    /// div().text_color(neutral_900())
    /// div().text_color(Color::from_rgb(17, 24, 39))
    /// div().text_color((17, 24, 39))
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use `StyledColor::default()` explicitly.
    ///
    /// [`bg`]: Self::bg
    fn text_color(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        self.get_style().text_color = sc.to_fill();
        self
    }

    // ---- border ----

    // ---- border ----

    /// Set the border color (Tailwind `border-{color}`). Creates a
    /// 1px border if none exists.
    ///
    /// Accepts anything convertible into [`StyledColor`], but — like CSS
    /// `border-color` — the **resolved value must be solid**. Gradients,
    /// shaders, and render callbacks panic; wrap the element in a `div()`
    /// with a gradient `background` if you need a gradient border.
    ///
    /// # Accepted types
    ///
    /// See [`StyledColor`]'s `# Conversions` section for the full list.
    /// The solid-returning subset is:
    ///
    /// - [`StyledColor`] wrapping a solid fill.
    /// - [`Color`] — solid RGB/ARGB.
    /// - `(u8, u8, u8)` / `(u8, u8, u8, f32)` / `(u8, u8, u8, u8)`.
    /// - [`Fill`], but only the `Fill::Color(..)` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// div().border_color(gray_300())
    /// div().border_color(Color::from_rgb(209, 213, 219))
    /// div().border_color((209, 213, 219))
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use `StyledColor::default()` explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the resolved value is a gradient, shader, or callback
    /// fill. `Border::fill` is a `Color`, so there's nowhere to store a
    /// non-solid fill:
    ///
    /// ```
    /// div().border_color(gray_300())                   // ✓
    /// // div().border_color(LinearGradient::new())     // ✗ panics
    /// ```
    fn border_color(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        let c = sc.to_color().unwrap_or_else(|| {
            panic!("border_color requires a solid color; got a gradient/shader/callback")
        });
        let style = self.get_style();
        let border = style.border.get_or_insert_with(|| Border::new().width(1.0));
        border.fill = c;
        self
    }

    // ---- ring ----

    /// Set the ring color. Alias for [`border_color`](Self::border_color)
    /// so shadcn class strings read literally.
    ///
    /// Accepts anything convertible into [`StyledColor`], but — like
    /// `border_color` — the resolved value must be **solid**. Gradients
    /// panic; see [`border_color`](Self::border_color#panics) for the
    /// full constraint.
    ///
    /// # Examples
    ///
    /// ```
    /// button().ring_color(ring())
    /// button().ring_color(Color::from_rgb(59, 130, 246))
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use `StyledColor::default()` explicitly.
    ///
    /// # Panics
    ///
    /// Same as [`border_color`](Self::border_color#panics) — panics on
    /// non-solid fills.
    fn ring_color(self, color: impl Into<StyledColor>) -> Self {
        self.border_color(color)
    }

    // ---- box shadows ----

    /// Recolor every currently-registered shadow.
    ///
    /// No-op when there are no shadows yet — Tailwind's `shadow-{color}`
    /// utilities are always paired with a size, and the same applies here.
    ///
    /// Accepts anything convertible into [`StyledColor`].
    ///
    /// # Examples
    ///
    /// ```
    /// div()
    ///     .shadow_md()
    ///     .shadow_color(blue_500())
    ///     .shadow_color(Color::from_rgb(59, 130, 246))
    ///     .shadow_color((59, 130, 246, 0.5))
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use `StyledColor::default()` explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the argument is a gradient, shader, or callback fill —
    /// shadow tints must be **solid**. Compose a tint from a palette
    /// function and `.opacity(..)`:
    ///
    /// ```
    /// div().shadow_md().shadow_color(black().opacity(0.2))   // ✓
    /// // div().shadow_md().shadow_color(LinearGradient::new())   // ✗ panics
    /// ```
    fn shadow_color(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        let c = sc.to_color().unwrap_or_else(|| {
            panic!("shadow_color requires a solid color; got a gradient/shader/callback")
        });
        let style = self.get_style();
        style.shadows = style.shadows.iter().map(|s| s.clone().color(c)).collect();
        self
    }

    // ---- text shadows ----

    /// Recolor every currently-registered text shadow, keeping offsets
    /// and sigmas.
    ///
    /// No-op when there are no text shadows yet — Tailwind's
    /// `text-shadow-{color}` utilities are always paired with a size,
    /// and the same applies here.
    ///
    /// Accepts anything convertible into [`StyledColor`].
    ///
    /// # Examples
    ///
    /// ```
    /// text("Hi")
    ///     .text_shadow_md()
    ///     .text_shadow_color(primary())
    ///     .text_shadow_color(Color::from_rgb(59, 130, 246))
    /// ```
    ///
    /// # Ambiguity
    ///
    /// `Default::default()` doesn't infer through `impl Into<StyledColor>`.
    /// Use `StyledColor::default()` explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the argument is a gradient, shader, or callback fill —
    /// shadow tints must be **solid**. Compose a tint from a palette
    /// function and `.opacity(..)`:
    ///
    /// ```
    /// text("Hi").text_shadow_md().text_shadow_color(black().opacity(0.2))   // ✓
    /// // text("Hi").text_shadow_md().text_shadow_color(LinearGradient::new())   // ✗ panics
    /// ```
    fn text_shadow_color(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        let c = sc.to_color().unwrap_or_else(|| {
            panic!("text_shadow_color requires a solid color; got a gradient/shader/callback")
        });
        let style = self.get_style();
        style.text_shadows = style
            .text_shadows
            .iter()
            .map(|s| TextShadow::new(c, s.offset, s.blur_sigma))
            .collect();
        self
    }

    /// Set `background` from a semantic token (`"primary"`, `"card"`, …).
    fn bg_token(self, theme: &Theme, key: &str) -> Self {
        let fill = theme.fill(key).unwrap_or_default();
        self.bg(StyledColor::from_fill(fill))
    }

    /// Set `background` from a component token with state fallback.
    fn bg_component(self, theme: &Theme, path: &str, state: ComponentState, field: &str) -> Self {
        let fill = theme.state_fill(path, state, field).unwrap_or_default();
        self.bg(StyledColor::from_fill(fill))
    }

    /// Same for the foreground.
    fn text_token(self, theme: &Theme, key: &str) -> Self {
        let fill = theme.fill(key).unwrap_or_default();
        self.text_color(StyledColor::from_fill(fill))
    }

    fn text_component(self, theme: &Theme, path: &str, state: ComponentState, field: &str) -> Self {
        let fill = theme.state_fill(path, state, field).unwrap_or_default();
        self.text_color(StyledColor::from_fill(fill))
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

    // ---- font size: theme-driven shortcuts ----
    //
    // Resolves `typography.fontSize.<key>` from the active theme. Each
    // shortcut accepts its own key plus (for `text_md`) the `base` alias
    // that `Style::default()` reads, so a theme that only defines
    // `fontSize.base` still drives `text_md()`.

    /// Extra-extra-small — `12px`. Resolves theme `fontSize.xs`.
    fn text_xs(self) -> Self {
        self.font_size(active_theme().font_size_or(&["xs"], 12.0))
    }
    /// Small — `14px`. Resolves theme `fontSize.sm`.
    fn text_sm(self) -> Self {
        self.font_size(active_theme().font_size_or(&["sm"], 14.0))
    }
    /// Base — `16px`. Resolves theme `fontSize.md` or `fontSize.base`.
    fn text_md(self) -> Self {
        self.font_size(active_theme().font_size_or(&["md", "base"], 16.0))
    }
    /// Large — `18px`. Resolves theme `fontSize.lg`.
    fn text_lg(self) -> Self {
        self.font_size(active_theme().font_size_or(&["lg"], 18.0))
    }
    /// Extra-large — `20px`. Resolves theme `fontSize.xl`.
    fn text_xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["xl"], 20.0))
    }
    /// 2× extra-large — `24px`. Resolves theme `fontSize.2xl`.
    fn text_2xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["2xl"], 24.0))
    }
    /// 3× extra-large — `30px`. Resolves theme `fontSize.3xl`.
    fn text_3xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["3xl"], 30.0))
    }
    /// 4× extra-large — `36px`. Resolves theme `fontSize.4xl`.
    fn text_4xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["4xl"], 36.0))
    }
    /// 5× extra-large — `48px`. Resolves theme `fontSize.5xl`.
    fn text_5xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["5xl"], 48.0))
    }
    /// 6× extra-large — `60px`. Resolves theme `fontSize.6xl`.
    fn text_6xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["6xl"], 60.0))
    }
    /// 7× extra-large — `72px`. Resolves theme `fontSize.7xl`.
    fn text_7xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["7xl"], 72.0))
    }
    /// 8× extra-large — `96px`. Resolves theme `fontSize.8xl`.
    fn text_8xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["8xl"], 96.0))
    }
    /// 9× extra-large — `128px`. Resolves theme `fontSize.9xl`.
    fn text_9xl(self) -> Self {
        self.font_size(active_theme().font_size_or(&["9xl"], 128.0))
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

    // ---- font weight: theme-driven shortcuts ----
    //
    // Resolves `typography.fontWeight.<key>` from the active theme,
    // falling back to the standard CSS weight. `font_semibold` also accepts
    // `"semi_bold"` as an alias.

    /// Hairline — `100`. Resolves theme `fontWeight.thin`.
    fn font_thin(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["thin"], 100),
        ))
    }
    /// Extra-light — `200`. Resolves theme `fontWeight.extralight`.
    fn font_extralight(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["extralight", "extra_light"], 200),
        ))
    }
    /// Light — `300`. Resolves theme `fontWeight.light`.
    fn font_light(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["light"], 300),
        ))
    }
    /// Regular — `400`. Resolves theme `fontWeight.normal` or `fontWeight.base`.
    fn font_normal(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["normal", "base"], 400),
        ))
    }
    /// Medium — `500`. Resolves theme `fontWeight.medium`.
    fn font_medium(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["medium"], 500),
        ))
    }
    /// Semi-bold — `600`. Resolves theme `fontWeight.semibold`.
    fn font_semibold(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["semibold", "semi_bold"], 600),
        ))
    }
    /// Bold — `700`. Resolves theme `fontWeight.bold`.
    fn font_bold(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["bold"], 700),
        ))
    }
    /// Extra-bold — `800`. Resolves theme `fontWeight.extrabold`.
    fn font_extrabold(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["extrabold", "extra_bold"], 800),
        ))
    }
    /// Black/heavy — `900`. Resolves theme `fontWeight.black`.
    fn font_black(self) -> Self {
        self.font_weight(weight_to_font_weight(
            active_theme().font_weight_or(&["black"], 900),
        ))
    }

    // ---- letter spacing ----

    /// Sets letter spacing, expressed in `em` (a multiple of the current font size).
    ///
    /// Because the value is relative to the font size, the resulting pixel
    /// spacing scales automatically with [`font_size`](Self::font_size).
    /// Call this **after** the font-size setter to get the expected result:
    ///
    /// ```
    /// text("Hi").text_lg().tracking(0.05) // 18px * 0.05 = 0.9px
    /// ```
    ///
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
    /// `font_*` shortcuts below) stacks them.
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
    fn font_family(mut self, font_family: impl Into<Cow<'static, str>>) -> Self {
        self.get_style().font_families.push(font_family.into());
        self
    }

    /// Append several font families at once, in order of preference.
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

    /// Replace the font stack with the theme's `fontFamily.sans` stack,
    /// falling back to the built-in sans stack.
    fn font_sans(self) -> Self {
        if let Some(stack) = active_theme().font_family_stack("sans") {
            return self.clear_font_families().font_families(stack);
        }
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

    /// Replace the font stack with the theme's `fontFamily.serif` stack,
    /// falling back to the built-in serif stack.
    fn font_serif(self) -> Self {
        if let Some(stack) = active_theme().font_family_stack("serif") {
            return self.clear_font_families().font_families(stack);
        }
        self.clear_font_families().font_families([
            "ui-serif",
            "Georgia",
            "Cambria",
            "Times New Roman",
            "Times",
            "serif",
        ])
    }

    /// Replace the font stack with the theme's `fontFamily.mono` stack,
    /// falling back to the built-in monospace stack.
    fn font_mono(self) -> Self {
        if let Some(stack) = active_theme().font_family_stack("mono") {
            return self.clear_font_families().font_families(stack);
        }
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
    fn font_style(mut self, font_style: impl Into<FontSlant>) -> Self {
        self.get_style().font_style = font_style.into();
        self
    }

    /// Italic text — Tailwind's `italic`.
    fn italic(self) -> Self {
        self.font_style(FontSlant::Italic)
    }

    /// Upright text — Tailwind's `not-italic`. This is the default.
    fn non_italic(self) -> Self {
        self.font_style(FontSlant::Upright)
    }

    /// Oblique text — slanted upright glyphs. No Tailwind equivalent.
    fn oblique(self) -> Self {
        self.font_style(FontSlant::Oblique)
    }

    // ---- font stretch (width) ----

    /// Sets the font stretch (width) axis.
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

    // ---- line height ----

    /// Sets the line height as a **unitless multiplier** of the font size.
    fn line_height(mut self, line_height: f32) -> Self {
        self.get_style().line_height = line_height;
        self
    }

    /// No extra leading — `1.0`. Tailwind's `leading-none`.
    fn leading_none(self) -> Self {
        self.line_height(1.0)
    }

    /// Tight leading — `1.25`. Tailwind's `leading-tight`.
    fn leading_tight(self) -> Self {
        self.line_height(1.25)
    }

    /// Snug leading — `1.375`. Tailwind's `leading-snug`.
    fn leading_snug(self) -> Self {
        self.line_height(1.375)
    }

    /// Normal leading — `1.5`. Tailwind's `leading-normal`.
    fn leading_normal(self) -> Self {
        self.line_height(1.5)
    }

    /// Relaxed leading — `1.625`. Tailwind's `leading-relaxed`.
    fn leading_relaxed(self) -> Self {
        self.line_height(1.625)
    }

    /// Loose leading — `2.0`. Tailwind's `leading-loose`.
    fn leading_loose(self) -> Self {
        self.line_height(2.0)
    }

    // ---- text shadow ----
    //
    // Freya's `TextShadow` takes a **blur sigma** (standard deviation),
    // not the CSS `blur-radius` you write in `text-shadow:` shorthand.
    // The conversion used below is the SVG spec's `sigma = radius / 2`.
    //
    // Presets **replace** the current `text_shadows` vector — Tailwind's
    // `text-shadow-*` utilities set the whole `text-shadow` property, so
    // `.text_shadow_sm().text_shadow_lg()` renders the `lg` value, not
    // both. For manual multi-layer composition, use `.text_shadow(..)`
    // (which appends) instead of the size presets.

    /// Append a single text shadow. Layers accumulate, so chaining
    /// `.text_shadow(a).text_shadow(b)` produces two shadows.
    ///
    /// Accepts anything `Into<TextShadow>`:
    ///
    /// ```
    /// text("Hi").text_shadow(TextShadow::new(black().into(), (0.0, 2.0), 1.0))
    /// ```
    fn text_shadow(mut self, text_shadow: impl Into<TextShadow>) -> Self {
        self.get_style().text_shadows.push(text_shadow.into());
        self
    }

    /// Remove every text shadow. Tailwind's `text-shadow-none`.
    fn text_shadow_none(mut self) -> Self {
        self.get_style().text_shadows.clear();
        self
    }

    /// Extra-extra-small — `0 1px 0 rgb(0 0 0 / 0.15)`.
    /// Tailwind's `text-shadow-2xs`.
    fn text_shadow_2xs(mut self) -> Self {
        self.get_style().text_shadows = vec![TextShadow::new(
            black().opacity(0.15).into(),
            (0.0, 1.0),
            0.0,
        )];
        self
    }

    /// Extra-small — `0 1px 1px rgb(0 0 0 / 0.2)`.
    /// Tailwind's `text-shadow-xs`.
    ///
    /// 1px CSS blur radius → 0.5 sigma.
    fn text_shadow_xs(mut self) -> Self {
        self.get_style().text_shadows = vec![TextShadow::new(
            black().opacity(0.2).into(),
            (0.0, 1.0),
            0.5,
        )];
        self
    }

    /// Small — two layers:
    /// `0 1px 2px rgb(0 0 0 / 0.15)` and `0 1px 3px rgb(0 0 0 / 0.2)`.
    /// Tailwind's `text-shadow-sm`.
    fn text_shadow_sm(mut self) -> Self {
        self.get_style().text_shadows = vec![
            TextShadow::new(black().opacity(0.15).into(), (0.0, 1.0), 1.0),
            TextShadow::new(black().opacity(0.2).into(), (0.0, 1.0), 1.5),
        ];
        self
    }

    /// Medium — three layers, all at `10%` black:
    /// `0 1px 2px`, `0 2px 4px`, `0 4px 8px`.
    /// Tailwind's `text-shadow-md`.
    fn text_shadow_md(mut self) -> Self {
        self.get_style().text_shadows = vec![
            TextShadow::new(black().opacity(0.1).into(), (0.0, 1.0), 1.0),
            TextShadow::new(black().opacity(0.1).into(), (0.0, 2.0), 2.0),
            TextShadow::new(black().opacity(0.1).into(), (0.0, 4.0), 4.0),
        ];
        self
    }

    /// Large — four layers, all at `10%` black:
    /// `0 1px 2px`, `0 3px 6px`, `0 6px 12px`, `0 10px 20px`.
    /// Tailwind's `text-shadow-lg`.
    fn text_shadow_lg(mut self) -> Self {
        self.get_style().text_shadows = vec![
            TextShadow::new(black().opacity(0.1).into(), (0.0, 1.0), 1.0),
            TextShadow::new(black().opacity(0.1).into(), (0.0, 3.0), 3.0),
            TextShadow::new(black().opacity(0.1).into(), (0.0, 6.0), 6.0),
            TextShadow::new(black().opacity(0.1).into(), (0.0, 10.0), 10.0),
        ];
        self
    }

    // ---- text decoration ----

    /// Sets the text decoration line.
    fn text_decoration(mut self, text_decoration: impl Into<TextDecoration>) -> Self {
        self.get_style().text_decoration = text_decoration.into();
        self
    }

    /// Draw a line underneath the text. Tailwind's `underline`.
    fn underline(self) -> Self {
        self.text_decoration(TextDecoration::Underline)
    }

    /// Draw a line above the text. Tailwind's `overline`.
    fn overline(self) -> Self {
        self.text_decoration(TextDecoration::Overline)
    }

    /// Draw a line through the middle of the text. Tailwind's `line-through`.
    fn line_through(self) -> Self {
        self.text_decoration(TextDecoration::LineThrough)
    }

    /// Remove any decoration line. Tailwind's `no-underline`.
    fn no_underline(self) -> Self {
        self.text_decoration(TextDecoration::None)
    }

    // ---- cursor ----

    /// Sets the mouse cursor icon shown when hovering the element.
    fn cursor(mut self, cursor: impl Into<CursorIcon>) -> Self {
        self.get_style().cursor = cursor.into();
        self.get_style().cursor_visible = true;
        self
    }

    /// Let the platform decide — Tailwind's `cursor-auto`.
    fn cursor_auto(self) -> Self {
        self.cursor(CursorIcon::Default)
    }

    /// The platform's default cursor. Tailwind's `cursor-default`.
    fn cursor_default(self) -> Self {
        self.cursor(CursorIcon::Default)
    }

    /// A context menu is available. Tailwind's `cursor-context-menu`.
    fn cursor_context_menu(self) -> Self {
        self.cursor(CursorIcon::ContextMenu)
    }

    /// Help is available. Tailwind's `cursor-help`.
    fn cursor_help(self) -> Self {
        self.cursor(CursorIcon::Help)
    }

    /// Something is clickable. Tailwind's `cursor-pointer`.
    fn cursor_pointer(self) -> Self {
        self.cursor(CursorIcon::Pointer)
    }

    /// The program is busy but accepts interaction. Tailwind's `cursor-progress`.
    fn cursor_progress(self) -> Self {
        self.cursor(CursorIcon::Progress)
    }

    /// The program is busy and the user should wait. Tailwind's `cursor-wait`.
    fn cursor_wait(self) -> Self {
        self.cursor(CursorIcon::Wait)
    }

    /// A cell may be selected. Tailwind's `cursor-cell`.
    fn cursor_cell(self) -> Self {
        self.cursor(CursorIcon::Cell)
    }

    /// A free-form selection. Tailwind's `cursor-crosshair`.
    fn cursor_crosshair(self) -> Self {
        self.cursor(CursorIcon::Crosshair)
    }

    /// Selectable text. Tailwind's `cursor-text`.
    fn cursor_text(self) -> Self {
        self.cursor(CursorIcon::Text)
    }

    /// Selectable vertical text. Tailwind's `cursor-vertical-text`.
    fn cursor_vertical_text(self) -> Self {
        self.cursor(CursorIcon::VerticalText)
    }

    /// An alias / shortcut will be created. Tailwind's `cursor-alias`.
    fn cursor_alias(self) -> Self {
        self.cursor(CursorIcon::Alias)
    }

    /// Something will be copied. Tailwind's `cursor-copy`.
    fn cursor_copy(self) -> Self {
        self.cursor(CursorIcon::Copy)
    }

    /// Something will be moved. Tailwind's `cursor-move`.
    fn cursor_move(self) -> Self {
        self.cursor(CursorIcon::Move)
    }

    /// The dragged item can't be dropped here. Tailwind's `cursor-no-drop`.
    fn cursor_no_drop(self) -> Self {
        self.cursor(CursorIcon::NoDrop)
    }

    /// The action isn't allowed. Tailwind's `cursor-not-allowed`.
    fn cursor_not_allowed(self) -> Self {
        self.cursor(CursorIcon::NotAllowed)
    }

    /// Something can be grabbed. Tailwind's `cursor-grab`.
    fn cursor_grab(self) -> Self {
        self.cursor(CursorIcon::Grab)
    }

    /// Something is being grabbed. Tailwind's `cursor-grabbing`.
    fn cursor_grabbing(self) -> Self {
        self.cursor(CursorIcon::Grabbing)
    }

    /// Resize the east border. Tailwind's `cursor-e-resize`.
    fn cursor_e_resize(self) -> Self {
        self.cursor(CursorIcon::EResize)
    }

    /// Resize the north border. Tailwind's `cursor-n-resize`.
    fn cursor_n_resize(self) -> Self {
        self.cursor(CursorIcon::NResize)
    }

    /// Resize the north-east corner. Tailwind's `cursor-ne-resize`.
    fn cursor_ne_resize(self) -> Self {
        self.cursor(CursorIcon::NeResize)
    }

    /// Resize the north-west corner. Tailwind's `cursor-nw-resize`.
    fn cursor_nw_resize(self) -> Self {
        self.cursor(CursorIcon::NwResize)
    }

    /// Resize the south border. Tailwind's `cursor-s-resize`.
    fn cursor_s_resize(self) -> Self {
        self.cursor(CursorIcon::SResize)
    }

    /// Resize the south-east corner. Tailwind's `cursor-se-resize`.
    fn cursor_se_resize(self) -> Self {
        self.cursor(CursorIcon::SeResize)
    }

    /// Resize the south-west corner. Tailwind's `cursor-sw-resize`.
    fn cursor_sw_resize(self) -> Self {
        self.cursor(CursorIcon::SwResize)
    }

    /// Resize the west border. Tailwind's `cursor-w-resize`.
    fn cursor_w_resize(self) -> Self {
        self.cursor(CursorIcon::WResize)
    }

    /// Resize horizontally. Tailwind's `cursor-ew-resize`.
    fn cursor_ew_resize(self) -> Self {
        self.cursor(CursorIcon::EwResize)
    }

    /// Resize vertically. Tailwind's `cursor-ns-resize`.
    fn cursor_ns_resize(self) -> Self {
        self.cursor(CursorIcon::NsResize)
    }

    /// Resize along the NE ↔ SW diagonal. Tailwind's `cursor-nesw-resize`.
    fn cursor_nesw_resize(self) -> Self {
        self.cursor(CursorIcon::NeswResize)
    }

    /// Resize along the NW ↔ SE diagonal. Tailwind's `cursor-nwse-resize`.
    fn cursor_nwse_resize(self) -> Self {
        self.cursor(CursorIcon::NwseResize)
    }

    /// Resize a column horizontally. Tailwind's `cursor-col-resize`.
    fn cursor_col_resize(self) -> Self {
        self.cursor(CursorIcon::ColResize)
    }

    /// Resize a row vertically. Tailwind's `cursor-row-resize`.
    fn cursor_row_resize(self) -> Self {
        self.cursor(CursorIcon::RowResize)
    }

    /// Scroll in any direction. Tailwind's `cursor-all-scroll`.
    fn cursor_all_scroll(self) -> Self {
        self.cursor(CursorIcon::AllScroll)
    }

    /// Zoom in. Tailwind's `cursor-zoom-in`.
    fn cursor_zoom_in(self) -> Self {
        self.cursor(CursorIcon::ZoomIn)
    }

    /// Zoom out. Tailwind's `cursor-zoom-out`.
    fn cursor_zoom_out(self) -> Self {
        self.cursor(CursorIcon::ZoomOut)
    }

    // ---- non-Tailwind cursor extensions ----

    /// The user will choose what action to take. No Tailwind equivalent.
    fn cursor_dnd_ask(self) -> Self {
        self.cursor(CursorIcon::DndAsk)
    }

    /// Something can be moved or resized in any direction. No Tailwind equivalent.
    fn cursor_all_resize(self) -> Self {
        self.cursor(CursorIcon::AllResize)
    }

    /// Sets whether the cursor is visible over this element.
    fn cursor_visible(mut self, visible: bool) -> Self {
        self.get_style().cursor_visible = visible;
        self
    }

    /// Hides the cursor while hovering the element. Tailwind's `cursor-none`.
    fn cursor_none(self) -> Self {
        self.cursor_visible(false)
    }

    // ---- overflow ----

    /// Sets how content that overflows the element's box is handled.
    fn overflow(mut self, overflow: impl Into<Overflow>) -> Self {
        self.get_style().overflow = overflow.into();
        self
    }

    /// Do not clip overflowing content. Freya's default.
    fn overflow_none(self) -> Self {
        self.overflow(Overflow::None)
    }

    /// Clip overflowing content at the element's bounds.
    fn overflow_clip(self) -> Self {
        self.overflow(Overflow::Clip)
    }

    // ---- backdrop blur ----

    /// Apply a Gaussian blur of the given radius (in pixels) to the backdrop.
    fn backdrop_blur(mut self, blur: f32) -> Self {
        self.get_style().blur = blur;
        self
    }

    /// No backdrop blur — `0px`. Tailwind's `backdrop-blur-none`.
    fn backdrop_blur_none(self) -> Self {
        self.backdrop_blur(0.0)
    }

    /// Extra-extra-small backdrop blur — `2px`. Tailwind's `backdrop-blur-2xs`.
    fn backdrop_blur_2xs(self) -> Self {
        self.backdrop_blur(2.0)
    }

    /// Extra-small backdrop blur — `4px`. Tailwind's `backdrop-blur-xs`.
    fn backdrop_blur_xs(self) -> Self {
        self.backdrop_blur(4.0)
    }

    /// Small backdrop blur — `8px`. Tailwind's `backdrop-blur-sm`.
    fn backdrop_blur_sm(self) -> Self {
        self.backdrop_blur(8.0)
    }

    /// Medium backdrop blur — `12px`. Tailwind's `backdrop-blur-md`.
    fn backdrop_blur_md(self) -> Self {
        self.backdrop_blur(12.0)
    }

    /// Large backdrop blur — `16px`. Tailwind's `backdrop-blur-lg`.
    fn backdrop_blur_lg(self) -> Self {
        self.backdrop_blur(16.0)
    }

    /// Extra-large backdrop blur — `24px`. Tailwind's `backdrop-blur-xl`.
    fn backdrop_blur_xl(self) -> Self {
        self.backdrop_blur(24.0)
    }

    /// 2× extra-large backdrop blur — `40px`. Tailwind's `backdrop-blur-2xl`.
    fn backdrop_blur_2xl(self) -> Self {
        self.backdrop_blur(40.0)
    }

    /// 3× extra-large backdrop blur — `64px`. Tailwind's `backdrop-blur-3xl`.
    fn backdrop_blur_3xl(self) -> Self {
        self.backdrop_blur(64.0)
    }

    // ======== PASTE INTO `StyledExt` (e.g. right after the backdrop-blur group) ========
    //
    // Needs this import at the top of styled.rs:
    //   use freya_core::elements::image::{AspectRatio, ImageCover, SamplingMode};

    // ================================ image ================================
    //
    // These mutate `Style`'s image fields. The image primitive's element-build
    // step is responsible for copying them into `ImageData` (see below).
    // They're no-ops for elements that aren't images.

    // ---- object fit ----

    /// Sets how the image is scaled to fit its bounds while preserving
    /// (or ignoring) its aspect ratio.
    ///
    /// The `object_*` shortcuts below delegate to this and match
    /// Tailwind's `object-fit` utilities.
    fn object_fit(mut self, aspect_ratio: impl Into<AspectRatio>) -> Self {
        self.get_style().object_fit = aspect_ratio.into();
        self
    }

    /// Scale so the whole image fits inside the bounds. Tailwind's
    /// `object-contain`.
    fn object_contain(self) -> Self {
        self.object_fit(AspectRatio::Min)
    }

    /// Scale so the image covers the whole bounds, cropping the overflow.
    /// Tailwind's `object-cover`.
    fn object_cover(self) -> Self {
        self.object_fit(AspectRatio::Max)
    }

    /// Stretch the image to the bounds, ignoring its aspect ratio.
    /// Tailwind's `object-fill`.
    fn object_fill(self) -> Self {
        self.object_fit(AspectRatio::None)
    }

    /// Keep the image at its natural (intrinsic) size. Tailwind's
    /// `object-none`.
    fn object_none(self) -> Self {
        self.object_fit(AspectRatio::Fit)
    }

    /// Scale so the image fits inside the bounds **only if it's bigger**;
    /// never scale a smaller image up. Tailwind's `object-scale-down`.
    ///
    /// # Approximation
    ///
    /// [`AspectRatio`] has no dedicated `ScaleDown` variant, so this
    /// falls back to [`AspectRatio::Min`] (always contain). When the
    /// image is **smaller** than the bounds, CSS's `object-scale-down`
    /// would keep the natural size while this shortcut scales it up.
    fn object_scale_down(self) -> Self {
        self.object_fit(AspectRatio::Min)
    }

    // ---- object position ----

    /// Sets where the image is anchored inside its bounds once scaled.
    ///
    /// [`ImageCover`] currently only models two anchors —
    /// [`ImageCover::Fill`] (top-left) and [`ImageCover::Center`] — so
    /// only `object_center` and `object_top_left` are available. The
    /// other Tailwind `object-*` position classes can't be expressed yet.
    fn object_position(mut self, image_cover: impl Into<ImageCover>) -> Self {
        self.get_style().object_position = image_cover.into();
        self
    }

    /// Center the image in its bounds. Tailwind's `object-center`.
    fn object_center(self) -> Self {
        self.object_position(ImageCover::Center)
    }

    /// Anchor the image to the top-left of its bounds. Tailwind's
    /// `object-top-left` / `object-left-top` (they're the same). This is
    /// the default.
    fn object_top_left(self) -> Self {
        self.object_position(ImageCover::Fill)
    }

    // ---- image rendering ----

    /// Sets the sampling (filtering) algorithm used when the image is
    /// scaled.
    ///
    /// The `image_*` rendering shortcuts and `sampling_*` raw shortcuts
    /// below all delegate to this.
    fn image_rendering(mut self, sampling: impl Into<SamplingMode>) -> Self {
        self.get_style().sampling_mode = sampling.into();
        self
    }

    /// Let Freya pick the best algorithm — currently
    /// [`SamplingMode::Trilinear`]. Tailwind's `image-auto`. This is the
    /// default.
    fn image_auto(self) -> Self {
        self.image_rendering(SamplingMode::Trilinear)
    }

    /// Nearest-neighbor — sharp pixels, best for pixel art. Tailwind's
    /// `image-pixelated`.
    fn image_pixelated(self) -> Self {
        self.image_rendering(SamplingMode::Nearest)
    }

    /// Smooth high-quality resampling. Tailwind's `image-smooth`.
    ///
    /// Maps to [`SamplingMode::Mitchell`] (Mitchell-Netravali cubic).
    /// If you prefer trilinear interpolation, use
    /// [`sampling_trilinear`](Self::sampling_trilinear).
    fn image_smooth(self) -> Self {
        self.image_rendering(SamplingMode::Mitchell)
    }

    /// Nearest-neighbor sampling. Same as
    /// [`image_pixelated`](Self::image_pixelated), named after the
    /// [`SamplingMode`] variant.
    fn sampling_nearest(self) -> Self {
        self.image_rendering(SamplingMode::Nearest)
    }

    /// Bilinear interpolation.
    fn sampling_bilinear(self) -> Self {
        self.image_rendering(SamplingMode::Bilinear)
    }

    /// Trilinear interpolation (bilinear + mipmaps). Freya's default.
    fn sampling_trilinear(self) -> Self {
        self.image_rendering(SamplingMode::Trilinear)
    }

    /// Mitchell-Netravali cubic resampling — smooth, no halos.
    fn sampling_mitchell(self) -> Self {
        self.image_rendering(SamplingMode::Mitchell)
    }

    /// Catmull-Rom cubic resampling — sharper, slightly more ringing.
    fn sampling_catmull_rom(self) -> Self {
        self.image_rendering(SamplingMode::CatmullRom)
    }

    // ---- image blur ----

    /// Applies a Gaussian blur of the given radius (in pixels) to the
    /// image itself.
    ///
    /// The `blur_*` shortcuts below delegate to this and match Tailwind's
    /// **filter** blur scale (`blur-*`, not `backdrop-blur-*` — the
    /// latter is [`backdrop_blur`](Self::backdrop_blur)).
    ///
    /// The value is clamped to `>= 0.0`, so `image_blur(-1.0)` is treated
    /// as `0.0` (no blur).
    fn image_blur(mut self, blur: f32) -> Self {
        self.get_style().image_blur = blur.max(0.0);
        self
    }

    /// No blur — `0px`. Tailwind's `blur-none`. This is the default.
    fn blur_none(self) -> Self {
        self.image_blur(0.0)
    }
    /// Extra-extra-small blur — `2px`. Tailwind's `blur-2xs` (new in v4).
    fn blur_2xs(self) -> Self {
        self.image_blur(2.0)
    }
    /// Extra-small blur — `4px`. Tailwind's `blur-xs`.
    fn blur_xs(self) -> Self {
        self.image_blur(4.0)
    }
    /// Small blur — `8px`. Tailwind's `blur-sm`.
    fn blur_sm(self) -> Self {
        self.image_blur(8.0)
    }
    /// Medium blur — `12px`. Tailwind's `blur-md`.
    fn blur_md(self) -> Self {
        self.image_blur(12.0)
    }
    /// Large blur — `16px`. Tailwind's `blur-lg`.
    fn blur_lg(self) -> Self {
        self.image_blur(16.0)
    }
    /// Extra-large blur — `24px`. Tailwind's `blur-xl`.
    fn blur_xl(self) -> Self {
        self.image_blur(24.0)
    }
    /// 2× extra-large blur — `40px`. Tailwind's `blur-2xl`.
    fn blur_2xl(self) -> Self {
        self.image_blur(40.0)
    }
    /// 3× extra-large blur — `64px`. Tailwind's `blur-3xl`.
    fn blur_3xl(self) -> Self {
        self.image_blur(64.0)
    }

    // ---- grid snapping (Freya-specific) ----

    /// Snap the image's destination rect (and its clip rect) to integer
    /// pixels. Avoids half-pixel anti-aliasing seams at non-integer
    /// positions or scale factors.
    ///
    /// No Tailwind equivalent — Tailwind's `image-pixelated` covers
    /// *sampling*, not *position* snapping. Use
    /// [`image_pixelated`](Self::image_pixelated) for the sampling side.
    fn snap_to_grid(mut self) -> Self {
        self.get_style().snap_to_grid = true;
        self
    }

    /// Stop snapping the image to the pixel grid. This is the default.
    fn no_snap_to_grid(mut self) -> Self {
        self.get_style().snap_to_grid = false;
        self
    }

    // ---- rotation ----

    /// Rotate the element by the given angle **in degrees** (clockwise).
    fn rotate(mut self, rotation: f32) -> Self {
        self.get_style().rotation = rotation;
        self
    }

    /// No rotation — `0deg`. Tailwind's `rotate-none` / `rotate-0`.
    fn rotate_none(self) -> Self {
        self.rotate(0.0)
    }

    /// Rotate `0deg`. Tailwind's `rotate-0`.
    fn rotate_0(self) -> Self {
        self.rotate(0.0)
    }

    /// Rotate `1deg` clockwise. Tailwind's `rotate-1`.
    fn rotate_1(self) -> Self {
        self.rotate(1.0)
    }

    /// Rotate `2deg` clockwise. Tailwind's `rotate-2`.
    fn rotate_2(self) -> Self {
        self.rotate(2.0)
    }

    /// Rotate `3deg` clockwise. Tailwind's `rotate-3`.
    fn rotate_3(self) -> Self {
        self.rotate(3.0)
    }

    /// Rotate `6deg` clockwise. Tailwind's `rotate-6`.
    fn rotate_6(self) -> Self {
        self.rotate(6.0)
    }

    /// Rotate `12deg` clockwise. Tailwind's `rotate-12`.
    fn rotate_12(self) -> Self {
        self.rotate(12.0)
    }

    /// Rotate `45deg` clockwise. Tailwind's `rotate-45`.
    fn rotate_45(self) -> Self {
        self.rotate(45.0)
    }

    /// Rotate `90deg` clockwise. Tailwind's `rotate-90`.
    fn rotate_90(self) -> Self {
        self.rotate(90.0)
    }

    /// Rotate `180deg` clockwise. Tailwind's `rotate-180`.
    fn rotate_180(self) -> Self {
        self.rotate(180.0)
    }

    // ---- negative rotation ----

    /// Rotate `-1deg`. Tailwind's `-rotate-1`.
    fn rotate_neg_1(self) -> Self {
        self.rotate(-1.0)
    }

    /// Rotate `-2deg`. Tailwind's `-rotate-2`.
    fn rotate_neg_2(self) -> Self {
        self.rotate(-2.0)
    }

    /// Rotate `-3deg`. Tailwind's `-rotate-3`.
    fn rotate_neg_3(self) -> Self {
        self.rotate(-3.0)
    }

    /// Rotate `-6deg`. Tailwind's `-rotate-6`.
    fn rotate_neg_6(self) -> Self {
        self.rotate(-6.0)
    }

    /// Rotate `-12deg`. Tailwind's `-rotate-12`.
    fn rotate_neg_12(self) -> Self {
        self.rotate(-12.0)
    }

    /// Rotate `-45deg`. Tailwind's `-rotate-45`.
    fn rotate_neg_45(self) -> Self {
        self.rotate(-45.0)
    }

    /// Rotate `-90deg`. Tailwind's `-rotate-90`.
    fn rotate_neg_90(self) -> Self {
        self.rotate(-90.0)
    }

    /// Rotate `-180deg`. Tailwind's `-rotate-180`.
    fn rotate_neg_180(self) -> Self {
        self.rotate(-180.0)
    }

    // ---- opacity ----

    /// Set the element's opacity, from `0.0` (transparent) to `1.0` (opaque).
    fn opacity(mut self, opacity: f32) -> Self {
        self.get_style().opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Fully transparent — `0%`. Tailwind's `opacity-0`.
    fn opacity_0(self) -> Self {
        self.opacity(0.0)
    }
    /// `5%` opaque. Tailwind's `opacity-5`.
    fn opacity_5(self) -> Self {
        self.opacity(0.05)
    }
    /// `10%` opaque. Tailwind's `opacity-10`.
    fn opacity_10(self) -> Self {
        self.opacity(0.10)
    }
    /// `15%` opaque. Tailwind's `opacity-15`.
    fn opacity_15(self) -> Self {
        self.opacity(0.15)
    }
    /// `20%` opaque. Tailwind's `opacity-20`.
    fn opacity_20(self) -> Self {
        self.opacity(0.20)
    }
    /// `25%` opaque. Tailwind's `opacity-25`.
    fn opacity_25(self) -> Self {
        self.opacity(0.25)
    }
    /// `30%` opaque. Tailwind's `opacity-30`.
    fn opacity_30(self) -> Self {
        self.opacity(0.30)
    }
    /// `35%` opaque. Tailwind's `opacity-35`.
    fn opacity_35(self) -> Self {
        self.opacity(0.35)
    }
    /// `40%` opaque. Tailwind's `opacity-40`.
    fn opacity_40(self) -> Self {
        self.opacity(0.40)
    }
    /// `45%` opaque. Tailwind's `opacity-45`.
    fn opacity_45(self) -> Self {
        self.opacity(0.45)
    }
    /// `50%` opaque. Tailwind's `opacity-50`.
    fn opacity_50(self) -> Self {
        self.opacity(0.50)
    }
    /// `55%` opaque. Tailwind's `opacity-55`.
    fn opacity_55(self) -> Self {
        self.opacity(0.55)
    }
    /// `60%` opaque. Tailwind's `opacity-60`.
    fn opacity_60(self) -> Self {
        self.opacity(0.60)
    }
    /// `65%` opaque. Tailwind's `opacity-65`.
    fn opacity_65(self) -> Self {
        self.opacity(0.65)
    }
    /// `70%` opaque. Tailwind's `opacity-70`.
    fn opacity_70(self) -> Self {
        self.opacity(0.70)
    }
    /// `75%` opaque. Tailwind's `opacity-75`.
    fn opacity_75(self) -> Self {
        self.opacity(0.75)
    }
    /// `80%` opaque. Tailwind's `opacity-80`.
    fn opacity_80(self) -> Self {
        self.opacity(0.80)
    }
    /// `85%` opaque. Tailwind's `opacity-85`.
    fn opacity_85(self) -> Self {
        self.opacity(0.85)
    }
    /// `90%` opaque. Tailwind's `opacity-90`.
    fn opacity_90(self) -> Self {
        self.opacity(0.90)
    }
    /// `95%` opaque. Tailwind's `opacity-95`.
    fn opacity_95(self) -> Self {
        self.opacity(0.95)
    }
    /// Fully opaque — `100%`. Tailwind's `opacity-100`. This is the default.
    fn opacity_100(self) -> Self {
        self.opacity(1.0)
    }

    // ---- scale ----

    /// Scale the element. See [`Scale`].
    fn scale(mut self, scale: impl Into<Scale>) -> Self {
        self.get_style().scale = scale.into();
        self
    }

    /// Set the horizontal scale factor, keeping the vertical one unchanged.
    fn scale_x(mut self, x: f32) -> Self {
        let mut s = self.get_style().scale;
        s.x = x;
        self.get_style().scale = s;
        self
    }

    /// Set the vertical scale factor, keeping the horizontal one unchanged.
    fn scale_y(mut self, y: f32) -> Self {
        let mut s = self.get_style().scale;
        s.y = y;
        self.get_style().scale = s;
        self
    }

    // ---- uniform scale ----

    /// Uniform `0%` scale. Tailwind's `scale-0`.
    fn scale_0(self) -> Self {
        self.scale(0.0)
    }
    /// Uniform `50%` scale. Tailwind's `scale-50`.
    fn scale_50(self) -> Self {
        self.scale(0.50)
    }
    /// Uniform `75%` scale. Tailwind's `scale-75`.
    fn scale_75(self) -> Self {
        self.scale(0.75)
    }
    /// Uniform `90%` scale. Tailwind's `scale-90`.
    fn scale_90(self) -> Self {
        self.scale(0.90)
    }
    /// Uniform `95%` scale. Tailwind's `scale-95`.
    fn scale_95(self) -> Self {
        self.scale(0.95)
    }
    /// Uniform `100%` scale. Tailwind's `scale-100`. This is the default.
    fn scale_100(self) -> Self {
        self.scale(1.0)
    }
    /// Uniform `105%` scale. Tailwind's `scale-105`.
    fn scale_105(self) -> Self {
        self.scale(1.05)
    }
    /// Uniform `110%` scale. Tailwind's `scale-110`.
    fn scale_110(self) -> Self {
        self.scale(1.10)
    }
    /// Uniform `125%` scale. Tailwind's `scale-125`.
    fn scale_125(self) -> Self {
        self.scale(1.25)
    }
    /// Uniform `150%` scale. Tailwind's `scale-150`.
    fn scale_150(self) -> Self {
        self.scale(1.50)
    }
    /// Uniform `200%` scale. Tailwind's `scale-200`.
    fn scale_200(self) -> Self {
        self.scale(2.0)
    }

    // ---- negative uniform scale ----

    /// Uniform `-50%` scale. Tailwind's `-scale-50`.
    fn scale_neg_50(self) -> Self {
        self.scale(-0.50)
    }
    /// Uniform `-75%` scale. Tailwind's `-scale-75`.
    fn scale_neg_75(self) -> Self {
        self.scale(-0.75)
    }
    /// Uniform `-90%` scale. Tailwind's `-scale-90`.
    fn scale_neg_90(self) -> Self {
        self.scale(-0.90)
    }
    /// Uniform `-95%` scale. Tailwind's `-scale-95`.
    fn scale_neg_95(self) -> Self {
        self.scale(-0.95)
    }
    /// Uniform `-100%` scale. Tailwind's `-scale-100`.
    fn scale_neg_100(self) -> Self {
        self.scale(-1.0)
    }
    /// Uniform `-105%` scale. Tailwind's `-scale-105`.
    fn scale_neg_105(self) -> Self {
        self.scale(-1.05)
    }
    /// Uniform `-110%` scale. Tailwind's `-scale-110`.
    fn scale_neg_110(self) -> Self {
        self.scale(-1.10)
    }
    /// Uniform `-125%` scale. Tailwind's `-scale-125`.
    fn scale_neg_125(self) -> Self {
        self.scale(-1.25)
    }
    /// Uniform `-150%` scale. Tailwind's `-scale-150`.
    fn scale_neg_150(self) -> Self {
        self.scale(-1.50)
    }
    /// Uniform `-200%` scale. Tailwind's `-scale-200`.
    fn scale_neg_200(self) -> Self {
        self.scale(-2.0)
    }

    // ---- horizontal scale ----

    /// Horizontal `50%` scale. Tailwind's `scale-x-50`.
    fn scale_x_50(self) -> Self {
        self.scale_x(0.50)
    }
    /// Horizontal `75%` scale. Tailwind's `scale-x-75`.
    fn scale_x_75(self) -> Self {
        self.scale_x(0.75)
    }
    /// Horizontal `90%` scale. Tailwind's `scale-x-90`.
    fn scale_x_90(self) -> Self {
        self.scale_x(0.90)
    }
    /// Horizontal `95%` scale. Tailwind's `scale-x-95`.
    fn scale_x_95(self) -> Self {
        self.scale_x(0.95)
    }
    /// Horizontal `100%` scale. Tailwind's `scale-x-100`.
    fn scale_x_100(self) -> Self {
        self.scale_x(1.0)
    }
    /// Horizontal `105%` scale. Tailwind's `scale-x-105`.
    fn scale_x_105(self) -> Self {
        self.scale_x(1.05)
    }
    /// Horizontal `110%` scale. Tailwind's `scale-x-110`.
    fn scale_x_110(self) -> Self {
        self.scale_x(1.10)
    }
    /// Horizontal `125%` scale. Tailwind's `scale-x-125`.
    fn scale_x_125(self) -> Self {
        self.scale_x(1.25)
    }
    /// Horizontal `150%` scale. Tailwind's `scale-x-150`.
    fn scale_x_150(self) -> Self {
        self.scale_x(1.50)
    }
    /// Horizontal `200%` scale. Tailwind's `scale-x-200`.
    fn scale_x_200(self) -> Self {
        self.scale_x(2.0)
    }

    // ---- vertical scale ----

    /// Vertical `50%` scale. Tailwind's `scale-y-50`.
    fn scale_y_50(self) -> Self {
        self.scale_y(0.50)
    }
    /// Vertical `75%` scale. Tailwind's `scale-y-75`.
    fn scale_y_75(self) -> Self {
        self.scale_y(0.75)
    }
    /// Vertical `90%` scale. Tailwind's `scale-y-90`.
    fn scale_y_90(self) -> Self {
        self.scale_y(0.90)
    }
    /// Vertical `95%` scale. Tailwind's `scale-y-95`.
    fn scale_y_95(self) -> Self {
        self.scale_y(0.95)
    }
    /// Vertical `100%` scale. Tailwind's `scale-y-100`.
    fn scale_y_100(self) -> Self {
        self.scale_y(1.0)
    }
    /// Vertical `105%` scale. Tailwind's `scale-y-105`.
    fn scale_y_105(self) -> Self {
        self.scale_y(1.05)
    }
    /// Vertical `110%` scale. Tailwind's `scale-y-110`.
    fn scale_y_110(self) -> Self {
        self.scale_y(1.10)
    }
    /// Vertical `125%` scale. Tailwind's `scale-y-125`.
    fn scale_y_125(self) -> Self {
        self.scale_y(1.25)
    }
    /// Vertical `150%` scale. Tailwind's `scale-y-150`.
    fn scale_y_150(self) -> Self {
        self.scale_y(1.50)
    }
    /// Vertical `200%` scale. Tailwind's `scale-y-200`.
    fn scale_y_200(self) -> Self {
        self.scale_y(2.0)
    }

    // ---- negative horizontal scale ----

    /// Horizontal `-50%` scale. Tailwind's `-scale-x-50`.
    fn scale_x_neg_50(self) -> Self {
        self.scale_x(-0.50)
    }
    /// Horizontal `-75%` scale. Tailwind's `-scale-x-75`.
    fn scale_x_neg_75(self) -> Self {
        self.scale_x(-0.75)
    }
    /// Horizontal `-90%` scale. Tailwind's `-scale-x-90`.
    fn scale_x_neg_90(self) -> Self {
        self.scale_x(-0.90)
    }
    /// Horizontal `-95%` scale. Tailwind's `-scale-x-95`.
    fn scale_x_neg_95(self) -> Self {
        self.scale_x(-0.95)
    }
    /// Horizontal `-100%` scale. Tailwind's `-scale-x-100`.
    fn scale_x_neg_100(self) -> Self {
        self.scale_x(-1.0)
    }
    /// Horizontal `-105%` scale. Tailwind's `-scale-x-105`.
    fn scale_x_neg_105(self) -> Self {
        self.scale_x(-1.05)
    }
    /// Horizontal `-110%` scale. Tailwind's `-scale-x-110`.
    fn scale_x_neg_110(self) -> Self {
        self.scale_x(-1.10)
    }
    /// Horizontal `-125%` scale. Tailwind's `-scale-x-125`.
    fn scale_x_neg_125(self) -> Self {
        self.scale_x(-1.25)
    }
    /// Horizontal `-150%` scale. Tailwind's `-scale-x-150`.
    fn scale_x_neg_150(self) -> Self {
        self.scale_x(-1.50)
    }
    /// Horizontal `-200%` scale. Tailwind's `-scale-x-200`.
    fn scale_x_neg_200(self) -> Self {
        self.scale_x(-2.0)
    }

    // ---- negative vertical scale ----

    /// Vertical `-50%` scale. Tailwind's `-scale-y-50`.
    fn scale_y_neg_50(self) -> Self {
        self.scale_y(-0.50)
    }
    /// Vertical `-75%` scale. Tailwind's `-scale-y-75`.
    fn scale_y_neg_75(self) -> Self {
        self.scale_y(-0.75)
    }
    /// Vertical `-90%` scale. Tailwind's `-scale-y-90`.
    fn scale_y_neg_90(self) -> Self {
        self.scale_y(-0.90)
    }
    /// Vertical `-95%` scale. Tailwind's `-scale-y-95`.
    fn scale_y_neg_95(self) -> Self {
        self.scale_y(-0.95)
    }
    /// Vertical `-100%` scale. Tailwind's `-scale-y-100`.
    fn scale_y_neg_100(self) -> Self {
        self.scale_y(-1.0)
    }
    /// Vertical `-105%` scale. Tailwind's `-scale-y-105`.
    fn scale_y_neg_105(self) -> Self {
        self.scale_y(-1.05)
    }
    /// Vertical `-110%` scale. Tailwind's `-scale-y-110`.
    fn scale_y_neg_110(self) -> Self {
        self.scale_y(-1.10)
    }
    /// Vertical `-125%` scale. Tailwind's `-scale-y-125`.
    fn scale_y_neg_125(self) -> Self {
        self.scale_y(-1.25)
    }
    /// Vertical `-150%` scale. Tailwind's `-scale-y-150`.
    fn scale_y_neg_150(self) -> Self {
        self.scale_y(-1.50)
    }
    /// Vertical `-200%` scale. Tailwind's `-scale-y-200`.
    fn scale_y_neg_200(self) -> Self {
        self.scale_y(-2.0)
    }

    // ---- transform origin ----

    /// Sets the reference point that [`scale`](Self::scale) and
    /// [`rotate`](Self::rotate) pivot around.
    fn transform_origin(mut self, origin: impl Into<TransformOrigin>) -> Self {
        self.get_style().transform_origin = origin.into();
        self
    }

    /// Pivot around the element's center. Tailwind's `origin-center`.
    fn origin_center(self) -> Self {
        self.transform_origin(TransformOrigin::center())
    }

    /// Pivot around the top edge. Tailwind's `origin-top`.
    fn origin_top(self) -> Self {
        self.transform_origin(TransformOrigin::top())
    }

    /// Pivot around the top-right corner. Tailwind's `origin-top-right`.
    fn origin_top_right(self) -> Self {
        self.transform_origin(TransformOrigin::top_right())
    }

    /// Pivot around the right edge. Tailwind's `origin-right`.
    fn origin_right(self) -> Self {
        self.transform_origin(TransformOrigin::right())
    }

    /// Pivot around the bottom-right corner. Tailwind's `origin-bottom-right`.
    fn origin_bottom_right(self) -> Self {
        self.transform_origin(TransformOrigin::bottom_right())
    }

    /// Pivot around the bottom edge. Tailwind's `origin-bottom`.
    fn origin_bottom(self) -> Self {
        self.transform_origin(TransformOrigin::bottom())
    }

    /// Pivot around the bottom-left corner. Tailwind's `origin-bottom-left`.
    fn origin_bottom_left(self) -> Self {
        self.transform_origin(TransformOrigin::bottom_left())
    }

    /// Pivot around the left edge. Tailwind's `origin-left`.
    fn origin_left(self) -> Self {
        self.transform_origin(TransformOrigin::left())
    }

    /// Pivot around the top-left corner. Tailwind's `origin-top-left`.
    fn origin_top_left(self) -> Self {
        self.transform_origin(TransformOrigin::top_left())
    }

    // ---- radius ----

    /// Set the element's corner radius. See [`CornerRadius`].
    fn radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.get_style().radius = radius.into();
        self
    }

    /// Set only the **top** edge radius (`top-left` + `top-right`).
    fn radius_t(mut self, size: f32) -> Self {
        self.get_style().radius.fill_top(size);
        self
    }

    /// Set only the **bottom** edge radius (`bottom-left` + `bottom-right`).
    fn radius_b(mut self, size: f32) -> Self {
        self.get_style().radius.fill_bottom(size);
        self
    }

    /// Set only the **left** edge radius (`top-left` + `bottom-left`).
    fn radius_l(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius = CornerRadius::new(size, r.top_right(), r.bottom_right(), size)
            .with_smoothing(r.smoothing());
        self
    }

    /// Set only the **right** edge radius (`top-right` + `bottom-right`).
    fn radius_r(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius = CornerRadius::new(r.top_left(), size, size, r.bottom_left())
            .with_smoothing(r.smoothing());
        self
    }

    /// Set only the **top-left** corner radius.
    fn radius_tl(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius =
            CornerRadius::new(size, r.top_right(), r.bottom_right(), r.bottom_left())
                .with_smoothing(r.smoothing());
        self
    }

    /// Set only the **top-right** corner radius.
    fn radius_tr(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius =
            CornerRadius::new(r.top_left(), size, r.bottom_right(), r.bottom_left())
                .with_smoothing(r.smoothing());
        self
    }

    /// Set only the **bottom-right** corner radius.
    fn radius_br(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius =
            CornerRadius::new(r.top_left(), r.top_right(), size, r.bottom_left())
                .with_smoothing(r.smoothing());
        self
    }

    /// Set only the **bottom-left** corner radius.
    fn radius_bl(mut self, size: f32) -> Self {
        let r = self.get_style().radius;
        self.get_style().radius =
            CornerRadius::new(r.top_left(), r.top_right(), r.bottom_right(), size)
                .with_smoothing(r.smoothing());
        self
    }

    // ---- uniform radius ----

    // ---- radius: theme-driven shortcuts ----
    //
    // Every shortcut resolves `radius.<key>` from the active theme,
    // falling back to Tailwind's pixel value. The "bare" variants
    // (`rounded`, `rounded_t`, …) look up `radius.default` so they can
    // be overridden independently of the sized variants.

    // ---- uniform radius ----

    /// No rounding — `0px`. Resolves theme `radius.none`.
    fn rounded_none(self) -> Self {
        self.radius(active_theme().radius_or("none", 0.0))
    }
    /// Extra-small — `2px`. Resolves theme `radius.xs`.
    fn rounded_xs(self) -> Self {
        self.radius(active_theme().radius_or("xs", 2.0))
    }
    /// Small — `4px`. Resolves theme `radius.sm`.
    fn rounded_sm(self) -> Self {
        self.radius(active_theme().radius_or("sm", 4.0))
    }
    /// Medium — `6px`. Resolves theme `radius.md`.
    fn rounded_md(self) -> Self {
        self.radius(active_theme().radius_or("md", 6.0))
    }
    /// Large — `8px`. Resolves theme `radius.lg`.
    fn rounded_lg(self) -> Self {
        self.radius(active_theme().radius_or("lg", 8.0))
    }
    /// Extra-large — `12px`. Resolves theme `radius.xl`.
    fn rounded_xl(self) -> Self {
        self.radius(active_theme().radius_or("xl", 12.0))
    }
    /// 2× extra-large — `16px`. Resolves theme `radius.2xl`.
    fn rounded_2xl(self) -> Self {
        self.radius(active_theme().radius_or("2xl", 16.0))
    }
    /// 3× extra-large — `24px`. Resolves theme `radius.3xl`.
    fn rounded_3xl(self) -> Self {
        self.radius(active_theme().radius_or("3xl", 24.0))
    }
    /// 4× extra-large — `32px`. Resolves theme `radius.4xl`.
    fn rounded_4xl(self) -> Self {
        self.radius(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill / circle — `9999px`. Resolves theme `radius.full`.
    fn rounded_full(self) -> Self {
        self.radius(active_theme().radius_or("full", 9999.0))
    }
    /// Round all corners with the theme's default radius.
    fn rounded(self) -> Self {
        self.radius(active_theme().radius_or("default", 4.0))
    }

    // ---- top edge (rounded-t-*) ----

    /// Round the top edge with the theme's default radius.
    fn rounded_t(self) -> Self {
        self.radius_t(active_theme().radius_or("default", 4.0))
    }
    /// No rounding on the top edge. Resolves theme `radius.none`.
    fn rounded_t_none(self) -> Self {
        self.radius_t(active_theme().radius_or("none", 0.0))
    }
    /// `2px` on the top edge. Resolves theme `radius.xs`.
    fn rounded_t_xs(self) -> Self {
        self.radius_t(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` on the top edge. Resolves theme `radius.sm`.
    fn rounded_t_sm(self) -> Self {
        self.radius_t(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` on the top edge. Resolves theme `radius.md`.
    fn rounded_t_md(self) -> Self {
        self.radius_t(active_theme().radius_or("md", 6.0))
    }
    /// `8px` on the top edge. Resolves theme `radius.lg`.
    fn rounded_t_lg(self) -> Self {
        self.radius_t(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` on the top edge. Resolves theme `radius.xl`.
    fn rounded_t_xl(self) -> Self {
        self.radius_t(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` on the top edge. Resolves theme `radius.2xl`.
    fn rounded_t_2xl(self) -> Self {
        self.radius_t(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` on the top edge. Resolves theme `radius.3xl`.
    fn rounded_t_3xl(self) -> Self {
        self.radius_t(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` on the top edge. Resolves theme `radius.4xl`.
    fn rounded_t_4xl(self) -> Self {
        self.radius_t(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill on the top edge. Resolves theme `radius.full`.
    fn rounded_t_full(self) -> Self {
        self.radius_t(active_theme().radius_or("full", 9999.0))
    }

    // ---- right edge (rounded-r-*) ----

    /// Round the right edge with the theme's default radius.
    fn rounded_r(self) -> Self {
        self.radius_r(active_theme().radius_or("default", 4.0))
    }
    /// No rounding on the right edge. Resolves theme `radius.none`.
    fn rounded_r_none(self) -> Self {
        self.radius_r(active_theme().radius_or("none", 0.0))
    }
    /// `2px` on the right edge. Resolves theme `radius.xs`.
    fn rounded_r_xs(self) -> Self {
        self.radius_r(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` on the right edge. Resolves theme `radius.sm`.
    fn rounded_r_sm(self) -> Self {
        self.radius_r(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` on the right edge. Resolves theme `radius.md`.
    fn rounded_r_md(self) -> Self {
        self.radius_r(active_theme().radius_or("md", 6.0))
    }
    /// `8px` on the right edge. Resolves theme `radius.lg`.
    fn rounded_r_lg(self) -> Self {
        self.radius_r(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` on the right edge. Resolves theme `radius.xl`.
    fn rounded_r_xl(self) -> Self {
        self.radius_r(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` on the right edge. Resolves theme `radius.2xl`.
    fn rounded_r_2xl(self) -> Self {
        self.radius_r(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` on the right edge. Resolves theme `radius.3xl`.
    fn rounded_r_3xl(self) -> Self {
        self.radius_r(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` on the right edge. Resolves theme `radius.4xl`.
    fn rounded_r_4xl(self) -> Self {
        self.radius_r(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill on the right edge. Resolves theme `radius.full`.
    fn rounded_r_full(self) -> Self {
        self.radius_r(active_theme().radius_or("full", 9999.0))
    }

    // ---- bottom edge (rounded-b-*) ----

    /// Round the bottom edge with the theme's default radius.
    fn rounded_b(self) -> Self {
        self.radius_b(active_theme().radius_or("default", 4.0))
    }
    /// No rounding on the bottom edge. Resolves theme `radius.none`.
    fn rounded_b_none(self) -> Self {
        self.radius_b(active_theme().radius_or("none", 0.0))
    }
    /// `2px` on the bottom edge. Resolves theme `radius.xs`.
    fn rounded_b_xs(self) -> Self {
        self.radius_b(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` on the bottom edge. Resolves theme `radius.sm`.
    fn rounded_b_sm(self) -> Self {
        self.radius_b(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` on the bottom edge. Resolves theme `radius.md`.
    fn rounded_b_md(self) -> Self {
        self.radius_b(active_theme().radius_or("md", 6.0))
    }
    /// `8px` on the bottom edge. Resolves theme `radius.lg`.
    fn rounded_b_lg(self) -> Self {
        self.radius_b(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` on the bottom edge. Resolves theme `radius.xl`.
    fn rounded_b_xl(self) -> Self {
        self.radius_b(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` on the bottom edge. Resolves theme `radius.2xl`.
    fn rounded_b_2xl(self) -> Self {
        self.radius_b(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` on the bottom edge. Resolves theme `radius.3xl`.
    fn rounded_b_3xl(self) -> Self {
        self.radius_b(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` on the bottom edge. Resolves theme `radius.4xl`.
    fn rounded_b_4xl(self) -> Self {
        self.radius_b(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill on the bottom edge. Resolves theme `radius.full`.
    fn rounded_b_full(self) -> Self {
        self.radius_b(active_theme().radius_or("full", 9999.0))
    }

    // ---- left edge (rounded-l-*) ----

    /// Round the left edge with the theme's default radius.
    fn rounded_l(self) -> Self {
        self.radius_l(active_theme().radius_or("default", 4.0))
    }
    /// No rounding on the left edge. Resolves theme `radius.none`.
    fn rounded_l_none(self) -> Self {
        self.radius_l(active_theme().radius_or("none", 0.0))
    }
    /// `2px` on the left edge. Resolves theme `radius.xs`.
    fn rounded_l_xs(self) -> Self {
        self.radius_l(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` on the left edge. Resolves theme `radius.sm`.
    fn rounded_l_sm(self) -> Self {
        self.radius_l(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` on the left edge. Resolves theme `radius.md`.
    fn rounded_l_md(self) -> Self {
        self.radius_l(active_theme().radius_or("md", 6.0))
    }
    /// `8px` on the left edge. Resolves theme `radius.lg`.
    fn rounded_l_lg(self) -> Self {
        self.radius_l(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` on the left edge. Resolves theme `radius.xl`.
    fn rounded_l_xl(self) -> Self {
        self.radius_l(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` on the left edge. Resolves theme `radius.2xl`.
    fn rounded_l_2xl(self) -> Self {
        self.radius_l(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` on the left edge. Resolves theme `radius.3xl`.
    fn rounded_l_3xl(self) -> Self {
        self.radius_l(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` on the left edge. Resolves theme `radius.4xl`.
    fn rounded_l_4xl(self) -> Self {
        self.radius_l(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill on the left edge. Resolves theme `radius.full`.
    fn rounded_l_full(self) -> Self {
        self.radius_l(active_theme().radius_or("full", 9999.0))
    }

    // ---- top-left corner (rounded-tl-*) ----

    /// Round the top-left corner with the theme's default radius.
    fn rounded_tl(self) -> Self {
        self.radius_tl(active_theme().radius_or("default", 4.0))
    }
    /// No rounding in the top-left corner. Resolves theme `radius.none`.
    fn rounded_tl_none(self) -> Self {
        self.radius_tl(active_theme().radius_or("none", 0.0))
    }
    /// `2px` in the top-left corner. Resolves theme `radius.xs`.
    fn rounded_tl_xs(self) -> Self {
        self.radius_tl(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` in the top-left corner. Resolves theme `radius.sm`.
    fn rounded_tl_sm(self) -> Self {
        self.radius_tl(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` in the top-left corner. Resolves theme `radius.md`.
    fn rounded_tl_md(self) -> Self {
        self.radius_tl(active_theme().radius_or("md", 6.0))
    }
    /// `8px` in the top-left corner. Resolves theme `radius.lg`.
    fn rounded_tl_lg(self) -> Self {
        self.radius_tl(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` in the top-left corner. Resolves theme `radius.xl`.
    fn rounded_tl_xl(self) -> Self {
        self.radius_tl(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` in the top-left corner. Resolves theme `radius.2xl`.
    fn rounded_tl_2xl(self) -> Self {
        self.radius_tl(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` in the top-left corner. Resolves theme `radius.3xl`.
    fn rounded_tl_3xl(self) -> Self {
        self.radius_tl(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` in the top-left corner. Resolves theme `radius.4xl`.
    fn rounded_tl_4xl(self) -> Self {
        self.radius_tl(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill in the top-left corner. Resolves theme `radius.full`.
    fn rounded_tl_full(self) -> Self {
        self.radius_tl(active_theme().radius_or("full", 9999.0))
    }

    // ---- top-right corner (rounded-tr-*) ----

    /// Round the top-right corner with the theme's default radius.
    fn rounded_tr(self) -> Self {
        self.radius_tr(active_theme().radius_or("default", 4.0))
    }
    /// No rounding in the top-right corner. Resolves theme `radius.none`.
    fn rounded_tr_none(self) -> Self {
        self.radius_tr(active_theme().radius_or("none", 0.0))
    }
    /// `2px` in the top-right corner. Resolves theme `radius.xs`.
    fn rounded_tr_xs(self) -> Self {
        self.radius_tr(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` in the top-right corner. Resolves theme `radius.sm`.
    fn rounded_tr_sm(self) -> Self {
        self.radius_tr(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` in the top-right corner. Resolves theme `radius.md`.
    fn rounded_tr_md(self) -> Self {
        self.radius_tr(active_theme().radius_or("md", 6.0))
    }
    /// `8px` in the top-right corner. Resolves theme `radius.lg`.
    fn rounded_tr_lg(self) -> Self {
        self.radius_tr(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` in the top-right corner. Resolves theme `radius.xl`.
    fn rounded_tr_xl(self) -> Self {
        self.radius_tr(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` in the top-right corner. Resolves theme `radius.2xl`.
    fn rounded_tr_2xl(self) -> Self {
        self.radius_tr(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` in the top-right corner. Resolves theme `radius.3xl`.
    fn rounded_tr_3xl(self) -> Self {
        self.radius_tr(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` in the top-right corner. Resolves theme `radius.4xl`.
    fn rounded_tr_4xl(self) -> Self {
        self.radius_tr(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill in the top-right corner. Resolves theme `radius.full`.
    fn rounded_tr_full(self) -> Self {
        self.radius_tr(active_theme().radius_or("full", 9999.0))
    }

    // ---- bottom-right corner (rounded-br-*) ----

    /// Round the bottom-right corner with the theme's default radius.
    fn rounded_br(self) -> Self {
        self.radius_br(active_theme().radius_or("default", 4.0))
    }
    /// No rounding in the bottom-right corner. Resolves theme `radius.none`.
    fn rounded_br_none(self) -> Self {
        self.radius_br(active_theme().radius_or("none", 0.0))
    }
    /// `2px` in the bottom-right corner. Resolves theme `radius.xs`.
    fn rounded_br_xs(self) -> Self {
        self.radius_br(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` in the bottom-right corner. Resolves theme `radius.sm`.
    fn rounded_br_sm(self) -> Self {
        self.radius_br(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` in the bottom-right corner. Resolves theme `radius.md`.
    fn rounded_br_md(self) -> Self {
        self.radius_br(active_theme().radius_or("md", 6.0))
    }
    /// `8px` in the bottom-right corner. Resolves theme `radius.lg`.
    fn rounded_br_lg(self) -> Self {
        self.radius_br(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` in the bottom-right corner. Resolves theme `radius.xl`.
    fn rounded_br_xl(self) -> Self {
        self.radius_br(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` in the bottom-right corner. Resolves theme `radius.2xl`.
    fn rounded_br_2xl(self) -> Self {
        self.radius_br(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` in the bottom-right corner. Resolves theme `radius.3xl`.
    fn rounded_br_3xl(self) -> Self {
        self.radius_br(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` in the bottom-right corner. Resolves theme `radius.4xl`.
    fn rounded_br_4xl(self) -> Self {
        self.radius_br(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill in the bottom-right corner. Resolves theme `radius.full`.
    fn rounded_br_full(self) -> Self {
        self.radius_br(active_theme().radius_or("full", 9999.0))
    }

    // ---- bottom-left corner (rounded-bl-*) ----

    /// Round the bottom-left corner with the theme's default radius.
    fn rounded_bl(self) -> Self {
        self.radius_bl(active_theme().radius_or("default", 4.0))
    }
    /// No rounding in the bottom-left corner. Resolves theme `radius.none`.
    fn rounded_bl_none(self) -> Self {
        self.radius_bl(active_theme().radius_or("none", 0.0))
    }
    /// `2px` in the bottom-left corner. Resolves theme `radius.xs`.
    fn rounded_bl_xs(self) -> Self {
        self.radius_bl(active_theme().radius_or("xs", 2.0))
    }
    /// `4px` in the bottom-left corner. Resolves theme `radius.sm`.
    fn rounded_bl_sm(self) -> Self {
        self.radius_bl(active_theme().radius_or("sm", 4.0))
    }
    /// `6px` in the bottom-left corner. Resolves theme `radius.md`.
    fn rounded_bl_md(self) -> Self {
        self.radius_bl(active_theme().radius_or("md", 6.0))
    }
    /// `8px` in the bottom-left corner. Resolves theme `radius.lg`.
    fn rounded_bl_lg(self) -> Self {
        self.radius_bl(active_theme().radius_or("lg", 8.0))
    }
    /// `12px` in the bottom-left corner. Resolves theme `radius.xl`.
    fn rounded_bl_xl(self) -> Self {
        self.radius_bl(active_theme().radius_or("xl", 12.0))
    }
    /// `16px` in the bottom-left corner. Resolves theme `radius.2xl`.
    fn rounded_bl_2xl(self) -> Self {
        self.radius_bl(active_theme().radius_or("2xl", 16.0))
    }
    /// `24px` in the bottom-left corner. Resolves theme `radius.3xl`.
    fn rounded_bl_3xl(self) -> Self {
        self.radius_bl(active_theme().radius_or("3xl", 24.0))
    }
    /// `32px` in the bottom-left corner. Resolves theme `radius.4xl`.
    fn rounded_bl_4xl(self) -> Self {
        self.radius_bl(active_theme().radius_or("4xl", 32.0))
    }
    /// Pill in the bottom-left corner. Resolves theme `radius.full`.
    fn rounded_bl_full(self) -> Self {
        self.radius_bl(active_theme().radius_or("full", 9999.0))
    }

    // ---- box shadows ----

    /// Append a single box shadow. Layers accumulate, so chaining
    /// `.shadow(a).shadow(b)` produces a two-layer shadow (which is what
    /// the Tailwind size presets do internally).
    ///
    /// Accepts anything `Into<Shadow>` — a `Shadow` from the builder, or
    /// a `(x, y, blur, spread, color)` tuple:
    ///
    /// ```
    /// div().shadow((0.0, 4.0, 8.0, 0.0, black().opacity(0.1)))
    /// ```
    fn shadow(mut self, shadow: impl Into<Shadow>) -> Self {
        self.get_style().shadows.push(shadow.into());
        self
    }

    /// Remove every box shadow. Tailwind's `shadow-none`.
    fn shadow_none(mut self) -> Self {
        self.get_style().shadows.clear();
        self
    }

    /// Extra-extra-small — `0 1px rgb(0 0 0 / 0.05)`. Tailwind's `shadow-2xs`.
    fn shadow_2xs(mut self) -> Self {
        self.get_style().shadows = vec![Shadow::new().y(1.0).color(black().opacity(0.05))];
        self
    }

    /// Extra-small — `0 1px 2px 0 rgb(0 0 0 / 0.05)`. Tailwind's `shadow-xs`.
    fn shadow_xs(mut self) -> Self {
        self.get_style().shadows =
            vec![Shadow::new().y(1.0).blur(2.0).color(black().opacity(0.05))];
        self
    }

    /// Small — two layers, `0 1px 3px` + `0 1px 2px -1px`. Tailwind's `shadow-sm`.
    fn shadow_sm(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new().y(1.0).blur(3.0).color(black().opacity(0.1)),
            Shadow::new()
                .y(1.0)
                .blur(2.0)
                .spread(-1.0)
                .color(black().opacity(0.1)),
        ];
        self
    }

    /// Medium — two layers, `0 4px 6px -1px` + `0 2px 4px -2px`. Tailwind's `shadow-md`.
    fn shadow_md(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new()
                .y(4.0)
                .blur(6.0)
                .spread(-1.0)
                .color(black().opacity(0.1)),
            Shadow::new()
                .y(2.0)
                .blur(4.0)
                .spread(-2.0)
                .color(black().opacity(0.1)),
        ];
        self
    }

    /// Large — two layers, `0 10px 15px -3px` + `0 4px 6px -4px`. Tailwind's `shadow-lg`.
    fn shadow_lg(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new()
                .y(10.0)
                .blur(15.0)
                .spread(-3.0)
                .color(black().opacity(0.1)),
            Shadow::new()
                .y(4.0)
                .blur(6.0)
                .spread(-4.0)
                .color(black().opacity(0.1)),
        ];
        self
    }

    /// Extra-large — two layers, `0 20px 25px -5px` + `0 8px 10px -6px`. Tailwind's `shadow-xl`.
    fn shadow_xl(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new()
                .y(20.0)
                .blur(25.0)
                .spread(-5.0)
                .color(black().opacity(0.1)),
            Shadow::new()
                .y(8.0)
                .blur(10.0)
                .spread(-6.0)
                .color(black().opacity(0.1)),
        ];
        self
    }

    /// 2× extra-large — `0 25px 50px -12px rgb(0 0 0 / 0.25)`. Tailwind's `shadow-2xl`.
    fn shadow_2xl(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new()
                .y(25.0)
                .blur(50.0)
                .spread(-12.0)
                .color(black().opacity(0.25)),
        ];
        self
    }

    /// Inset shadow — `inset 0 2px 4px 0 rgb(0 0 0 / 0.05)`. Tailwind's `shadow-inner`.
    fn shadow_inner(mut self) -> Self {
        self.get_style().shadows = vec![
            Shadow::new()
                .y(2.0)
                .blur(4.0)
                .color(black().opacity(0.05))
                .inset(),
        ];
        self
    }

    /// Truncate overflowing content with an ellipsis. Shorthand for [`text_ellipsis`](Self::text_ellipsis) + [`overflow_clip`](Self::overflow_clip). Tailwind's `truncate`.
    fn truncate(self) -> Self {
        self.text_ellipsis().overflow_clip()
    }

    // ================================ layout ================================
    //
    // These mutate `Style`'s layout fields, not `LayoutData`. Your
    // element-build step is responsible for copying `Style::*` into
    // `LayoutData` before layout runs.

    // ---- flex direction / alignment / position ----
    //
    // Requires `Alignment`, `Direction` and `Position` in scope (see the import
    // line in the reply).

    /// Lay children out left-to-right. Tailwind's `flex-row`.
    fn flex_row(self) -> Self {
        self.direction(Direction::Horizontal)
    }
    /// Lay children out top-to-bottom. Tailwind's `flex-col`.
    fn flex_col(self) -> Self {
        self.direction(Direction::Vertical)
    }
    /// Sets the layout direction of the children.
    fn direction(mut self, direction: Direction) -> Self {
        self.get_style().direction = direction;
        self
    }
    /// Align children on the main axis (start). Tailwind's `justify-start`.
    fn justify_start(self) -> Self {
        self.main_align(Alignment::Start)
    }
    /// Align children on the main axis (center). Tailwind's `justify-center`.
    fn justify_center(self) -> Self {
        self.main_align(Alignment::Center)
    }
    /// Align children on the main axis (end). Tailwind's `justify-end`.
    fn justify_end(self) -> Self {
        self.main_align(Alignment::End)
    }
    /// Align children on the main axis (between). Tailwind's `justify-between`.
    fn justify_between(self) -> Self {
        self.main_align(Alignment::SpaceBetween)
    }
    /// Align children on the main axis (around). Tailwind's `justify-around`.
    fn justify_around(self) -> Self {
        self.main_align(Alignment::SpaceAround)
    }
    /// Align children on the main axis (evenly). Tailwind's `justify-evenly`.
    fn justify_evenly(self) -> Self {
        self.main_align(Alignment::SpaceEvenly)
    }
    /// Sets how children are aligned along the main axis.
    fn main_align(mut self, align: Alignment) -> Self {
        self.get_style().main_align = align;
        self
    }
    /// Align children on the cross axis (start). Tailwind's `items-start`.
    fn items_start(self) -> Self {
        self.cross_align(Alignment::Start)
    }
    /// Align children on the cross axis (center). Tailwind's `items-center`.
    fn items_center(self) -> Self {
        self.cross_align(Alignment::Center)
    }
    /// Align children on the cross axis (end). Tailwind's `items-end`.
    fn items_end(self) -> Self {
        self.cross_align(Alignment::End)
    }
    /// Sets how children are aligned along the cross axis.
    fn cross_align(mut self, align: Alignment) -> Self {
        self.get_style().cross_align = align;
        self
    }
    /// Position the element absolutely. Tailwind's `absolute`.
    fn absolute(mut self) -> Self {
        self.get_style().position = Position::new_absolute();
        self
    }
    /// Position the element in the normal flow. Tailwind's `relative` / `static`.
    fn relative(mut self) -> Self {
        self.get_style().position = Position::new_stacked();
        self
    }

    // ---- padding ----

    /// Set padding on all four sides, in pixels.
    fn p(mut self, v: f32) -> Self {
        self.get_style().padding = Gaps::new_all(v);
        self
    }

    /// Set horizontal (left + right) padding in pixels, preserving the other sides.
    fn px(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), v, g.bottom(), v);
        self
    }

    /// Set vertical (top + bottom) padding in pixels, preserving the other sides.
    fn py(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(v, g.right(), v, g.left());
        self
    }

    /// Set top padding in pixels, preserving the other sides.
    fn pt(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(v, g.right(), g.bottom(), g.left());
        self
    }

    /// Set right padding in pixels, preserving the other sides.
    fn pr(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), v, g.bottom(), g.left());
        self
    }

    /// Set bottom padding in pixels, preserving the other sides.
    fn pb(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), g.right(), v, g.left());
        self
    }

    /// Set left padding in pixels, preserving the other sides.
    fn pl(mut self, v: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), g.right(), g.bottom(), v);
        self
    }

    // padding: `p_*` shortcuts (Tailwind `p-*`)
    /// `0px` padding on all sides. Tailwind's `p-0`.
    fn p_0(self) -> Self {
        self.p(0.0)
    }
    /// `2px` padding on all sides. Tailwind's `p-0.5`.
    fn p_0_p5(self) -> Self {
        self.p(2.0)
    }
    /// `4px` padding on all sides. Tailwind's `p-1`.
    fn p_1(self) -> Self {
        self.p(4.0)
    }
    /// `6px` padding on all sides. Tailwind's `p-1.5`.
    fn p_1_p5(self) -> Self {
        self.p(6.0)
    }
    /// `8px` padding on all sides. Tailwind's `p-2`.
    fn p_2(self) -> Self {
        self.p(8.0)
    }
    /// `10px` padding on all sides. Tailwind's `p-2.5`.
    fn p_2_p5(self) -> Self {
        self.p(10.0)
    }
    /// `12px` padding on all sides. Tailwind's `p-3`.
    fn p_3(self) -> Self {
        self.p(12.0)
    }
    /// `14px` padding on all sides. Tailwind's `p-3.5`.
    fn p_3_p5(self) -> Self {
        self.p(14.0)
    }
    /// `16px` padding on all sides. Tailwind's `p-4`.
    fn p_4(self) -> Self {
        self.p(16.0)
    }
    /// `20px` padding on all sides. Tailwind's `p-5`.
    fn p_5(self) -> Self {
        self.p(20.0)
    }
    /// `24px` padding on all sides. Tailwind's `p-6`.
    fn p_6(self) -> Self {
        self.p(24.0)
    }
    /// `28px` padding on all sides. Tailwind's `p-7`.
    fn p_7(self) -> Self {
        self.p(28.0)
    }
    /// `32px` padding on all sides. Tailwind's `p-8`.
    fn p_8(self) -> Self {
        self.p(32.0)
    }
    /// `36px` padding on all sides. Tailwind's `p-9`.
    fn p_9(self) -> Self {
        self.p(36.0)
    }
    /// `40px` padding on all sides. Tailwind's `p-10`.
    fn p_10(self) -> Self {
        self.p(40.0)
    }
    /// `44px` padding on all sides. Tailwind's `p-11`.
    fn p_11(self) -> Self {
        self.p(44.0)
    }
    /// `48px` padding on all sides. Tailwind's `p-12`.
    fn p_12(self) -> Self {
        self.p(48.0)
    }
    /// `56px` padding on all sides. Tailwind's `p-14`.
    fn p_14(self) -> Self {
        self.p(56.0)
    }
    /// `64px` padding on all sides. Tailwind's `p-16`.
    fn p_16(self) -> Self {
        self.p(64.0)
    }
    /// `80px` padding on all sides. Tailwind's `p-20`.
    fn p_20(self) -> Self {
        self.p(80.0)
    }
    /// `96px` padding on all sides. Tailwind's `p-24`.
    fn p_24(self) -> Self {
        self.p(96.0)
    }
    /// `112px` padding on all sides. Tailwind's `p-28`.
    fn p_28(self) -> Self {
        self.p(112.0)
    }
    /// `128px` padding on all sides. Tailwind's `p-32`.
    fn p_32(self) -> Self {
        self.p(128.0)
    }
    /// `144px` padding on all sides. Tailwind's `p-36`.
    fn p_36(self) -> Self {
        self.p(144.0)
    }
    /// `160px` padding on all sides. Tailwind's `p-40`.
    fn p_40(self) -> Self {
        self.p(160.0)
    }
    /// `176px` padding on all sides. Tailwind's `p-44`.
    fn p_44(self) -> Self {
        self.p(176.0)
    }
    /// `192px` padding on all sides. Tailwind's `p-48`.
    fn p_48(self) -> Self {
        self.p(192.0)
    }
    /// `208px` padding on all sides. Tailwind's `p-52`.
    fn p_52(self) -> Self {
        self.p(208.0)
    }
    /// `224px` padding on all sides. Tailwind's `p-56`.
    fn p_56(self) -> Self {
        self.p(224.0)
    }
    /// `240px` padding on all sides. Tailwind's `p-60`.
    fn p_60(self) -> Self {
        self.p(240.0)
    }
    /// `256px` padding on all sides. Tailwind's `p-64`.
    fn p_64(self) -> Self {
        self.p(256.0)
    }
    /// `288px` padding on all sides. Tailwind's `p-72`.
    fn p_72(self) -> Self {
        self.p(288.0)
    }
    /// `320px` padding on all sides. Tailwind's `p-80`.
    fn p_80(self) -> Self {
        self.p(320.0)
    }
    /// `384px` padding on all sides. Tailwind's `p-96`.
    fn p_96(self) -> Self {
        self.p(384.0)
    }
    /// `1px` padding on all sides. Tailwind's `p-px`.
    fn p_px(self) -> Self {
        self.p(1.0)
    }

    // padding: `px_*` shortcuts (Tailwind `px-*`)
    /// `0px` horizontal (left + right) padding. Tailwind's `px-0`.
    fn px_0(self) -> Self {
        self.px(0.0)
    }
    /// `2px` horizontal (left + right) padding. Tailwind's `px-0.5`.
    fn px_0_p5(self) -> Self {
        self.px(2.0)
    }
    /// `4px` horizontal (left + right) padding. Tailwind's `px-1`.
    fn px_1(self) -> Self {
        self.px(4.0)
    }
    /// `6px` horizontal (left + right) padding. Tailwind's `px-1.5`.
    fn px_1_p5(self) -> Self {
        self.px(6.0)
    }
    /// `8px` horizontal (left + right) padding. Tailwind's `px-2`.
    fn px_2(self) -> Self {
        self.px(8.0)
    }
    /// `10px` horizontal (left + right) padding. Tailwind's `px-2.5`.
    fn px_2_p5(self) -> Self {
        self.px(10.0)
    }
    /// `12px` horizontal (left + right) padding. Tailwind's `px-3`.
    fn px_3(self) -> Self {
        self.px(12.0)
    }
    /// `14px` horizontal (left + right) padding. Tailwind's `px-3.5`.
    fn px_3_p5(self) -> Self {
        self.px(14.0)
    }
    /// `16px` horizontal (left + right) padding. Tailwind's `px-4`.
    fn px_4(self) -> Self {
        self.px(16.0)
    }
    /// `20px` horizontal (left + right) padding. Tailwind's `px-5`.
    fn px_5(self) -> Self {
        self.px(20.0)
    }
    /// `24px` horizontal (left + right) padding. Tailwind's `px-6`.
    fn px_6(self) -> Self {
        self.px(24.0)
    }
    /// `28px` horizontal (left + right) padding. Tailwind's `px-7`.
    fn px_7(self) -> Self {
        self.px(28.0)
    }
    /// `32px` horizontal (left + right) padding. Tailwind's `px-8`.
    fn px_8(self) -> Self {
        self.px(32.0)
    }
    /// `36px` horizontal (left + right) padding. Tailwind's `px-9`.
    fn px_9(self) -> Self {
        self.px(36.0)
    }
    /// `40px` horizontal (left + right) padding. Tailwind's `px-10`.
    fn px_10(self) -> Self {
        self.px(40.0)
    }
    /// `44px` horizontal (left + right) padding. Tailwind's `px-11`.
    fn px_11(self) -> Self {
        self.px(44.0)
    }
    /// `48px` horizontal (left + right) padding. Tailwind's `px-12`.
    fn px_12(self) -> Self {
        self.px(48.0)
    }
    /// `56px` horizontal (left + right) padding. Tailwind's `px-14`.
    fn px_14(self) -> Self {
        self.px(56.0)
    }
    /// `64px` horizontal (left + right) padding. Tailwind's `px-16`.
    fn px_16(self) -> Self {
        self.px(64.0)
    }
    /// `80px` horizontal (left + right) padding. Tailwind's `px-20`.
    fn px_20(self) -> Self {
        self.px(80.0)
    }
    /// `96px` horizontal (left + right) padding. Tailwind's `px-24`.
    fn px_24(self) -> Self {
        self.px(96.0)
    }
    /// `112px` horizontal (left + right) padding. Tailwind's `px-28`.
    fn px_28(self) -> Self {
        self.px(112.0)
    }
    /// `128px` horizontal (left + right) padding. Tailwind's `px-32`.
    fn px_32(self) -> Self {
        self.px(128.0)
    }
    /// `144px` horizontal (left + right) padding. Tailwind's `px-36`.
    fn px_36(self) -> Self {
        self.px(144.0)
    }
    /// `160px` horizontal (left + right) padding. Tailwind's `px-40`.
    fn px_40(self) -> Self {
        self.px(160.0)
    }
    /// `176px` horizontal (left + right) padding. Tailwind's `px-44`.
    fn px_44(self) -> Self {
        self.px(176.0)
    }
    /// `192px` horizontal (left + right) padding. Tailwind's `px-48`.
    fn px_48(self) -> Self {
        self.px(192.0)
    }
    /// `208px` horizontal (left + right) padding. Tailwind's `px-52`.
    fn px_52(self) -> Self {
        self.px(208.0)
    }
    /// `224px` horizontal (left + right) padding. Tailwind's `px-56`.
    fn px_56(self) -> Self {
        self.px(224.0)
    }
    /// `240px` horizontal (left + right) padding. Tailwind's `px-60`.
    fn px_60(self) -> Self {
        self.px(240.0)
    }
    /// `256px` horizontal (left + right) padding. Tailwind's `px-64`.
    fn px_64(self) -> Self {
        self.px(256.0)
    }
    /// `288px` horizontal (left + right) padding. Tailwind's `px-72`.
    fn px_72(self) -> Self {
        self.px(288.0)
    }
    /// `320px` horizontal (left + right) padding. Tailwind's `px-80`.
    fn px_80(self) -> Self {
        self.px(320.0)
    }
    /// `384px` horizontal (left + right) padding. Tailwind's `px-96`.
    fn px_96(self) -> Self {
        self.px(384.0)
    }
    /// `1px` horizontal (left + right) padding. Tailwind's `px-px`.
    fn px_px(self) -> Self {
        self.px(1.0)
    }

    // padding: `py_*` shortcuts (Tailwind `py-*`)
    /// `0px` vertical (top + bottom) padding. Tailwind's `py-0`.
    fn py_0(self) -> Self {
        self.py(0.0)
    }
    /// `2px` vertical (top + bottom) padding. Tailwind's `py-0.5`.
    fn py_0_p5(self) -> Self {
        self.py(2.0)
    }
    /// `4px` vertical (top + bottom) padding. Tailwind's `py-1`.
    fn py_1(self) -> Self {
        self.py(4.0)
    }
    /// `6px` vertical (top + bottom) padding. Tailwind's `py-1.5`.
    fn py_1_p5(self) -> Self {
        self.py(6.0)
    }
    /// `8px` vertical (top + bottom) padding. Tailwind's `py-2`.
    fn py_2(self) -> Self {
        self.py(8.0)
    }
    /// `10px` vertical (top + bottom) padding. Tailwind's `py-2.5`.
    fn py_2_p5(self) -> Self {
        self.py(10.0)
    }
    /// `12px` vertical (top + bottom) padding. Tailwind's `py-3`.
    fn py_3(self) -> Self {
        self.py(12.0)
    }
    /// `14px` vertical (top + bottom) padding. Tailwind's `py-3.5`.
    fn py_3_p5(self) -> Self {
        self.py(14.0)
    }
    /// `16px` vertical (top + bottom) padding. Tailwind's `py-4`.
    fn py_4(self) -> Self {
        self.py(16.0)
    }
    /// `20px` vertical (top + bottom) padding. Tailwind's `py-5`.
    fn py_5(self) -> Self {
        self.py(20.0)
    }
    /// `24px` vertical (top + bottom) padding. Tailwind's `py-6`.
    fn py_6(self) -> Self {
        self.py(24.0)
    }
    /// `28px` vertical (top + bottom) padding. Tailwind's `py-7`.
    fn py_7(self) -> Self {
        self.py(28.0)
    }
    /// `32px` vertical (top + bottom) padding. Tailwind's `py-8`.
    fn py_8(self) -> Self {
        self.py(32.0)
    }
    /// `36px` vertical (top + bottom) padding. Tailwind's `py-9`.
    fn py_9(self) -> Self {
        self.py(36.0)
    }
    /// `40px` vertical (top + bottom) padding. Tailwind's `py-10`.
    fn py_10(self) -> Self {
        self.py(40.0)
    }
    /// `44px` vertical (top + bottom) padding. Tailwind's `py-11`.
    fn py_11(self) -> Self {
        self.py(44.0)
    }
    /// `48px` vertical (top + bottom) padding. Tailwind's `py-12`.
    fn py_12(self) -> Self {
        self.py(48.0)
    }
    /// `56px` vertical (top + bottom) padding. Tailwind's `py-14`.
    fn py_14(self) -> Self {
        self.py(56.0)
    }
    /// `64px` vertical (top + bottom) padding. Tailwind's `py-16`.
    fn py_16(self) -> Self {
        self.py(64.0)
    }
    /// `80px` vertical (top + bottom) padding. Tailwind's `py-20`.
    fn py_20(self) -> Self {
        self.py(80.0)
    }
    /// `96px` vertical (top + bottom) padding. Tailwind's `py-24`.
    fn py_24(self) -> Self {
        self.py(96.0)
    }
    /// `112px` vertical (top + bottom) padding. Tailwind's `py-28`.
    fn py_28(self) -> Self {
        self.py(112.0)
    }
    /// `128px` vertical (top + bottom) padding. Tailwind's `py-32`.
    fn py_32(self) -> Self {
        self.py(128.0)
    }
    /// `144px` vertical (top + bottom) padding. Tailwind's `py-36`.
    fn py_36(self) -> Self {
        self.py(144.0)
    }
    /// `160px` vertical (top + bottom) padding. Tailwind's `py-40`.
    fn py_40(self) -> Self {
        self.py(160.0)
    }
    /// `176px` vertical (top + bottom) padding. Tailwind's `py-44`.
    fn py_44(self) -> Self {
        self.py(176.0)
    }
    /// `192px` vertical (top + bottom) padding. Tailwind's `py-48`.
    fn py_48(self) -> Self {
        self.py(192.0)
    }
    /// `208px` vertical (top + bottom) padding. Tailwind's `py-52`.
    fn py_52(self) -> Self {
        self.py(208.0)
    }
    /// `224px` vertical (top + bottom) padding. Tailwind's `py-56`.
    fn py_56(self) -> Self {
        self.py(224.0)
    }
    /// `240px` vertical (top + bottom) padding. Tailwind's `py-60`.
    fn py_60(self) -> Self {
        self.py(240.0)
    }
    /// `256px` vertical (top + bottom) padding. Tailwind's `py-64`.
    fn py_64(self) -> Self {
        self.py(256.0)
    }
    /// `288px` vertical (top + bottom) padding. Tailwind's `py-72`.
    fn py_72(self) -> Self {
        self.py(288.0)
    }
    /// `320px` vertical (top + bottom) padding. Tailwind's `py-80`.
    fn py_80(self) -> Self {
        self.py(320.0)
    }
    /// `384px` vertical (top + bottom) padding. Tailwind's `py-96`.
    fn py_96(self) -> Self {
        self.py(384.0)
    }
    /// `1px` vertical (top + bottom) padding. Tailwind's `py-px`.
    fn py_px(self) -> Self {
        self.py(1.0)
    }

    // padding: `pt_*` shortcuts (Tailwind `pt-*`)
    /// `0px` top padding. Tailwind's `pt-0`.
    fn pt_0(self) -> Self {
        self.pt(0.0)
    }
    /// `2px` top padding. Tailwind's `pt-0.5`.
    fn pt_0_p5(self) -> Self {
        self.pt(2.0)
    }
    /// `4px` top padding. Tailwind's `pt-1`.
    fn pt_1(self) -> Self {
        self.pt(4.0)
    }
    /// `6px` top padding. Tailwind's `pt-1.5`.
    fn pt_1_p5(self) -> Self {
        self.pt(6.0)
    }
    /// `8px` top padding. Tailwind's `pt-2`.
    fn pt_2(self) -> Self {
        self.pt(8.0)
    }
    /// `10px` top padding. Tailwind's `pt-2.5`.
    fn pt_2_p5(self) -> Self {
        self.pt(10.0)
    }
    /// `12px` top padding. Tailwind's `pt-3`.
    fn pt_3(self) -> Self {
        self.pt(12.0)
    }
    /// `14px` top padding. Tailwind's `pt-3.5`.
    fn pt_3_p5(self) -> Self {
        self.pt(14.0)
    }
    /// `16px` top padding. Tailwind's `pt-4`.
    fn pt_4(self) -> Self {
        self.pt(16.0)
    }
    /// `20px` top padding. Tailwind's `pt-5`.
    fn pt_5(self) -> Self {
        self.pt(20.0)
    }
    /// `24px` top padding. Tailwind's `pt-6`.
    fn pt_6(self) -> Self {
        self.pt(24.0)
    }
    /// `28px` top padding. Tailwind's `pt-7`.
    fn pt_7(self) -> Self {
        self.pt(28.0)
    }
    /// `32px` top padding. Tailwind's `pt-8`.
    fn pt_8(self) -> Self {
        self.pt(32.0)
    }
    /// `36px` top padding. Tailwind's `pt-9`.
    fn pt_9(self) -> Self {
        self.pt(36.0)
    }
    /// `40px` top padding. Tailwind's `pt-10`.
    fn pt_10(self) -> Self {
        self.pt(40.0)
    }
    /// `44px` top padding. Tailwind's `pt-11`.
    fn pt_11(self) -> Self {
        self.pt(44.0)
    }
    /// `48px` top padding. Tailwind's `pt-12`.
    fn pt_12(self) -> Self {
        self.pt(48.0)
    }
    /// `56px` top padding. Tailwind's `pt-14`.
    fn pt_14(self) -> Self {
        self.pt(56.0)
    }
    /// `64px` top padding. Tailwind's `pt-16`.
    fn pt_16(self) -> Self {
        self.pt(64.0)
    }
    /// `80px` top padding. Tailwind's `pt-20`.
    fn pt_20(self) -> Self {
        self.pt(80.0)
    }
    /// `96px` top padding. Tailwind's `pt-24`.
    fn pt_24(self) -> Self {
        self.pt(96.0)
    }
    /// `112px` top padding. Tailwind's `pt-28`.
    fn pt_28(self) -> Self {
        self.pt(112.0)
    }
    /// `128px` top padding. Tailwind's `pt-32`.
    fn pt_32(self) -> Self {
        self.pt(128.0)
    }
    /// `144px` top padding. Tailwind's `pt-36`.
    fn pt_36(self) -> Self {
        self.pt(144.0)
    }
    /// `160px` top padding. Tailwind's `pt-40`.
    fn pt_40(self) -> Self {
        self.pt(160.0)
    }
    /// `176px` top padding. Tailwind's `pt-44`.
    fn pt_44(self) -> Self {
        self.pt(176.0)
    }
    /// `192px` top padding. Tailwind's `pt-48`.
    fn pt_48(self) -> Self {
        self.pt(192.0)
    }
    /// `208px` top padding. Tailwind's `pt-52`.
    fn pt_52(self) -> Self {
        self.pt(208.0)
    }
    /// `224px` top padding. Tailwind's `pt-56`.
    fn pt_56(self) -> Self {
        self.pt(224.0)
    }
    /// `240px` top padding. Tailwind's `pt-60`.
    fn pt_60(self) -> Self {
        self.pt(240.0)
    }
    /// `256px` top padding. Tailwind's `pt-64`.
    fn pt_64(self) -> Self {
        self.pt(256.0)
    }
    /// `288px` top padding. Tailwind's `pt-72`.
    fn pt_72(self) -> Self {
        self.pt(288.0)
    }
    /// `320px` top padding. Tailwind's `pt-80`.
    fn pt_80(self) -> Self {
        self.pt(320.0)
    }
    /// `384px` top padding. Tailwind's `pt-96`.
    fn pt_96(self) -> Self {
        self.pt(384.0)
    }
    /// `1px` top padding. Tailwind's `pt-px`.
    fn pt_px(self) -> Self {
        self.pt(1.0)
    }

    // padding: `pr_*` shortcuts (Tailwind `pr-*`)
    /// `0px` right padding. Tailwind's `pr-0`.
    fn pr_0(self) -> Self {
        self.pr(0.0)
    }
    /// `2px` right padding. Tailwind's `pr-0.5`.
    fn pr_0_p5(self) -> Self {
        self.pr(2.0)
    }
    /// `4px` right padding. Tailwind's `pr-1`.
    fn pr_1(self) -> Self {
        self.pr(4.0)
    }
    /// `6px` right padding. Tailwind's `pr-1.5`.
    fn pr_1_p5(self) -> Self {
        self.pr(6.0)
    }
    /// `8px` right padding. Tailwind's `pr-2`.
    fn pr_2(self) -> Self {
        self.pr(8.0)
    }
    /// `10px` right padding. Tailwind's `pr-2.5`.
    fn pr_2_p5(self) -> Self {
        self.pr(10.0)
    }
    /// `12px` right padding. Tailwind's `pr-3`.
    fn pr_3(self) -> Self {
        self.pr(12.0)
    }
    /// `14px` right padding. Tailwind's `pr-3.5`.
    fn pr_3_p5(self) -> Self {
        self.pr(14.0)
    }
    /// `16px` right padding. Tailwind's `pr-4`.
    fn pr_4(self) -> Self {
        self.pr(16.0)
    }
    /// `20px` right padding. Tailwind's `pr-5`.
    fn pr_5(self) -> Self {
        self.pr(20.0)
    }
    /// `24px` right padding. Tailwind's `pr-6`.
    fn pr_6(self) -> Self {
        self.pr(24.0)
    }
    /// `28px` right padding. Tailwind's `pr-7`.
    fn pr_7(self) -> Self {
        self.pr(28.0)
    }
    /// `32px` right padding. Tailwind's `pr-8`.
    fn pr_8(self) -> Self {
        self.pr(32.0)
    }
    /// `36px` right padding. Tailwind's `pr-9`.
    fn pr_9(self) -> Self {
        self.pr(36.0)
    }
    /// `40px` right padding. Tailwind's `pr-10`.
    fn pr_10(self) -> Self {
        self.pr(40.0)
    }
    /// `44px` right padding. Tailwind's `pr-11`.
    fn pr_11(self) -> Self {
        self.pr(44.0)
    }
    /// `48px` right padding. Tailwind's `pr-12`.
    fn pr_12(self) -> Self {
        self.pr(48.0)
    }
    /// `56px` right padding. Tailwind's `pr-14`.
    fn pr_14(self) -> Self {
        self.pr(56.0)
    }
    /// `64px` right padding. Tailwind's `pr-16`.
    fn pr_16(self) -> Self {
        self.pr(64.0)
    }
    /// `80px` right padding. Tailwind's `pr-20`.
    fn pr_20(self) -> Self {
        self.pr(80.0)
    }
    /// `96px` right padding. Tailwind's `pr-24`.
    fn pr_24(self) -> Self {
        self.pr(96.0)
    }
    /// `112px` right padding. Tailwind's `pr-28`.
    fn pr_28(self) -> Self {
        self.pr(112.0)
    }
    /// `128px` right padding. Tailwind's `pr-32`.
    fn pr_32(self) -> Self {
        self.pr(128.0)
    }
    /// `144px` right padding. Tailwind's `pr-36`.
    fn pr_36(self) -> Self {
        self.pr(144.0)
    }
    /// `160px` right padding. Tailwind's `pr-40`.
    fn pr_40(self) -> Self {
        self.pr(160.0)
    }
    /// `176px` right padding. Tailwind's `pr-44`.
    fn pr_44(self) -> Self {
        self.pr(176.0)
    }
    /// `192px` right padding. Tailwind's `pr-48`.
    fn pr_48(self) -> Self {
        self.pr(192.0)
    }
    /// `208px` right padding. Tailwind's `pr-52`.
    fn pr_52(self) -> Self {
        self.pr(208.0)
    }
    /// `224px` right padding. Tailwind's `pr-56`.
    fn pr_56(self) -> Self {
        self.pr(224.0)
    }
    /// `240px` right padding. Tailwind's `pr-60`.
    fn pr_60(self) -> Self {
        self.pr(240.0)
    }
    /// `256px` right padding. Tailwind's `pr-64`.
    fn pr_64(self) -> Self {
        self.pr(256.0)
    }
    /// `288px` right padding. Tailwind's `pr-72`.
    fn pr_72(self) -> Self {
        self.pr(288.0)
    }
    /// `320px` right padding. Tailwind's `pr-80`.
    fn pr_80(self) -> Self {
        self.pr(320.0)
    }
    /// `384px` right padding. Tailwind's `pr-96`.
    fn pr_96(self) -> Self {
        self.pr(384.0)
    }
    /// `1px` right padding. Tailwind's `pr-px`.
    fn pr_px(self) -> Self {
        self.pr(1.0)
    }

    // padding: `pb_*` shortcuts (Tailwind `pb-*`)
    /// `0px` bottom padding. Tailwind's `pb-0`.
    fn pb_0(self) -> Self {
        self.pb(0.0)
    }
    /// `2px` bottom padding. Tailwind's `pb-0.5`.
    fn pb_0_p5(self) -> Self {
        self.pb(2.0)
    }
    /// `4px` bottom padding. Tailwind's `pb-1`.
    fn pb_1(self) -> Self {
        self.pb(4.0)
    }
    /// `6px` bottom padding. Tailwind's `pb-1.5`.
    fn pb_1_p5(self) -> Self {
        self.pb(6.0)
    }
    /// `8px` bottom padding. Tailwind's `pb-2`.
    fn pb_2(self) -> Self {
        self.pb(8.0)
    }
    /// `10px` bottom padding. Tailwind's `pb-2.5`.
    fn pb_2_p5(self) -> Self {
        self.pb(10.0)
    }
    /// `12px` bottom padding. Tailwind's `pb-3`.
    fn pb_3(self) -> Self {
        self.pb(12.0)
    }
    /// `14px` bottom padding. Tailwind's `pb-3.5`.
    fn pb_3_p5(self) -> Self {
        self.pb(14.0)
    }
    /// `16px` bottom padding. Tailwind's `pb-4`.
    fn pb_4(self) -> Self {
        self.pb(16.0)
    }
    /// `20px` bottom padding. Tailwind's `pb-5`.
    fn pb_5(self) -> Self {
        self.pb(20.0)
    }
    /// `24px` bottom padding. Tailwind's `pb-6`.
    fn pb_6(self) -> Self {
        self.pb(24.0)
    }
    /// `28px` bottom padding. Tailwind's `pb-7`.
    fn pb_7(self) -> Self {
        self.pb(28.0)
    }
    /// `32px` bottom padding. Tailwind's `pb-8`.
    fn pb_8(self) -> Self {
        self.pb(32.0)
    }
    /// `36px` bottom padding. Tailwind's `pb-9`.
    fn pb_9(self) -> Self {
        self.pb(36.0)
    }
    /// `40px` bottom padding. Tailwind's `pb-10`.
    fn pb_10(self) -> Self {
        self.pb(40.0)
    }
    /// `44px` bottom padding. Tailwind's `pb-11`.
    fn pb_11(self) -> Self {
        self.pb(44.0)
    }
    /// `48px` bottom padding. Tailwind's `pb-12`.
    fn pb_12(self) -> Self {
        self.pb(48.0)
    }
    /// `56px` bottom padding. Tailwind's `pb-14`.
    fn pb_14(self) -> Self {
        self.pb(56.0)
    }
    /// `64px` bottom padding. Tailwind's `pb-16`.
    fn pb_16(self) -> Self {
        self.pb(64.0)
    }
    /// `80px` bottom padding. Tailwind's `pb-20`.
    fn pb_20(self) -> Self {
        self.pb(80.0)
    }
    /// `96px` bottom padding. Tailwind's `pb-24`.
    fn pb_24(self) -> Self {
        self.pb(96.0)
    }
    /// `112px` bottom padding. Tailwind's `pb-28`.
    fn pb_28(self) -> Self {
        self.pb(112.0)
    }
    /// `128px` bottom padding. Tailwind's `pb-32`.
    fn pb_32(self) -> Self {
        self.pb(128.0)
    }
    /// `144px` bottom padding. Tailwind's `pb-36`.
    fn pb_36(self) -> Self {
        self.pb(144.0)
    }
    /// `160px` bottom padding. Tailwind's `pb-40`.
    fn pb_40(self) -> Self {
        self.pb(160.0)
    }
    /// `176px` bottom padding. Tailwind's `pb-44`.
    fn pb_44(self) -> Self {
        self.pb(176.0)
    }
    /// `192px` bottom padding. Tailwind's `pb-48`.
    fn pb_48(self) -> Self {
        self.pb(192.0)
    }
    /// `208px` bottom padding. Tailwind's `pb-52`.
    fn pb_52(self) -> Self {
        self.pb(208.0)
    }
    /// `224px` bottom padding. Tailwind's `pb-56`.
    fn pb_56(self) -> Self {
        self.pb(224.0)
    }
    /// `240px` bottom padding. Tailwind's `pb-60`.
    fn pb_60(self) -> Self {
        self.pb(240.0)
    }
    /// `256px` bottom padding. Tailwind's `pb-64`.
    fn pb_64(self) -> Self {
        self.pb(256.0)
    }
    /// `288px` bottom padding. Tailwind's `pb-72`.
    fn pb_72(self) -> Self {
        self.pb(288.0)
    }
    /// `320px` bottom padding. Tailwind's `pb-80`.
    fn pb_80(self) -> Self {
        self.pb(320.0)
    }
    /// `384px` bottom padding. Tailwind's `pb-96`.
    fn pb_96(self) -> Self {
        self.pb(384.0)
    }
    /// `1px` bottom padding. Tailwind's `pb-px`.
    fn pb_px(self) -> Self {
        self.pb(1.0)
    }

    // padding: `pl_*` shortcuts (Tailwind `pl-*`)
    /// `0px` left padding. Tailwind's `pl-0`.
    fn pl_0(self) -> Self {
        self.pl(0.0)
    }
    /// `2px` left padding. Tailwind's `pl-0.5`.
    fn pl_0_p5(self) -> Self {
        self.pl(2.0)
    }
    /// `4px` left padding. Tailwind's `pl-1`.
    fn pl_1(self) -> Self {
        self.pl(4.0)
    }
    /// `6px` left padding. Tailwind's `pl-1.5`.
    fn pl_1_p5(self) -> Self {
        self.pl(6.0)
    }
    /// `8px` left padding. Tailwind's `pl-2`.
    fn pl_2(self) -> Self {
        self.pl(8.0)
    }
    /// `10px` left padding. Tailwind's `pl-2.5`.
    fn pl_2_p5(self) -> Self {
        self.pl(10.0)
    }
    /// `12px` left padding. Tailwind's `pl-3`.
    fn pl_3(self) -> Self {
        self.pl(12.0)
    }
    /// `14px` left padding. Tailwind's `pl-3.5`.
    fn pl_3_p5(self) -> Self {
        self.pl(14.0)
    }
    /// `16px` left padding. Tailwind's `pl-4`.
    fn pl_4(self) -> Self {
        self.pl(16.0)
    }
    /// `20px` left padding. Tailwind's `pl-5`.
    fn pl_5(self) -> Self {
        self.pl(20.0)
    }
    /// `24px` left padding. Tailwind's `pl-6`.
    fn pl_6(self) -> Self {
        self.pl(24.0)
    }
    /// `28px` left padding. Tailwind's `pl-7`.
    fn pl_7(self) -> Self {
        self.pl(28.0)
    }
    /// `32px` left padding. Tailwind's `pl-8`.
    fn pl_8(self) -> Self {
        self.pl(32.0)
    }
    /// `36px` left padding. Tailwind's `pl-9`.
    fn pl_9(self) -> Self {
        self.pl(36.0)
    }
    /// `40px` left padding. Tailwind's `pl-10`.
    fn pl_10(self) -> Self {
        self.pl(40.0)
    }
    /// `44px` left padding. Tailwind's `pl-11`.
    fn pl_11(self) -> Self {
        self.pl(44.0)
    }
    /// `48px` left padding. Tailwind's `pl-12`.
    fn pl_12(self) -> Self {
        self.pl(48.0)
    }
    /// `56px` left padding. Tailwind's `pl-14`.
    fn pl_14(self) -> Self {
        self.pl(56.0)
    }
    /// `64px` left padding. Tailwind's `pl-16`.
    fn pl_16(self) -> Self {
        self.pl(64.0)
    }
    /// `80px` left padding. Tailwind's `pl-20`.
    fn pl_20(self) -> Self {
        self.pl(80.0)
    }
    /// `96px` left padding. Tailwind's `pl-24`.
    fn pl_24(self) -> Self {
        self.pl(96.0)
    }
    /// `112px` left padding. Tailwind's `pl-28`.
    fn pl_28(self) -> Self {
        self.pl(112.0)
    }
    /// `128px` left padding. Tailwind's `pl-32`.
    fn pl_32(self) -> Self {
        self.pl(128.0)
    }
    /// `144px` left padding. Tailwind's `pl-36`.
    fn pl_36(self) -> Self {
        self.pl(144.0)
    }
    /// `160px` left padding. Tailwind's `pl-40`.
    fn pl_40(self) -> Self {
        self.pl(160.0)
    }
    /// `176px` left padding. Tailwind's `pl-44`.
    fn pl_44(self) -> Self {
        self.pl(176.0)
    }
    /// `192px` left padding. Tailwind's `pl-48`.
    fn pl_48(self) -> Self {
        self.pl(192.0)
    }
    /// `208px` left padding. Tailwind's `pl-52`.
    fn pl_52(self) -> Self {
        self.pl(208.0)
    }
    /// `224px` left padding. Tailwind's `pl-56`.
    fn pl_56(self) -> Self {
        self.pl(224.0)
    }
    /// `240px` left padding. Tailwind's `pl-60`.
    fn pl_60(self) -> Self {
        self.pl(240.0)
    }
    /// `256px` left padding. Tailwind's `pl-64`.
    fn pl_64(self) -> Self {
        self.pl(256.0)
    }
    /// `288px` left padding. Tailwind's `pl-72`.
    fn pl_72(self) -> Self {
        self.pl(288.0)
    }
    /// `320px` left padding. Tailwind's `pl-80`.
    fn pl_80(self) -> Self {
        self.pl(320.0)
    }
    /// `384px` left padding. Tailwind's `pl-96`.
    fn pl_96(self) -> Self {
        self.pl(384.0)
    }
    /// `1px` left padding. Tailwind's `pl-px`.
    fn pl_px(self) -> Self {
        self.pl(1.0)
    }

    // ---- spacing: theme-driven named shortcuts ----
    //
    // Resolves `spacing.<key>` from the active theme, falling back to a
    // Tailwind-ish pixel scale. Use the numeric `p_1`, `p_2`, … shortcuts
    // for fixed-pixel values; use these for theme-driven ones.

    // ---- padding: uniform ----

    /// Theme `spacing.xs` (fallback 4px).
    fn p_xs(self) -> Self {
        self.p(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` (fallback 8px).
    fn p_sm(self) -> Self {
        self.p(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` (fallback 12px).
    fn p_md(self) -> Self {
        self.p(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` (fallback 16px).
    fn p_lg(self) -> Self {
        self.p(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` (fallback 20px).
    fn p_xl(self) -> Self {
        self.p(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` (fallback 24px).
    fn p_2xl(self) -> Self {
        self.p(active_theme().spacing_or("2xl", 24.0))
    }

    // ---- padding: horizontal / vertical ----

    /// Theme `spacing.xs` horizontal padding.
    fn px_xs(self) -> Self {
        self.px(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` horizontal padding.
    fn px_sm(self) -> Self {
        self.px(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` horizontal padding.
    fn px_md(self) -> Self {
        self.px(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` horizontal padding.
    fn px_lg(self) -> Self {
        self.px(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` horizontal padding.
    fn px_xl(self) -> Self {
        self.px(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` horizontal padding.
    fn px_2xl(self) -> Self {
        self.px(active_theme().spacing_or("2xl", 24.0))
    }

    /// Theme `spacing.xs` vertical padding.
    fn py_xs(self) -> Self {
        self.py(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` vertical padding.
    fn py_sm(self) -> Self {
        self.py(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` vertical padding.
    fn py_md(self) -> Self {
        self.py(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` vertical padding.
    fn py_lg(self) -> Self {
        self.py(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` vertical padding.
    fn py_xl(self) -> Self {
        self.py(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` vertical padding.
    fn py_2xl(self) -> Self {
        self.py(active_theme().spacing_or("2xl", 24.0))
    }

    // ---- margin: uniform ----

    /// Theme `spacing.xs` margin.
    fn m_xs(self) -> Self {
        self.m(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` margin.
    fn m_sm(self) -> Self {
        self.m(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` margin.
    fn m_md(self) -> Self {
        self.m(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` margin.
    fn m_lg(self) -> Self {
        self.m(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` margin.
    fn m_xl(self) -> Self {
        self.m(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` margin.
    fn m_2xl(self) -> Self {
        self.m(active_theme().spacing_or("2xl", 24.0))
    }

    // ---- margin: horizontal / vertical ----

    /// Theme `spacing.xs` horizontal margin.
    fn mx_xs(self) -> Self {
        self.mx(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` horizontal margin.
    fn mx_sm(self) -> Self {
        self.mx(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` horizontal margin.
    fn mx_md(self) -> Self {
        self.mx(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` horizontal margin.
    fn mx_lg(self) -> Self {
        self.mx(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` horizontal margin.
    fn mx_xl(self) -> Self {
        self.mx(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` horizontal margin.
    fn mx_2xl(self) -> Self {
        self.mx(active_theme().spacing_or("2xl", 24.0))
    }

    /// Theme `spacing.xs` vertical margin.
    fn my_xs(self) -> Self {
        self.my(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` vertical margin.
    fn my_sm(self) -> Self {
        self.my(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` vertical margin.
    fn my_md(self) -> Self {
        self.my(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` vertical margin.
    fn my_lg(self) -> Self {
        self.my(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` vertical margin.
    fn my_xl(self) -> Self {
        self.my(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` vertical margin.
    fn my_2xl(self) -> Self {
        self.my(active_theme().spacing_or("2xl", 24.0))
    }

    // ---- gap ----

    /// Theme `spacing.xs` gap.
    fn gap_xs(self) -> Self {
        self.gap(active_theme().spacing_or("xs", 4.0))
    }
    /// Theme `spacing.sm` gap.
    fn gap_sm(self) -> Self {
        self.gap(active_theme().spacing_or("sm", 8.0))
    }
    /// Theme `spacing.md` gap.
    fn gap_md(self) -> Self {
        self.gap(active_theme().spacing_or("md", 12.0))
    }
    /// Theme `spacing.lg` gap.
    fn gap_lg(self) -> Self {
        self.gap(active_theme().spacing_or("lg", 16.0))
    }
    /// Theme `spacing.xl` gap.
    fn gap_xl(self) -> Self {
        self.gap(active_theme().spacing_or("xl", 20.0))
    }
    /// Theme `spacing.2xl` gap.
    fn gap_2xl(self) -> Self {
        self.gap(active_theme().spacing_or("2xl", 24.0))
    }

    // ---- margin ----

    /// Set margin on all four sides, in pixels.
    fn m(mut self, v: f32) -> Self {
        self.get_style().margin = Gaps::new_all(v);
        self
    }

    /// Set horizontal (left + right) margin in pixels, preserving the other sides.
    fn mx(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), v, g.bottom(), v);
        self
    }

    /// Set vertical (top + bottom) margin in pixels, preserving the other sides.
    fn my(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(v, g.right(), v, g.left());
        self
    }

    /// Set top margin in pixels, preserving the other sides.
    fn mt(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(v, g.right(), g.bottom(), g.left());
        self
    }

    /// Set right margin in pixels, preserving the other sides.
    fn mr(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), v, g.bottom(), g.left());
        self
    }

    /// Set bottom margin in pixels, preserving the other sides.
    fn mb(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), g.right(), v, g.left());
        self
    }

    /// Set left margin in pixels, preserving the other sides.
    fn ml(mut self, v: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), g.right(), g.bottom(), v);
        self
    }

    // margin: `m_*` shortcuts (Tailwind `m-*`)
    /// `0px` margin on all sides. Tailwind's `m-0`.
    fn m_0(self) -> Self {
        self.m(0.0)
    }
    /// `2px` margin on all sides. Tailwind's `m-0.5`.
    fn m_0_p5(self) -> Self {
        self.m(2.0)
    }
    /// `4px` margin on all sides. Tailwind's `m-1`.
    fn m_1(self) -> Self {
        self.m(4.0)
    }
    /// `6px` margin on all sides. Tailwind's `m-1.5`.
    fn m_1_p5(self) -> Self {
        self.m(6.0)
    }
    /// `8px` margin on all sides. Tailwind's `m-2`.
    fn m_2(self) -> Self {
        self.m(8.0)
    }
    /// `10px` margin on all sides. Tailwind's `m-2.5`.
    fn m_2_p5(self) -> Self {
        self.m(10.0)
    }
    /// `12px` margin on all sides. Tailwind's `m-3`.
    fn m_3(self) -> Self {
        self.m(12.0)
    }
    /// `14px` margin on all sides. Tailwind's `m-3.5`.
    fn m_3_p5(self) -> Self {
        self.m(14.0)
    }
    /// `16px` margin on all sides. Tailwind's `m-4`.
    fn m_4(self) -> Self {
        self.m(16.0)
    }
    /// `20px` margin on all sides. Tailwind's `m-5`.
    fn m_5(self) -> Self {
        self.m(20.0)
    }
    /// `24px` margin on all sides. Tailwind's `m-6`.
    fn m_6(self) -> Self {
        self.m(24.0)
    }
    /// `28px` margin on all sides. Tailwind's `m-7`.
    fn m_7(self) -> Self {
        self.m(28.0)
    }
    /// `32px` margin on all sides. Tailwind's `m-8`.
    fn m_8(self) -> Self {
        self.m(32.0)
    }
    /// `36px` margin on all sides. Tailwind's `m-9`.
    fn m_9(self) -> Self {
        self.m(36.0)
    }
    /// `40px` margin on all sides. Tailwind's `m-10`.
    fn m_10(self) -> Self {
        self.m(40.0)
    }
    /// `44px` margin on all sides. Tailwind's `m-11`.
    fn m_11(self) -> Self {
        self.m(44.0)
    }
    /// `48px` margin on all sides. Tailwind's `m-12`.
    fn m_12(self) -> Self {
        self.m(48.0)
    }
    /// `56px` margin on all sides. Tailwind's `m-14`.
    fn m_14(self) -> Self {
        self.m(56.0)
    }
    /// `64px` margin on all sides. Tailwind's `m-16`.
    fn m_16(self) -> Self {
        self.m(64.0)
    }
    /// `80px` margin on all sides. Tailwind's `m-20`.
    fn m_20(self) -> Self {
        self.m(80.0)
    }
    /// `96px` margin on all sides. Tailwind's `m-24`.
    fn m_24(self) -> Self {
        self.m(96.0)
    }
    /// `112px` margin on all sides. Tailwind's `m-28`.
    fn m_28(self) -> Self {
        self.m(112.0)
    }
    /// `128px` margin on all sides. Tailwind's `m-32`.
    fn m_32(self) -> Self {
        self.m(128.0)
    }
    /// `144px` margin on all sides. Tailwind's `m-36`.
    fn m_36(self) -> Self {
        self.m(144.0)
    }
    /// `160px` margin on all sides. Tailwind's `m-40`.
    fn m_40(self) -> Self {
        self.m(160.0)
    }
    /// `176px` margin on all sides. Tailwind's `m-44`.
    fn m_44(self) -> Self {
        self.m(176.0)
    }
    /// `192px` margin on all sides. Tailwind's `m-48`.
    fn m_48(self) -> Self {
        self.m(192.0)
    }
    /// `208px` margin on all sides. Tailwind's `m-52`.
    fn m_52(self) -> Self {
        self.m(208.0)
    }
    /// `224px` margin on all sides. Tailwind's `m-56`.
    fn m_56(self) -> Self {
        self.m(224.0)
    }
    /// `240px` margin on all sides. Tailwind's `m-60`.
    fn m_60(self) -> Self {
        self.m(240.0)
    }
    /// `256px` margin on all sides. Tailwind's `m-64`.
    fn m_64(self) -> Self {
        self.m(256.0)
    }
    /// `288px` margin on all sides. Tailwind's `m-72`.
    fn m_72(self) -> Self {
        self.m(288.0)
    }
    /// `320px` margin on all sides. Tailwind's `m-80`.
    fn m_80(self) -> Self {
        self.m(320.0)
    }
    /// `384px` margin on all sides. Tailwind's `m-96`.
    fn m_96(self) -> Self {
        self.m(384.0)
    }
    /// `1px` margin on all sides. Tailwind's `m-px`.
    fn m_px(self) -> Self {
        self.m(1.0)
    }
    /// Negative `-2px` margin on all sides. Tailwind's `-m-0.5`.
    fn m_neg_0_p5(self) -> Self {
        self.m(-2.0)
    }
    /// Negative `-4px` margin on all sides. Tailwind's `-m-1`.
    fn m_neg_1(self) -> Self {
        self.m(-4.0)
    }
    /// Negative `-6px` margin on all sides. Tailwind's `-m-1.5`.
    fn m_neg_1_p5(self) -> Self {
        self.m(-6.0)
    }
    /// Negative `-8px` margin on all sides. Tailwind's `-m-2`.
    fn m_neg_2(self) -> Self {
        self.m(-8.0)
    }
    /// Negative `-10px` margin on all sides. Tailwind's `-m-2.5`.
    fn m_neg_2_p5(self) -> Self {
        self.m(-10.0)
    }
    /// Negative `-12px` margin on all sides. Tailwind's `-m-3`.
    fn m_neg_3(self) -> Self {
        self.m(-12.0)
    }
    /// Negative `-14px` margin on all sides. Tailwind's `-m-3.5`.
    fn m_neg_3_p5(self) -> Self {
        self.m(-14.0)
    }
    /// Negative `-16px` margin on all sides. Tailwind's `-m-4`.
    fn m_neg_4(self) -> Self {
        self.m(-16.0)
    }
    /// Negative `-20px` margin on all sides. Tailwind's `-m-5`.
    fn m_neg_5(self) -> Self {
        self.m(-20.0)
    }
    /// Negative `-24px` margin on all sides. Tailwind's `-m-6`.
    fn m_neg_6(self) -> Self {
        self.m(-24.0)
    }
    /// Negative `-28px` margin on all sides. Tailwind's `-m-7`.
    fn m_neg_7(self) -> Self {
        self.m(-28.0)
    }
    /// Negative `-32px` margin on all sides. Tailwind's `-m-8`.
    fn m_neg_8(self) -> Self {
        self.m(-32.0)
    }
    /// Negative `-36px` margin on all sides. Tailwind's `-m-9`.
    fn m_neg_9(self) -> Self {
        self.m(-36.0)
    }
    /// Negative `-40px` margin on all sides. Tailwind's `-m-10`.
    fn m_neg_10(self) -> Self {
        self.m(-40.0)
    }
    /// Negative `-44px` margin on all sides. Tailwind's `-m-11`.
    fn m_neg_11(self) -> Self {
        self.m(-44.0)
    }
    /// Negative `-48px` margin on all sides. Tailwind's `-m-12`.
    fn m_neg_12(self) -> Self {
        self.m(-48.0)
    }
    /// Negative `-56px` margin on all sides. Tailwind's `-m-14`.
    fn m_neg_14(self) -> Self {
        self.m(-56.0)
    }
    /// Negative `-64px` margin on all sides. Tailwind's `-m-16`.
    fn m_neg_16(self) -> Self {
        self.m(-64.0)
    }

    // margin: `mx_*` shortcuts (Tailwind `mx-*`)
    /// `0px` horizontal (left + right) margin. Tailwind's `mx-0`.
    fn mx_0(self) -> Self {
        self.mx(0.0)
    }
    /// `2px` horizontal (left + right) margin. Tailwind's `mx-0.5`.
    fn mx_0_p5(self) -> Self {
        self.mx(2.0)
    }
    /// `4px` horizontal (left + right) margin. Tailwind's `mx-1`.
    fn mx_1(self) -> Self {
        self.mx(4.0)
    }
    /// `6px` horizontal (left + right) margin. Tailwind's `mx-1.5`.
    fn mx_1_p5(self) -> Self {
        self.mx(6.0)
    }
    /// `8px` horizontal (left + right) margin. Tailwind's `mx-2`.
    fn mx_2(self) -> Self {
        self.mx(8.0)
    }
    /// `10px` horizontal (left + right) margin. Tailwind's `mx-2.5`.
    fn mx_2_p5(self) -> Self {
        self.mx(10.0)
    }
    /// `12px` horizontal (left + right) margin. Tailwind's `mx-3`.
    fn mx_3(self) -> Self {
        self.mx(12.0)
    }
    /// `14px` horizontal (left + right) margin. Tailwind's `mx-3.5`.
    fn mx_3_p5(self) -> Self {
        self.mx(14.0)
    }
    /// `16px` horizontal (left + right) margin. Tailwind's `mx-4`.
    fn mx_4(self) -> Self {
        self.mx(16.0)
    }
    /// `20px` horizontal (left + right) margin. Tailwind's `mx-5`.
    fn mx_5(self) -> Self {
        self.mx(20.0)
    }
    /// `24px` horizontal (left + right) margin. Tailwind's `mx-6`.
    fn mx_6(self) -> Self {
        self.mx(24.0)
    }
    /// `28px` horizontal (left + right) margin. Tailwind's `mx-7`.
    fn mx_7(self) -> Self {
        self.mx(28.0)
    }
    /// `32px` horizontal (left + right) margin. Tailwind's `mx-8`.
    fn mx_8(self) -> Self {
        self.mx(32.0)
    }
    /// `36px` horizontal (left + right) margin. Tailwind's `mx-9`.
    fn mx_9(self) -> Self {
        self.mx(36.0)
    }
    /// `40px` horizontal (left + right) margin. Tailwind's `mx-10`.
    fn mx_10(self) -> Self {
        self.mx(40.0)
    }
    /// `44px` horizontal (left + right) margin. Tailwind's `mx-11`.
    fn mx_11(self) -> Self {
        self.mx(44.0)
    }
    /// `48px` horizontal (left + right) margin. Tailwind's `mx-12`.
    fn mx_12(self) -> Self {
        self.mx(48.0)
    }
    /// `56px` horizontal (left + right) margin. Tailwind's `mx-14`.
    fn mx_14(self) -> Self {
        self.mx(56.0)
    }
    /// `64px` horizontal (left + right) margin. Tailwind's `mx-16`.
    fn mx_16(self) -> Self {
        self.mx(64.0)
    }
    /// `80px` horizontal (left + right) margin. Tailwind's `mx-20`.
    fn mx_20(self) -> Self {
        self.mx(80.0)
    }
    /// `96px` horizontal (left + right) margin. Tailwind's `mx-24`.
    fn mx_24(self) -> Self {
        self.mx(96.0)
    }
    /// `112px` horizontal (left + right) margin. Tailwind's `mx-28`.
    fn mx_28(self) -> Self {
        self.mx(112.0)
    }
    /// `128px` horizontal (left + right) margin. Tailwind's `mx-32`.
    fn mx_32(self) -> Self {
        self.mx(128.0)
    }
    /// `144px` horizontal (left + right) margin. Tailwind's `mx-36`.
    fn mx_36(self) -> Self {
        self.mx(144.0)
    }
    /// `160px` horizontal (left + right) margin. Tailwind's `mx-40`.
    fn mx_40(self) -> Self {
        self.mx(160.0)
    }
    /// `176px` horizontal (left + right) margin. Tailwind's `mx-44`.
    fn mx_44(self) -> Self {
        self.mx(176.0)
    }
    /// `192px` horizontal (left + right) margin. Tailwind's `mx-48`.
    fn mx_48(self) -> Self {
        self.mx(192.0)
    }
    /// `208px` horizontal (left + right) margin. Tailwind's `mx-52`.
    fn mx_52(self) -> Self {
        self.mx(208.0)
    }
    /// `224px` horizontal (left + right) margin. Tailwind's `mx-56`.
    fn mx_56(self) -> Self {
        self.mx(224.0)
    }
    /// `240px` horizontal (left + right) margin. Tailwind's `mx-60`.
    fn mx_60(self) -> Self {
        self.mx(240.0)
    }
    /// `256px` horizontal (left + right) margin. Tailwind's `mx-64`.
    fn mx_64(self) -> Self {
        self.mx(256.0)
    }
    /// `288px` horizontal (left + right) margin. Tailwind's `mx-72`.
    fn mx_72(self) -> Self {
        self.mx(288.0)
    }
    /// `320px` horizontal (left + right) margin. Tailwind's `mx-80`.
    fn mx_80(self) -> Self {
        self.mx(320.0)
    }
    /// `384px` horizontal (left + right) margin. Tailwind's `mx-96`.
    fn mx_96(self) -> Self {
        self.mx(384.0)
    }
    /// `1px` horizontal (left + right) margin. Tailwind's `mx-px`.
    fn mx_px(self) -> Self {
        self.mx(1.0)
    }
    /// Negative `-2px` margin (horizontal (left + right)). Tailwind's `-mx-0.5`.
    fn mx_neg_0_p5(self) -> Self {
        self.mx(-2.0)
    }
    /// Negative `-4px` margin (horizontal (left + right)). Tailwind's `-mx-1`.
    fn mx_neg_1(self) -> Self {
        self.mx(-4.0)
    }
    /// Negative `-6px` margin (horizontal (left + right)). Tailwind's `-mx-1.5`.
    fn mx_neg_1_p5(self) -> Self {
        self.mx(-6.0)
    }
    /// Negative `-8px` margin (horizontal (left + right)). Tailwind's `-mx-2`.
    fn mx_neg_2(self) -> Self {
        self.mx(-8.0)
    }
    /// Negative `-10px` margin (horizontal (left + right)). Tailwind's `-mx-2.5`.
    fn mx_neg_2_p5(self) -> Self {
        self.mx(-10.0)
    }
    /// Negative `-12px` margin (horizontal (left + right)). Tailwind's `-mx-3`.
    fn mx_neg_3(self) -> Self {
        self.mx(-12.0)
    }
    /// Negative `-14px` margin (horizontal (left + right)). Tailwind's `-mx-3.5`.
    fn mx_neg_3_p5(self) -> Self {
        self.mx(-14.0)
    }
    /// Negative `-16px` margin (horizontal (left + right)). Tailwind's `-mx-4`.
    fn mx_neg_4(self) -> Self {
        self.mx(-16.0)
    }
    /// Negative `-20px` margin (horizontal (left + right)). Tailwind's `-mx-5`.
    fn mx_neg_5(self) -> Self {
        self.mx(-20.0)
    }
    /// Negative `-24px` margin (horizontal (left + right)). Tailwind's `-mx-6`.
    fn mx_neg_6(self) -> Self {
        self.mx(-24.0)
    }
    /// Negative `-28px` margin (horizontal (left + right)). Tailwind's `-mx-7`.
    fn mx_neg_7(self) -> Self {
        self.mx(-28.0)
    }
    /// Negative `-32px` margin (horizontal (left + right)). Tailwind's `-mx-8`.
    fn mx_neg_8(self) -> Self {
        self.mx(-32.0)
    }
    /// Negative `-36px` margin (horizontal (left + right)). Tailwind's `-mx-9`.
    fn mx_neg_9(self) -> Self {
        self.mx(-36.0)
    }
    /// Negative `-40px` margin (horizontal (left + right)). Tailwind's `-mx-10`.
    fn mx_neg_10(self) -> Self {
        self.mx(-40.0)
    }
    /// Negative `-44px` margin (horizontal (left + right)). Tailwind's `-mx-11`.
    fn mx_neg_11(self) -> Self {
        self.mx(-44.0)
    }
    /// Negative `-48px` margin (horizontal (left + right)). Tailwind's `-mx-12`.
    fn mx_neg_12(self) -> Self {
        self.mx(-48.0)
    }
    /// Negative `-56px` margin (horizontal (left + right)). Tailwind's `-mx-14`.
    fn mx_neg_14(self) -> Self {
        self.mx(-56.0)
    }
    /// Negative `-64px` margin (horizontal (left + right)). Tailwind's `-mx-16`.
    fn mx_neg_16(self) -> Self {
        self.mx(-64.0)
    }

    // margin: `my_*` shortcuts (Tailwind `my-*`)
    /// `0px` vertical (top + bottom) margin. Tailwind's `my-0`.
    fn my_0(self) -> Self {
        self.my(0.0)
    }
    /// `2px` vertical (top + bottom) margin. Tailwind's `my-0.5`.
    fn my_0_p5(self) -> Self {
        self.my(2.0)
    }
    /// `4px` vertical (top + bottom) margin. Tailwind's `my-1`.
    fn my_1(self) -> Self {
        self.my(4.0)
    }
    /// `6px` vertical (top + bottom) margin. Tailwind's `my-1.5`.
    fn my_1_p5(self) -> Self {
        self.my(6.0)
    }
    /// `8px` vertical (top + bottom) margin. Tailwind's `my-2`.
    fn my_2(self) -> Self {
        self.my(8.0)
    }
    /// `10px` vertical (top + bottom) margin. Tailwind's `my-2.5`.
    fn my_2_p5(self) -> Self {
        self.my(10.0)
    }
    /// `12px` vertical (top + bottom) margin. Tailwind's `my-3`.
    fn my_3(self) -> Self {
        self.my(12.0)
    }
    /// `14px` vertical (top + bottom) margin. Tailwind's `my-3.5`.
    fn my_3_p5(self) -> Self {
        self.my(14.0)
    }
    /// `16px` vertical (top + bottom) margin. Tailwind's `my-4`.
    fn my_4(self) -> Self {
        self.my(16.0)
    }
    /// `20px` vertical (top + bottom) margin. Tailwind's `my-5`.
    fn my_5(self) -> Self {
        self.my(20.0)
    }
    /// `24px` vertical (top + bottom) margin. Tailwind's `my-6`.
    fn my_6(self) -> Self {
        self.my(24.0)
    }
    /// `28px` vertical (top + bottom) margin. Tailwind's `my-7`.
    fn my_7(self) -> Self {
        self.my(28.0)
    }
    /// `32px` vertical (top + bottom) margin. Tailwind's `my-8`.
    fn my_8(self) -> Self {
        self.my(32.0)
    }
    /// `36px` vertical (top + bottom) margin. Tailwind's `my-9`.
    fn my_9(self) -> Self {
        self.my(36.0)
    }
    /// `40px` vertical (top + bottom) margin. Tailwind's `my-10`.
    fn my_10(self) -> Self {
        self.my(40.0)
    }
    /// `44px` vertical (top + bottom) margin. Tailwind's `my-11`.
    fn my_11(self) -> Self {
        self.my(44.0)
    }
    /// `48px` vertical (top + bottom) margin. Tailwind's `my-12`.
    fn my_12(self) -> Self {
        self.my(48.0)
    }
    /// `56px` vertical (top + bottom) margin. Tailwind's `my-14`.
    fn my_14(self) -> Self {
        self.my(56.0)
    }
    /// `64px` vertical (top + bottom) margin. Tailwind's `my-16`.
    fn my_16(self) -> Self {
        self.my(64.0)
    }
    /// `80px` vertical (top + bottom) margin. Tailwind's `my-20`.
    fn my_20(self) -> Self {
        self.my(80.0)
    }
    /// `96px` vertical (top + bottom) margin. Tailwind's `my-24`.
    fn my_24(self) -> Self {
        self.my(96.0)
    }
    /// `112px` vertical (top + bottom) margin. Tailwind's `my-28`.
    fn my_28(self) -> Self {
        self.my(112.0)
    }
    /// `128px` vertical (top + bottom) margin. Tailwind's `my-32`.
    fn my_32(self) -> Self {
        self.my(128.0)
    }
    /// `144px` vertical (top + bottom) margin. Tailwind's `my-36`.
    fn my_36(self) -> Self {
        self.my(144.0)
    }
    /// `160px` vertical (top + bottom) margin. Tailwind's `my-40`.
    fn my_40(self) -> Self {
        self.my(160.0)
    }
    /// `176px` vertical (top + bottom) margin. Tailwind's `my-44`.
    fn my_44(self) -> Self {
        self.my(176.0)
    }
    /// `192px` vertical (top + bottom) margin. Tailwind's `my-48`.
    fn my_48(self) -> Self {
        self.my(192.0)
    }
    /// `208px` vertical (top + bottom) margin. Tailwind's `my-52`.
    fn my_52(self) -> Self {
        self.my(208.0)
    }
    /// `224px` vertical (top + bottom) margin. Tailwind's `my-56`.
    fn my_56(self) -> Self {
        self.my(224.0)
    }
    /// `240px` vertical (top + bottom) margin. Tailwind's `my-60`.
    fn my_60(self) -> Self {
        self.my(240.0)
    }
    /// `256px` vertical (top + bottom) margin. Tailwind's `my-64`.
    fn my_64(self) -> Self {
        self.my(256.0)
    }
    /// `288px` vertical (top + bottom) margin. Tailwind's `my-72`.
    fn my_72(self) -> Self {
        self.my(288.0)
    }
    /// `320px` vertical (top + bottom) margin. Tailwind's `my-80`.
    fn my_80(self) -> Self {
        self.my(320.0)
    }
    /// `384px` vertical (top + bottom) margin. Tailwind's `my-96`.
    fn my_96(self) -> Self {
        self.my(384.0)
    }
    /// `1px` vertical (top + bottom) margin. Tailwind's `my-px`.
    fn my_px(self) -> Self {
        self.my(1.0)
    }
    /// Negative `-2px` margin (vertical (top + bottom)). Tailwind's `-my-0.5`.
    fn my_neg_0_p5(self) -> Self {
        self.my(-2.0)
    }
    /// Negative `-4px` margin (vertical (top + bottom)). Tailwind's `-my-1`.
    fn my_neg_1(self) -> Self {
        self.my(-4.0)
    }
    /// Negative `-6px` margin (vertical (top + bottom)). Tailwind's `-my-1.5`.
    fn my_neg_1_p5(self) -> Self {
        self.my(-6.0)
    }
    /// Negative `-8px` margin (vertical (top + bottom)). Tailwind's `-my-2`.
    fn my_neg_2(self) -> Self {
        self.my(-8.0)
    }
    /// Negative `-10px` margin (vertical (top + bottom)). Tailwind's `-my-2.5`.
    fn my_neg_2_p5(self) -> Self {
        self.my(-10.0)
    }
    /// Negative `-12px` margin (vertical (top + bottom)). Tailwind's `-my-3`.
    fn my_neg_3(self) -> Self {
        self.my(-12.0)
    }
    /// Negative `-14px` margin (vertical (top + bottom)). Tailwind's `-my-3.5`.
    fn my_neg_3_p5(self) -> Self {
        self.my(-14.0)
    }
    /// Negative `-16px` margin (vertical (top + bottom)). Tailwind's `-my-4`.
    fn my_neg_4(self) -> Self {
        self.my(-16.0)
    }
    /// Negative `-20px` margin (vertical (top + bottom)). Tailwind's `-my-5`.
    fn my_neg_5(self) -> Self {
        self.my(-20.0)
    }
    /// Negative `-24px` margin (vertical (top + bottom)). Tailwind's `-my-6`.
    fn my_neg_6(self) -> Self {
        self.my(-24.0)
    }
    /// Negative `-28px` margin (vertical (top + bottom)). Tailwind's `-my-7`.
    fn my_neg_7(self) -> Self {
        self.my(-28.0)
    }
    /// Negative `-32px` margin (vertical (top + bottom)). Tailwind's `-my-8`.
    fn my_neg_8(self) -> Self {
        self.my(-32.0)
    }
    /// Negative `-36px` margin (vertical (top + bottom)). Tailwind's `-my-9`.
    fn my_neg_9(self) -> Self {
        self.my(-36.0)
    }
    /// Negative `-40px` margin (vertical (top + bottom)). Tailwind's `-my-10`.
    fn my_neg_10(self) -> Self {
        self.my(-40.0)
    }
    /// Negative `-44px` margin (vertical (top + bottom)). Tailwind's `-my-11`.
    fn my_neg_11(self) -> Self {
        self.my(-44.0)
    }
    /// Negative `-48px` margin (vertical (top + bottom)). Tailwind's `-my-12`.
    fn my_neg_12(self) -> Self {
        self.my(-48.0)
    }
    /// Negative `-56px` margin (vertical (top + bottom)). Tailwind's `-my-14`.
    fn my_neg_14(self) -> Self {
        self.my(-56.0)
    }
    /// Negative `-64px` margin (vertical (top + bottom)). Tailwind's `-my-16`.
    fn my_neg_16(self) -> Self {
        self.my(-64.0)
    }

    // margin: `mt_*` shortcuts (Tailwind `mt-*`)
    /// `0px` top margin. Tailwind's `mt-0`.
    fn mt_0(self) -> Self {
        self.mt(0.0)
    }
    /// `2px` top margin. Tailwind's `mt-0.5`.
    fn mt_0_p5(self) -> Self {
        self.mt(2.0)
    }
    /// `4px` top margin. Tailwind's `mt-1`.
    fn mt_1(self) -> Self {
        self.mt(4.0)
    }
    /// `6px` top margin. Tailwind's `mt-1.5`.
    fn mt_1_p5(self) -> Self {
        self.mt(6.0)
    }
    /// `8px` top margin. Tailwind's `mt-2`.
    fn mt_2(self) -> Self {
        self.mt(8.0)
    }
    /// `10px` top margin. Tailwind's `mt-2.5`.
    fn mt_2_p5(self) -> Self {
        self.mt(10.0)
    }
    /// `12px` top margin. Tailwind's `mt-3`.
    fn mt_3(self) -> Self {
        self.mt(12.0)
    }
    /// `14px` top margin. Tailwind's `mt-3.5`.
    fn mt_3_p5(self) -> Self {
        self.mt(14.0)
    }
    /// `16px` top margin. Tailwind's `mt-4`.
    fn mt_4(self) -> Self {
        self.mt(16.0)
    }
    /// `20px` top margin. Tailwind's `mt-5`.
    fn mt_5(self) -> Self {
        self.mt(20.0)
    }
    /// `24px` top margin. Tailwind's `mt-6`.
    fn mt_6(self) -> Self {
        self.mt(24.0)
    }
    /// `28px` top margin. Tailwind's `mt-7`.
    fn mt_7(self) -> Self {
        self.mt(28.0)
    }
    /// `32px` top margin. Tailwind's `mt-8`.
    fn mt_8(self) -> Self {
        self.mt(32.0)
    }
    /// `36px` top margin. Tailwind's `mt-9`.
    fn mt_9(self) -> Self {
        self.mt(36.0)
    }
    /// `40px` top margin. Tailwind's `mt-10`.
    fn mt_10(self) -> Self {
        self.mt(40.0)
    }
    /// `44px` top margin. Tailwind's `mt-11`.
    fn mt_11(self) -> Self {
        self.mt(44.0)
    }
    /// `48px` top margin. Tailwind's `mt-12`.
    fn mt_12(self) -> Self {
        self.mt(48.0)
    }
    /// `56px` top margin. Tailwind's `mt-14`.
    fn mt_14(self) -> Self {
        self.mt(56.0)
    }
    /// `64px` top margin. Tailwind's `mt-16`.
    fn mt_16(self) -> Self {
        self.mt(64.0)
    }
    /// `80px` top margin. Tailwind's `mt-20`.
    fn mt_20(self) -> Self {
        self.mt(80.0)
    }
    /// `96px` top margin. Tailwind's `mt-24`.
    fn mt_24(self) -> Self {
        self.mt(96.0)
    }
    /// `112px` top margin. Tailwind's `mt-28`.
    fn mt_28(self) -> Self {
        self.mt(112.0)
    }
    /// `128px` top margin. Tailwind's `mt-32`.
    fn mt_32(self) -> Self {
        self.mt(128.0)
    }
    /// `144px` top margin. Tailwind's `mt-36`.
    fn mt_36(self) -> Self {
        self.mt(144.0)
    }
    /// `160px` top margin. Tailwind's `mt-40`.
    fn mt_40(self) -> Self {
        self.mt(160.0)
    }
    /// `176px` top margin. Tailwind's `mt-44`.
    fn mt_44(self) -> Self {
        self.mt(176.0)
    }
    /// `192px` top margin. Tailwind's `mt-48`.
    fn mt_48(self) -> Self {
        self.mt(192.0)
    }
    /// `208px` top margin. Tailwind's `mt-52`.
    fn mt_52(self) -> Self {
        self.mt(208.0)
    }
    /// `224px` top margin. Tailwind's `mt-56`.
    fn mt_56(self) -> Self {
        self.mt(224.0)
    }
    /// `240px` top margin. Tailwind's `mt-60`.
    fn mt_60(self) -> Self {
        self.mt(240.0)
    }
    /// `256px` top margin. Tailwind's `mt-64`.
    fn mt_64(self) -> Self {
        self.mt(256.0)
    }
    /// `288px` top margin. Tailwind's `mt-72`.
    fn mt_72(self) -> Self {
        self.mt(288.0)
    }
    /// `320px` top margin. Tailwind's `mt-80`.
    fn mt_80(self) -> Self {
        self.mt(320.0)
    }
    /// `384px` top margin. Tailwind's `mt-96`.
    fn mt_96(self) -> Self {
        self.mt(384.0)
    }
    /// `1px` top margin. Tailwind's `mt-px`.
    fn mt_px(self) -> Self {
        self.mt(1.0)
    }
    /// Negative `-2px` margin (top). Tailwind's `-mt-0.5`.
    fn mt_neg_0_p5(self) -> Self {
        self.mt(-2.0)
    }
    /// Negative `-4px` margin (top). Tailwind's `-mt-1`.
    fn mt_neg_1(self) -> Self {
        self.mt(-4.0)
    }
    /// Negative `-6px` margin (top). Tailwind's `-mt-1.5`.
    fn mt_neg_1_p5(self) -> Self {
        self.mt(-6.0)
    }
    /// Negative `-8px` margin (top). Tailwind's `-mt-2`.
    fn mt_neg_2(self) -> Self {
        self.mt(-8.0)
    }
    /// Negative `-10px` margin (top). Tailwind's `-mt-2.5`.
    fn mt_neg_2_p5(self) -> Self {
        self.mt(-10.0)
    }
    /// Negative `-12px` margin (top). Tailwind's `-mt-3`.
    fn mt_neg_3(self) -> Self {
        self.mt(-12.0)
    }
    /// Negative `-14px` margin (top). Tailwind's `-mt-3.5`.
    fn mt_neg_3_p5(self) -> Self {
        self.mt(-14.0)
    }
    /// Negative `-16px` margin (top). Tailwind's `-mt-4`.
    fn mt_neg_4(self) -> Self {
        self.mt(-16.0)
    }
    /// Negative `-20px` margin (top). Tailwind's `-mt-5`.
    fn mt_neg_5(self) -> Self {
        self.mt(-20.0)
    }
    /// Negative `-24px` margin (top). Tailwind's `-mt-6`.
    fn mt_neg_6(self) -> Self {
        self.mt(-24.0)
    }
    /// Negative `-28px` margin (top). Tailwind's `-mt-7`.
    fn mt_neg_7(self) -> Self {
        self.mt(-28.0)
    }
    /// Negative `-32px` margin (top). Tailwind's `-mt-8`.
    fn mt_neg_8(self) -> Self {
        self.mt(-32.0)
    }
    /// Negative `-36px` margin (top). Tailwind's `-mt-9`.
    fn mt_neg_9(self) -> Self {
        self.mt(-36.0)
    }
    /// Negative `-40px` margin (top). Tailwind's `-mt-10`.
    fn mt_neg_10(self) -> Self {
        self.mt(-40.0)
    }
    /// Negative `-44px` margin (top). Tailwind's `-mt-11`.
    fn mt_neg_11(self) -> Self {
        self.mt(-44.0)
    }
    /// Negative `-48px` margin (top). Tailwind's `-mt-12`.
    fn mt_neg_12(self) -> Self {
        self.mt(-48.0)
    }
    /// Negative `-56px` margin (top). Tailwind's `-mt-14`.
    fn mt_neg_14(self) -> Self {
        self.mt(-56.0)
    }
    /// Negative `-64px` margin (top). Tailwind's `-mt-16`.
    fn mt_neg_16(self) -> Self {
        self.mt(-64.0)
    }

    // margin: `mr_*` shortcuts (Tailwind `mr-*`)
    /// `0px` right margin. Tailwind's `mr-0`.
    fn mr_0(self) -> Self {
        self.mr(0.0)
    }
    /// `2px` right margin. Tailwind's `mr-0.5`.
    fn mr_0_p5(self) -> Self {
        self.mr(2.0)
    }
    /// `4px` right margin. Tailwind's `mr-1`.
    fn mr_1(self) -> Self {
        self.mr(4.0)
    }
    /// `6px` right margin. Tailwind's `mr-1.5`.
    fn mr_1_p5(self) -> Self {
        self.mr(6.0)
    }
    /// `8px` right margin. Tailwind's `mr-2`.
    fn mr_2(self) -> Self {
        self.mr(8.0)
    }
    /// `10px` right margin. Tailwind's `mr-2.5`.
    fn mr_2_p5(self) -> Self {
        self.mr(10.0)
    }
    /// `12px` right margin. Tailwind's `mr-3`.
    fn mr_3(self) -> Self {
        self.mr(12.0)
    }
    /// `14px` right margin. Tailwind's `mr-3.5`.
    fn mr_3_p5(self) -> Self {
        self.mr(14.0)
    }
    /// `16px` right margin. Tailwind's `mr-4`.
    fn mr_4(self) -> Self {
        self.mr(16.0)
    }
    /// `20px` right margin. Tailwind's `mr-5`.
    fn mr_5(self) -> Self {
        self.mr(20.0)
    }
    /// `24px` right margin. Tailwind's `mr-6`.
    fn mr_6(self) -> Self {
        self.mr(24.0)
    }
    /// `28px` right margin. Tailwind's `mr-7`.
    fn mr_7(self) -> Self {
        self.mr(28.0)
    }
    /// `32px` right margin. Tailwind's `mr-8`.
    fn mr_8(self) -> Self {
        self.mr(32.0)
    }
    /// `36px` right margin. Tailwind's `mr-9`.
    fn mr_9(self) -> Self {
        self.mr(36.0)
    }
    /// `40px` right margin. Tailwind's `mr-10`.
    fn mr_10(self) -> Self {
        self.mr(40.0)
    }
    /// `44px` right margin. Tailwind's `mr-11`.
    fn mr_11(self) -> Self {
        self.mr(44.0)
    }
    /// `48px` right margin. Tailwind's `mr-12`.
    fn mr_12(self) -> Self {
        self.mr(48.0)
    }
    /// `56px` right margin. Tailwind's `mr-14`.
    fn mr_14(self) -> Self {
        self.mr(56.0)
    }
    /// `64px` right margin. Tailwind's `mr-16`.
    fn mr_16(self) -> Self {
        self.mr(64.0)
    }
    /// `80px` right margin. Tailwind's `mr-20`.
    fn mr_20(self) -> Self {
        self.mr(80.0)
    }
    /// `96px` right margin. Tailwind's `mr-24`.
    fn mr_24(self) -> Self {
        self.mr(96.0)
    }
    /// `112px` right margin. Tailwind's `mr-28`.
    fn mr_28(self) -> Self {
        self.mr(112.0)
    }
    /// `128px` right margin. Tailwind's `mr-32`.
    fn mr_32(self) -> Self {
        self.mr(128.0)
    }
    /// `144px` right margin. Tailwind's `mr-36`.
    fn mr_36(self) -> Self {
        self.mr(144.0)
    }
    /// `160px` right margin. Tailwind's `mr-40`.
    fn mr_40(self) -> Self {
        self.mr(160.0)
    }
    /// `176px` right margin. Tailwind's `mr-44`.
    fn mr_44(self) -> Self {
        self.mr(176.0)
    }
    /// `192px` right margin. Tailwind's `mr-48`.
    fn mr_48(self) -> Self {
        self.mr(192.0)
    }
    /// `208px` right margin. Tailwind's `mr-52`.
    fn mr_52(self) -> Self {
        self.mr(208.0)
    }
    /// `224px` right margin. Tailwind's `mr-56`.
    fn mr_56(self) -> Self {
        self.mr(224.0)
    }
    /// `240px` right margin. Tailwind's `mr-60`.
    fn mr_60(self) -> Self {
        self.mr(240.0)
    }
    /// `256px` right margin. Tailwind's `mr-64`.
    fn mr_64(self) -> Self {
        self.mr(256.0)
    }
    /// `288px` right margin. Tailwind's `mr-72`.
    fn mr_72(self) -> Self {
        self.mr(288.0)
    }
    /// `320px` right margin. Tailwind's `mr-80`.
    fn mr_80(self) -> Self {
        self.mr(320.0)
    }
    /// `384px` right margin. Tailwind's `mr-96`.
    fn mr_96(self) -> Self {
        self.mr(384.0)
    }
    /// `1px` right margin. Tailwind's `mr-px`.
    fn mr_px(self) -> Self {
        self.mr(1.0)
    }
    /// Negative `-2px` margin (right). Tailwind's `-mr-0.5`.
    fn mr_neg_0_p5(self) -> Self {
        self.mr(-2.0)
    }
    /// Negative `-4px` margin (right). Tailwind's `-mr-1`.
    fn mr_neg_1(self) -> Self {
        self.mr(-4.0)
    }
    /// Negative `-6px` margin (right). Tailwind's `-mr-1.5`.
    fn mr_neg_1_p5(self) -> Self {
        self.mr(-6.0)
    }
    /// Negative `-8px` margin (right). Tailwind's `-mr-2`.
    fn mr_neg_2(self) -> Self {
        self.mr(-8.0)
    }
    /// Negative `-10px` margin (right). Tailwind's `-mr-2.5`.
    fn mr_neg_2_p5(self) -> Self {
        self.mr(-10.0)
    }
    /// Negative `-12px` margin (right). Tailwind's `-mr-3`.
    fn mr_neg_3(self) -> Self {
        self.mr(-12.0)
    }
    /// Negative `-14px` margin (right). Tailwind's `-mr-3.5`.
    fn mr_neg_3_p5(self) -> Self {
        self.mr(-14.0)
    }
    /// Negative `-16px` margin (right). Tailwind's `-mr-4`.
    fn mr_neg_4(self) -> Self {
        self.mr(-16.0)
    }
    /// Negative `-20px` margin (right). Tailwind's `-mr-5`.
    fn mr_neg_5(self) -> Self {
        self.mr(-20.0)
    }
    /// Negative `-24px` margin (right). Tailwind's `-mr-6`.
    fn mr_neg_6(self) -> Self {
        self.mr(-24.0)
    }
    /// Negative `-28px` margin (right). Tailwind's `-mr-7`.
    fn mr_neg_7(self) -> Self {
        self.mr(-28.0)
    }
    /// Negative `-32px` margin (right). Tailwind's `-mr-8`.
    fn mr_neg_8(self) -> Self {
        self.mr(-32.0)
    }
    /// Negative `-36px` margin (right). Tailwind's `-mr-9`.
    fn mr_neg_9(self) -> Self {
        self.mr(-36.0)
    }
    /// Negative `-40px` margin (right). Tailwind's `-mr-10`.
    fn mr_neg_10(self) -> Self {
        self.mr(-40.0)
    }
    /// Negative `-44px` margin (right). Tailwind's `-mr-11`.
    fn mr_neg_11(self) -> Self {
        self.mr(-44.0)
    }
    /// Negative `-48px` margin (right). Tailwind's `-mr-12`.
    fn mr_neg_12(self) -> Self {
        self.mr(-48.0)
    }
    /// Negative `-56px` margin (right). Tailwind's `-mr-14`.
    fn mr_neg_14(self) -> Self {
        self.mr(-56.0)
    }
    /// Negative `-64px` margin (right). Tailwind's `-mr-16`.
    fn mr_neg_16(self) -> Self {
        self.mr(-64.0)
    }

    // margin: `mb_*` shortcuts (Tailwind `mb-*`)
    /// `0px` bottom margin. Tailwind's `mb-0`.
    fn mb_0(self) -> Self {
        self.mb(0.0)
    }
    /// `2px` bottom margin. Tailwind's `mb-0.5`.
    fn mb_0_p5(self) -> Self {
        self.mb(2.0)
    }
    /// `4px` bottom margin. Tailwind's `mb-1`.
    fn mb_1(self) -> Self {
        self.mb(4.0)
    }
    /// `6px` bottom margin. Tailwind's `mb-1.5`.
    fn mb_1_p5(self) -> Self {
        self.mb(6.0)
    }
    /// `8px` bottom margin. Tailwind's `mb-2`.
    fn mb_2(self) -> Self {
        self.mb(8.0)
    }
    /// `10px` bottom margin. Tailwind's `mb-2.5`.
    fn mb_2_p5(self) -> Self {
        self.mb(10.0)
    }
    /// `12px` bottom margin. Tailwind's `mb-3`.
    fn mb_3(self) -> Self {
        self.mb(12.0)
    }
    /// `14px` bottom margin. Tailwind's `mb-3.5`.
    fn mb_3_p5(self) -> Self {
        self.mb(14.0)
    }
    /// `16px` bottom margin. Tailwind's `mb-4`.
    fn mb_4(self) -> Self {
        self.mb(16.0)
    }
    /// `20px` bottom margin. Tailwind's `mb-5`.
    fn mb_5(self) -> Self {
        self.mb(20.0)
    }
    /// `24px` bottom margin. Tailwind's `mb-6`.
    fn mb_6(self) -> Self {
        self.mb(24.0)
    }
    /// `28px` bottom margin. Tailwind's `mb-7`.
    fn mb_7(self) -> Self {
        self.mb(28.0)
    }
    /// `32px` bottom margin. Tailwind's `mb-8`.
    fn mb_8(self) -> Self {
        self.mb(32.0)
    }
    /// `36px` bottom margin. Tailwind's `mb-9`.
    fn mb_9(self) -> Self {
        self.mb(36.0)
    }
    /// `40px` bottom margin. Tailwind's `mb-10`.
    fn mb_10(self) -> Self {
        self.mb(40.0)
    }
    /// `44px` bottom margin. Tailwind's `mb-11`.
    fn mb_11(self) -> Self {
        self.mb(44.0)
    }
    /// `48px` bottom margin. Tailwind's `mb-12`.
    fn mb_12(self) -> Self {
        self.mb(48.0)
    }
    /// `56px` bottom margin. Tailwind's `mb-14`.
    fn mb_14(self) -> Self {
        self.mb(56.0)
    }
    /// `64px` bottom margin. Tailwind's `mb-16`.
    fn mb_16(self) -> Self {
        self.mb(64.0)
    }
    /// `80px` bottom margin. Tailwind's `mb-20`.
    fn mb_20(self) -> Self {
        self.mb(80.0)
    }
    /// `96px` bottom margin. Tailwind's `mb-24`.
    fn mb_24(self) -> Self {
        self.mb(96.0)
    }
    /// `112px` bottom margin. Tailwind's `mb-28`.
    fn mb_28(self) -> Self {
        self.mb(112.0)
    }
    /// `128px` bottom margin. Tailwind's `mb-32`.
    fn mb_32(self) -> Self {
        self.mb(128.0)
    }
    /// `144px` bottom margin. Tailwind's `mb-36`.
    fn mb_36(self) -> Self {
        self.mb(144.0)
    }
    /// `160px` bottom margin. Tailwind's `mb-40`.
    fn mb_40(self) -> Self {
        self.mb(160.0)
    }
    /// `176px` bottom margin. Tailwind's `mb-44`.
    fn mb_44(self) -> Self {
        self.mb(176.0)
    }
    /// `192px` bottom margin. Tailwind's `mb-48`.
    fn mb_48(self) -> Self {
        self.mb(192.0)
    }
    /// `208px` bottom margin. Tailwind's `mb-52`.
    fn mb_52(self) -> Self {
        self.mb(208.0)
    }
    /// `224px` bottom margin. Tailwind's `mb-56`.
    fn mb_56(self) -> Self {
        self.mb(224.0)
    }
    /// `240px` bottom margin. Tailwind's `mb-60`.
    fn mb_60(self) -> Self {
        self.mb(240.0)
    }
    /// `256px` bottom margin. Tailwind's `mb-64`.
    fn mb_64(self) -> Self {
        self.mb(256.0)
    }
    /// `288px` bottom margin. Tailwind's `mb-72`.
    fn mb_72(self) -> Self {
        self.mb(288.0)
    }
    /// `320px` bottom margin. Tailwind's `mb-80`.
    fn mb_80(self) -> Self {
        self.mb(320.0)
    }
    /// `384px` bottom margin. Tailwind's `mb-96`.
    fn mb_96(self) -> Self {
        self.mb(384.0)
    }
    /// `1px` bottom margin. Tailwind's `mb-px`.
    fn mb_px(self) -> Self {
        self.mb(1.0)
    }
    /// Negative `-2px` margin (bottom). Tailwind's `-mb-0.5`.
    fn mb_neg_0_p5(self) -> Self {
        self.mb(-2.0)
    }
    /// Negative `-4px` margin (bottom). Tailwind's `-mb-1`.
    fn mb_neg_1(self) -> Self {
        self.mb(-4.0)
    }
    /// Negative `-6px` margin (bottom). Tailwind's `-mb-1.5`.
    fn mb_neg_1_p5(self) -> Self {
        self.mb(-6.0)
    }
    /// Negative `-8px` margin (bottom). Tailwind's `-mb-2`.
    fn mb_neg_2(self) -> Self {
        self.mb(-8.0)
    }
    /// Negative `-10px` margin (bottom). Tailwind's `-mb-2.5`.
    fn mb_neg_2_p5(self) -> Self {
        self.mb(-10.0)
    }
    /// Negative `-12px` margin (bottom). Tailwind's `-mb-3`.
    fn mb_neg_3(self) -> Self {
        self.mb(-12.0)
    }
    /// Negative `-14px` margin (bottom). Tailwind's `-mb-3.5`.
    fn mb_neg_3_p5(self) -> Self {
        self.mb(-14.0)
    }
    /// Negative `-16px` margin (bottom). Tailwind's `-mb-4`.
    fn mb_neg_4(self) -> Self {
        self.mb(-16.0)
    }
    /// Negative `-20px` margin (bottom). Tailwind's `-mb-5`.
    fn mb_neg_5(self) -> Self {
        self.mb(-20.0)
    }
    /// Negative `-24px` margin (bottom). Tailwind's `-mb-6`.
    fn mb_neg_6(self) -> Self {
        self.mb(-24.0)
    }
    /// Negative `-28px` margin (bottom). Tailwind's `-mb-7`.
    fn mb_neg_7(self) -> Self {
        self.mb(-28.0)
    }
    /// Negative `-32px` margin (bottom). Tailwind's `-mb-8`.
    fn mb_neg_8(self) -> Self {
        self.mb(-32.0)
    }
    /// Negative `-36px` margin (bottom). Tailwind's `-mb-9`.
    fn mb_neg_9(self) -> Self {
        self.mb(-36.0)
    }
    /// Negative `-40px` margin (bottom). Tailwind's `-mb-10`.
    fn mb_neg_10(self) -> Self {
        self.mb(-40.0)
    }
    /// Negative `-44px` margin (bottom). Tailwind's `-mb-11`.
    fn mb_neg_11(self) -> Self {
        self.mb(-44.0)
    }
    /// Negative `-48px` margin (bottom). Tailwind's `-mb-12`.
    fn mb_neg_12(self) -> Self {
        self.mb(-48.0)
    }
    /// Negative `-56px` margin (bottom). Tailwind's `-mb-14`.
    fn mb_neg_14(self) -> Self {
        self.mb(-56.0)
    }
    /// Negative `-64px` margin (bottom). Tailwind's `-mb-16`.
    fn mb_neg_16(self) -> Self {
        self.mb(-64.0)
    }

    // margin: `ml_*` shortcuts (Tailwind `ml-*`)
    /// `0px` left margin. Tailwind's `ml-0`.
    fn ml_0(self) -> Self {
        self.ml(0.0)
    }
    /// `2px` left margin. Tailwind's `ml-0.5`.
    fn ml_0_p5(self) -> Self {
        self.ml(2.0)
    }
    /// `4px` left margin. Tailwind's `ml-1`.
    fn ml_1(self) -> Self {
        self.ml(4.0)
    }
    /// `6px` left margin. Tailwind's `ml-1.5`.
    fn ml_1_p5(self) -> Self {
        self.ml(6.0)
    }
    /// `8px` left margin. Tailwind's `ml-2`.
    fn ml_2(self) -> Self {
        self.ml(8.0)
    }
    /// `10px` left margin. Tailwind's `ml-2.5`.
    fn ml_2_p5(self) -> Self {
        self.ml(10.0)
    }
    /// `12px` left margin. Tailwind's `ml-3`.
    fn ml_3(self) -> Self {
        self.ml(12.0)
    }
    /// `14px` left margin. Tailwind's `ml-3.5`.
    fn ml_3_p5(self) -> Self {
        self.ml(14.0)
    }
    /// `16px` left margin. Tailwind's `ml-4`.
    fn ml_4(self) -> Self {
        self.ml(16.0)
    }
    /// `20px` left margin. Tailwind's `ml-5`.
    fn ml_5(self) -> Self {
        self.ml(20.0)
    }
    /// `24px` left margin. Tailwind's `ml-6`.
    fn ml_6(self) -> Self {
        self.ml(24.0)
    }
    /// `28px` left margin. Tailwind's `ml-7`.
    fn ml_7(self) -> Self {
        self.ml(28.0)
    }
    /// `32px` left margin. Tailwind's `ml-8`.
    fn ml_8(self) -> Self {
        self.ml(32.0)
    }
    /// `36px` left margin. Tailwind's `ml-9`.
    fn ml_9(self) -> Self {
        self.ml(36.0)
    }
    /// `40px` left margin. Tailwind's `ml-10`.
    fn ml_10(self) -> Self {
        self.ml(40.0)
    }
    /// `44px` left margin. Tailwind's `ml-11`.
    fn ml_11(self) -> Self {
        self.ml(44.0)
    }
    /// `48px` left margin. Tailwind's `ml-12`.
    fn ml_12(self) -> Self {
        self.ml(48.0)
    }
    /// `56px` left margin. Tailwind's `ml-14`.
    fn ml_14(self) -> Self {
        self.ml(56.0)
    }
    /// `64px` left margin. Tailwind's `ml-16`.
    fn ml_16(self) -> Self {
        self.ml(64.0)
    }
    /// `80px` left margin. Tailwind's `ml-20`.
    fn ml_20(self) -> Self {
        self.ml(80.0)
    }
    /// `96px` left margin. Tailwind's `ml-24`.
    fn ml_24(self) -> Self {
        self.ml(96.0)
    }
    /// `112px` left margin. Tailwind's `ml-28`.
    fn ml_28(self) -> Self {
        self.ml(112.0)
    }
    /// `128px` left margin. Tailwind's `ml-32`.
    fn ml_32(self) -> Self {
        self.ml(128.0)
    }
    /// `144px` left margin. Tailwind's `ml-36`.
    fn ml_36(self) -> Self {
        self.ml(144.0)
    }
    /// `160px` left margin. Tailwind's `ml-40`.
    fn ml_40(self) -> Self {
        self.ml(160.0)
    }
    /// `176px` left margin. Tailwind's `ml-44`.
    fn ml_44(self) -> Self {
        self.ml(176.0)
    }
    /// `192px` left margin. Tailwind's `ml-48`.
    fn ml_48(self) -> Self {
        self.ml(192.0)
    }
    /// `208px` left margin. Tailwind's `ml-52`.
    fn ml_52(self) -> Self {
        self.ml(208.0)
    }
    /// `224px` left margin. Tailwind's `ml-56`.
    fn ml_56(self) -> Self {
        self.ml(224.0)
    }
    /// `240px` left margin. Tailwind's `ml-60`.
    fn ml_60(self) -> Self {
        self.ml(240.0)
    }
    /// `256px` left margin. Tailwind's `ml-64`.
    fn ml_64(self) -> Self {
        self.ml(256.0)
    }
    /// `288px` left margin. Tailwind's `ml-72`.
    fn ml_72(self) -> Self {
        self.ml(288.0)
    }
    /// `320px` left margin. Tailwind's `ml-80`.
    fn ml_80(self) -> Self {
        self.ml(320.0)
    }
    /// `384px` left margin. Tailwind's `ml-96`.
    fn ml_96(self) -> Self {
        self.ml(384.0)
    }
    /// `1px` left margin. Tailwind's `ml-px`.
    fn ml_px(self) -> Self {
        self.ml(1.0)
    }
    /// Negative `-2px` margin (left). Tailwind's `-ml-0.5`.
    fn ml_neg_0_p5(self) -> Self {
        self.ml(-2.0)
    }
    /// Negative `-4px` margin (left). Tailwind's `-ml-1`.
    fn ml_neg_1(self) -> Self {
        self.ml(-4.0)
    }
    /// Negative `-6px` margin (left). Tailwind's `-ml-1.5`.
    fn ml_neg_1_p5(self) -> Self {
        self.ml(-6.0)
    }
    /// Negative `-8px` margin (left). Tailwind's `-ml-2`.
    fn ml_neg_2(self) -> Self {
        self.ml(-8.0)
    }
    /// Negative `-10px` margin (left). Tailwind's `-ml-2.5`.
    fn ml_neg_2_p5(self) -> Self {
        self.ml(-10.0)
    }
    /// Negative `-12px` margin (left). Tailwind's `-ml-3`.
    fn ml_neg_3(self) -> Self {
        self.ml(-12.0)
    }
    /// Negative `-14px` margin (left). Tailwind's `-ml-3.5`.
    fn ml_neg_3_p5(self) -> Self {
        self.ml(-14.0)
    }
    /// Negative `-16px` margin (left). Tailwind's `-ml-4`.
    fn ml_neg_4(self) -> Self {
        self.ml(-16.0)
    }
    /// Negative `-20px` margin (left). Tailwind's `-ml-5`.
    fn ml_neg_5(self) -> Self {
        self.ml(-20.0)
    }
    /// Negative `-24px` margin (left). Tailwind's `-ml-6`.
    fn ml_neg_6(self) -> Self {
        self.ml(-24.0)
    }
    /// Negative `-28px` margin (left). Tailwind's `-ml-7`.
    fn ml_neg_7(self) -> Self {
        self.ml(-28.0)
    }
    /// Negative `-32px` margin (left). Tailwind's `-ml-8`.
    fn ml_neg_8(self) -> Self {
        self.ml(-32.0)
    }
    /// Negative `-36px` margin (left). Tailwind's `-ml-9`.
    fn ml_neg_9(self) -> Self {
        self.ml(-36.0)
    }
    /// Negative `-40px` margin (left). Tailwind's `-ml-10`.
    fn ml_neg_10(self) -> Self {
        self.ml(-40.0)
    }
    /// Negative `-44px` margin (left). Tailwind's `-ml-11`.
    fn ml_neg_11(self) -> Self {
        self.ml(-44.0)
    }
    /// Negative `-48px` margin (left). Tailwind's `-ml-12`.
    fn ml_neg_12(self) -> Self {
        self.ml(-48.0)
    }
    /// Negative `-56px` margin (left). Tailwind's `-ml-14`.
    fn ml_neg_14(self) -> Self {
        self.ml(-56.0)
    }
    /// Negative `-64px` margin (left). Tailwind's `-ml-16`.
    fn ml_neg_16(self) -> Self {
        self.ml(-64.0)
    }

    // ---- width ----

    /// Set the width, in pixels.
    fn w(mut self, v: f32) -> Self {
        self.get_style().width = Size::Pixels(Length::new(v));
        self
    }
    /// Set the width to any `Size`.
    fn w_size(mut self, size: impl Into<Size>) -> Self {
        self.get_style().width = size.into();
        self
    }

    /// `0px` width. Tailwind's `w-0`.
    fn w_0(self) -> Self {
        self.w(0.0)
    }
    /// `2px` width. Tailwind's `w-0.5`.
    fn w_0_p5(self) -> Self {
        self.w(2.0)
    }
    /// `4px` width. Tailwind's `w-1`.
    fn w_1(self) -> Self {
        self.w(4.0)
    }
    /// `6px` width. Tailwind's `w-1.5`.
    fn w_1_p5(self) -> Self {
        self.w(6.0)
    }
    /// `8px` width. Tailwind's `w-2`.
    fn w_2(self) -> Self {
        self.w(8.0)
    }
    /// `10px` width. Tailwind's `w-2.5`.
    fn w_2_p5(self) -> Self {
        self.w(10.0)
    }
    /// `12px` width. Tailwind's `w-3`.
    fn w_3(self) -> Self {
        self.w(12.0)
    }
    /// `14px` width. Tailwind's `w-3.5`.
    fn w_3_p5(self) -> Self {
        self.w(14.0)
    }
    /// `16px` width. Tailwind's `w-4`.
    fn w_4(self) -> Self {
        self.w(16.0)
    }
    /// `20px` width. Tailwind's `w-5`.
    fn w_5(self) -> Self {
        self.w(20.0)
    }
    /// `24px` width. Tailwind's `w-6`.
    fn w_6(self) -> Self {
        self.w(24.0)
    }
    /// `28px` width. Tailwind's `w-7`.
    fn w_7(self) -> Self {
        self.w(28.0)
    }
    /// `32px` width. Tailwind's `w-8`.
    fn w_8(self) -> Self {
        self.w(32.0)
    }
    /// `36px` width. Tailwind's `w-9`.
    fn w_9(self) -> Self {
        self.w(36.0)
    }
    /// `40px` width. Tailwind's `w-10`.
    fn w_10(self) -> Self {
        self.w(40.0)
    }
    /// `44px` width. Tailwind's `w-11`.
    fn w_11(self) -> Self {
        self.w(44.0)
    }
    /// `48px` width. Tailwind's `w-12`.
    fn w_12(self) -> Self {
        self.w(48.0)
    }
    /// `56px` width. Tailwind's `w-14`.
    fn w_14(self) -> Self {
        self.w(56.0)
    }
    /// `64px` width. Tailwind's `w-16`.
    fn w_16(self) -> Self {
        self.w(64.0)
    }
    /// `80px` width. Tailwind's `w-20`.
    fn w_20(self) -> Self {
        self.w(80.0)
    }
    /// `96px` width. Tailwind's `w-24`.
    fn w_24(self) -> Self {
        self.w(96.0)
    }
    /// `112px` width. Tailwind's `w-28`.
    fn w_28(self) -> Self {
        self.w(112.0)
    }
    /// `128px` width. Tailwind's `w-32`.
    fn w_32(self) -> Self {
        self.w(128.0)
    }
    /// `144px` width. Tailwind's `w-36`.
    fn w_36(self) -> Self {
        self.w(144.0)
    }
    /// `160px` width. Tailwind's `w-40`.
    fn w_40(self) -> Self {
        self.w(160.0)
    }
    /// `176px` width. Tailwind's `w-44`.
    fn w_44(self) -> Self {
        self.w(176.0)
    }
    /// `192px` width. Tailwind's `w-48`.
    fn w_48(self) -> Self {
        self.w(192.0)
    }
    /// `208px` width. Tailwind's `w-52`.
    fn w_52(self) -> Self {
        self.w(208.0)
    }
    /// `224px` width. Tailwind's `w-56`.
    fn w_56(self) -> Self {
        self.w(224.0)
    }
    /// `240px` width. Tailwind's `w-60`.
    fn w_60(self) -> Self {
        self.w(240.0)
    }
    /// `256px` width. Tailwind's `w-64`.
    fn w_64(self) -> Self {
        self.w(256.0)
    }
    /// `288px` width. Tailwind's `w-72`.
    fn w_72(self) -> Self {
        self.w(288.0)
    }
    /// `320px` width. Tailwind's `w-80`.
    fn w_80(self) -> Self {
        self.w(320.0)
    }
    /// `384px` width. Tailwind's `w-96`.
    fn w_96(self) -> Self {
        self.w(384.0)
    }
    /// `1px` width. Tailwind's `w-px`.
    fn w_px(self) -> Self {
        self.w(1.0)
    }

    /// Shrink the width to content. Tailwind's `w-auto` / `w-fit`.
    fn w_auto(mut self) -> Self {
        self.get_style().width = Size::Inner;
        self
    }
    /// Fill the parent's width. Tailwind's `w-full`.
    fn w_full(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Fill the viewport width (approximated as 100% of parent). Tailwind's `w-screen`.
    fn w_screen(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Half the parent's width. Tailwind's `w-1/2`.
    fn w_half(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(50.0));
        self
    }

    // Fractions of the parent's width. `_N_D` = `N/D`, so `w_1_5` is
    // `w-1/5` (20%), distinct from the `_p5` half-step `w_1_p5` = `w-1.5` (6px).
    /// 1/3 of the parent's width (`33.333%`). Tailwind's `w-1/3`.
    fn w_1_3(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0 / 3.0));
        self
    }
    /// 2/3 of the parent's width (`66.667%`). Tailwind's `w-2/3`.
    fn w_2_3(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(200.0 / 3.0));
        self
    }
    /// 1/4 of the parent's width (`25.0%`). Tailwind's `w-1/4`.
    fn w_1_4(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(25.0));
        self
    }
    /// 2/4 of the parent's width (`50.0%`). Tailwind's `w-2/4`.
    fn w_2_4(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(50.0));
        self
    }
    /// 3/4 of the parent's width (`75.0%`). Tailwind's `w-3/4`.
    fn w_3_4(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(75.0));
        self
    }
    /// 1/5 of the parent's width (`20.0%`). Tailwind's `w-1/5`.
    fn w_1_5(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(20.0));
        self
    }
    /// 2/5 of the parent's width (`40.0%`). Tailwind's `w-2/5`.
    fn w_2_5(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(40.0));
        self
    }
    /// 3/5 of the parent's width (`60.0%`). Tailwind's `w-3/5`.
    fn w_3_5(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(60.0));
        self
    }
    /// 4/5 of the parent's width (`80.0%`). Tailwind's `w-4/5`.
    fn w_4_5(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(80.0));
        self
    }
    /// 1/6 of the parent's width (`16.667%`). Tailwind's `w-1/6`.
    fn w_1_6(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0 / 6.0));
        self
    }
    /// 2/6 of the parent's width (`33.333%`). Tailwind's `w-2/6`.
    fn w_2_6(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(200.0 / 6.0));
        self
    }
    /// 3/6 of the parent's width (`50.0%`). Tailwind's `w-3/6`.
    fn w_3_6(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(50.0));
        self
    }
    /// 4/6 of the parent's width (`66.667%`). Tailwind's `w-4/6`.
    fn w_4_6(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(400.0 / 6.0));
        self
    }
    /// 5/6 of the parent's width (`83.333%`). Tailwind's `w-5/6`.
    fn w_5_6(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(500.0 / 6.0));
        self
    }
    /// 1/12 of the parent's width (`8.333%`). Tailwind's `w-1/12`.
    fn w_1_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0 / 12.0));
        self
    }
    /// 2/12 of the parent's width (`16.667%`). Tailwind's `w-2/12`.
    fn w_2_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(200.0 / 12.0));
        self
    }
    /// 3/12 of the parent's width (`25.0%`). Tailwind's `w-3/12`.
    fn w_3_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(25.0));
        self
    }
    /// 4/12 of the parent's width (`33.333%`). Tailwind's `w-4/12`.
    fn w_4_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(400.0 / 12.0));
        self
    }
    /// 5/12 of the parent's width (`41.667%`). Tailwind's `w-5/12`.
    fn w_5_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(500.0 / 12.0));
        self
    }
    /// 6/12 of the parent's width (`50.0%`). Tailwind's `w-6/12`.
    fn w_6_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(50.0));
        self
    }
    /// 7/12 of the parent's width (`58.333%`). Tailwind's `w-7/12`.
    fn w_7_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(700.0 / 12.0));
        self
    }
    /// 8/12 of the parent's width (`66.667%`). Tailwind's `w-8/12`.
    fn w_8_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(800.0 / 12.0));
        self
    }
    /// 9/12 of the parent's width (`75.0%`). Tailwind's `w-9/12`.
    fn w_9_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(75.0));
        self
    }
    /// 10/12 of the parent's width (`83.333%`). Tailwind's `w-10/12`.
    fn w_10_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(1000.0 / 12.0));
        self
    }
    /// 11/12 of the parent's width (`91.667%`). Tailwind's `w-11/12`.
    fn w_11_12(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(1100.0 / 12.0));
        self
    }

    // ---- height ----

    /// Set the height, in pixels.
    fn h(mut self, v: f32) -> Self {
        self.get_style().height = Size::Pixels(Length::new(v));
        self
    }
    /// Set the height to any `Size`.
    fn h_size(mut self, size: impl Into<Size>) -> Self {
        self.get_style().height = size.into();
        self
    }

    /// `0px` height. Tailwind's `h-0`.
    fn h_0(self) -> Self {
        self.h(0.0)
    }
    /// `2px` height. Tailwind's `h-0.5`.
    fn h_0_p5(self) -> Self {
        self.h(2.0)
    }
    /// `4px` height. Tailwind's `h-1`.
    fn h_1(self) -> Self {
        self.h(4.0)
    }
    /// `6px` height. Tailwind's `h-1.5`.
    fn h_1_p5(self) -> Self {
        self.h(6.0)
    }
    /// `8px` height. Tailwind's `h-2`.
    fn h_2(self) -> Self {
        self.h(8.0)
    }
    /// `10px` height. Tailwind's `h-2.5`.
    fn h_2_p5(self) -> Self {
        self.h(10.0)
    }
    /// `12px` height. Tailwind's `h-3`.
    fn h_3(self) -> Self {
        self.h(12.0)
    }
    /// `14px` height. Tailwind's `h-3.5`.
    fn h_3_p5(self) -> Self {
        self.h(14.0)
    }
    /// `16px` height. Tailwind's `h-4`.
    fn h_4(self) -> Self {
        self.h(16.0)
    }
    /// `20px` height. Tailwind's `h-5`.
    fn h_5(self) -> Self {
        self.h(20.0)
    }
    /// `24px` height. Tailwind's `h-6`.
    fn h_6(self) -> Self {
        self.h(24.0)
    }
    /// `28px` height. Tailwind's `h-7`.
    fn h_7(self) -> Self {
        self.h(28.0)
    }
    /// `32px` height. Tailwind's `h-8`.
    fn h_8(self) -> Self {
        self.h(32.0)
    }
    /// `36px` height. Tailwind's `h-9`.
    fn h_9(self) -> Self {
        self.h(36.0)
    }
    /// `40px` height. Tailwind's `h-10`.
    fn h_10(self) -> Self {
        self.h(40.0)
    }
    /// `44px` height. Tailwind's `h-11`.
    fn h_11(self) -> Self {
        self.h(44.0)
    }
    /// `48px` height. Tailwind's `h-12`.
    fn h_12(self) -> Self {
        self.h(48.0)
    }
    /// `56px` height. Tailwind's `h-14`.
    fn h_14(self) -> Self {
        self.h(56.0)
    }
    /// `64px` height. Tailwind's `h-16`.
    fn h_16(self) -> Self {
        self.h(64.0)
    }
    /// `80px` height. Tailwind's `h-20`.
    fn h_20(self) -> Self {
        self.h(80.0)
    }
    /// `96px` height. Tailwind's `h-24`.
    fn h_24(self) -> Self {
        self.h(96.0)
    }
    /// `112px` height. Tailwind's `h-28`.
    fn h_28(self) -> Self {
        self.h(112.0)
    }
    /// `128px` height. Tailwind's `h-32`.
    fn h_32(self) -> Self {
        self.h(128.0)
    }
    /// `144px` height. Tailwind's `h-36`.
    fn h_36(self) -> Self {
        self.h(144.0)
    }
    /// `160px` height. Tailwind's `h-40`.
    fn h_40(self) -> Self {
        self.h(160.0)
    }
    /// `176px` height. Tailwind's `h-44`.
    fn h_44(self) -> Self {
        self.h(176.0)
    }
    /// `192px` height. Tailwind's `h-48`.
    fn h_48(self) -> Self {
        self.h(192.0)
    }
    /// `208px` height. Tailwind's `h-52`.
    fn h_52(self) -> Self {
        self.h(208.0)
    }
    /// `224px` height. Tailwind's `h-56`.
    fn h_56(self) -> Self {
        self.h(224.0)
    }
    /// `240px` height. Tailwind's `h-60`.
    fn h_60(self) -> Self {
        self.h(240.0)
    }
    /// `256px` height. Tailwind's `h-64`.
    fn h_64(self) -> Self {
        self.h(256.0)
    }
    /// `288px` height. Tailwind's `h-72`.
    fn h_72(self) -> Self {
        self.h(288.0)
    }
    /// `320px` height. Tailwind's `h-80`.
    fn h_80(self) -> Self {
        self.h(320.0)
    }
    /// `384px` height. Tailwind's `h-96`.
    fn h_96(self) -> Self {
        self.h(384.0)
    }
    /// `1px` height. Tailwind's `h-px`.
    fn h_px(self) -> Self {
        self.h(1.0)
    }

    /// Shrink the height to content. Tailwind's `h-auto` / `h-fit`.
    fn h_auto(mut self) -> Self {
        self.get_style().height = Size::Inner;
        self
    }
    /// Fill the parent's height. Tailwind's `h-full`.
    fn h_full(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Fill the viewport height (approximated as 100% of parent). Tailwind's `h-screen`.
    fn h_screen(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Half the parent's height. Tailwind's `h-1/2`.
    fn h_half(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(50.0));
        self
    }

    // Fractions of the parent's height. `_N_D` = `N/D`, so `h_1_5` is
    // `h-1/5` (20%), distinct from the `_p5` half-step `h_1_p5` = `h-1.5` (6px).
    /// 1/3 of the parent's height (`33.333%`). Tailwind's `h-1/3`.
    fn h_1_3(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0 / 3.0));
        self
    }
    /// 2/3 of the parent's height (`66.667%`). Tailwind's `h-2/3`.
    fn h_2_3(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(200.0 / 3.0));
        self
    }
    /// 1/4 of the parent's height (`25.0%`). Tailwind's `h-1/4`.
    fn h_1_4(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(25.0));
        self
    }
    /// 2/4 of the parent's height (`50.0%`). Tailwind's `h-2/4`.
    fn h_2_4(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(50.0));
        self
    }
    /// 3/4 of the parent's height (`75.0%`). Tailwind's `h-3/4`.
    fn h_3_4(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(75.0));
        self
    }
    /// 1/5 of the parent's height (`20.0%`). Tailwind's `h-1/5`.
    fn h_1_5(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(20.0));
        self
    }
    /// 2/5 of the parent's height (`40.0%`). Tailwind's `h-2/5`.
    fn h_2_5(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(40.0));
        self
    }
    /// 3/5 of the parent's height (`60.0%`). Tailwind's `h-3/5`.
    fn h_3_5(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(60.0));
        self
    }
    /// 4/5 of the parent's height (`80.0%`). Tailwind's `h-4/5`.
    fn h_4_5(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(80.0));
        self
    }
    /// 1/6 of the parent's height (`16.667%`). Tailwind's `h-1/6`.
    fn h_1_6(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0 / 6.0));
        self
    }
    /// 2/6 of the parent's height (`33.333%`). Tailwind's `h-2/6`.
    fn h_2_6(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(200.0 / 6.0));
        self
    }
    /// 3/6 of the parent's height (`50.0%`). Tailwind's `h-3/6`.
    fn h_3_6(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(50.0));
        self
    }
    /// 4/6 of the parent's height (`66.667%`). Tailwind's `h-4/6`.
    fn h_4_6(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(400.0 / 6.0));
        self
    }
    /// 5/6 of the parent's height (`83.333%`). Tailwind's `h-5/6`.
    fn h_5_6(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(500.0 / 6.0));
        self
    }
    /// 1/12 of the parent's height (`8.333%`). Tailwind's `h-1/12`.
    fn h_1_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0 / 12.0));
        self
    }
    /// 2/12 of the parent's height (`16.667%`). Tailwind's `h-2/12`.
    fn h_2_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(200.0 / 12.0));
        self
    }
    /// 3/12 of the parent's height (`25.0%`). Tailwind's `h-3/12`.
    fn h_3_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(25.0));
        self
    }
    /// 4/12 of the parent's height (`33.333%`). Tailwind's `h-4/12`.
    fn h_4_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(400.0 / 12.0));
        self
    }
    /// 5/12 of the parent's height (`41.667%`). Tailwind's `h-5/12`.
    fn h_5_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(500.0 / 12.0));
        self
    }
    /// 6/12 of the parent's height (`50.0%`). Tailwind's `h-6/12`.
    fn h_6_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(50.0));
        self
    }
    /// 7/12 of the parent's height (`58.333%`). Tailwind's `h-7/12`.
    fn h_7_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(700.0 / 12.0));
        self
    }
    /// 8/12 of the parent's height (`66.667%`). Tailwind's `h-8/12`.
    fn h_8_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(800.0 / 12.0));
        self
    }
    /// 9/12 of the parent's height (`75.0%`). Tailwind's `h-9/12`.
    fn h_9_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(75.0));
        self
    }
    /// 10/12 of the parent's height (`83.333%`). Tailwind's `h-10/12`.
    fn h_10_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(1000.0 / 12.0));
        self
    }
    /// 11/12 of the parent's height (`91.667%`). Tailwind's `h-11/12`.
    fn h_11_12(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(1100.0 / 12.0));
        self
    }

    // ---- size (width + height) ----

    /// Set width and height to the same size, in pixels. Tailwind's `size-*`.
    fn size(mut self, v: f32) -> Self {
        self.get_style().width = Size::Pixels(Length::new(v));
        self.get_style().height = Size::Pixels(Length::new(v));
        self
    }
    /// `0px` width and height. Tailwind's `size-0`.
    fn size_0(self) -> Self {
        self.size(0.0)
    }
    /// `2px` width and height. Tailwind's `size-0.5`.
    fn size_0_p5(self) -> Self {
        self.size(2.0)
    }
    /// `4px` width and height. Tailwind's `size-1`.
    fn size_1(self) -> Self {
        self.size(4.0)
    }
    /// `6px` width and height. Tailwind's `size-1.5`.
    fn size_1_p5(self) -> Self {
        self.size(6.0)
    }
    /// `8px` width and height. Tailwind's `size-2`.
    fn size_2(self) -> Self {
        self.size(8.0)
    }
    /// `10px` width and height. Tailwind's `size-2.5`.
    fn size_2_p5(self) -> Self {
        self.size(10.0)
    }
    /// `12px` width and height. Tailwind's `size-3`.
    fn size_3(self) -> Self {
        self.size(12.0)
    }
    /// `14px` width and height. Tailwind's `size-3.5`.
    fn size_3_p5(self) -> Self {
        self.size(14.0)
    }
    /// `16px` width and height. Tailwind's `size-4`.
    fn size_4(self) -> Self {
        self.size(16.0)
    }
    /// `20px` width and height. Tailwind's `size-5`.
    fn size_5(self) -> Self {
        self.size(20.0)
    }
    /// `24px` width and height. Tailwind's `size-6`.
    fn size_6(self) -> Self {
        self.size(24.0)
    }
    /// `28px` width and height. Tailwind's `size-7`.
    fn size_7(self) -> Self {
        self.size(28.0)
    }
    /// `32px` width and height. Tailwind's `size-8`.
    fn size_8(self) -> Self {
        self.size(32.0)
    }
    /// `36px` width and height. Tailwind's `size-9`.
    fn size_9(self) -> Self {
        self.size(36.0)
    }
    /// `40px` width and height. Tailwind's `size-10`.
    fn size_10(self) -> Self {
        self.size(40.0)
    }
    /// `44px` width and height. Tailwind's `size-11`.
    fn size_11(self) -> Self {
        self.size(44.0)
    }
    /// `48px` width and height. Tailwind's `size-12`.
    fn size_12(self) -> Self {
        self.size(48.0)
    }
    /// `56px` width and height. Tailwind's `size-14`.
    fn size_14(self) -> Self {
        self.size(56.0)
    }
    /// `64px` width and height. Tailwind's `size-16`.
    fn size_16(self) -> Self {
        self.size(64.0)
    }
    /// `80px` width and height. Tailwind's `size-20`.
    fn size_20(self) -> Self {
        self.size(80.0)
    }
    /// `96px` width and height. Tailwind's `size-24`.
    fn size_24(self) -> Self {
        self.size(96.0)
    }
    /// `112px` width and height. Tailwind's `size-28`.
    fn size_28(self) -> Self {
        self.size(112.0)
    }
    /// `128px` width and height. Tailwind's `size-32`.
    fn size_32(self) -> Self {
        self.size(128.0)
    }
    /// `144px` width and height. Tailwind's `size-36`.
    fn size_36(self) -> Self {
        self.size(144.0)
    }
    /// `160px` width and height. Tailwind's `size-40`.
    fn size_40(self) -> Self {
        self.size(160.0)
    }
    /// `176px` width and height. Tailwind's `size-44`.
    fn size_44(self) -> Self {
        self.size(176.0)
    }
    /// `192px` width and height. Tailwind's `size-48`.
    fn size_48(self) -> Self {
        self.size(192.0)
    }
    /// `208px` width and height. Tailwind's `size-52`.
    fn size_52(self) -> Self {
        self.size(208.0)
    }
    /// `224px` width and height. Tailwind's `size-56`.
    fn size_56(self) -> Self {
        self.size(224.0)
    }
    /// `240px` width and height. Tailwind's `size-60`.
    fn size_60(self) -> Self {
        self.size(240.0)
    }
    /// `256px` width and height. Tailwind's `size-64`.
    fn size_64(self) -> Self {
        self.size(256.0)
    }
    /// `288px` width and height. Tailwind's `size-72`.
    fn size_72(self) -> Self {
        self.size(288.0)
    }
    /// `320px` width and height. Tailwind's `size-80`.
    fn size_80(self) -> Self {
        self.size(320.0)
    }
    /// `384px` width and height. Tailwind's `size-96`.
    fn size_96(self) -> Self {
        self.size(384.0)
    }
    /// `1px` width and height. Tailwind's `size-px`.
    fn size_px(self) -> Self {
        self.size(1.0)
    }
    /// Shrink width and height to content. Tailwind's `size-auto` / `size-fit`.
    fn size_auto(mut self) -> Self {
        self.get_style().width = Size::Inner;
        self.get_style().height = Size::Inner;
        self
    }
    /// Fill the parent's width and height. Tailwind's `size-full`.
    fn size_full(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0));
        self.get_style().height = Size::Percentage(Length::new(100.0));
        self
    }

    // ---- min-width ----

    /// Set the minimum width, in pixels.
    fn min_w(mut self, v: f32) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(v));
        self
    }
    /// `0px` minimum width. Tailwind's `min-w-0`.
    fn min_w_0(self) -> Self {
        self.min_w(0.0)
    }
    /// `2px` minimum width. Tailwind's `min-w-0.5`.
    fn min_w_0_p5(self) -> Self {
        self.min_w(2.0)
    }
    /// `4px` minimum width. Tailwind's `min-w-1`.
    fn min_w_1(self) -> Self {
        self.min_w(4.0)
    }
    /// `6px` minimum width. Tailwind's `min-w-1.5`.
    fn min_w_1_p5(self) -> Self {
        self.min_w(6.0)
    }
    /// `8px` minimum width. Tailwind's `min-w-2`.
    fn min_w_2(self) -> Self {
        self.min_w(8.0)
    }
    /// `10px` minimum width. Tailwind's `min-w-2.5`.
    fn min_w_2_p5(self) -> Self {
        self.min_w(10.0)
    }
    /// `12px` minimum width. Tailwind's `min-w-3`.
    fn min_w_3(self) -> Self {
        self.min_w(12.0)
    }
    /// `14px` minimum width. Tailwind's `min-w-3.5`.
    fn min_w_3_p5(self) -> Self {
        self.min_w(14.0)
    }
    /// `16px` minimum width. Tailwind's `min-w-4`.
    fn min_w_4(self) -> Self {
        self.min_w(16.0)
    }
    /// `20px` minimum width. Tailwind's `min-w-5`.
    fn min_w_5(self) -> Self {
        self.min_w(20.0)
    }
    /// `24px` minimum width. Tailwind's `min-w-6`.
    fn min_w_6(self) -> Self {
        self.min_w(24.0)
    }
    /// `28px` minimum width. Tailwind's `min-w-7`.
    fn min_w_7(self) -> Self {
        self.min_w(28.0)
    }
    /// `32px` minimum width. Tailwind's `min-w-8`.
    fn min_w_8(self) -> Self {
        self.min_w(32.0)
    }
    /// `36px` minimum width. Tailwind's `min-w-9`.
    fn min_w_9(self) -> Self {
        self.min_w(36.0)
    }
    /// `40px` minimum width. Tailwind's `min-w-10`.
    fn min_w_10(self) -> Self {
        self.min_w(40.0)
    }
    /// `44px` minimum width. Tailwind's `min-w-11`.
    fn min_w_11(self) -> Self {
        self.min_w(44.0)
    }
    /// `48px` minimum width. Tailwind's `min-w-12`.
    fn min_w_12(self) -> Self {
        self.min_w(48.0)
    }
    /// `56px` minimum width. Tailwind's `min-w-14`.
    fn min_w_14(self) -> Self {
        self.min_w(56.0)
    }
    /// `64px` minimum width. Tailwind's `min-w-16`.
    fn min_w_16(self) -> Self {
        self.min_w(64.0)
    }
    /// `80px` minimum width. Tailwind's `min-w-20`.
    fn min_w_20(self) -> Self {
        self.min_w(80.0)
    }
    /// `96px` minimum width. Tailwind's `min-w-24`.
    fn min_w_24(self) -> Self {
        self.min_w(96.0)
    }
    /// `112px` minimum width. Tailwind's `min-w-28`.
    fn min_w_28(self) -> Self {
        self.min_w(112.0)
    }
    /// `128px` minimum width. Tailwind's `min-w-32`.
    fn min_w_32(self) -> Self {
        self.min_w(128.0)
    }
    /// `144px` minimum width. Tailwind's `min-w-36`.
    fn min_w_36(self) -> Self {
        self.min_w(144.0)
    }
    /// `160px` minimum width. Tailwind's `min-w-40`.
    fn min_w_40(self) -> Self {
        self.min_w(160.0)
    }
    /// `176px` minimum width. Tailwind's `min-w-44`.
    fn min_w_44(self) -> Self {
        self.min_w(176.0)
    }
    /// `192px` minimum width. Tailwind's `min-w-48`.
    fn min_w_48(self) -> Self {
        self.min_w(192.0)
    }
    /// `208px` minimum width. Tailwind's `min-w-52`.
    fn min_w_52(self) -> Self {
        self.min_w(208.0)
    }
    /// `224px` minimum width. Tailwind's `min-w-56`.
    fn min_w_56(self) -> Self {
        self.min_w(224.0)
    }
    /// `240px` minimum width. Tailwind's `min-w-60`.
    fn min_w_60(self) -> Self {
        self.min_w(240.0)
    }
    /// `256px` minimum width. Tailwind's `min-w-64`.
    fn min_w_64(self) -> Self {
        self.min_w(256.0)
    }
    /// `288px` minimum width. Tailwind's `min-w-72`.
    fn min_w_72(self) -> Self {
        self.min_w(288.0)
    }
    /// `320px` minimum width. Tailwind's `min-w-80`.
    fn min_w_80(self) -> Self {
        self.min_w(320.0)
    }
    /// `384px` minimum width. Tailwind's `min-w-96`.
    fn min_w_96(self) -> Self {
        self.min_w(384.0)
    }
    /// `1px` minimum width. Tailwind's `min-w-px`.
    fn min_w_px(self) -> Self {
        self.min_w(1.0)
    }
    /// Content-sized minimum width. Tailwind's `min-w-fit`.
    fn min_w_fit(mut self) -> Self {
        self.get_style().min_width = Size::Inner;
        self
    }
    /// Minimum width of 100% of the parent. Tailwind's `min-w-full`.
    fn min_w_full(mut self) -> Self {
        self.get_style().min_width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Minimum width of the viewport (approximated as 100% of parent). Tailwind's `min-w-screen`.
    fn min_w_screen(mut self) -> Self {
        self.get_style().min_width = Size::Percentage(Length::new(100.0));
        self
    }

    // ---- min-height ----

    /// Set the minimum height, in pixels.
    fn min_h(mut self, v: f32) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(v));
        self
    }
    /// `0px` minimum height. Tailwind's `min-h-0`.
    fn min_h_0(self) -> Self {
        self.min_h(0.0)
    }
    /// `2px` minimum height. Tailwind's `min-h-0.5`.
    fn min_h_0_p5(self) -> Self {
        self.min_h(2.0)
    }
    /// `4px` minimum height. Tailwind's `min-h-1`.
    fn min_h_1(self) -> Self {
        self.min_h(4.0)
    }
    /// `6px` minimum height. Tailwind's `min-h-1.5`.
    fn min_h_1_p5(self) -> Self {
        self.min_h(6.0)
    }
    /// `8px` minimum height. Tailwind's `min-h-2`.
    fn min_h_2(self) -> Self {
        self.min_h(8.0)
    }
    /// `10px` minimum height. Tailwind's `min-h-2.5`.
    fn min_h_2_p5(self) -> Self {
        self.min_h(10.0)
    }
    /// `12px` minimum height. Tailwind's `min-h-3`.
    fn min_h_3(self) -> Self {
        self.min_h(12.0)
    }
    /// `14px` minimum height. Tailwind's `min-h-3.5`.
    fn min_h_3_p5(self) -> Self {
        self.min_h(14.0)
    }
    /// `16px` minimum height. Tailwind's `min-h-4`.
    fn min_h_4(self) -> Self {
        self.min_h(16.0)
    }
    /// `20px` minimum height. Tailwind's `min-h-5`.
    fn min_h_5(self) -> Self {
        self.min_h(20.0)
    }
    /// `24px` minimum height. Tailwind's `min-h-6`.
    fn min_h_6(self) -> Self {
        self.min_h(24.0)
    }
    /// `28px` minimum height. Tailwind's `min-h-7`.
    fn min_h_7(self) -> Self {
        self.min_h(28.0)
    }
    /// `32px` minimum height. Tailwind's `min-h-8`.
    fn min_h_8(self) -> Self {
        self.min_h(32.0)
    }
    /// `36px` minimum height. Tailwind's `min-h-9`.
    fn min_h_9(self) -> Self {
        self.min_h(36.0)
    }
    /// `40px` minimum height. Tailwind's `min-h-10`.
    fn min_h_10(self) -> Self {
        self.min_h(40.0)
    }
    /// `44px` minimum height. Tailwind's `min-h-11`.
    fn min_h_11(self) -> Self {
        self.min_h(44.0)
    }
    /// `48px` minimum height. Tailwind's `min-h-12`.
    fn min_h_12(self) -> Self {
        self.min_h(48.0)
    }
    /// `56px` minimum height. Tailwind's `min-h-14`.
    fn min_h_14(self) -> Self {
        self.min_h(56.0)
    }
    /// `64px` minimum height. Tailwind's `min-h-16`.
    fn min_h_16(self) -> Self {
        self.min_h(64.0)
    }
    /// `80px` minimum height. Tailwind's `min-h-20`.
    fn min_h_20(self) -> Self {
        self.min_h(80.0)
    }
    /// `96px` minimum height. Tailwind's `min-h-24`.
    fn min_h_24(self) -> Self {
        self.min_h(96.0)
    }
    /// `112px` minimum height. Tailwind's `min-h-28`.
    fn min_h_28(self) -> Self {
        self.min_h(112.0)
    }
    /// `128px` minimum height. Tailwind's `min-h-32`.
    fn min_h_32(self) -> Self {
        self.min_h(128.0)
    }
    /// `144px` minimum height. Tailwind's `min-h-36`.
    fn min_h_36(self) -> Self {
        self.min_h(144.0)
    }
    /// `160px` minimum height. Tailwind's `min-h-40`.
    fn min_h_40(self) -> Self {
        self.min_h(160.0)
    }
    /// `176px` minimum height. Tailwind's `min-h-44`.
    fn min_h_44(self) -> Self {
        self.min_h(176.0)
    }
    /// `192px` minimum height. Tailwind's `min-h-48`.
    fn min_h_48(self) -> Self {
        self.min_h(192.0)
    }
    /// `208px` minimum height. Tailwind's `min-h-52`.
    fn min_h_52(self) -> Self {
        self.min_h(208.0)
    }
    /// `224px` minimum height. Tailwind's `min-h-56`.
    fn min_h_56(self) -> Self {
        self.min_h(224.0)
    }
    /// `240px` minimum height. Tailwind's `min-h-60`.
    fn min_h_60(self) -> Self {
        self.min_h(240.0)
    }
    /// `256px` minimum height. Tailwind's `min-h-64`.
    fn min_h_64(self) -> Self {
        self.min_h(256.0)
    }
    /// `288px` minimum height. Tailwind's `min-h-72`.
    fn min_h_72(self) -> Self {
        self.min_h(288.0)
    }
    /// `320px` minimum height. Tailwind's `min-h-80`.
    fn min_h_80(self) -> Self {
        self.min_h(320.0)
    }
    /// `384px` minimum height. Tailwind's `min-h-96`.
    fn min_h_96(self) -> Self {
        self.min_h(384.0)
    }
    /// `1px` minimum height. Tailwind's `min-h-px`.
    fn min_h_px(self) -> Self {
        self.min_h(1.0)
    }
    /// Content-sized minimum height. Tailwind's `min-h-fit`.
    fn min_h_fit(mut self) -> Self {
        self.get_style().min_height = Size::Inner;
        self
    }
    /// Minimum height of 100% of the parent. Tailwind's `min-h-full`.
    fn min_h_full(mut self) -> Self {
        self.get_style().min_height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Minimum height of the viewport (approximated as 100% of parent). Tailwind's `min-h-screen`.
    fn min_h_screen(mut self) -> Self {
        self.get_style().min_height = Size::Percentage(Length::new(100.0));
        self
    }

    // ---- max-width ----

    /// Set the maximum width, in pixels.
    fn max_w(mut self, v: f32) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(v));
        self
    }
    /// `0px` maximum width. Tailwind's `max-w-0`.
    fn max_w_0(self) -> Self {
        self.max_w(0.0)
    }
    /// `2px` maximum width. Tailwind's `max-w-0.5`.
    fn max_w_0_p5(self) -> Self {
        self.max_w(2.0)
    }
    /// `4px` maximum width. Tailwind's `max-w-1`.
    fn max_w_1(self) -> Self {
        self.max_w(4.0)
    }
    /// `6px` maximum width. Tailwind's `max-w-1.5`.
    fn max_w_1_p5(self) -> Self {
        self.max_w(6.0)
    }
    /// `8px` maximum width. Tailwind's `max-w-2`.
    fn max_w_2(self) -> Self {
        self.max_w(8.0)
    }
    /// `10px` maximum width. Tailwind's `max-w-2.5`.
    fn max_w_2_p5(self) -> Self {
        self.max_w(10.0)
    }
    /// `12px` maximum width. Tailwind's `max-w-3`.
    fn max_w_3(self) -> Self {
        self.max_w(12.0)
    }
    /// `14px` maximum width. Tailwind's `max-w-3.5`.
    fn max_w_3_p5(self) -> Self {
        self.max_w(14.0)
    }
    /// `16px` maximum width. Tailwind's `max-w-4`.
    fn max_w_4(self) -> Self {
        self.max_w(16.0)
    }
    /// `20px` maximum width. Tailwind's `max-w-5`.
    fn max_w_5(self) -> Self {
        self.max_w(20.0)
    }
    /// `24px` maximum width. Tailwind's `max-w-6`.
    fn max_w_6(self) -> Self {
        self.max_w(24.0)
    }
    /// `28px` maximum width. Tailwind's `max-w-7`.
    fn max_w_7(self) -> Self {
        self.max_w(28.0)
    }
    /// `32px` maximum width. Tailwind's `max-w-8`.
    fn max_w_8(self) -> Self {
        self.max_w(32.0)
    }
    /// `36px` maximum width. Tailwind's `max-w-9`.
    fn max_w_9(self) -> Self {
        self.max_w(36.0)
    }
    /// `40px` maximum width. Tailwind's `max-w-10`.
    fn max_w_10(self) -> Self {
        self.max_w(40.0)
    }
    /// `44px` maximum width. Tailwind's `max-w-11`.
    fn max_w_11(self) -> Self {
        self.max_w(44.0)
    }
    /// `48px` maximum width. Tailwind's `max-w-12`.
    fn max_w_12(self) -> Self {
        self.max_w(48.0)
    }
    /// `56px` maximum width. Tailwind's `max-w-14`.
    fn max_w_14(self) -> Self {
        self.max_w(56.0)
    }
    /// `64px` maximum width. Tailwind's `max-w-16`.
    fn max_w_16(self) -> Self {
        self.max_w(64.0)
    }
    /// `80px` maximum width. Tailwind's `max-w-20`.
    fn max_w_20(self) -> Self {
        self.max_w(80.0)
    }
    /// `96px` maximum width. Tailwind's `max-w-24`.
    fn max_w_24(self) -> Self {
        self.max_w(96.0)
    }
    /// `112px` maximum width. Tailwind's `max-w-28`.
    fn max_w_28(self) -> Self {
        self.max_w(112.0)
    }
    /// `128px` maximum width. Tailwind's `max-w-32`.
    fn max_w_32(self) -> Self {
        self.max_w(128.0)
    }
    /// `144px` maximum width. Tailwind's `max-w-36`.
    fn max_w_36(self) -> Self {
        self.max_w(144.0)
    }
    /// `160px` maximum width. Tailwind's `max-w-40`.
    fn max_w_40(self) -> Self {
        self.max_w(160.0)
    }
    /// `176px` maximum width. Tailwind's `max-w-44`.
    fn max_w_44(self) -> Self {
        self.max_w(176.0)
    }
    /// `192px` maximum width. Tailwind's `max-w-48`.
    fn max_w_48(self) -> Self {
        self.max_w(192.0)
    }
    /// `208px` maximum width. Tailwind's `max-w-52`.
    fn max_w_52(self) -> Self {
        self.max_w(208.0)
    }
    /// `224px` maximum width. Tailwind's `max-w-56`.
    fn max_w_56(self) -> Self {
        self.max_w(224.0)
    }
    /// `240px` maximum width. Tailwind's `max-w-60`.
    fn max_w_60(self) -> Self {
        self.max_w(240.0)
    }
    /// `256px` maximum width. Tailwind's `max-w-64`.
    fn max_w_64(self) -> Self {
        self.max_w(256.0)
    }
    /// `288px` maximum width. Tailwind's `max-w-72`.
    fn max_w_72(self) -> Self {
        self.max_w(288.0)
    }
    /// `320px` maximum width. Tailwind's `max-w-80`.
    fn max_w_80(self) -> Self {
        self.max_w(320.0)
    }
    /// `384px` maximum width. Tailwind's `max-w-96`.
    fn max_w_96(self) -> Self {
        self.max_w(384.0)
    }
    /// `1px` maximum width. Tailwind's `max-w-px`.
    fn max_w_px(self) -> Self {
        self.max_w(1.0)
    }
    /// Content-sized maximum width. Tailwind's `max-w-fit`.
    fn max_w_fit(mut self) -> Self {
        self.get_style().max_width = Size::Inner;
        self
    }
    /// Maximum width of 100% of the parent. Tailwind's `max-w-full`.
    fn max_w_full(mut self) -> Self {
        self.get_style().max_width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Maximum width of the viewport (approximated as 100% of parent). Tailwind's `max-w-screen`.
    fn max_w_screen(mut self) -> Self {
        self.get_style().max_width = Size::Percentage(Length::new(100.0));
        self
    }
    /// `320px` maximum width (container size). Tailwind's `max-w-xs`.
    fn max_w_xs(self) -> Self {
        self.max_w(320.0)
    }
    /// `384px` maximum width (container size). Tailwind's `max-w-sm`.
    fn max_w_sm(self) -> Self {
        self.max_w(384.0)
    }
    /// `448px` maximum width (container size). Tailwind's `max-w-md`.
    fn max_w_md(self) -> Self {
        self.max_w(448.0)
    }
    /// `512px` maximum width (container size). Tailwind's `max-w-lg`.
    fn max_w_lg(self) -> Self {
        self.max_w(512.0)
    }
    /// `576px` maximum width (container size). Tailwind's `max-w-xl`.
    fn max_w_xl(self) -> Self {
        self.max_w(576.0)
    }
    /// `672px` maximum width (container size). Tailwind's `max-w-2xl`.
    fn max_w_2xl(self) -> Self {
        self.max_w(672.0)
    }
    /// `768px` maximum width (container size). Tailwind's `max-w-3xl`.
    fn max_w_3xl(self) -> Self {
        self.max_w(768.0)
    }
    /// `896px` maximum width (container size). Tailwind's `max-w-4xl`.
    fn max_w_4xl(self) -> Self {
        self.max_w(896.0)
    }
    /// `1024px` maximum width (container size). Tailwind's `max-w-5xl`.
    fn max_w_5xl(self) -> Self {
        self.max_w(1024.0)
    }
    /// `1152px` maximum width (container size). Tailwind's `max-w-6xl`.
    fn max_w_6xl(self) -> Self {
        self.max_w(1152.0)
    }
    /// `1280px` maximum width (container size). Tailwind's `max-w-7xl`.
    fn max_w_7xl(self) -> Self {
        self.max_w(1280.0)
    }

    // ---- max-height ----

    /// Set the maximum height, in pixels.
    fn max_h(mut self, v: f32) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(v));
        self
    }
    /// `0px` maximum height. Tailwind's `max-h-0`.
    fn max_h_0(self) -> Self {
        self.max_h(0.0)
    }
    /// `2px` maximum height. Tailwind's `max-h-0.5`.
    fn max_h_0_p5(self) -> Self {
        self.max_h(2.0)
    }
    /// `4px` maximum height. Tailwind's `max-h-1`.
    fn max_h_1(self) -> Self {
        self.max_h(4.0)
    }
    /// `6px` maximum height. Tailwind's `max-h-1.5`.
    fn max_h_1_p5(self) -> Self {
        self.max_h(6.0)
    }
    /// `8px` maximum height. Tailwind's `max-h-2`.
    fn max_h_2(self) -> Self {
        self.max_h(8.0)
    }
    /// `10px` maximum height. Tailwind's `max-h-2.5`.
    fn max_h_2_p5(self) -> Self {
        self.max_h(10.0)
    }
    /// `12px` maximum height. Tailwind's `max-h-3`.
    fn max_h_3(self) -> Self {
        self.max_h(12.0)
    }
    /// `14px` maximum height. Tailwind's `max-h-3.5`.
    fn max_h_3_p5(self) -> Self {
        self.max_h(14.0)
    }
    /// `16px` maximum height. Tailwind's `max-h-4`.
    fn max_h_4(self) -> Self {
        self.max_h(16.0)
    }
    /// `20px` maximum height. Tailwind's `max-h-5`.
    fn max_h_5(self) -> Self {
        self.max_h(20.0)
    }
    /// `24px` maximum height. Tailwind's `max-h-6`.
    fn max_h_6(self) -> Self {
        self.max_h(24.0)
    }
    /// `28px` maximum height. Tailwind's `max-h-7`.
    fn max_h_7(self) -> Self {
        self.max_h(28.0)
    }
    /// `32px` maximum height. Tailwind's `max-h-8`.
    fn max_h_8(self) -> Self {
        self.max_h(32.0)
    }
    /// `36px` maximum height. Tailwind's `max-h-9`.
    fn max_h_9(self) -> Self {
        self.max_h(36.0)
    }
    /// `40px` maximum height. Tailwind's `max-h-10`.
    fn max_h_10(self) -> Self {
        self.max_h(40.0)
    }
    /// `44px` maximum height. Tailwind's `max-h-11`.
    fn max_h_11(self) -> Self {
        self.max_h(44.0)
    }
    /// `48px` maximum height. Tailwind's `max-h-12`.
    fn max_h_12(self) -> Self {
        self.max_h(48.0)
    }
    /// `56px` maximum height. Tailwind's `max-h-14`.
    fn max_h_14(self) -> Self {
        self.max_h(56.0)
    }
    /// `64px` maximum height. Tailwind's `max-h-16`.
    fn max_h_16(self) -> Self {
        self.max_h(64.0)
    }
    /// `80px` maximum height. Tailwind's `max-h-20`.
    fn max_h_20(self) -> Self {
        self.max_h(80.0)
    }
    /// `96px` maximum height. Tailwind's `max-h-24`.
    fn max_h_24(self) -> Self {
        self.max_h(96.0)
    }
    /// `112px` maximum height. Tailwind's `max-h-28`.
    fn max_h_28(self) -> Self {
        self.max_h(112.0)
    }
    /// `128px` maximum height. Tailwind's `max-h-32`.
    fn max_h_32(self) -> Self {
        self.max_h(128.0)
    }
    /// `144px` maximum height. Tailwind's `max-h-36`.
    fn max_h_36(self) -> Self {
        self.max_h(144.0)
    }
    /// `160px` maximum height. Tailwind's `max-h-40`.
    fn max_h_40(self) -> Self {
        self.max_h(160.0)
    }
    /// `176px` maximum height. Tailwind's `max-h-44`.
    fn max_h_44(self) -> Self {
        self.max_h(176.0)
    }
    /// `192px` maximum height. Tailwind's `max-h-48`.
    fn max_h_48(self) -> Self {
        self.max_h(192.0)
    }
    /// `208px` maximum height. Tailwind's `max-h-52`.
    fn max_h_52(self) -> Self {
        self.max_h(208.0)
    }
    /// `224px` maximum height. Tailwind's `max-h-56`.
    fn max_h_56(self) -> Self {
        self.max_h(224.0)
    }
    /// `240px` maximum height. Tailwind's `max-h-60`.
    fn max_h_60(self) -> Self {
        self.max_h(240.0)
    }
    /// `256px` maximum height. Tailwind's `max-h-64`.
    fn max_h_64(self) -> Self {
        self.max_h(256.0)
    }
    /// `288px` maximum height. Tailwind's `max-h-72`.
    fn max_h_72(self) -> Self {
        self.max_h(288.0)
    }
    /// `320px` maximum height. Tailwind's `max-h-80`.
    fn max_h_80(self) -> Self {
        self.max_h(320.0)
    }
    /// `384px` maximum height. Tailwind's `max-h-96`.
    fn max_h_96(self) -> Self {
        self.max_h(384.0)
    }
    /// `1px` maximum height. Tailwind's `max-h-px`.
    fn max_h_px(self) -> Self {
        self.max_h(1.0)
    }
    /// Content-sized maximum height. Tailwind's `max-h-fit`.
    fn max_h_fit(mut self) -> Self {
        self.get_style().max_height = Size::Inner;
        self
    }
    /// Maximum height of 100% of the parent. Tailwind's `max-h-full`.
    fn max_h_full(mut self) -> Self {
        self.get_style().max_height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Maximum height of the viewport (approximated as 100% of parent). Tailwind's `max-h-screen`.
    fn max_h_screen(mut self) -> Self {
        self.get_style().max_height = Size::Percentage(Length::new(100.0));
        self
    }

    // ---- gap ----

    /// Set the gap between adjacent children, in pixels.
    fn gap(mut self, gap: f32) -> Self {
        self.get_style().spacing = Length::new(gap);
        self
    }
    /// `0px` gap between children. Tailwind's `gap-0`.
    fn gap_0(self) -> Self {
        self.gap(0.0)
    }
    /// `2px` gap between children. Tailwind's `gap-0.5`.
    fn gap_0_p5(self) -> Self {
        self.gap(2.0)
    }
    /// `4px` gap between children. Tailwind's `gap-1`.
    fn gap_1(self) -> Self {
        self.gap(4.0)
    }
    /// `6px` gap between children. Tailwind's `gap-1.5`.
    fn gap_1_p5(self) -> Self {
        self.gap(6.0)
    }
    /// `8px` gap between children. Tailwind's `gap-2`.
    fn gap_2(self) -> Self {
        self.gap(8.0)
    }
    /// `10px` gap between children. Tailwind's `gap-2.5`.
    fn gap_2_p5(self) -> Self {
        self.gap(10.0)
    }
    /// `12px` gap between children. Tailwind's `gap-3`.
    fn gap_3(self) -> Self {
        self.gap(12.0)
    }
    /// `14px` gap between children. Tailwind's `gap-3.5`.
    fn gap_3_p5(self) -> Self {
        self.gap(14.0)
    }
    /// `16px` gap between children. Tailwind's `gap-4`.
    fn gap_4(self) -> Self {
        self.gap(16.0)
    }
    /// `20px` gap between children. Tailwind's `gap-5`.
    fn gap_5(self) -> Self {
        self.gap(20.0)
    }
    /// `24px` gap between children. Tailwind's `gap-6`.
    fn gap_6(self) -> Self {
        self.gap(24.0)
    }
    /// `28px` gap between children. Tailwind's `gap-7`.
    fn gap_7(self) -> Self {
        self.gap(28.0)
    }
    /// `32px` gap between children. Tailwind's `gap-8`.
    fn gap_8(self) -> Self {
        self.gap(32.0)
    }
    /// `36px` gap between children. Tailwind's `gap-9`.
    fn gap_9(self) -> Self {
        self.gap(36.0)
    }
    /// `40px` gap between children. Tailwind's `gap-10`.
    fn gap_10(self) -> Self {
        self.gap(40.0)
    }
    /// `44px` gap between children. Tailwind's `gap-11`.
    fn gap_11(self) -> Self {
        self.gap(44.0)
    }
    /// `48px` gap between children. Tailwind's `gap-12`.
    fn gap_12(self) -> Self {
        self.gap(48.0)
    }
    /// `56px` gap between children. Tailwind's `gap-14`.
    fn gap_14(self) -> Self {
        self.gap(56.0)
    }
    /// `64px` gap between children. Tailwind's `gap-16`.
    fn gap_16(self) -> Self {
        self.gap(64.0)
    }
    /// `80px` gap between children. Tailwind's `gap-20`.
    fn gap_20(self) -> Self {
        self.gap(80.0)
    }
    /// `96px` gap between children. Tailwind's `gap-24`.
    fn gap_24(self) -> Self {
        self.gap(96.0)
    }
    /// `112px` gap between children. Tailwind's `gap-28`.
    fn gap_28(self) -> Self {
        self.gap(112.0)
    }
    /// `128px` gap between children. Tailwind's `gap-32`.
    fn gap_32(self) -> Self {
        self.gap(128.0)
    }
    /// `144px` gap between children. Tailwind's `gap-36`.
    fn gap_36(self) -> Self {
        self.gap(144.0)
    }
    /// `160px` gap between children. Tailwind's `gap-40`.
    fn gap_40(self) -> Self {
        self.gap(160.0)
    }
    /// `176px` gap between children. Tailwind's `gap-44`.
    fn gap_44(self) -> Self {
        self.gap(176.0)
    }
    /// `192px` gap between children. Tailwind's `gap-48`.
    fn gap_48(self) -> Self {
        self.gap(192.0)
    }
    /// `208px` gap between children. Tailwind's `gap-52`.
    fn gap_52(self) -> Self {
        self.gap(208.0)
    }
    /// `224px` gap between children. Tailwind's `gap-56`.
    fn gap_56(self) -> Self {
        self.gap(224.0)
    }
    /// `240px` gap between children. Tailwind's `gap-60`.
    fn gap_60(self) -> Self {
        self.gap(240.0)
    }
    /// `256px` gap between children. Tailwind's `gap-64`.
    fn gap_64(self) -> Self {
        self.gap(256.0)
    }
    /// `288px` gap between children. Tailwind's `gap-72`.
    fn gap_72(self) -> Self {
        self.gap(288.0)
    }
    /// `320px` gap between children. Tailwind's `gap-80`.
    fn gap_80(self) -> Self {
        self.gap(320.0)
    }
    /// `384px` gap between children. Tailwind's `gap-96`.
    fn gap_96(self) -> Self {
        self.gap(384.0)
    }
    /// `1px` gap between children. Tailwind's `gap-px`.
    fn gap_px(self) -> Self {
        self.gap(1.0)
    }

    // ---- z-index / layer ----

    /// Sets the painting layer. `Layer::Relative(n)` is the closest thing to `z-index: n`,
    /// but it is relative to the *parent's* layer. Use `Overlay` for modals and dropdowns.
    fn layer(mut self, layer: impl Into<Layer>) -> Self {
        self.get_style().layer = layer.into();
        self
    }

    // ---- z-index: theme-driven shortcuts ----
    //
    // Each shortcut resolves `zIndex.<key>` from the active theme,
    // falling back to the Tailwind numeric scale.

    /// Auto — behaves as `z-0`. Resolves theme `zIndex.auto` then `zIndex.0`.
    fn z_auto(self) -> Self {
        let theme = active_theme();
        let n = theme.z_index_or("auto", theme.z_index_or("0", 0));
        self.layer(Layer::Relative(n as i16))
    }
    /// Theme `zIndex.0` (fallback 0).
    fn z_0(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("0", 0) as i16))
    }
    /// Theme `zIndex.10` (fallback 10).
    fn z_10(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("10", 10) as i16))
    }
    /// Theme `zIndex.20` (fallback 20).
    fn z_20(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("20", 20) as i16))
    }
    /// Theme `zIndex.30` (fallback 30).
    fn z_30(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("30", 30) as i16))
    }
    /// Theme `zIndex.40` (fallback 40).
    fn z_40(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("40", 40) as i16))
    }
    /// Theme `zIndex.50` (fallback 50).
    fn z_50(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("50", 50) as i16))
    }
    /// Theme `zIndex.-10` (fallback −10).
    fn z_neg_10(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("-10", -10) as i16))
    }
    /// Theme `zIndex.-20` (fallback −20).
    fn z_neg_20(self) -> Self {
        self.layer(Layer::Relative(active_theme().z_index_or("-20", -20) as i16))
    }
    /// Overlay layer — always on top. No theme lookup (Freya semantic).
    fn z_overlay(self) -> Self {
        self.layer(Layer::Overlay)
    }
    /// Overlay with an explicit level. No theme lookup (Freya semantic).
    fn z_overlay_level(self, level: u8) -> Self {
        self.layer(Layer::OverlayLevel(level))
    }

    /// Approximate Tailwind's `ring-<width>` with a 1px border on all sides.
    fn ring_1(self) -> Self {
        self.border_width(1.0)
    }
    /// Approximate Tailwind's `ring-2`.
    fn ring_2(self) -> Self {
        self.border_width(2.0)
    }
    /// Approximate Tailwind's `ring-[3px]`.
    fn ring_3(self) -> Self {
        self.border_width(3.0)
    }
    /// Approximate Tailwind's `ring` — 3px in Tailwind v4.
    fn ring(self) -> Self {
        self.ring_3()
    }
    /// No ring — clears the border like Tailwind's `ring-0`.
    fn ring_0(self) -> Self {
        self.border_width(0.0)
    }

    /// Set ring color from a theme token (`"ring"`, `"destructive"`, …).
    fn ring_token(self, theme: &Theme, key: &str) -> Self {
        self.border_token(theme, key)
    }

    // ---- border ----
    //
    // `.border(..)` replaces the whole `Option<Border>` (matches the
    // `Style::border` field). Width, color, and alignment helpers create
    // a default border if none exists yet, so the order in the chain
    // doesn't matter:
    //
    // ```
    // div().border_2().border_color(gray_200())
    // // same result as
    // div().border_color(gray_200()).border_2()
    // ```

    /// Replace the border wholesale (matches the `Style::border` field).
    fn border(mut self, border: impl Into<Option<Border>>) -> Self {
        self.get_style().border = border.into();
        self
    }

    /// Remove the border entirely (`border: None`). For "0px width but
    /// still a border object" use [`border_0`](StyledExt::border_0).
    fn border_none(mut self) -> Self {
        self.get_style().border = None;
        self
    }

    // ---- width (all sides) ----

    /// 0px border (Tailwind `border-0`).
    fn border_0(self) -> Self {
        self.border_width(0.0)
    }

    /// 1px border (Tailwind `border`).
    fn border_1(self) -> Self {
        self.border_width(1.0)
    }

    /// 2px border (Tailwind `border-2`).
    fn border_2(self) -> Self {
        self.border_width(2.0)
    }

    /// 3px border (no direct Tailwind equivalent — Tailwind's `border-*`
    /// jumps 2 → 4; `ring-[3px]` is the usual source of this value).
    fn border_3(self) -> Self {
        self.border_width(3.0)
    }

    /// 4px border (Tailwind `border-4`).
    fn border_4(self) -> Self {
        self.border_width(4.0)
    }

    /// 8px border (Tailwind `border-8`).
    fn border_8(self) -> Self {
        self.border_width(8.0)
    }

    // ---- width (per side) ----

    /// Top border with the given width (Tailwind `border-t`).
    fn border_t(mut self, width: impl Into<f32>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.top = width.into();
        self
    }

    /// Right border with the given width (Tailwind `border-r`).
    fn border_r(mut self, width: impl Into<f32>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.right = width.into();
        self
    }

    /// Bottom border with the given width (Tailwind `border-b`).
    fn border_b(mut self, width: impl Into<f32>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.bottom = width.into();
        self
    }

    /// Left border with the given width (Tailwind `border-l`).
    fn border_l(mut self, width: impl Into<f32>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.left = width.into();
        self
    }

    /// Left + right border with the given width (Tailwind `border-x`).
    fn border_x(mut self, width: impl Into<f32>) -> Self {
        let w = width.into();
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.left = w;
        border.width.right = w;
        self
    }

    /// Top + bottom border with the given width (Tailwind `border-y`).
    fn border_y(mut self, width: impl Into<f32>) -> Self {
        let w = width.into();
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width.top = w;
        border.width.bottom = w;
        self
    }

    /// Set the whole [`BorderWidth`] at once (per-side widths).
    fn border_width(mut self, width: impl Into<BorderWidth>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.width = width.into();
        self
    }

    // ---- alignment (freya-specific, no Tailwind equivalent) ----

    /// Border drawn inside the element's bounds (`BorderAlignment::Inner`).
    fn border_inner(self) -> Self {
        self.border_alignment(BorderAlignment::Inner)
    }

    /// Border drawn outside the element's bounds (`BorderAlignment::Outer`).
    fn border_outer(self) -> Self {
        self.border_alignment(BorderAlignment::Outer)
    }

    /// Border centered on the element's edge (`BorderAlignment::Center`).
    fn border_center(self) -> Self {
        self.border_alignment(BorderAlignment::Center)
    }

    /// Set how the border is aligned to the element's edge.
    fn border_alignment(mut self, alignment: impl Into<BorderAlignment>) -> Self {
        let style = self.get_style();
        let border = style
            .border
            .get_or_insert_with(|| Border::new().fill(black()));
        border.alignment = alignment.into();
        self
    }

    // ---- variants (pseudo-state style patches) ----
    //
    // These *register* a callback on the style. The primitive element
    // applies them per render via `style.variants.apply_all(&live, style)`
    // whenever the corresponding interaction state is active. Ending a
    // variant is simply *not* pushing its kind into `live` on the next
    // render — never persist an applied style back into state.
    //
    // Priority among simultaneously-live variants is defined by
    // `Variant::priority` (disabled > active > checked > selected >
    // focus_within > focus_visible > focus > hover), regardless of the
    // order they were registered or listed in `live`.

    /// Patch the style while pointer is over the element. Tailwind's `hover:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// text("Hi").bg(neutral_100()).hover(|s| s.bg(neutral_200()))
    /// ```
    fn hover<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Hover, Arc::new(f));
        self
    }

    /// Patch the style while the element has focus. Tailwind's `focus:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// input().focus(|s| s.bg(white()))
    /// ```
    fn focus<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Focus, Arc::new(f));
        self
    }

    /// Patch the style while the element has focus via the keyboard. Tailwind's `focus-visible:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// button().focus_visible(|s| s.opacity(0.9))
    /// ```
    fn focus_visible<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::FocusVisible, Arc::new(f));
        self
    }

    /// Patch the style while the element or any descendant has focus. Tailwind's `focus-within:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// card().focus_within(|s| s.bg(neutral_50()))
    /// ```
    fn focus_within<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::FocusWithin, Arc::new(f));
        self
    }

    /// Patch the style while the element is being pressed. Tailwind's `active:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// button().active(|s| s.scale_95())
    /// ```
    fn active<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Active, Arc::new(f));
        self
    }

    /// Patch the style while the element is disabled. Tailwind's `disabled:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// button().disabled(|s| s.opacity_50().cursor_not_allowed())
    /// ```
    fn disabled<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Disabled, Arc::new(f));
        self
    }

    /// Patch the style while the element is checked (checkbox, switch, ...). Tailwind's `checked:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// checkbox().checked(|s| s.bg(neutral_900()))
    /// ```
    fn checked<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Checked, Arc::new(f));
        self
    }

    /// Patch the style while the element is selected. Tailwind's `selected:*`.
    ///
    /// The closure receives the base [`Style`] and returns the patched one.
    ///
    /// ```
    /// item().selected(|s| s.bg(neutral_100()))
    /// ```
    fn selected<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::Selected, Arc::new(f));
        self
    }

    /// Register a **mode-conditional** patch that fires when the current
    /// theme is dark (`theme.appearance == Appearance::Dark`).
    ///
    /// The `Dark` condition is added to the active set automatically by
    /// [`Style::resolve_variants`](Style::resolve_variants) —
    /// you do **not** need to pass it in the interaction flags yourself.
    ///
    /// # Precedence
    ///
    /// `Dark` has the lowest priority (`5`), so any interaction state
    /// overrides it:
    ///
    /// - `hover > dark` → `.dark(|s| s.bg(a)).hover(|s| s.bg(b))` paints
    ///   `b` while hovering in dark mode.
    /// - `dark:hover` (compound) beats plain `hover` because of the
    ///   specificity tie-break in `StyleVariants::apply_all`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let style = Style::default()
    ///     .bg_white()
    ///     .dark(|s| s.bg_black())
    ///     .hover(|s| s.bg_blue_500());
    /// ```
    fn dark<F>(self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        let mut s = self;
        s.get_style().variants.push(StyleVariant::Dark, Arc::new(f));
        s
    }

    /// Patch the style when the element contains an SVG child.
    ///
    /// Only fires when the element's own style declares this variant and
    /// a descendant `Icon` signals `HasKind::Svg` to the nearest
    /// [`HasContext`]. For explicit control (no automatic tracking),
    /// use `.with_has_svg(cond)` instead.
    fn has_svg<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::HasSvg, Arc::new(f));
        self
    }

    /// Patch the style when the element contains an Image child.
    fn has_img<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::HasImg, Arc::new(f));
        self
    }

    /// Patch the style when the element contains a checked checkbox / radio.
    fn has_checked<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::HasChecked, Arc::new(f));
        self
    }

    /// Patch the style when the element contains a selected child.
    fn has_selected<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::HasSelected, Arc::new(f));
        self
    }

    /// Patch the style when the element contains a disabled child.
    fn has_disabled<F>(mut self, f: F) -> Self
    where
        F: Fn(Style) -> Style + Send + Sync + 'static,
    {
        self.get_style()
            .variants
            .push(StyleVariant::HasDisabled, Arc::new(f));
        self
    }

    fn style(self, style: impl Into<StyledString>) -> Self {
        style.into().apply(self)
    }
}

pub trait ThemedExt: StyledExt + Sized {
    /// `background` ← `theme.fill(key)`
    fn bg_token(mut self, theme: &Theme, key: &str) -> Self {
        if let Some(fill) = theme.fill(key) {
            self.get_style().background = fill;
        }
        self
    }

    /// `background` ← state-aware component token
    fn bg_component(
        mut self,
        theme: &Theme,
        path: &str,
        state: ComponentState,
        field: &str,
    ) -> Self {
        if let Some(fill) = theme.state_fill(path, state, field) {
            self.get_style().background = fill;
        }
        self
    }

    /// Set the border color from a semantic token (`"border"`, `"ring"`,
    /// `"input"`, …). Creates a 1px border if none exists yet.
    fn border_token(mut self, theme: &Theme, key: &str) -> Self {
        if let Some(color) = theme.color(key) {
            let style = self.get_style();
            let border = style.border.get_or_insert_with(|| Border::new().width(1.0));
            border.fill = color.into();
        }
        self
    }

    /// Replace the font stack with `typography.fontFamily.<key>` from the
    /// given theme. No-op if the key is absent.
    fn font_family_token(mut self, theme: &Theme, key: &str) -> Self {
        if let Some(stack) = theme.font_family_stack(key) {
            self.get_style().font_families = stack;
        }
        self
    }

    /// `text_color` ← `theme.fill(key)`
    fn text_token(mut self, theme: &Theme, key: &str) -> Self {
        if let Some(fill) = theme.fill(key) {
            self.get_style().text_color = fill;
        }
        self
    }

    /// `text_color` ← state-aware component token
    fn text_component(
        mut self,
        theme: &Theme,
        path: &str,
        state: ComponentState,
        field: &str,
    ) -> Self {
        if let Some(fill) = theme.state_fill(path, state, field) {
            self.get_style().text_color = fill;
        }
        self
    }

    fn radius_token(self, theme: &Theme, key: &str) -> Self {
        self.radius(theme.radius(key))
    }
    fn p_token(self, theme: &Theme, key: &str) -> Self {
        self.p(theme.spacing(key))
    }
    fn gap_token(self, theme: &Theme, key: &str) -> Self {
        self.gap(theme.spacing(key))
    }
}

impl<T: StyledExt> ThemedExt for T {}

/// Map a CSS font-weight number to Freya's `FontWeight`.
pub fn weight_to_font_weight(w: u16) -> FontWeight {
    match w {
        100 => FontWeight::THIN,
        200 => FontWeight::EXTRA_LIGHT,
        300 => FontWeight::LIGHT,
        400 => FontWeight::NORMAL,
        500 => FontWeight::MEDIUM,
        600 => FontWeight::SEMI_BOLD,
        700 => FontWeight::BOLD,
        800 => FontWeight::EXTRA_BOLD,
        900 => FontWeight::BLACK,
        _ => FontWeight::NORMAL,
    }
}
