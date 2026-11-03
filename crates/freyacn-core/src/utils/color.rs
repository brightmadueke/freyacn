//! `Color` extensions for the helpers that don't exist on Freya's `Color`
//! type. Every function clamps to `0..=255` and treats `NaN` as `0.0`.
//!
//! # Relationship to `StyledColor`
//!
//! `StyledColor` is the *primary* color API. These extensions exist for
//! contexts where you already have a `Color` and shouldn't need to wrap
//! it (`to_color()` round-trips, reading a token out of a `Fill`, …). They
//! deliberately mirror `StyledColor`'s clamping behavior so switching
//! between the two never changes the result.

use freya::prelude::Color;

#[inline]
fn clamp_unit(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

#[inline]
fn unit_to_u8(v: f32) -> u8 {
    (clamp_unit(v) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Extensions for [`Color`]. Import once and every `Color` value gains
/// these methods.
///
/// ```
/// use freya::prelude::Color;
/// use freya_cn::utils::ColorExt;
///
/// let red = Color::from_rgb(255, 0, 0);
/// assert_eq!(red.with_alpha(0.5), Color::from_argb(128, 255, 0, 0));
/// ```
pub trait ColorExt: Sized {
    /// Return `self` with its alpha channel **overwritten** by `alpha`
    /// (clamped to `[0, 1]`; `NaN` → `0`).
    ///
    /// Unlike [`multiply_alpha`](Self::multiply_alpha), this discards the
    /// existing alpha.
    fn with_alpha(self, alpha: f32) -> Color;

    /// Return `self` with its alpha channel **multiplied** by `factor`
    /// (clamped to `[0, 1]`; `NaN` → `0`).
    ///
    /// A `factor` of `1.0` is a no-op; `0.0` makes the color fully
    /// transparent. Values above `1.0` are clamped, so this can only
    /// **reduce** opacity — matching `StyledColor::with_opacity`.
    fn multiply_alpha(self, factor: f32) -> Color;

    /// Linearly interpolate from `self` toward `other` by `ratio`
    /// (clamped to `[0, 1]`; `NaN` → `0`).
    ///
    /// This is a per-channel lerp, **not** Porter-Duff compositing:
    /// `ratio = 1` always returns `other`, even if `other` is transparent.
    /// If you need "draw `other` on top of `self` at alpha `ratio`", that's
    /// a different formula — ask and I'll write it.
    fn blend_toward(self, other: Color, ratio: f32) -> Color;

    /// WCAG relative luminance in `0.0..=1.0`.
    ///
    /// `0.0` = black, `1.0` = white. Used by
    /// [`contrast_ratio`](Self::contrast_ratio) and by any "should I put
    /// white or black text on this background?" heuristic.
    fn luminance(self) -> f32;

    /// WCAG contrast ratio between `self` and `other`, in `1.0..=21.0`.
    ///
    /// `4.5` is WCAG AA for body text; `3.0` for large text and UI
    /// elements. `theme.rs::validate` uses the same formula — this is the
    /// public, reusable form.
    fn contrast_ratio(self, other: Color) -> f32;

    /// `true` when [`luminance`](Self::luminance) is below `0.5`.
    ///
    /// Not a formal WCAG check — just a fast heuristic for picking a text
    /// color to go on top of `self`.
    fn is_dark(self) -> bool;
}

impl ColorExt for Color {
    fn with_alpha(self, alpha: f32) -> Color {
        Color::from_argb(unit_to_u8(alpha), self.r(), self.g(), self.b())
    }

    fn multiply_alpha(self, factor: f32) -> Color {
        let a = (self.a() as f32 * clamp_unit(factor))
            .round()
            .clamp(0.0, 255.0) as u8;
        Color::from_argb(a, self.r(), self.g(), self.b())
    }

    fn blend_toward(self, other: Color, ratio: f32) -> Color {
        let t = clamp_unit(ratio);
        let lerp = |a: u8, b: u8| -> u8 {
            let af = a as f32;
            let bf = b as f32;
            (af + (bf - af) * t).round().clamp(0.0, 255.0) as u8
        };
        Color::from_argb(
            lerp(self.a(), other.a()),
            lerp(self.r(), other.r()),
            lerp(self.g(), other.g()),
            lerp(self.b(), other.b()),
        )
    }

    fn luminance(self) -> f32 {
        let channel = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(self.r()) + 0.7152 * channel(self.g()) + 0.0722 * channel(self.b())
    }

    fn contrast_ratio(self, other: Color) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    fn is_dark(self) -> bool {
        self.luminance() < 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_alpha_clamps() {
        let red = Color::from_rgb(255, 0, 0);
        assert_eq!(red.with_alpha(1.5), red);
        assert_eq!(red.with_alpha(-0.5), Color::from_argb(0, 255, 0, 0));
        assert_eq!(red.with_alpha(f32::NAN), Color::from_argb(0, 255, 0, 0));
    }

    #[test]
    fn multiply_alpha_only_reduces() {
        let semi = Color::from_argb(200, 100, 100, 100);
        assert_eq!(semi.multiply_alpha(2.0), semi);
        assert_eq!(
            semi.multiply_alpha(0.5),
            Color::from_argb(100, 100, 100, 100)
        );
    }

    #[test]
    fn blend_is_clamped_lerp() {
        let black = Color::from_rgb(0, 0, 0);
        let white = Color::from_rgb(255, 255, 255);
        assert_eq!(black.blend_toward(white, 2.0), white);
        assert_eq!(black.blend_toward(white, -1.0), black);
        assert_eq!(
            black.blend_toward(white, 0.5),
            Color::from_rgb(128, 128, 128)
        );
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let black = Color::from_rgb(0, 0, 0);
        let white = Color::from_rgb(255, 255, 255);
        let r = black.contrast_ratio(white);
        assert!((r - 21.0).abs() < 0.1, "got {r}");
    }
}
