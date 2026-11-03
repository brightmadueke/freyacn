//! Inline Tailwind-style class strings for [`StyledExt`].
//!
//! Colour utilities (`bg-*`, `text-*`, `border-*`, `ring-*`) resolve
//! their `<color>` argument in three steps: the built-in palette, the
//! active theme's fill map, then the active theme's color map. Tokens
//! the theme defines — `primary`, `destructive`, `ring`, `input`,
//! `background`, `foreground`, `secondary`, `accent`, `surface`, … —
//! resolve via those last two.

use crate::styling::colors::{self as c, StyledColor};
use crate::styling::style::Style;
use crate::styling::style_variants::{StyleVariant, VariantFn};
use crate::styling::styled::StyledExt;
use freya_core::data::Overflow;
use freya_core::elements::image::{AspectRatio, ImageCover, SamplingMode};
use freya_core::prelude::{
    CursorIcon, Fill, FontFeature, FontSlant, FontWeight, FontWidth, Layer, TextAlign,
    TextDecoration, TextOverflow, TransformOrigin,
};
use std::borrow::Cow;
use std::sync::Arc;
use torin::prelude::{Alignment, Direction, Length};
use torin::size::Size;

// ============================================================================
// Public type
// ============================================================================

#[derive(Clone, Default)]
pub struct StyledString {
    base: Vec<Op>,
    variants: Vec<(Vec<VariantKind>, Arc<[String]>)>,
}

impl StyledString {
    pub fn parse(input: &str) -> Self {
        let mut this = Self::default();
        let mut variant_classes: Vec<(Vec<VariantKind>, Vec<String>)> = Vec::new();

        for class in split_classes(input) {
            let (prefixes, rest) = split_variants(class);

            let mut kinds: Vec<VariantKind> = Vec::with_capacity(prefixes.len());
            for prefix in &prefixes {
                match VariantKind::from_str(prefix) {
                    Some(k) => kinds.push(k),
                    None => panic_unknown(class),
                }
            }
            kinds.sort();
            kinds.dedup();

            let op = match parse_utility(rest) {
                Some(op) => op,
                None => panic_unknown(class),
            };

            if kinds.is_empty() {
                this.base.push(op);
            } else {
                drop(op);
                match variant_classes.iter_mut().find(|(k, _)| *k == kinds) {
                    Some((_, classes)) => classes.push(rest.to_owned()),
                    None => variant_classes.push((kinds, vec![rest.to_owned()])),
                }
            }
        }

        this.variants = variant_classes
            .into_iter()
            .map(|(k, v)| (k, Arc::from(v.into_boxed_slice())))
            .collect();

        this.base.sort_by_key(Op::is_tracking);
        this
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_empty() && self.variants.is_empty()
    }

    pub fn apply<T: StyledExt>(&self, target: T) -> T {
        let mut t = apply_ops(target, &self.base);

        for (kinds, classes) in &self.variants {
            let style_kinds: Vec<StyleVariant> =
                kinds.iter().map(|k| k.to_style_variant()).collect();
            let classes = Arc::clone(classes);
            let f: VariantFn = Arc::new(move |s: Style| {
                let mut ops: Vec<Op> = classes
                    .iter()
                    .map(|c| match parse_utility(c) {
                        Some(op) => op,
                        None => panic_unknown(c),
                    })
                    .collect();
                ops.sort_by_key(Op::is_tracking);
                apply_ops(s, &ops)
            });

            t.get_style().variants.push_compound(style_kinds, f);
        }
        t
    }
}

impl From<&str> for StyledString {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}
impl From<String> for StyledString {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}
impl From<&String> for StyledString {
    fn from(s: &String) -> Self {
        Self::parse(s)
    }
}
impl From<Cow<'static, str>> for StyledString {
    fn from(s: Cow<'static, str>) -> Self {
        Self::parse(&s)
    }
}

#[track_caller]
fn panic_unknown(class: &str) -> ! {
    let (prefixes, rest) = split_variants(class);
    let hint = if prefixes.is_empty() {
        String::new()
    } else {
        let unknown: Vec<&str> = prefixes
            .iter()
            .copied()
            .filter(|p| VariantKind::from_str(p).is_none())
            .collect();
        if !unknown.is_empty() {
            format!(
                " (unknown variant prefix(es) {unknown:?}; expected one of \
                 `dark`, `hover`, `focus`, `focus-visible`, `focus-within`, \
                 `active`, `disabled`, `checked`, `selected`, `aria-invalid`, \
                 `aria-checked`, `aria-disabled`, `aria-expanded`, `aria-pressed`, \
                 `aria-readonly`, `aria-required`, `aria-selected`, `aria-hidden`)"
            )
        } else if rest.is_empty() {
            " (class ends with `:` but no utility follows)".to_string()
        } else {
            format!(" (utility `{rest}` after the variant prefix(es) didn't parse)")
        }
    };
    panic!(
        "[StyledString] unknown class `{class}`{hint}. \
         Expected a Tailwind-style utility (e.g. `p-4`, `bg-blue-500`, `font-bold`), \
         a variant-prefixed class (e.g. `hover:bg-blue-500`, `dark:hover:bg-blue-500`), \
         a ring utility (e.g. `ring`, `ring-2`, `ring-[3px]`, `ring-ring/50`), \
         or a bracketed arbitrary value (e.g. `w-[300px]`, `bg-[#ff0000]`). \
         Colour names must be either a palette entry (`blue-500`, `neutral-950`, \
         `white`, `black`, `transparent`) or a token the active theme defines \
         (`primary`, `primary-foreground`, `ring`, `input`, …). \
         If the token is in the theme JSON but still missing here, the theme's \
         `fill()` / `color()` accessors don't expose it — see the debug log line \
         printed just before this panic for the exact keys that were tried."
    );
}

// ============================================================================
// Splitting
// ============================================================================

fn split_classes(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;

    for (i, ch) in input.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            c if c.is_whitespace() && depth == 0 => {
                if i > start {
                    out.push(&input[start..i]);
                }
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }

    if start < input.len() {
        out.push(&input[start..]);
    }
    out
}

fn split_variants(class: &str) -> (Vec<&str>, &str) {
    let mut depth = 0i32;
    let mut prefixes = Vec::new();
    let mut last = 0;

    for (i, ch) in class.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => {
                prefixes.push(&class[last..i]);
                last = i + 1;
            }
            _ => {}
        }
    }

    (prefixes, &class[last..])
}

// ============================================================================
// Variants
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum VariantKind {
    Dark,
    Hover,
    Focus,
    FocusVisible,
    FocusWithin,
    Selected,
    AriaSelected,
    Checked,
    AriaChecked,
    Active,
    AriaExpanded,
    AriaPressed,
    AriaInvalid,
    AriaRequired,
    AriaReadonly,
    AriaHidden,
    AriaDisabled,
    Disabled,
}

impl VariantKind {
    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "dark" => Self::Dark,
            "hover" => Self::Hover,
            "focus" => Self::Focus,
            "focus-visible" => Self::FocusVisible,
            "focus-within" => Self::FocusWithin,
            "selected" => Self::Selected,
            "aria-selected" => Self::AriaSelected,
            "checked" => Self::Checked,
            "aria-checked" => Self::AriaChecked,
            "active" => Self::Active,
            "aria-expanded" => Self::AriaExpanded,
            "aria-pressed" => Self::AriaPressed,
            "aria-invalid" => Self::AriaInvalid,
            "aria-required" => Self::AriaRequired,
            "aria-readonly" => Self::AriaReadonly,
            "aria-hidden" => Self::AriaHidden,
            "aria-disabled" => Self::AriaDisabled,
            "disabled" => Self::Disabled,
            _ => return None,
        })
    }

    fn to_style_variant(self) -> StyleVariant {
        match self {
            Self::Dark => StyleVariant::Dark,
            Self::Hover => StyleVariant::Hover,
            Self::Focus => StyleVariant::Focus,
            Self::FocusVisible => StyleVariant::FocusVisible,
            Self::FocusWithin => StyleVariant::FocusWithin,
            Self::Selected => StyleVariant::Selected,
            Self::AriaSelected => StyleVariant::AriaSelected,
            Self::Checked => StyleVariant::Checked,
            Self::AriaChecked => StyleVariant::AriaChecked,
            Self::Active => StyleVariant::Active,
            Self::AriaExpanded => StyleVariant::AriaExpanded,
            Self::AriaPressed => StyleVariant::AriaPressed,
            Self::AriaInvalid => StyleVariant::AriaInvalid,
            Self::AriaRequired => StyleVariant::AriaRequired,
            Self::AriaReadonly => StyleVariant::AriaReadonly,
            Self::AriaHidden => StyleVariant::AriaHidden,
            Self::AriaDisabled => StyleVariant::AriaDisabled,
            Self::Disabled => StyleVariant::Disabled,
        }
    }
}

// ============================================================================
// Operations
// ============================================================================

#[derive(Clone, Copy)]
enum Side {
    All,
    X,
    Y,
    T,
    R,
    B,
    L,
}

#[derive(Clone, Copy)]
enum Edge {
    All,
    T,
    R,
    B,
    L,
    Tl,
    Tr,
    Br,
    Bl,
}

#[derive(Clone, Copy)]
enum BorderEdge {
    All,
    X,
    Y,
    T,
    R,
    B,
    L,
}

#[derive(Clone, Copy)]
enum Dim {
    Width,
    Height,
    Both,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
}

#[derive(Clone, Copy)]
enum Family {
    Sans,
    Serif,
    Mono,
}

#[derive(Clone, Copy)]
enum ShadowSize {
    None,
    S2xs,
    Xs,
    Sm,
    Md,
    Lg,
}

#[derive(Clone)]
enum Op {
    Bg(StyledColor),
    TextColor(StyledColor),
    TextAlign(TextAlign),
    TextOverflow(TextOverflow),
    FontSize(f32),
    FontWeight(FontWeight),
    FontStyle(FontSlant),
    FontStretch(FontWidth),
    FontFeature(FontFeature),
    FontFamily(Family),
    Tracking(f32),
    LineHeight(f32),
    TextShadow(ShadowSize),
    TextDecoration(TextDecoration),
    Cursor(CursorIcon),
    CursorHidden,
    Overflow(Overflow),
    BackdropBlur(f32),
    Rotate(f32),
    Opacity(f32),
    Scale(f32),
    ScaleX(f32),
    ScaleY(f32),
    Origin(TransformOrigin),
    Radius(Edge, f32),
    BorderWidth(BorderEdge, f32),
    BorderColor(StyledColor),
    BorderNone,
    BorderInner,
    BorderOuter,
    BorderCenter,
    Padding(Side, f32),
    Margin(Side, f32),
    Dim(Dim, Size),
    Gap(f32),
    Direction(Direction),
    MainAlign(Alignment),
    CrossAlign(Alignment),
    Absolute,
    Relative,
    Truncate,
    Layer(Layer),
    ObjectFit(AspectRatio),
    ObjectScaleDown,
    ObjectPosition(ImageCover),
    Sampling(SamplingMode),
    ImageBlur(f32),
    SnapToGrid(bool),
}

impl Op {
    fn is_tracking(&self) -> bool {
        matches!(self, Op::Tracking(_))
    }

    fn negated(self) -> Option<Op> {
        Some(match self {
            Op::Margin(s, v) => Op::Margin(s, -v),
            Op::Rotate(v) => Op::Rotate(-v),
            Op::Scale(v) => Op::Scale(-v),
            Op::ScaleX(v) => Op::ScaleX(-v),
            Op::ScaleY(v) => Op::ScaleY(-v),
            Op::Layer(Layer::Relative(n)) => Op::Layer(Layer::Relative(n.checked_neg()?)),
            _ => return None,
        })
    }

    fn apply<T: StyledExt>(self, t: T) -> T {
        match self {
            Op::Bg(col) => t.bg(col),
            Op::TextColor(col) => t.text_color(col),
            Op::TextAlign(a) => t.text_align(a),
            Op::TextOverflow(o) => t.text_overflow(o),
            Op::FontSize(s) => t.font_size(s),
            Op::FontWeight(w) => t.font_weight(w),
            Op::FontStyle(s) => t.font_style(s),
            Op::FontStretch(s) => t.font_stretch(s),
            Op::FontFeature(f) => t.font_feature(f),
            Op::FontFamily(f) => match f {
                Family::Sans => t.font_sans(),
                Family::Serif => t.font_serif(),
                Family::Mono => t.font_mono(),
            },
            Op::Tracking(em) => t.tracking(em),
            Op::LineHeight(l) => t.line_height(l),
            Op::TextShadow(s) => match s {
                ShadowSize::None => t.text_shadow_none(),
                ShadowSize::S2xs => t.text_shadow_2xs(),
                ShadowSize::Xs => t.text_shadow_xs(),
                ShadowSize::Sm => t.text_shadow_sm(),
                ShadowSize::Md => t.text_shadow_md(),
                ShadowSize::Lg => t.text_shadow_lg(),
            },
            Op::TextDecoration(d) => t.text_decoration(d),
            Op::Cursor(cur) => t.cursor(cur),
            Op::CursorHidden => t.cursor_none(),
            Op::Overflow(o) => t.overflow(o),
            Op::BackdropBlur(b) => t.backdrop_blur(b),
            Op::Rotate(r) => t.rotate(r),
            Op::Opacity(o) => t.opacity(o),
            Op::Scale(s) => t.scale(s),
            Op::ScaleX(s) => t.scale_x(s),
            Op::ScaleY(s) => t.scale_y(s),
            Op::Origin(o) => t.transform_origin(o),
            Op::Radius(e, v) => match e {
                Edge::All => t.radius(v),
                Edge::T => t.radius_t(v),
                Edge::R => t.radius_r(v),
                Edge::B => t.radius_b(v),
                Edge::L => t.radius_l(v),
                Edge::Tl => t.radius_tl(v),
                Edge::Tr => t.radius_tr(v),
                Edge::Br => t.radius_br(v),
                Edge::Bl => t.radius_bl(v),
            },
            Op::BorderWidth(edge, width) => match edge {
                BorderEdge::All => t.border_width(width),
                BorderEdge::X => t.border_x(width),
                BorderEdge::Y => t.border_y(width),
                BorderEdge::T => t.border_t(width),
                BorderEdge::R => t.border_r(width),
                BorderEdge::B => t.border_b(width),
                BorderEdge::L => t.border_l(width),
            },
            Op::BorderColor(c) => t.border_color(c),
            Op::BorderNone => t.border_none(),
            Op::BorderInner => t.border_inner(),
            Op::BorderOuter => t.border_outer(),
            Op::BorderCenter => t.border_center(),
            Op::Padding(s, v) => match s {
                Side::All => t.p(v),
                Side::X => t.px(v),
                Side::Y => t.py(v),
                Side::T => t.pt(v),
                Side::R => t.pr(v),
                Side::B => t.pb(v),
                Side::L => t.pl(v),
            },
            Op::Margin(s, v) => match s {
                Side::All => t.m(v),
                Side::X => t.mx(v),
                Side::Y => t.my(v),
                Side::T => t.mt(v),
                Side::R => t.mr(v),
                Side::B => t.mb(v),
                Side::L => t.ml(v),
            },
            Op::Dim(dim, size) => {
                let mut t = t;
                let s = t.get_style();
                match dim {
                    Dim::Width => s.width = size,
                    Dim::Height => s.height = size,
                    Dim::Both => {
                        s.width = size.clone();
                        s.height = size;
                    }
                    Dim::MinWidth => s.min_width = size,
                    Dim::MinHeight => s.min_height = size,
                    Dim::MaxWidth => s.max_width = size,
                    Dim::MaxHeight => s.max_height = size,
                }
                t
            }
            Op::Gap(v) => t.gap(v),
            Op::Direction(d) => t.direction(d),
            Op::MainAlign(a) => t.main_align(a),
            Op::CrossAlign(a) => t.cross_align(a),
            Op::Absolute => t.absolute(),
            Op::Relative => t.relative(),
            Op::Truncate => t.truncate(),
            Op::Layer(l) => t.layer(l),
            Op::ObjectFit(a) => t.object_fit(a),
            Op::ObjectScaleDown => t.object_scale_down(),
            Op::ObjectPosition(p) => t.object_position(p),
            Op::Sampling(m) => t.image_rendering(m),
            Op::ImageBlur(b) => t.image_blur(b),
            Op::SnapToGrid(true) => t.snap_to_grid(),
            Op::SnapToGrid(false) => t.no_snap_to_grid(),
        }
    }
}

fn apply_ops<T: StyledExt>(target: T, ops: &[Op]) -> T {
    ops.iter().cloned().fold(target, |t, op| op.apply(t))
}

// ============================================================================
// Image utilities
// ============================================================================

fn parse_image(c: &str) -> Option<Op> {
    Some(match c {
        "object-contain" => Op::ObjectFit(AspectRatio::Min),
        "object-cover" => Op::ObjectFit(AspectRatio::Max),
        "object-fill" => Op::ObjectFit(AspectRatio::None),
        "object-none" => Op::ObjectFit(AspectRatio::Fit),
        "object-scale-down" => Op::ObjectScaleDown,
        "object-center" => Op::ObjectPosition(ImageCover::Center),
        "object-top-left" | "object-left-top" => Op::ObjectPosition(ImageCover::Fill),
        "image-auto" => Op::Sampling(SamplingMode::Trilinear),
        "image-pixelated" => Op::Sampling(SamplingMode::Nearest),
        "image-smooth" => Op::Sampling(SamplingMode::Mitchell),
        "sampling-nearest" => Op::Sampling(SamplingMode::Nearest),
        "sampling-bilinear" => Op::Sampling(SamplingMode::Bilinear),
        "sampling-trilinear" => Op::Sampling(SamplingMode::Trilinear),
        "sampling-mitchell" => Op::Sampling(SamplingMode::Mitchell),
        "sampling-catmull-rom" => Op::Sampling(SamplingMode::CatmullRom),
        "snap-to-grid" => Op::SnapToGrid(true),
        "no-snap-to-grid" => Op::SnapToGrid(false),
        "blur" => Op::ImageBlur(8.0),
        "blur-none" => Op::ImageBlur(0.0),
        "blur-2xs" => Op::ImageBlur(2.0),
        "blur-xs" => Op::ImageBlur(4.0),
        "blur-sm" => Op::ImageBlur(8.0),
        "blur-md" => Op::ImageBlur(12.0),
        "blur-lg" => Op::ImageBlur(16.0),
        "blur-xl" => Op::ImageBlur(24.0),
        "blur-2xl" => Op::ImageBlur(40.0),
        "blur-3xl" => Op::ImageBlur(64.0),
        other => Op::ImageBlur(arbitrary_px(other.strip_prefix("blur-")?)?),
    })
}

// ============================================================================
// Value helpers
// ============================================================================

fn px(v: f32) -> Size {
    Size::Pixels(Length::new(v))
}
fn pct(v: f32) -> Size {
    Size::Percentage(Length::new(v))
}

fn plain_num(s: &str) -> Option<f32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    s.parse().ok()
}

fn signed_num(s: &str) -> Option<f32> {
    match s.strip_prefix('-') {
        Some(rest) => plain_num(rest).map(|n| -n),
        None => plain_num(s),
    }
}

fn arbitrary(s: &str) -> Option<&str> {
    s.strip_prefix('[')?.strip_suffix(']')
}

fn arbitrary_px(s: &str) -> Option<f32> {
    let inner = arbitrary(s)?.trim();
    if let Some(n) = inner.strip_suffix("px") {
        plain_num(n)
    } else if let Some(n) = inner.strip_suffix("rem") {
        plain_num(n).map(|n| n * 16.0)
    } else {
        plain_num(inner)
    }
}

fn arbitrary_num(s: &str) -> Option<f32> {
    signed_num(arbitrary(s)?.trim())
}

fn spacing(s: &str) -> Option<f32> {
    if s == "px" {
        Some(1.0)
    } else if let Some(n) = plain_num(s) {
        Some(n * 4.0)
    } else {
        arbitrary_px(s)
    }
}

fn percent(s: &str) -> Option<f32> {
    plain_num(s).map(|n| n / 100.0).or_else(|| arbitrary_num(s))
}

fn size_value(v: &str) -> Option<Size> {
    match v {
        "auto" | "fit" => return Some(Size::Inner),
        "full" | "screen" => return Some(pct(100.0)),
        _ => {}
    }
    if let Some((a, b)) = v.split_once('/') {
        let (a, b) = (plain_num(a)?, plain_num(b)?);
        if b == 0.0 {
            return None;
        }
        return Some(pct(a * 100.0 / b));
    }
    if let Some(n) = arbitrary(v).and_then(|i| i.strip_suffix('%')) {
        return plain_num(n).map(pct);
    }
    spacing(v).map(px)
}

fn container_size(v: &str) -> Option<f32> {
    Some(match v {
        "xs" => 320.0,
        "sm" => 384.0,
        "md" => 448.0,
        "lg" => 512.0,
        "xl" => 576.0,
        "2xl" => 672.0,
        "3xl" => 768.0,
        "4xl" => 896.0,
        "5xl" => 1024.0,
        "6xl" => 1152.0,
        "7xl" => 1280.0,
        _ => return None,
    })
}

fn color_value(v: &str) -> Option<StyledColor> {
    let (name, alpha) = match v.split_once('/') {
        Some((n, a)) => (n, Some(plain_num(a)? / 100.0)),
        None => (v, None),
    };
    let col = color_name_or_hex(name)?;
    Some(match alpha {
        Some(a) => col.opacity(a),
        None => col,
    })
}

fn color_name_or_hex(name: &str) -> Option<StyledColor> {
    if let Some(inner) = arbitrary(name) {
        let (r, g, b, a) = parse_hex_rgba(inner.trim())?;
        return Some(StyledColor::argb(a, r, g, b));
    }
    color(name)
}

fn parse_hex_rgba(hex: &str) -> Option<(u8, u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    let expand = |h: &str| -> String { h.chars().flat_map(|c| [c, c]).collect() };
    let (rgb, alpha) = match s.len() {
        3 => (expand(s), 255u8),
        4 => {
            let a = u8::from_str_radix(&expand(&s[3..]), 16).ok()?;
            (expand(&s[..3]), a)
        }
        6 => (s.to_string(), 255u8),
        8 => {
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            (s[..6].to_string(), a)
        }
        _ => return None,
    };
    if rgb.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    Some((r, g, b, alpha))
}

/// Resolve a colour name.
///
/// Lookup order:
///
/// 1. the built-in palette (`blue-500`, `neutral-950`, `white`, …)
/// 2. the active theme's **fill** map, tried under every common naming
///    convention: `primary-foreground`, `primaryForeground`,
///    `primary_foreground`, `primary.foreground`, `--primary-foreground`
/// 3. the active theme's **color** map (border-only: `ring`, `input`,
///    `border`), with the same naming variations
///
/// Returns `None` only when nothing matches. In debug builds a
/// diagnostic line is printed with the exact keys that were tried,
/// so a missing token in the theme is easy to spot.
fn color(name: &str) -> Option<StyledColor> {
    // 1. Built-in palette (fast path).
    if let Some(col) = builtin_color(name) {
        return Some(col);
    }

    let theme = crate::theme::active_theme();

    // 2. Candidate key spellings for this token name.
    let mut candidates: Vec<String> = Vec::with_capacity(8);
    candidates.push(name.to_string()); // as written in the class
    let camel = kebab_to_camel(name);
    if camel != name {
        candidates.push(camel.clone()); // primaryForeground
    }
    let snake = name.replace('-', "_");
    if !candidates.contains(&snake) {
        candidates.push(snake); // primary_foreground
    }
    let dotted = name.replace('-', ".");
    if !candidates.contains(&dotted) {
        candidates.push(dotted); // primary.foreground
    }
    candidates.push(format!("--{}", name)); // --primary-foreground

    // 3. Try `fill` under every spelling, then `color`.
    for candidate in &candidates {
        if let Some(fill) = theme.fill(candidate) {
            return Some(StyledColor::from_fill(fill));
        }
    }
    for candidate in &candidates {
        if let Some(color) = theme.color(candidate) {
            return Some(StyledColor::from_fill(Fill::Color(color)));
        }
    }

    // 4. Diagnostic — only compiled into debug builds, only on failure.
    #[cfg(debug_assertions)]
    {
        eprintln!(
            "[StyledString] color({name:?}): not found in theme. \
             Tried keys: {candidates:?}. \
             If the theme JSON contains `{name}`, its `fill()` / `color()` \
             accessors don't expose that key. Add the token to the theme's \
             fill or color map under one of those spellings — usually \
             `{camel}` for backgrounds/text and `{name}` (or `ring`/`input`/`border`) \
             for borders."
        );
    }

    None
}

/// `primary-foreground` → `primaryForeground`
/// `sidebar-accent-foreground` → `sidebarAccentForeground`
/// `ring` → `ring`
/// `--` / `__` treated as separators (each triggers the next upper).
fn kebab_to_camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper_next = false;
    for ch in name.chars() {
        if ch == '-' || ch == '_' {
            upper_next = true;
        } else if upper_next {
            for c in ch.to_uppercase() {
                out.push(c);
            }
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn builtin_color(name: &str) -> Option<StyledColor> {
    Some(match name {
        "black" => c::black(),
        "white" => c::white(),
        "transparent" => c::transparent(),
        "mauve-50" => c::mauve_50(),
        "mauve-100" => c::mauve_100(),
        "mauve-200" => c::mauve_200(),
        "mauve-300" => c::mauve_300(),
        "mauve-400" => c::mauve_400(),
        "mauve-500" => c::mauve_500(),
        "mauve-600" => c::mauve_600(),
        "mauve-700" => c::mauve_700(),
        "mauve-800" => c::mauve_800(),
        "mauve-900" => c::mauve_900(),
        "mauve-950" => c::mauve_950(),
        "olive-50" => c::olive_50(),
        "olive-100" => c::olive_100(),
        "olive-200" => c::olive_200(),
        "olive-300" => c::olive_300(),
        "olive-400" => c::olive_400(),
        "olive-500" => c::olive_500(),
        "olive-600" => c::olive_600(),
        "olive-700" => c::olive_700(),
        "olive-800" => c::olive_800(),
        "olive-900" => c::olive_900(),
        "olive-950" => c::olive_950(),
        "mist-50" => c::mist_50(),
        "mist-100" => c::mist_100(),
        "mist-200" => c::mist_200(),
        "mist-300" => c::mist_300(),
        "mist-400" => c::mist_400(),
        "mist-500" => c::mist_500(),
        "mist-600" => c::mist_600(),
        "mist-700" => c::mist_700(),
        "mist-800" => c::mist_800(),
        "mist-900" => c::mist_900(),
        "mist-950" => c::mist_950(),
        "taupe-50" => c::taupe_50(),
        "taupe-100" => c::taupe_100(),
        "taupe-200" => c::taupe_200(),
        "taupe-300" => c::taupe_300(),
        "taupe-400" => c::taupe_400(),
        "taupe-500" => c::taupe_500(),
        "taupe-600" => c::taupe_600(),
        "taupe-700" => c::taupe_700(),
        "taupe-800" => c::taupe_800(),
        "taupe-900" => c::taupe_900(),
        "taupe-950" => c::taupe_950(),
        "slate-50" => c::slate_50(),
        "slate-100" => c::slate_100(),
        "slate-200" => c::slate_200(),
        "slate-300" => c::slate_300(),
        "slate-400" => c::slate_400(),
        "slate-500" => c::slate_500(),
        "slate-600" => c::slate_600(),
        "slate-700" => c::slate_700(),
        "slate-800" => c::slate_800(),
        "slate-900" => c::slate_900(),
        "slate-950" => c::slate_950(),
        "gray-50" => c::gray_50(),
        "gray-100" => c::gray_100(),
        "gray-200" => c::gray_200(),
        "gray-300" => c::gray_300(),
        "gray-400" => c::gray_400(),
        "gray-500" => c::gray_500(),
        "gray-600" => c::gray_600(),
        "gray-700" => c::gray_700(),
        "gray-800" => c::gray_800(),
        "gray-900" => c::gray_900(),
        "gray-950" => c::gray_950(),
        "zinc-50" => c::zinc_50(),
        "zinc-100" => c::zinc_100(),
        "zinc-200" => c::zinc_200(),
        "zinc-300" => c::zinc_300(),
        "zinc-400" => c::zinc_400(),
        "zinc-500" => c::zinc_500(),
        "zinc-600" => c::zinc_600(),
        "zinc-700" => c::zinc_700(),
        "zinc-800" => c::zinc_800(),
        "zinc-900" => c::zinc_900(),
        "zinc-950" => c::zinc_950(),
        "neutral-50" => c::neutral_50(),
        "neutral-100" => c::neutral_100(),
        "neutral-200" => c::neutral_200(),
        "neutral-300" => c::neutral_300(),
        "neutral-400" => c::neutral_400(),
        "neutral-500" => c::neutral_500(),
        "neutral-600" => c::neutral_600(),
        "neutral-700" => c::neutral_700(),
        "neutral-800" => c::neutral_800(),
        "neutral-900" => c::neutral_900(),
        "neutral-950" => c::neutral_950(),
        "stone-50" => c::stone_50(),
        "stone-100" => c::stone_100(),
        "stone-200" => c::stone_200(),
        "stone-300" => c::stone_300(),
        "stone-400" => c::stone_400(),
        "stone-500" => c::stone_500(),
        "stone-600" => c::stone_600(),
        "stone-700" => c::stone_700(),
        "stone-800" => c::stone_800(),
        "stone-900" => c::stone_900(),
        "stone-950" => c::stone_950(),
        "red-50" => c::red_50(),
        "red-100" => c::red_100(),
        "red-200" => c::red_200(),
        "red-300" => c::red_300(),
        "red-400" => c::red_400(),
        "red-500" => c::red_500(),
        "red-600" => c::red_600(),
        "red-700" => c::red_700(),
        "red-800" => c::red_800(),
        "red-900" => c::red_900(),
        "red-950" => c::red_950(),
        "orange-50" => c::orange_50(),
        "orange-100" => c::orange_100(),
        "orange-200" => c::orange_200(),
        "orange-300" => c::orange_300(),
        "orange-400" => c::orange_400(),
        "orange-500" => c::orange_500(),
        "orange-600" => c::orange_600(),
        "orange-700" => c::orange_700(),
        "orange-800" => c::orange_800(),
        "orange-900" => c::orange_900(),
        "orange-950" => c::orange_950(),
        "amber-50" => c::amber_50(),
        "amber-100" => c::amber_100(),
        "amber-200" => c::amber_200(),
        "amber-300" => c::amber_300(),
        "amber-400" => c::amber_400(),
        "amber-500" => c::amber_500(),
        "amber-600" => c::amber_600(),
        "amber-700" => c::amber_700(),
        "amber-800" => c::amber_800(),
        "amber-900" => c::amber_900(),
        "amber-950" => c::amber_950(),
        "yellow-50" => c::yellow_50(),
        "yellow-100" => c::yellow_100(),
        "yellow-200" => c::yellow_200(),
        "yellow-300" => c::yellow_300(),
        "yellow-400" => c::yellow_400(),
        "yellow-500" => c::yellow_500(),
        "yellow-600" => c::yellow_600(),
        "yellow-700" => c::yellow_700(),
        "yellow-800" => c::yellow_800(),
        "yellow-900" => c::yellow_900(),
        "yellow-950" => c::yellow_950(),
        "lime-50" => c::lime_50(),
        "lime-100" => c::lime_100(),
        "lime-200" => c::lime_200(),
        "lime-300" => c::lime_300(),
        "lime-400" => c::lime_400(),
        "lime-500" => c::lime_500(),
        "lime-600" => c::lime_600(),
        "lime-700" => c::lime_700(),
        "lime-800" => c::lime_800(),
        "lime-900" => c::lime_900(),
        "lime-950" => c::lime_950(),
        "green-50" => c::green_50(),
        "green-100" => c::green_100(),
        "green-200" => c::green_200(),
        "green-300" => c::green_300(),
        "green-400" => c::green_400(),
        "green-500" => c::green_500(),
        "green-600" => c::green_600(),
        "green-700" => c::green_700(),
        "green-800" => c::green_800(),
        "green-900" => c::green_900(),
        "green-950" => c::green_950(),
        "emerald-50" => c::emerald_50(),
        "emerald-100" => c::emerald_100(),
        "emerald-200" => c::emerald_200(),
        "emerald-300" => c::emerald_300(),
        "emerald-400" => c::emerald_400(),
        "emerald-500" => c::emerald_500(),
        "emerald-600" => c::emerald_600(),
        "emerald-700" => c::emerald_700(),
        "emerald-800" => c::emerald_800(),
        "emerald-900" => c::emerald_900(),
        "emerald-950" => c::emerald_950(),
        "teal-50" => c::teal_50(),
        "teal-100" => c::teal_100(),
        "teal-200" => c::teal_200(),
        "teal-300" => c::teal_300(),
        "teal-400" => c::teal_400(),
        "teal-500" => c::teal_500(),
        "teal-600" => c::teal_600(),
        "teal-700" => c::teal_700(),
        "teal-800" => c::teal_800(),
        "teal-900" => c::teal_900(),
        "teal-950" => c::teal_950(),
        "cyan-50" => c::cyan_50(),
        "cyan-100" => c::cyan_100(),
        "cyan-200" => c::cyan_200(),
        "cyan-300" => c::cyan_300(),
        "cyan-400" => c::cyan_400(),
        "cyan-500" => c::cyan_500(),
        "cyan-600" => c::cyan_600(),
        "cyan-700" => c::cyan_700(),
        "cyan-800" => c::cyan_800(),
        "cyan-900" => c::cyan_900(),
        "cyan-950" => c::cyan_950(),
        "sky-50" => c::sky_50(),
        "sky-100" => c::sky_100(),
        "sky-200" => c::sky_200(),
        "sky-300" => c::sky_300(),
        "sky-400" => c::sky_400(),
        "sky-500" => c::sky_500(),
        "sky-600" => c::sky_600(),
        "sky-700" => c::sky_700(),
        "sky-800" => c::sky_800(),
        "sky-900" => c::sky_900(),
        "sky-950" => c::sky_950(),
        "blue-50" => c::blue_50(),
        "blue-100" => c::blue_100(),
        "blue-200" => c::blue_200(),
        "blue-300" => c::blue_300(),
        "blue-400" => c::blue_400(),
        "blue-500" => c::blue_500(),
        "blue-600" => c::blue_600(),
        "blue-700" => c::blue_700(),
        "blue-800" => c::blue_800(),
        "blue-900" => c::blue_900(),
        "blue-950" => c::blue_950(),
        "indigo-50" => c::indigo_50(),
        "indigo-100" => c::indigo_100(),
        "indigo-200" => c::indigo_200(),
        "indigo-300" => c::indigo_300(),
        "indigo-400" => c::indigo_400(),
        "indigo-500" => c::indigo_500(),
        "indigo-600" => c::indigo_600(),
        "indigo-700" => c::indigo_700(),
        "indigo-800" => c::indigo_800(),
        "indigo-900" => c::indigo_900(),
        "indigo-950" => c::indigo_950(),
        "violet-50" => c::violet_50(),
        "violet-100" => c::violet_100(),
        "violet-200" => c::violet_200(),
        "violet-300" => c::violet_300(),
        "violet-400" => c::violet_400(),
        "violet-500" => c::violet_500(),
        "violet-600" => c::violet_600(),
        "violet-700" => c::violet_700(),
        "violet-800" => c::violet_800(),
        "violet-900" => c::violet_900(),
        "violet-950" => c::violet_950(),
        "purple-50" => c::purple_50(),
        "purple-100" => c::purple_100(),
        "purple-200" => c::purple_200(),
        "purple-300" => c::purple_300(),
        "purple-400" => c::purple_400(),
        "purple-500" => c::purple_500(),
        "purple-600" => c::purple_600(),
        "purple-700" => c::purple_700(),
        "purple-800" => c::purple_800(),
        "purple-900" => c::purple_900(),
        "purple-950" => c::purple_950(),
        "fuchsia-50" => c::fuchsia_50(),
        "fuchsia-100" => c::fuchsia_100(),
        "fuchsia-200" => c::fuchsia_200(),
        "fuchsia-300" => c::fuchsia_300(),
        "fuchsia-400" => c::fuchsia_400(),
        "fuchsia-500" => c::fuchsia_500(),
        "fuchsia-600" => c::fuchsia_600(),
        "fuchsia-700" => c::fuchsia_700(),
        "fuchsia-800" => c::fuchsia_800(),
        "fuchsia-900" => c::fuchsia_900(),
        "fuchsia-950" => c::fuchsia_950(),
        "pink-50" => c::pink_50(),
        "pink-100" => c::pink_100(),
        "pink-200" => c::pink_200(),
        "pink-300" => c::pink_300(),
        "pink-400" => c::pink_400(),
        "pink-500" => c::pink_500(),
        "pink-600" => c::pink_600(),
        "pink-700" => c::pink_700(),
        "pink-800" => c::pink_800(),
        "pink-900" => c::pink_900(),
        "pink-950" => c::pink_950(),
        "rose-50" => c::rose_50(),
        "rose-100" => c::rose_100(),
        "rose-200" => c::rose_200(),
        "rose-300" => c::rose_300(),
        "rose-400" => c::rose_400(),
        "rose-500" => c::rose_500(),
        "rose-600" => c::rose_600(),
        "rose-700" => c::rose_700(),
        "rose-800" => c::rose_800(),
        "rose-900" => c::rose_900(),
        "rose-950" => c::rose_950(),
        _ => return None,
    })
}

// ============================================================================
// Parser
// ============================================================================

fn parse_utility(class: &str) -> Option<Op> {
    let (neg, base) = match class.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, class),
    };
    let op = parse_base(base)?;
    if neg { op.negated() } else { Some(op) }
}

fn parse_static(c: &str) -> Option<Op> {
    Some(match c {
        "italic" => Op::FontStyle(FontSlant::Italic),
        "not-italic" => Op::FontStyle(FontSlant::Upright),
        "underline" => Op::TextDecoration(TextDecoration::Underline),
        "overline" => Op::TextDecoration(TextDecoration::Overline),
        "line-through" => Op::TextDecoration(TextDecoration::LineThrough),
        "no-underline" => Op::TextDecoration(TextDecoration::None),
        "truncate" => Op::Truncate,
        "tabular-nums" => Op::FontFeature(FontFeature::TABULAR_NUMBERS),
        "slashed-zero" => Op::FontFeature(FontFeature::SLASHED_ZERO),
        "diagonal-fractions" => Op::FontFeature(FontFeature::FRACTIONS),
        "small-caps" => Op::FontFeature(FontFeature::SMALL_CAPS),
        "flex-row" => Op::Direction(Direction::Horizontal),
        "flex-col" => Op::Direction(Direction::Vertical),
        "absolute" => Op::Absolute,
        "relative" | "static" => Op::Relative,
        "backdrop-blur" => Op::BackdropBlur(8.0),
        _ => return None,
    })
}

fn parse_base(c: &str) -> Option<Op> {
    if let Some(op) = parse_static(c) {
        return Some(op);
    }
    if let Some(op) = parse_image(c) {
        return Some(op);
    }
    if let Some(op) = border_class(c) {
        return Some(op);
    }

    if c == "rounded" {
        return rounded("");
    }
    if let Some(r) = c.strip_prefix("rounded-") {
        return rounded(r);
    }
    if let Some(v) = c.strip_prefix("text-shadow-") {
        return Some(Op::TextShadow(match v {
            "none" => ShadowSize::None,
            "2xs" => ShadowSize::S2xs,
            "xs" => ShadowSize::Xs,
            "sm" => ShadowSize::Sm,
            "md" => ShadowSize::Md,
            "lg" => ShadowSize::Lg,
            _ => return None,
        }));
    }
    if let Some(v) = c.strip_prefix("backdrop-blur-") {
        return backdrop_blur(v);
    }
    if let Some(v) = c.strip_prefix("font-stretch-") {
        return font_stretch(v);
    }
    if let Some(v) = c.strip_prefix("min-w-") {
        return Some(Op::Dim(Dim::MinWidth, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("min-h-") {
        return Some(Op::Dim(Dim::MinHeight, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("max-w-") {
        if let Some(px_v) = container_size(v) {
            return Some(Op::Dim(Dim::MaxWidth, px(px_v)));
        }
        return Some(Op::Dim(Dim::MaxWidth, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("max-h-") {
        return Some(Op::Dim(Dim::MaxHeight, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("scale-x-") {
        return percent(v).map(Op::ScaleX);
    }
    if let Some(v) = c.strip_prefix("scale-y-") {
        return percent(v).map(Op::ScaleY);
    }

    if c == "ring" {
        return Some(Op::BorderWidth(BorderEdge::All, 3.0));
    }

    let (head, v) = c.split_once('-')?;
    match head {
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" => {
            Some(Op::Padding(side(&head[1..])?, spacing(v)?))
        }
        "m" | "mx" | "my" | "mt" | "mr" | "mb" | "ml" => {
            Some(Op::Margin(side(&head[1..])?, spacing(v)?))
        }
        "w" => Some(Op::Dim(Dim::Width, size_value(v)?)),
        "h" => Some(Op::Dim(Dim::Height, size_value(v)?)),
        "size" => Some(Op::Dim(Dim::Both, size_value(v)?)),
        "gap" => spacing(v).map(Op::Gap),
        "bg" => color_value(v).map(Op::Bg),
        "text" => text(v),
        "font" => font(v),
        "tracking" => tracking(v),
        "leading" => leading(v),
        "cursor" => cursor(v),
        "z" => z_index(v),
        "overflow" => match v {
            "hidden" | "clip" => Some(Op::Overflow(Overflow::Clip)),
            "visible" => Some(Op::Overflow(Overflow::None)),
            _ => None,
        },
        "opacity" => percent(v).map(|o| Op::Opacity(o.clamp(0.0, 1.0))),
        "rotate" => {
            if v == "none" {
                Some(Op::Rotate(0.0))
            } else if let Some(n) = plain_num(v) {
                Some(Op::Rotate(n))
            } else {
                let inner = arbitrary(v)?;
                plain_num(inner.strip_suffix("deg").unwrap_or(inner)).map(Op::Rotate)
            }
        }
        "scale" => percent(v).map(Op::Scale),
        "origin" => origin(v),
        "justify" => Some(Op::MainAlign(match v {
            "start" => Alignment::Start,
            "center" => Alignment::Center,
            "end" => Alignment::End,
            "between" => Alignment::SpaceBetween,
            "around" => Alignment::SpaceAround,
            "evenly" => Alignment::SpaceEvenly,
            _ => return None,
        })),
        "items" => Some(Op::CrossAlign(match v {
            "start" => Alignment::Start,
            "center" => Alignment::Center,
            "end" => Alignment::End,
            _ => return None,
        })),
        "ring" => {
            if let Some(n) = plain_num(v).or_else(|| arbitrary_px(v)) {
                Some(Op::BorderWidth(BorderEdge::All, n))
            } else {
                color_value(v).map(Op::BorderColor)
            }
        }
        _ => None,
    }
}

fn side(s: &str) -> Option<Side> {
    Some(match s {
        "" => Side::All,
        "x" => Side::X,
        "y" => Side::Y,
        "t" => Side::T,
        "r" => Side::R,
        "b" => Side::B,
        "l" => Side::L,
        _ => return None,
    })
}

fn edge_of(s: &str) -> Option<Edge> {
    Some(match s {
        "t" => Edge::T,
        "r" => Edge::R,
        "b" => Edge::B,
        "l" => Edge::L,
        "tl" => Edge::Tl,
        "tr" => Edge::Tr,
        "br" => Edge::Br,
        "bl" => Edge::Bl,
        _ => return None,
    })
}

fn border_edge_of(s: &str) -> Option<BorderEdge> {
    Some(match s {
        "x" => BorderEdge::X,
        "y" => BorderEdge::Y,
        "t" => BorderEdge::T,
        "r" => BorderEdge::R,
        "b" => BorderEdge::B,
        "l" => BorderEdge::L,
        _ => return None,
    })
}

fn rounded(rest: &str) -> Option<Op> {
    if rest.is_empty() {
        return Some(Op::Radius(Edge::All, 4.0));
    }
    let (edge, size) = match rest.split_once('-') {
        Some((e, s)) => match edge_of(e) {
            Some(e) => (e, Some(s)),
            None => (Edge::All, Some(rest)),
        },
        None => match edge_of(rest) {
            Some(e) => (e, None),
            None => (Edge::All, Some(rest)),
        },
    };
    let radius = match size {
        None => 4.0,
        Some("none") => 0.0,
        Some("xs") => 2.0,
        Some("sm") => 4.0,
        Some("md") => 6.0,
        Some("lg") => 8.0,
        Some("xl") => 12.0,
        Some("2xl") => 16.0,
        Some("3xl") => 24.0,
        Some("4xl") => 32.0,
        Some("full") => 9999.0,
        Some(other) => arbitrary_px(other)?,
    };
    Some(Op::Radius(edge, radius))
}

fn border_class(c: &str) -> Option<Op> {
    if c == "border" {
        return Some(Op::BorderWidth(BorderEdge::All, 1.0));
    }
    if c == "border-none" {
        return Some(Op::BorderNone);
    }
    if c == "border-inner" {
        return Some(Op::BorderInner);
    }
    if c == "border-outer" {
        return Some(Op::BorderOuter);
    }
    if c == "border-center" {
        return Some(Op::BorderCenter);
    }

    let rest = c.strip_prefix("border-")?;

    if let Some((head, tail)) = rest.split_once('-') {
        if let Some(edge) = border_edge_of(head) {
            if let Some(n) = plain_num(tail).or_else(|| arbitrary_px(tail)) {
                return Some(Op::BorderWidth(edge, n));
            }
            if let Some(col) = color_value(tail) {
                return Some(Op::BorderColor(col));
            }
            return None;
        }
    }

    if let Some(n) = plain_num(rest).or_else(|| arbitrary_px(rest)) {
        return Some(Op::BorderWidth(BorderEdge::All, n));
    }
    if let Some(col) = color_value(rest) {
        return Some(Op::BorderColor(col));
    }
    if let Some(edge) = border_edge_of(rest) {
        return Some(Op::BorderWidth(edge, 1.0));
    }
    None
}

fn backdrop_blur(v: &str) -> Option<Op> {
    Some(Op::BackdropBlur(match v {
        "none" => 0.0,
        "2xs" => 2.0,
        "xs" => 4.0,
        "sm" => 8.0,
        "md" => 12.0,
        "lg" => 16.0,
        "xl" => 24.0,
        "2xl" => 40.0,
        "3xl" => 64.0,
        other => arbitrary_px(other)?,
    }))
}

fn z_index(v: &str) -> Option<Op> {
    match v {
        "auto" => return Some(Op::Layer(Layer::Relative(0))),
        "overlay" => return Some(Op::Layer(Layer::Overlay)),
        _ => {}
    }
    if let Some(level) = v.strip_prefix("overlay-") {
        let n = plain_num(level)?;
        if n.fract() != 0.0 {
            return None;
        }
        let level = u8::try_from(n as i32).ok()?;
        return Some(Op::Layer(Layer::OverlayLevel(level)));
    }
    let n = plain_num(v).or_else(|| arbitrary(v).and_then(plain_num))?;
    if n.fract() != 0.0 {
        return None;
    }
    let n_i16 = i16::try_from(n as i32).ok()?;
    Some(Op::Layer(Layer::Relative(n_i16)))
}

fn text(v: &str) -> Option<Op> {
    let size = match v {
        "xs" => Some(12.0),
        "sm" => Some(14.0),
        "base" | "md" => Some(16.0),
        "lg" => Some(18.0),
        "xl" => Some(20.0),
        "2xl" => Some(24.0),
        "3xl" => Some(30.0),
        "4xl" => Some(36.0),
        "5xl" => Some(48.0),
        "6xl" => Some(60.0),
        "7xl" => Some(72.0),
        "8xl" => Some(96.0),
        "9xl" => Some(128.0),
        _ => None,
    };
    if let Some(s) = size {
        return Some(Op::FontSize(s));
    }
    match v {
        "left" => return Some(Op::TextAlign(TextAlign::Left)),
        "right" => return Some(Op::TextAlign(TextAlign::Right)),
        "center" => return Some(Op::TextAlign(TextAlign::Center)),
        "justify" => return Some(Op::TextAlign(TextAlign::Justify)),
        "start" => return Some(Op::TextAlign(TextAlign::Start)),
        "end" => return Some(Op::TextAlign(TextAlign::End)),
        "clip" => return Some(Op::TextOverflow(TextOverflow::Clip)),
        "ellipsis" => return Some(Op::TextOverflow(TextOverflow::Ellipsis)),
        _ => {}
    }
    if let Some(px_v) = arbitrary_px(v) {
        return Some(Op::FontSize(px_v));
    }
    color_value(v).map(Op::TextColor)
}

fn font(v: &str) -> Option<Op> {
    Some(match v {
        "thin" => Op::FontWeight(FontWeight::THIN),
        "extralight" => Op::FontWeight(FontWeight::EXTRA_LIGHT),
        "light" => Op::FontWeight(FontWeight::LIGHT),
        "normal" => Op::FontWeight(FontWeight::NORMAL),
        "medium" => Op::FontWeight(FontWeight::MEDIUM),
        "semibold" => Op::FontWeight(FontWeight::SEMI_BOLD),
        "bold" => Op::FontWeight(FontWeight::BOLD),
        "extrabold" => Op::FontWeight(FontWeight::EXTRA_BOLD),
        "black" => Op::FontWeight(FontWeight::BLACK),
        "sans" => Op::FontFamily(Family::Sans),
        "serif" => Op::FontFamily(Family::Serif),
        "mono" => Op::FontFamily(Family::Mono),
        _ => return None,
    })
}

fn font_stretch(v: &str) -> Option<Op> {
    Some(Op::FontStretch(match v {
        "ultra-condensed" => FontWidth::ULTRA_CONDENSED,
        "extra-condensed" => FontWidth::EXTRA_CONDENSED,
        "condensed" => FontWidth::CONDENSED,
        "semi-condensed" => FontWidth::SEMI_CONDENSED,
        "normal" => FontWidth::NORMAL,
        "semi-expanded" => FontWidth::SEMI_EXPANDED,
        "extra-expanded" => FontWidth::EXTRA_EXPANDED,
        "ultra-expanded" => FontWidth::ULTRA_EXPANDED,
        _ => return None,
    }))
}

fn tracking(v: &str) -> Option<Op> {
    Some(Op::Tracking(match v {
        "tighter" => -0.05,
        "tight" => -0.025,
        "normal" => 0.0,
        "wide" => 0.025,
        "wider" => 0.05,
        "widest" => 0.1,
        other => {
            let inner = arbitrary(other)?.trim();
            signed_num(inner.strip_suffix("em").unwrap_or(inner))?
        }
    }))
}

fn leading(v: &str) -> Option<Op> {
    Some(Op::LineHeight(match v {
        "none" => 1.0,
        "tight" => 1.25,
        "snug" => 1.375,
        "normal" => 1.5,
        "relaxed" => 1.625,
        "loose" => 2.0,
        other => arbitrary_num(other)?,
    }))
}

fn origin(v: &str) -> Option<Op> {
    Some(Op::Origin(match v {
        "center" => TransformOrigin::center(),
        "top" => TransformOrigin::top(),
        "top-right" => TransformOrigin::top_right(),
        "right" => TransformOrigin::right(),
        "bottom-right" => TransformOrigin::bottom_right(),
        "bottom" => TransformOrigin::bottom(),
        "bottom-left" => TransformOrigin::bottom_left(),
        "left" => TransformOrigin::left(),
        "top-left" => TransformOrigin::top_left(),
        _ => return None,
    }))
}

fn cursor(v: &str) -> Option<Op> {
    if v == "none" {
        return Some(Op::CursorHidden);
    }
    Some(Op::Cursor(match v {
        "auto" | "default" => CursorIcon::Default,
        "context-menu" => CursorIcon::ContextMenu,
        "help" => CursorIcon::Help,
        "pointer" => CursorIcon::Pointer,
        "progress" => CursorIcon::Progress,
        "wait" => CursorIcon::Wait,
        "cell" => CursorIcon::Cell,
        "crosshair" => CursorIcon::Crosshair,
        "text" => CursorIcon::Text,
        "vertical-text" => CursorIcon::VerticalText,
        "alias" => CursorIcon::Alias,
        "copy" => CursorIcon::Copy,
        "move" => CursorIcon::Move,
        "no-drop" => CursorIcon::NoDrop,
        "not-allowed" => CursorIcon::NotAllowed,
        "grab" => CursorIcon::Grab,
        "grabbing" => CursorIcon::Grabbing,
        "e-resize" => CursorIcon::EResize,
        "n-resize" => CursorIcon::NResize,
        "ne-resize" => CursorIcon::NeResize,
        "nw-resize" => CursorIcon::NwResize,
        "s-resize" => CursorIcon::SResize,
        "se-resize" => CursorIcon::SeResize,
        "sw-resize" => CursorIcon::SwResize,
        "w-resize" => CursorIcon::WResize,
        "ew-resize" => CursorIcon::EwResize,
        "ns-resize" => CursorIcon::NsResize,
        "nesw-resize" => CursorIcon::NeswResize,
        "nwse-resize" => CursorIcon::NwseResize,
        "col-resize" => CursorIcon::ColResize,
        "row-resize" => CursorIcon::RowResize,
        "all-scroll" => CursorIcon::AllScroll,
        "zoom-in" => CursorIcon::ZoomIn,
        "zoom-out" => CursorIcon::ZoomOut,
        "dnd-ask" => CursorIcon::DndAsk,
        "all-resize" => CursorIcon::AllResize,
        _ => return None,
    }))
}
