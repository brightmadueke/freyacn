use crate::styling::colors::black;
use freya_core::data::Overflow;
use freya_core::prelude::{
    CornerRadius, CursorIcon, Fill, FontFeature, FontSize, FontSlant, FontWeight, FontWidth,
    LetterSpacing, Scale, TextAlign, TextDecoration, TextOverflow, TextShadow, TransformOrigin,
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
}

impl Default for Style {
    fn default() -> Self {
        Self {
            // ---- paint ----
            background: Fill::default(),
            text_color: black().into(),
            text_align: TextAlign::default(),

            // ---- typography ----
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

            // ---- interaction ----
            cursor: CursorIcon::default(),
            cursor_visible: true,

            // ---- effects ----
            overflow: Overflow::default(),
            blur: f32::default(),
            rotation: f32::default(),
            opacity: f32::default(),
            scale: Scale { x: 1.0, y: 1.0 },
            transform_origin: TransformOrigin::default(),

            // ---- radius ----
            radius: CornerRadius::default(),

            // ---- layout: dimensions ----
            width: Size::default(),
            height: Size::default(),
            min_width: Size::default(),
            min_height: Size::default(),
            max_width: Size::default(),
            max_height: Size::default(),

            // ---- layout: box model ----
            padding: Gaps::default(),
            margin: Gaps::default(),

            // ---- layout: flow ----
            spacing: Length::default(),
            offset_x: Length::default(),
            offset_y: Length::default(),
            direction: Direction::default(),
            main_align: Alignment::default(),
            cross_align: Alignment::default(),
            position: Position::default(),
            content: Content::default(),
        }
    }
}
