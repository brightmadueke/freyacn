//! Runtime CSS parser for the `style!` macro.
//!
//! Accepts property/value declarations without a selector, e.g.
//!
//! ```ignore
//! style!(r#"
//!     display: flex;
//!     padding: 20px;
//!     width: calc(100% - 20px);
//!     color: #ff0000;
//!     background-color: rgba(0, 0, 0, 0.4);
//! "#)
//! ```

use crate::stylesheet::style::Style;
use crate::stylesheet::types::*;
use crate::stylesheet::unit_value::UnitValue;

// ============================================================================
// Public entry point
// ============================================================================

/// Parse a CSS-like string (declarations only) into a `Style`.
///
/// * `property: value;` pairs, `;` terminated.
/// * Unknown properties are ignored.
/// * `/* block comments */` and `// line comments` are stripped.
/// * A trailing `!important` is stripped from values.
pub fn parse_style(css: &str) -> Style {
    let mut style = Style::default();
    for (name, value) in parse_declarations(css) {
        apply_declaration(&mut style, &name, &value);
    }
    style
}

// ============================================================================
// Declaration scanner
// ============================================================================

fn parse_declarations(input: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut name = String::new();
    let mut value = String::new();
    let mut in_name = true;
    let mut in_comment = false;
    let mut line_comment = false;
    let mut in_string = false;
    let mut parens = 0u32;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if line_comment {
            if c == '\n' {
                line_comment = false;
            }
            continue;
        }
        if in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_comment = false;
            }
            continue;
        }
        if in_string {
            value.push(c);
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                in_comment = true;
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                line_comment = true;
            }
            '"' => {
                in_string = true;
                value.push(c);
            }
            '(' => {
                parens += 1;
                value.push(c);
            }
            ')' => {
                parens = parens.saturating_sub(1);
                value.push(c);
            }
            ':' if in_name => {
                in_name = false;
            }
            ';' if parens == 0 => {
                push_decl(&mut out, &name, &value);
                name.clear();
                value.clear();
                in_name = true;
            }
            other => {
                if in_name {
                    name.push(other);
                } else {
                    value.push(other);
                }
            }
        }
    }
    // Trailing declaration without terminating `;`.
    push_decl(&mut out, &name, &value);
    out
}

fn push_decl(out: &mut Vec<(String, String)>, name: &str, value: &str) {
    let name = name.trim();
    let mut value = value.trim();
    // Strip trailing `!important`.
    if let Some(stripped) = value.strip_suffix("!important") {
        value = stripped.trim_end();
    }
    if name.is_empty() || value.is_empty() {
        return;
    }
    out.push((name.replace('-', "_"), value.to_string()));
}

// ============================================================================
// Value helpers
// ============================================================================

fn parse_size_value(s: &str) -> SizeValue {
    // `SizeValue::from_str` is infallible; fallback never fires.
    s.parse().unwrap_or(SizeValue::Auto)
}

fn parse_color(s: &str) -> Color {
    s.parse().unwrap_or(Color::BLACK)
}

fn parse_line_height(s: &str) -> LineHeight {
    s.parse().unwrap_or(LineHeight::Normal)
}

/// Split on whitespace, but not inside `(...)`.
fn split_space(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0u32;
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            ' ' | '\t' | '\n' | '\r' if depth == 0 => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Split on commas, but not inside `(...)` or quoted strings.
fn split_comma(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0u32;
    let mut in_string = false;
    for c in s.chars() {
        if in_string {
            cur.push(c);
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                cur.push(c);
            }
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            ',' if depth == 0 => {
                let t = cur.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    let t = cur.trim();
    if !t.is_empty() {
        out.push(t.to_string());
    }
    out
}

fn split_string_list(s: &str) -> Vec<String> {
    split_comma(s)
        .into_iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

// ============================================================================
// Extended helpers
// ============================================================================

/// True iff the token starts like a UnitValue (digit, sign, or decimal point).
fn looks_like_unit_value(s: &str) -> bool {
    s.chars().next().map_or(false, |c| {
        c.is_ascii_digit() || c == '-' || c == '+' || c == '.'
    })
}

/// Parse `<time>` (`"0.3s"`, `"300ms"`).
fn parse_time(s: &str) -> Option<f32> {
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<f32>().ok().map(|v| v / 1000.0)
    } else {
        s.strip_suffix('s').and_then(|v| v.parse::<f32>().ok())
    }
}

/// Parse `<angle>` (`"45deg"`, `"0.5turn"`, `"3.14rad"`, bare number → deg).
fn parse_angle(s: &str) -> Option<f32> {
    if let Some(v) = s.strip_suffix("deg") {
        v.parse().ok()
    } else if let Some(v) = s.strip_suffix("rad") {
        v.parse::<f32>().ok().map(f32::to_degrees)
    } else if let Some(v) = s.strip_suffix("turn") {
        v.parse::<f32>().ok().map(|v| v * 360.0)
    } else if let Some(v) = s.strip_suffix("grad") {
        v.parse::<f32>().ok().map(|v| v * 0.9)
    } else {
        s.parse().ok()
    }
}

/// Parse `50%` → `0.5`, `1.5` → `1.5`, `2` → `2.0`.
fn parse_percentage_or_number(s: &str) -> Option<f32> {
    if let Some(pct) = s.strip_suffix('%') {
        pct.parse::<f32>().ok().map(|v| v / 100.0)
    } else {
        s.parse().ok()
    }
}

// ---------------------------------------------------------------------------
// CssImage
// ---------------------------------------------------------------------------

fn parse_css_image(s: &str) -> Option<CssImage> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.eq_ignore_ascii_case("none") {
        return Some(CssImage::None);
    }
    if let Some(rest) = s.strip_prefix("url(") {
        if let Some(inner) = rest.strip_suffix(')') {
            let inner = inner.trim().trim_matches(|c| c == '"' || c == '\'');
            return Some(CssImage::Url(inner.to_string()));
        }
    }
    // Unknown form — store the raw string.
    Some(CssImage::Url(s.to_string()))
}

fn parse_css_image_list(s: &str) -> Vec<CssImage> {
    split_comma(s)
        .into_iter()
        .filter_map(|t| parse_css_image(&t))
        .collect()
}

// ---------------------------------------------------------------------------
// Position2D
// ---------------------------------------------------------------------------

fn parse_position_2d(s: &str) -> Option<Position2D> {
    let parts = split_space(s);
    match parts.len() {
        1 => {
            let p = &parts[0];
            if let Ok(h) = p.parse::<HorizontalKeyword>() {
                return Some(Position2D::Keywords(h, VerticalKeyword::Center));
            }
            if let Ok(v) = p.parse::<VerticalKeyword>() {
                return Some(Position2D::Keywords(HorizontalKeyword::Center, v));
            }
            Some(Position2D::X(UnitValue::new(p.as_str())))
        }
        2 => {
            let p1 = &parts[0];
            let p2 = &parts[1];
            if let (Ok(h), Ok(v)) = (
                p1.parse::<HorizontalKeyword>(),
                p2.parse::<VerticalKeyword>(),
            ) {
                return Some(Position2D::Keywords(h, v));
            }
            Some(Position2D::XY(
                UnitValue::new(p1.as_str()),
                UnitValue::new(p2.as_str()),
            ))
        }
        _ => None,
    }
}

fn parse_position_2d_list(s: &str) -> Vec<Position2D> {
    split_comma(s)
        .into_iter()
        .filter_map(|t| parse_position_2d(&t))
        .collect()
}

// ---------------------------------------------------------------------------
// BackgroundSize
// ---------------------------------------------------------------------------

fn parse_background_size(s: &str) -> Option<BackgroundSize> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("cover") {
        return Some(BackgroundSize::Cover);
    }
    if s.eq_ignore_ascii_case("contain") {
        return Some(BackgroundSize::Contain);
    }
    let parts = split_space(s);
    match parts.len() {
        1 => Some(BackgroundSize::Explicit(
            parse_size_value(&parts[0]),
            SizeValue::Auto,
        )),
        2 => Some(BackgroundSize::Explicit(
            parse_size_value(&parts[0]),
            parse_size_value(&parts[1]),
        )),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// BoxShadow / TextShadow
// ---------------------------------------------------------------------------

fn parse_box_shadow(s: &str) -> Option<BoxShadow> {
    let parts = split_space(s);
    let mut inset = false;
    let mut unit_values: Vec<String> = Vec::new();
    let mut color: Option<Color> = None;

    for p in parts {
        if p.eq_ignore_ascii_case("inset") {
            inset = true;
        } else if !looks_like_unit_value(&p) && color.is_none() {
            color = Some(parse_color(&p));
        } else {
            unit_values.push(p);
        }
    }

    if unit_values.len() < 2 {
        return None;
    }

    Some(BoxShadow {
        offset_x: UnitValue::new(unit_values[0].as_str()),
        offset_y: UnitValue::new(unit_values[1].as_str()),
        blur: unit_values
            .get(2)
            .map(|s| UnitValue::new(s.as_str()))
            .unwrap_or_else(UnitValue::zero),
        spread: unit_values
            .get(3)
            .map(|s| UnitValue::new(s.as_str()))
            .unwrap_or_else(UnitValue::zero),
        color: color.unwrap_or(Color::BLACK),
        inset,
    })
}

fn parse_box_shadow_list(s: &str) -> Vec<BoxShadow> {
    split_comma(s)
        .into_iter()
        .filter_map(|t| parse_box_shadow(&t))
        .collect()
}

fn parse_text_shadow(s: &str) -> Option<TextShadow> {
    let parts = split_space(s);
    let mut unit_values: Vec<String> = Vec::new();
    let mut color: Option<Color> = None;

    for p in parts {
        if !looks_like_unit_value(&p) && color.is_none() {
            color = Some(parse_color(&p));
        } else {
            unit_values.push(p);
        }
    }

    if unit_values.len() < 2 {
        return None;
    }

    Some(TextShadow {
        offset_x: UnitValue::new(unit_values[0].as_str()),
        offset_y: UnitValue::new(unit_values[1].as_str()),
        blur: unit_values
            .get(2)
            .map(|s| UnitValue::new(s.as_str()))
            .unwrap_or_else(UnitValue::zero),
        color: color.unwrap_or(Color::BLACK),
    })
}

fn parse_text_shadow_list(s: &str) -> Vec<TextShadow> {
    split_comma(s)
        .into_iter()
        .filter_map(|t| parse_text_shadow(&t))
        .collect()
}

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

fn parse_filter_function(s: &str) -> Option<FilterFunction> {
    let s = s.trim();
    let (name, arg) = s.split_once('(')?;
    let arg = arg.strip_suffix(')')?;
    let name = name.trim().to_ascii_lowercase();
    let arg = arg.trim();

    match name.as_str() {
        "blur" => Some(FilterFunction::Blur(UnitValue::new(arg))),
        "brightness" => parse_percentage_or_number(arg).map(FilterFunction::Brightness),
        "contrast" => parse_percentage_or_number(arg).map(FilterFunction::Contrast),
        "grayscale" => parse_percentage_or_number(arg).map(FilterFunction::Grayscale),
        "hue-rotate" => parse_angle(arg).map(FilterFunction::HueRotate),
        "invert" => parse_percentage_or_number(arg).map(FilterFunction::Invert),
        "opacity" => parse_percentage_or_number(arg).map(FilterFunction::Opacity),
        "saturate" => parse_percentage_or_number(arg).map(FilterFunction::Saturate),
        "sepia" => parse_percentage_or_number(arg).map(FilterFunction::Sepia),
        "drop-shadow" => parse_box_shadow(arg).map(FilterFunction::DropShadow),
        _ => None,
    }
}

/// Split a filter list on top-level `)` so `blur(5px)brightness(.5)` works.
fn parse_filter_list(s: &str) -> Vec<FilterFunction> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0u32;
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
                if depth == 0 {
                    if let Some(f) = parse_filter_function(&cur) {
                        out.push(f);
                    }
                    cur.clear();
                }
            }
            _ if depth == 0 && c.is_whitespace() => {}
            _ => cur.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

fn parse_transform_function(s: &str) -> Option<TransformFunction> {
    let s = s.trim();
    let (name, args) = s.split_once('(')?;
    let args = args.strip_suffix(')')?;
    let name = name.trim().to_ascii_lowercase();
    let parts: Vec<&str> = args.split(',').map(str::trim).collect();

    match name.as_str() {
        "translate" => match parts.len() {
            1 => Some(TransformFunction::Translate(
                UnitValue::new(parts[0]),
                UnitValue::zero(),
            )),
            2 => Some(TransformFunction::Translate(
                UnitValue::new(parts[0]),
                UnitValue::new(parts[1]),
            )),
            _ => None,
        },
        "translatex" => parts
            .first()
            .map(|p| TransformFunction::TranslateX(UnitValue::new(*p))),
        "translatey" => parts
            .first()
            .map(|p| TransformFunction::TranslateY(UnitValue::new(*p))),
        "scale" => match parts.len() {
            1 => parts[0]
                .parse::<f32>()
                .ok()
                .map(|v| TransformFunction::Scale(v, v)),
            2 => {
                let x = parts[0].parse::<f32>().ok()?;
                let y = parts[1].parse::<f32>().ok()?;
                Some(TransformFunction::Scale(x, y))
            }
            _ => None,
        },
        "scalex" => parts
            .first()
            .and_then(|p| p.parse::<f32>().ok())
            .map(TransformFunction::ScaleX),
        "scaley" => parts
            .first()
            .and_then(|p| p.parse::<f32>().ok())
            .map(TransformFunction::ScaleY),
        "rotate" => parts
            .first()
            .and_then(|p| parse_angle(p))
            .map(TransformFunction::Rotate),
        "skew" => match parts.len() {
            1 => parse_angle(parts[0]).map(|x| TransformFunction::Skew(x, 0.0)),
            2 => {
                let x = parse_angle(parts[0])?;
                let y = parse_angle(parts[1])?;
                Some(TransformFunction::Skew(x, y))
            }
            _ => None,
        },
        "matrix" if parts.len() == 6 => {
            let nums: Vec<f32> = parts.iter().filter_map(|p| p.parse().ok()).collect();
            if nums.len() == 6 {
                Some(TransformFunction::Matrix(
                    nums[0], nums[1], nums[2], nums[3], nums[4], nums[5],
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split a transform list on top-level `)` so `translate(10px)rotate(45deg)` works.
fn parse_transform_list(s: &str) -> Vec<TransformFunction> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0u32;
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
                if depth == 0 {
                    if let Some(f) = parse_transform_function(&cur) {
                        out.push(f);
                    }
                    cur.clear();
                }
            }
            _ if depth == 0 && c.is_whitespace() => {}
            _ => cur.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

fn is_timing_function(s: &str) -> bool {
    matches!(
        s,
        "ease" | "ease-in" | "ease-out" | "ease-in-out" | "linear" | "step-start" | "step-end"
    ) || s.starts_with("cubic-bezier(")
        || s.starts_with("steps(")
}

fn parse_transition(s: &str) -> Transition {
    let parts = split_space(s);
    let mut property = String::new();
    let mut duration = 0.0_f32;
    let mut timing = String::new();
    let mut delay = 0.0_f32;
    let mut time_count = 0;

    for p in parts {
        if let Some(t) = parse_time(&p) {
            if time_count == 0 {
                duration = t;
            } else if time_count == 1 {
                delay = t;
            }
            time_count += 1;
        } else if is_timing_function(&p) {
            timing = p;
        } else if property.is_empty() {
            property = p;
        }
    }

    Transition {
        property: if property.is_empty() {
            "all".to_string()
        } else {
            property
        },
        duration,
        timing: if timing.is_empty() {
            "ease".to_string()
        } else {
            timing
        },
        delay,
    }
}

fn parse_transition_list(s: &str) -> Vec<Transition> {
    split_comma(s)
        .into_iter()
        .map(|t| parse_transition(&t))
        .collect()
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

fn parse_animation(s: &str) -> Option<Animation> {
    const DIRECTIONS: &[&str] = &["normal", "reverse", "alternate", "alternate-reverse"];
    const FILL_MODES: &[&str] = &["forwards", "backwards", "both"];
    const PLAY_STATES: &[&str] = &["running", "paused"];

    let parts = split_space(s);
    let mut name = String::new();
    let mut duration = 0.0_f32;
    let mut timing = String::new();
    let mut delay = 0.0_f32;
    let mut iteration_count = String::new();
    let mut direction = String::new();
    let mut fill_mode = String::new();
    let mut play_state = String::new();
    let mut time_count = 0;

    for p in parts {
        if let Some(t) = parse_time(&p) {
            if time_count == 0 {
                duration = t;
            } else if time_count == 1 {
                delay = t;
            }
            time_count += 1;
        } else if is_timing_function(&p) {
            timing = p;
        } else if DIRECTIONS.contains(&p.as_str()) {
            direction = p;
        } else if FILL_MODES.contains(&p.as_str()) {
            fill_mode = p;
        } else if PLAY_STATES.contains(&p.as_str()) {
            play_state = p;
        } else if p == "infinite" || p.parse::<u32>().is_ok() || p.parse::<f32>().is_ok() {
            iteration_count = p;
        } else if name.is_empty() {
            name = p;
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Animation {
        name,
        duration,
        timing: if timing.is_empty() {
            "ease".to_string()
        } else {
            timing
        },
        delay,
        iteration_count: if iteration_count.is_empty() {
            "1".to_string()
        } else {
            iteration_count
        },
        direction: if direction.is_empty() {
            "normal".to_string()
        } else {
            direction
        },
        fill_mode: if fill_mode.is_empty() {
            "none".to_string()
        } else {
            fill_mode
        },
        play_state: if play_state.is_empty() {
            "running".to_string()
        } else {
            play_state
        },
    })
}

fn parse_animation_list(s: &str) -> Vec<Animation> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Vec::new();
    }
    split_comma(s)
        .into_iter()
        .filter_map(|t| parse_animation(&t))
        .collect()
}

// ---------------------------------------------------------------------------
// GridLine
// ---------------------------------------------------------------------------

fn parse_grid_line(s: &str) -> Option<GridLine> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("auto") {
        return Some(GridLine::Auto);
    }
    if let Some(rest) = s.strip_prefix("span ") {
        return rest.trim().parse::<u32>().ok().map(GridLine::Span);
    }
    if let Ok(n) = s.parse::<i32>() {
        return Some(GridLine::Line(n));
    }
    Some(GridLine::Named(s.to_string()))
}

// ============================================================================
// Setter macros
// ============================================================================

macro_rules! opt_enum {
    ($style:expr, $field:ident, $value:expr) => {
        if let Ok(v) = $value.parse() {
            $style.$field = Some(v);
        }
    };
}

macro_rules! opt_unit {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = Some(UnitValue::new($value))
    };
}

macro_rules! opt_size {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = Some(parse_size_value($value))
    };
}

macro_rules! opt_color {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = Some(parse_color($value))
    };
}

macro_rules! opt_f32 {
    ($style:expr, $field:ident, $value:expr) => {
        if let Ok(v) = $value.parse() {
            $style.$field = Some(v);
        }
    };
}

macro_rules! opt_i32 {
    ($style:expr, $field:ident, $value:expr) => {
        if let Ok(v) = $value.parse() {
            $style.$field = Some(v);
        }
    };
}

macro_rules! opt_str {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = Some($value.to_string())
    };
}

macro_rules! vec_unit {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = split_space($value)
            .into_iter()
            .map(UnitValue::new)
            .collect()
    };
}

macro_rules! vec_size {
    ($style:expr, $field:ident, $value:expr) => {
        $style.$field = split_space($value)
            .iter()
            .map(|s| parse_size_value(s))
            .collect()
    };
}

// ============================================================================
// Dispatch
// ============================================================================

fn apply_declaration(style: &mut Style, name: &str, value: &str) {
    match name {
        // ==================================================================
        // BOX MODEL & SIZING
        // ==================================================================
        "display" => {
            if let Ok(v) = value.parse() {
                style.display = vec![v];
            }
        }
        "box_sizing" => opt_enum!(style, box_sizing, value),
        "position" => opt_enum!(style, position, value),
        "top" => opt_size!(style, top, value),
        "right" => opt_size!(style, right, value),
        "bottom" => opt_size!(style, bottom, value),
        "left" => opt_size!(style, left, value),
        "inset" => vec_size!(style, inset, value),
        "inset_inline" => vec_size!(style, inset_inline, value),
        "inset_block" => vec_size!(style, inset_block, value),
        "inset_inline_start" => opt_size!(style, inset_inline_start, value),
        "inset_inline_end" => opt_size!(style, inset_inline_end, value),
        "inset_block_start" => opt_size!(style, inset_block_start, value),
        "inset_block_end" => opt_size!(style, inset_block_end, value),
        "z_index" => opt_i32!(style, z_index, value),
        "width" => opt_size!(style, width, value),
        "height" => opt_size!(style, height, value),
        "min_width" => opt_size!(style, min_width, value),
        "min_height" => opt_size!(style, min_height, value),
        "max_width" => opt_size!(style, max_width, value),
        "max_height" => opt_size!(style, max_height, value),
        "inline_size" => opt_size!(style, inline_size, value),
        "block_size" => opt_size!(style, block_size, value),
        "min_inline_size" => opt_size!(style, min_inline_size, value),
        "min_block_size" => opt_size!(style, min_block_size, value),
        "max_inline_size" => opt_size!(style, max_inline_size, value),
        "max_block_size" => opt_size!(style, max_block_size, value),
        "aspect_ratio" => {
            if let Some((w, h)) = value.split_once('/') {
                if let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                    style.aspect_ratio = Some(AspectRatio::new(w, h));
                }
            } else if let Ok(v) = value.parse::<f32>() {
                style.aspect_ratio = Some(AspectRatio::new(v, 1.0));
            }
        }
        "margin" => vec_size!(style, margin, value),
        "margin_top" => opt_size!(style, margin_top, value),
        "margin_right" => opt_size!(style, margin_right, value),
        "margin_bottom" => opt_size!(style, margin_bottom, value),
        "margin_left" => opt_size!(style, margin_left, value),
        "margin_inline" => vec_size!(style, margin_inline, value),
        "margin_block" => vec_size!(style, margin_block, value),
        "margin_inline_start" => opt_size!(style, margin_inline_start, value),
        "margin_inline_end" => opt_size!(style, margin_inline_end, value),
        "margin_block_start" => opt_size!(style, margin_block_start, value),
        "margin_block_end" => opt_size!(style, margin_block_end, value),
        "padding" => vec_unit!(style, padding, value),
        "padding_top" => opt_unit!(style, padding_top, value),
        "padding_right" => opt_unit!(style, padding_right, value),
        "padding_bottom" => opt_unit!(style, padding_bottom, value),
        "padding_left" => opt_unit!(style, padding_left, value),
        "padding_inline" => vec_unit!(style, padding_inline, value),
        "padding_block" => vec_unit!(style, padding_block, value),
        "padding_inline_start" => opt_unit!(style, padding_inline_start, value),
        "padding_inline_end" => opt_unit!(style, padding_inline_end, value),
        "padding_block_start" => opt_unit!(style, padding_block_start, value),
        "padding_block_end" => opt_unit!(style, padding_block_end, value),
        "overflow" => opt_enum!(style, overflow, value),
        "overflow_x" => opt_enum!(style, overflow_x, value),
        "overflow_y" => opt_enum!(style, overflow_y, value),
        "overflow_clip_margin" => opt_unit!(style, overflow_clip_margin, value),
        "visibility" => opt_enum!(style, visibility, value),
        "float" => opt_enum!(style, float, value),
        "clear" => opt_enum!(style, clear, value),
        "object_fit" => opt_enum!(style, object_fit, value),
        "object_position" => {
            if let Some(p) = parse_position_2d(value) {
                style.object_position = Some(p);
            }
        }

        // ==================================================================
        // FLEXBOX
        // ==================================================================
        "flex_direction" => opt_enum!(style, flex_direction, value),
        "flex_wrap" => opt_enum!(style, flex_wrap, value),
        "flex_flow" => style.flex_flow = split_space(value),
        "flex_grow" => opt_f32!(style, flex_grow, value),
        "flex_shrink" => opt_f32!(style, flex_shrink, value),
        "flex_basis" => opt_size!(style, flex_basis, value),
        "flex" => style.flex = split_space(value),
        "order" => opt_i32!(style, order, value),
        "justify_content" => opt_enum!(style, justify_content, value),
        "align_items" => opt_enum!(style, align_items, value),
        "align_self" => opt_enum!(style, align_self, value),
        "align_content" => opt_enum!(style, align_content, value),

        // ==================================================================
        // GRID
        // ==================================================================
        "grid_template_columns" => opt_str!(style, grid_template_columns, value),
        "grid_template_rows" => opt_str!(style, grid_template_rows, value),
        "grid_template_areas" => opt_str!(style, grid_template_areas, value),
        "grid_auto_columns" => opt_str!(style, grid_auto_columns, value),
        "grid_auto_rows" => opt_str!(style, grid_auto_rows, value),
        "grid_auto_flow" => opt_enum!(style, grid_auto_flow, value),
        "grid" => style.grid = split_space(value),
        "grid_column" => style.grid_column = split_space(value),
        "grid_row" => style.grid_row = split_space(value),
        "grid_area" => style.grid_area = split_space(value),
        "grid_column_start" => {
            if let Some(g) = parse_grid_line(value) {
                style.grid_column_start = Some(g);
            }
        }
        "grid_column_end" => {
            if let Some(g) = parse_grid_line(value) {
                style.grid_column_end = Some(g);
            }
        }
        "grid_row_start" => {
            if let Some(g) = parse_grid_line(value) {
                style.grid_row_start = Some(g);
            }
        }
        "grid_row_end" => {
            if let Some(g) = parse_grid_line(value) {
                style.grid_row_end = Some(g);
            }
        }
        "gap" => vec_unit!(style, gap, value),
        "row_gap" => opt_unit!(style, row_gap, value),
        "column_gap" => opt_unit!(style, column_gap, value),
        "justify_items" => opt_enum!(style, justify_items, value),
        "justify_self" => opt_enum!(style, justify_self, value),

        // ==================================================================
        // TYPOGRAPHY
        // ==================================================================
        "font_family" => style.font_family = Some(split_string_list(value)),
        "font_size" => opt_unit!(style, font_size, value),
        "font_weight" => opt_enum!(style, font_weight, value),
        "font_style" => opt_enum!(style, font_style, value),
        "font_stretch" => opt_enum!(style, font_stretch, value),
        "font_variant" => style.font_variant = split_space(value),
        "font_variant_caps" => opt_str!(style, font_variant_caps, value),
        "font_variant_ligatures" => opt_str!(style, font_variant_ligatures, value),
        "font_variant_numeric" => opt_str!(style, font_variant_numeric, value),
        "font_variant_east_asian" => opt_str!(style, font_variant_east_asian, value),
        "font_feature_settings" => opt_str!(style, font_feature_settings, value),
        "font_variation_settings" => opt_str!(style, font_variation_settings, value),
        "font_kerning" => opt_enum!(style, font_kerning, value),
        "font" => style.font = split_space(value),
        "line_height" => style.line_height = Some(parse_line_height(value)),
        "letter_spacing" => opt_unit!(style, letter_spacing, value),
        "word_spacing" => opt_unit!(style, word_spacing, value),
        "text_align" => opt_enum!(style, text_align, value),
        "text_align_last" => opt_enum!(style, text_align_last, value),
        "text_indent" => opt_unit!(style, text_indent, value),
        "text_transform" => opt_enum!(style, text_transform, value),
        "text_overflow" => opt_enum!(style, text_overflow, value),
        "text_rendering" => opt_enum!(style, text_rendering, value),
        "text_orientation" => opt_enum!(style, text_orientation, value),
        "text_wrap" => opt_str!(style, text_wrap, value),
        "text_decoration_line" => opt_enum!(style, text_decoration_line, value),
        "text_decoration_style" => opt_enum!(style, text_decoration_style, value),
        "text_decoration_color" => opt_color!(style, text_decoration_color, value),
        "text_decoration_thickness" => opt_unit!(style, text_decoration_thickness, value),
        "text_decoration_underline_offset" => {
            opt_unit!(style, text_decoration_underline_offset, value)
        }
        "text_decoration_skip_ink" => opt_enum!(style, text_decoration_skip_ink, value),
        "text_underline_position" => opt_enum!(style, text_underline_position, value),
        "text_decoration" => style.text_decoration = split_space(value),
        "text_shadow" => {
            style.text_shadow = parse_text_shadow_list(value);
        }
        "vertical_align" => opt_enum!(style, vertical_align, value),
        "white_space" => opt_enum!(style, white_space, value),
        "white_space_collapse" => opt_str!(style, white_space_collapse, value),
        "word_break" => opt_enum!(style, word_break, value),
        "overflow_wrap" => opt_enum!(style, overflow_wrap, value),
        "line_break" => opt_str!(style, line_break, value),
        "hyphens" => opt_enum!(style, hyphens, value),
        "tab_size" => opt_unit!(style, tab_size, value),
        "direction" => opt_enum!(style, direction, value),
        "unicode_bidi" => opt_enum!(style, unicode_bidi, value),
        "writing_mode" => opt_enum!(style, writing_mode, value),

        // ==================================================================
        // COLORS & BACKGROUNDS
        // ==================================================================
        "color" => opt_color!(style, color, value),
        "opacity" => opt_f32!(style, opacity, value),
        "background_color" => opt_color!(style, background_color, value),
        "background_image" => {
            style.background_image = parse_css_image_list(value);
        }
        "background_position" => {
            style.background_position = parse_position_2d_list(value);
        }
        "background_position_x" => opt_enum!(style, background_position_x, value),
        "background_position_y" => opt_enum!(style, background_position_y, value),
        "background_size" => {
            if let Some(s) = parse_background_size(value) {
                style.background_size = Some(s);
            }
        }
        "background_repeat" => opt_enum!(style, background_repeat, value),
        "background_attachment" => opt_enum!(style, background_attachment, value),
        "background_clip" => opt_enum!(style, background_clip, value),
        "background_origin" => opt_enum!(style, background_origin, value),
        "background_blend_mode" => opt_enum!(style, background_blend_mode, value),
        "background" => style.background = split_space(value),

        // ==================================================================
        // BORDERS & OUTLINES
        // ==================================================================
        "border_style" => opt_enum!(style, border_style, value),
        "border_width" => opt_unit!(style, border_width, value),
        "border_color" => opt_color!(style, border_color, value),
        "border_top_style" => opt_enum!(style, border_top_style, value),
        "border_top_width" => opt_unit!(style, border_top_width, value),
        "border_top_color" => opt_color!(style, border_top_color, value),
        "border_right_style" => opt_enum!(style, border_right_style, value),
        "border_right_width" => opt_unit!(style, border_right_width, value),
        "border_right_color" => opt_color!(style, border_right_color, value),
        "border_bottom_style" => opt_enum!(style, border_bottom_style, value),
        "border_bottom_width" => opt_unit!(style, border_bottom_width, value),
        "border_bottom_color" => opt_color!(style, border_bottom_color, value),
        "border_left_style" => opt_enum!(style, border_left_style, value),
        "border_left_width" => opt_unit!(style, border_left_width, value),
        "border_left_color" => opt_color!(style, border_left_color, value),
        "border_inline_start_style" => opt_enum!(style, border_inline_start_style, value),
        "border_inline_start_width" => opt_unit!(style, border_inline_start_width, value),
        "border_inline_start_color" => opt_color!(style, border_inline_start_color, value),
        "border_inline_end_style" => opt_enum!(style, border_inline_end_style, value),
        "border_inline_end_width" => opt_unit!(style, border_inline_end_width, value),
        "border_inline_end_color" => opt_color!(style, border_inline_end_color, value),
        "border_block_start_style" => opt_enum!(style, border_block_start_style, value),
        "border_block_start_width" => opt_unit!(style, border_block_start_width, value),
        "border_block_start_color" => opt_color!(style, border_block_start_color, value),
        "border_block_end_style" => opt_enum!(style, border_block_end_style, value),
        "border_block_end_width" => opt_unit!(style, border_block_end_width, value),
        "border_block_end_color" => opt_color!(style, border_block_end_color, value),
        "border_radius" => vec_unit!(style, border_radius, value),
        "border_top_left_radius" => opt_unit!(style, border_top_left_radius, value),
        "border_top_right_radius" => opt_unit!(style, border_top_right_radius, value),
        "border_bottom_right_radius" => opt_unit!(style, border_bottom_right_radius, value),
        "border_bottom_left_radius" => opt_unit!(style, border_bottom_left_radius, value),
        "border_collapse" => opt_enum!(style, border_collapse, value),
        "border_spacing" => vec_unit!(style, border_spacing, value),
        "border_image" => style.border_image = split_space(value),
        "border_image_source" => {
            style.border_image_source = parse_css_image(value);
        }
        "border_image_slice" => opt_str!(style, border_image_slice, value),
        "border_image_width" => opt_str!(style, border_image_width, value),
        "border_image_outset" => opt_str!(style, border_image_outset, value),
        "border_image_repeat" => opt_str!(style, border_image_repeat, value),
        "outline_style" => opt_enum!(style, outline_style, value),
        "outline_width" => opt_unit!(style, outline_width, value),
        "outline_color" => opt_color!(style, outline_color, value),
        "outline_offset" => opt_unit!(style, outline_offset, value),
        "outline" => style.outline = split_space(value),

        // ==================================================================
        // VISUAL EFFECTS
        // ==================================================================
        "box_shadow" => {
            style.box_shadow = parse_box_shadow_list(value);
        }
        "filter" => {
            style.filter = parse_filter_list(value);
        }
        "backdrop_filter" => {
            style.backdrop_filter = parse_filter_list(value);
        }
        "mix_blend_mode" => opt_enum!(style, mix_blend_mode, value),
        "isolation" => opt_enum!(style, isolation, value),
        "clip_path" => opt_str!(style, clip_path, value),
        "mask_image" => {
            style.mask_image = parse_css_image_list(value);
        }
        "mask_mode" => opt_str!(style, mask_mode, value),
        "mask_repeat" => opt_enum!(style, mask_repeat, value),
        "mask_position" => {
            style.mask_position = parse_position_2d_list(value);
        }
        "mask_clip" => opt_enum!(style, mask_clip, value),
        "mask_origin" => opt_enum!(style, mask_origin, value),
        "mask_size" => {
            if let Some(s) = parse_background_size(value) {
                style.mask_size = Some(s);
            }
        }
        "mask_composite" => opt_str!(style, mask_composite, value),
        "mask" => style.mask = split_space(value),

        // ==================================================================
        // TRANSFORMS & PERSPECTIVE
        // ==================================================================
        "transform" => {
            style.transform = parse_transform_list(value);
        }
        "transform_origin" => {
            if let Some(p) = parse_position_2d(value) {
                style.transform_origin = Some(p);
            }
        }
        "transform_style" => opt_enum!(style, transform_style, value),
        "transform_box" => opt_enum!(style, transform_box, value),
        "perspective" => opt_unit!(style, perspective, value),
        "perspective_origin" => {
            if let Some(p) = parse_position_2d(value) {
                style.perspective_origin = Some(p);
            }
        }
        "backface_visibility" => opt_enum!(style, backface_visibility, value),
        "rotate" => opt_f32!(style, rotate, value),
        "scale" => {
            let parts = split_space(value);
            if parts.len() == 1 {
                if let Ok(v) = parts[0].parse::<f32>() {
                    style.scale = Some((v, v));
                }
            } else if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    style.scale = Some((x, y));
                }
            }
        }
        "translate" => {
            let parts = split_space(value);
            if parts.len() == 2 {
                style.translate = Some((
                    UnitValue::new(parts[0].as_str()),
                    UnitValue::new(parts[1].as_str()),
                ));
            }
        }

        // ==================================================================
        // TRANSITIONS & ANIMATIONS
        // ==================================================================
        "transition_property" => opt_str!(style, transition_property, value),
        "transition_duration" => opt_f32!(style, transition_duration, value),
        "transition_timing_function" => opt_str!(style, transition_timing_function, value),
        "transition_delay" => opt_f32!(style, transition_delay, value),
        "transition" => {
            style.transition = parse_transition_list(value);
        }
        "animation_name" => opt_str!(style, animation_name, value),
        "animation_duration" => opt_f32!(style, animation_duration, value),
        "animation_timing_function" => opt_str!(style, animation_timing_function, value),
        "animation_delay" => opt_f32!(style, animation_delay, value),
        "animation_iteration_count" => opt_str!(style, animation_iteration_count, value),
        "animation_direction" => opt_str!(style, animation_direction, value),
        "animation_fill_mode" => opt_str!(style, animation_fill_mode, value),
        "animation_play_state" => opt_str!(style, animation_play_state, value),
        "animation" => {
            style.animation = parse_animation_list(value);
        }

        // ==================================================================
        // LISTS
        // ==================================================================
        "list_style_type" => opt_str!(style, list_style_type, value),
        "list_style_position" => opt_enum!(style, list_style_position, value),
        "list_style_image" => {
            style.list_style_image = parse_css_image(value);
        }
        "list_style" => style.list_style = split_space(value),

        // ==================================================================
        // TABLES
        // ==================================================================
        "table_layout" => opt_enum!(style, table_layout, value),
        "caption_side" => opt_enum!(style, caption_side, value),
        "empty_cells" => opt_enum!(style, empty_cells, value),

        // ==================================================================
        // INTERACTION & UI
        // ==================================================================
        "cursor" => opt_enum!(style, cursor, value),
        "pointer_events" => opt_enum!(style, pointer_events, value),
        "user_select" => opt_enum!(style, user_select, value),
        "touch_action" => opt_str!(style, touch_action, value),
        "resize" => opt_enum!(style, resize, value),
        "appearance" => opt_enum!(style, appearance, value),
        "caret_color" => opt_color!(style, caret_color, value),
        "caret_shape" => opt_str!(style, caret_shape, value),
        "accent_color" => opt_color!(style, accent_color, value),
        "color_scheme" => opt_enum!(style, color_scheme, value),
        "forced_color_adjust" => opt_str!(style, forced_color_adjust, value),
        "print_color_adjust" => opt_str!(style, print_color_adjust, value),
        "scroll_behavior" => opt_enum!(style, scroll_behavior, value),
        "overscroll_behavior" => opt_enum!(style, overscroll_behavior, value),
        "overscroll_behavior_x" => opt_enum!(style, overscroll_behavior_x, value),
        "overscroll_behavior_y" => opt_enum!(style, overscroll_behavior_y, value),
        "scrollbar_width" => opt_enum!(style, scrollbar_width, value),
        "scrollbar_gutter" => opt_enum!(style, scrollbar_gutter, value),
        "scrollbar_color" => {
            let parts = split_space(value);
            if parts.len() == 2 {
                style.scrollbar_color = Some((parse_color(&parts[0]), parse_color(&parts[1])));
            }
        }
        "scroll_snap_type" => {
            let parts = split_space(value);
            if let Some(axis_str) = parts.first() {
                if let Ok(axis) = axis_str.parse::<ScrollSnapTypeAxis>() {
                    let strictness = parts
                        .get(1)
                        .and_then(|s| s.parse::<ScrollSnapStrictness>().ok())
                        .unwrap_or(ScrollSnapStrictness::Proximity);
                    style.scroll_snap_type = Some((axis, strictness));
                }
            }
        }
        "scroll_snap_align" => opt_enum!(style, scroll_snap_align, value),
        "scroll_snap_stop" => opt_enum!(style, scroll_snap_stop, value),
        "scroll_margin" => vec_unit!(style, scroll_margin, value),
        "scroll_padding" => vec_unit!(style, scroll_padding, value),

        // ==================================================================
        // MULTI-COLUMN / FRAGMENTATION
        // ==================================================================
        "columns" => style.columns = split_space(value),
        "column_count" => {
            if let Ok(v) = value.parse::<u32>() {
                style.column_count = Some(v);
            }
        }
        "column_width" => opt_unit!(style, column_width, value),
        "column_rule_style" => opt_enum!(style, column_rule_style, value),
        "column_rule_width" => opt_unit!(style, column_rule_width, value),
        "column_rule_color" => opt_color!(style, column_rule_color, value),
        "column_rule" => style.column_rule = split_space(value),
        "column_span" => opt_str!(style, column_span, value),
        "column_fill" => opt_str!(style, column_fill, value),
        "break_before" => opt_enum!(style, break_before, value),
        "break_after" => opt_enum!(style, break_after, value),
        "break_inside" => opt_enum!(style, break_inside, value),

        // ==================================================================
        // CONTAINMENT & MISC
        // ==================================================================
        "contain" => {
            style.contain = split_space(value)
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
        }
        "content_visibility" => opt_str!(style, content_visibility, value),
        "container_name" => opt_str!(style, container_name, value),
        "container_type" => opt_str!(style, container_type, value),
        "container" => style.container = split_space(value),
        "will_change" => opt_str!(style, will_change, value),
        "content" => opt_str!(style, content, value),
        "quotes" => opt_str!(style, quotes, value),
        "counter_reset" => opt_str!(style, counter_reset, value),
        "counter_increment" => opt_str!(style, counter_increment, value),
        "counter_set" => opt_str!(style, counter_set, value),
        "all" => opt_enum!(style, all, value),
        "zoom" => opt_f32!(style, zoom, value),
        "image_rendering" => opt_enum!(style, image_rendering, value),
        "color_interpolation" => opt_str!(style, color_interpolation, value),

        // Unknown properties are silently ignored.
        _ => {}
    }
}

// ============================================================================
// Tests — unchanged from your version
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(css: &str) -> Style {
        parse_style(css)
    }

    fn px(v: f32) -> UnitValue {
        UnitValue::px(v)
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {b}, got {a}");
    }

    fn unwrap_UnitValue(s: &Option<SizeValue>) -> UnitValue {
        match s {
            Some(SizeValue::UnitValue(u)) => u.clone(),
            other => panic!("expected SizeValue::UnitValue, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Declaration scanner
    // ------------------------------------------------------------------

    #[test]
    fn empty_input_returns_default_style() {
        let s = parse("");
        assert!(s.display.is_empty());
        assert!(s.width.is_none());
        assert!(s.color.is_none());
    }

    #[test]
    fn whitespace_only_input_returns_default() {
        let s = parse("   \n\t  \r\n  ");
        assert!(s.width.is_none());
    }

    #[test]
    fn single_declaration() {
        let s = parse("width: 100px;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
    }

    #[test]
    fn trailing_declaration_without_semicolon() {
        let s = parse("width: 100px");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
    }

    #[test]
    fn multiple_declarations() {
        let s = parse("width: 100px; height: 50px; color: red;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(50.0))));
        assert_eq!(s.color, Some(Color::Keyword("red".into())));
    }

    #[test]
    fn missing_value_is_ignored() {
        let s = parse("width: ; height: 10px;");
        assert!(s.width.is_none());
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(10.0))));
    }

    #[test]
    fn missing_colon_means_no_property() {
        let s = parse("width 100px; height: 10px;");
        assert!(s.width.is_none());
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(10.0))));
    }

    #[test]
    fn extra_semicolons_are_tolerated() {
        let s = parse(";;; width: 10px;;; height: 20px;;;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(20.0))));
    }

    #[test]
    fn block_comment_is_skipped() {
        let s = parse("/* hello */ width: 10px; /* world */");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
    }

    #[test]
    fn multi_line_block_comment_is_skipped() {
        let s = parse("/* line 1\nline 2\nline 3 */ height: 20px;");
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(20.0))));
    }

    #[test]
    fn line_comment_is_skipped() {
        let s = parse("// a comment\nwidth: 10px;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
    }

    #[test]
    fn inline_comments_between_declarations() {
        let s = parse("width: 10px; /* between */ height: 20px; // trailing\npadding: 5px;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(20.0))));
        assert_eq!(s.padding, vec![px(5.0)]);
    }

    #[test]
    fn unterminated_block_comment_swallows_rest() {
        let s = parse("width: 10px; /* unterminated height: 20px;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
        assert!(s.height.is_none());
    }

    #[test]
    fn important_suffix_is_stripped() {
        let s = parse("width: 100px !important;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
    }

    #[test]
    fn important_without_space_is_stripped() {
        let s = parse("width: 100px!important;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
    }

    #[test]
    fn quoted_colon_does_not_break_name() {
        let s = parse(r#"grid-template-areas: "a:b" "c d";"#);
        assert_eq!(s.grid_template_areas, Some(r#""a:b" "c d""#.to_string()));
    }

    #[test]
    fn semicolon_inside_string_is_not_terminator() {
        let s = parse(r#"content: "a;b";"#);
        assert_eq!(s.content, Some(r#""a;b""#.to_string()));
    }

    #[test]
    fn semicolon_inside_parens_is_not_terminator() {
        let s = parse("background: url(a;b) red;");
        assert_eq!(
            s.background,
            vec!["url(a;b)".to_string(), "red".to_string()],
        );
    }

    // ------------------------------------------------------------------
    // Enums
    // ------------------------------------------------------------------

    #[test]
    fn enum_box_sizing() {
        assert_eq!(
            parse("box-sizing: border-box;").box_sizing,
            Some(BoxSizing::BorderBox)
        );
        assert_eq!(
            parse("box-sizing: content-box;").box_sizing,
            Some(BoxSizing::ContentBox)
        );
    }

    #[test]
    fn enum_position() {
        assert_eq!(
            parse("position: absolute;").position,
            Some(Position::Absolute)
        );
    }

    #[test]
    fn enum_overflow() {
        let s = parse("overflow: hidden; overflow-x: auto; overflow-y: scroll;");
        assert_eq!(s.overflow, Some(Overflow::Hidden));
        assert_eq!(s.overflow_x, Some(Overflow::Auto));
        assert_eq!(s.overflow_y, Some(Overflow::Scroll));
    }

    #[test]
    fn enum_flex_direction() {
        assert_eq!(
            parse("flex-direction: column-reverse;").flex_direction,
            Some(FlexDirection::ColumnReverse)
        );
    }

    #[test]
    fn enum_invalid_value_leaves_field_none() {
        let s = parse("box-sizing: garbage; position: nonsense;");
        assert!(s.box_sizing.is_none());
        assert!(s.position.is_none());
    }

    #[test]
    fn enum_case_sensitive_for_values() {
        assert!(parse("box-sizing: BORDER-BOX;").box_sizing.is_none());
    }

    #[test]
    fn display_sets_vec_of_one() {
        assert_eq!(parse("display: flex;").display, vec![Display::Flex]);
    }

    #[test]
    fn display_invalid_leaves_empty_vec() {
        assert!(parse("display: garbage;").display.is_empty());
    }

    // ------------------------------------------------------------------
    // Units
    // ------------------------------------------------------------------

    #[test]
    fn unit_single_value() {
        assert_eq!(parse("padding: 20px;").padding, vec![px(20.0)]);
    }

    #[test]
    fn unit_multiple_values() {
        assert_eq!(
            parse("padding: 20px 10px 5px 15px;").padding,
            vec![px(20.0), px(10.0), px(5.0), px(15.0)]
        );
    }

    #[test]
    fn unit_shorthand_named_fields() {
        let s = parse("padding-top: 1rem; padding-bottom: 2em;");
        assert_eq!(s.padding_top, Some(UnitValue::rem(1.0)));
        assert_eq!(s.padding_bottom, Some(UnitValue::em(2.0)));
    }

    #[test]
    fn unit_calc_is_parsed_through_unit_value() {
        let s = parse("width: calc(100% - 20px);");
        let uv = unwrap_UnitValue(&s.width);
        assert_eq!(uv.to_px(), None);
        approx(uv.with_context(800.0, 16.0, 16.0).to_px().unwrap(), 780.0);
    }

    #[test]
    fn unit_clamp_in_padding() {
        let s = parse("padding: clamp(1rem, 2.5vw, 2rem);");
        assert_eq!(s.padding.len(), 1);
        approx(
            s.padding[0]
                .clone()
                .with_viewport(1000.0, 600.0)
                .to_px()
                .unwrap(),
            25.0,
        );
    }

    // ------------------------------------------------------------------
    // SizeValue
    // ------------------------------------------------------------------

    #[test]
    fn size_auto() {
        assert_eq!(parse("width: auto;").width, Some(SizeValue::Auto));
    }

    #[test]
    fn size_percent() {
        let uv = unwrap_UnitValue(&parse("width: 50%;").width);
        approx(uv.with_context(200.0, 16.0, 16.0).to_px().unwrap(), 100.0);
    }

    #[test]
    fn size_min_content() {
        assert_eq!(
            parse("min-width: min-content;").min_width,
            Some(SizeValue::MinContent)
        );
    }

    #[test]
    fn size_multiple_margin() {
        assert_eq!(
            parse("margin: 10px auto;").margin,
            vec![SizeValue::UnitValue(px(10.0)), SizeValue::Auto]
        );
    }

    // ------------------------------------------------------------------
    // Color
    // ------------------------------------------------------------------

    #[test]
    fn color_hex_short() {
        assert_eq!(parse("color: #f00;").color, Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn color_hex_long() {
        assert_eq!(parse("color: #ff0000;").color, Some(Color::rgb(255, 0, 0)));
    }

    #[test]
    fn color_hex_with_alpha() {
        assert_eq!(
            parse("color: #ff000080;").color,
            Some(Color::rgba(255, 0, 0, 128))
        );
    }

    #[test]
    fn color_named_keywords() {
        assert_eq!(parse("color: black;").color, Some(Color::BLACK));
        assert_eq!(parse("color: white;").color, Some(Color::WHITE));
        assert_eq!(parse("color: transparent;").color, Some(Color::TRANSPARENT));
    }

    #[test]
    fn color_current_color() {
        assert_eq!(
            parse("color: currentcolor;").color,
            Some(Color::CURRENT_COLOR)
        );
        assert_eq!(
            parse("color: currentColor;").color,
            Some(Color::CURRENT_COLOR)
        );
    }

    #[test]
    fn color_other_named_becomes_keyword() {
        assert_eq!(
            parse("color: rebeccapurple;").color,
            Some(Color::Keyword("rebeccapurple".into()))
        );
    }

    #[test]
    fn color_function_preserved_as_keyword() {
        assert_eq!(
            parse("color: rgba(0, 0, 0, 0.4);").color,
            Some(Color::Keyword("rgba(0, 0, 0, 0.4)".into()))
        );
    }

    #[test]
    fn color_short_property_names() {
        let s = parse("background-color: #fff; border-color: #000;");
        assert_eq!(s.background_color, Some(Color::WHITE));
        assert_eq!(s.border_color, Some(Color::BLACK));
    }

    // ------------------------------------------------------------------
    // LineHeight
    // ------------------------------------------------------------------

    #[test]
    fn line_height_normal() {
        assert_eq!(
            parse("line-height: normal;").line_height,
            Some(LineHeight::Normal)
        );
    }

    #[test]
    fn line_height_number() {
        assert_eq!(
            parse("line-height: 1.5;").line_height,
            Some(LineHeight::Number(1.5))
        );
    }

    #[test]
    fn line_height_UnitValue() {
        match parse("line-height: 24px;").line_height {
            Some(LineHeight::UnitValue(uv)) => approx(uv.to_px().unwrap(), 24.0),
            other => panic!("expected LineHeight::UnitValue, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // f32 / i32
    // ------------------------------------------------------------------

    #[test]
    fn f32_opacity() {
        assert_eq!(parse("opacity: 0.5;").opacity, Some(0.5));
    }

    #[test]
    fn f32_zero() {
        assert_eq!(parse("opacity: 0;").opacity, Some(0.0));
    }

    #[test]
    fn f32_invalid_leaves_none() {
        assert!(parse("opacity: half;").opacity.is_none());
    }

    #[test]
    fn i32_z_index() {
        assert_eq!(parse("z-index: 10;").z_index, Some(10));
    }

    #[test]
    fn i32_negative() {
        assert_eq!(parse("z-index: -1;").z_index, Some(-1));
    }

    #[test]
    fn i32_invalid_leaves_none() {
        assert!(parse("z-index: high;").z_index.is_none());
    }

    #[test]
    fn f32_flex_grow_shrink() {
        let s = parse("flex-grow: 1; flex-shrink: 0.5;");
        assert_eq!(s.flex_grow, Some(1.0));
        assert_eq!(s.flex_shrink, Some(0.5));
    }

    // ------------------------------------------------------------------
    // Option<String>
    // ------------------------------------------------------------------

    #[test]
    fn string_grid_template() {
        assert_eq!(
            parse("grid-template-columns: 1fr 2fr;").grid_template_columns,
            Some("1fr 2fr".to_string())
        );
    }

    #[test]
    fn string_grid_template_with_function() {
        assert_eq!(
            parse("grid-template-columns: repeat(3, 1fr);").grid_template_columns,
            Some("repeat(3, 1fr)".to_string())
        );
    }

    #[test]
    fn string_preserves_quotes() {
        assert_eq!(
            parse(r#"grid-template-areas: "a b" "c d";"#).grid_template_areas,
            Some(r#""a b" "c d""#.to_string())
        );
    }

    // ------------------------------------------------------------------
    // FontFamily
    // ------------------------------------------------------------------

    #[test]
    fn font_family_comma_separated() {
        assert_eq!(
            parse(r#"font-family: "Inter", "Helvetica Neue", sans-serif;"#).font_family,
            Some(vec![
                "Inter".to_string(),
                "Helvetica Neue".to_string(),
                "sans-serif".to_string(),
            ])
        );
    }

    #[test]
    fn font_family_single() {
        assert_eq!(
            parse("font-family: serif;").font_family,
            Some(vec!["serif".to_string()])
        );
    }

    // ------------------------------------------------------------------
    // Special cases
    // ------------------------------------------------------------------

    #[test]
    fn aspect_ratio_with_slash() {
        assert_eq!(
            parse("aspect-ratio: 16 / 9;").aspect_ratio,
            Some(AspectRatio::new(16.0, 9.0))
        );
    }

    #[test]
    fn aspect_ratio_single_value() {
        assert_eq!(
            parse("aspect-ratio: 2;").aspect_ratio,
            Some(AspectRatio::new(2.0, 1.0))
        );
    }

    #[test]
    fn aspect_ratio_invalid_leaves_none() {
        assert!(parse("aspect-ratio: garbage;").aspect_ratio.is_none());
    }

    #[test]
    fn scale_one_argument() {
        assert_eq!(parse("scale: 1.5;").scale, Some((1.5, 1.5)));
    }

    #[test]
    fn scale_two_arguments() {
        assert_eq!(parse("scale: 2 0.5;").scale, Some((2.0, 0.5)));
    }

    #[test]
    fn scale_invalid_leaves_none() {
        assert!(parse("scale: big;").scale.is_none());
    }

    #[test]
    fn translate_two_units() {
        let (x, y) = parse("translate: 10px 20px;").translate.unwrap();
        approx(x.to_px().unwrap(), 10.0);
        approx(y.to_px().unwrap(), 20.0);
    }

    #[test]
    fn translate_requires_two_arguments() {
        assert!(parse("translate: 10px;").translate.is_none());
    }

    #[test]
    fn contain_multiple_values() {
        assert_eq!(
            parse("contain: size layout paint;").contain,
            vec![Contain::Size, Contain::Layout, Contain::Paint]
        );
    }

    #[test]
    fn contain_invalid_values_filtered_out() {
        assert_eq!(
            parse("contain: garbage size nonsense;").contain,
            vec![Contain::Size]
        );
    }

    // ------------------------------------------------------------------
    // Flex / grid shorthands
    // ------------------------------------------------------------------

    #[test]
    fn flex_shorthand_stored_as_vec() {
        assert_eq!(
            parse("flex: 1 0 auto;").flex,
            vec!["1".to_string(), "0".to_string(), "auto".to_string()]
        );
    }

    #[test]
    fn flex_flow_shorthand() {
        assert_eq!(
            parse("flex-flow: row wrap;").flex_flow,
            vec!["row".to_string(), "wrap".to_string()]
        );
    }

    #[test]
    fn grid_area_shorthand() {
        let s = parse("grid-area: 1 / 2 / 3 / 4;");
        assert_eq!(s.grid_area.len(), 7);
        assert_eq!(s.grid_area[0], "1");
        assert_eq!(s.grid_area[1], "/");
    }

    #[test]
    fn background_shorthand_splits_on_whitespace() {
        assert_eq!(
            parse("background: red url(x.png) no-repeat fixed;").background,
            vec![
                "red".to_string(),
                "url(x.png)".to_string(),
                "no-repeat".to_string(),
                "fixed".to_string(),
            ]
        );
    }

    // ------------------------------------------------------------------
    // Vec<UnitValue>
    // ------------------------------------------------------------------

    #[test]
    fn border_radius_four_values() {
        assert_eq!(
            parse("border-radius: 4px 8px 4px 8px;").border_radius,
            vec![px(4.0), px(8.0), px(4.0), px(8.0)]
        );
    }

    #[test]
    fn border_spacing_two_values() {
        assert_eq!(
            parse("border-spacing: 2px 4px;").border_spacing,
            vec![px(2.0), px(4.0)]
        );
    }

    #[test]
    fn gap_multiple() {
        assert_eq!(parse("gap: 8px 16px;").gap, vec![px(8.0), px(16.0)]);
    }

    #[test]
    fn scroll_margin_two_values() {
        assert_eq!(
            parse("scroll-margin: 10px 20px;").scroll_margin,
            vec![px(10.0), px(20.0)]
        );
    }

    // ------------------------------------------------------------------
    // Unknown / ignored
    // ------------------------------------------------------------------

    #[test]
    fn unknown_property_is_ignored() {
        assert_eq!(
            parse("my-invented-property: whatever; width: 10px;").width,
            Some(SizeValue::UnitValue(px(10.0)))
        );
    }

    #[test]
    fn unknown_property_does_not_touch_other_fields() {
        let s = parse("my-invented-property: whatever; width: 10px;");
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(10.0))));
        assert!(s.box_shadow.is_empty());
        assert!(s.transform.is_empty());
    }

    // ------------------------------------------------------------------
    // GridLine
    // ------------------------------------------------------------------

    #[test]
    fn grid_line_numeric() {
        let s = parse("grid-column-start: 2; grid-row-end: 4;");
        assert_eq!(s.grid_column_start, Some(GridLine::Line(2)));
        assert_eq!(s.grid_row_end, Some(GridLine::Line(4)));
    }

    #[test]
    fn grid_line_auto() {
        assert_eq!(
            parse("grid-column-start: auto;").grid_column_start,
            Some(GridLine::Auto)
        );
    }

    #[test]
    fn grid_line_span() {
        assert_eq!(
            parse("grid-column-end: span 3;").grid_column_end,
            Some(GridLine::Span(3))
        );
    }

    #[test]
    fn grid_line_named() {
        assert_eq!(
            parse("grid-row-start: sidebar;").grid_row_start,
            Some(GridLine::Named("sidebar".into()))
        );
    }

    // ------------------------------------------------------------------
    // CssImage
    // ------------------------------------------------------------------

    #[test]
    fn css_image_none() {
        assert_eq!(
            parse("background-image: none;").background_image,
            vec![CssImage::None]
        );
    }

    #[test]
    fn css_image_url_double_quotes() {
        assert_eq!(
            parse(r#"background-image: url("cat.png");"#).background_image,
            vec![CssImage::Url("cat.png".into())]
        );
    }

    #[test]
    fn css_image_url_single_quotes() {
        assert_eq!(
            parse("background-image: url('cat.png');").background_image,
            vec![CssImage::Url("cat.png".into())]
        );
    }

    #[test]
    fn css_image_url_no_quotes() {
        assert_eq!(
            parse("background-image: url(cat.png);").background_image,
            vec![CssImage::Url("cat.png".into())]
        );
    }

    #[test]
    fn css_image_multiple_layers() {
        assert_eq!(
            parse("background-image: url(a.png), url(b.png);").background_image,
            vec![CssImage::Url("a.png".into()), CssImage::Url("b.png".into()),]
        );
    }

    #[test]
    fn border_image_source_and_list_style_image() {
        let s = parse("border-image-source: url(border.png); list-style-image: url(dot.png);");
        assert_eq!(
            s.border_image_source,
            Some(CssImage::Url("border.png".into()))
        );
        assert_eq!(s.list_style_image, Some(CssImage::Url("dot.png".into())));
    }

    #[test]
    fn mask_image_list() {
        assert_eq!(
            parse("mask-image: url(m1.svg), url(m2.svg);")
                .mask_image
                .len(),
            2
        );
    }

    // ------------------------------------------------------------------
    // Position2D
    // ------------------------------------------------------------------

    #[test]
    fn position_2d_single_keyword() {
        assert_eq!(
            parse("object-position: center;").object_position,
            Some(Position2D::Keywords(
                HorizontalKeyword::Center,
                VerticalKeyword::Center,
            ))
        );
    }

    #[test]
    fn position_2d_two_keywords() {
        assert_eq!(
            parse("object-position: right top;").object_position,
            Some(Position2D::Keywords(
                HorizontalKeyword::Right,
                VerticalKeyword::Top,
            ))
        );
    }

    #[test]
    fn position_2d_two_UnitValues() {
        match parse("object-position: 10px 20px;").object_position {
            Some(Position2D::XY(x, y)) => {
                approx(x.to_px().unwrap(), 10.0);
                approx(y.to_px().unwrap(), 20.0);
            }
            other => panic!("expected XY, got {other:?}"),
        }
    }

    #[test]
    fn position_2d_single_UnitValue() {
        match parse("object-position: 30%;").object_position {
            Some(Position2D::X(x)) => approx(x.to_percent().unwrap(), 30.0),
            other => panic!("expected X, got {other:?}"),
        }
    }

    #[test]
    fn background_position_list() {
        assert_eq!(
            parse("background-position: center, 10px 20px;")
                .background_position
                .len(),
            2
        );
    }

    #[test]
    fn transform_origin_2d() {
        assert_eq!(
            parse("transform-origin: left top;").transform_origin,
            Some(Position2D::Keywords(
                HorizontalKeyword::Left,
                VerticalKeyword::Top,
            ))
        );
    }

    #[test]
    fn mask_position_list() {
        assert_eq!(
            parse("mask-position: center, 50% 50%;").mask_position.len(),
            2
        );
    }

    // ------------------------------------------------------------------
    // BackgroundSize
    // ------------------------------------------------------------------

    #[test]
    fn background_size_cover() {
        assert_eq!(
            parse("background-size: cover;").background_size,
            Some(BackgroundSize::Cover)
        );
    }

    #[test]
    fn background_size_contain() {
        assert_eq!(
            parse("background-size: contain;").background_size,
            Some(BackgroundSize::Contain)
        );
    }

    #[test]
    fn background_size_explicit_two_values() {
        match parse("background-size: 100px 50px;").background_size {
            Some(BackgroundSize::Explicit(w, h)) => match (w, h) {
                (SizeValue::UnitValue(w), SizeValue::UnitValue(h)) => {
                    approx(w.to_px().unwrap(), 100.0);
                    approx(h.to_px().unwrap(), 50.0);
                }
                _ => panic!("expected two UnitValues"),
            },
            other => panic!("expected Explicit, got {other:?}"),
        }
    }

    #[test]
    fn mask_size_cover() {
        assert_eq!(
            parse("mask-size: cover;").mask_size,
            Some(BackgroundSize::Cover)
        );
    }

    // ------------------------------------------------------------------
    // BoxShadow / TextShadow
    // ------------------------------------------------------------------

    #[test]
    fn box_shadow_minimal() {
        let s = parse("box-shadow: 0 0 10px;");
        assert_eq!(s.box_shadow.len(), 1);
        let sh = &s.box_shadow[0];
        approx(sh.offset_x.to_px().unwrap(), 0.0);
        approx(sh.offset_y.to_px().unwrap(), 0.0);
        approx(sh.blur.to_px().unwrap(), 10.0);
        approx(sh.spread.to_px().unwrap(), 0.0);
        assert_eq!(sh.color, Color::BLACK);
        assert!(!sh.inset);
    }

    #[test]
    fn box_shadow_full() {
        let s = parse("box-shadow: 2px 4px 8px 1px #ff0000 inset;");
        let sh = &s.box_shadow[0];
        approx(sh.offset_x.to_px().unwrap(), 2.0);
        approx(sh.offset_y.to_px().unwrap(), 4.0);
        approx(sh.blur.to_px().unwrap(), 8.0);
        approx(sh.spread.to_px().unwrap(), 1.0);
        assert_eq!(sh.color, Color::rgb(255, 0, 0));
        assert!(sh.inset);
    }

    #[test]
    fn box_shadow_color_first() {
        let s = parse("box-shadow: red 0 0 5px;");
        let sh = &s.box_shadow[0];
        assert_eq!(sh.color, Color::Keyword("red".into()));
        approx(sh.blur.to_px().unwrap(), 5.0);
    }

    #[test]
    fn box_shadow_multiple() {
        assert_eq!(
            parse("box-shadow: 0 0 5px black, 0 2px 10px rgba(0,0,0,0.5);")
                .box_shadow
                .len(),
            2
        );
    }

    #[test]
    fn text_shadow_basic() {
        let s = parse("text-shadow: 1px 2px 3px #000000;");
        let sh = &s.text_shadow[0];
        approx(sh.offset_x.to_px().unwrap(), 1.0);
        approx(sh.offset_y.to_px().unwrap(), 2.0);
        approx(sh.blur.to_px().unwrap(), 3.0);
        assert_eq!(sh.color, Color::BLACK);
    }

    #[test]
    fn text_shadow_multiple() {
        assert_eq!(
            parse("text-shadow: 1px 1px red, 2px 2px blue;")
                .text_shadow
                .len(),
            2
        );
    }

    // ------------------------------------------------------------------
    // Filter
    // ------------------------------------------------------------------

    #[test]
    fn filter_none() {
        assert!(parse("filter: none;").filter.is_empty());
    }

    #[test]
    fn filter_single() {
        let s = parse("filter: blur(5px);");
        assert_eq!(s.filter.len(), 1);
        match &s.filter[0] {
            FilterFunction::Blur(u) => approx(u.to_px().unwrap(), 5.0),
            other => panic!("expected Blur, got {other:?}"),
        }
    }

    #[test]
    fn filter_multiple() {
        let s = parse("filter: brightness(0.5) contrast(150%) saturate(2);");
        assert_eq!(s.filter.len(), 3);
        match &s.filter[0] {
            FilterFunction::Brightness(b) => approx(*b, 0.5),
            other => panic!("expected Brightness, got {other:?}"),
        }
        match &s.filter[1] {
            FilterFunction::Contrast(c) => approx(*c, 1.5),
            other => panic!("expected Contrast, got {other:?}"),
        }
    }

    #[test]
    fn filter_hue_rotate_deg() {
        match &parse("filter: hue-rotate(90deg);").filter[0] {
            FilterFunction::HueRotate(d) => approx(*d, 90.0),
            other => panic!("expected HueRotate, got {other:?}"),
        }
    }

    #[test]
    fn filter_drop_shadow() {
        assert!(matches!(
            parse("filter: drop-shadow(0 2px 4px #000);").filter[0],
            FilterFunction::DropShadow(_)
        ));
    }

    #[test]
    fn backdrop_filter_is_parsed() {
        assert_eq!(
            parse("backdrop-filter: blur(8px);").backdrop_filter.len(),
            1
        );
    }

    // ------------------------------------------------------------------
    // Transform
    // ------------------------------------------------------------------

    #[test]
    fn transform_none() {
        assert!(parse("transform: none;").transform.is_empty());
    }

    #[test]
    fn transform_translate() {
        match &parse("transform: translate(10px, 20px);").transform[0] {
            TransformFunction::Translate(x, y) => {
                approx(x.to_px().unwrap(), 10.0);
                approx(y.to_px().unwrap(), 20.0);
            }
            other => panic!("expected Translate, got {other:?}"),
        }
    }

    #[test]
    fn transform_rotate_deg() {
        match &parse("transform: rotate(45deg);").transform[0] {
            TransformFunction::Rotate(d) => approx(*d, 45.0),
            other => panic!("expected Rotate, got {other:?}"),
        }
    }

    #[test]
    fn transform_rotate_turn() {
        match &parse("transform: rotate(0.5turn);").transform[0] {
            TransformFunction::Rotate(d) => approx(*d, 180.0),
            other => panic!("expected Rotate, got {other:?}"),
        }
    }

    #[test]
    fn transform_scale_one_arg() {
        match &parse("transform: scale(1.5);").transform[0] {
            TransformFunction::Scale(x, y) => {
                approx(*x, 1.5);
                approx(*y, 1.5);
            }
            other => panic!("expected Scale, got {other:?}"),
        }
    }

    #[test]
    fn transform_scale_two_args() {
        match &parse("transform: scale(2, 0.5);").transform[0] {
            TransformFunction::Scale(x, y) => {
                approx(*x, 2.0);
                approx(*y, 0.5);
            }
            other => panic!("expected Scale, got {other:?}"),
        }
    }

    #[test]
    fn transform_chain_without_spaces() {
        let s = parse("transform: translate(10px)rotate(45deg);");
        assert_eq!(s.transform.len(), 2);
        assert!(matches!(s.transform[0], TransformFunction::Translate(_, _)));
        assert!(matches!(s.transform[1], TransformFunction::Rotate(_)));
    }

    #[test]
    fn transform_matrix() {
        match &parse("transform: matrix(1, 0, 0, 1, 10, 20);").transform[0] {
            TransformFunction::Matrix(a, _b, _c, _d, _e, f) => {
                approx(*a, 1.0);
                approx(*f, 20.0);
            }
            other => panic!("expected Matrix, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Transition
    // ------------------------------------------------------------------

    #[test]
    fn transition_full() {
        let s = parse("transition: opacity 0.3s ease 0.1s;");
        let t = &s.transition[0];
        assert_eq!(t.property, "opacity");
        approx(t.duration, 0.3);
        assert_eq!(t.timing, "ease");
        approx(t.delay, 0.1);
    }

    #[test]
    fn transition_ms() {
        let s = parse("transition: color 300ms;");
        approx(s.transition[0].duration, 0.3);
        assert_eq!(s.transition[0].property, "color");
        assert_eq!(s.transition[0].timing, "ease");
    }

    #[test]
    fn transition_multiple() {
        let s = parse("transition: opacity 0.3s ease, transform 0.3s ease-out;");
        assert_eq!(s.transition.len(), 2);
        assert_eq!(s.transition[0].property, "opacity");
        assert_eq!(s.transition[1].property, "transform");
        assert_eq!(s.transition[1].timing, "ease-out");
    }

    #[test]
    fn transition_cubic_bezier() {
        assert_eq!(
            parse("transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);").transition[0].timing,
            "cubic-bezier(0.4, 0, 0.2, 1)"
        );
    }

    // ------------------------------------------------------------------
    // Animation
    // ------------------------------------------------------------------

    #[test]
    fn animation_none() {
        assert!(parse("animation: none;").animation.is_empty());
    }

    #[test]
    fn animation_minimal() {
        let s = parse("animation: pulse 1s;");
        let a = &s.animation[0];
        assert_eq!(a.name, "pulse");
        approx(a.duration, 1.0);
        assert_eq!(a.iteration_count, "1");
        assert_eq!(a.direction, "normal");
        assert_eq!(a.fill_mode, "none");
        assert_eq!(a.play_state, "running");
    }

    #[test]
    fn animation_full() {
        let s = parse("animation: spin 2s linear 0.5s infinite alternate both paused;");
        let a = &s.animation[0];
        assert_eq!(a.name, "spin");
        approx(a.duration, 2.0);
        assert_eq!(a.timing, "linear");
        approx(a.delay, 0.5);
        assert_eq!(a.iteration_count, "infinite");
        assert_eq!(a.direction, "alternate");
        assert_eq!(a.fill_mode, "both");
        assert_eq!(a.play_state, "paused");
    }

    #[test]
    fn animation_multiple() {
        let s = parse("animation: a 1s ease, b 2s linear infinite;");
        assert_eq!(s.animation.len(), 2);
        assert_eq!(s.animation[0].name, "a");
        assert_eq!(s.animation[1].name, "b");
        assert_eq!(s.animation[1].iteration_count, "infinite");
    }

    // ------------------------------------------------------------------
    // Pairs
    // ------------------------------------------------------------------

    #[test]
    fn scrollbar_color_pair() {
        assert_eq!(
            parse("scrollbar-color: #000000 #ffffff;").scrollbar_color,
            Some((Color::BLACK, Color::WHITE))
        );
    }

    #[test]
    fn scrollbar_color_invalid_leaves_none() {
        assert!(parse("scrollbar-color: red;").scrollbar_color.is_none());
    }

    #[test]
    fn scroll_snap_type_axis_and_strictness() {
        assert_eq!(
            parse("scroll-snap-type: x mandatory;").scroll_snap_type,
            Some((ScrollSnapTypeAxis::X, ScrollSnapStrictness::Mandatory))
        );
    }

    #[test]
    fn scroll_snap_type_axis_only() {
        assert_eq!(
            parse("scroll-snap-type: both;").scroll_snap_type,
            Some((ScrollSnapTypeAxis::Both, ScrollSnapStrictness::Proximity))
        );
    }

    #[test]
    fn scroll_snap_type_none() {
        assert_eq!(
            parse("scroll-snap-type: none;").scroll_snap_type,
            Some((ScrollSnapTypeAxis::None, ScrollSnapStrictness::Proximity))
        );
    }

    // ------------------------------------------------------------------
    // Integration
    // ------------------------------------------------------------------

    #[test]
    fn realistic_card_style() {
        let css = r#"
            /* Card layout */
            display: flex;
            flex-direction: column;
            gap: 12px;
            padding: 20px;
            width: calc(100% - 40px);
            max-width: 480px;

            /* Colors */
            color: #222222;
            background-color: #ffffff;
            border-radius: 8px;

            /* Typography */
            font-family: "Inter", sans-serif;
            font-size: 1rem;
            line-height: 1.5;
            text-align: left;

            /* Interaction */
            cursor: pointer;
            transition-duration: 0.2;
            transition-timing-function: ease-out;
        "#;

        let s = parse(css);

        assert_eq!(s.display, vec![Display::Flex]);
        assert_eq!(s.flex_direction, Some(FlexDirection::Column));
        assert_eq!(s.gap, vec![px(12.0)]);
        assert_eq!(s.padding, vec![px(20.0)]);
        assert_eq!(s.max_width, Some(SizeValue::UnitValue(px(480.0))));
        assert_eq!(s.color, Some(Color::hex(0x222222)));
        assert_eq!(s.background_color, Some(Color::hex(0xffffff)));
        assert_eq!(s.border_radius, vec![px(8.0)]);
        assert_eq!(
            s.font_family,
            Some(vec!["Inter".to_string(), "sans-serif".to_string()])
        );
        approx(s.font_size.as_ref().unwrap().to_px().unwrap(), 16.0);
        assert_eq!(s.line_height, Some(LineHeight::Number(1.5)));
        assert_eq!(s.text_align, Some(TextAlign::Left));
        assert_eq!(s.cursor, Some(Cursor::Pointer));
        assert_eq!(s.transition_duration, Some(0.2));
        assert_eq!(s.transition_timing_function, Some("ease-out".to_string()));

        let w = unwrap_UnitValue(&s.width);
        assert_eq!(w.to_px(), None);
        approx(w.with_context(800.0, 16.0, 16.0).to_px().unwrap(), 760.0);
    }

    #[test]
    fn realistic_button_with_shadows_and_transition() {
        let css = r#"
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 12px 24px;
            background-color: #3b82f6;
            color: #ffffff;
            border-radius: 6px;
            box-shadow: 0 1px 2px rgba(0,0,0,0.05), 0 4px 12px rgba(59,130,246,0.3);
            transform: translate(0, 0);
            transition: background-color 0.15s ease, transform 0.15s ease;
        "#;
        let s = parse(css);

        assert_eq!(s.display, vec![Display::Flex]);
        assert_eq!(s.padding, vec![px(12.0), px(24.0)]);
        assert_eq!(s.background_color, Some(Color::hex(0x3b82f6)));
        assert_eq!(s.border_radius, vec![px(6.0)]);
        assert_eq!(s.box_shadow.len(), 2);
        assert_eq!(s.transform.len(), 1);
        assert_eq!(s.transition.len(), 2);
    }

    #[test]
    fn realistic_complex_background() {
        let css = r#"
            background-image: url("hero.jpg"), linear-gradient(...);
            background-size: cover;
            background-position: center;
            background-repeat: no-repeat;
        "#;
        let s = parse(css);
        assert_eq!(s.background_image.len(), 2);
        assert_eq!(s.background_size, Some(BackgroundSize::Cover));
        assert!(matches!(
            s.background_position[0],
            Position2D::Keywords(_, _)
        ));
        assert_eq!(s.background_repeat, Some(BackgroundRepeat::NoRepeat));
    }

    #[test]
    fn calc_min_max_clamp_all_survive_parser() {
        let s = parse(
            r#"
            width: calc(100% - 20px);
            min-height: min(100vh, 500px);
            max-height: max(200px, 50%);
            font-size: clamp(1rem, 2.5vw, 2rem);
        "#,
        );
        assert!(matches!(s.width, Some(SizeValue::UnitValue(_))));
        assert!(matches!(s.min_height, Some(SizeValue::UnitValue(_))));
        assert!(matches!(s.max_height, Some(SizeValue::UnitValue(_))));

        let fs = s.font_size.as_ref().unwrap();
        approx(
            fs.clone().with_viewport(1000.0, 600.0).to_px().unwrap(),
            25.0,
        );
    }

    #[test]
    fn logical_properties() {
        let s = parse(
            "inset-inline-start: 10px; \
             padding-block: 4px 8px; \
             margin-inline: auto;",
        );
        assert_eq!(s.inset_inline_start, Some(SizeValue::UnitValue(px(10.0))));
        assert_eq!(s.padding_block, vec![px(4.0), px(8.0)]);
        assert_eq!(s.margin_inline, vec![SizeValue::Auto]);
    }

    #[test]
    fn important_and_comments_combined() {
        let s = parse(
            r#"
            /* reset */
            width: 100px !important;
            // also reset
            height: 50px!important;
        "#,
        );
        assert_eq!(s.width, Some(SizeValue::UnitValue(px(100.0))));
        assert_eq!(s.height, Some(SizeValue::UnitValue(px(50.0))));
    }

    #[test]
    fn later_declaration_wins() {
        assert_eq!(
            parse("width: 10px; width: 20px;").width,
            Some(SizeValue::UnitValue(px(20.0)))
        );
    }

    #[test]
    fn font_weight_variants() {
        assert_eq!(
            parse("font-weight: bold;").font_weight,
            Some(FontWeight::Bold)
        );
        assert_eq!(
            parse("font-weight: 700;").font_weight,
            Some(FontWeight::W700)
        );
        assert_eq!(
            parse("font-weight: 400;").font_weight,
            Some(FontWeight::W400)
        );
    }
}
