//! # Accordion
//!
//! shadcn-shaped accordion built on Freya's native animation primitives.
//!
//! All animation values — durations and start/end values — live as
//! constants at the top of this file.
//!
//! ## Animation: linear
//!
//! Both the height and chevron animations are **linear** (no
//! `.function()` / `.ease()`). Front-loaded easing curves leave a slow
//! tail that reads as "jumped then stuck." Linear motion feels its full
//! duration and can be shortened without looking choppy.
//!
//! Both animations are started **inside the click handler** (matching
//! Freya's native `Accordion`), so the first click works.
//!
//! ## Dividers
//!
//! `AccordionItem` sets `border_b(1.0)` + `border_color` on its
//! `Style` for every item except the last. `Div` renders `border_b` on
//! a flex-column as a child rect — matching CSS `border-b last:border-b-0`.
//!
//! ## Hover underline
//!
//! `hover:underline` lives on the `Label` inside the trigger with
//! `flex-1` so the label fills the trigger row. Without `flex-1`, the
//! label only covers its own text bounds and the underline only fires
//! when the pointer is directly over the text. With `flex-1`, the label
//! extends to the trigger's full width and the underline fires anywhere
//! on the row.
//! ## Interactive unit
//!
//! The trigger `Div` is the interactive element. Its `Label` and
//! `AnimatedChevron` are non-interactive by default, so the trigger's
//! `hover:` / `active:` stylesheet stay applied when the pointer moves
//! onto the label or the chevron. Hover state is inherited by the
//! label via `DivActiveState`, which is what makes `hover:underline`
//! work without any `pointer-events` fiddling in the showcase.

use crate::prelude::*;
use freya::animation::{AnimNum, OnCreation, UseAnimation, use_animation};
use std::cell::RefCell;
use std::rc::Rc;

// ==============================================================
// Tuning knobs
// ==============================================================

/// Content height animation duration, in milliseconds.
const HEIGHT_ANIM_MS: u64 = 1;

/// Content height animation start / end values, in percent.
const HEIGHT_ANIM_FROM: f32 = 0.0;
const HEIGHT_ANIM_TO: f32 = 100.0;

/// Chevron rotation duration, in milliseconds.
const CHEVRON_ANIM_MS: u64 = 1;

/// Chevron rotation start / end angles, in degrees.
const CHEVRON_ANIM_FROM: f32 = 0.0;
const CHEVRON_ANIM_TO: f32 = 180.0;

// ==============================================================
// Public types
// ==============================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AccordionType {
    #[default]
    Single,
    Multiple,
}

// ==============================================================
// Contexts
// ==============================================================

#[derive(Clone)]
pub struct ItemHandle {
    pub open: State<bool>,
    pub height_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>>,
    pub chevron_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>>,
}

impl PartialEq for ItemHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.height_animation, &other.height_animation)
            && Rc::ptr_eq(&self.chevron_animation, &other.chevron_animation)
    }
}

#[derive(Clone)]
pub struct AccordionCoordination {
    pub type_: AccordionType,
    pub collapsible: bool,
    pub disabled: bool,
    pub currently_open: Rc<RefCell<Option<String>>>,
    pub registry: Rc<RefCell<FxHashMap<String, ItemHandle>>>,
}

impl PartialEq for AccordionCoordination {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
            && self.collapsible == other.collapsible
            && self.disabled == other.disabled
            && Rc::ptr_eq(&self.currently_open, &other.currently_open)
            && Rc::ptr_eq(&self.registry, &other.registry)
    }
}

#[derive(Clone)]
pub struct AccordionItemData {
    pub value: String,
    pub local_open: State<bool>,
    pub disabled: bool,
    pub trigger_id: AccessibilityId,
    pub content_id: AccessibilityId,
    pub height_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>>,
    pub chevron_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>>,
}

impl PartialEq for AccordionItemData {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.disabled == other.disabled
            && Rc::ptr_eq(&self.height_animation, &other.height_animation)
            && Rc::ptr_eq(&self.chevron_animation, &other.chevron_animation)
    }
}

// ==============================================================
// Animated primitives
// ==============================================================

#[component(props(open: bool = false), ext(children))]
#[derive(PartialEq, Clone)]
fn CollapsibleHeight() -> impl IntoElement {
    let is_open = *open;
    let item_data: Option<AccordionItemData> = try_consume_context();

    let animation = use_animation(move |conf| {
        if is_open {
            conf.on_creation(OnCreation::Finish);
        }
        AnimNum::new(HEIGHT_ANIM_FROM, HEIGHT_ANIM_TO).time(HEIGHT_ANIM_MS)
    });

    if let Some(data) = &item_data {
        *data.height_animation.borrow_mut() = Some(animation.clone());
    }

    let pct: f32 = animation.get().value();

    let mut r = rect()
        .width(Size::percent(100.0))
        .overflow(Overflow::Clip)
        .visible_height(VisibleSize::inner_percent(pct));

    for child in children.iter() {
        r = r.child(child.clone());
    }
    r
}

#[component(props(open: bool = false), ext())]
#[derive(PartialEq, Clone)]
fn AnimatedChevron() -> impl IntoElement {
    let is_open = *open;
    let item_data: Option<AccordionItemData> = try_consume_context();

    let animation = use_animation(move |conf| {
        if is_open {
            conf.on_creation(OnCreation::Finish);
        }
        AnimNum::new(CHEVRON_ANIM_FROM, CHEVRON_ANIM_TO).time(CHEVRON_ANIM_MS)
    });

    if let Some(data) = &item_data {
        *data.chevron_animation.borrow_mut() = Some(animation.clone());
    }

    let rotation: f32 = animation.get().value();

    rect()
        .rotate(rotation)
        .child(Icon().svg(chevron_down()).size_4())
}

// ==============================================================
// Root
// ==============================================================

#[component(
    props(
        type_: AccordionType = AccordionType::Single,
        collapsible: bool = true,
        disabled: bool = false,
        default_value: Option<Vec<String>> = None,
    ),
    ext(style, children, key),
)]
#[derive(PartialEq, Clone)]
pub fn Accordion() -> impl IntoElement {
    let type_ = *type_;
    let collapsible = *collapsible;
    let disabled = *disabled;

    let defaults = default_value.clone().unwrap_or_default();
    let defaults = match type_ {
        AccordionType::Single => defaults.into_iter().take(1).collect::<Vec<_>>(),
        AccordionType::Multiple => defaults,
    };

    let initially_open = defaults.first().cloned();
    let currently_open = use_hook(move || Rc::new(RefCell::new(initially_open)));

    let registry = use_hook(|| Rc::new(RefCell::new(FxHashMap::<String, ItemHandle>::default())));

    let defaults_for_ctx = defaults.clone();
    use_provide_context(move || defaults_for_ctx.clone());

    let coord = AccordionCoordination {
        type_,
        collapsible,
        disabled,
        currently_open,
        registry,
    };
    use_provide_context(move || coord.clone());

    Div()
        .style("flex flex-col w-full")
        .merge_style(style.clone())
        .children(children.clone())
}

impl AccordionComponent {
    pub fn item(
        self,
        value: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        self.child(
            AccordionItem()
                .value(value.into())
                .child(
                    AccordionTrigger().child(
                        Label()
                            .text(title.into())
                            .style("flex-1 h-auto hover:underline"),
                    ),
                )
                .child(
                    AccordionContent().child(
                        Label()
                            .text(body.into())
                            .style("text-sm text-muted-foreground leading-relaxed"),
                    ),
                ),
        )
    }
}

// ==============================================================
// Item
// ==============================================================

#[component(
    props(
        value: String = String::new(),
        disabled: bool = false,
    ),
    ext(style, children, key),
)]
#[derive(PartialEq, Clone)]
pub fn AccordionItem() -> impl IntoElement {
    let value = value.clone();
    let local_disabled = *disabled;

    let defaults: Option<Vec<String>> = try_consume_context();
    let coord: Option<AccordionCoordination> = try_consume_context();
    let pos: Option<DivPosition> = try_consume_context();
    let is_last = pos.map_or(false, |p| p.index == p.total - 1);

    let initial_open = defaults.as_ref().map_or(false, |d| d.contains(&value));
    let local_open = use_state(move || initial_open);

    let height_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    let chevron_animation: Rc<RefCell<Option<UseAnimation<AnimNum>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    if let Some(c) = &coord {
        c.registry.borrow_mut().insert(
            value.clone(),
            ItemHandle {
                open: local_open,
                height_animation: height_animation.clone(),
                chevron_animation: chevron_animation.clone(),
            },
        );
    }

    let item_disabled = local_disabled || coord.as_ref().map_or(false, |c| c.disabled);

    let trigger_id = use_a11y();
    let content_id = use_a11y();

    use_provide_context(move || AccordionItemData {
        value: value.clone(),
        local_open,
        disabled: item_disabled,
        trigger_id,
        content_id,
        height_animation: height_animation.clone(),
        chevron_animation: chevron_animation.clone(),
    });

    let mut item_style = Style::new();
    if !is_last {
        let state = use_cn_theme();
        let border = state.read().border;
        item_style = item_style.border_b(1.0).border_color(border);
    }

    Div()
        .style("flex flex-col w-full")
        .merge_style(item_style)
        .merge_style(style.clone())
        .children(children.clone())
}

// ==============================================================
// Trigger
// ==============================================================

#[component(props(), ext(style, children, key))]
#[derive(PartialEq, Clone)]
pub fn AccordionTrigger() -> impl IntoElement {
    let item_ctx: Option<AccordionItemData> = try_consume_context();
    let coord: Option<AccordionCoordination> = try_consume_context();

    let placeholder = use_state(|| false);

    let (local_open, disabled, trigger_id, value, height_anim, chevron_anim) = match &item_ctx {
        Some(i) => (
            i.local_open,
            i.disabled,
            Some(i.trigger_id.clone()),
            i.value.clone(),
            Some(i.height_animation.clone()),
            Some(i.chevron_animation.clone()),
        ),
        None => (placeholder, true, None, String::new(), None, None),
    };

    let is_open = *local_open.read();

    let coord_for_press = coord.clone();
    let value_for_press = value.clone();
    let height_for_press = height_anim.clone();
    let chevron_for_press = chevron_anim.clone();

    let focus_border = Style::new()
        .border_width(1.0)
        .border_color(Color::TRANSPARENT);

    let mut trigger = Div()
        .style(
            "flex flex-row items-center justify-between gap-4 w-full rounded-md \
             py-4 text-left text-sm font-medium select-none cursor-pointer \
             focus-visible:border-ring \
             disabled:pointer-events-none disabled:opacity-50",
        )
        .merge_style(focus_border)
        .merge_style(style.clone())
        .focusable(!disabled)
        .disabled(disabled)
        .a11y_role(AccessibilityRole::Button)
        .a11y_expanded(is_open)
        .on_press(move |_| {
            let Some(coord) = &coord_for_press else {
                return;
            };
            let mut lo = local_open;
            let was_open = *lo.read();
            let will_open = !was_open;

            if will_open {
                if matches!(coord.type_, AccordionType::Single) {
                    let prev_val = coord.currently_open.borrow_mut().take();
                    let to_close = prev_val.filter(|p| p != &value_for_press);
                    if let Some(prev_val) = to_close {
                        let handle = coord.registry.borrow().get(&prev_val).cloned();
                        if let Some(handle) = handle {
                            let mut s = handle.open;
                            s.set(false);
                            if let Some(anim) = &mut *handle.height_animation.borrow_mut() {
                                anim.reverse();
                            }
                            if let Some(anim) = &mut *handle.chevron_animation.borrow_mut() {
                                anim.reverse();
                            }
                        }
                    }
                    *coord.currently_open.borrow_mut() = Some(value_for_press.clone());
                }
                lo.set(true);
                if let Some(anim) = height_for_press.as_ref() {
                    if let Some(a) = &mut *anim.borrow_mut() {
                        a.start();
                    }
                }
                if let Some(anim) = chevron_for_press.as_ref() {
                    if let Some(a) = &mut *anim.borrow_mut() {
                        a.start();
                    }
                }
            } else {
                if matches!(coord.type_, AccordionType::Single) && !coord.collapsible {
                    return;
                }
                if coord.currently_open.borrow().as_ref() == Some(&value_for_press) {
                    *coord.currently_open.borrow_mut() = None;
                }
                lo.set(false);
                if let Some(anim) = height_for_press.as_ref() {
                    if let Some(a) = &mut *anim.borrow_mut() {
                        a.reverse();
                    }
                }
                if let Some(anim) = chevron_for_press.as_ref() {
                    if let Some(a) = &mut *anim.borrow_mut() {
                        a.reverse();
                    }
                }
            }
        })
        .children(children.clone())
        .child(
            Div()
                .style("pointer-events-none shrink-0 text-muted-foreground")
                .child(AnimatedChevron().open(is_open)),
        );

    if let Some(id) = trigger_id {
        trigger = trigger.custom_a11y_id(id);
    }

    trigger
}

// ==============================================================
// Content
// ==============================================================

#[component(props(), ext(style, children, key))]
#[derive(PartialEq, Clone)]
pub fn AccordionContent() -> impl IntoElement {
    let item_ctx: Option<AccordionItemData> = try_consume_context();

    let placeholder = use_state(|| false);

    let (local_open, trigger_id, content_id) = match &item_ctx {
        Some(i) => (
            i.local_open,
            Some(i.trigger_id.clone()),
            Some(i.content_id.clone()),
        ),
        None => (placeholder, None, None),
    };

    let is_open = *local_open.read();

    let mut content = Div()
        .style("w-full text-sm")
        .merge_style(style.clone())
        .a11y_role(AccessibilityRole::Region)
        .a11y_hidden(!is_open);

    if let Some(tid) = trigger_id {
        content = content.a11y_labelled_by(tid);
    }
    if let Some(cid) = content_id {
        content = content.custom_a11y_id(cid);
    }

    content.child(
        CollapsibleHeight()
            .open(is_open)
            .child(Div().style("pt-0 pb-4").children(children.clone())),
    )
}
