//! Fluent styling extensions for FreyaCN image elements.

use freya_core::elements::image::{AspectRatio, Image, ImageCover, ImageData, SamplingMode};
use freya_core::prelude::ImageExt;

/// Fluent styling extensions for image elements.
///
/// Mirrors the Tailwind utilities that map onto [`ImageData`]. Every
/// method consumes and returns `Self`, so calls chain like `StyledExt`:
///
/// ```ignore
/// image(handle)
///     .object_cover()
///     .object_center()
///     .image_smooth()
///     .blur_md()
/// ```
///
/// This is separate from [`StyledExt`](crate::styling::styled_ext::StyledExt)
/// because image-specific options live on [`ImageData`], not `Style`. Both
/// traits can be in scope at once without conflict — their method names
/// are disjoint.
///
/// The trait is organized into the following groups:
///
/// - **Object fit** — [`object_fit`](Self::object_fit) and its shortcuts
/// - **Object position** — [`object_position`](Self::object_position) and its shortcuts
/// - **Image rendering** — [`image_rendering`](Self::image_rendering), the
///   `image_*` rendering shortcuts, and the `sampling_*` raw shortcuts
/// - **Blur** — [`image_blur`](Self::image_blur) and the `blur_*` shortcuts
/// - **Grid snapping** — [`snap_to_grid`](Self::snap_to_grid) /
///   [`no_snap_to_grid`](Self::no_snap_to_grid) (Freya-specific, no Tailwind class)
pub trait ImageStyledExt: Sized {
    /// Returns a mutable reference to the underlying [`ImageData`].
    fn get_image_data(&mut self) -> &mut ImageData;

    // ---- object fit ----

    /// Sets how the image is scaled to fit its bounds while preserving
    /// (or ignoring) its aspect ratio.
    ///
    /// The `object_*` shortcuts below delegate to this and match
    /// Tailwind's `object-fit` utilities.
    fn object_fit(mut self, aspect_ratio: impl Into<AspectRatio>) -> Self {
        self.get_image_data().aspect_ratio = aspect_ratio.into();
        self
    }

    /// Scale so the whole image fits inside the bounds. Tailwind's
    /// `object-contain`.
    fn object_contain(self) -> Self {
        self.object_fit(AspectRatio::Min)
    }

    /// Scale so the image covers the whole bounds, cropping the overflow.
    /// Tailwind's `object-cover`.
    fn object_cover(self) -> Self {
        self.object_fit(AspectRatio::Max)
    }

    /// Stretch the image to the bounds, ignoring its aspect ratio.
    /// Tailwind's `object-fill`.
    fn object_fill(self) -> Self {
        self.object_fit(AspectRatio::None)
    }

    /// Keep the image at its natural (intrinsic) size. Tailwind's
    /// `object-none`.
    fn object_none(self) -> Self {
        self.object_fit(AspectRatio::Fit)
    }

    /// Scale so the image fits inside the bounds **only if it's bigger**;
    /// never scale a smaller image up. Tailwind's `object-scale-down`.
    ///
    /// # Approximation
    ///
    /// [`AspectRatio`] has no dedicated `ScaleDown` variant, so this
    /// falls back to [`AspectRatio::Min`] (always contain). When the
    /// image is **smaller** than the bounds, CSS's `object-scale-down`
    /// would keep the natural size while this shortcut scales it up.
    fn object_scale_down(self) -> Self {
        self.object_fit(AspectRatio::Min)
    }

    // ---- object position ----

    /// Sets where the image is anchored inside its bounds once scaled.
    ///
    /// [`ImageCover`] currently only models two anchors —
    /// [`ImageCover::Fill`] (top-left) and [`ImageCover::Center`] — so
    /// only `object_center` and `object_top_left` are available. The
    /// other Tailwind `object-*` position classes can't be expressed yet;
    /// see the summary at the end of this trait.
    fn object_position(mut self, image_cover: impl Into<ImageCover>) -> Self {
        self.get_image_data().image_cover = image_cover.into();
        self
    }

    /// Center the image in its bounds. Tailwind's `object-center`.
    fn object_center(self) -> Self {
        self.object_position(ImageCover::Center)
    }

    /// Anchor the image to the top-left of its bounds. Tailwind's
    /// `object-top-left` / `object-left-top` (they're the same). This is
    /// the default.
    fn object_top_left(self) -> Self {
        self.object_position(ImageCover::Fill)
    }

    // ---- image rendering ----

    /// Sets the sampling (filtering) algorithm used when the image is
    /// scaled.
    ///
    /// The `image_*` rendering shortcuts and `sampling_*` raw shortcuts
    /// below all delegate to this.
    fn image_rendering(mut self, sampling: impl Into<SamplingMode>) -> Self {
        self.get_image_data().sampling_mode = sampling.into();
        self
    }

    /// Let Freya pick the best algorithm — currently
    /// [`SamplingMode::Trilinear`]. Tailwind's `image-auto`. This is the
    /// default.
    fn image_auto(self) -> Self {
        self.image_rendering(SamplingMode::Trilinear)
    }

    /// Nearest-neighbor — sharp pixels, best for pixel art. Tailwind's
    /// `image-pixelated`.
    fn image_pixelated(self) -> Self {
        self.image_rendering(SamplingMode::Nearest)
    }

    /// Smooth high-quality resampling. Tailwind's `image-smooth`.
    ///
    /// Maps to [`SamplingMode::Mitchell`] (Mitchell-Netravali cubic),
    /// which is the smooth filter that closest matches CSS's `smooth`
    /// hint. If you prefer trilinear interpolation, use
    /// [`image_trilinear`](Self::image_trilinear).
    fn image_smooth(self) -> Self {
        self.image_rendering(SamplingMode::Mitchell)
    }

    /// Nearest-neighbor sampling. Same as
    /// [`image_pixelated`](Self::image_pixelated), but named after the
    /// [`SamplingMode`] variant for callers who want the raw concept.
    fn sampling_nearest(self) -> Self {
        self.image_rendering(SamplingMode::Nearest)
    }

    /// Bilinear interpolation.
    fn sampling_bilinear(self) -> Self {
        self.image_rendering(SamplingMode::Bilinear)
    }

    /// Trilinear interpolation (bilinear + mipmaps). Freya's default.
    fn sampling_trilinear(self) -> Self {
        self.image_rendering(SamplingMode::Trilinear)
    }

    /// Mitchell-Netravali cubic resampling — smooth, no halos.
    fn sampling_mitchell(self) -> Self {
        self.image_rendering(SamplingMode::Mitchell)
    }

    /// Catmull-Rom cubic resampling — sharper, slightly more ringing.
    fn sampling_catmull_rom(self) -> Self {
        self.image_rendering(SamplingMode::CatmullRom)
    }

    // ---- blur ----

    /// Applies a Gaussian blur of the given radius (in pixels) to the
    /// image itself.
    ///
    /// The `blur_*` shortcuts below delegate to this and match Tailwind's
    /// **filter** blur scale (`blur-*`, not `backdrop-blur-*` — the
    /// latter is [`backdrop_blur`](crate::styling::styled_ext::StyledExt::backdrop_blur)
    /// on `StyledExt`).
    ///
    /// The value is clamped to `>= 0.0`, so `blur(-1.0)` is treated as
    /// `0.0` (no blur).
    fn image_blur(mut self, blur: f32) -> Self {
        self.get_image_data().blur = blur.max(0.0);
        self
    }

    /// No blur — `0px`. Tailwind's `blur-none`. This is the default.
    fn blur_none(self) -> Self {
        self.image_blur(0.0)
    }
    /// Extra-extra-small blur — `2px`. Tailwind's `blur-2xs` (new in v4).
    fn blur_2xs(self) -> Self {
        self.image_blur(2.0)
    }
    /// Extra-small blur — `4px`. Tailwind's `blur-xs`.
    fn blur_xs(self) -> Self {
        self.image_blur(4.0)
    }
    /// Small blur — `8px`. Tailwind's `blur-sm`.
    fn blur_sm(self) -> Self {
        self.image_blur(8.0)
    }
    /// Medium blur — `12px`. Tailwind's `blur-md`.
    fn blur_md(self) -> Self {
        self.image_blur(12.0)
    }
    /// Large blur — `16px`. Tailwind's `blur-lg`.
    fn blur_lg(self) -> Self {
        self.image_blur(16.0)
    }
    /// Extra-large blur — `24px`. Tailwind's `blur-xl`.
    fn blur_xl(self) -> Self {
        self.image_blur(24.0)
    }
    /// 2× extra-large blur — `40px`. Tailwind's `blur-2xl`.
    fn blur_2xl(self) -> Self {
        self.image_blur(40.0)
    }
    /// 3× extra-large blur — `64px`. Tailwind's `blur-3xl`.
    fn blur_3xl(self) -> Self {
        self.image_blur(64.0)
    }

    // ---- grid snapping (Freya-specific) ----

    /// Snap the image's destination rect (and its clip rect) to integer
    /// pixels. Avoids half-pixel anti-aliasing seams at non-integer
    /// positions or scale factors.
    ///
    /// No Tailwind equivalent — Tailwind's `image-pixelated` covers
    /// *sampling*, not *position* snapping. Use
    /// [`image_pixelated`](Self::image_pixelated) for the sampling side.
    fn snap_to_grid(mut self) -> Self {
        self.get_image_data().snap_to_grid = true;
        self
    }

    /// Stop snapping the image to the pixel grid. This is the default.
    fn no_snap_to_grid(mut self) -> Self {
        self.get_image_data().snap_to_grid = false;
        self
    }
}

// ---- Impl for Freya's `Image` element ----

impl ImageStyledExt for Image {
    fn get_image_data(&mut self) -> &mut ImageData {
        ImageExt::get_image_data(self)
    }
}
