use crate::styling::colors::black;
use crate::styling::style_variants::StyleVariants;
use crate::styling::styled::StyledExt;
use crate::theme::active_theme;
use freya_core::data::Overflow;
use freya_core::prelude::{
    AspectRatio, CornerRadius, CursorIcon, Fill, FontFeature, FontSize, FontSlant, FontWeight,
    FontWidth, ImageCover, LetterSpacing, SamplingMode, Scale, TextAlign, TextDecoration,
    TextOverflow, TextShadow, TransformOrigin,
};
use std::borrow::Cow;
use torin::{
    content::Content,
    gaps::Gaps,
    prelude::{Alignment, Direction, Length, Position},
    size::Size,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    // ---- paint ----
    pub background: Fill,
    pub text_color: Fill,
    pub text_align: TextAlign,

    // ---- typography ----
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    pub letter_spacing: LetterSpacing,
    pub text_overflow: TextOverflow,
    pub font_features: Vec<FontFeature>,
    pub font_families: Vec<Cow<'static, str>>,
    pub font_style: FontSlant,
    pub font_stretch: FontWidth,
    pub line_height: f32,
    pub text_shadows: Vec<TextShadow>,
    pub text_decoration: TextDecoration,

    // ---- interaction ----
    pub cursor: CursorIcon,
    pub cursor_visible: bool,

    // ---- effects ----
    pub overflow: Overflow,
    pub blur: f32,
    pub rotation: f32,
    pub opacity: f32,
    pub scale: Scale,
    pub transform_origin: TransformOrigin,

    // ---- radius ----
    pub radius: CornerRadius,

    // ---- layout: dimensions ----
    pub width: Size,
    pub height: Size,
    pub min_width: Size,
    pub min_height: Size,
    pub max_width: Size,
    pub max_height: Size,

    // ---- layout: box model ----
    pub padding: Gaps,
    pub margin: Gaps,

    // ---- layout: flow ----
    pub spacing: Length,  // gap between children
    pub offset_x: Length, // shift children horizontally
    pub offset_y: Length, // shift children vertically
    pub direction: Direction,
    pub main_align: Alignment,
    pub cross_align: Alignment,
    pub position: Position,
    pub content: Content,

    pub object_fit: AspectRatio,
    pub object_position: ImageCover,
    pub sampling_mode: SamplingMode,
    pub image_blur: f32,
    pub snap_to_grid: bool,

    pub variants: StyleVariants,
}

impl Default for Style {
    fn default() -> Self {
        let theme = active_theme();
        let mut style = Self::bare();

        // ---- colors: the two that every element should inherit ----
        if let Some(fill) = theme.fill("background") {
            style.background = fill;
        }
        if let Some(fill) = theme.fill("foreground") {
            style.text_color = fill;
        }

        // ---- typography: base size + sans stack ----
        if let Some((size, _line_height)) = theme.typography.font_size.get("base") {
            style.font_size = (*size).into();
        }
        if let Some(sans) = theme.typography.font_family.get("sans") {
            style.font_families = sans
                .split(',')
                .map(|f| f.trim())
                .filter(|f| !f.is_empty())
                .map(|f| Cow::Owned(f.to_owned()))
                .collect();
        }

        style
    }
}

impl Style {
    /// A `Style` with **no** theme influence. Use this in tests, snapshots,
    /// or any place where ambient thread-local state must not leak in.
    pub fn bare() -> Self {
        Self {
            // exactly the current `Default` body, verbatim
            background: Fill::default(),
            text_color: black().into(),
            text_align: TextAlign::default(),
            font_size: FontSize::default(),
            font_weight: FontWeight::default(),
            letter_spacing: LetterSpacing::default(),
            text_overflow: TextOverflow::default(),
            font_features: Vec::default(),
            font_families: Vec::default(),
            font_style: FontSlant::default(),
            font_stretch: FontWidth::default(),
            line_height: f32::default(),
            text_shadows: Vec::default(),
            text_decoration: TextDecoration::default(),
            cursor: CursorIcon::default(),
            cursor_visible: true,
            overflow: Overflow::default(),
            blur: f32::default(),
            rotation: f32::default(),
            opacity: f32::default(),
            scale: Scale { x: 1.0, y: 1.0 },
            transform_origin: TransformOrigin::default(),
            radius: CornerRadius::default(),
            width: Size::default(),
            height: Size::default(),
            min_width: Size::default(),
            min_height: Size::default(),
            max_width: Size::default(),
            max_height: Size::default(),
            padding: Gaps::default(),
            margin: Gaps::default(),
            spacing: Length::default(),
            offset_x: Length::default(),
            offset_y: Length::default(),
            direction: Direction::default(),
            main_align: Alignment::default(),
            cross_align: Alignment::default(),
            position: Position::default(),
            content: Content::default(),
            object_fit: AspectRatio::default(),
            object_position: ImageCover::default(),
            sampling_mode: SamplingMode::default(),
            image_blur: 0.0,
            snap_to_grid: false,
            variants: StyleVariants::default(),
        }
    }

    /// A `Style` seeded from an explicit `Theme`, ignoring thread-local
    /// state. Useful for tests, previews, and any place where you want
    /// deterministic theme application.
    pub fn themed(theme: &crate::theme::Theme) -> Self {
        let mut style = Self::bare();
        if let Some(fill) = theme.fill("background") {
            style.background = fill;
        }
        if let Some(fill) = theme.fill("foreground") {
            style.text_color = fill;
        }
        if let Some((size, _)) = theme.typography.font_size.get("base") {
            style.font_size = (*size).into();
        }
        if let Some(sans) = theme.typography.font_family.get("sans") {
            style.font_families = sans
                .split(',')
                .map(|f| f.trim())
                .filter(|f| !f.is_empty())
                .map(|f| Cow::Owned(f.to_owned()))
                .collect();
        }
        style
    }
}

impl StyledExt for Style {
    fn get_style(&mut self) -> &mut Style {
        self
    }
}
