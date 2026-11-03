use crate::styling::colors::black;
use crate::styling::style_variants::StyleVariants;
use crate::styling::styled::{StyledExt, weight_to_font_weight};
use crate::theme::active_theme;
use freya_core::data::Overflow;
use freya_core::prelude::{
    AspectRatio, Border, CornerRadius, CursorIcon, Fill, FontFeature, FontSize, FontSlant,
    FontWeight, FontWidth, ImageCover, Layer, LetterSpacing, SamplingMode, Scale, TextAlign,
    TextDecoration, TextOverflow, TextShadow, TransformOrigin,
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
    pub layer: Layer,

    // ---- border ----
    pub border: Option<Border>,

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

        // ---- typography: base size (prefer `md`, fall back to `base`) ----
        if let Some((size, _)) = theme
            .typography
            .font_size
            .get("md")
            .or_else(|| theme.typography.font_size.get("base"))
        {
            style.font_size = (*size).into();
        }

        // ---- typography: base weight (prefer `base`, fall back to `normal`) ----
        if let Some(w) = theme
            .typography
            .font_weight
            .get("base")
            .or_else(|| theme.typography.font_weight.get("normal"))
        {
            style.font_weight = weight_to_font_weight(*w);
        }

        // ---- typography: sans stack ----
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
            opacity: 1.0,
            scale: Scale { x: 1.0, y: 1.0 },
            transform_origin: TransformOrigin::default(),
            layer: Layer::default(),
            border: None,
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

        if let Some((size, _)) = theme
            .typography
            .font_size
            .get("md")
            .or_else(|| theme.typography.font_size.get("base"))
        {
            style.font_size = (*size).into();
        }

        if let Some(w) = theme
            .typography
            .font_weight
            .get("base")
            .or_else(|| theme.typography.font_weight.get("normal"))
        {
            style.font_weight = weight_to_font_weight(*w);
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

    /// The subset of properties that flow from a parent element to its
    /// descendants, mirroring the CSS `inherit`-able property set.
    ///
    /// Non-inheritable fields (`background`, `border`, `padding`,
    /// `margin`, `width`, `direction`, `content`, `variants`, effects,
    /// layout, media, …) are reset to their [`Style::bare`] values so that
    /// `parent.inheritable_subset()` can be merged *beneath* a child's own
    /// style without the parent bleeding box-model or paint properties into
    /// the child.
    ///
    /// # Caveat
    ///
    /// Because [`Style::merge_style`] uses a "differs from [`Style::bare`]"
    /// test, a field whose inherited value happens to equal its bare
    /// default **cannot** be explicitly restored by a child by setting it
    /// back to that default. `font_size`, `text_color`, and similar fields
    /// are most affected. If true CSS-like `unset` semantics are required,
    /// replace [`Style::merge_style`] with a patch-based merge built on
    /// `Option<T>` fields.
    pub fn inheritable_subset(&self) -> Style {
        let mut s = Style::bare();

        // ---- paint: foreground only (background does *not* inherit) ----
        s.text_color = self.text_color.clone();
        s.text_align = self.text_align.clone();

        // ---- typography: everything here cascades in CSS ----
        s.font_size = self.font_size.clone();
        s.font_weight = self.font_weight.clone();
        s.letter_spacing = self.letter_spacing.clone();
        s.text_overflow = self.text_overflow.clone();
        s.font_features = self.font_features.clone();
        s.font_families = self.font_families.clone();
        s.font_style = self.font_style.clone();
        s.font_stretch = self.font_stretch.clone();
        s.line_height = self.line_height;
        s.text_shadows = self.text_shadows.clone();
        s.text_decoration = self.text_decoration.clone();

        // ---- interaction ----
        s.cursor = self.cursor.clone();
        s.cursor_visible = self.cursor_visible;

        // ---- border: does NOT inherit (matches CSS) ----
        // left as `None` from `Style::bare()`

        // ---- layout: flow direction is inherited in CSS (`direction`) ----
        // Enable this if you want RTL to cascade down the tree:
        // s.direction = self.direction.clone();

        s
    }

    /// Merge `other` into `self`, with `other` taking precedence.
    ///
    /// Any field of `other` whose value differs from [`Style::bare`]'s
    /// counterpart is copied over. Fields that still hold their default
    /// value are left untouched, which makes this method convenient for
    /// composing *partial* overrides:
    ///
    /// ```
    /// let styled = Style::default().merge_style(&Style {
    ///     background: red().into(),
    ///     padding: Gaps::new(8.0, 8.0, 8.0, 8.0),
    ///     ..Style::bare()
    /// });
    /// ```
    ///
    /// Because of the "differs from default" rule, this method cannot be
    /// used to *reset* a field back to its default value; do that with a
    /// direct assignment or by starting from [`Style::bare`].
    ///
    /// Returns `self` so that merges can be chained.
    pub fn merge_style(mut self, other: &Style) -> Self {
        let bare = Style::bare();

        // ---- paint ----
        if other.background != bare.background {
            self.background = other.background.clone();
        }
        if other.text_color != bare.text_color {
            self.text_color = other.text_color.clone();
        }
        if other.text_align != bare.text_align {
            self.text_align = other.text_align.clone();
        }

        // ---- typography ----
        if other.font_size != bare.font_size {
            self.font_size = other.font_size.clone();
        }
        if other.font_weight != bare.font_weight {
            self.font_weight = other.font_weight.clone();
        }
        if other.letter_spacing != bare.letter_spacing {
            self.letter_spacing = other.letter_spacing.clone();
        }
        if other.text_overflow != bare.text_overflow {
            self.text_overflow = other.text_overflow.clone();
        }
        if !other.font_features.is_empty() {
            self.font_features = other.font_features.clone();
        }
        if !other.font_families.is_empty() {
            self.font_families = other.font_families.clone();
        }
        if other.font_style != bare.font_style {
            self.font_style = other.font_style.clone();
        }
        if other.font_stretch != bare.font_stretch {
            self.font_stretch = other.font_stretch.clone();
        }
        if other.line_height != bare.line_height {
            self.line_height = other.line_height;
        }
        if !other.text_shadows.is_empty() {
            self.text_shadows = other.text_shadows.clone();
        }
        if other.text_decoration != bare.text_decoration {
            self.text_decoration = other.text_decoration.clone();
        }

        // ---- interaction ----
        if other.cursor != bare.cursor {
            self.cursor = other.cursor.clone();
        }
        if other.cursor_visible != bare.cursor_visible {
            self.cursor_visible = other.cursor_visible;
        }

        // ---- effects ----
        if other.overflow != bare.overflow {
            self.overflow = other.overflow.clone();
        }
        if other.blur != bare.blur {
            self.blur = other.blur;
        }
        if other.rotation != bare.rotation {
            self.rotation = other.rotation;
        }
        if other.opacity != bare.opacity {
            self.opacity = other.opacity;
        }
        if other.scale != bare.scale {
            self.scale = other.scale.clone();
        }
        if other.transform_origin != bare.transform_origin {
            self.transform_origin = other.transform_origin.clone();
        }
        if other.layer != bare.layer {
            self.layer = other.layer.clone();
        }

        // ---- border ----
        if other.border != bare.border {
            self.border = other.border.clone();
        }

        // ---- radius ----
        if other.radius != bare.radius {
            self.radius = other.radius.clone();
        }

        // ---- layout: dimensions ----
        if other.width != bare.width {
            self.width = other.width.clone();
        }
        if other.height != bare.height {
            self.height = other.height.clone();
        }
        if other.min_width != bare.min_width {
            self.min_width = other.min_width.clone();
        }
        if other.min_height != bare.min_height {
            self.min_height = other.min_height.clone();
        }
        if other.max_width != bare.max_width {
            self.max_width = other.max_width.clone();
        }
        if other.max_height != bare.max_height {
            self.max_height = other.max_height.clone();
        }

        // ---- layout: box model ----
        if other.padding != bare.padding {
            self.padding = other.padding.clone();
        }
        if other.margin != bare.margin {
            self.margin = other.margin.clone();
        }

        // ---- layout: flow ----
        if other.spacing != bare.spacing {
            self.spacing = other.spacing.clone();
        }
        if other.offset_x != bare.offset_x {
            self.offset_x = other.offset_x.clone();
        }
        if other.offset_y != bare.offset_y {
            self.offset_y = other.offset_y.clone();
        }
        if other.direction != bare.direction {
            self.direction = other.direction.clone();
        }
        if other.main_align != bare.main_align {
            self.main_align = other.main_align.clone();
        }
        if other.cross_align != bare.cross_align {
            self.cross_align = other.cross_align.clone();
        }
        if other.position != bare.position {
            self.position = other.position.clone();
        }
        if other.content != bare.content {
            self.content = other.content.clone();
        }

        // ---- media ----
        if other.object_fit != bare.object_fit {
            self.object_fit = other.object_fit.clone();
        }
        if other.object_position != bare.object_position {
            self.object_position = other.object_position.clone();
        }
        if other.sampling_mode != bare.sampling_mode {
            self.sampling_mode = other.sampling_mode.clone();
        }
        if other.image_blur != bare.image_blur {
            self.image_blur = other.image_blur;
        }
        if other.snap_to_grid != bare.snap_to_grid {
            self.snap_to_grid = other.snap_to_grid;
        }

        // ---- variants ----
        if other.variants != bare.variants {
            self.variants = other.variants.clone();
        }

        self
    }
}

impl StyledExt for Style {
    fn get_style(&mut self) -> &mut Style {
        self
    }
}

/// Context payload carrying the resolved style of the nearest `Div` ancestor.
/// Wrapped in a newtype so it can't be confused with any other `Style`
/// context in the tree.
#[derive(Clone, PartialEq)]
pub struct InheritedStyle(pub Style);
