use crate::prelude::CursorIcon;
use crate::styling::colors::{StyledColor, black};
use crate::styling::style::Style;
use freya_core::data::Overflow;
use freya_core::prelude::{
    CornerRadius, FontFeature, FontSize, FontSlant, FontWeight, FontWidth, LetterSpacing, Scale,
    TextAlign, TextDecoration, TextOverflow, TextShadow, TransformOrigin,
};
use std::borrow::Cow;
use torin::prelude::{Gaps, Length};
use torin::size::Size;

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
/// - **Margin** — [`m`](Self::m), [`mx`](Self::mx), … and the `m_*`/`mx_*`/… shortcuts
/// - **Width / height** — the `w_*` and `h_*` shortcuts
/// - **Min / max dimensions** — the `min_w_*` / `max_w_*` / `min_h_*` / `max_h_*` shortcuts
/// - **Gap** — [`gap`](Self::gap) and the `gap_*` shortcuts
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
    /// spacing scales automatically with [`font_size`](Self::font_size).
    /// Call this **after** the font-size setter to get the expected result:
    ///
    /// ```ignore
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

    /// Replace the font stack with the default sans-serif stack.
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

    /// Appends a text shadow.
    fn text_shadow(mut self, text_shadow: impl Into<TextShadow>) -> Self {
        self.get_style().text_shadows.push(text_shadow.into());
        self
    }

    /// Extra-extra-small shadow — `0 1px 0` at 15 % black.
    fn text_shadow_2xs(self) -> Self {
        self.text_shadow(TextShadow::new(
            black().opacity(0.15).into(),
            (0.0, 1.0),
            0.0,
        ))
    }

    /// Extra-small shadow — `0 1px 1px` at 15 % black.
    fn text_shadow_xs(self) -> Self {
        self.text_shadow(TextShadow::new(
            black().opacity(0.15).into(),
            (0.0, 1.0),
            1.0,
        ))
    }

    /// Small shadow — `0 1px 2px` at 7.5 % black.
    fn text_shadow_sm(self) -> Self {
        self.text_shadow(TextShadow::new(
            black().opacity(0.075).into(),
            (0.0, 1.0),
            2.0,
        ))
    }

    /// Medium shadow — `0 2px 4px` at 10 % black.
    fn text_shadow_md(self) -> Self {
        self.text_shadow(TextShadow::new(
            black().opacity(0.1).into(),
            (0.0, 2.0),
            4.0,
        ))
    }

    /// Large shadow — `0 4px 8px` at 10 % black.
    fn text_shadow_lg(self) -> Self {
        self.text_shadow(TextShadow::new(
            black().opacity(0.1).into(),
            (0.0, 4.0),
            8.0,
        ))
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

    /// No rounding — `0px`. Tailwind's `rounded-none`.
    fn rounded_none(self) -> Self {
        self.radius(0.0)
    }
    /// Extra-small — `2px`. Tailwind's `rounded-xs`.
    fn rounded_xs(self) -> Self {
        self.radius(2.0)
    }
    /// Small — `4px`. Tailwind's `rounded-sm`.
    fn rounded_sm(self) -> Self {
        self.radius(4.0)
    }
    /// Medium — `6px`. Tailwind's `rounded-md`.
    fn rounded_md(self) -> Self {
        self.radius(6.0)
    }
    /// Large — `8px`. Tailwind's `rounded-lg`.
    fn rounded_lg(self) -> Self {
        self.radius(8.0)
    }
    /// Extra-large — `12px`. Tailwind's `rounded-xl`.
    fn rounded_xl(self) -> Self {
        self.radius(12.0)
    }
    /// 2× extra-large — `16px`. Tailwind's `rounded-2xl`.
    fn rounded_2xl(self) -> Self {
        self.radius(16.0)
    }
    /// 3× extra-large — `24px`. Tailwind's `rounded-3xl`.
    fn rounded_3xl(self) -> Self {
        self.radius(24.0)
    }
    /// 4× extra-large — `32px`. Tailwind's `rounded-4xl`.
    fn rounded_4xl(self) -> Self {
        self.radius(32.0)
    }
    /// Pill / circle — `9999px`. Tailwind's `rounded-full`.
    fn rounded_full(self) -> Self {
        self.radius(9999.0)
    }

    // ---- top edge (rounded-t-*) ----

    fn rounded_t_none(self) -> Self {
        self.radius_t(0.0)
    }
    fn rounded_t_xs(self) -> Self {
        self.radius_t(2.0)
    }
    fn rounded_t_sm(self) -> Self {
        self.radius_t(4.0)
    }
    fn rounded_t_md(self) -> Self {
        self.radius_t(6.0)
    }
    fn rounded_t_lg(self) -> Self {
        self.radius_t(8.0)
    }
    fn rounded_t_xl(self) -> Self {
        self.radius_t(12.0)
    }
    fn rounded_t_2xl(self) -> Self {
        self.radius_t(16.0)
    }
    fn rounded_t_3xl(self) -> Self {
        self.radius_t(24.0)
    }
    fn rounded_t_4xl(self) -> Self {
        self.radius_t(32.0)
    }
    fn rounded_t_full(self) -> Self {
        self.radius_t(9999.0)
    }

    // ---- right edge (rounded-r-*) ----

    fn rounded_r_none(self) -> Self {
        self.radius_r(0.0)
    }
    fn rounded_r_xs(self) -> Self {
        self.radius_r(2.0)
    }
    fn rounded_r_sm(self) -> Self {
        self.radius_r(4.0)
    }
    fn rounded_r_md(self) -> Self {
        self.radius_r(6.0)
    }
    fn rounded_r_lg(self) -> Self {
        self.radius_r(8.0)
    }
    fn rounded_r_xl(self) -> Self {
        self.radius_r(12.0)
    }
    fn rounded_r_2xl(self) -> Self {
        self.radius_r(16.0)
    }
    fn rounded_r_3xl(self) -> Self {
        self.radius_r(24.0)
    }
    fn rounded_r_4xl(self) -> Self {
        self.radius_r(32.0)
    }
    fn rounded_r_full(self) -> Self {
        self.radius_r(9999.0)
    }

    // ---- bottom edge (rounded-b-*) ----

    fn rounded_b_none(self) -> Self {
        self.radius_b(0.0)
    }
    fn rounded_b_xs(self) -> Self {
        self.radius_b(2.0)
    }
    fn rounded_b_sm(self) -> Self {
        self.radius_b(4.0)
    }
    fn rounded_b_md(self) -> Self {
        self.radius_b(6.0)
    }
    fn rounded_b_lg(self) -> Self {
        self.radius_b(8.0)
    }
    fn rounded_b_xl(self) -> Self {
        self.radius_b(12.0)
    }
    fn rounded_b_2xl(self) -> Self {
        self.radius_b(16.0)
    }
    fn rounded_b_3xl(self) -> Self {
        self.radius_b(24.0)
    }
    fn rounded_b_4xl(self) -> Self {
        self.radius_b(32.0)
    }
    fn rounded_b_full(self) -> Self {
        self.radius_b(9999.0)
    }

    // ---- left edge (rounded-l-*) ----

    fn rounded_l_none(self) -> Self {
        self.radius_l(0.0)
    }
    fn rounded_l_xs(self) -> Self {
        self.radius_l(2.0)
    }
    fn rounded_l_sm(self) -> Self {
        self.radius_l(4.0)
    }
    fn rounded_l_md(self) -> Self {
        self.radius_l(6.0)
    }
    fn rounded_l_lg(self) -> Self {
        self.radius_l(8.0)
    }
    fn rounded_l_xl(self) -> Self {
        self.radius_l(12.0)
    }
    fn rounded_l_2xl(self) -> Self {
        self.radius_l(16.0)
    }
    fn rounded_l_3xl(self) -> Self {
        self.radius_l(24.0)
    }
    fn rounded_l_4xl(self) -> Self {
        self.radius_l(32.0)
    }
    fn rounded_l_full(self) -> Self {
        self.radius_l(9999.0)
    }

    // ---- top-left corner (rounded-tl-*) ----

    fn rounded_tl_none(self) -> Self {
        self.radius_tl(0.0)
    }
    fn rounded_tl_xs(self) -> Self {
        self.radius_tl(2.0)
    }
    fn rounded_tl_sm(self) -> Self {
        self.radius_tl(4.0)
    }
    fn rounded_tl_md(self) -> Self {
        self.radius_tl(6.0)
    }
    fn rounded_tl_lg(self) -> Self {
        self.radius_tl(8.0)
    }
    fn rounded_tl_xl(self) -> Self {
        self.radius_tl(12.0)
    }
    fn rounded_tl_2xl(self) -> Self {
        self.radius_tl(16.0)
    }
    fn rounded_tl_3xl(self) -> Self {
        self.radius_tl(24.0)
    }
    fn rounded_tl_4xl(self) -> Self {
        self.radius_tl(32.0)
    }
    fn rounded_tl_full(self) -> Self {
        self.radius_tl(9999.0)
    }

    // ---- top-right corner (rounded-tr-*) ----

    fn rounded_tr_none(self) -> Self {
        self.radius_tr(0.0)
    }
    fn rounded_tr_xs(self) -> Self {
        self.radius_tr(2.0)
    }
    fn rounded_tr_sm(self) -> Self {
        self.radius_tr(4.0)
    }
    fn rounded_tr_md(self) -> Self {
        self.radius_tr(6.0)
    }
    fn rounded_tr_lg(self) -> Self {
        self.radius_tr(8.0)
    }
    fn rounded_tr_xl(self) -> Self {
        self.radius_tr(12.0)
    }
    fn rounded_tr_2xl(self) -> Self {
        self.radius_tr(16.0)
    }
    fn rounded_tr_3xl(self) -> Self {
        self.radius_tr(24.0)
    }
    fn rounded_tr_4xl(self) -> Self {
        self.radius_tr(32.0)
    }
    fn rounded_tr_full(self) -> Self {
        self.radius_tr(9999.0)
    }

    // ---- bottom-right corner (rounded-br-*) ----

    fn rounded_br_none(self) -> Self {
        self.radius_br(0.0)
    }
    fn rounded_br_xs(self) -> Self {
        self.radius_br(2.0)
    }
    fn rounded_br_sm(self) -> Self {
        self.radius_br(4.0)
    }
    fn rounded_br_md(self) -> Self {
        self.radius_br(6.0)
    }
    fn rounded_br_lg(self) -> Self {
        self.radius_br(8.0)
    }
    fn rounded_br_xl(self) -> Self {
        self.radius_br(12.0)
    }
    fn rounded_br_2xl(self) -> Self {
        self.radius_br(16.0)
    }
    fn rounded_br_3xl(self) -> Self {
        self.radius_br(24.0)
    }
    fn rounded_br_4xl(self) -> Self {
        self.radius_br(32.0)
    }
    fn rounded_br_full(self) -> Self {
        self.radius_br(9999.0)
    }

    // ---- bottom-left corner (rounded-bl-*) ----

    fn rounded_bl_none(self) -> Self {
        self.radius_bl(0.0)
    }
    fn rounded_bl_xs(self) -> Self {
        self.radius_bl(2.0)
    }
    fn rounded_bl_sm(self) -> Self {
        self.radius_bl(4.0)
    }
    fn rounded_bl_md(self) -> Self {
        self.radius_bl(6.0)
    }
    fn rounded_bl_lg(self) -> Self {
        self.radius_bl(8.0)
    }
    fn rounded_bl_xl(self) -> Self {
        self.radius_bl(12.0)
    }
    fn rounded_bl_2xl(self) -> Self {
        self.radius_bl(16.0)
    }
    fn rounded_bl_3xl(self) -> Self {
        self.radius_bl(24.0)
    }
    fn rounded_bl_4xl(self) -> Self {
        self.radius_bl(32.0)
    }
    fn rounded_bl_full(self) -> Self {
        self.radius_bl(9999.0)
    }

    // ================================ layout ================================
    //
    // These mutate `Style`'s layout fields, not `LayoutData`. Your
    // element-build step is responsible for copying `Style::*` into
    // `LayoutData` before layout runs.

    // ---- padding ----

    /// Set padding on all four sides.
    fn p(mut self, v: f32) -> Self {
        self.get_style().padding = Gaps::new_all(v);
        self
    }
    /// Set horizontal padding (left + right), preserving top/bottom.
    fn px(mut self, x: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), x, g.bottom(), x);
        self
    }
    /// Set vertical padding (top + bottom), preserving left/right.
    fn py(mut self, y: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(y, g.right(), y, g.left());
        self
    }
    /// Set top padding, preserving the other three sides.
    fn pt(mut self, t: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(t, g.right(), g.bottom(), g.left());
        self
    }
    /// Set right padding, preserving the other three sides.
    fn pr(mut self, r: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), r, g.bottom(), g.left());
        self
    }
    /// Set bottom padding, preserving the other three sides.
    fn pb(mut self, b: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), g.right(), b, g.left());
        self
    }
    /// Set left padding, preserving the other three sides.
    fn pl(mut self, l: f32) -> Self {
        let g = self.get_style().padding;
        self.get_style().padding = Gaps::new(g.top(), g.right(), g.bottom(), l);
        self
    }

    fn p_0(self) -> Self {
        self.p(0.0)
    }
    fn p_1(self) -> Self {
        self.p(4.0)
    }
    fn p_2(self) -> Self {
        self.p(8.0)
    }
    fn p_3(self) -> Self {
        self.p(12.0)
    }
    fn p_4(self) -> Self {
        self.p(16.0)
    }
    fn p_5(self) -> Self {
        self.p(20.0)
    }
    fn p_6(self) -> Self {
        self.p(24.0)
    }
    fn p_8(self) -> Self {
        self.p(32.0)
    }
    fn p_10(self) -> Self {
        self.p(40.0)
    }
    fn p_12(self) -> Self {
        self.p(48.0)
    }
    fn p_16(self) -> Self {
        self.p(64.0)
    }
    fn p_20(self) -> Self {
        self.p(80.0)
    }
    fn p_24(self) -> Self {
        self.p(96.0)
    }

    fn px_0(self) -> Self {
        self.px(0.0)
    }
    fn px_1(self) -> Self {
        self.px(4.0)
    }
    fn px_2(self) -> Self {
        self.px(8.0)
    }
    fn px_3(self) -> Self {
        self.px(12.0)
    }
    fn px_4(self) -> Self {
        self.px(16.0)
    }
    fn px_5(self) -> Self {
        self.px(20.0)
    }
    fn px_6(self) -> Self {
        self.px(24.0)
    }
    fn px_8(self) -> Self {
        self.px(32.0)
    }
    fn px_10(self) -> Self {
        self.px(40.0)
    }
    fn px_12(self) -> Self {
        self.px(48.0)
    }
    fn px_16(self) -> Self {
        self.px(64.0)
    }
    fn px_20(self) -> Self {
        self.px(80.0)
    }
    fn px_24(self) -> Self {
        self.px(96.0)
    }

    fn py_0(self) -> Self {
        self.py(0.0)
    }
    fn py_1(self) -> Self {
        self.py(4.0)
    }
    fn py_2(self) -> Self {
        self.py(8.0)
    }
    fn py_3(self) -> Self {
        self.py(12.0)
    }
    fn py_4(self) -> Self {
        self.py(16.0)
    }
    fn py_5(self) -> Self {
        self.py(20.0)
    }
    fn py_6(self) -> Self {
        self.py(24.0)
    }
    fn py_8(self) -> Self {
        self.py(32.0)
    }
    fn py_10(self) -> Self {
        self.py(40.0)
    }
    fn py_12(self) -> Self {
        self.py(48.0)
    }
    fn py_16(self) -> Self {
        self.py(64.0)
    }
    fn py_20(self) -> Self {
        self.py(80.0)
    }
    fn py_24(self) -> Self {
        self.py(96.0)
    }

    fn pt_0(self) -> Self {
        self.pt(0.0)
    }
    fn pt_1(self) -> Self {
        self.pt(4.0)
    }
    fn pt_2(self) -> Self {
        self.pt(8.0)
    }
    fn pt_3(self) -> Self {
        self.pt(12.0)
    }
    fn pt_4(self) -> Self {
        self.pt(16.0)
    }
    fn pt_5(self) -> Self {
        self.pt(20.0)
    }
    fn pt_6(self) -> Self {
        self.pt(24.0)
    }
    fn pt_8(self) -> Self {
        self.pt(32.0)
    }
    fn pt_10(self) -> Self {
        self.pt(40.0)
    }
    fn pt_12(self) -> Self {
        self.pt(48.0)
    }
    fn pt_16(self) -> Self {
        self.pt(64.0)
    }
    fn pt_20(self) -> Self {
        self.pt(80.0)
    }
    fn pt_24(self) -> Self {
        self.pt(96.0)
    }

    fn pr_0(self) -> Self {
        self.pr(0.0)
    }
    fn pr_1(self) -> Self {
        self.pr(4.0)
    }
    fn pr_2(self) -> Self {
        self.pr(8.0)
    }
    fn pr_3(self) -> Self {
        self.pr(12.0)
    }
    fn pr_4(self) -> Self {
        self.pr(16.0)
    }
    fn pr_5(self) -> Self {
        self.pr(20.0)
    }
    fn pr_6(self) -> Self {
        self.pr(24.0)
    }
    fn pr_8(self) -> Self {
        self.pr(32.0)
    }
    fn pr_10(self) -> Self {
        self.pr(40.0)
    }
    fn pr_12(self) -> Self {
        self.pr(48.0)
    }
    fn pr_16(self) -> Self {
        self.pr(64.0)
    }
    fn pr_20(self) -> Self {
        self.pr(80.0)
    }
    fn pr_24(self) -> Self {
        self.pr(96.0)
    }

    fn pb_0(self) -> Self {
        self.pb(0.0)
    }
    fn pb_1(self) -> Self {
        self.pb(4.0)
    }
    fn pb_2(self) -> Self {
        self.pb(8.0)
    }
    fn pb_3(self) -> Self {
        self.pb(12.0)
    }
    fn pb_4(self) -> Self {
        self.pb(16.0)
    }
    fn pb_5(self) -> Self {
        self.pb(20.0)
    }
    fn pb_6(self) -> Self {
        self.pb(24.0)
    }
    fn pb_8(self) -> Self {
        self.pb(32.0)
    }
    fn pb_10(self) -> Self {
        self.pb(40.0)
    }
    fn pb_12(self) -> Self {
        self.pb(48.0)
    }
    fn pb_16(self) -> Self {
        self.pb(64.0)
    }
    fn pb_20(self) -> Self {
        self.pb(80.0)
    }
    fn pb_24(self) -> Self {
        self.pb(96.0)
    }

    fn pl_0(self) -> Self {
        self.pl(0.0)
    }
    fn pl_1(self) -> Self {
        self.pl(4.0)
    }
    fn pl_2(self) -> Self {
        self.pl(8.0)
    }
    fn pl_3(self) -> Self {
        self.pl(12.0)
    }
    fn pl_4(self) -> Self {
        self.pl(16.0)
    }
    fn pl_5(self) -> Self {
        self.pl(20.0)
    }
    fn pl_6(self) -> Self {
        self.pl(24.0)
    }
    fn pl_8(self) -> Self {
        self.pl(32.0)
    }
    fn pl_10(self) -> Self {
        self.pl(40.0)
    }
    fn pl_12(self) -> Self {
        self.pl(48.0)
    }
    fn pl_16(self) -> Self {
        self.pl(64.0)
    }
    fn pl_20(self) -> Self {
        self.pl(80.0)
    }
    fn pl_24(self) -> Self {
        self.pl(96.0)
    }

    // ---- margin ----

    /// Set margin on all four sides.
    fn m(mut self, v: f32) -> Self {
        self.get_style().margin = Gaps::new_all(v);
        self
    }
    fn mx(mut self, x: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), x, g.bottom(), x);
        self
    }
    fn my(mut self, y: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(y, g.right(), y, g.left());
        self
    }
    fn mt(mut self, t: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(t, g.right(), g.bottom(), g.left());
        self
    }
    fn mr(mut self, r: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), r, g.bottom(), g.left());
        self
    }
    fn mb(mut self, b: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), g.right(), b, g.left());
        self
    }
    fn ml(mut self, l: f32) -> Self {
        let g = self.get_style().margin;
        self.get_style().margin = Gaps::new(g.top(), g.right(), g.bottom(), l);
        self
    }

    fn m_0(self) -> Self {
        self.m(0.0)
    }
    fn m_1(self) -> Self {
        self.m(4.0)
    }
    fn m_2(self) -> Self {
        self.m(8.0)
    }
    fn m_3(self) -> Self {
        self.m(12.0)
    }
    fn m_4(self) -> Self {
        self.m(16.0)
    }
    fn m_5(self) -> Self {
        self.m(20.0)
    }
    fn m_6(self) -> Self {
        self.m(24.0)
    }
    fn m_8(self) -> Self {
        self.m(32.0)
    }
    fn m_10(self) -> Self {
        self.m(40.0)
    }
    fn m_12(self) -> Self {
        self.m(48.0)
    }
    fn m_16(self) -> Self {
        self.m(64.0)
    }
    fn m_20(self) -> Self {
        self.m(80.0)
    }
    fn m_24(self) -> Self {
        self.m(96.0)
    }

    fn mx_0(self) -> Self {
        self.mx(0.0)
    }
    fn mx_1(self) -> Self {
        self.mx(4.0)
    }
    fn mx_2(self) -> Self {
        self.mx(8.0)
    }
    fn mx_3(self) -> Self {
        self.mx(12.0)
    }
    fn mx_4(self) -> Self {
        self.mx(16.0)
    }
    fn mx_5(self) -> Self {
        self.mx(20.0)
    }
    fn mx_6(self) -> Self {
        self.mx(24.0)
    }
    fn mx_8(self) -> Self {
        self.mx(32.0)
    }
    fn mx_10(self) -> Self {
        self.mx(40.0)
    }
    fn mx_12(self) -> Self {
        self.mx(48.0)
    }
    fn mx_16(self) -> Self {
        self.mx(64.0)
    }
    fn mx_20(self) -> Self {
        self.mx(80.0)
    }
    fn mx_24(self) -> Self {
        self.mx(96.0)
    }

    fn my_0(self) -> Self {
        self.my(0.0)
    }
    fn my_1(self) -> Self {
        self.my(4.0)
    }
    fn my_2(self) -> Self {
        self.my(8.0)
    }
    fn my_3(self) -> Self {
        self.my(12.0)
    }
    fn my_4(self) -> Self {
        self.my(16.0)
    }
    fn my_5(self) -> Self {
        self.my(20.0)
    }
    fn my_6(self) -> Self {
        self.my(24.0)
    }
    fn my_8(self) -> Self {
        self.my(32.0)
    }
    fn my_10(self) -> Self {
        self.my(40.0)
    }
    fn my_12(self) -> Self {
        self.my(48.0)
    }
    fn my_16(self) -> Self {
        self.my(64.0)
    }
    fn my_20(self) -> Self {
        self.my(80.0)
    }
    fn my_24(self) -> Self {
        self.my(96.0)
    }

    fn mt_0(self) -> Self {
        self.mt(0.0)
    }
    fn mt_1(self) -> Self {
        self.mt(4.0)
    }
    fn mt_2(self) -> Self {
        self.mt(8.0)
    }
    fn mt_3(self) -> Self {
        self.mt(12.0)
    }
    fn mt_4(self) -> Self {
        self.mt(16.0)
    }
    fn mt_5(self) -> Self {
        self.mt(20.0)
    }
    fn mt_6(self) -> Self {
        self.mt(24.0)
    }
    fn mt_8(self) -> Self {
        self.mt(32.0)
    }
    fn mt_10(self) -> Self {
        self.mt(40.0)
    }
    fn mt_12(self) -> Self {
        self.mt(48.0)
    }
    fn mt_16(self) -> Self {
        self.mt(64.0)
    }
    fn mt_20(self) -> Self {
        self.mt(80.0)
    }
    fn mt_24(self) -> Self {
        self.mt(96.0)
    }

    fn mr_0(self) -> Self {
        self.mr(0.0)
    }
    fn mr_1(self) -> Self {
        self.mr(4.0)
    }
    fn mr_2(self) -> Self {
        self.mr(8.0)
    }
    fn mr_3(self) -> Self {
        self.mr(12.0)
    }
    fn mr_4(self) -> Self {
        self.mr(16.0)
    }
    fn mr_5(self) -> Self {
        self.mr(20.0)
    }
    fn mr_6(self) -> Self {
        self.mr(24.0)
    }
    fn mr_8(self) -> Self {
        self.mr(32.0)
    }
    fn mr_10(self) -> Self {
        self.mr(40.0)
    }
    fn mr_12(self) -> Self {
        self.mr(48.0)
    }
    fn mr_16(self) -> Self {
        self.mr(64.0)
    }
    fn mr_20(self) -> Self {
        self.mr(80.0)
    }
    fn mr_24(self) -> Self {
        self.mr(96.0)
    }

    fn mb_0(self) -> Self {
        self.mb(0.0)
    }
    fn mb_1(self) -> Self {
        self.mb(4.0)
    }
    fn mb_2(self) -> Self {
        self.mb(8.0)
    }
    fn mb_3(self) -> Self {
        self.mb(12.0)
    }
    fn mb_4(self) -> Self {
        self.mb(16.0)
    }
    fn mb_5(self) -> Self {
        self.mb(20.0)
    }
    fn mb_6(self) -> Self {
        self.mb(24.0)
    }
    fn mb_8(self) -> Self {
        self.mb(32.0)
    }
    fn mb_10(self) -> Self {
        self.mb(40.0)
    }
    fn mb_12(self) -> Self {
        self.mb(48.0)
    }
    fn mb_16(self) -> Self {
        self.mb(64.0)
    }
    fn mb_20(self) -> Self {
        self.mb(80.0)
    }
    fn mb_24(self) -> Self {
        self.mb(96.0)
    }

    fn ml_0(self) -> Self {
        self.ml(0.0)
    }
    fn ml_1(self) -> Self {
        self.ml(4.0)
    }
    fn ml_2(self) -> Self {
        self.ml(8.0)
    }
    fn ml_3(self) -> Self {
        self.ml(12.0)
    }
    fn ml_4(self) -> Self {
        self.ml(16.0)
    }
    fn ml_5(self) -> Self {
        self.ml(20.0)
    }
    fn ml_6(self) -> Self {
        self.ml(24.0)
    }
    fn ml_8(self) -> Self {
        self.ml(32.0)
    }
    fn ml_10(self) -> Self {
        self.ml(40.0)
    }
    fn ml_12(self) -> Self {
        self.ml(48.0)
    }
    fn ml_16(self) -> Self {
        self.ml(64.0)
    }
    fn ml_20(self) -> Self {
        self.ml(80.0)
    }
    fn ml_24(self) -> Self {
        self.ml(96.0)
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

    fn w_0(self) -> Self {
        self.w(0.0)
    }
    fn w_1(self) -> Self {
        self.w(4.0)
    }
    fn w_2(self) -> Self {
        self.w(8.0)
    }
    fn w_3(self) -> Self {
        self.w(12.0)
    }
    fn w_4(self) -> Self {
        self.w(16.0)
    }
    fn w_5(self) -> Self {
        self.w(20.0)
    }
    fn w_6(self) -> Self {
        self.w(24.0)
    }
    fn w_8(self) -> Self {
        self.w(32.0)
    }
    fn w_10(self) -> Self {
        self.w(40.0)
    }
    fn w_12(self) -> Self {
        self.w(48.0)
    }
    fn w_16(self) -> Self {
        self.w(64.0)
    }
    fn w_20(self) -> Self {
        self.w(80.0)
    }
    fn w_24(self) -> Self {
        self.w(96.0)
    }
    fn w_32(self) -> Self {
        self.w(128.0)
    }
    fn w_40(self) -> Self {
        self.w(160.0)
    }
    fn w_48(self) -> Self {
        self.w(192.0)
    }
    fn w_64(self) -> Self {
        self.w(256.0)
    }

    /// Shrink to content. Tailwind's `w-auto` / `w-fit`.
    fn w_auto(mut self) -> Self {
        self.get_style().width = Size::Inner;
        self
    }
    /// Fill the parent's width. Tailwind's `w-full`.
    fn w_full(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Fill viewport width (approximated as 100% of parent). Tailwind's `w-screen`.
    fn w_screen(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0));
        self
    }
    /// Half parent width. Tailwind's `w-1/2`.
    fn w_half(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(50.0));
        self
    }
    /// One third of parent width. Tailwind's `w-1/3`.
    fn w_1_3(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(100.0 / 3.0));
        self
    }
    /// Two thirds of parent width. Tailwind's `w-2/3`.
    fn w_2_3(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(200.0 / 3.0));
        self
    }
    /// One quarter of parent width. Tailwind's `w-1/4`.
    fn w_1_4(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(25.0));
        self
    }
    /// Three quarters of parent width. Tailwind's `w-3/4`.
    fn w_3_4(mut self) -> Self {
        self.get_style().width = Size::Percentage(Length::new(75.0));
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

    fn h_0(self) -> Self {
        self.h(0.0)
    }
    fn h_1(self) -> Self {
        self.h(4.0)
    }
    fn h_2(self) -> Self {
        self.h(8.0)
    }
    fn h_3(self) -> Self {
        self.h(12.0)
    }
    fn h_4(self) -> Self {
        self.h(16.0)
    }
    fn h_5(self) -> Self {
        self.h(20.0)
    }
    fn h_6(self) -> Self {
        self.h(24.0)
    }
    fn h_8(self) -> Self {
        self.h(32.0)
    }
    fn h_10(self) -> Self {
        self.h(40.0)
    }
    fn h_12(self) -> Self {
        self.h(48.0)
    }
    fn h_16(self) -> Self {
        self.h(64.0)
    }
    fn h_20(self) -> Self {
        self.h(80.0)
    }
    fn h_24(self) -> Self {
        self.h(96.0)
    }
    fn h_32(self) -> Self {
        self.h(128.0)
    }
    fn h_40(self) -> Self {
        self.h(160.0)
    }
    fn h_48(self) -> Self {
        self.h(192.0)
    }
    fn h_64(self) -> Self {
        self.h(256.0)
    }

    /// Shrink to content. Tailwind's `h-auto` / `h-fit`.
    fn h_auto(mut self) -> Self {
        self.get_style().height = Size::Inner;
        self
    }
    /// Fill the parent's height. Tailwind's `h-full`.
    fn h_full(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Fill viewport height (approximated as 100% of parent). Tailwind's `h-screen`.
    fn h_screen(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(100.0));
        self
    }
    /// Half parent height. Tailwind's `h-1/2`.
    fn h_half(mut self) -> Self {
        self.get_style().height = Size::Percentage(Length::new(50.0));
        self
    }

    // ---- min / max ----

    fn min_w_0(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(0.0));
        self
    }
    fn min_w_4(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(16.0));
        self
    }
    fn min_w_8(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(32.0));
        self
    }
    fn min_w_16(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(64.0));
        self
    }
    fn min_w_32(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(128.0));
        self
    }
    fn min_w_64(mut self) -> Self {
        self.get_style().min_width = Size::Pixels(Length::new(256.0));
        self
    }
    fn min_w_fit(mut self) -> Self {
        self.get_style().min_width = Size::Inner;
        self
    }
    fn min_w_full(mut self) -> Self {
        self.get_style().min_width = Size::Percentage(Length::new(100.0));
        self
    }

    fn min_h_0(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(0.0));
        self
    }
    fn min_h_4(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(16.0));
        self
    }
    fn min_h_8(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(32.0));
        self
    }
    fn min_h_16(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(64.0));
        self
    }
    fn min_h_32(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(128.0));
        self
    }
    fn min_h_64(mut self) -> Self {
        self.get_style().min_height = Size::Pixels(Length::new(256.0));
        self
    }
    fn min_h_fit(mut self) -> Self {
        self.get_style().min_height = Size::Inner;
        self
    }
    fn min_h_full(mut self) -> Self {
        self.get_style().min_height = Size::Percentage(Length::new(100.0));
        self
    }
    fn min_h_screen(mut self) -> Self {
        self.get_style().min_height = Size::Percentage(Length::new(100.0));
        self
    }

    fn max_w_0(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(0.0));
        self
    }
    fn max_w_4(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(16.0));
        self
    }
    fn max_w_8(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(32.0));
        self
    }
    fn max_w_16(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(64.0));
        self
    }
    fn max_w_32(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(128.0));
        self
    }
    fn max_w_64(mut self) -> Self {
        self.get_style().max_width = Size::Pixels(Length::new(256.0));
        self
    }
    fn max_w_fit(mut self) -> Self {
        self.get_style().max_width = Size::Inner;
        self
    }
    fn max_w_full(mut self) -> Self {
        self.get_style().max_width = Size::Percentage(Length::new(100.0));
        self
    }

    fn max_h_0(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(0.0));
        self
    }
    fn max_h_4(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(16.0));
        self
    }
    fn max_h_8(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(32.0));
        self
    }
    fn max_h_16(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(64.0));
        self
    }
    fn max_h_32(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(128.0));
        self
    }
    fn max_h_64(mut self) -> Self {
        self.get_style().max_height = Size::Pixels(Length::new(256.0));
        self
    }
    fn max_h_fit(mut self) -> Self {
        self.get_style().max_height = Size::Inner;
        self
    }
    fn max_h_full(mut self) -> Self {
        self.get_style().max_height = Size::Percentage(Length::new(100.0));
        self
    }

    // ---- gap ----

    /// Set the gap between adjacent children, in pixels.
    fn gap(mut self, gap: f32) -> Self {
        self.get_style().spacing = Length::new(gap);
        self
    }

    fn gap_0(self) -> Self {
        self.gap(0.0)
    }
    fn gap_1(self) -> Self {
        self.gap(4.0)
    }
    fn gap_2(self) -> Self {
        self.gap(8.0)
    }
    fn gap_3(self) -> Self {
        self.gap(12.0)
    }
    fn gap_4(self) -> Self {
        self.gap(16.0)
    }
    fn gap_5(self) -> Self {
        self.gap(20.0)
    }
    fn gap_6(self) -> Self {
        self.gap(24.0)
    }
    fn gap_8(self) -> Self {
        self.gap(32.0)
    }
    fn gap_10(self) -> Self {
        self.gap(40.0)
    }
    fn gap_12(self) -> Self {
        self.gap(48.0)
    }
    fn gap_16(self) -> Self {
        self.gap(64.0)
    }
    fn gap_20(self) -> Self {
        self.gap(80.0)
    }
    fn gap_24(self) -> Self {
        self.gap(96.0)
    }
}
