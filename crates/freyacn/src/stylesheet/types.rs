//! CSS type definitions.
//!
//! Contains the `impl_from_str_or_string_for_enum!` macro and all enum /
//! struct types referenced by `StyleSheet` in `style.rs`.

use crate::stylesheet::unit_value::UnitValue;
use strum::VariantNames;
// ============================================================================
// Helper: panic + FromStr macro
// ============================================================================

fn helpful_panic(name: &str, value: &str, options: &[&str]) -> ! {
    panic!(
        "Invalid {}: {:?}, expected one of: [{}]",
        name,
        value,
        options.join(", "),
    )
}

macro_rules! impl_from_str_or_string_for_enum {
    ($enum_name:ident, $($variant:ident => $s:literal $(| $alias:literal)*),+ $(,)?) => {
        impl ::core::str::FromStr for $enum_name {
            type Err = ::std::string::String;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                match s {
                    $($s $(| $alias)* => ::core::result::Result::Ok($enum_name::$variant),)+
                    _ => ::core::result::Result::Err(::std::format!(
                        concat!("Invalid ", stringify!($enum_name), ": {:?}"), s
                    )),
                }
            }
        }

        impl ::core::convert::From<&str> for $enum_name {
            fn from(s: &str) -> Self {
                <Self as ::core::str::FromStr>::from_str(s).unwrap_or_else(|_| {
                    helpful_panic(stringify!($enum_name), s, &[$($s $(, $alias)*),+])
                })
            }
        }

        impl ::core::convert::From<::std::string::String> for $enum_name {
            fn from(s: ::std::string::String) -> Self {
                Self::from(s.as_str())
            }
        }
    };
}

// ============================================================================
// Core utility types
// ============================================================================

/// CSS global keywords, valid on every property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum GlobalKeyword {
    Inherit,
    Initial,
    Unset,
    Revert,
    RevertLayer,
}

/// A CSS `<color>` value.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgba(u8, u8, u8, u8),
    /// Named color keyword or function string (e.g. `"red"`, `"rgb(...)"`).
    Keyword(String),
    CurrentColor,
}

impl Color {
    pub const BLACK: Self = Self::Rgba(0, 0, 0, 255);
    pub const WHITE: Self = Self::Rgba(255, 255, 255, 255);
    pub const TRANSPARENT: Self = Self::Rgba(0, 0, 0, 0);
    pub const CURRENT_COLOR: Self = Self::CurrentColor;

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgba(r, g, b, 255)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::Rgba(r, g, b, a)
    }

    pub fn hex(rgb: u32) -> Self {
        Self::Rgba(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            255,
        )
    }

    fn parse_hex(hex: &str) -> Option<Self> {
        let expand = |c: char| -> Option<u8> {
            let v = c.to_digit(16)? as u8;
            Some(v * 17)
        };
        match hex.len() {
            3 => {
                let b = hex.as_bytes();
                let r = expand(b[0] as char)?;
                let g = expand(b[1] as char)?;
                let bl = expand(b[2] as char)?;
                Some(Self::Rgba(r, g, bl, 255))
            }
            4 => {
                let b = hex.as_bytes();
                let r = expand(b[0] as char)?;
                let g = expand(b[1] as char)?;
                let bl = expand(b[2] as char)?;
                let a = expand(b[3] as char)?;
                Some(Self::Rgba(r, g, bl, a))
            }
            6 => {
                let v = u32::from_str_radix(hex, 16).ok()?;
                Some(Self::hex(v))
            }
            8 => {
                let v = u32::from_str_radix(hex, 16).ok()?;
                Some(Self::Rgba(
                    ((v >> 24) & 0xff) as u8,
                    ((v >> 16) & 0xff) as u8,
                    ((v >> 8) & 0xff) as u8,
                    (v & 0xff) as u8,
                ))
            }
            _ => None,
        }
    }
}

impl core::str::FromStr for Color {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix('#') {
            if let Some(c) = Self::parse_hex(hex) {
                return Ok(c);
            }
        }
        Ok(match s {
            "currentcolor" | "currentColor" => Self::CurrentColor,
            "transparent" => Self::TRANSPARENT,
            "black" => Self::BLACK,
            "white" => Self::WHITE,
            other => Self::Keyword(other.to_string()),
        })
    }
}

/// A CSS `auto`-or-UnitValue value (width, height, margin, inset, …).
#[derive(Debug, Clone, PartialEq)]
pub enum SizeValue {
    UnitValue(UnitValue),
    Auto,
    MinContent,
    MaxContent,
    FitContent(UnitValue),
    Stretch,
    Contain,
}

impl core::str::FromStr for SizeValue {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "auto" => Self::Auto,
            "min-content" => Self::MinContent,
            "max-content" => Self::MaxContent,
            "stretch" => Self::Stretch,
            "contain" => Self::Contain,
            _ => {
                if let Some(inner) = s
                    .strip_prefix("fit-content(")
                    .and_then(|x| x.strip_suffix(')'))
                {
                    Self::FitContent(UnitValue::new(inner))
                } else {
                    Self::UnitValue(UnitValue::new(s))
                }
            }
        })
    }
}

/// A CSS `line-height` value.
#[derive(Debug, Clone, PartialEq)]
pub enum LineHeight {
    Normal,
    Number(f32),
    UnitValue(UnitValue),
}

impl core::str::FromStr for LineHeight {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "normal" {
            return Ok(Self::Normal);
        }
        if let Ok(n) = s.parse::<f32>() {
            return Ok(Self::Number(n));
        }
        Ok(Self::UnitValue(UnitValue::new(s)))
    }
}

/// A CSS `<image>` value (non-gradient subset).
#[derive(Debug, Clone, PartialEq)]
pub enum CssImage {
    None,
    Url(String),
}

pub type FontFamily = Vec<String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum HorizontalKeyword {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum VerticalKeyword {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position2D {
    X(UnitValue),
    XY(UnitValue, UnitValue),
    Keywords(HorizontalKeyword, VerticalKeyword),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundSize {
    Cover,
    Contain,
    Explicit(SizeValue, SizeValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    pub offset_x: UnitValue,
    pub offset_y: UnitValue,
    pub blur: UnitValue,
    pub spread: UnitValue,
    pub color: Color,
    pub inset: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextShadow {
    pub offset_x: UnitValue,
    pub offset_y: UnitValue,
    pub blur: UnitValue,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    Translate(UnitValue, UnitValue),
    TranslateX(UnitValue),
    TranslateY(UnitValue),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32),
    Skew(f32, f32),
    Matrix(f32, f32, f32, f32, f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterFunction {
    Blur(UnitValue),
    Brightness(f32),
    Contrast(f32),
    DropShadow(BoxShadow),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub property: String,
    pub duration: f32,
    pub timing: String,
    pub delay: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub timing: String,
    pub delay: f32,
    pub iteration_count: String,
    pub direction: String,
    pub fill_mode: String,
    pub play_state: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridLine {
    Auto,
    Line(i32),
    Span(u32),
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectRatio(pub f32, pub f32);

impl AspectRatio {
    pub const fn new(w: f32, h: f32) -> Self {
        Self(w, h)
    }
}

// ============================================================================
// Property enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Display {
    Block,
    InlineBlock,
    Inline,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Contents,
    FlowRoot,
    GridLanes,
    List,
    ListItem,
    Table,
    InlineTable,
    TableCaption,
    TableCell,
    TableColumn,
    TableColumnGroup,
    TableFooter,
    TableHeaderGroup,
    TableRow,
    TableRowGroup,
    Ruby,
    RubyBase,
    RubyText,
    RubyBaseContainer,
    RubyTextContainer,
    RunIn,
    Inherit,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BoxSizing {
    BorderBox,
    ContentBox,
    Inherit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Overflow {
    Auto,
    Visible,
    Hidden,
    Scroll,
    Clip,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Float {
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ObjectFit {
    Cover,
    Contain,
    Center,
    Fill,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FlexWrap {
    Wrap,
    Nowrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Start,
    End,
    Left,
    Right,
    Normal,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Start,
    End,
    SelfStart,
    SelfEnd,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum AlignSelf {
    Center,
    FlexEnd,
    Stretch,
    Auto,
    FlexStart,
    Baseline,
    Start,
    End,
    SelfStart,
    SelfEnd,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum AlignContent {
    Normal,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
    Start,
    End,
    Baseline,
    FirstBaseline,
    LastBaseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum JustifyItems {
    Normal,
    Start,
    End,
    Center,
    Left,
    Right,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum JustifySelf {
    Auto,
    Normal,
    Start,
    End,
    Center,
    Left,
    Right,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
    InlineStart,
    InlineEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Unit {
    Px,
    Rem,
    Em,
    #[strum(serialize = "%", serialize = "percent")]
    Percent,
    Cm,
    Mm,
    In,
    Pt,
    Pc,
    Ex,
    Ch,
    Vw,
    Vh,
    Vmin,
    Vmax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BorderStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum OutlineStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
    MatchParent,
    JustifyAll,
    Inherit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextAlignLast {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    #[strum(serialize = "100")]
    W100,
    #[strum(serialize = "200")]
    W200,
    #[strum(serialize = "300")]
    W300,
    #[strum(serialize = "400")]
    W400,
    #[strum(serialize = "500")]
    W500,
    #[strum(serialize = "600")]
    W600,
    #[strum(serialize = "700")]
    W700,
    #[strum(serialize = "800")]
    W800,
    #[strum(serialize = "900")]
    W900,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum FontKerning {
    Auto,
    Normal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Cursor {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ColResize,
    RowResize,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ZoomIn,
    ZoomOut,
    Inherit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum VerticalAlign {
    Baseline,
    Sub,
    Super,
    Top,
    TextTop,
    Middle,
    Bottom,
    TextBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
    Blink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextDecorationSkipInk {
    Auto,
    None,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextUnderlinePosition {
    Auto,
    Under,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextTransform {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
    FullWidth,
    FullSizeKana,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextRendering {
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TextOrientation {
    Mixed,
    Upright,
    Sideways,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Hyphens {
    None,
    Manual,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
    Lr,
    Rl,
    Tb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Direction {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum UnicodeBidi {
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum OverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BackgroundRepeat {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
    Space,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BackgroundAttachment {
    Scroll,
    Fixed,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BackgroundClip {
    BorderBox,
    PaddingBox,
    ContentBox,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BackgroundOrigin {
    BorderBox,
    PaddingBox,
    ContentBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BorderCollapse {
    Collapse,
    Separate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TableLayout {
    Auto,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum CaptionSide {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum EmptyCells {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ListStylePosition {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum PointerEvents {
    Auto,
    None,
    VisiblePainted,
    VisibleFill,
    VisibleStroke,
    Visible,
    Painted,
    Fill,
    Stroke,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum UserSelect {
    Auto,
    Text,
    None,
    Contain,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Resize {
    None,
    Both,
    Horizontal,
    Vertical,
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollBehavior {
    Auto,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum OverscrollBehavior {
    Auto,
    Contain,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Appearance {
    None,
    Auto,
    Base,
    BaseSelect,
    Button,
    Checkbox,
    Listbox,
    Menulist,
    MenulistButton,
    Meter,
    ProgressBar,
    PushButton,
    Radio,
    Searchfield,
    SliderHorizontal,
    SliderVertical,
    SquareButton,
    Textarea,
    Textfield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BreakBefore {
    Auto,
    Avoid,
    Always,
    All,
    AvoidPage,
    Page,
    Left,
    Right,
    Recto,
    Verso,
    AvoidColumn,
    Column,
    AvoidRegion,
    Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BreakAfter {
    Auto,
    Avoid,
    Always,
    All,
    AvoidPage,
    Page,
    Left,
    Right,
    Recto,
    Verso,
    AvoidColumn,
    Column,
    AvoidRegion,
    Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BreakInside {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TransformStyle {
    Flat,
    Preserve3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum BackfaceVisibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum TransformBox {
    ContentBox,
    BorderBox,
    FillBox,
    StrokeBox,
    ViewBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Isolation {
    Auto,
    Isolate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Contain {
    None,
    Strict,
    Content,
    Size,
    InlineSize,
    Layout,
    Style,
    Paint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ImageRendering {
    Auto,
    CrispEdges,
    Pixelated,
    Smooth,
    HighQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ColorScheme {
    Normal,
    Light,
    Dark,
    LightDark,
    OnlyLight,
    OnlyDark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollbarWidth {
    Auto,
    Thin,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollbarGutter {
    Auto,
    Stable,
    StableBothEdges,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollSnapTypeAxis {
    None,
    X,
    Y,
    Block,
    Inline,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollSnapStrictness {
    Proximity,
    Mandatory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollSnapAlign {
    None,
    Start,
    End,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ScrollSnapStop {
    Normal,
    Always,
}

// ============================================================================
// FromStr / From<&str> / From<String> impls
// ============================================================================

impl_from_str_or_string_for_enum!(
    GlobalKeyword,
    Inherit => "inherit",
    Initial => "initial",
    Unset => "unset",
    Revert => "revert",
    RevertLayer => "revert-layer",
);

impl_from_str_or_string_for_enum!(
    HorizontalKeyword,
    Left => "left",
    Center => "center",
    Right => "right",
);

impl_from_str_or_string_for_enum!(
    VerticalKeyword,
    Top => "top",
    Center => "center",
    Bottom => "bottom",
);

impl_from_str_or_string_for_enum!(
    Display,
    Block => "block",
    InlineBlock => "inline-block",
    Inline => "inline",
    Flex => "flex",
    InlineFlex => "inline-flex",
    Grid => "grid",
    InlineGrid => "inline-grid",
    Contents => "contents",
    FlowRoot => "flow-root",
    GridLanes => "grid-lanes",
    List => "list",
    ListItem => "list-item",
    Table => "table",
    InlineTable => "inline-table",
    TableCaption => "table-caption",
    TableCell => "table-cell",
    TableColumn => "table-column",
    TableColumnGroup => "table-column-group",
    TableFooter => "table-footer",
    TableHeaderGroup => "table-header-group",
    TableRow => "table-row",
    TableRowGroup => "table-row-group",
    Ruby => "ruby",
    RubyBase => "ruby-base",
    RubyText => "ruby-text",
    RubyBaseContainer => "ruby-base-container",
    RubyTextContainer => "ruby-text-container",
    RunIn => "run-in",
    Inherit => "inherit",
    None => "none",
);

impl_from_str_or_string_for_enum!(
    BoxSizing,
    BorderBox => "border-box",
    ContentBox => "content-box",
    Inherit => "inherit",
);

impl_from_str_or_string_for_enum!(
    Position,
    Static => "static",
    Relative => "relative",
    Absolute => "absolute",
    Fixed => "fixed",
    Sticky => "sticky",
);

impl_from_str_or_string_for_enum!(
    Overflow,
    Auto => "auto",
    Visible => "visible",
    Hidden => "hidden",
    Scroll => "scroll",
    Clip => "clip",
    Overlay => "overlay",
);

impl_from_str_or_string_for_enum!(
    Float,
    Left => "left",
    Right => "right",
    None => "none",
);

impl_from_str_or_string_for_enum!(
    ObjectFit,
    Cover => "cover",
    Contain => "contain",
    Center => "center",
    Fill => "fill",
    None => "none",
    ScaleDown => "scale-down",
);

impl_from_str_or_string_for_enum!(
    Visibility,
    Visible => "visible",
    Hidden => "hidden",
    Collapse => "collapse",
);

impl_from_str_or_string_for_enum!(
    FlexDirection,
    Row => "row",
    Column => "column",
    RowReverse => "row-reverse",
    ColumnReverse => "column-reverse",
);

impl_from_str_or_string_for_enum!(
    FlexWrap,
    Wrap => "wrap",
    Nowrap => "nowrap",
    WrapReverse => "wrap-reverse",
);

impl_from_str_or_string_for_enum!(
    JustifyContent,
    FlexStart => "flex-start",
    FlexEnd => "flex-end",
    Center => "center",
    SpaceBetween => "space-between",
    SpaceAround => "space-around",
    SpaceEvenly => "space-evenly",
    Start => "start",
    End => "end",
    Left => "left",
    Right => "right",
    Normal => "normal",
    Stretch => "stretch",
);

impl_from_str_or_string_for_enum!(
    AlignItems,
    FlexStart => "flex-start",
    FlexEnd => "flex-end",
    Center => "center",
    Baseline => "baseline",
    Stretch => "stretch",
    Start => "start",
    End => "end",
    SelfStart => "self-start",
    SelfEnd => "self-end",
    Normal => "normal",
);

impl_from_str_or_string_for_enum!(
    AlignSelf,
    Center => "center",
    FlexEnd => "flex-end",
    Stretch => "stretch",
    Auto => "auto",
    FlexStart => "flex-start",
    Baseline => "baseline",
    Start => "start",
    End => "end",
    SelfStart => "self-start",
    SelfEnd => "self-end",
    Normal => "normal",
);

impl_from_str_or_string_for_enum!(
    AlignContent,
    Normal => "normal",
    FlexStart => "flex-start",
    FlexEnd => "flex-end",
    Center => "center",
    SpaceBetween => "space-between",
    SpaceAround => "space-around",
    SpaceEvenly => "space-evenly",
    Stretch => "stretch",
    Start => "start",
    End => "end",
    Baseline => "baseline",
    FirstBaseline => "first-baseline",
    LastBaseline => "last-baseline",
);

impl_from_str_or_string_for_enum!(
    JustifyItems,
    Normal => "normal",
    Start => "start",
    End => "end",
    Center => "center",
    Left => "left",
    Right => "right",
    Baseline => "baseline",
    Stretch => "stretch",
);

impl_from_str_or_string_for_enum!(
    JustifySelf,
    Auto => "auto",
    Normal => "normal",
    Start => "start",
    End => "end",
    Center => "center",
    Left => "left",
    Right => "right",
    Baseline => "baseline",
    Stretch => "stretch",
);

impl_from_str_or_string_for_enum!(
    GridAutoFlow,
    Row => "row",
    Column => "column",
    RowDense => "row-dense",
    ColumnDense => "column-dense",
);

impl_from_str_or_string_for_enum!(
    Clear,
    None => "none",
    Left => "left",
    Right => "right",
    Both => "both",
    InlineStart => "inline-start",
    InlineEnd => "inline-end",
);

impl_from_str_or_string_for_enum!(
    Unit,
    Px => "px",
    Rem => "rem",
    Em => "em",
    Percent => "%" | "percent",
    Cm => "cm",
    Mm => "mm",
    In => "in",
    Pt => "pt",
    Pc => "pc",
    Ex => "ex",
    Ch => "ch",
    Vw => "vw",
    Vh => "vh",
    Vmin => "vmin",
    Vmax => "vmax",
);

impl_from_str_or_string_for_enum!(
    BorderStyle,
    None => "none",
    Hidden => "hidden",
    Dotted => "dotted",
    Dashed => "dashed",
    Solid => "solid",
    Double => "double",
    Groove => "groove",
    Ridge => "ridge",
    Inset => "inset",
    Outset => "outset",
);

impl_from_str_or_string_for_enum!(
    OutlineStyle,
    None => "none",
    Hidden => "hidden",
    Dotted => "dotted",
    Dashed => "dashed",
    Solid => "solid",
    Double => "double",
    Groove => "groove",
    Ridge => "ridge",
    Inset => "inset",
    Outset => "outset",
);

impl_from_str_or_string_for_enum!(
    TextAlign,
    Left => "left",
    Right => "right",
    Center => "center",
    Justify => "justify",
    Start => "start",
    End => "end",
    MatchParent => "match-parent",
    JustifyAll => "justify-all",
    Inherit => "inherit",
);

impl_from_str_or_string_for_enum!(
    TextAlignLast,
    Auto => "auto",
    Start => "start",
    End => "end",
    Left => "left",
    Right => "right",
    Center => "center",
    Justify => "justify",
);

impl_from_str_or_string_for_enum!(
    FontWeight,
    Normal => "normal",
    Bold => "bold",
    Bolder => "bolder",
    Lighter => "lighter",
    W100 => "100",
    W200 => "200",
    W300 => "300",
    W400 => "400",
    W500 => "500",
    W600 => "600",
    W700 => "700",
    W800 => "800",
    W900 => "900",
);

impl_from_str_or_string_for_enum!(
    FontStyle,
    Normal => "normal",
    Italic => "italic",
    Oblique => "oblique",
);

impl_from_str_or_string_for_enum!(
    FontStretch,
    UltraCondensed => "ultra-condensed",
    ExtraCondensed => "extra-condensed",
    Condensed => "condensed",
    SemiCondensed => "semi-condensed",
    Normal => "normal",
    SemiExpanded => "semi-expanded",
    Expanded => "expanded",
    ExtraExpanded => "extra-expanded",
    UltraExpanded => "ultra-expanded",
);

impl_from_str_or_string_for_enum!(
    FontKerning,
    Auto => "auto",
    Normal => "normal",
    None => "none",
);

impl_from_str_or_string_for_enum!(
    Cursor,
    Auto => "auto",
    Default => "default",
    None => "none",
    ContextMenu => "context-menu",
    Help => "help",
    Pointer => "pointer",
    Progress => "progress",
    Wait => "wait",
    Cell => "cell",
    Crosshair => "crosshair",
    Text => "text",
    VerticalText => "vertical-text",
    Alias => "alias",
    Copy => "copy",
    Move => "move",
    NoDrop => "no-drop",
    NotAllowed => "not-allowed",
    Grab => "grab",
    Grabbing => "grabbing",
    AllScroll => "all-scroll",
    ColResize => "col-resize",
    RowResize => "row-resize",
    EResize => "e-resize",
    NResize => "n-resize",
    NeResize => "ne-resize",
    NwResize => "nw-resize",
    SResize => "s-resize",
    SeResize => "se-resize",
    SwResize => "sw-resize",
    WResize => "w-resize",
    EwResize => "ew-resize",
    NsResize => "ns-resize",
    NeswResize => "nesw-resize",
    NwseResize => "nwse-resize",
    ZoomIn => "zoom-in",
    ZoomOut => "zoom-out",
    Inherit => "inherit",
);

impl_from_str_or_string_for_enum!(
    VerticalAlign,
    Baseline => "baseline",
    Sub => "sub",
    Super => "super",
    Top => "top",
    TextTop => "text-top",
    Middle => "middle",
    Bottom => "bottom",
    TextBottom => "text-bottom",
);

impl_from_str_or_string_for_enum!(
    TextDecoration,
    None => "none",
    Underline => "underline",
    Overline => "overline",
    LineThrough => "line-through",
    Blink => "blink",
);

impl_from_str_or_string_for_enum!(
    TextDecorationStyle,
    Solid => "solid",
    Double => "double",
    Dotted => "dotted",
    Dashed => "dashed",
    Wavy => "wavy",
);

impl_from_str_or_string_for_enum!(
    TextDecorationSkipInk,
    Auto => "auto",
    None => "none",
    All => "all",
);

impl_from_str_or_string_for_enum!(
    TextUnderlinePosition,
    Auto => "auto",
    Under => "under",
    Left => "left",
    Right => "right",
);

impl_from_str_or_string_for_enum!(
    TextTransform,
    None => "none",
    Capitalize => "capitalize",
    Uppercase => "uppercase",
    Lowercase => "lowercase",
    FullWidth => "full-width",
    FullSizeKana => "full-size-kana",
);

impl_from_str_or_string_for_enum!(
    TextOverflow,
    Clip => "clip",
    Ellipsis => "ellipsis",
);

impl_from_str_or_string_for_enum!(
    TextRendering,
    Auto => "auto",
    OptimizeSpeed => "optimize-speed",
    OptimizeLegibility => "optimize-legibility",
    GeometricPrecision => "geometric-precision",
);

impl_from_str_or_string_for_enum!(
    TextOrientation,
    Mixed => "mixed",
    Upright => "upright",
    Sideways => "sideways",
);

impl_from_str_or_string_for_enum!(
    WordBreak,
    Normal => "normal",
    BreakAll => "break-all",
    KeepAll => "keep-all",
    BreakWord => "break-word",
);

impl_from_str_or_string_for_enum!(
    Hyphens,
    None => "none",
    Manual => "manual",
    Auto => "auto",
);

impl_from_str_or_string_for_enum!(
    WritingMode,
    HorizontalTb => "horizontal-tb",
    VerticalRl => "vertical-rl",
    VerticalLr => "vertical-lr",
    SidewaysRl => "sideways-rl",
    SidewaysLr => "sideways-lr",
    Lr => "lr",
    Rl => "rl",
    Tb => "tb",
);

impl_from_str_or_string_for_enum!(
    Direction,
    Ltr => "ltr",
    Rtl => "rtl",
);

impl_from_str_or_string_for_enum!(
    UnicodeBidi,
    Normal => "normal",
    Embed => "embed",
    Isolate => "isolate",
    BidiOverride => "bidi-override",
    IsolateOverride => "isolate-override",
    Plaintext => "plaintext",
);

impl_from_str_or_string_for_enum!(
    OverflowWrap,
    Normal => "normal",
    BreakWord => "break-word",
    Anywhere => "anywhere",
);

impl_from_str_or_string_for_enum!(
    BackgroundRepeat,
    Repeat => "repeat",
    RepeatX => "repeat-x",
    RepeatY => "repeat-y",
    NoRepeat => "no-repeat",
    Space => "space",
    Round => "round",
);

impl_from_str_or_string_for_enum!(
    BackgroundAttachment,
    Scroll => "scroll",
    Fixed => "fixed",
    Local => "local",
);

impl_from_str_or_string_for_enum!(
    BackgroundClip,
    BorderBox => "border-box",
    PaddingBox => "padding-box",
    ContentBox => "content-box",
    Text => "text",
);

impl_from_str_or_string_for_enum!(
    BackgroundOrigin,
    BorderBox => "border-box",
    PaddingBox => "padding-box",
    ContentBox => "content-box",
);

impl_from_str_or_string_for_enum!(
    BlendMode,
    Normal => "normal",
    Multiply => "multiply",
    Screen => "screen",
    Overlay => "overlay",
    Darken => "darken",
    Lighten => "lighten",
    ColorDodge => "color-dodge",
    ColorBurn => "color-burn",
    HardLight => "hard-light",
    SoftLight => "soft-light",
    Difference => "difference",
    Exclusion => "exclusion",
    Hue => "hue",
    Saturation => "saturation",
    Color => "color",
    Luminosity => "luminosity",
);

impl_from_str_or_string_for_enum!(
    BorderCollapse,
    Collapse => "collapse",
    Separate => "separate",
);

impl_from_str_or_string_for_enum!(
    TableLayout,
    Auto => "auto",
    Fixed => "fixed",
);

impl_from_str_or_string_for_enum!(
    CaptionSide,
    Top => "top",
    Bottom => "bottom",
);

impl_from_str_or_string_for_enum!(
    EmptyCells,
    Show => "show",
    Hide => "hide",
);

impl_from_str_or_string_for_enum!(
    WhiteSpace,
    Normal => "normal",
    Nowrap => "nowrap",
    Pre => "pre",
    PreWrap => "pre-wrap",
    PreLine => "pre-line",
    BreakSpaces => "break-spaces",
);

impl_from_str_or_string_for_enum!(
    ListStylePosition,
    Inside => "inside",
    Outside => "outside",
);

impl_from_str_or_string_for_enum!(
    PointerEvents,
    Auto => "auto",
    None => "none",
    VisiblePainted => "visible-painted",
    VisibleFill => "visible-fill",
    VisibleStroke => "visible-stroke",
    Visible => "visible",
    Painted => "painted",
    Fill => "fill",
    Stroke => "stroke",
    All => "all",
);

impl_from_str_or_string_for_enum!(
    UserSelect,
    Auto => "auto",
    Text => "text",
    None => "none",
    Contain => "contain",
    All => "all",
);

impl_from_str_or_string_for_enum!(
    Resize,
    None => "none",
    Both => "both",
    Horizontal => "horizontal",
    Vertical => "vertical",
    Block => "block",
    Inline => "inline",
);

impl_from_str_or_string_for_enum!(
    ScrollBehavior,
    Auto => "auto",
    Smooth => "smooth",
);

impl_from_str_or_string_for_enum!(
    OverscrollBehavior,
    Auto => "auto",
    Contain => "contain",
    None => "none",
);

impl_from_str_or_string_for_enum!(
    Appearance,
    None => "none",
    Auto => "auto",
    Base => "base",
    BaseSelect => "base-select",
    Button => "button",
    Checkbox => "checkbox",
    Listbox => "listbox",
    Menulist => "menulist",
    MenulistButton => "menulist-button",
    Meter => "meter",
    ProgressBar => "progress-bar",
    PushButton => "push-button",
    Radio => "radio",
    Searchfield => "searchfield",
    SliderHorizontal => "slider-horizontal",
    SliderVertical => "slider-vertical",
    SquareButton => "square-button",
    Textarea => "textarea",
    Textfield => "textfield",
);

impl_from_str_or_string_for_enum!(
    BreakBefore,
    Auto => "auto",
    Avoid => "avoid",
    Always => "always",
    All => "all",
    AvoidPage => "avoid-page",
    Page => "page",
    Left => "left",
    Right => "right",
    Recto => "recto",
    Verso => "verso",
    AvoidColumn => "avoid-column",
    Column => "column",
    AvoidRegion => "avoid-region",
    Region => "region",
);

impl_from_str_or_string_for_enum!(
    BreakAfter,
    Auto => "auto",
    Avoid => "avoid",
    Always => "always",
    All => "all",
    AvoidPage => "avoid-page",
    Page => "page",
    Left => "left",
    Right => "right",
    Recto => "recto",
    Verso => "verso",
    AvoidColumn => "avoid-column",
    Column => "column",
    AvoidRegion => "avoid-region",
    Region => "region",
);

impl_from_str_or_string_for_enum!(
    BreakInside,
    Auto => "auto",
    Avoid => "avoid",
    AvoidPage => "avoid-page",
    AvoidColumn => "avoid-column",
    AvoidRegion => "avoid-region",
);

impl_from_str_or_string_for_enum!(
    TransformStyle,
    Flat => "flat",
    Preserve3d => "preserve-3d",
);

impl_from_str_or_string_for_enum!(
    BackfaceVisibility,
    Visible => "visible",
    Hidden => "hidden",
);

impl_from_str_or_string_for_enum!(
    TransformBox,
    ContentBox => "content-box",
    BorderBox => "border-box",
    FillBox => "fill-box",
    StrokeBox => "stroke-box",
    ViewBox => "view-box",
);

impl_from_str_or_string_for_enum!(
    Isolation,
    Auto => "auto",
    Isolate => "isolate",
);

impl_from_str_or_string_for_enum!(
    Contain,
    None => "none",
    Strict => "strict",
    Content => "content",
    Size => "size",
    InlineSize => "inline-size",
    Layout => "layout",
    Style => "style",
    Paint => "paint",
);

impl_from_str_or_string_for_enum!(
    ImageRendering,
    Auto => "auto",
    CrispEdges => "crisp-edges",
    Pixelated => "pixelated",
    Smooth => "smooth",
    HighQuality => "high-quality",
);

impl_from_str_or_string_for_enum!(
    ColorScheme,
    Normal => "normal",
    Light => "light",
    Dark => "dark",
    LightDark => "light-dark",
    OnlyLight => "only-light",
    OnlyDark => "only-dark",
);

impl_from_str_or_string_for_enum!(
    ScrollbarWidth,
    Auto => "auto",
    Thin => "thin",
    None => "none",
);

impl_from_str_or_string_for_enum!(
    ScrollbarGutter,
    Auto => "auto",
    Stable => "stable",
    StableBothEdges => "stable-both-edges",
);

impl_from_str_or_string_for_enum!(
    ScrollSnapTypeAxis,
    None => "none",
    X => "x",
    Y => "y",
    Block => "block",
    Inline => "inline",
    Both => "both",
);

impl_from_str_or_string_for_enum!(
    ScrollSnapStrictness,
    Proximity => "proximity",
    Mandatory => "mandatory",
);

impl_from_str_or_string_for_enum!(
    ScrollSnapAlign,
    None => "none",
    Start => "start",
    End => "end",
    Center => "center",
);

impl_from_str_or_string_for_enum!(
    ScrollSnapStop,
    Normal => "normal",
    Always => "always",
);

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_str() {
        let _: Display = "block".into();
        let _: Display = "inline-block".into();
        let _: Overflow = "clip".into();
        let _: ObjectFit = "scale-down".into();
    }

    #[test]
    fn UnitValue_unit_aliases() {
        assert_eq!("%".parse::<Unit>().unwrap(), Unit::Percent);
        assert_eq!("percent".parse::<Unit>().unwrap(), Unit::Percent);
        assert_eq!("px".parse::<Unit>().unwrap(), Unit::Px);
    }

    #[test]
    fn from_str_works() {
        assert!(matches!("grid".parse::<Display>(), Ok(Display::Grid)));
        assert!("bogus".parse::<Display>().is_err());
    }

    #[test]
    #[should_panic(expected = "Invalid Display")]
    fn bad_input_panics() {
        let _: Display = "bogus".into();
    }

    #[test]
    fn color_hex_parses() {
        assert_eq!("#ff0000".parse::<Color>().unwrap(), Color::rgb(255, 0, 0));
        assert_eq!("#00ff00".parse::<Color>().unwrap(), Color::rgb(0, 255, 0));
        assert_eq!("#0000ff".parse::<Color>().unwrap(), Color::rgb(0, 0, 255));
    }

    #[test]
    fn color_named_parses() {
        assert_eq!("black".parse::<Color>().unwrap(), Color::BLACK);
        assert_eq!("white".parse::<Color>().unwrap(), Color::WHITE);
        assert_eq!("transparent".parse::<Color>().unwrap(), Color::TRANSPARENT);
        assert_eq!(
            "rebeccapurple".parse::<Color>().unwrap(),
            Color::Keyword("rebeccapurple".into())
        );
    }

    #[test]
    fn size_value_parses() {
        assert!(matches!("auto".parse::<SizeValue>(), Ok(SizeValue::Auto)));
        assert!(matches!(
            "min-content".parse::<SizeValue>(),
            Ok(SizeValue::MinContent)
        ));
        assert!(matches!(
            "10px".parse::<SizeValue>(),
            Ok(SizeValue::UnitValue(_))
        ));
    }

    #[test]
    fn line_height_parses() {
        assert_eq!("normal".parse::<LineHeight>().unwrap(), LineHeight::Normal);
        assert_eq!(
            "1.5".parse::<LineHeight>().unwrap(),
            LineHeight::Number(1.5)
        );
        assert!(matches!(
            "20px".parse::<LineHeight>(),
            Ok(LineHeight::UnitValue(_))
        ));
    }

    /// Round-trip every `VariantNames` string for every enum below.
    macro_rules! roundtrip_tests {
        ($($name:ident: $ty:ident),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    for v in $ty::VARIANTS {
                        assert!(
                            v.parse::<$ty>().is_ok(),
                            "{}::VARIANTS contains unparseable {:?}",
                            stringify!($ty),
                            v,
                        );
                    }
                }
            )*
        };
    }

    roundtrip_tests!(
        rt_global_keyword: GlobalKeyword,
        rt_horizontal_keyword: HorizontalKeyword,
        rt_vertical_keyword: VerticalKeyword,
        rt_display: Display,
        rt_box_sizing: BoxSizing,
        rt_position: Position,
        rt_overflow: Overflow,
        rt_float: Float,
        rt_object_fit: ObjectFit,
        rt_visibility: Visibility,
        rt_flex_direction: FlexDirection,
        rt_flex_wrap: FlexWrap,
        rt_justify_content: JustifyContent,
        rt_align_items: AlignItems,
        rt_align_self: AlignSelf,
        rt_align_content: AlignContent,
        rt_justify_items: JustifyItems,
        rt_justify_self: JustifySelf,
        rt_grid_auto_flow: GridAutoFlow,
        rt_clear: Clear,
        rt_UnitValue_unit: Unit,
        rt_border_style: BorderStyle,
        rt_outline_style: OutlineStyle,
        rt_text_align: TextAlign,
        rt_text_align_last: TextAlignLast,
        rt_font_weight: FontWeight,
        rt_font_style: FontStyle,
        rt_font_stretch: FontStretch,
        rt_font_kerning: FontKerning,
        rt_cursor: Cursor,
        rt_vertical_align: VerticalAlign,
        rt_text_decoration: TextDecoration,
        rt_text_decoration_style: TextDecorationStyle,
        rt_text_decoration_skip_ink: TextDecorationSkipInk,
        rt_text_underline_position: TextUnderlinePosition,
        rt_text_transform: TextTransform,
        rt_text_overflow: TextOverflow,
        rt_text_rendering: TextRendering,
        rt_text_orientation: TextOrientation,
        rt_word_break: WordBreak,
        rt_hyphens: Hyphens,
        rt_writing_mode: WritingMode,
        rt_direction: Direction,
        rt_unicode_bidi: UnicodeBidi,
        rt_overflow_wrap: OverflowWrap,
        rt_background_repeat: BackgroundRepeat,
        rt_background_attachment: BackgroundAttachment,
        rt_background_clip: BackgroundClip,
        rt_background_origin: BackgroundOrigin,
        rt_blend_mode: BlendMode,
        rt_border_collapse: BorderCollapse,
        rt_table_layout: TableLayout,
        rt_caption_side: CaptionSide,
        rt_empty_cells: EmptyCells,
        rt_white_space: WhiteSpace,
        rt_list_style_position: ListStylePosition,
        rt_pointer_events: PointerEvents,
        rt_user_select: UserSelect,
        rt_resize: Resize,
        rt_scroll_behavior: ScrollBehavior,
        rt_overscroll_behavior: OverscrollBehavior,
        rt_appearance: Appearance,
        rt_break_before: BreakBefore,
        rt_break_after: BreakAfter,
        rt_break_inside: BreakInside,
        rt_transform_style: TransformStyle,
        rt_backface_visibility: BackfaceVisibility,
        rt_transform_box: TransformBox,
        rt_isolation: Isolation,
        rt_contain: Contain,
        rt_image_rendering: ImageRendering,
        rt_color_scheme: ColorScheme,
        rt_scrollbar_width: ScrollbarWidth,
        rt_scrollbar_gutter: ScrollbarGutter,
        rt_scroll_snap_type_axis: ScrollSnapTypeAxis,
        rt_scroll_snap_strictness: ScrollSnapStrictness,
        rt_scroll_snap_align: ScrollSnapAlign,
        rt_scroll_snap_stop: ScrollSnapStop,
    );
}
