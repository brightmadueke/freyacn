//! The `Style` struct — one field per CSS property.

use crate::stylesheet::types::*;
use crate::stylesheet::unit_value::UnitValue;

#[derive(Debug, Clone, Default)]
pub struct Style {
    // ======================================================================
    // BOX MODEL & SIZING
    // ======================================================================
    pub(crate) display: Vec<Display>,
    pub(crate) box_sizing: Option<BoxSizing>,
    pub(crate) position: Option<Position>,

    // Physical inset
    pub(crate) top: Option<SizeValue>,
    pub(crate) right: Option<SizeValue>,
    pub(crate) bottom: Option<SizeValue>,
    pub(crate) left: Option<SizeValue>,

    // Logical inset
    pub(crate) inset: Vec<SizeValue>,
    pub(crate) inset_inline: Vec<SizeValue>,
    pub(crate) inset_block: Vec<SizeValue>,
    pub(crate) inset_inline_start: Option<SizeValue>,
    pub(crate) inset_inline_end: Option<SizeValue>,
    pub(crate) inset_block_start: Option<SizeValue>,
    pub(crate) inset_block_end: Option<SizeValue>,

    pub(crate) z_index: Option<i32>,

    // Physical sizing
    pub(crate) width: Option<SizeValue>,
    pub(crate) height: Option<SizeValue>,
    pub(crate) min_width: Option<SizeValue>,
    pub(crate) min_height: Option<SizeValue>,
    pub(crate) max_width: Option<SizeValue>,
    pub(crate) max_height: Option<SizeValue>,

    // Logical sizing
    pub(crate) inline_size: Option<SizeValue>,
    pub(crate) block_size: Option<SizeValue>,
    pub(crate) min_inline_size: Option<SizeValue>,
    pub(crate) min_block_size: Option<SizeValue>,
    pub(crate) max_inline_size: Option<SizeValue>,
    pub(crate) max_block_size: Option<SizeValue>,

    pub(crate) aspect_ratio: Option<AspectRatio>,

    // Margin
    pub(crate) margin: Vec<SizeValue>,
    pub(crate) margin_top: Option<SizeValue>,
    pub(crate) margin_right: Option<SizeValue>,
    pub(crate) margin_bottom: Option<SizeValue>,
    pub(crate) margin_left: Option<SizeValue>,
    pub(crate) margin_inline: Vec<SizeValue>,
    pub(crate) margin_block: Vec<SizeValue>,
    pub(crate) margin_inline_start: Option<SizeValue>,
    pub(crate) margin_inline_end: Option<SizeValue>,
    pub(crate) margin_block_start: Option<SizeValue>,
    pub(crate) margin_block_end: Option<SizeValue>,

    // Padding
    pub(crate) padding: Vec<UnitValue>,
    pub(crate) padding_top: Option<UnitValue>,
    pub(crate) padding_right: Option<UnitValue>,
    pub(crate) padding_bottom: Option<UnitValue>,
    pub(crate) padding_left: Option<UnitValue>,
    pub(crate) padding_inline: Vec<UnitValue>,
    pub(crate) padding_block: Vec<UnitValue>,
    pub(crate) padding_inline_start: Option<UnitValue>,
    pub(crate) padding_inline_end: Option<UnitValue>,
    pub(crate) padding_block_start: Option<UnitValue>,
    pub(crate) padding_block_end: Option<UnitValue>,

    // Overflow & clipping
    pub(crate) overflow: Option<Overflow>,
    pub(crate) overflow_x: Option<Overflow>,
    pub(crate) overflow_y: Option<Overflow>,
    pub(crate) overflow_clip_margin: Option<UnitValue>,

    // Visibility / float
    pub(crate) visibility: Option<Visibility>,
    pub(crate) float: Option<Float>,
    pub(crate) clear: Option<Clear>,

    // Object fit
    pub(crate) object_fit: Option<ObjectFit>,
    pub(crate) object_position: Option<Position2D>,

    // ======================================================================
    // FLEXBOX
    // ======================================================================
    pub(crate) flex_direction: Option<FlexDirection>,
    pub(crate) flex_wrap: Option<FlexWrap>,
    pub(crate) flex_flow: Vec<String>,
    pub(crate) flex_grow: Option<f32>,
    pub(crate) flex_shrink: Option<f32>,
    pub(crate) flex_basis: Option<SizeValue>,
    pub(crate) flex: Vec<String>,
    pub(crate) order: Option<i32>,
    pub(crate) justify_content: Option<JustifyContent>,
    pub(crate) align_items: Option<AlignItems>,
    pub(crate) align_self: Option<AlignSelf>,
    pub(crate) align_content: Option<AlignContent>,

    // ======================================================================
    // GRID
    // ======================================================================
    pub(crate) grid_template_columns: Option<String>,
    pub(crate) grid_template_rows: Option<String>,
    pub(crate) grid_template_areas: Option<String>,
    pub(crate) grid_auto_columns: Option<String>,
    pub(crate) grid_auto_rows: Option<String>,
    pub(crate) grid_auto_flow: Option<GridAutoFlow>,
    pub(crate) grid: Vec<String>,

    pub(crate) grid_column_start: Option<GridLine>,
    pub(crate) grid_column_end: Option<GridLine>,
    pub(crate) grid_row_start: Option<GridLine>,
    pub(crate) grid_row_end: Option<GridLine>,
    pub(crate) grid_column: Vec<String>,
    pub(crate) grid_row: Vec<String>,
    pub(crate) grid_area: Vec<String>,

    pub(crate) gap: Vec<UnitValue>,
    pub(crate) row_gap: Option<UnitValue>,
    pub(crate) column_gap: Option<UnitValue>,

    pub(crate) justify_items: Option<JustifyItems>,
    pub(crate) justify_self: Option<JustifySelf>,

    // ======================================================================
    // TYPOGRAPHY
    // ======================================================================
    pub(crate) font_family: Option<FontFamily>,
    pub(crate) font_size: Option<UnitValue>,
    pub(crate) font_weight: Option<FontWeight>,
    pub(crate) font_style: Option<FontStyle>,
    pub(crate) font_stretch: Option<FontStretch>,
    pub(crate) font_variant: Vec<String>,
    pub(crate) font_variant_caps: Option<String>,
    pub(crate) font_variant_ligatures: Option<String>,
    pub(crate) font_variant_numeric: Option<String>,
    pub(crate) font_variant_east_asian: Option<String>,
    pub(crate) font_feature_settings: Option<String>,
    pub(crate) font_variation_settings: Option<String>,
    pub(crate) font_kerning: Option<FontKerning>,
    pub(crate) font: Vec<String>,

    pub(crate) line_height: Option<LineHeight>,
    pub(crate) letter_spacing: Option<UnitValue>,
    pub(crate) word_spacing: Option<UnitValue>,

    pub(crate) text_align: Option<TextAlign>,
    pub(crate) text_align_last: Option<TextAlignLast>,
    pub(crate) text_indent: Option<UnitValue>,
    pub(crate) text_transform: Option<TextTransform>,
    pub(crate) text_overflow: Option<TextOverflow>,
    pub(crate) text_rendering: Option<TextRendering>,
    pub(crate) text_orientation: Option<TextOrientation>,
    pub(crate) text_wrap: Option<String>,

    pub(crate) text_decoration_line: Option<TextDecoration>,
    pub(crate) text_decoration_style: Option<TextDecorationStyle>,
    pub(crate) text_decoration_color: Option<Color>,
    pub(crate) text_decoration_thickness: Option<UnitValue>,
    pub(crate) text_decoration_underline_offset: Option<UnitValue>,
    pub(crate) text_decoration_skip_ink: Option<TextDecorationSkipInk>,
    pub(crate) text_underline_position: Option<TextUnderlinePosition>,
    pub(crate) text_decoration: Vec<String>,

    pub(crate) text_shadow: Vec<TextShadow>,

    pub(crate) vertical_align: Option<VerticalAlign>,
    pub(crate) white_space: Option<WhiteSpace>,
    pub(crate) white_space_collapse: Option<String>,
    pub(crate) word_break: Option<WordBreak>,
    pub(crate) overflow_wrap: Option<OverflowWrap>,
    pub(crate) line_break: Option<String>,
    pub(crate) hyphens: Option<Hyphens>,
    pub(crate) tab_size: Option<UnitValue>,

    pub(crate) direction: Option<Direction>,
    pub(crate) unicode_bidi: Option<UnicodeBidi>,
    pub(crate) writing_mode: Option<WritingMode>,

    // ======================================================================
    // COLORS & BACKGROUNDS
    // ======================================================================
    pub(crate) color: Option<Color>,
    pub(crate) opacity: Option<f32>,

    pub(crate) background_color: Option<Color>,
    pub(crate) background_image: Vec<CssImage>,
    pub(crate) background_position: Vec<Position2D>,
    pub(crate) background_position_x: Option<HorizontalKeyword>,
    pub(crate) background_position_y: Option<VerticalKeyword>,
    pub(crate) background_size: Option<BackgroundSize>,
    pub(crate) background_repeat: Option<BackgroundRepeat>,
    pub(crate) background_attachment: Option<BackgroundAttachment>,
    pub(crate) background_clip: Option<BackgroundClip>,
    pub(crate) background_origin: Option<BackgroundOrigin>,
    pub(crate) background_blend_mode: Option<BlendMode>,
    pub(crate) background: Vec<String>,

    // ======================================================================
    // BORDERS & OUTLINES
    // ======================================================================
    pub(crate) border_style: Option<BorderStyle>,
    pub(crate) border_width: Option<UnitValue>,
    pub(crate) border_color: Option<Color>,

    pub(crate) border_top_style: Option<BorderStyle>,
    pub(crate) border_top_width: Option<UnitValue>,
    pub(crate) border_top_color: Option<Color>,
    pub(crate) border_right_style: Option<BorderStyle>,
    pub(crate) border_right_width: Option<UnitValue>,
    pub(crate) border_right_color: Option<Color>,
    pub(crate) border_bottom_style: Option<BorderStyle>,
    pub(crate) border_bottom_width: Option<UnitValue>,
    pub(crate) border_bottom_color: Option<Color>,
    pub(crate) border_left_style: Option<BorderStyle>,
    pub(crate) border_left_width: Option<UnitValue>,
    pub(crate) border_left_color: Option<Color>,

    pub(crate) border_inline_start_style: Option<BorderStyle>,
    pub(crate) border_inline_start_width: Option<UnitValue>,
    pub(crate) border_inline_start_color: Option<Color>,
    pub(crate) border_inline_end_style: Option<BorderStyle>,
    pub(crate) border_inline_end_width: Option<UnitValue>,
    pub(crate) border_inline_end_color: Option<Color>,
    pub(crate) border_block_start_style: Option<BorderStyle>,
    pub(crate) border_block_start_width: Option<UnitValue>,
    pub(crate) border_block_start_color: Option<Color>,
    pub(crate) border_block_end_style: Option<BorderStyle>,
    pub(crate) border_block_end_width: Option<UnitValue>,
    pub(crate) border_block_end_color: Option<Color>,

    pub(crate) border_radius: Vec<UnitValue>,
    pub(crate) border_top_left_radius: Option<UnitValue>,
    pub(crate) border_top_right_radius: Option<UnitValue>,
    pub(crate) border_bottom_right_radius: Option<UnitValue>,
    pub(crate) border_bottom_left_radius: Option<UnitValue>,

    pub(crate) border_image_source: Option<CssImage>,
    pub(crate) border_image_slice: Option<String>,
    pub(crate) border_image_width: Option<String>,
    pub(crate) border_image_outset: Option<String>,
    pub(crate) border_image_repeat: Option<String>,
    pub(crate) border_image: Vec<String>,

    pub(crate) border_collapse: Option<BorderCollapse>,
    pub(crate) border_spacing: Vec<UnitValue>,

    pub(crate) outline_style: Option<OutlineStyle>,
    pub(crate) outline_width: Option<UnitValue>,
    pub(crate) outline_color: Option<Color>,
    pub(crate) outline_offset: Option<UnitValue>,
    pub(crate) outline: Vec<String>,

    // ======================================================================
    // VISUAL EFFECTS
    // ======================================================================
    pub(crate) box_shadow: Vec<BoxShadow>,
    pub(crate) filter: Vec<FilterFunction>,
    pub(crate) backdrop_filter: Vec<FilterFunction>,
    pub(crate) mix_blend_mode: Option<BlendMode>,
    pub(crate) isolation: Option<Isolation>,
    pub(crate) clip_path: Option<String>,

    pub(crate) mask_image: Vec<CssImage>,
    pub(crate) mask_mode: Option<String>,
    pub(crate) mask_repeat: Option<BackgroundRepeat>,
    pub(crate) mask_position: Vec<Position2D>,
    pub(crate) mask_clip: Option<BackgroundClip>,
    pub(crate) mask_origin: Option<BackgroundOrigin>,
    pub(crate) mask_size: Option<BackgroundSize>,
    pub(crate) mask_composite: Option<String>,
    pub(crate) mask: Vec<String>,

    // ======================================================================
    // TRANSFORMS & PERSPECTIVE
    // ======================================================================
    pub(crate) transform: Vec<TransformFunction>,
    pub(crate) transform_origin: Option<Position2D>,
    pub(crate) transform_style: Option<TransformStyle>,
    pub(crate) transform_box: Option<TransformBox>,
    pub(crate) perspective: Option<UnitValue>,
    pub(crate) perspective_origin: Option<Position2D>,
    pub(crate) backface_visibility: Option<BackfaceVisibility>,
    pub(crate) rotate: Option<f32>,
    pub(crate) scale: Option<(f32, f32)>,
    pub(crate) translate: Option<(UnitValue, UnitValue)>,

    // ======================================================================
    // TRANSITIONS & ANIMATIONS
    // ======================================================================
    pub(crate) transition_property: Option<String>,
    pub(crate) transition_duration: Option<f32>,
    pub(crate) transition_timing_function: Option<String>,
    pub(crate) transition_delay: Option<f32>,
    pub(crate) transition: Vec<Transition>,

    pub(crate) animation_name: Option<String>,
    pub(crate) animation_duration: Option<f32>,
    pub(crate) animation_timing_function: Option<String>,
    pub(crate) animation_delay: Option<f32>,
    pub(crate) animation_iteration_count: Option<String>,
    pub(crate) animation_direction: Option<String>,
    pub(crate) animation_fill_mode: Option<String>,
    pub(crate) animation_play_state: Option<String>,
    pub(crate) animation: Vec<Animation>,

    // ======================================================================
    // LISTS
    // ======================================================================
    pub(crate) list_style_type: Option<String>,
    pub(crate) list_style_position: Option<ListStylePosition>,
    pub(crate) list_style_image: Option<CssImage>,
    pub(crate) list_style: Vec<String>,

    // ======================================================================
    // TABLES
    // ======================================================================
    pub(crate) table_layout: Option<TableLayout>,
    pub(crate) caption_side: Option<CaptionSide>,
    pub(crate) empty_cells: Option<EmptyCells>,

    // ======================================================================
    // INTERACTION & UI
    // ======================================================================
    pub(crate) cursor: Option<Cursor>,
    pub(crate) pointer_events: Option<PointerEvents>,
    pub(crate) user_select: Option<UserSelect>,
    pub(crate) touch_action: Option<String>,
    pub(crate) resize: Option<Resize>,
    pub(crate) appearance: Option<Appearance>,
    pub(crate) caret_color: Option<Color>,
    pub(crate) caret_shape: Option<String>,
    pub(crate) accent_color: Option<Color>,
    pub(crate) color_scheme: Option<ColorScheme>,
    pub(crate) forced_color_adjust: Option<String>,
    pub(crate) print_color_adjust: Option<String>,

    // Scrolling
    pub(crate) scroll_behavior: Option<ScrollBehavior>,
    pub(crate) overscroll_behavior: Option<OverscrollBehavior>,
    pub(crate) overscroll_behavior_x: Option<OverscrollBehavior>,
    pub(crate) overscroll_behavior_y: Option<OverscrollBehavior>,
    pub(crate) scrollbar_width: Option<ScrollbarWidth>,
    pub(crate) scrollbar_gutter: Option<ScrollbarGutter>,
    pub(crate) scrollbar_color: Option<(Color, Color)>,

    // Scroll snap
    pub(crate) scroll_snap_type: Option<(ScrollSnapTypeAxis, ScrollSnapStrictness)>,
    pub(crate) scroll_snap_align: Option<ScrollSnapAlign>,
    pub(crate) scroll_snap_stop: Option<ScrollSnapStop>,
    pub(crate) scroll_margin: Vec<UnitValue>,
    pub(crate) scroll_padding: Vec<UnitValue>,

    // ======================================================================
    // MULTI-COLUMN / FRAGMENTATION
    // ======================================================================
    pub(crate) columns: Vec<String>,
    pub(crate) column_count: Option<u32>,
    pub(crate) column_width: Option<UnitValue>,
    pub(crate) column_rule_style: Option<BorderStyle>,
    pub(crate) column_rule_width: Option<UnitValue>,
    pub(crate) column_rule_color: Option<Color>,
    pub(crate) column_rule: Vec<String>,
    pub(crate) column_span: Option<String>,
    pub(crate) column_fill: Option<String>,

    pub(crate) break_before: Option<BreakBefore>,
    pub(crate) break_after: Option<BreakAfter>,
    pub(crate) break_inside: Option<BreakInside>,

    // ======================================================================
    // CONTAINMENT & MISC
    // ======================================================================
    pub(crate) contain: Vec<Contain>,
    pub(crate) content_visibility: Option<String>,
    pub(crate) container_name: Option<String>,
    pub(crate) container_type: Option<String>,
    pub(crate) container: Vec<String>,
    pub(crate) will_change: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) quotes: Option<String>,
    pub(crate) counter_reset: Option<String>,
    pub(crate) counter_increment: Option<String>,
    pub(crate) counter_set: Option<String>,
    pub(crate) all: Option<GlobalKeyword>,
    pub(crate) zoom: Option<f32>,
    pub(crate) image_rendering: Option<ImageRendering>,
    pub(crate) color_interpolation: Option<String>,
}

// ============================================================================
// Helper traits
// ============================================================================

/// Anything convertible to `Option<String>`.
///
/// Lets `Option<String>` setters accept `"…"`, `String`, `Some("…")`,
/// `Some(String)`, and `None`.
pub trait IntoOptString {
    fn into_opt_string(self) -> Option<String>;
}

impl IntoOptString for String {
    fn into_opt_string(self) -> Option<String> {
        Some(self)
    }
}

impl IntoOptString for &str {
    fn into_opt_string(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoOptString for Option<String> {
    fn into_opt_string(self) -> Option<String> {
        self
    }
}

impl IntoOptString for Option<&str> {
    fn into_opt_string(self) -> Option<String> {
        self.map(str::to_string)
    }
}

/// Anything convertible to `Vec<String>`.
///
/// Lets `Vec<String>` setters accept `["a", "b"]`, `vec!["a", "b"]`,
/// `["a".to_string(), …]`, and so on.
pub trait IntoVecString {
    fn into_vec_string(self) -> Vec<String>;
}

impl IntoVecString for Vec<String> {
    fn into_vec_string(self) -> Vec<String> {
        self
    }
}

impl IntoVecString for Vec<&str> {
    fn into_vec_string(self) -> Vec<String> {
        self.into_iter().map(String::from).collect()
    }
}

impl IntoVecString for &[&str] {
    fn into_vec_string(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl<const N: usize> IntoVecString for [&str; N] {
    fn into_vec_string(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl<const N: usize> IntoVecString for [String; N] {
    fn into_vec_string(self) -> Vec<String> {
        self.into()
    }
}

// ============================================================================
// Setter macros
// ============================================================================

macro_rules! setters_opt {
    ($($field:ident: $ty:ty),* $(,)?) => {
        $(
            #[must_use = "builder methods return the modified Style"]
            pub fn $field(mut self, value: impl Into<Option<$ty>>) -> Self {
                self.$field = value.into();
                self
            }
        )*
    };
}

macro_rules! setters_vec {
    ($($field:ident: $ty:ty),* $(,)?) => {
        $(
            #[must_use = "builder methods return the modified Style"]
            pub(crate) fn $field(mut self, value: impl Into<Vec<$ty>>) -> Self {
                self.$field = value.into();
                self
            }
        )*
    };
}

macro_rules! setters_string {
    ($($field:ident),* $(,)?) => {
        $(
            #[must_use = "builder methods return the modified Style"]
            pub(crate) fn $field(mut self, value: impl IntoOptString) -> Self {
                self.$field = value.into_opt_string();
                self
            }
        )*
    };
}

macro_rules! setters_vec_string {
    ($($field:ident),* $(,)?) => {
        $(
            #[must_use = "builder methods return the modified Style"]
            pub(crate) fn $field(mut self, value: impl IntoVecString) -> Self {
                self.$field = value.into_vec_string();
                self
            }
        )*
    };
}

impl Style {
    // ======================================================================
    // BOX MODEL & SIZING
    // ======================================================================
    setters_vec!(
        display: Display,
        inset: SizeValue,
        inset_inline: SizeValue,
        inset_block: SizeValue,
        margin: SizeValue,
        margin_inline: SizeValue,
        margin_block: SizeValue,
        padding: UnitValue,
        padding_inline: UnitValue,
        padding_block: UnitValue,
    );
    setters_opt!(
        box_sizing: BoxSizing,
        position: Position,
        top: SizeValue,
        right: SizeValue,
        bottom: SizeValue,
        left: SizeValue,
        inset_inline_start: SizeValue,
        inset_inline_end: SizeValue,
        inset_block_start: SizeValue,
        inset_block_end: SizeValue,
        z_index: i32,
        width: SizeValue,
        height: SizeValue,
        min_width: SizeValue,
        min_height: SizeValue,
        max_width: SizeValue,
        max_height: SizeValue,
        inline_size: SizeValue,
        block_size: SizeValue,
        min_inline_size: SizeValue,
        min_block_size: SizeValue,
        max_inline_size: SizeValue,
        max_block_size: SizeValue,
        aspect_ratio: AspectRatio,
        margin_top: SizeValue,
        margin_right: SizeValue,
        margin_bottom: SizeValue,
        margin_left: SizeValue,
        margin_inline_start: SizeValue,
        margin_inline_end: SizeValue,
        margin_block_start: SizeValue,
        margin_block_end: SizeValue,
        padding_top: UnitValue,
        padding_right: UnitValue,
        padding_bottom: UnitValue,
        padding_left: UnitValue,
        padding_inline_start: UnitValue,
        padding_inline_end: UnitValue,
        padding_block_start: UnitValue,
        padding_block_end: UnitValue,
        overflow: Overflow,
        overflow_x: Overflow,
        overflow_y: Overflow,
        overflow_clip_margin: UnitValue,
        visibility: Visibility,
        float: Float,
        clear: Clear,
        object_fit: ObjectFit,
        object_position: Position2D,
    );

    // ======================================================================
    // FLEXBOX
    // ======================================================================
    setters_vec_string!(flex_flow, flex,);
    setters_opt!(
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
        flex_grow: f32,
        flex_shrink: f32,
        flex_basis: SizeValue,
        order: i32,
        justify_content: JustifyContent,
        align_items: AlignItems,
        align_self: AlignSelf,
        align_content: AlignContent,
    );

    // ======================================================================
    // GRID
    // ======================================================================
    setters_vec!(gap: UnitValue);
    setters_vec_string!(grid, grid_column, grid_row, grid_area,);
    setters_string!(
        grid_template_columns,
        grid_template_rows,
        grid_template_areas,
        grid_auto_columns,
        grid_auto_rows,
    );
    setters_opt!(
        grid_auto_flow: GridAutoFlow,
        grid_column_start: GridLine,
        grid_column_end: GridLine,
        grid_row_start: GridLine,
        grid_row_end: GridLine,
        row_gap: UnitValue,
        column_gap: UnitValue,
        justify_items: JustifyItems,
        justify_self: JustifySelf,
    );

    // ======================================================================
    // TYPOGRAPHY
    // ======================================================================
    setters_vec!(text_shadow: TextShadow);
    setters_vec_string!(font_variant, font, text_decoration,);
    setters_string!(
        font_variant_caps,
        font_variant_ligatures,
        font_variant_numeric,
        font_variant_east_asian,
        font_feature_settings,
        font_variation_settings,
        text_wrap,
        white_space_collapse,
        line_break,
    );
    setters_opt!(
        font_family: FontFamily,
        font_size: UnitValue,
        font_weight: FontWeight,
        font_style: FontStyle,
        font_stretch: FontStretch,
        font_kerning: FontKerning,
        line_height: LineHeight,
        letter_spacing: UnitValue,
        word_spacing: UnitValue,
        text_align: TextAlign,
        text_align_last: TextAlignLast,
        text_indent: UnitValue,
        text_transform: TextTransform,
        text_overflow: TextOverflow,
        text_rendering: TextRendering,
        text_orientation: TextOrientation,
        text_decoration_line: TextDecoration,
        text_decoration_style: TextDecorationStyle,
        text_decoration_color: Color,
        text_decoration_thickness: UnitValue,
        text_decoration_underline_offset: UnitValue,
        text_decoration_skip_ink: TextDecorationSkipInk,
        text_underline_position: TextUnderlinePosition,
        vertical_align: VerticalAlign,
        white_space: WhiteSpace,
        word_break: WordBreak,
        overflow_wrap: OverflowWrap,
        hyphens: Hyphens,
        tab_size: UnitValue,
        direction: Direction,
        unicode_bidi: UnicodeBidi,
        writing_mode: WritingMode,
    );

    // ======================================================================
    // COLORS & BACKGROUNDS
    // ======================================================================
    setters_vec!(
        background_image: CssImage,
        background_position: Position2D,
    );
    setters_vec_string!(background);
    setters_opt!(
        color: Color,
        opacity: f32,
        background_color: Color,
        background_position_x: HorizontalKeyword,
        background_position_y: VerticalKeyword,
        background_size: BackgroundSize,
        background_repeat: BackgroundRepeat,
        background_attachment: BackgroundAttachment,
        background_clip: BackgroundClip,
        background_origin: BackgroundOrigin,
        background_blend_mode: BlendMode,
    );

    // ======================================================================
    // BORDERS & OUTLINES
    // ======================================================================
    setters_vec!(
        border_radius: UnitValue,
        border_spacing: UnitValue,
    );
    setters_vec_string!(border_image, outline,);
    setters_string!(
        border_image_slice,
        border_image_width,
        border_image_outset,
        border_image_repeat,
    );
    setters_opt!(
        border_style: BorderStyle,
        border_width: UnitValue,
        border_color: Color,
        border_top_style: BorderStyle,
        border_top_width: UnitValue,
        border_top_color: Color,
        border_right_style: BorderStyle,
        border_right_width: UnitValue,
        border_right_color: Color,
        border_bottom_style: BorderStyle,
        border_bottom_width: UnitValue,
        border_bottom_color: Color,
        border_left_style: BorderStyle,
        border_left_width: UnitValue,
        border_left_color: Color,
        border_inline_start_style: BorderStyle,
        border_inline_start_width: UnitValue,
        border_inline_start_color: Color,
        border_inline_end_style: BorderStyle,
        border_inline_end_width: UnitValue,
        border_inline_end_color: Color,
        border_block_start_style: BorderStyle,
        border_block_start_width: UnitValue,
        border_block_start_color: Color,
        border_block_end_style: BorderStyle,
        border_block_end_width: UnitValue,
        border_block_end_color: Color,
        border_top_left_radius: UnitValue,
        border_top_right_radius: UnitValue,
        border_bottom_right_radius: UnitValue,
        border_bottom_left_radius: UnitValue,
        border_image_source: CssImage,
        border_collapse: BorderCollapse,
        outline_style: OutlineStyle,
        outline_width: UnitValue,
        outline_color: Color,
        outline_offset: UnitValue,
    );

    // ======================================================================
    // VISUAL EFFECTS
    // ======================================================================
    setters_vec!(
        box_shadow: BoxShadow,
        filter: FilterFunction,
        backdrop_filter: FilterFunction,
        mask_image: CssImage,
        mask_position: Position2D,
    );
    setters_vec_string!(mask);
    setters_string!(clip_path, mask_mode, mask_composite,);
    setters_opt!(
        mix_blend_mode: BlendMode,
        isolation: Isolation,
        mask_repeat: BackgroundRepeat,
        mask_clip: BackgroundClip,
        mask_origin: BackgroundOrigin,
        mask_size: BackgroundSize,
    );

    // ======================================================================
    // TRANSFORMS & PERSPECTIVE
    // ======================================================================
    setters_vec!(transform: TransformFunction);
    setters_opt!(
        transform_origin: Position2D,
        transform_style: TransformStyle,
        transform_box: TransformBox,
        perspective: UnitValue,
        perspective_origin: Position2D,
        backface_visibility: BackfaceVisibility,
        rotate: f32,
        scale: (f32, f32),
        translate: (UnitValue, UnitValue),
    );

    // ======================================================================
    // TRANSITIONS & ANIMATIONS
    // ======================================================================
    setters_vec!(
        transition: Transition,
        animation: Animation,
    );
    setters_string!(
        transition_property,
        transition_timing_function,
        animation_name,
        animation_timing_function,
        animation_iteration_count,
        animation_direction,
        animation_fill_mode,
        animation_play_state,
    );
    setters_opt!(
        transition_duration: f32,
        transition_delay: f32,
        animation_duration: f32,
        animation_delay: f32,
    );

    // ======================================================================
    // LISTS
    // ======================================================================
    setters_vec_string!(list_style);
    setters_string!(list_style_type);
    setters_opt!(
        list_style_position: ListStylePosition,
        list_style_image: CssImage,
    );

    // ======================================================================
    // TABLES
    // ======================================================================
    setters_opt!(
        table_layout: TableLayout,
        caption_side: CaptionSide,
        empty_cells: EmptyCells,
    );

    // ======================================================================
    // INTERACTION & UI
    // ======================================================================
    setters_string!(
        touch_action,
        caret_shape,
        forced_color_adjust,
        print_color_adjust,
    );
    setters_vec!(
        scroll_margin: UnitValue,
        scroll_padding: UnitValue,
    );
    setters_opt!(
        cursor: Cursor,
        pointer_events: PointerEvents,
        user_select: UserSelect,
        resize: Resize,
        appearance: Appearance,
        caret_color: Color,
        accent_color: Color,
        color_scheme: ColorScheme,
        scroll_behavior: ScrollBehavior,
        overscroll_behavior: OverscrollBehavior,
        overscroll_behavior_x: OverscrollBehavior,
        overscroll_behavior_y: OverscrollBehavior,
        scrollbar_width: ScrollbarWidth,
        scrollbar_gutter: ScrollbarGutter,
        scrollbar_color: (Color, Color),
        scroll_snap_type: (ScrollSnapTypeAxis, ScrollSnapStrictness),
        scroll_snap_align: ScrollSnapAlign,
        scroll_snap_stop: ScrollSnapStop,
    );

    // ======================================================================
    // MULTI-COLUMN / FRAGMENTATION
    // ======================================================================
    setters_vec_string!(columns, column_rule,);
    setters_string!(column_span, column_fill,);
    setters_opt!(
        column_count: u32,
        column_width: UnitValue,
        column_rule_style: BorderStyle,
        column_rule_width: UnitValue,
        column_rule_color: Color,
        break_before: BreakBefore,
        break_after: BreakAfter,
        break_inside: BreakInside,
    );

    // ======================================================================
    // CONTAINMENT & MISC
    // ======================================================================
    setters_vec!(contain: Contain);
    setters_vec_string!(container);
    setters_string!(
        content_visibility,
        container_name,
        container_type,
        will_change,
        content,
        quotes,
        counter_reset,
        counter_increment,
        counter_set,
        color_interpolation,
    );
    setters_opt!(
        all: GlobalKeyword,
        zoom: f32,
        image_rendering: ImageRendering,
    );
}

impl Style {
    /// Merge `other` into `self`. Fields set in `other` override `self`.
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        // One field at a time — overrides only when `other` has a value.
        macro_rules! merge_opt { ($($f:ident),*) => { $( if other.$f.is_some() { self.$f = other.$f; } )* } }
        macro_rules! merge_vec { ($($f:ident),*) => { $( if !other.$f.is_empty() { self.$f = other.$f; } )* } }

        merge_opt!(
            box_sizing,
            position,
            z_index,
            aspect_ratio,
            width,
            height,
            min_width,
            min_height,
            max_width,
            max_height,
            overflow,
            overflow_x,
            overflow_y,
            visibility,
            float,
            clear,
            flex_direction,
            flex_wrap,
            flex_grow,
            flex_shrink,
            flex_basis,
            justify_content,
            align_items,
            align_self,
            align_content,
            color,
            opacity,
            background_color,
            border_style,
            border_width,
            border_color,
            font_size,
            font_weight,
            font_style,
            line_height,
            text_align,
            text_transform,
            text_overflow,
            cursor,
            pointer_events,
            user_select,
            transition_property,
            transition_duration,
            transition_timing_function,
            transition_delay,
            scale,
            rotate,
            translate
        );
        merge_vec!(
            display,
            inset,
            margin,
            padding,
            gap,
            border_radius,
            box_shadow,
            text_shadow,
            transform,
            filter,
            backdrop_filter,
            transition,
            animation,
            background,
            contain // …
        );
        self
    }
}
