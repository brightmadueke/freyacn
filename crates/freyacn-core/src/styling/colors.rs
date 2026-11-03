//! All FreyaCN palette colors as functions returning [`StyledColor`].
//!
//! `StyledColor` is an enhanced [`Fill`]: it wraps any `Fill` variant
//! (solid color, `LinearGradient`, `RadialGradient`, `ConicGradient`,
//! `ShaderFill`, or `RenderCallback`) and adds a global `.opacity(..)`
//! multiplier that is applied to solid colors on conversion.
//!
//! ```
//! // Solid color, fully opaque
//! let a: Fill = blue_500().into();
//!
//! // Solid color, half-transparent
//! let b: Fill = blue_500().opacity(0.5).into();
//!
//! // Gradient — opacity is ignored (set per-stop alphas instead)
//! let g: Fill = StyledColor::from(
//!     LinearGradient::new()
//!         .stop((blue_500().into(), 0.0))
//!         .stop((purple_500().into(), 100.0)),
//! ).into();
//!
//! // Gradient, one stop dimmed manually
//! let f: Fill = StyledColor::from(
//!     LinearGradient::new()
//!         .stop((blue_500().opacity(0.2).to_color().unwrap(), 0.0))
//!         .stop((purple_500().to_color().unwrap(), 100.0)),
//! ).into();
//! ```

use std::hash::{Hash, Hasher};

use freya::prelude::*;
use paste::paste;

// ---------------------------------------------------------------------
// StyledColor
// ---------------------------------------------------------------------

/// An enhanced [`Fill`] with a global opacity multiplier.
///
/// - Solid colors (`Fill::Color`): opacity is applied to the alpha on
///   conversion via [`StyledColor::to_fill`] / [`StyledColor::to_color`].
/// - Gradients, shaders, and callbacks: opacity is **ignored** — those
///   fills carry their own per-stop colors, so use `.opacity(..)` on
///   the individual stops instead.
///
/// `StyledColor` is **not `Copy`** (because `Fill` isn't, since it holds
/// `Box`es for gradients/shaders). Clone if you need to duplicate it.
#[derive(Clone, Debug, PartialEq)]
pub struct StyledColor {
    fill: Fill,
    opacity: f32,
}

/// Clamp a `f32` to `[0.0, 1.0]`, treating `NaN` as `0.0`.
///
/// `const`-compatible: uses the `a != a` NaN test (which is a genuine
/// inequality only for `NaN`) so it works in `const fn` bodies without
/// relying on `f32::is_nan` being `const` on every supported compiler.
#[inline]
const fn clamp_unit(a: f32) -> f32 {
    if a != a {
        // NaN
        0.0
    } else if a < 0.0 {
        0.0
    } else if a > 1.0 {
        1.0
    } else {
        a
    }
}

impl StyledColor {
    // ---- Constructors ----

    /// Fully opaque solid color from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            fill: Fill::Color(Color::from_rgb(r, g, b)),
            opacity: 1.0,
        }
    }

    /// Solid colour from separate `(a, r, g, b)` byte components, in
    /// that order. Equivalent to `rgba(r, g, b, a as f32 / 255.0)`.
    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            fill: Fill::Color(Color::from_rgb(r, g, b)),
            opacity: a as f32 / 255.0,
        }
    }

    /// Solid color with an explicit `0.0..=1.0` opacity. Values outside
    /// the range (and `NaN`) are clamped to `[0.0, 1.0]` (`NaN` → `0.0`).
    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            fill: Fill::Color(Color::from_rgb(r, g, b)),
            opacity: clamp_unit(a),
        }
    }

    /// Wrap an arbitrary [`Fill`] with no extra opacity.
    pub fn from_fill(fill: Fill) -> Self {
        Self { fill, opacity: 1.0 }
    }

    // ---- Accessors ----

    /// Borrow the underlying [`Fill`] (without applying opacity).
    pub fn fill(&self) -> &Fill {
        &self.fill
    }

    /// Returns the current opacity multiplier.
    pub fn current_opacity(&self) -> f32 {
        self.opacity
    }

    /// `true` when this wraps a solid color (`Fill::Color`).
    pub fn is_solid(&self) -> bool {
        matches!(self.fill, Fill::Color(_))
    }

    /// Resolve to a solid [`Color`] with opacity applied, or `None` if
    /// this is a gradient/shader/callback fill.
    pub fn to_color(&self) -> Option<Color> {
        match &self.fill {
            Fill::Color(c) => {
                let alpha = ((c.a() as f32) * self.opacity).round() as u8;
                Some(c.with_a(alpha))
            }
            _ => None,
        }
    }

    // ---- Modifiers ----

    /// Override the opacity multiplier (`0.0` transparent, `1.0` opaque).
    /// Ignored for non-solid fills. Values outside the range (and `NaN`)
    /// are clamped to `[0.0, 1.0]` (`NaN` → `0.0`).
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = clamp_unit(opacity);
        self
    }

    /// Multiply the existing opacity by `factor`. Ignored for non-solid
    /// fills.
    ///
    /// `factor` is clamped to `[0.0, 1.0]` **before** multiplication, so
    /// this method can only reduce (or leave unchanged) the current
    /// opacity — it cannot brighten past the existing value. `NaN` is
    /// treated as `0.0` (fully transparent).
    pub const fn with_opacity(mut self, factor: f32) -> Self {
        self.opacity *= clamp_unit(factor);
        self
    }

    // ---- Conversion ----

    /// Convert to a Freya [`Fill`], applying the stored opacity to
    /// solid colors. Non-solid fills are returned unchanged.
    pub fn to_fill(self) -> Fill {
        match self.fill {
            Fill::Color(c) => {
                let alpha = ((c.a() as f32) * self.opacity).round() as u8;
                Fill::Color(c.with_a(alpha))
            }
            other => other,
        }
    }
}

impl Default for StyledColor {
    fn default() -> Self {
        Self {
            fill: Fill::default(),
            opacity: 1.0,
        }
    }
}

impl Hash for StyledColor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fill.hash(state);
        // `f32` isn't `Hash`; hash the bit pattern instead.
        self.opacity.to_bits().hash(state);
    }
}

// ---- Conversions ----

impl From<StyledColor> for Fill {
    fn from(c: StyledColor) -> Fill {
        c.to_fill()
    }
}

impl From<StyledColor> for Color {
    /// Extract the solid color, applying opacity.
    ///
    /// # Panics
    /// Panics if the `StyledColor` wraps a gradient, shader, or callback
    /// fill. Use [`StyledColor::to_color`] for a non-panicking variant.
    fn from(c: StyledColor) -> Color {
        c.to_color()
            .expect("StyledColor does not wrap a solid Color")
    }
}

impl From<Color> for StyledColor {
    fn from(c: Color) -> Self {
        Self {
            fill: Fill::Color(c),
            opacity: 1.0,
        }
    }
}

impl From<Fill> for StyledColor {
    fn from(fill: Fill) -> Self {
        Self { fill, opacity: 1.0 }
    }
}

impl From<LinearGradient> for StyledColor {
    fn from(g: LinearGradient) -> Self {
        Self {
            fill: Fill::from(g),
            opacity: 1.0,
        }
    }
}

impl From<RadialGradient> for StyledColor {
    fn from(g: RadialGradient) -> Self {
        Self {
            fill: Fill::from(g),
            opacity: 1.0,
        }
    }
}

impl From<ConicGradient> for StyledColor {
    fn from(g: ConicGradient) -> Self {
        Self {
            fill: Fill::from(g),
            opacity: 1.0,
        }
    }
}

impl From<ShaderFill> for StyledColor {
    fn from(s: ShaderFill) -> Self {
        Self {
            fill: Fill::from(s),
            opacity: 1.0,
        }
    }
}

impl From<RenderCallback> for StyledColor {
    fn from(cb: RenderCallback) -> Self {
        Self {
            fill: Fill::from(cb),
            opacity: 1.0,
        }
    }
}

// Tuple constructors — mirror `impl From<(u8, u8, u8)> for Fill`
impl From<(u8, u8, u8)> for StyledColor {
    fn from(c: (u8, u8, u8)) -> Self {
        Self::rgb(c.0, c.1, c.2)
    }
}
impl From<(u8, u8, u8, f32)> for StyledColor {
    fn from(c: (u8, u8, u8, f32)) -> Self {
        Self::rgba(c.0, c.1, c.2, c.3)
    }
}
impl From<(u8, u8, u8, u8)> for StyledColor {
    fn from(c: (u8, u8, u8, u8)) -> Self {
        Self::argb(c.3, c.0, c.1, c.2)
    }
}

// ---------------------------------------------------------------------
// Palette functions (all return StyledColor)
// ---------------------------------------------------------------------

macro_rules! family {
    ($fam:ident; $($shade:literal: ($r:expr, $g:expr, $b:expr)),* $(,)?) => {
        paste! {
            $(
                #[inline]
                pub const fn [<$fam _ $shade>]() -> StyledColor {
                    StyledColor::rgb($r, $g, $b)
                }
            )*
        }
    };
}

pub const fn white() -> StyledColor {
    StyledColor::rgb(255, 255, 255)
}
pub const fn black() -> StyledColor {
    StyledColor::rgb(0, 0, 0)
}
/// Fully transparent (`rgba(0, 0, 0, 0)`). Tailwind's `transparent`.
///
/// Returns a **solid** `Fill::Color` with alpha `0`, not `Fill::None`.
pub const fn transparent() -> StyledColor {
    StyledColor::rgb(0, 0, 0).opacity(0.0)
}

// ---- Neutrals ----
family!(stone;
    50: (250, 250, 249), 100: (245, 245, 244), 200: (231, 229, 228),
    300: (214, 211, 209), 400: (166, 160, 155), 500: (121, 113, 107),
    600: (87, 83, 77), 700: (68, 64, 59), 800: (41, 37, 36),
    900: (28, 25, 23), 950: (12, 10, 9),
);
family!(mauve;
    50: (250, 250, 250), 100: (243, 241, 243), 200: (231, 228, 231),
    300: (215, 208, 215), 400: (168, 158, 169), 500: (121, 105, 123),
    600: (89, 76, 91), 700: (70, 57, 71), 800: (42, 33, 44),
    900: (29, 22, 30), 950: (12, 9, 12),
);
family!(olive;
    50: (251, 251, 249), 100: (244, 244, 240), 200: (232, 232, 227),
    300: (216, 216, 208), 400: (171, 171, 156), 500: (124, 124, 103),
    600: (91, 91, 75), 700: (71, 71, 57), 800: (43, 43, 34),
    900: (29, 29, 22), 950: (12, 12, 9),
);
family!(mist;
    50: (249, 251, 251), 100: (241, 243, 243), 200: (227, 231, 232),
    300: (208, 214, 216), 400: (156, 168, 171), 500: (103, 120, 124),
    600: (75, 88, 91), 700: (57, 68, 71), 800: (34, 41, 43),
    900: (22, 27, 29), 950: (9, 11, 12),
);
family!(taupe;
    50: (251, 250, 249), 100: (243, 241, 241), 200: (232, 228, 227),
    300: (216, 210, 208), 400: (171, 160, 156), 500: (124, 109, 103),
    600: (91, 79, 75), 700: (71, 60, 57), 800: (43, 36, 34),
    900: (29, 24, 22), 950: (12, 10, 9),
);
family!(slate;
    50: (248, 250, 252), 100: (241, 245, 249), 200: (226, 232, 240),
    300: (203, 213, 225), 400: (148, 163, 184), 500: (100, 116, 139),
    600: (71, 85, 105), 700: (51, 65, 85), 800: (30, 41, 59),
    900: (15, 23, 42), 950: (2, 6, 23),
);
family!(gray;
    50: (249, 250, 251), 100: (243, 244, 246), 200: (229, 231, 235),
    300: (209, 213, 219), 400: (156, 163, 175), 500: (107, 114, 128),
    600: (75, 85, 99), 700: (55, 65, 81), 800: (31, 41, 55),
    900: (17, 24, 39), 950: (3, 7, 18),
);
family!(zinc;
    50: (250, 250, 250), 100: (244, 244, 245), 200: (228, 228, 231),
    300: (212, 212, 216), 400: (161, 161, 170), 500: (113, 113, 122),
    600: (82, 82, 91), 700: (63, 63, 70), 800: (39, 39, 42),
    900: (24, 24, 27), 950: (9, 9, 11),
);
family!(neutral;
    50: (250, 250, 250), 100: (245, 245, 245), 200: (229, 229, 229),
    300: (212, 212, 212), 400: (163, 163, 163), 500: (115, 115, 115),
    600: (82, 82, 82), 700: (64, 64, 64), 800: (38, 38, 38),
    900: (23, 23, 23), 950: (10, 10, 10),
);

// ---- Chromatic ----
family!(red;
    50: (254, 242, 242), 100: (254, 226, 226), 200: (254, 202, 202),
    300: (252, 165, 165), 400: (248, 113, 113), 500: (239, 68, 68),
    600: (220, 38, 38), 700: (185, 28, 28), 800: (153, 27, 27),
    900: (127, 29, 29), 950: (69, 10, 10),
);
family!(orange;
    50: (255, 247, 237), 100: (255, 237, 213), 200: (254, 215, 170),
    300: (253, 186, 116), 400: (251, 146, 60), 500: (249, 115, 22),
    600: (234, 88, 12), 700: (194, 65, 12), 800: (154, 52, 18),
    900: (124, 45, 18), 950: (67, 20, 7),
);
family!(amber;
    50: (255, 251, 235), 100: (254, 243, 199), 200: (253, 230, 138),
    300: (252, 211, 77), 400: (251, 191, 36), 500: (245, 158, 11),
    600: (217, 119, 6), 700: (180, 83, 9), 800: (146, 64, 14),
    900: (120, 53, 15), 950: (69, 26, 3),
);
family!(yellow;
    50: (254, 252, 232), 100: (254, 249, 195), 200: (254, 240, 138),
    300: (253, 224, 71), 400: (250, 204, 21), 500: (234, 179, 8),
    600: (202, 138, 4), 700: (161, 98, 7), 800: (133, 77, 14),
    900: (113, 63, 18), 950: (66, 32, 6),
);
family!(lime;
    50: (247, 254, 231), 100: (236, 252, 203), 200: (217, 249, 157),
    300: (190, 242, 100), 400: (163, 230, 53), 500: (132, 204, 22),
    600: (101, 163, 13), 700: (77, 124, 15), 800: (63, 98, 18),
    900: (54, 83, 20), 950: (26, 46, 5),
);
family!(green;
    50: (240, 253, 244), 100: (220, 252, 231), 200: (187, 247, 208),
    300: (134, 239, 172), 400: (74, 222, 128), 500: (34, 197, 94),
    600: (22, 163, 74), 700: (21, 128, 61), 800: (22, 101, 52),
    900: (20, 83, 45), 950: (5, 46, 22),
);
family!(emerald;
    50: (236, 253, 245), 100: (209, 250, 229), 200: (167, 243, 208),
    300: (110, 231, 183), 400: (52, 211, 153), 500: (16, 185, 129),
    600: (5, 150, 105), 700: (4, 120, 87), 800: (6, 95, 70),
    900: (6, 78, 59), 950: (2, 44, 34),
);
family!(teal;
    50: (240, 253, 250), 100: (204, 251, 241), 200: (153, 246, 228),
    300: (94, 234, 212), 400: (45, 212, 191), 500: (20, 184, 166),
    600: (13, 148, 136), 700: (15, 118, 110), 800: (17, 94, 89),
    900: (19, 78, 74), 950: (4, 47, 46),
);
family!(cyan;
    50: (236, 254, 255), 100: (207, 250, 254), 200: (165, 243, 252),
    300: (103, 232, 249), 400: (34, 211, 238), 500: (6, 182, 212),
    600: (8, 145, 178), 700: (14, 116, 144), 800: (21, 94, 117),
    900: (22, 78, 99), 950: (8, 51, 68),
);
family!(sky;
    50: (240, 249, 255), 100: (224, 242, 254), 200: (186, 230, 253),
    300: (125, 211, 252), 400: (56, 189, 248), 500: (14, 165, 233),
    600: (2, 132, 199), 700: (3, 105, 161), 800: (7, 89, 133),
    900: (12, 74, 110), 950: (8, 47, 73),
);
family!(blue;
    50: (239, 246, 255), 100: (219, 234, 254), 200: (191, 219, 254),
    300: (147, 197, 253), 400: (96, 165, 250), 500: (59, 130, 246),
    600: (37, 99, 235), 700: (29, 78, 216), 800: (30, 64, 175),
    900: (30, 58, 138), 950: (23, 37, 84),
);
family!(indigo;
    50: (238, 242, 255), 100: (224, 231, 255), 200: (199, 210, 254),
    300: (165, 180, 252), 400: (129, 140, 248), 500: (99, 102, 241),
    600: (79, 70, 229), 700: (67, 56, 202), 800: (55, 48, 163),
    900: (49, 46, 129), 950: (30, 27, 75),
);
family!(violet;
    50: (245, 243, 255), 100: (237, 233, 254), 200: (221, 214, 254),
    300: (196, 181, 253), 400: (167, 139, 250), 500: (139, 92, 246),
    600: (124, 58, 237), 700: (109, 40, 217), 800: (91, 33, 182),
    900: (76, 29, 149), 950: (46, 16, 101),
);
family!(purple;
    50: (250, 245, 255), 100: (243, 232, 255), 200: (233, 213, 255),
    300: (216, 180, 254), 400: (192, 132, 252), 500: (168, 85, 247),
    600: (147, 51, 234), 700: (126, 34, 206), 800: (107, 33, 168),
    900: (88, 28, 135), 950: (59, 7, 100),
);
family!(fuchsia;
    50: (253, 244, 255), 100: (250, 232, 255), 200: (245, 208, 254),
    300: (240, 171, 252), 400: (232, 121, 249), 500: (217, 70, 239),
    600: (192, 38, 211), 700: (162, 28, 175), 800: (134, 25, 143),
    900: (112, 26, 117), 950: (74, 4, 78),
);
family!(pink;
    50: (253, 242, 248), 100: (252, 231, 243), 200: (251, 207, 232),
    300: (249, 168, 212), 400: (244, 114, 182), 500: (236, 72, 153),
    600: (219, 39, 119), 700: (190, 24, 93), 800: (157, 23, 77),
    900: (131, 24, 67), 950: (80, 7, 36),
);
family!(rose;
    50: (255, 241, 242), 100: (255, 228, 230), 200: (254, 205, 211),
    300: (253, 164, 175), 400: (251, 113, 133), 500: (244, 63, 94),
    600: (225, 29, 72), 700: (190, 18, 60), 800: (159, 18, 57),
    900: (136, 19, 55), 950: (76, 5, 25),
);
