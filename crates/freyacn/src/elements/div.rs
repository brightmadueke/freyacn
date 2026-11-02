use crate::styling::apply::apply_to_rect;
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledExt};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Component, EventHandlers, rect, try_consume_context, use_a11y, use_focus, use_provide_context,
    use_state,
};

/// The generic container element.
///
/// ## Focus behavior
///
/// A `Div` becomes focusable when any of these is present:
///
/// * `:focus` / `:focus-visible` / `:focus-within` styling
/// * `.on_focus(..)` / `.on_blur(..)` / focus-visible callbacks
///
/// When focusable, clicking the `Div` requests focus — matching
/// `<div tabindex="0">` behavior. A plain clickable `div()` with only
/// `.on_press(..)` stays non-focusable.
#[derive(PartialEq, Clone)]
pub struct Div {
    style: Style,
    event_handlers: EventHandlers,
    children: Vec<Element>,
    key: DiffKey,
    accessibility: AccessibilityData,
    disabled: bool,
    checked: bool,
    selected: bool,
    semantic_events: SemanticEvents,
}

/// Shorthand constructor: `div().style("p-4 hover:bg-blue-100").child(...)`
pub fn div() -> Div {
    Div::new()
}

impl Div {
    pub fn new() -> Self {
        Self {
            // `Style::bare()`: this field holds user overrides only.
            // Theme + inherited ancestor values are layered at render time.
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            children: Vec::new(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),
            disabled: false,
            checked: false,
            selected: false,
            semantic_events: SemanticEvents::default(),
        }
    }

    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }

    /// Replace the semantic-events bundle wholesale. Useful when a
    /// wrapper element (e.g. `Checkbox`) owns its own `SemanticEvents`
    /// and wants to delegate to this `Div`.
    pub fn with_semantic_events(mut self, events: SemanticEvents) -> Self {
        self.semantic_events = events;
        self
    }
}

impl Default for Div {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledExt for Div {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl ChildrenExt for Div {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl KeyExt for Div {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Div {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Div {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Div {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

impl Component for Div {
    fn render(&self) -> impl IntoElement {
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        // ── 1. Base style ───────────────────────────────────────────
        // bare < ancestor inheritable < self.style < live variants
        let base = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset().merge_style(&self.style),
            None => Style::default().merge_style(&self.style),
        };

        // ── 2. Live variants ────────────────────────────────────────
        let variants = &self.style.variants;
        let needs_hover_active_tracking =
            variants.has(StyleVariant::Hover) || variants.has(StyleVariant::Active);
        let wants_focus = variants.has(StyleVariant::Focus)
            || variants.has(StyleVariant::FocusVisible)
            || variants.has(StyleVariant::FocusWithin)
            || self.semantic_events.has_focus_callbacks();

        let focus_state = focus();
        let focused_kb = focus_state == Focus::Keyboard;
        let focused = matches!(focus_state, Focus::Keyboard | Focus::Pointer);

        let mut live = Vec::with_capacity(8);
        if self.disabled {
            live.push(StyleVariant::Disabled);
        } else {
            if *hovering.read() {
                live.push(StyleVariant::Hover);
            }
            if *pressing.read() {
                live.push(StyleVariant::Active);
            }
            if focused {
                live.push(StyleVariant::Focus);
                live.push(StyleVariant::FocusWithin);
            }
            if focused_kb {
                live.push(StyleVariant::FocusVisible);
            }
        }
        if self.checked {
            live.push(StyleVariant::Checked);
        }
        if self.selected {
            live.push(StyleVariant::Selected);
        }

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events ─────────────────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Provide to descendants ───────────────────────────────
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 5. Render ───────────────────────────────────────────────
        let can_focus = !self.disabled && wants_focus;

        let mut el = apply_to_rect(rect(), &style);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        if can_focus {
            el = el.a11y_id(a11y_id).a11y_focusable(true);
        }

        if !self.disabled {
            // Focusable divs request focus on click — matching
            // `<div tabindex="0">`. Non-focusable divs pass `None` and
            // therefore only track hover/press for styling.
            let focus_on_press = if can_focus { Some(a11y_id) } else { None };
            el = attach_pointer_handlers(
                el,
                &self.semantic_events,
                hovering,
                pressing,
                needs_hover_active_tracking,
                focus_on_press,
            );
        }

        el.children(self.children.clone())
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
