use crate::styling::apply::apply_to_rect;
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
/// Freya's `Overflow` enum only has `None` and `Clip` — there is no
/// `Overflow::Scroll`. Div fills that gap by delegating to `ScrollView`
/// when scrolling is enabled:
///
/// * `.scroll_y()` — vertical scroll, internal controller, scrollbar
///   shown. Layout direction becomes `Vertical`.
/// * `.scroll_x()` — horizontal scroll, internal controller, scrollbar
///   shown. Layout direction becomes `Horizontal`.
/// * `.with_scroll_controller(c)` — external controller for programmatic
///   scroll (`scroll_to_x` / `scroll_to_y`). Direction falls back to
///   `Style::direction` (or `Vertical` if unset).
/// * `.show_scrollbar(false)` / `.hide_scrollbar()` — opt out of the
///   scrollbar while keeping wheel/touch scrolling.
///
/// The outer `Div`'s size constrains the scroll viewport. Without an
/// explicit `.height(..)` (for `scroll_y`) or `.width(..)` (for
/// `scroll_x`), the viewport sizes to content and no scroll happens.
///
/// ```
/// div()
///     .height(Size::px(200.0))
///     .scroll_y()
///     .children(items)
/// ```
///
/// Non-scrolling `div()`s carry no overhead — the internal
/// `ScrollController` is created but unused.
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
            focusable_override: None,

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

    /// Force the focusability flag, overriding the default derivation
    /// (which is "focusable iff a `:focus`-family variant or a focus
    /// callback is registered"). Useful for elements that need to be
    /// focusable without consuming any focus events — `input()` is the
    /// canonical case.
    pub fn with_focusable(mut self, v: bool) -> Self {
        self.focusable_override = Some(v);
        self
    }

    /// Replace the semantic-events bundle wholesale. Useful when a
    /// wrapper element (e.g. `Checkbox`) owns its own `SemanticEvents`
    /// and wants to delegate to this `Div`.
    pub fn with_semantic_events(mut self, events: SemanticEvents) -> Self {
        self.semantic_events = events;
        self
    }

    // ── scrolling ──────────────────────────────────────────────────

    /// Enable vertical scrolling. Children are wrapped in a `ScrollView`
    /// laid out vertically; a scrollbar appears when content overflows.
    ///
    /// The outer `Div`'s size constrains the viewport — set `.height(..)`
    /// or `.max_height(..)` so scrolling has a bound. Without a bound,
    /// the viewport sizes to content and no scroll happens.
    ///
    /// ```
    /// div()
    ///     .height(Size::px(200.0))
    ///     .scroll_y()
    ///     .children(items)
    /// ```
    pub fn scroll_y(mut self) -> Self {
        self.scroll_direction = Some(Direction::Vertical);
        self
    }

    /// Enable horizontal scrolling. Children are wrapped in a `ScrollView`
    /// laid out horizontally; a scrollbar appears when content overflows.
    ///
    /// Set `.width(..)` or `.max_width(..)` so scrolling has a bound.
    pub fn scroll_x(mut self) -> Self {
        self.scroll_direction = Some(Direction::Horizontal);
        self
    }

    /// Enable scrolling with an external controller for programmatic
    /// scroll (`scroll_to_x` / `scroll_to_y`). Direction falls back to
    /// `Style::direction` when `scroll_y()` / `scroll_x()` aren't called
    /// explicitly, so `.direction(..)` alone also works to pick the axis.
    ///
    /// ```
    /// let controller = use_scroll_controller(ScrollConfig::default);
    /// div()
    ///     .height(Size::px(200.0))
    ///     .with_scroll_controller(controller)
    ///     .children(items)
    /// ```
    pub fn with_scroll_controller(mut self, controller: ScrollController) -> Self {
        self.scroll_controller = Some(controller);
        self
    }

    /// Show a scrollbar when content overflows. Applies only when
    /// scrolling is enabled (`scroll_y` / `scroll_x` /
    /// `with_scroll_controller`). Defaults to `true`.
    pub fn show_scrollbar(mut self, v: bool) -> Self {
        self.show_scrollbar = v;
        self
    }

    /// Hide the scrollbar while keeping wheel / touch scrolling.
    /// Shorthand for `.show_scrollbar(false)`.
    pub fn hide_scrollbar(mut self) -> Self {
        self.show_scrollbar = false;
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
        // Hooks must run unconditionally. The internal controller is
        // unused when scrolling is disabled — cheap enough to allocate
        // per div (two `State<f32>` handles).
        let auto_scroll_controller = use_scroll_controller(ScrollConfig::default);

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
        let wants_focus = self.focusable_override.unwrap_or_else(|| {
            variants.has(StyleVariant::Focus)
                || variants.has(StyleVariant::FocusVisible)
                || variants.has(StyleVariant::FocusWithin)
                || self.semantic_events.has_focus_callbacks()
        });

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

        // ── 5. Render outer box ─────────────────────────────────────
        let can_focus = !self.disabled && wants_focus;

        let mut el = apply_to_rect(rect(), &style);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        if can_focus {
            el = el.a11y_id(a11y_id).a11y_focusable(true);
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
            // `scroll_y` / `scroll_x` set an explicit direction.
            // `with_scroll_controller` alone falls back to the style's
            // direction, so `.direction(..)` also picks the axis.
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

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
