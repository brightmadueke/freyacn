//! # FreyaCN Label Component
//!
//! A text primitive driven by the same [`Style`] system as `Div`.
//!
//! ## Structure
//!
//! `Label` renders a `rect()` wrapper around a `label()`:
//!
//! ```text
//! rect()        <- box stylesheet, `.interactive(true)`, events, a11y
//!   └─ label()  <- text stylesheet
//! ```
//!
//! The wrapper is always present. `label()` isn't a reliable pointer
//! target in 0.5.0-rc.10, so the wrapper carries `.interactive(true)`
//! and the pointer handlers.
//!
//! ## Hover tracking
//!
//! `State<u8>` with sentinel values `IDLE` and `HOVERED`. Neither is
//! `u8::default()`, so the reactive store never skips the write. Both
//! `on_pointer_leave` and `on_pointer_out` clear the state.
//!
//! ## Inheriting from a parent
//!
//! `inherit_state` defaults to `true`: `Label` inherits its parent
//! `Div`'s hover/focus/active/disabled flags so `hover:underline`
//! follows the parent button. Combined with `Div`'s
//! `inherit_state = false` default, this gives "the button is hovered
//! → its labels react" without "the row is hovered → every child
//! reacts".
//! ## Interactive by default? No.
//!
//! `Label` is **non-interactive** unless the caller opts in with
//! `.pointer-events-auto()`. It inherits hover / focus / active /
//! disabled from the parent `Div` via `DivActiveState`, so a label
//! inside a `Button` reacts to the button's hover without stealing it.
//!
//! If a label were interactive by default, moving the pointer onto
//! its text would fire `pointer_leave` on the parent button, dropping
//! the button's `hover:*` stylesheet. Freya's hit tester routes each event
//! to the topmost interactive element, and the label would win. See
//! `Div`'s module docs for the full pattern.
//!
//! Standalone usage:
//!
//! ```ignore
//! Label()
//!     .text("Hover me")
//!     .style("pointer-events-auto hover:underline")
//! ```

use crate::extensions::{
    DivActiveState, DivPosition, GroupState, StateFlags, TextEllipsis, TextTransform, Visibility,
    Whitespace,
};
use crate::prelude::*;

const IDLE: u8 = 1;
const HOVERED: u8 = 2;

// ==============================================================
// Helpers
// ==============================================================

fn transform_text(text: &str, transform: Option<TextTransform>) -> String {
    match transform {
        Some(TextTransform::Uppercase) => text.to_uppercase(),
        Some(TextTransform::Lowercase) => text.to_lowercase(),
        Some(TextTransform::Capitalize) => {
            let mut out = String::with_capacity(text.len());
            let mut at_word_start = true;
            for ch in text.chars() {
                if ch.is_whitespace() {
                    at_word_start = true;
                    out.push(ch);
                } else if at_word_start {
                    out.extend(ch.to_uppercase());
                    at_word_start = false;
                } else {
                    out.push(ch);
                }
            }
            out
        }
        Some(TextTransform::None) | None => text.to_owned(),
    }
}

const INHERITED_FLAGS: [StateFlags; 5] = [
    StateFlags::HOVER,
    StateFlags::ACTIVE,
    StateFlags::FOCUS,
    StateFlags::FOCUS_VISIBLE,
    StateFlags::DISABLED,
];

// ==============================================================
// Component
// ==============================================================

#[component(
    props(
        text: String = String::new(),
        disabled: bool = false,
        focusable: bool = false,
        inherit_state: bool = true,
        custom_a11y_id: Option<AccessibilityId> = None,
        a11y_role: Option<AccessibilityRole> = None,
        a11y_labelled_by: Option<AccessibilityId> = None,
        a11y_hidden: Option<bool> = None,
    ),
    ext(style, event_handlers, accessibility, key),
)]
#[struct_fields(
    on_hover: EventHandler<Event<PointerEventData>>,
    on_press: EventHandler<Event<PressEventData>>,
)]
#[derive(PartialEq, Clone)]
pub fn Label() {
    let hovering = use_state(|| IDLE);
    let pressed = use_state(|| IDLE);
    let a11y_id = use_a11y();
    let focus = use_focus(a11y_id);

    let group_state: Option<GroupState> = try_consume_context();
    let position: Option<DivPosition> = try_consume_context();
    let inherited: Option<DivActiveState> = try_consume_context();

    let has_states = !style.states.is_empty();

    let mut active = StateFlags::NONE;

    {
        let state = use_cn_theme();
        if state.read().is_dark {
            active = active.or(StateFlags::DARK);
        }
    }

    if *disabled {
        active = active.or(StateFlags::DISABLED);
    }

    if has_states {
        if *hovering.read() == HOVERED {
            active = active.or(StateFlags::HOVER);
        }
        if *pressed.read() == HOVERED {
            active = active.or(StateFlags::ACTIVE);
        }
        if focus().is_focused() {
            active = active.or(StateFlags::FOCUS).or(StateFlags::FOCUS_VISIBLE);
        }

        if let Some(pos) = position.as_ref() {
            if pos.total == 1 {
                active = active.or(StateFlags::ONLY);
            } else {
                if pos.index == 0 {
                    active = active.or(StateFlags::FIRST);
                }
                if pos.index == pos.total - 1 {
                    active = active.or(StateFlags::LAST);
                }
            }
            if pos.index % 2 == 0 {
                active = active.or(StateFlags::ODD);
            } else {
                active = active.or(StateFlags::EVEN);
            }
        }

        if let Some(grp) = group_state.as_ref() {
            if *grp.hovered.read() {
                active = active.or(StateFlags::GROUP_HOVER);
            }
            if *grp.focused.read() {
                active = active.or(StateFlags::GROUP_FOCUS);
            }
        }

        if *inherit_state {
            if let Some(parent) = inherited.as_ref() {
                let parent_flags = *parent.flags.read();
                for flag in INHERITED_FLAGS {
                    if flag.is_subset_of(parent_flags) {
                        active = active.or(flag);
                    }
                }
            }
        }
    }

    let style = style.clone().with_active_states(active);

    if matches!(style.visibility, Some(Visibility::Collapse)) {
        return rect()
            .width(Size::px(0.0))
            .height(Size::px(0.0))
            .into_element();
    }

    let mut lbl = label().text(transform_text(text, style.text_transform));

    if let Some(colour) = style.text_color {
        lbl = lbl.color(colour);
    }
    if let Some(size) = style.font_size {
        lbl = lbl.font_size(size);
    }
    if let Some(slant) = style.font_slant {
        lbl = lbl.font_slant(slant);
    }
    if let Some(weight) = style.font_weight {
        lbl = lbl.font_weight(weight);
    }
    if let Some(width) = style.font_width {
        lbl = lbl.font_width(width);
    }
    if let Some(align) = style.text_align {
        lbl = lbl.text_align(align);
    }
    if let Some(deco) = style.text_decoration {
        lbl = lbl.text_decoration(deco);
    }
    if let Some(height) = style.text_height {
        lbl = lbl.text_height(height);
    }
    for family in &style.font_families {
        lbl = lbl.font_family(family.clone());
    }
    for shadow in &style.text_shadows {
        lbl = lbl.text_shadow(shadow.clone());
    }
    if let Some(height) = style.text_height {
        lbl = lbl.text_height(height);
    }
    // NEW: tracking / letter-spacing
    if let Some(spacing) = style.letter_spacing {
        lbl = lbl.letter_spacing(spacing);
    }
    for family in &style.font_families {
        lbl = lbl.font_family(family.clone());
    }

    let max_lines: Option<usize> = style
        .line_clamp
        .map(|n| n as usize)
        .or_else(|| matches!(style.whitespace, Some(Whitespace::Nowrap)).then_some(1));
    lbl = lbl.max_lines(max_lines);

    let ellipsis = style
        .text_ellipsis
        .or_else(|| style.line_clamp.map(|_| TextEllipsis::Ellipsis));
    if let Some(ellipsis) = ellipsis {
        lbl = lbl.text_overflow(match ellipsis {
            TextEllipsis::Clip => TextOverflow::Clip,
            TextEllipsis::Ellipsis => TextOverflow::Ellipsis,
        });
    }

    lbl = lbl.line_height(style.line_height);

    if style.width.is_some() {
        lbl = lbl.width(Size::percent(100.0));
    }

    let base_opacity = style.opacity.unwrap_or(1.0);
    let opacity = if matches!(style.visibility, Some(Visibility::Hidden)) {
        0.0
    } else {
        base_opacity
    };

    let mut wrapper = rect()
        // Non-interactive by default. Hover is inherited from the
        // parent Div via `DivActiveState`. If the label were
        // interactive, it would become the topmost hit-test target
        // when the pointer moves onto it, and Freya would fire
        // `pointer_leave` on the parent — dropping the button's or
        // trigger's hover state. Opt in with `.style("pointer-events-auto")`
        // when a standalone label needs its own hover.
        .interactive(style.pointer_events.unwrap_or(false))
        .background(style.background.clone().unwrap_or_default())
        .corner_radius(style.corner_radius.unwrap_or_default())
        .border(style.to_border())
        .padding(style.padding.unwrap_or_default())
        .margin(style.margin.unwrap_or_default())
        .position(style.to_position().unwrap_or_default())
        .overflow(style.overflow.unwrap_or_default())
        .opacity(opacity);

    if let Some(size) = style.width.clone() {
        wrapper = wrapper.width(size);
    }
    if let Some(size) = style.height.clone() {
        wrapper = wrapper.height(size);
    }
    if let Some(size) = style.min_width.clone() {
        wrapper = wrapper.min_width(size);
    }
    if let Some(size) = style.min_height.clone() {
        wrapper = wrapper.min_height(size);
    }
    if let Some(size) = style.max_width.clone() {
        wrapper = wrapper.max_width(size);
    }
    if let Some(size) = style.max_height.clone() {
        wrapper = wrapper.max_height(size);
    }
    if let Some(size) = style.visible_width.clone() {
        wrapper = wrapper.visible_width(size);
    }
    if let Some(size) = style.visible_height.clone() {
        wrapper = wrapper.visible_height(size);
    }
    if let Some(z) = style.z_index.clone() {
        let clamped = z.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        wrapper = wrapper.layer(clamped);
    }
    if let Some(deg) = style.rotate.clone() {
        wrapper = wrapper.rotate(deg);
    }
    if let Some(scale) = style.scale.clone() {
        wrapper = wrapper.scale(scale);
    }
    if let Some(origin) = style.transform_origin.clone() {
        wrapper = wrapper.transform_origin(origin);
    }
    if let Some(radius) = style.blur.clone() {
        wrapper = wrapper.backdrop_blur(radius);
    }
    if let Some(x) = style.offset_x.clone() {
        wrapper = wrapper.offset_x(x);
    }
    if let Some(y) = style.offset_y.clone() {
        wrapper = wrapper.offset_y(y);
    }
    for shadow in &style.shadows.clone() {
        wrapper = wrapper.shadow(shadow.clone());
    }

    let effective_a11y_id = custom_a11y_id.clone().unwrap_or_else(|| a11y_id.clone());

    wrapper = wrapper
        .accessibility(accessibility.clone())
        .a11y_id(effective_a11y_id)
        .a11y_focusable(*focusable && !*disabled);

    if a11y_role.is_some() || a11y_labelled_by.is_some() || a11y_hidden.is_some() {
        let role = a11y_role.clone();
        let labelled_by = a11y_labelled_by.clone();
        let hidden = *a11y_hidden;
        wrapper = wrapper.a11y_builder(move |b| {
            if let Some(r) = role {
                b.set_role(r);
            }
            if let Some(l) = &labelled_by {
                b.set_labelled_by([l.clone()]);
            }
            if let Some(h) = hidden {
                if h {
                    b.set_hidden();
                }
            }
        });
    }

    wrapper = wrapper.event_handlers(event_handlers.clone());

    if !*disabled {
        let on_hover = on_hover.clone();
        let on_press = on_press.clone();

        let mut h_enter = hovering;
        let mut h_leave = hovering;
        let mut p_down = pressed;
        let mut p_leave = pressed;

        wrapper = wrapper
            .on_pointer_enter(move |e: Event<PointerEventData>| {
                *h_enter.write() = HOVERED;
                if let Some(handler) = &on_hover {
                    handler.call(e);
                }
            })
            .on_pointer_leave(move |_| {
                *h_leave.write() = IDLE;
                *p_leave.write() = IDLE;
            })
            .on_pointer_down(move |_| {
                *p_down.write() = HOVERED;
            })
            .on_press(move |e: Event<PressEventData>| {
                if let Some(handler) = &on_press {
                    handler.call(e);
                }
            });
    }

    wrapper.child(lbl).into_element()
}

// ==============================================================
// Builder-method extension on the generated `LabelComponent`
// ==============================================================

impl LabelComponent {
    pub fn on_hover(mut self, on_hover: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.on_hover = Some(on_hover.into());
        self
    }
    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }
}
