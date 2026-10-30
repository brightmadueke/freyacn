//! Tailwind-style utility class parser.
//!
//! `parse_classes("flex items-center p-4 bg-blue-500")` produces a [`Style`]
//! equivalent to calling the `style!` macro with the corresponding CSS.
//!
//! Supports:
//! * Fixed keywords (`flex`, `items-center`, `font-bold`, …)
//! * Scale utilities (`p-4`, `gap-2`, `w-1/2`, …)
//! * Negative utilities (`-mt-4`, `-inset-2`, …)
//! * Palette colours (`bg-blue-500`, `text-slate-900`, …)
//! * Arbitrary values (`bg-[#ff0000]`, `w-[137px]`, …)
//!
//! **Not supported (deliberately):** responsive (`md:`), state (`hover:`),
//! dark-mode (`dark:`) variants — these need runtime context this crate
//! doesn't yet have.

use crate::stylesheet::style::Style;
use crate::stylesheet::types::*;
use crate::stylesheet::unit_value::UnitValue;

// ============================================================================
// Public entry point
// ============================================================================

/// Parse a whitespace-separated list of Tailwind-style utility classes
/// into a [`Style`].
///
/// Unknown classes are silently ignored.
pub fn parse_classes(input: &str) -> Style {
    let mut style = Style::default();
    for class in input.split_whitespace() {
        apply_class(&mut style, class);
    }
    style
}

// ============================================================================
// Dispatch
// ============================================================================

fn apply_class(style: &mut Style, class: &str) {
    // Arbitrary values come first because they can shadow prefixed utilities.
    if try_arbitrary(style, class) {
        return;
    }

    // Negative prefix: `-mt-4` → mt-4 with negative value.
    let (negative, rest) = match class.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, class),
    };

    if try_keyword(style, rest) {
        return;
    }
    if try_position(style, rest, negative) {
        return;
    }
    if try_flex(style, rest) {
        return;
    }
    if try_grid(style, rest) {
        return;
    }
    if try_spacing(style, rest, negative) {
        return;
    }
    if try_sizing(style, rest) {
        return;
    }
    if try_typography(style, rest) {
        return;
    }
    if try_border(style, rest) {
        return;
    }
    if try_rounded(style, rest) {
        return;
    }
    if try_color(style, rest) {
        return;
    }
    if try_effects(style, rest) {
        return;
    }
    if try_overflow(style, rest) {
        return;
    }
    if try_visibility(style, rest) {
        return;
    }
    if try_cursor(style, rest) {
        return;
    }
    if try_transition(style, rest) {
        return;
    }
    if try_transform(style, rest) {
        return;
    }
    if try_misc(style, rest) {
        return;
    }
}

// ============================================================================
// Arbitrary values: bg-[#f00], w-[137px], p-[2.5rem], text-[...]
// ============================================================================

fn try_arbitrary(style: &mut Style, class: &str) -> bool {
    let (prefix, value) = match class
        .split_once("-[")
        .and_then(|(p, v)| v.strip_suffix(']').map(|v| (p, v)))
    {
        Some(pair) => pair,
        None => return false,
    };
    let uv = || UnitValue::new(value);
    let sv = || SizeValue::UnitValue(UnitValue::new(value));

    match prefix {
        "w" => style.width = Some(sv()),
        "h" => style.height = Some(sv()),
        "min-w" => style.min_width = Some(sv()),
        "max-w" => style.max_width = Some(sv()),
        "min-h" => style.min_height = Some(sv()),
        "max-h" => style.max_height = Some(sv()),
        "p" => style.padding = vec![uv()],
        "px" => {
            style.padding_left = Some(uv());
            style.padding_right = Some(uv());
        }
        "py" => {
            style.padding_top = Some(uv());
            style.padding_bottom = Some(uv());
        }
        "pt" => style.padding_top = Some(uv()),
        "pr" => style.padding_right = Some(uv()),
        "pb" => style.padding_bottom = Some(uv()),
        "pl" => style.padding_left = Some(uv()),
        "m" => style.margin = vec![sv()],
        "mx" => {
            style.margin_left = Some(sv());
            style.margin_right = Some(sv());
        }
        "my" => {
            style.margin_top = Some(sv());
            style.margin_bottom = Some(sv());
        }
        "mt" => style.margin_top = Some(sv()),
        "mr" => style.margin_right = Some(sv()),
        "mb" => style.margin_bottom = Some(sv()),
        "ml" => style.margin_left = Some(sv()),
        "gap" => style.gap = vec![uv()],
        "gap-x" => style.column_gap = Some(uv()),
        "gap-y" => style.row_gap = Some(uv()),
        "top" => style.top = Some(sv()),
        "right" => style.right = Some(sv()),
        "bottom" => style.bottom = Some(sv()),
        "left" => style.left = Some(sv()),
        "inset" => style.inset = vec![sv()],
        "bg" => style.background_color = Some(value.parse().unwrap_or(Color::BLACK)),
        "text" => style.color = Some(value.parse().unwrap_or(Color::BLACK)),
        "border" => style.border_color = Some(value.parse().unwrap_or(Color::BLACK)),
        "rounded" => style.border_radius = vec![uv()],
        "opacity" => {
            if let Ok(n) = value.parse::<f32>() {
                style.opacity = Some(n);
            }
        }
        "z" => {
            if let Ok(n) = value.parse::<i32>() {
                style.z_index = Some(n);
            }
        }
        "text-size" => style.font_size = Some(uv()),
        _ => return false,
    }
    true
}

// ============================================================================
// Fixed keywords
// ============================================================================

fn try_keyword(style: &mut Style, class: &str) -> bool {
    match class {
        // --- Display ---
        "block" => style.display = vec![Display::Block],
        "inline-block" => style.display = vec![Display::InlineBlock],
        "inline" => style.display = vec![Display::Inline],
        "flex" => style.display = vec![Display::Flex],
        "inline-flex" => style.display = vec![Display::InlineFlex],
        "grid" => style.display = vec![Display::Grid],
        "inline-grid" => style.display = vec![Display::InlineGrid],
        "contents" => style.display = vec![Display::Contents],
        "flow-root" => style.display = vec![Display::FlowRoot],
        "hidden" => style.display = vec![Display::None],
        "table" => style.display = vec![Display::Table],
        "inline-table" => style.display = vec![Display::InlineTable],
        "list-item" => style.display = vec![Display::ListItem],

        // --- Position ---
        "static" => style.position = Some(Position::Static),
        "fixed" => style.position = Some(Position::Fixed),
        "absolute" => style.position = Some(Position::Absolute),
        "relative" => style.position = Some(Position::Relative),
        "sticky" => style.position = Some(Position::Sticky),

        // --- Visibility ---
        "visible" => style.visibility = Some(Visibility::Visible),
        "invisible" => style.visibility = Some(Visibility::Hidden),
        "collapse" => style.visibility = Some(Visibility::Collapse),

        // --- Flex container ---
        "flex-row" => style.flex_direction = Some(FlexDirection::Row),
        "flex-row-reverse" => style.flex_direction = Some(FlexDirection::RowReverse),
        "flex-col" => style.flex_direction = Some(FlexDirection::Column),
        "flex-col-reverse" => style.flex_direction = Some(FlexDirection::ColumnReverse),
        "flex-wrap" => style.flex_wrap = Some(FlexWrap::Wrap),
        "flex-wrap-reverse" => style.flex_wrap = Some(FlexWrap::WrapReverse),
        "flex-nowrap" => style.flex_wrap = Some(FlexWrap::Nowrap),

        // --- Flex shortcuts ---
        "flex-1" => {
            style.flex_grow = Some(1.0);
            style.flex_shrink = Some(1.0);
            style.flex_basis = Some(SizeValue::UnitValue(UnitValue::percent(0.0)));
        }
        "flex-auto" => {
            style.flex_grow = Some(1.0);
            style.flex_shrink = Some(1.0);
            style.flex_basis = Some(SizeValue::Auto);
        }
        "flex-initial" => {
            style.flex_grow = Some(0.0);
            style.flex_shrink = Some(1.0);
            style.flex_basis = Some(SizeValue::Auto);
        }
        "flex-none" => {
            style.flex_grow = Some(0.0);
            style.flex_shrink = Some(0.0);
            style.flex_basis = Some(SizeValue::Auto);
        }
        "grow" => style.flex_grow = Some(1.0),
        "grow-0" => style.flex_grow = Some(0.0),
        "shrink" => style.flex_shrink = Some(1.0),
        "shrink-0" => style.flex_shrink = Some(0.0),

        // --- Align / justify ---
        "items-start" => style.align_items = Some(AlignItems::FlexStart),
        "items-end" => style.align_items = Some(AlignItems::FlexEnd),
        "items-center" => style.align_items = Some(AlignItems::Center),
        "items-baseline" => style.align_items = Some(AlignItems::Baseline),
        "items-stretch" => style.align_items = Some(AlignItems::Stretch),

        "self-auto" => style.align_self = Some(AlignSelf::Auto),
        "self-start" => style.align_self = Some(AlignSelf::FlexStart),
        "self-end" => style.align_self = Some(AlignSelf::FlexEnd),
        "self-center" => style.align_self = Some(AlignSelf::Center),
        "self-stretch" => style.align_self = Some(AlignSelf::Stretch),
        "self-baseline" => style.align_self = Some(AlignSelf::Baseline),

        "justify-start" => style.justify_content = Some(JustifyContent::FlexStart),
        "justify-end" => style.justify_content = Some(JustifyContent::FlexEnd),
        "justify-center" => style.justify_content = Some(JustifyContent::Center),
        "justify-between" => style.justify_content = Some(JustifyContent::SpaceBetween),
        "justify-around" => style.justify_content = Some(JustifyContent::SpaceAround),
        "justify-evenly" => style.justify_content = Some(JustifyContent::SpaceEvenly),

        // --- Typography keywords ---
        "italic" => style.font_style = Some(FontStyle::Italic),
        "not-italic" => style.font_style = Some(FontStyle::Normal),

        "font-thin" => style.font_weight = Some(FontWeight::W100),
        "font-extralight" => style.font_weight = Some(FontWeight::W200),
        "font-light" => style.font_weight = Some(FontWeight::W300),
        "font-normal" => style.font_weight = Some(FontWeight::W400),
        "font-medium" => style.font_weight = Some(FontWeight::W500),
        "font-semibold" => style.font_weight = Some(FontWeight::W600),
        "font-bold" => style.font_weight = Some(FontWeight::W700),
        "font-extrabold" => style.font_weight = Some(FontWeight::W800),
        "font-black" => style.font_weight = Some(FontWeight::W900),

        "text-left" => style.text_align = Some(TextAlign::Left),
        "text-center" => style.text_align = Some(TextAlign::Center),
        "text-right" => style.text_align = Some(TextAlign::Right),
        "text-justify" => style.text_align = Some(TextAlign::Justify),
        "text-start" => style.text_align = Some(TextAlign::Start),
        "text-end" => style.text_align = Some(TextAlign::End),

        "underline" => style.text_decoration_line = Some(TextDecoration::Underline),
        "overline" => style.text_decoration_line = Some(TextDecoration::Overline),
        "line-through" => style.text_decoration_line = Some(TextDecoration::LineThrough),
        "no-underline" => style.text_decoration_line = Some(TextDecoration::None),

        "uppercase" => style.text_transform = Some(TextTransform::Uppercase),
        "lowercase" => style.text_transform = Some(TextTransform::Lowercase),
        "capitalize" => style.text_transform = Some(TextTransform::Capitalize),
        "normal-case" => style.text_transform = Some(TextTransform::None),

        "truncate" => {
            style.text_overflow = Some(TextOverflow::Ellipsis);
            style.white_space = Some(WhiteSpace::Nowrap);
            style.overflow = Some(Overflow::Hidden);
        }
        "text-ellipsis" => style.text_overflow = Some(TextOverflow::Ellipsis),
        "text-clip" => style.text_overflow = Some(TextOverflow::Clip),

        "whitespace-normal" => style.white_space = Some(WhiteSpace::Normal),
        "whitespace-nowrap" => style.white_space = Some(WhiteSpace::Nowrap),
        "whitespace-pre" => style.white_space = Some(WhiteSpace::Pre),
        "whitespace-pre-wrap" => style.white_space = Some(WhiteSpace::PreWrap),
        "whitespace-pre-line" => style.white_space = Some(WhiteSpace::PreLine),
        "whitespace-break-spaces" => style.white_space = Some(WhiteSpace::BreakSpaces),

        // --- Borders ---
        "border" => {
            style.border_width = Some(UnitValue::px(1.0));
            style.border_style = Some(BorderStyle::Solid);
        }
        "border-0" => style.border_width = Some(UnitValue::zero()),
        "border-2" => style.border_width = Some(UnitValue::px(2.0)),
        "border-4" => style.border_width = Some(UnitValue::px(4.0)),
        "border-8" => style.border_width = Some(UnitValue::px(8.0)),
        "border-solid" => style.border_style = Some(BorderStyle::Solid),
        "border-dashed" => style.border_style = Some(BorderStyle::Dashed),
        "border-dotted" => style.border_style = Some(BorderStyle::Dotted),
        "border-double" => style.border_style = Some(BorderStyle::Double),
        "border-none" => style.border_style = Some(BorderStyle::None),

        // --- Effects ---
        "shadow-sm" => style.box_shadow = vec![box_shadow(1.0, 2.0, 0.0, 0.05, false)],
        "shadow" => style.box_shadow = vec![box_shadow(1.0, 3.0, 0.0, 0.1, false)],
        "shadow-md" => style.box_shadow = vec![box_shadow(4.0, 6.0, -1.0, 0.1, false)],
        "shadow-lg" => style.box_shadow = vec![box_shadow(10.0, 15.0, -3.0, 0.1, false)],
        "shadow-xl" => style.box_shadow = vec![box_shadow(20.0, 25.0, -5.0, 0.1, false)],
        "shadow-2xl" => style.box_shadow = vec![box_shadow(25.0, 50.0, -12.0, 0.25, false)],
        "shadow-inner" => style.box_shadow = vec![box_shadow(2.0, 4.0, 0.0, 0.05, true)],
        "shadow-none" => style.box_shadow.clear(),

        // --- Overflow ---
        "overflow-auto" => style.overflow = Some(Overflow::Auto),
        "overflow-hidden" => style.overflow = Some(Overflow::Hidden),
        "overflow-clip" => style.overflow = Some(Overflow::Clip),
        "overflow-visible" => style.overflow = Some(Overflow::Visible),
        "overflow-scroll" => style.overflow = Some(Overflow::Scroll),
        "overflow-x-auto" => style.overflow_x = Some(Overflow::Auto),
        "overflow-x-hidden" => style.overflow_x = Some(Overflow::Hidden),
        "overflow-x-scroll" => style.overflow_x = Some(Overflow::Scroll),
        "overflow-y-auto" => style.overflow_y = Some(Overflow::Auto),
        "overflow-y-hidden" => style.overflow_y = Some(Overflow::Hidden),
        "overflow-y-scroll" => style.overflow_y = Some(Overflow::Scroll),

        // --- Interaction ---
        "pointer-events-none" => style.pointer_events = Some(PointerEvents::None),
        "pointer-events-auto" => style.pointer_events = Some(PointerEvents::Auto),

        "select-none" => style.user_select = Some(UserSelect::None),
        "select-text" => style.user_select = Some(UserSelect::Text),
        "select-all" => style.user_select = Some(UserSelect::All),
        "select-auto" => style.user_select = Some(UserSelect::Auto),

        // --- Transitions (keyword presets) ---
        "transition-none" => {
            style.transition_property = Some("none".into());
            style.transition_duration = Some(0.0);
        }
        "transition-all" => {
            style.transition_property = Some("all".into());
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "transition" => {
            style.transition_property = Some(
                "color, background-color, border-color, opacity, box-shadow, transform".into(),
            );
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "transition-colors" => {
            style.transition_property =
                Some("color, background-color, border-color, fill, stroke".into());
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "transition-opacity" => {
            style.transition_property = Some("opacity".into());
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "transition-shadow" => {
            style.transition_property = Some("box-shadow".into());
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "transition-transform" => {
            style.transition_property = Some("transform".into());
            style.transition_duration = Some(0.15);
            style.transition_timing_function = Some("cubic-bezier(0.4,0,0.2,1)".into());
        }
        "ease-linear" => style.transition_timing_function = Some("linear".into()),
        "ease-in" => style.transition_timing_function = Some("ease-in".into()),
        "ease-out" => style.transition_timing_function = Some("ease-out".into()),
        "ease-in-out" => style.transition_timing_function = Some("ease-in-out".into()),

        _ => return false,
    }
    true
}

fn box_shadow(oy: f32, blur: f32, spread: f32, alpha: f32, inset: bool) -> BoxShadow {
    BoxShadow {
        offset_x: UnitValue::zero(),
        offset_y: UnitValue::px(oy),
        blur: UnitValue::px(blur),
        spread: UnitValue::px(spread),
        color: Color::Keyword(format!("rgba(0,0,0,{})", alpha)),
        inset,
    }
}

// ============================================================================
// Position & inset
// ============================================================================

fn try_position(style: &mut Style, class: &str, negative: bool) -> bool {
    // z-{n}
    if let Some(rest) = class.strip_prefix("z-") {
        if rest == "auto" {
            style.z_index = Some(i32::MAX);
            return true;
        }
        if let Ok(n) = rest.parse::<i32>() {
            style.z_index = Some(n);
            return true;
        }
    }

    // inset, top, right, bottom, left
    for (prefix, apply) in [
        (
            "inset-x-",
            Box::new(|s: &mut Style, v: UnitValue| {
                let sv = SizeValue::UnitValue(v);
                s.left = Some(sv.clone());
                s.right = Some(sv);
            }) as Box<dyn Fn(&mut Style, UnitValue)>,
        ),
        (
            "inset-y-",
            Box::new(|s: &mut Style, v: UnitValue| {
                let sv = SizeValue::UnitValue(v);
                s.top = Some(sv.clone());
                s.bottom = Some(sv);
            }),
        ),
        (
            "inset-",
            Box::new(|s: &mut Style, v: UnitValue| {
                s.inset = vec![SizeValue::UnitValue(v)];
            }),
        ),
        (
            "top-",
            Box::new(|s: &mut Style, v: UnitValue| {
                s.top = Some(SizeValue::UnitValue(v));
            }),
        ),
        (
            "right-",
            Box::new(|s: &mut Style, v: UnitValue| {
                s.right = Some(SizeValue::UnitValue(v));
            }),
        ),
        (
            "bottom-",
            Box::new(|s: &mut Style, v: UnitValue| {
                s.bottom = Some(SizeValue::UnitValue(v));
            }),
        ),
        (
            "left-",
            Box::new(|s: &mut Style, v: UnitValue| {
                s.left = Some(SizeValue::UnitValue(v));
            }),
        ),
    ] {
        if let Some(rest) = class.strip_prefix(prefix) {
            // Tailwind's inset accepts any scale value OR `auto` OR `full`.
            if rest == "auto" {
                let sv = SizeValue::Auto;
                match prefix {
                    "inset-x-" => {
                        style.left = Some(sv.clone());
                        style.right = Some(sv);
                    }
                    "inset-y-" => {
                        style.top = Some(sv.clone());
                        style.bottom = Some(sv);
                    }
                    "inset-" => style.inset = vec![sv],
                    "top-" => style.top = Some(sv),
                    "right-" => style.right = Some(sv),
                    "bottom-" => style.bottom = Some(sv),
                    "left-" => style.left = Some(sv),
                    _ => unreachable!(),
                }
                return true;
            }
            if rest == "full" {
                let sv = SizeValue::UnitValue(UnitValue::percent(100.0));
                apply(
                    style,
                    match sv {
                        SizeValue::UnitValue(u) => u,
                        _ => unreachable!(),
                    },
                );
                return true;
            }
            if let Some(uv) = parse_scale(rest, negative) {
                apply(style, uv);
                return true;
            }
        }
    }
    false
}

// ============================================================================
// Flex prefixed (`basis-*`)
// ============================================================================

fn try_flex(style: &mut Style, class: &str) -> bool {
    if let Some(rest) = class.strip_prefix("basis-") {
        match rest {
            "auto" => {
                style.flex_basis = Some(SizeValue::Auto);
                return true;
            }
            "full" => {
                style.flex_basis = Some(SizeValue::UnitValue(UnitValue::percent(100.0)));
                return true;
            }
            _ => {
                if let Some(uv) = parse_scale(rest, false) {
                    style.flex_basis = Some(SizeValue::UnitValue(uv));
                    return true;
                }
            }
        }
    }
    false
}

// ============================================================================
// Grid (`grid-cols-{n}`, `grid-rows-{n}`)
// ============================================================================

fn try_grid(style: &mut Style, class: &str) -> bool {
    for (prefix, target) in [
        ("grid-cols-", &mut style.grid_template_columns as *mut _),
        ("grid-rows-", &mut style.grid_template_rows as *mut _),
    ] {
        if let Some(rest) = class.strip_prefix(prefix) {
            let n: u32 = match rest.parse() {
                Ok(n) if (1..=12).contains(&n) => n,
                _ => return false,
            };
            let value = std::iter::repeat("minmax(0, 1fr)")
                .take(n as usize)
                .collect::<Vec<_>>()
                .join(" ");
            // SAFETY: `target` is a pointer into `style`, which outlives this call.
            unsafe { *target = Some(value) };
            return true;
        }
    }
    false
}

// ============================================================================
// Spacing (m, p, gap)
// ============================================================================

fn try_spacing(style: &mut Style, class: &str, negative: bool) -> bool {
    // Longest prefixes first so `gap-x-` doesn't get eaten by `gap-`.
    for prefix in [
        "gap-x-", "gap-y-", "gap-", "pt-", "pr-", "pb-", "pl-", "px-", "py-", "p-", "mt-", "mr-",
        "mb-", "ml-", "mx-", "my-", "m-",
    ] {
        let Some(rest) = class.strip_prefix(prefix) else {
            continue;
        };

        // Auto is only valid for margin.
        if rest == "auto" && prefix.starts_with('m') {
            let sv = SizeValue::Auto;
            match prefix {
                "mt-" => style.margin_top = Some(sv),
                "mr-" => style.margin_right = Some(sv),
                "mb-" => style.margin_bottom = Some(sv),
                "ml-" => style.margin_left = Some(sv),
                "mx-" => {
                    style.margin_left = Some(sv.clone());
                    style.margin_right = Some(sv);
                }
                "my-" => {
                    style.margin_top = Some(sv.clone());
                    style.margin_bottom = Some(sv);
                }
                "m-" => style.margin = vec![sv],
                _ => unreachable!(),
            }
            return true;
        }

        let Some(uv) = parse_scale(rest, negative) else {
            continue;
        };
        let sv = || SizeValue::UnitValue(uv.clone());

        match prefix {
            "p-" => style.padding = vec![uv],
            "pt-" => style.padding_top = Some(uv),
            "pr-" => style.padding_right = Some(uv),
            "pb-" => style.padding_bottom = Some(uv),
            "pl-" => style.padding_left = Some(uv),
            "px-" => {
                style.padding_left = Some(uv.clone());
                style.padding_right = Some(uv);
            }
            "py-" => {
                style.padding_top = Some(uv.clone());
                style.padding_bottom = Some(uv);
            }
            "m-" => style.margin = vec![sv()],
            "mt-" => style.margin_top = Some(sv()),
            "mr-" => style.margin_right = Some(sv()),
            "mb-" => style.margin_bottom = Some(sv()),
            "ml-" => style.margin_left = Some(sv()),
            "mx-" => {
                style.margin_left = Some(sv());
                style.margin_right = Some(sv());
            }
            "my-" => {
                style.margin_top = Some(sv());
                style.margin_bottom = Some(sv());
            }
            "gap-" => style.gap = vec![uv],
            "gap-x-" => style.column_gap = Some(uv),
            "gap-y-" => style.row_gap = Some(uv),
            _ => return false,
        }
        return true;
    }
    false
}

// ============================================================================
// Sizing
// ============================================================================

fn try_sizing(style: &mut Style, class: &str) -> bool {
    for prefix in ["min-w-", "min-h-", "max-w-", "max-h-", "size-", "w-", "h-"] {
        let Some(rest) = class.strip_prefix(prefix) else {
            continue;
        };

        // `size-{n}` sets both width and height.
        let set_both = prefix == "size-";

        let value = match rest {
            "auto" => SizeValue::Auto,
            "full" => SizeValue::UnitValue(UnitValue::percent(100.0)),
            "screen" => match prefix {
                "w-" | "min-w-" | "max-w-" => SizeValue::UnitValue(UnitValue::vw(100.0)),
                "h-" | "min-h-" | "max-h-" => SizeValue::UnitValue(UnitValue::vh(100.0)),
                _ => SizeValue::Auto,
            },
            "min" => SizeValue::MinContent,
            "max" => SizeValue::MaxContent,
            "fit" => SizeValue::FitContent(UnitValue::percent(100.0)),
            "px" => SizeValue::UnitValue(UnitValue::px(1.0)),
            _ => {
                if let Some(uv) = parse_fraction(rest).or_else(|| parse_scale(rest, false)) {
                    SizeValue::UnitValue(uv)
                } else {
                    return false;
                }
            }
        };

        if set_both {
            style.width = Some(value.clone());
            style.height = Some(value);
            return true;
        }

        match prefix {
            "min-w-" => style.min_width = Some(value),
            "min-h-" => style.min_height = Some(value),
            "max-w-" => style.max_width = Some(value),
            "max-h-" => style.max_height = Some(value),
            "w-" => style.width = Some(value),
            "h-" => style.height = Some(value),
            _ => return false,
        }
        return true;
    }
    false
}

// ============================================================================
// Typography (`text-{size}` — align/color handled elsewhere)
// ============================================================================

fn try_typography(style: &mut Style, class: &str) -> bool {
    let Some(rest) = class.strip_prefix("text-") else {
        return false;
    };
    let size = match rest {
        "xs" => 0.75,
        "sm" => 0.875,
        "base" => 1.0,
        "lg" => 1.125,
        "xl" => 1.25,
        "2xl" => 1.5,
        "3xl" => 1.875,
        "4xl" => 2.25,
        "5xl" => 3.0,
        "6xl" => 3.75,
        "7xl" => 4.5,
        "8xl" => 6.0,
        "9xl" => 8.0,
        _ => return false,
    };
    style.font_size = Some(UnitValue::rem(size));
    true
}

// ============================================================================
// Borders (`border-{side}[-{width}]`)
// ============================================================================

fn try_border(style: &mut Style, class: &str) -> bool {
    let Some(rest) = class.strip_prefix("border-") else {
        return false;
    };
    let (side, width_part) = match rest.split_once('-') {
        Some((s, w)) => (s, Some(w)),
        None => (rest, None),
    };
    if !matches!(side, "t" | "r" | "b" | "l" | "x" | "y") {
        return false;
    }

    let width = match width_part {
        None => 1.0,
        Some("0") => 0.0,
        Some("2") => 2.0,
        Some("4") => 4.0,
        Some("8") => 8.0,
        _ => return false,
    };
    let uv = || UnitValue::px(width);

    match side {
        "t" => style.border_top_width = Some(uv()),
        "r" => style.border_right_width = Some(uv()),
        "b" => style.border_bottom_width = Some(uv()),
        "l" => style.border_left_width = Some(uv()),
        "x" => {
            style.border_left_width = Some(uv());
            style.border_right_width = Some(uv());
        }
        "y" => {
            style.border_top_width = Some(uv());
            style.border_bottom_width = Some(uv());
        }
        _ => return false,
    }
    if style.border_style.is_none() {
        style.border_style = Some(BorderStyle::Solid);
    }
    true
}

// ============================================================================
// Rounded corners
// ============================================================================

fn try_rounded(style: &mut Style, class: &str) -> bool {
    let Some(rest) = class.strip_prefix("rounded") else {
        return false;
    };

    let (sides, size) = if rest.is_empty() {
        (None, "DEFAULT")
    } else if let Some(s) = rest.strip_prefix('-') {
        match s.split_once('-') {
            Some((side, sz)) => (Some(side), sz),
            None if matches!(
                s,
                "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "full" | "none"
            ) =>
            {
                (None, s)
            }
            None => (Some(s), "DEFAULT"),
        }
    } else {
        return false;
    };

    let uv = match size {
        "DEFAULT" => UnitValue::rem(0.25),
        "none" => UnitValue::zero(),
        "sm" => UnitValue::rem(0.125),
        "md" => UnitValue::rem(0.375),
        "lg" => UnitValue::rem(0.5),
        "xl" => UnitValue::rem(0.75),
        "2xl" => UnitValue::rem(1.0),
        "3xl" => UnitValue::rem(1.5),
        "full" => UnitValue::px(9999.0),
        _ => return false,
    };

    match sides {
        None => {
            style.border_top_left_radius = Some(uv.clone());
            style.border_top_right_radius = Some(uv.clone());
            style.border_bottom_right_radius = Some(uv.clone());
            style.border_bottom_left_radius = Some(uv);
        }
        Some("t") => {
            style.border_top_left_radius = Some(uv.clone());
            style.border_top_right_radius = Some(uv);
        }
        Some("r") => {
            style.border_top_right_radius = Some(uv.clone());
            style.border_bottom_right_radius = Some(uv);
        }
        Some("b") => {
            style.border_bottom_right_radius = Some(uv.clone());
            style.border_bottom_left_radius = Some(uv);
        }
        Some("l") => {
            style.border_top_left_radius = Some(uv.clone());
            style.border_bottom_left_radius = Some(uv);
        }
        Some("tl") => style.border_top_left_radius = Some(uv),
        Some("tr") => style.border_top_right_radius = Some(uv),
        Some("br") => style.border_bottom_right_radius = Some(uv),
        Some("bl") => style.border_bottom_left_radius = Some(uv),
        _ => return false,
    }
    true
}

// ============================================================================
// Colours
// ============================================================================

fn try_color(style: &mut Style, class: &str) -> bool {
    if let Some(rest) = class.strip_prefix("bg-") {
        if let Some(c) = palette_color(rest) {
            style.background_color = Some(c);
            return true;
        }
    }
    if let Some(rest) = class.strip_prefix("text-") {
        if let Some(c) = palette_color(rest) {
            style.color = Some(c);
            return true;
        }
    }
    if let Some(rest) = class.strip_prefix("border-") {
        if let Some(c) = palette_color(rest) {
            style.border_color = Some(c);
            return true;
        }
    }
    false
}

/// Resolve a Tailwind palette colour from `"{color}-{shade}"` or a
/// keyword (`black`, `white`, `transparent`, `current`).
fn palette_color(s: &str) -> Option<Color> {
    match s {
        "black" => return Some(Color::BLACK),
        "white" => return Some(Color::WHITE),
        "transparent" => return Some(Color::TRANSPARENT),
        "current" => return Some(Color::CURRENT_COLOR),
        _ => {}
    }

    let (name, shade) = s.rsplit_once('-')?;
    let idx = match shade {
        "50" => 0,
        "100" => 1,
        "200" => 2,
        "300" => 3,
        "400" => 4,
        "500" => 5,
        "600" => 6,
        "700" => 7,
        "800" => 8,
        "900" => 9,
        "950" => 10,
        _ => return None,
    };
    let shades: [u32; 11] = match name {
        "slate" => [
            0xf8fafc, 0xf1f5f9, 0xe2e8f0, 0xcbd5e1, 0x94a3b8, 0x64748b, 0x475569, 0x334155,
            0x1e293b, 0x0f172a, 0x020617,
        ],
        "gray" => [
            0xf9fafb, 0xf3f4f6, 0xe5e7eb, 0xd1d5db, 0x9ca3af, 0x6b7280, 0x4b5563, 0x374151,
            0x1f2937, 0x111827, 0x030712,
        ],
        "zinc" => [
            0xfafafa, 0xf4f4f5, 0xe4e4e7, 0xd4d4d8, 0xa1a1aa, 0x71717a, 0x52525b, 0x3f3f46,
            0x27272a, 0x18181b, 0x09090b,
        ],
        "neutral" => [
            0xfafafa, 0xf5f5f5, 0xe5e5e5, 0xd4d4d4, 0xa3a3a3, 0x737373, 0x525252, 0x404040,
            0x262626, 0x171717, 0x0a0a0a,
        ],
        "stone" => [
            0xfafaf9, 0xf5f5f4, 0xe7e5e4, 0xd6d3d1, 0xa8a29e, 0x78716c, 0x57534e, 0x44403c,
            0x292524, 0x1c1917, 0x0c0a09,
        ],
        "red" => [
            0xfef2f2, 0xfee2e2, 0xfecaca, 0xfca5a5, 0xf87171, 0xef4444, 0xdc2626, 0xb91c1c,
            0x991b1b, 0x7f1d1d, 0x450a0a,
        ],
        "orange" => [
            0xfff7ed, 0xffedd5, 0xfed7aa, 0xfdba74, 0xfb923c, 0xf97316, 0xea580c, 0xc2410c,
            0x9a3412, 0x7c2d12, 0x431407,
        ],
        "amber" => [
            0xfffbeb, 0xfef3c7, 0xfde68a, 0xfcd34d, 0xfbbf24, 0xf59e0b, 0xd97706, 0xb45309,
            0x92400e, 0x78350f, 0x451a03,
        ],
        "yellow" => [
            0xfefce8, 0xfef9c3, 0xfef08a, 0xfde047, 0xfacc15, 0xeab308, 0xca8a04, 0xa16207,
            0x854d0e, 0x713f12, 0x422006,
        ],
        "lime" => [
            0xf7fee7, 0xecfccb, 0xd9f99d, 0xbef264, 0xa3e635, 0x84cc16, 0x65a30d, 0x4d7c0f,
            0x3f6212, 0x365314, 0x1a2e05,
        ],
        "green" => [
            0xf0fdf4, 0xdcfce7, 0xbbf7d0, 0x86efac, 0x4ade80, 0x22c55e, 0x16a34a, 0x15803d,
            0x166534, 0x14532d, 0x052e16,
        ],
        "emerald" => [
            0xecfdf5, 0xd1fae5, 0xa7f3d0, 0x6ee7b7, 0x34d399, 0x10b981, 0x059669, 0x047857,
            0x065f46, 0x064e3b, 0x022c22,
        ],
        "teal" => [
            0xf0fdfa, 0xccfbf1, 0x99f6e4, 0x5eead4, 0x2dd4bf, 0x14b8a6, 0x0d9488, 0x0f766e,
            0x115e59, 0x134e4a, 0x042f2e,
        ],
        "cyan" => [
            0xecfeff, 0xcffafe, 0xa5f3fc, 0x67e8f9, 0x22d3ee, 0x06b6d4, 0x0891b2, 0x0e7490,
            0x155e75, 0x164e63, 0x083344,
        ],
        "sky" => [
            0xf0f9ff, 0xe0f2fe, 0xbae6fd, 0x7dd3fc, 0x38bdf8, 0x0ea5e9, 0x0284c7, 0x0369a1,
            0x075985, 0x0c4a6e, 0x082f49,
        ],
        "blue" => [
            0xeff6ff, 0xdbeafe, 0xbfdbfe, 0x93c5fd, 0x60a5fa, 0x3b82f6, 0x2563eb, 0x1d4ed8,
            0x1e40af, 0x1e3a8a, 0x172554,
        ],
        "indigo" => [
            0xeef2ff, 0xe0e7ff, 0xc7d2fe, 0xa5b4fc, 0x818cf8, 0x6366f1, 0x4f46e5, 0x4338ca,
            0x3730a3, 0x312e81, 0x1e1b4b,
        ],
        "violet" => [
            0xf5f3ff, 0xede9fe, 0xddd6fe, 0xc4b5fd, 0xa78bfa, 0x8b5cf6, 0x7c3aed, 0x6d28d9,
            0x5b21b6, 0x4c1d95, 0x2e1065,
        ],
        "purple" => [
            0xfaf5ff, 0xf3e8ff, 0xe9d5ff, 0xd8b4fe, 0xc084fc, 0xa855f7, 0x9333ea, 0x7e22ce,
            0x6b21a8, 0x581c87, 0x3b0764,
        ],
        "fuchsia" => [
            0xfdf4ff, 0xfae8ff, 0xf5d0fe, 0xf0abfc, 0xe879f9, 0xd946ef, 0xc026d3, 0xa21caf,
            0x86198f, 0x701a75, 0x4a044e,
        ],
        "pink" => [
            0xfdf2f8, 0xfce7f3, 0xfbcfe8, 0xf9a8d4, 0xf472b6, 0xec4899, 0xdb2777, 0xbe185d,
            0x9d174d, 0x831843, 0x500724,
        ],
        "rose" => [
            0xfff1f2, 0xffe4e6, 0xfecdd3, 0xfda4af, 0xfb7185, 0xf43f5e, 0xe11d48, 0xbe123c,
            0x9f1239, 0x881337, 0x4c0519,
        ],
        _ => return None,
    };
    Some(Color::hex(shades[idx]))
}

// ============================================================================
// Effects (opacity, shadow — keywords handled above)
// ============================================================================

fn try_effects(style: &mut Style, class: &str) -> bool {
    if let Some(rest) = class.strip_prefix("opacity-") {
        if let Ok(n) = rest.parse::<f32>() {
            if (0.0..=100.0).contains(&n) {
                style.opacity = Some(n / 100.0);
                return true;
            }
        }
    }
    false
}

// ============================================================================
// Overflow (x/y axis already handled by try_keyword)
// ============================================================================

fn try_overflow(_style: &mut Style, _class: &str) -> bool {
    // All overflow-* handled by try_keyword.
    false
}

// ============================================================================
// Visibility (already handled by try_keyword)
// ============================================================================

fn try_visibility(_style: &mut Style, _class: &str) -> bool {
    false
}

// ============================================================================
// Cursor
// ============================================================================

fn try_cursor(style: &mut Style, class: &str) -> bool {
    let Some(rest) = class.strip_prefix("cursor-") else {
        return false;
    };
    if let Ok(c) = rest.parse::<Cursor>() {
        style.cursor = Some(c);
        return true;
    }
    false
}

// ============================================================================
// Transition durations
// ============================================================================

fn try_transition(style: &mut Style, class: &str) -> bool {
    let Some(rest) = class.strip_prefix("duration-") else {
        return false;
    };
    let ms: u32 = match rest.parse() {
        Ok(n) => n,
        Err(_) => return false,
    };
    style.transition_duration = Some(ms as f32 / 1000.0);
    true
}

// ============================================================================
// Transforms (scale, rotate, translate)
// ============================================================================

fn try_transform(style: &mut Style, class: &str) -> bool {
    if let Some(rest) = class.strip_prefix("scale-") {
        if let Ok(n) = rest.parse::<f32>() {
            style.scale = Some((n / 100.0, n / 100.0));
            return true;
        }
    }
    if let Some(rest) = class.strip_prefix("rotate-") {
        if let Ok(n) = rest.parse::<f32>() {
            style.rotate = Some(n);
            return true;
        }
    }
    if let Some(rest) = class.strip_prefix("translate-x-") {
        if let Some(uv) = parse_scale(rest, false) {
            style.translate = Some((
                uv,
                style
                    .translate
                    .as_ref()
                    .map(|(_, y)| y.clone())
                    .unwrap_or_else(UnitValue::zero),
            ));
            return true;
        }
    }
    if let Some(rest) = class.strip_prefix("translate-y-") {
        if let Some(uv) = parse_scale(rest, false) {
            style.translate = Some((
                style
                    .translate
                    .as_ref()
                    .map(|(x, _)| x.clone())
                    .unwrap_or_else(UnitValue::zero),
                uv,
            ));
            return true;
        }
    }
    false
}

// ============================================================================
// Misc (pointer-events / select — already handled above, reserved for future)
// ============================================================================

fn try_misc(_style: &mut Style, _class: &str) -> bool {
    false
}

// ============================================================================
// Scale / fraction helpers
// ============================================================================

/// Tailwind's spacing scale (rem values from numeric scale tokens).
fn scale_rem(n: f32) -> Option<f32> {
    Some(match n {
        0.0 => 0.0,
        0.5 => 0.125,
        1.0 => 0.25,
        1.5 => 0.375,
        2.0 => 0.5,
        2.5 => 0.625,
        3.0 => 0.75,
        3.5 => 0.875,
        4.0 => 1.0,
        5.0 => 1.25,
        6.0 => 1.5,
        7.0 => 1.75,
        8.0 => 2.0,
        9.0 => 2.25,
        10.0 => 2.5,
        11.0 => 2.75,
        12.0 => 3.0,
        14.0 => 3.5,
        16.0 => 4.0,
        20.0 => 5.0,
        24.0 => 6.0,
        28.0 => 7.0,
        32.0 => 8.0,
        36.0 => 9.0,
        40.0 => 10.0,
        44.0 => 11.0,
        48.0 => 12.0,
        52.0 => 13.0,
        56.0 => 14.0,
        60.0 => 15.0,
        64.0 => 16.0,
        72.0 => 18.0,
        80.0 => 20.0,
        96.0 => 24.0,
        _ => return None,
    })
}

fn parse_scale(v: &str, negative: bool) -> Option<UnitValue> {
    if v == "px" {
        return Some(UnitValue::px(if negative { -1.0 } else { 1.0 }));
    }
    if v == "0" {
        return Some(UnitValue::zero());
    }
    let n: f32 = v.parse().ok()?;
    let rem = scale_rem(n)?;
    Some(UnitValue::rem(if negative { -rem } else { rem }))
}

fn parse_fraction(v: &str) -> Option<UnitValue> {
    let (num, den) = v.split_once('/')?;
    let n: f32 = num.parse().ok()?;
    let d: f32 = den.parse().ok()?;
    if d == 0.0 {
        return None;
    }
    Some(UnitValue::percent(n / d * 100.0))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn px(v: f32) -> UnitValue {
        UnitValue::px(v)
    }

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {b}, got {a}");
    }

    fn unwrap_uv(s: &Option<SizeValue>) -> UnitValue {
        match s {
            Some(SizeValue::UnitValue(u)) => u.clone(),
            other => panic!("expected SizeValue::UnitValue, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Basic dispatch
    // ------------------------------------------------------------------

    #[test]
    fn empty_input_returns_default() {
        let s = parse_classes("");
        assert!(s.display.is_empty());
    }

    #[test]
    fn unknown_class_is_ignored() {
        let s = parse_classes("nonsense-xyz flex");
        assert_eq!(s.display, vec![Display::Flex]);
    }

    #[test]
    fn whitespace_is_split() {
        let s = parse_classes("  flex   items-center\n p-4  ");
        assert_eq!(s.display, vec![Display::Flex]);
        assert_eq!(s.align_items, Some(AlignItems::Center));
        assert_eq!(s.padding, vec![UnitValue::rem(1.0)]);
    }

    // ------------------------------------------------------------------
    // Display & flex keywords
    // ------------------------------------------------------------------

    #[test]
    fn display_keywords() {
        assert_eq!(parse_classes("block").display, vec![Display::Block]);
        assert_eq!(parse_classes("flex").display, vec![Display::Flex]);
        assert_eq!(parse_classes("hidden").display, vec![Display::None]);
        assert_eq!(parse_classes("grid").display, vec![Display::Grid]);
    }

    #[test]
    fn position_keywords() {
        assert_eq!(parse_classes("absolute").position, Some(Position::Absolute));
        assert_eq!(parse_classes("relative").position, Some(Position::Relative));
    }

    #[test]
    fn flex_container_keywords() {
        assert_eq!(
            parse_classes("flex-col").flex_direction,
            Some(FlexDirection::Column)
        );
        assert_eq!(parse_classes("flex-wrap").flex_wrap, Some(FlexWrap::Wrap));
        assert_eq!(
            parse_classes("items-center").align_items,
            Some(AlignItems::Center)
        );
        assert_eq!(
            parse_classes("justify-between").justify_content,
            Some(JustifyContent::SpaceBetween)
        );
    }

    #[test]
    fn flex_children_keywords() {
        let s = parse_classes("flex-1");
        assert_eq!(s.flex_grow, Some(1.0));
        assert_eq!(s.flex_shrink, Some(1.0));
    }

    // ------------------------------------------------------------------
    // Spacing scale
    // ------------------------------------------------------------------

    #[test]
    fn spacing_p() {
        assert_eq!(parse_classes("p-4").padding, vec![UnitValue::rem(1.0)]);
        assert_eq!(parse_classes("p-0").padding, vec![UnitValue::zero()]);
        assert_eq!(parse_classes("p-px").padding, vec![px(1.0)]);
    }

    #[test]
    fn spacing_px_py() {
        let s = parse_classes("px-2 py-4");
        assert_eq!(s.padding_left, Some(UnitValue::rem(0.5)));
        assert_eq!(s.padding_right, Some(UnitValue::rem(0.5)));
        assert_eq!(s.padding_top, Some(UnitValue::rem(1.0)));
        assert_eq!(s.padding_bottom, Some(UnitValue::rem(1.0)));
    }

    #[test]
    fn spacing_negative() {
        let s = parse_classes("-mt-4");
        assert_eq!(
            s.margin_top,
            Some(SizeValue::UnitValue(UnitValue::rem(-1.0)))
        );
    }

    #[test]
    fn spacing_auto() {
        assert_eq!(parse_classes("mx-auto").margin_left, Some(SizeValue::Auto));
        assert_eq!(parse_classes("mx-auto").margin_right, Some(SizeValue::Auto));
    }

    #[test]
    fn spacing_gap() {
        assert_eq!(parse_classes("gap-4").gap, vec![UnitValue::rem(1.0)]);
        assert_eq!(
            parse_classes("gap-x-2").column_gap,
            Some(UnitValue::rem(0.5))
        );
        assert_eq!(parse_classes("gap-y-2").row_gap, Some(UnitValue::rem(0.5)));
    }

    // ------------------------------------------------------------------
    // Sizing
    // ------------------------------------------------------------------

    #[test]
    fn sizing_fixed() {
        assert_eq!(unwrap_uv(&parse_classes("w-4").width), UnitValue::rem(1.0));
        assert_eq!(unwrap_uv(&parse_classes("h-8").height), UnitValue::rem(2.0));
    }

    #[test]
    fn sizing_full() {
        assert!(matches!(
            parse_classes("w-full").width,
            Some(SizeValue::UnitValue(u)) if u.to_percent() == Some(100.0)
        ));
    }

    #[test]
    fn sizing_screen() {
        assert!(matches!(
            parse_classes("w-screen").width,
            Some(SizeValue::UnitValue(u)) if u.to_percent().is_none()
        ));
        assert!(matches!(
            parse_classes("h-screen").height,
            Some(SizeValue::UnitValue(u)) if u.to_percent().is_none()
        ));
    }

    #[test]
    fn sizing_fraction() {
        let w = unwrap_uv(&parse_classes("w-1/2").width);
        approx(w.to_percent().unwrap(), 50.0);
        let w = unwrap_uv(&parse_classes("w-3/4").width);
        approx(w.to_percent().unwrap(), 75.0);
    }

    #[test]
    fn sizing_auto_min_max_fit() {
        assert_eq!(parse_classes("w-auto").width, Some(SizeValue::Auto));
        assert_eq!(parse_classes("w-min").width, Some(SizeValue::MinContent));
        assert_eq!(parse_classes("w-max").width, Some(SizeValue::MaxContent));
    }

    #[test]
    fn sizing_size_sets_both() {
        let s = parse_classes("size-4");
        assert_eq!(unwrap_uv(&s.width), UnitValue::rem(1.0));
        assert_eq!(unwrap_uv(&s.height), UnitValue::rem(1.0));
    }

    // ------------------------------------------------------------------
    // Typography
    // ------------------------------------------------------------------

    #[test]
    fn font_sizes() {
        assert_eq!(
            parse_classes("text-base").font_size,
            Some(UnitValue::rem(1.0))
        );
        assert_eq!(
            parse_classes("text-xl").font_size,
            Some(UnitValue::rem(1.25))
        );
        assert_eq!(
            parse_classes("text-3xl").font_size,
            Some(UnitValue::rem(1.875))
        );
    }

    #[test]
    fn font_weights() {
        assert_eq!(
            parse_classes("font-bold").font_weight,
            Some(FontWeight::W700)
        );
        assert_eq!(
            parse_classes("font-thin").font_weight,
            Some(FontWeight::W100)
        );
    }

    #[test]
    fn text_alignment() {
        assert_eq!(
            parse_classes("text-center").text_align,
            Some(TextAlign::Center)
        );
    }

    #[test]
    fn text_decoration_and_transform() {
        assert_eq!(
            parse_classes("underline").text_decoration_line,
            Some(TextDecoration::Underline)
        );
        assert_eq!(
            parse_classes("uppercase").text_transform,
            Some(TextTransform::Uppercase)
        );
    }

    #[test]
    fn truncate_sets_three_properties() {
        let s = parse_classes("truncate");
        assert_eq!(s.text_overflow, Some(TextOverflow::Ellipsis));
        assert_eq!(s.white_space, Some(WhiteSpace::Nowrap));
        assert_eq!(s.overflow, Some(Overflow::Hidden));
    }

    // ------------------------------------------------------------------
    // Colours
    // ------------------------------------------------------------------

    #[test]
    fn bg_keywords() {
        assert_eq!(
            parse_classes("bg-black").background_color,
            Some(Color::BLACK)
        );
        assert_eq!(
            parse_classes("bg-white").background_color,
            Some(Color::WHITE)
        );
        assert_eq!(
            parse_classes("bg-transparent").background_color,
            Some(Color::TRANSPARENT)
        );
    }

    #[test]
    fn bg_palette() {
        assert_eq!(
            parse_classes("bg-blue-500").background_color,
            Some(Color::hex(0x3b82f6))
        );
        assert_eq!(
            parse_classes("bg-red-500").background_color,
            Some(Color::hex(0xef4444))
        );
        assert_eq!(
            parse_classes("bg-slate-900").background_color,
            Some(Color::hex(0x0f172a))
        );
    }

    #[test]
    fn text_color() {
        assert_eq!(parse_classes("text-white").color, Some(Color::WHITE));
        assert_eq!(
            parse_classes("text-emerald-600").color,
            Some(Color::hex(0x059669))
        );
    }

    #[test]
    fn border_color() {
        assert_eq!(
            parse_classes("border-gray-300").border_color,
            Some(Color::hex(0xd1d5db))
        );
    }

    // ------------------------------------------------------------------
    // Borders & rounded
    // ------------------------------------------------------------------

    #[test]
    fn border_width_keyword() {
        assert_eq!(
            parse_classes("border").border_width,
            Some(UnitValue::px(1.0))
        );
        assert_eq!(
            parse_classes("border-2").border_width,
            Some(UnitValue::px(2.0))
        );
    }

    #[test]
    fn border_style_keyword() {
        assert_eq!(
            parse_classes("border-dashed").border_style,
            Some(BorderStyle::Dashed)
        );
    }

    #[test]
    fn border_per_side() {
        assert_eq!(
            parse_classes("border-t").border_top_width,
            Some(UnitValue::px(1.0))
        );
        assert_eq!(
            parse_classes("border-x-2").border_left_width,
            Some(UnitValue::px(2.0))
        );
        assert_eq!(
            parse_classes("border-x-2").border_right_width,
            Some(UnitValue::px(2.0))
        );
    }

    #[test]
    fn rounded_scale() {
        assert_eq!(
            parse_classes("rounded").border_top_left_radius,
            Some(UnitValue::rem(0.25))
        );
        assert_eq!(
            parse_classes("rounded-lg").border_top_left_radius,
            Some(UnitValue::rem(0.5))
        );
        assert_eq!(
            parse_classes("rounded-full").border_top_left_radius,
            Some(UnitValue::px(9999.0))
        );
    }

    #[test]
    fn rounded_per_corner() {
        assert_eq!(
            parse_classes("rounded-tl-lg").border_top_left_radius,
            Some(UnitValue::rem(0.5))
        );
        assert_eq!(
            parse_classes("rounded-t-lg").border_top_right_radius,
            Some(UnitValue::rem(0.5))
        );
    }

    // ------------------------------------------------------------------
    // Effects
    // ------------------------------------------------------------------

    #[test]
    fn opacity_scale() {
        assert_eq!(parse_classes("opacity-50").opacity, Some(0.5));
        assert_eq!(parse_classes("opacity-100").opacity, Some(1.0));
        assert_eq!(parse_classes("opacity-0").opacity, Some(0.0));
    }

    #[test]
    fn shadows() {
        assert_eq!(parse_classes("shadow-md").box_shadow.len(), 1);
        assert!(parse_classes("shadow-none").box_shadow.is_empty());
        assert!(parse_classes("shadow-inner").box_shadow[0].inset);
    }

    // ------------------------------------------------------------------
    // Transitions
    // ------------------------------------------------------------------

    #[test]
    fn transition_preset() {
        let s = parse_classes("transition");
        assert_eq!(s.transition_duration, Some(0.15));
        assert!(s.transition_property.as_deref().unwrap().contains("color"));
    }

    #[test]
    fn duration() {
        assert_eq!(parse_classes("duration-300").transition_duration, Some(0.3));
        assert_eq!(
            parse_classes("duration-1000").transition_duration,
            Some(1.0)
        );
    }

    // ------------------------------------------------------------------
    // Arbitrary values
    // ------------------------------------------------------------------

    #[test]
    fn arbitrary_width() {
        let w = unwrap_uv(&parse_classes("w-[137px]").width);
        approx(w.to_px().unwrap(), 137.0);
    }

    #[test]
    fn arbitrary_color() {
        assert_eq!(
            parse_classes("bg-[#ff00ff]").background_color,
            Some(Color::hex(0xff00ff))
        );
        assert_eq!(
            parse_classes("text-[#123456]").color,
            Some(Color::hex(0x123456))
        );
    }

    #[test]
    fn arbitrary_padding() {
        let s = parse_classes("p-[2.5rem]");
        assert_eq!(s.padding[0].to_px().unwrap(), 40.0);
    }

    // ------------------------------------------------------------------
    // Integration
    // ------------------------------------------------------------------

    #[test]
    fn realistic_card() {
        let s = parse_classes("flex flex-col gap-4 p-6 bg-white rounded-lg shadow-md");
        assert_eq!(s.display, vec![Display::Flex]);
        assert_eq!(s.flex_direction, Some(FlexDirection::Column));
        assert_eq!(s.gap, vec![UnitValue::rem(1.0)]);
        assert_eq!(s.padding, vec![UnitValue::rem(1.5)]);
        assert_eq!(s.background_color, Some(Color::WHITE));
        assert_eq!(s.border_top_left_radius, Some(UnitValue::rem(0.5)));
        assert_eq!(s.box_shadow.len(), 1);
    }

    #[test]
    fn realistic_button() {
        let s = parse_classes(
            "inline-flex items-center justify-center px-4 py-2 \
             bg-blue-600 text-white font-semibold rounded-md \
             hover:ignored transition-colors duration-200",
        );
        assert_eq!(s.display, vec![Display::InlineFlex]);
        assert_eq!(s.align_items, Some(AlignItems::Center));
        assert_eq!(s.justify_content, Some(JustifyContent::Center));
        assert_eq!(s.padding_left, Some(UnitValue::rem(1.0)));
        assert_eq!(s.padding_top, Some(UnitValue::rem(0.5)));
        assert_eq!(s.background_color, Some(Color::hex(0x2563eb)));
        assert_eq!(s.color, Some(Color::WHITE));
        assert_eq!(s.font_weight, Some(FontWeight::W600));
        assert_eq!(s.border_top_left_radius, Some(UnitValue::rem(0.375)));
        assert_eq!(s.transition_duration, Some(0.2));
        // `hover:ignored` was silently dropped
    }
}
