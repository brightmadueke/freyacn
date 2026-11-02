//! Shared semantic-event layer for interactive and semi-interactive components.

use freya::prelude::Paragraph as ParagraphPrimitive;
use freya::prelude::*;
use freya_core::elements::rect::Rect;
use std::cell::RefCell;
use std::rc::Rc;

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

/// Attaches hover / press handlers when needed.
///
/// `focus_on_press` is an [`AccessibilityId`] (from `use_a11y()`), because
/// only `AccessibilityId` exposes `request_focus()`.
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

/// Fires focus / blur / focus-visible callbacks on transitions.
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
