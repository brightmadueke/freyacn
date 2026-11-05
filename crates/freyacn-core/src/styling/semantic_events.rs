//! Shared semantic-event layer for interactive and semi-interactive components.

use crate::styling::style_variants::StyleVariant;
use freya::prelude::Paragraph as ParagraphPrimitive;
use freya::prelude::*;
use freya_core::elements::rect::Rect;
use std::cell::RefCell;
use std::rc::Rc;

// ─────────────────────────────────────────────────────────────────────
// Aria state
// ─────────────────────────────────────────────────────────────────────

/// Boolean aria-* flags that components expose. Mirrors the `aria-*`
/// variant prefixes the class parser understands, plus the state a11y
/// attributes each maps to.
///
/// Each field is a plain `bool` because Freya's `AccessibilityData`
/// already carries the actual accessibility attribute — this struct is
/// only used to translate the *style* variant.
///
/// Call [`AriaState::active_variants`] from a component's `render` to get
/// the `StyleVariant`s to push into the live-variant set.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct AriaState {
    pub invalid: bool,
    pub disabled: bool,
    pub expanded: bool,
    pub checked: bool,
    pub selected: bool,
    pub pressed: bool,
    pub readonly: bool,
    pub required: bool,
    pub hidden: bool,
}

impl AriaState {
    /// Every `StyleVariant::Aria*` whose flag is set, in a stable order.
    pub fn active_variants(self) -> Vec<StyleVariant> {
        let mut out = Vec::new();
        if self.invalid {
            out.push(StyleVariant::AriaInvalid);
        }
        if self.disabled {
            out.push(StyleVariant::AriaDisabled);
        }
        if self.expanded {
            out.push(StyleVariant::AriaExpanded);
        }
        if self.checked {
            out.push(StyleVariant::AriaChecked);
        }
        if self.selected {
            out.push(StyleVariant::AriaSelected);
        }
        if self.pressed {
            out.push(StyleVariant::AriaPressed);
        }
        if self.readonly {
            out.push(StyleVariant::AriaReadonly);
        }
        if self.required {
            out.push(StyleVariant::AriaRequired);
        }
        if self.hidden {
            out.push(StyleVariant::AriaHidden);
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────
// Has: state — manual overrides
// ─────────────────────────────────────────────────────────────────────

/// Explicit `has:` flags, set via `.with_has_svg(..)` / `.with_has_img(..)`
/// / etc.
///
/// Unlike automatic signaling (a descendant `Icon` firing
/// `HasKind::Svg`), the manual flags have **no one-frame delay** and
/// **do reset** the moment the flag is set back to `false` — Freya has
/// no unmount hook, so automatic signals accumulate but manual flags
/// don't.
///
/// Call [`HasManualFlags::active_variants`] from `render` and extend
/// the live-variant set.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct HasManualFlags {
    pub svg: bool,
    pub img: bool,
    pub checked: bool,
    pub selected: bool,
    pub disabled: bool,
}

impl HasManualFlags {
    /// Every `StyleVariant::Has*` whose flag is set, in a stable order.
    pub fn active_variants(self) -> Vec<StyleVariant> {
        let mut out = Vec::new();
        if self.svg {
            out.push(StyleVariant::HasSvg);
        }
        if self.img {
            out.push(StyleVariant::HasImg);
        }
        if self.checked {
            out.push(StyleVariant::HasChecked);
        }
        if self.selected {
            out.push(StyleVariant::HasSelected);
        }
        if self.disabled {
            out.push(StyleVariant::HasDisabled);
        }
        out
    }

    pub fn is_empty(self) -> bool {
        !self.svg && !self.img && !self.checked && !self.selected && !self.disabled
    }
}

// ─────────────────────────────────────────────────────────────────────
// Data
// ─────────────────────────────────────────────────────────────────────

#[derive(Default, Clone, PartialEq)]
pub struct SemanticEvents {
    pub on_hover: Option<EventHandler<Event<PointerEventData>>>,
    pub on_hover_end: Option<EventHandler<Event<PointerEventData>>>,
    pub on_pressed: Option<EventHandler<Event<PointerEventData>>>,
    pub on_released: Option<EventHandler<Event<PointerEventData>>>,
    pub on_focus: Option<NoArgCallback<()>>,
    pub on_blur: Option<NoArgCallback<()>>,
    pub on_focus_visible: Option<NoArgCallback<()>>,
    pub on_focus_visible_end: Option<NoArgCallback<()>>,

    /// Current aria-* state, driving the `aria-*:` variant prefixes.
    pub aria_state: AriaState,

    /// Manual `has:` flags, driving the `has-*:` variant prefixes.
    /// These sit alongside automatic signaling from descendants — both
    /// sources get merged into the live-variant set at render time.
    pub has_state: HasManualFlags,
}

impl SemanticEvents {
    pub fn has_pointer_callbacks(&self) -> bool {
        self.on_hover.is_some()
            || self.on_hover_end.is_some()
            || self.on_pressed.is_some()
            || self.on_released.is_some()
    }

    pub fn has_focus_callbacks(&self) -> bool {
        self.on_focus.is_some()
            || self.on_blur.is_some()
            || self.on_focus_visible.is_some()
            || self.on_focus_visible_end.is_some()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Builder trait
// ─────────────────────────────────────────────────────────────────────

pub trait HasSemanticEvents: Sized {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents;

    /// Replace this component's event bundle wholesale.
    fn semantic_events(mut self, events: SemanticEvents) -> Self {
        *self.semantic_events_mut() = events;
        self
    }

    // ── individual callback builders ───────────────────────────────

    fn on_hover(mut self, cb: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.semantic_events_mut().on_hover = Some(cb.into());
        self
    }
    fn on_hover_end(mut self, cb: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.semantic_events_mut().on_hover_end = Some(cb.into());
        self
    }
    fn on_pressed(mut self, cb: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.semantic_events_mut().on_pressed = Some(cb.into());
        self
    }
    fn on_released(mut self, cb: impl Into<EventHandler<Event<PointerEventData>>>) -> Self {
        self.semantic_events_mut().on_released = Some(cb.into());
        self
    }
    fn on_focus(mut self, cb: impl Into<NoArgCallback<()>>) -> Self {
        self.semantic_events_mut().on_focus = Some(cb.into());
        self
    }
    fn on_blur(mut self, cb: impl Into<NoArgCallback<()>>) -> Self {
        self.semantic_events_mut().on_blur = Some(cb.into());
        self
    }
    fn on_focus_visible(mut self, cb: impl Into<NoArgCallback<()>>) -> Self {
        self.semantic_events_mut().on_focus_visible = Some(cb.into());
        self
    }
    fn on_focus_visible_end(mut self, cb: impl Into<NoArgCallback<()>>) -> Self {
        self.semantic_events_mut().on_focus_visible_end = Some(cb.into());
        self
    }

    // ── aria state builders ────────────────────────────────────────

    /// Replace the full aria state in one call.
    fn aria_state(mut self, state: AriaState) -> Self {
        self.semantic_events_mut().aria_state = state;
        self
    }

    fn with_aria_invalid(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.invalid = v;
        self
    }
    fn with_aria_disabled(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.disabled = v;
        self
    }
    fn with_aria_expanded(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.expanded = v;
        self
    }
    fn with_aria_checked(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.checked = v;
        self
    }
    fn with_aria_selected(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.selected = v;
        self
    }
    fn with_aria_pressed(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.pressed = v;
        self
    }
    fn with_aria_readonly(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.readonly = v;
        self
    }
    fn with_aria_required(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.required = v;
        self
    }
    fn with_aria_hidden(mut self, v: bool) -> Self {
        self.semantic_events_mut().aria_state.hidden = v;
        self
    }

    // ── has: state builders ────────────────────────────────────────

    /// Replace the full manual `has:` state in one call.
    fn has_state(mut self, state: HasManualFlags) -> Self {
        self.semantic_events_mut().has_state = state;
        self
    }

    /// Declare that this element contains an SVG child. Fires
    /// `has-[svg]:` classes on this element without requiring a
    /// descendant `Icon` to signal — useful when the child is
    /// conditional and you want instant updates (no one-frame delay)
    /// or when the child is rendered outside `Icon`.
    fn with_has_svg(mut self, v: bool) -> Self {
        self.semantic_events_mut().has_state.svg = v;
        self
    }

    /// Declare that this element contains an Image child. See
    /// [`with_has_svg`](Self::with_has_svg) for the automatic-vs-manual
    /// tradeoff.
    fn with_has_img(mut self, v: bool) -> Self {
        self.semantic_events_mut().has_state.img = v;
        self
    }

    /// Declare that this element contains a checked control.
    fn with_has_checked(mut self, v: bool) -> Self {
        self.semantic_events_mut().has_state.checked = v;
        self
    }

    /// Declare that this element contains a selected child.
    fn with_has_selected(mut self, v: bool) -> Self {
        self.semantic_events_mut().has_state.selected = v;
        self
    }

    /// Declare that this element contains a disabled child.
    fn with_has_disabled(mut self, v: bool) -> Self {
        self.semantic_events_mut().has_state.disabled = v;
        self
    }
}

impl HasSemanticEvents for SemanticEvents {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        self
    }
}

// ─────────────────────────────────────────────────────────────────────
// Pointer dispatch
// ─────────────────────────────────────────────────────────────────────

pub trait PointerEventHandler: Sized {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self;
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self;
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self;
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self;
}

impl PointerEventHandler for Rect {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_enter(f)
    }
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_leave(f)
    }
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_down(f)
    }
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_global_pointer_up(f)
    }
}

impl PointerEventHandler for Label {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_enter(f)
    }
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_leave(f)
    }
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_down(f)
    }
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_global_pointer_up(f)
    }
}

impl PointerEventHandler for freya::elements::image::Image {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_enter(f)
    }
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_leave(f)
    }
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_down(f)
    }
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_global_pointer_up(f)
    }
}

impl PointerEventHandler for ParagraphPrimitive {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_enter(f)
    }
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_leave(f)
    }
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_down(f)
    }
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_global_pointer_up(f)
    }
}

impl PointerEventHandler for freya::components::SvgViewer {
    fn pointer_enter(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_enter(f)
    }
    fn pointer_leave(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_leave(f)
    }
    fn pointer_down(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_pointer_down(f)
    }
    fn global_pointer_up(self, f: impl FnMut(Event<PointerEventData>) + 'static) -> Self {
        self.on_global_pointer_up(f)
    }
}

pub fn attach_pointer_handlers<E: PointerEventHandler>(
    el: E,
    events: &SemanticEvents,
    mut hovering: State<bool>,
    mut pressing: State<bool>,
    needs_variant_tracking: bool,
    focus_on_press: Option<AccessibilityId>,
) -> E {
    let needs_handlers =
        needs_variant_tracking || events.has_pointer_callbacks() || focus_on_press.is_some();

    if !needs_handlers {
        return el;
    }

    let on_hover = events.on_hover.clone();
    let on_hover_end = events.on_hover_end.clone();
    let on_pressed = events.on_pressed.clone();
    let on_released = events.on_released.clone();

    el.pointer_enter(move |e| {
        hovering.set(true);
        if let Some(cb) = &on_hover {
            cb.call(e);
        }
    })
    .pointer_leave(move |e| {
        hovering.set(false);
        if let Some(cb) = &on_hover_end {
            cb.call(e);
        }
    })
    .pointer_down(move |e| {
        if let Some(id) = focus_on_press {
            id.request_focus();
        }
        pressing.set(true);
        if let Some(cb) = &on_pressed {
            cb.call(e);
        }
    })
    .global_pointer_up(move |e| {
        pressing.set(false);
        if let Some(cb) = &on_released {
            cb.call(e);
        }
    })
}

// ─────────────────────────────────────────────────────────────────────
// Focus dispatch — custom hook
// ─────────────────────────────────────────────────────────────────────

fn is_focused(state: Focus) -> bool {
    matches!(state, Focus::Keyboard | Focus::Pointer)
}

pub fn use_semantic_focus_events(events: &SemanticEvents, focus_state: Focus) {
    let prev_focus: Rc<RefCell<Focus>> = use_hook(|| Rc::new(RefCell::new(focus_state)));
    use_side_effect({
        let prev_focus = prev_focus.clone();
        let on_focus = events.on_focus.clone();
        let on_blur = events.on_blur.clone();
        let on_focus_visible = events.on_focus_visible.clone();
        let on_focus_visible_end = events.on_focus_visible_end.clone();
        move || {
            let cur = focus_state;
            let prev = *prev_focus.borrow();
            if prev == cur {
                return;
            }

            let was_focused = is_focused(prev);
            let is_focused_cur = is_focused(cur);

            if !was_focused && is_focused_cur {
                if let Some(cb) = &on_focus {
                    cb.call();
                }
            } else if was_focused && !is_focused_cur {
                if let Some(cb) = &on_blur {
                    cb.call();
                }
            }

            let was_kb = prev == Focus::Keyboard;
            let is_kb = cur == Focus::Keyboard;

            if !was_kb && is_kb {
                if let Some(cb) = &on_focus_visible {
                    cb.call();
                }
            } else if was_kb && !is_kb {
                if let Some(cb) = &on_focus_visible_end {
                    cb.call();
                }
            }

            *prev_focus.borrow_mut() = cur;
        }
    });
}
