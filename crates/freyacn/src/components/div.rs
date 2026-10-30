//! # Div — styled primitive wrapper
//!
//! Renders a Freya `rect()` after folding the `ext`-provided [`Style`]
//! into concrete values. Handles scrolling, interaction state,
//! animations, accessibility forwarding, per-side borders, grid layout,
//! and propagates its active state to descendants via `DivActiveState`.
//!
//! ## Grid
//!
//! `Div` acts as both a grid **container** and a grid **item**:
//!
//! - `Div().style("grid-cols-3")` — the container. Provides
//!   `DivGridColsCtx(Some(3))` to direct children and switches to
//!   `Content::Wrap + Direction::Horizontal + spacing` (the flex-wrap
//!   approximation of CSS Grid; the torin layout engine has no native
//!   grid primitive).
//! - `Div().style("col-span-2")` inside the above — the item. Reads
//!   `DivGridColsCtx` from the nearest `Div` ancestor and sizes itself
//!   to `(100 / cols) * span %`. `col_span` defaults to 1 when unset.
//!
//! **Constraints:**
//!
//! - **Grid children must be `Div`s.** A `Label`, `Icon`, or raw `rect`
//!   inside a grid has no grid awareness and gets `Size::auto()`. Wrap
//!   them: `Div().col_span(1).child(label())`. Same constraint the
//!   standalone `Grid`/`GridItem` components have.
//! - **`space_x` / `space_y` are ignored in grid mode.** Grid spacing
//!   comes from `gap-*` (`Style::spacing`), which is already applied.
//! - **`grid_rows` / `row_span` are stored but not applied.** torin has
//!   no row-track primitive; same limitation as `Grid`/`GridItem`.
//! - **Only direct children are items.** A `Div` between the grid and a
//!   grandchild shadows the context (`Some(None)`), matching CSS.
//!
//! ## Hover tracking
//!
//! Hover and press state use `State<u8>` with sentinel values `IDLE`
//! and `HOVERED`. Neither is `u8::default()` (`0`), so the reactive
//! store never skips the write. Both `on_pointer_leave` and
//! `on_pointer_out` clear the state.
//!
//! ## State inheritance and interactive children
//!
//! Freya's hit tester resolves each pointer event to the **topmost
//! interactive** element under the pointer. If both a parent `Div`
//! and one of its children are interactive, moving the pointer onto
//! the child fires `pointer_leave` on the parent and the parent's
//! hover state clears — even though the pointer is still visually
//! "inside" the parent.
//!
//! The pattern across this crate is therefore:
//!
//! * The **outermost** element of a widget (`Button`, `AccordionTrigger`,
//!   any `Div` with `hover:` / `active:` stylesheet) is interactive. It
//!   owns hover and press state.
//! * **Children are non-interactive** (`pointer-events-none` in DOM
//!   terms). They inherit the parent's state via `DivActiveState` and
//!   react to it with the same `hover:` / `active:` / `focus:`
//!   variants.
//! * Callers who genuinely need a self-hovering child (a standalone
//!   `Label` in a list, an `Icon` acting as a button) opt in with
//!   `.pointer-events-auto()` on that child's style.
//!
//! `Div`'s `inherit_state` prop (default `false`) controls whether a
//! `Div` inherits its parent's flags. `Label` sets it to `true` — a
//! label always follows its parent's hover — while nested layout `Div`s
//! keep the `false` default so a hovered row doesn't light up every
//! child.
//!
//! ## `space-x-*` / `space-y-*`
//!
//! `space_x` and `space_y` are applied as `margin-left` / `margin-top`
//! on every child **except the first**, matching Tailwind's
//! `space-x-*` / `space-y-*` semantics. Negative values
//! (`-space-x-2`) produce overlap. `gap-*` (via `Style::spacing`) is
//! applied independently on the parent, so `gap-2 space-x-1` produces
//! both a 8px gap and a 4px left margin on non-first children.
//!
//! In **grid mode** (`grid_cols` set), `space_x` / `space_y` are not
//! applied — use `gap-*` for grid spacing.

use crate::extensions::{
    AnimationName, DivActiveState, DivGridColsCtx, DivPosition, GroupState, TextEllipsis,
};
use crate::prelude::*;
use freya::animation::{AnimNum, OnCreation, OnFinish, use_animation};

const IDLE: u8 = 1;
const HOVERED: u8 = 2;

fn grid_cell_pct(cols: u32) -> f32 {
    100.0 / cols.max(1) as f32
}

fn apply_size_to_scroll_view(style: &Style, mut sv: ScrollView) -> ScrollView {
    if let Some(w) = style.width.clone() {
        sv = sv.width(w);
    }
    if let Some(h) = style.height.clone() {
        sv = sv.height(h);
    }
    if let Some(w) = style.max_width.clone() {
        sv = sv.max_width(w);
    }
    if let Some(h) = style.max_height.clone() {
        sv = sv.max_height(h);
    }
    sv
}

fn apply_size_to_rect(style: &Style, mut r: Rect) -> Rect {
    if let Some(w) = style.width.clone() {
        r = r.width(w);
    }
    if let Some(h) = style.height.clone() {
        r = r.height(h);
    }
    if let Some(w) = style.min_width.clone() {
        r = r.min_width(w);
    }
    if let Some(h) = style.min_height.clone() {
        r = r.min_height(h);
    }
    if let Some(w) = style.max_width.clone() {
        r = r.max_width(w);
    }
    if let Some(h) = style.max_height.clone() {
        r = r.max_height(h);
    }
    r
}

#[component(
    props(
        disabled: bool = false,
        focusable: bool = false,
        inherit_state: bool = false,
        custom_a11y_id: Option<AccessibilityId> = None,
        a11y_role: Option<AccessibilityRole> = None,
        a11y_expanded: Option<bool> = None,
        a11y_labelled_by: Option<AccessibilityId> = None,
        a11y_hidden: Option<bool> = None,
    ),
    ext(style, children, event_handlers, accessibility, key),
)]
#[struct_fields(
    on_hover: EventHandler<Event<PointerEventData>>,
    on_press: EventHandler<Event<PressEventData>>,
)]
#[derive(PartialEq, Clone)]
pub fn Div() {
    let hovering = use_state(|| IDLE);
    let pressed = use_state(|| IDLE);
    let a11y_id = use_a11y();
    let focus = use_focus(a11y_id);

    let group_state: Option<GroupState> = try_consume_context();
    let position: Option<DivPosition> = try_consume_context();
    let parent_state: Option<DivActiveState> = try_consume_context();
    let parent_grid: Option<DivGridColsCtx> = try_consume_context();

    let state = use_cn_theme();
    let mut active = StateFlags::NONE;
    if state.read().is_dark {
        active = active.or(StateFlags::DARK);
    }

    if *hovering.read() == HOVERED {
        active = active.or(StateFlags::HOVER);
    }
    if *pressed.read() == HOVERED {
        active = active.or(StateFlags::ACTIVE);
    }
    if focus().is_focused() {
        active = active.or(StateFlags::FOCUS).or(StateFlags::FOCUS_VISIBLE);
    }
    if *disabled {
        active = active.or(StateFlags::DISABLED);
    }

    if *inherit_state {
        if let Some(ps) = &parent_state {
            active = active.or(*ps.flags.read());
        }
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

    let mut flags_state = use_state(|| StateFlags::NONE);
    if *flags_state.peek() != active {
        *flags_state.write() = active;
    }
    use_provide_context(move || DivActiveState { flags: flags_state });

    // ── Grid context ──────────────────────────────────────────
    //
    // Every `Div` provides `DivGridColsCtx`, even when it has no
    // `grid_cols`. The `Some(None)` value *shadows* an ancestor grid
    // so grandchildren don't treat themselves as grid items — only
    // direct children of a `Div` with `grid_cols(n)` are items.
    let own_grid_cols = style.grid_cols;
    let mut grid_cols_state = use_state::<Option<u32>>(|| None);
    if *grid_cols_state.peek() != own_grid_cols {
        *grid_cols_state.write() = own_grid_cols;
    }
    use_provide_context(move || DivGridColsCtx {
        cols: grid_cols_state,
    });

    // ── Grid item width ───────────────────────────────────────
    //
    // If we're a direct child of a grid container, compute our width
    // from `col_span` (default 1) and the parent's column count. A
    // user-supplied `width` still wins — same as CSS, where an
    // explicit `width` overrides `grid-template-columns`.
    let grid_item_width: Option<Size> = parent_grid.and_then(|parent| {
        let cols_opt = *parent.cols.peek();
        cols_opt.map(|cols| {
            let cols = cols.max(1);
            let span = style.col_span.unwrap_or(1).max(1).min(cols);
            Size::percent((100.0 / cols as f32) * span as f32)
        })
    });

    let style = style.clone().with_active_states(active);

    let anim_spec = style.animation.clone();
    let is_animating = anim_spec.as_ref().map_or(false, |s| !s.is_none());
    let duration_ms = anim_spec
        .as_ref()
        .map_or(1000u32, |s| s.duration_ms.max(1.0) as u32);
    let base_opacity = style.opacity.unwrap_or(1.0);

    let animation = use_animation(move |conf| {
        if is_animating {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::restart());
        }
        AnimNum::new(0.0_f32, 1.0_f32).time(duration_ms as u64)
    });

    let t: f32 = if is_animating {
        animation.get().value()
    } else {
        0.0
    };
    let is_scroll = style.scroll_axis.is_some();

    let border_color = style.border_color.or(style.divide_color);
    let has_uniform =
        style.border_width.is_some() || style.divide_x.is_some() || style.divide_y.is_some();
    let has_per_side = !has_uniform
        && border_color.is_some()
        && (style.border_top_width.is_some()
            || style.border_bottom_width.is_some()
            || style.border_left_width.is_some()
            || style.border_right_width.is_some());

    let is_vertical = matches!(style.direction.as_ref(), Some(Direction::Vertical) | None);

    let (main_pre_w, main_post_w, cross_pre_w, cross_post_w) = if is_vertical {
        (
            style.border_top_width,
            style.border_bottom_width,
            style.border_left_width,
            style.border_right_width,
        )
    } else {
        (
            style.border_left_width,
            style.border_right_width,
            style.border_top_width,
            style.border_bottom_width,
        )
    };

    let padding = style.padding.unwrap_or_default();
    let has_padding = padding.top() > 0.0
        || padding.bottom() > 0.0
        || padding.left() > 0.0
        || padding.right() > 0.0;

    let needs_padding_wrap =
        has_per_side && has_padding && (main_pre_w.is_some() || main_post_w.is_some());
    let has_cross = cross_pre_w.is_some() || cross_post_w.is_some();
    let needs_cross_wrap = has_per_side && has_cross;

    let spacing = style.spacing.unwrap_or(0.0);
    let total = children.len();
    let grid_cols = style.grid_cols;

    let space_x_val = style.space_x.unwrap_or(0.0);
    let space_y_val = style.space_y.unwrap_or(0.0);

    let has_rounded_corners = style.corner_radius.is_some();

    let mut visual = rect()
        .background(style.background.clone().unwrap_or_default())
        .corner_radius(style.corner_radius.unwrap_or_default())
        .border(if has_per_side {
            None
        } else {
            style.to_border()
        })
        .padding(if needs_padding_wrap {
            Gaps::default()
        } else {
            padding
        })
        .margin(style.margin.unwrap_or_default())
        .position(style.to_position().unwrap_or_default())
        .overflow(if has_per_side && has_rounded_corners {
            Overflow::Clip
        } else {
            style.overflow.unwrap_or_default()
        })
        .opacity(base_opacity)
        .interactive(style.pointer_events.unwrap_or(true));

    visual = visual
        .direction(style.direction.clone().unwrap_or_default())
        .spacing(if needs_padding_wrap { 0.0 } else { spacing });

    if !needs_padding_wrap {
        visual = visual
            .main_align(style.main_align.clone().unwrap_or_default())
            .cross_align(style.cross_align.clone().unwrap_or_default())
            .content(style.content.clone().unwrap_or_default());
    }

    if let (true, Some(w), Some(color)) = (has_per_side, main_pre_w, border_color) {
        let strip_is_vertical = !is_vertical;
        let mut strip = if strip_is_vertical {
            rect()
                .width(Size::px(w))
                .height(Size::percent(100.0))
                .background(color)
        } else {
            rect()
                .width(Size::percent(100.0))
                .height(Size::px(w))
                .background(color)
        };
        strip = strip.margin(if needs_padding_wrap {
            Gaps::default()
        } else if is_vertical {
            Gaps::new(0.0, 0.0, -spacing, 0.0)
        } else {
            Gaps::new(0.0, -spacing, 0.0, 0.0)
        });
        visual = visual.child(strip);
    }

    if needs_padding_wrap {
        let mut content = rect()
            .direction(style.direction.clone().unwrap_or_default())
            .main_align(style.main_align.clone().unwrap_or_default())
            .cross_align(style.cross_align.clone().unwrap_or_default())
            .content(style.content.clone().unwrap_or_default())
            .spacing(spacing)
            .padding(padding);

        if is_vertical {
            content = content.width(Size::percent(100.0));
        } else {
            content = content.height(Size::percent(100.0));
        }

        if let Some(cols) = grid_cols {
            content = content
                .content(Content::Wrap { wrap_spacing: None })
                .direction(Direction::Horizontal);
            let cell_pct = grid_cell_pct(cols);
            for (i, child) in children.iter().enumerate() {
                content = content.child(
                    rect()
                        .width(Size::percent(cell_pct))
                        .content(Content::Fit)
                        .child(
                            WithPosition()
                                .index(i)
                                .total(total)
                                .space_x(space_x_val)
                                .space_y(space_y_val)
                                .child(child.clone())
                                .into_element(),
                        ),
                );
            }
        } else {
            for (i, child) in children.iter().enumerate() {
                content = content.child(
                    WithPosition()
                        .index(i)
                        .total(total)
                        .space_x(space_x_val)
                        .space_y(space_y_val)
                        .child(child.clone())
                        .into_element(),
                );
            }
        }

        visual = visual.child(content);
    } else if let Some(_cols) = grid_cols {
        // ── Grid container ────────────────────────────────────
        //
        // `Content::Wrap + Direction::Horizontal + spacing` is the
        // flex-wrap approximation of CSS Grid. Children render
        // *directly* — no fixed-width `rect()` wrapper — so each
        // `Div` item can size itself from `col_span` via the
        // `DivGridColsCtx` we provided above.
        //
        // `WithPosition` still wraps each child (for `first:` /
        // `last:` / `DivPosition` context) but with `bare(true)` so
        // it doesn't add a width-bearing rect around the item.
        //
        // `space_x` / `space_y` are intentionally not applied in grid
        // mode — use `gap-*` for grid spacing instead.
        visual = visual
            .content(Content::Wrap { wrap_spacing: None })
            .direction(Direction::Horizontal);
        for (i, child) in children.iter().enumerate() {
            visual = visual.child(
                WithPosition()
                    .index(i)
                    .total(total)
                    .bare(true)
                    .child(child.clone())
                    .into_element(),
            );
        }
    } else {
        for (i, child) in children.iter().enumerate() {
            visual = visual.child(
                WithPosition()
                    .index(i)
                    .total(total)
                    .space_x(space_x_val)
                    .space_y(space_y_val)
                    .child(child.clone())
                    .into_element(),
            );
        }
    }

    if let (true, Some(w), Some(color)) = (has_per_side, main_post_w, border_color) {
        let strip_is_vertical = !is_vertical;
        let mut strip = if strip_is_vertical {
            rect()
                .width(Size::px(w))
                .height(Size::percent(100.0))
                .background(color)
        } else {
            rect()
                .width(Size::percent(100.0))
                .height(Size::px(w))
                .background(color)
        };
        strip = strip.margin(if needs_padding_wrap {
            Gaps::default()
        } else if is_vertical {
            Gaps::new(-spacing, 0.0, 0.0, 0.0)
        } else {
            Gaps::new(0.0, 0.0, 0.0, -spacing)
        });
        visual = visual.child(strip);
    }

    if !is_scroll && !needs_cross_wrap {
        visual = apply_size_to_rect(&style, visual);
    }

    // Grid item width applied after the style-driven sizing so the
    // user's explicit `width` wins over the span-computed width.
    if style.width.is_none() {
        if let Some(w) = grid_item_width {
            visual = visual.width(w);
        }
    }

    if let Some(c) = style.text_color {
        visual = visual.color(c);
    }
    if let Some(s) = style.font_size {
        visual = visual.font_size(s);
    }
    if let Some(s) = style.font_slant {
        visual = visual.font_slant(s);
    }
    if let Some(s) = style.font_weight {
        visual = visual.font_weight(s);
    }
    if let Some(a) = style.text_align {
        visual = visual.text_align(a);
    }
    if let Some(d) = style.text_decoration {
        visual = visual.text_decoration(d);
    }
    if let Some(w) = style.font_width {
        visual = visual.font_width(w);
    }
    if let Some(h) = style.text_height {
        visual = visual.text_height(h);
    }
    for family in &style.font_families {
        visual = visual.font_family(family.clone());
    }
    for shadow in &style.text_shadows {
        visual = visual.text_shadow(shadow.clone());
    }

    if let Some(ellipsis) = style.text_ellipsis {
        visual = visual.text_overflow(match ellipsis {
            TextEllipsis::Clip => TextOverflow::Clip,
            TextEllipsis::Ellipsis => TextOverflow::Ellipsis,
        });
    }
    if let Some(z) = style.z_index.clone() {
        let clamped = z.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        visual = visual.layer(clamped);
    }
    if let Some(w) = style.visible_width.clone() {
        visual = visual.visible_width(w);
    }
    if let Some(h) = style.visible_height.clone() {
        visual = visual.visible_height(h);
    }
    if let Some(deg) = style.rotate.clone() {
        visual = visual.rotate(deg);
    }
    if let Some(s) = style.scale.clone() {
        visual = visual.scale(s);
    }
    if let Some(origin) = style.transform_origin.clone() {
        visual = visual.transform_origin(origin);
    }
    if let Some(radius) = style.blur.clone() {
        visual = visual.backdrop_blur(radius);
    }
    if let Some(x) = style.offset_x.clone() {
        visual = visual.offset_x(x);
    }
    if let Some(y) = style.offset_y.clone() {
        visual = visual.offset_y(y);
    }
    for shadow in &style.shadows.clone() {
        visual = visual.shadow(shadow.clone());
    }

    if is_animating {
        if let Some(spec) = anim_spec.as_ref() {
            visual = match spec.name {
                AnimationName::Spin => visual.rotate(t * 360.0),
                AnimationName::Pulse => {
                    let wave: f32 = 0.5 + 0.5 * (1.0 - (2.0_f32 * t - 1.0).abs());
                    visual.opacity(base_opacity * wave)
                }
                AnimationName::Ping => visual.opacity(base_opacity * (1.0 - t)).scale(1.0 + t),
                AnimationName::Bounce | AnimationName::None => visual,
            };
        }
    }

    let effective_a11y_id = if let Some(id) = custom_a11y_id {
        id.clone()
    } else {
        a11y_id
    };

    visual = visual
        .accessibility(accessibility.clone())
        .a11y_id(effective_a11y_id)
        .a11y_focusable(*focusable && !*disabled);

    {
        let role = a11y_role.clone();
        let exp = *a11y_expanded;
        let lbl = a11y_labelled_by.clone();
        let hid = *a11y_hidden;
        if role.is_some() || exp.is_some() || lbl.is_some() || hid.is_some() {
            visual = visual.a11y_builder(move |b| {
                if let Some(r) = role {
                    b.set_role(r);
                }
                if let Some(e) = exp {
                    b.set_expanded(e);
                }
                if let Some(l) = &lbl {
                    b.set_labelled_by([l.clone()]);
                }
                if let Some(h) = hid {
                    if h {
                        b.set_hidden();
                    }
                }
            });
        }
    }

    visual = visual.event_handlers(event_handlers.clone());

    if !*disabled {
        let on_hover = on_hover.clone();
        let on_press = on_press.clone();

        let mut h_enter = hovering;
        let mut h_leave = hovering;
        let mut p_down = pressed;
        let mut p_leave = pressed;

        visual = visual
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

    if needs_cross_wrap {
        if let Some(color) = border_color {
            let cross_dir = if is_vertical {
                Direction::Horizontal
            } else {
                Direction::Vertical
            };

            let mut wrapper = rect()
                .direction(cross_dir)
                .margin(style.margin.unwrap_or_default());

            if let Some(w) = style.width.clone() {
                wrapper = wrapper.width(w);
            } else if is_vertical {
                wrapper = wrapper.width(Size::percent(100.0));
            }

            if let Some(h) = style.height.clone() {
                wrapper = wrapper.height(h);
            }

            let strip_vertical = is_vertical;
            if let Some(w) = cross_pre_w {
                wrapper = wrapper.child(if strip_vertical {
                    rect()
                        .width(Size::px(w))
                        .height(Size::percent(100.0))
                        .background(color)
                } else {
                    rect()
                        .width(Size::percent(100.0))
                        .height(Size::px(w))
                        .background(color)
                });
            }

            let mut v = visual;
            if is_vertical {
                v = v.width(Size::flex(1.0));
            } else {
                v = v.height(Size::flex(1.0));
            }
            wrapper = wrapper.child(v);

            if let Some(w) = cross_post_w {
                wrapper = wrapper.child(if strip_vertical {
                    rect()
                        .width(Size::px(w))
                        .height(Size::percent(100.0))
                        .background(color)
                } else {
                    rect()
                        .width(Size::percent(100.0))
                        .height(Size::px(w))
                        .background(color)
                });
            }

            return if let Some(axis) = style.scroll_axis {
                let direction = match axis {
                    ScrollAxis::Vertical => Direction::Vertical,
                    ScrollAxis::Horizontal => Direction::Horizontal,
                    ScrollAxis::Both => Direction::Vertical,
                };
                ScrollView::new()
                    .direction(direction)
                    .show_scrollbar(style.scrollbar.unwrap_or(true))
                    .child(wrapper)
                    .into_element()
            } else {
                wrapper.into_element()
            };
        }
    }

    let final_element: Element = if let Some(axis) = style.scroll_axis {
        let direction = match axis {
            ScrollAxis::Vertical => Direction::Vertical,
            ScrollAxis::Horizontal => Direction::Horizontal,
            ScrollAxis::Both => Direction::Vertical,
        };
        let sv = ScrollView::new()
            .direction(direction)
            .show_scrollbar(style.scrollbar.unwrap_or(true))
            .child(visual);
        apply_size_to_scroll_view(&style, sv).into_element()
    } else {
        visual.into_element()
    };

    if is_animating {
        if let Some(spec) = anim_spec.as_ref() {
            if matches!(spec.name, AnimationName::Bounce) {
                let phase: f32 = (t * std::f32::consts::PI).sin().abs();
                let y: f32 = -phase * 16.0;
                return rect()
                    .width(Size::auto())
                    .height(Size::auto())
                    .offset_y(y)
                    .child(final_element)
                    .into_element();
            }
        }
    }

    final_element
}

impl DivComponent {
    pub fn on_hover(mut self, on_hover: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.on_hover = Some(on_hover.into());
        self
    }
    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }
}

#[component(
    props(
        index: usize = 0,
        total: usize = 0,
        space_x: f32 = 0.0,
        space_y: f32 = 0.0,
        bare: bool = false,
    ),
    ext(children),
)]
#[derive(PartialEq, Clone)]
fn WithPosition() -> impl IntoElement {
    let idx = *index;
    let tot = *total;
    let sx = *space_x;
    let sy = *space_y;
    let bare = *bare;

    use_provide_context(move || DivPosition {
        index: idx,
        total: tot,
    });

    // `bare` mode: the child is the layout element — no `rect()` wrapper.
    // Used by grid containers so `Div` items can self-size via
    // `DivGridColsCtx` without a fixed-width parent rect overriding them.
    if bare {
        if let Some(first) = children.first() {
            return first.clone();
        }
        return rect().into_element();
    }

    let mut r = rect();

    if idx > 0 && (sx != 0.0 || sy != 0.0) {
        r = r.margin(Gaps::new(sy, 0.0, 0.0, sx));
    }

    for child in children.iter() {
        r = r.child(child.clone());
    }
    r.into_element()
}
