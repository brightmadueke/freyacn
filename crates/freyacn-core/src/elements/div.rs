use crate::styling::apply::apply_to_rect;
use crate::styling::has::{HasContext, HasFlags};
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledExt};
use freya::components::{ScrollConfig, ScrollController, ScrollView, use_scroll_controller};
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
use torin::prelude::Direction;

/// The generic container element.
///
/// ## Focus behavior
///
/// A `Div` becomes focusable in any of these cases:
///
/// * A `:focus` / `:focus-visible` / `:focus-within` variant is registered
/// * A `.on_focus(..)` / `.on_blur(..)` / focus-visible callback is
///   registered
/// * `.with_focusable(true)` is called explicitly
///
/// When focusable, clicking requests focus — matching `<div tabindex="0">`.
/// A plain `div()` with only `.on_press(..)` stays non-focusable.
///
/// ## Scrolling
///
/// See [`ScrollView`] docs. `.scroll_y()` / `.scroll_x()` /
/// `.with_scroll_controller(..)`.
///
/// ## Shadows
///
/// Box shadows (`.shadow_*()`) render on this `Div`'s rect via
/// `apply_to_rect`. Text shadows (`.text_shadow_*()`) don't paint on a
/// `Div` directly — no glyph content — but cascade into text
/// descendants through `InheritedStyle`.
///
/// ## Has: signaling
///
/// Every `Div` creates a reactive [`HasFlags`] bag and provides a
/// [`HasContext`] to its descendants, so any child that matches a
/// signal (an `Icon`, an `Image`, a checked checkbox, …) sets the
/// matching flag.
///
/// ## A11y id override
///
/// By default `Div` generates its own a11y id via `use_a11y()`. A parent
/// component that wants to own the focus target — e.g. `Input`, which
/// needs the outer div to be the focused element so key events route
/// there — can pass an override via `.with_a11y_id(id)`. The div's own
/// `use_a11y()` still runs (hooks must run unconditionally), but the
/// override replaces the id everywhere it matters: the rect's
/// `.a11y_id(..)`, `use_focus(..)`, and the `focus_on_press` passed to
/// `attach_pointer_handlers`.
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
    focusable_override: Option<bool>,

    /// Override the auto-generated a11y id.
    ///
    /// When `Some`, `build_element` uses this instead of the id from
    /// `use_a11y()`. Lets a parent component own the a11y identity of
    /// this div — required for cases like `Input`, which needs to focus
    /// the div when the inner paragraph (which intercepts clicks) is
    /// pressed.
    a11y_id_override: Option<AccessibilityId>,

    // Scroll config
    scroll_direction: Option<Direction>,
    scroll_controller: Option<ScrollController>,
    show_scrollbar: bool,
}

/// Shorthand constructor: `div().style("p-4 hover:bg-blue-100").child(...)`
pub fn div() -> Div {
    Div::new()
}

impl Div {
    pub fn new() -> Self {
        Self {
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            children: Vec::new(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),
            disabled: false,
            checked: false,
            selected: false,
            semantic_events: SemanticEvents::default(),
            focusable_override: None,

            a11y_id_override: None,

            scroll_direction: None,
            scroll_controller: None,
            show_scrollbar: true,
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

    pub fn with_focusable(mut self, v: bool) -> Self {
        self.focusable_override = Some(v);
        self
    }

    /// Override the a11y id used for focus and pointer handlers.
    ///
    /// Use this when a parent component needs to own the focus target —
    /// for example, `Input` creates an id in its own `render` scope and
    /// passes it here, so calling `request_focus()` on that id focuses
    /// the div. Without the override, the div would get a *different*
    /// auto-generated id, and the parent's focus call would target an
    /// unrelated element.
    pub fn with_a11y_id(mut self, id: AccessibilityId) -> Self {
        self.a11y_id_override = Some(id);
        self
    }

    pub fn with_semantic_events(mut self, events: SemanticEvents) -> Self {
        self.semantic_events = events;
        self
    }

    // ── scrolling ──────────────────────────────────────────────────

    pub fn scroll_y(mut self) -> Self {
        self.scroll_direction = Some(Direction::Vertical);
        self
    }

    pub fn scroll_x(mut self) -> Self {
        self.scroll_direction = Some(Direction::Horizontal);
        self
    }

    pub fn with_scroll_controller(mut self, controller: ScrollController) -> Self {
        self.scroll_controller = Some(controller);
        self
    }

    pub fn show_scrollbar(mut self, v: bool) -> Self {
        self.show_scrollbar = v;
        self
    }

    pub fn hide_scrollbar(mut self) -> Self {
        self.show_scrollbar = false;
        self
    }

    pub(crate) fn build_element(&self) -> Rect {
        // Hooks must run unconditionally.
        let auto_scroll_controller = use_scroll_controller(ScrollConfig::default);

        // `use_a11y()` is called unconditionally (hooks must run in a
        // stable order every render). If an override is set, we use it
        // for the div's focus tracking and pointer handlers, so the
        // caller's `request_focus()` on that same id focuses this div.
        let auto_a11y_id = use_a11y();
        let a11y_id = self.a11y_id_override.unwrap_or(auto_a11y_id);
        let focus = use_focus(a11y_id);

        let hovering = use_state(|| false);
        let pressing = use_state(|| false);

        // `has:` support — reactive flag bag (hook, runs unconditionally)
        // and context provision (hook, same HasFlags handle every render
        // — `HasContext: PartialEq` returns `true` so no spurious re-renders).
        let mut has_flags = HasFlags::new();
        use_provide_context({
            let has_flags = has_flags.clone();
            move || HasContext(has_flags)
        });

        // ── 1. Base style ───────────────────────────────────────────
        let base = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset().merge_style(&self.style),
            None => Style::default().merge_style(&self.style),
        };

        // ── 2. Live variants ────────────────────────────────────────
        let variants = &self.style.variants;
        let needs_hover_active_tracking =
            variants.has(StyleVariant::Hover) || variants.has(StyleVariant::Active);
        let wants_focus = self.focusable_override.unwrap_or_else(|| {
            variants.has(StyleVariant::Focus)
                || variants.has(StyleVariant::FocusVisible)
                || variants.has(StyleVariant::FocusWithin)
                || self.semantic_events.has_focus_callbacks()
        });

        let focus_state = focus();
        let focused_kb = focus_state == Focus::Keyboard;
        let focused = matches!(focus_state, Focus::Keyboard | Focus::Pointer);

        let mut live = Vec::with_capacity(16);
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

        // `aria-*` state → live variants
        live.extend(self.semantic_events.aria_state.active_variants());

        // `has:` state → live variants
        //
        // Two sources:
        // 1. Automatic signals from descendants — only read (and hence
        //    subscribe) when the Div actually declares a `has:` variant.
        // 2. Manual flags from `.with_has_svg(..)` etc. — always read,
        //    cheap, no-op when empty.
        if variants.has_any_has() {
            live.extend(has_flags.active_variants());
        }
        live.extend(self.semantic_events.has_state.active_variants());

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events ─────────────────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Provide to descendants ───────────────────────────────
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 5. Render outer box ─────────────────────────────────────
        let can_focus = !self.disabled && wants_focus;

        let mut el = apply_to_rect(rect(), &style).a11y_id(a11y_id);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        if can_focus {
            el = el.a11y_focusable(true);
        }

        if !self.disabled {
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

        // ── 6. Children: inline or wrapped in a ScrollView ──────────
        let is_scrollable = self.scroll_direction.is_some() || self.scroll_controller.is_some();

        if is_scrollable {
            let direction = self.scroll_direction.unwrap_or(style.direction);
            let controller = self.scroll_controller.unwrap_or(auto_scroll_controller);

            let scroll_view = ScrollView::new_controlled(controller)
                .direction(direction)
                .show_scrollbar(self.show_scrollbar)
                .width(Size::fill())
                .height(Size::fill())
                .children(self.children.clone());

            el = el.child(scroll_view);
        } else {
            el = el.children(self.children.clone());
        }

        el
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
        self.build_element()
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
