use crate::styling::colors::black;
use freya_core::prelude::{
    Fill, FontFeature, FontSize, FontSlant, FontWeight, FontWidth, LetterSpacing, TextAlign,
    TextOverflow,
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub background: Fill,
    pub text_color: Fill,
    pub text_align: TextAlign,
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    pub letter_spacing: LetterSpacing,
    pub text_overflow: TextOverflow,
    pub font_features: Vec<FontFeature>,
    pub font_families: Vec<Cow<'static, str>>,
    pub font_style: FontSlant,
    pub font_stretch: FontWidth,
}

impl Default for Style {
    fn default() -> Self {
        Self {
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
        }
    }
}
