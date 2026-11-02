use freya::prelude::{Color, Fill};

// ---- Color helpers ----
pub fn color_with_alpha(color: Color, alpha: f32) -> Color {
    let r = color.r();
    let g = color.g();
    let b = color.b();
    let a = (alpha * 255.0) as u8;
    Color::from_argb(a, r, g, b)
}

pub fn blend_colors(base: Color, blend: Color, ratio: f32) -> Color {
    let r1 = base.r() as f32;
    let g1 = base.g() as f32;
    let b1 = base.b() as f32;
    let a1 = base.a() as f32 / 255.0;
    let r2 = blend.r() as f32;
    let g2 = blend.g() as f32;
    let b2 = blend.b() as f32;
    let a2 = blend.a() as f32 / 255.0;
    let r = r1 + (r2 - r1) * ratio;
    let g = g1 + (g2 - g1) * ratio;
    let b = b1 + (b2 - b1) * ratio;
    let a = a1 + (a2 - a1) * ratio;
    Color::from_argb((a * 255.0) as u8, r as u8, g as u8, b as u8)
}

pub fn get_color_from_fill(fill: Fill) -> Color {
    match fill {
        Fill::Color(color) => color,
        // If it's a gradient or shader, fall back to a default color
        _ => Color::TRANSPARENT,
    }
}

pub fn apply_opacity(c: Color, factor: f32) -> Color {
    let a = (c.a() as f32 * factor) as u8;
    Color::from_argb(a, c.r(), c.g(), c.b())
}
