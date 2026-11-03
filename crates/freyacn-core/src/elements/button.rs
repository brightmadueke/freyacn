use crate::elements::div::Div;
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{Style, StyledColor, StyledExt};
use crate::theme::active_theme;
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::IntoElement;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use torin::gaps::Gaps;

#[derive(PartialEq, Clone)]
pub struct Button {
    style: Style,
    event_handlers: EventHandlers,
    children: Vec<Element>,
    key: DiffKey,
    accessibility: AccessibilityData,

    disabled: bool,
    focusable: bool,
    use_defaults: bool,
    selected: bool,

    semantic_events: SemanticEvents,

    on_selected: Option<EventHandler<Event<PressEventData>>>,
    on_deselected: Option<EventHandler<Event<PressEventData>>>,
}

pub fn button() -> Button {
    Button::new()
}

impl Button {
    pub fn new() -> Self {
        Self {
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            children: Vec::new(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),
            disabled: false,
            focusable: true,
            use_defaults: true,
            selected: false,
            semantic_events: SemanticEvents::default(),
            on_selected: None,
            on_deselected: None,
        }
    }

    /// Disable the button.
    ///
    /// Sets the native `disabled` flag **and** the `aria-disabled` state,
    /// so both class families fire without extra calls:
    ///
    /// ```
    /// button()
    ///     .with_disabled(true)
    ///     .style("disabled:opacity-50 aria-disabled:cursor-not-allowed")
    /// ```
    ///
    /// Native `disabled` blocks pointer events (via `Div`'s gating) and
    /// disables focus; `aria-disabled` alone would only affect styling.
    /// Because this method sets both, event suppression and the a11y
    /// attribute stay in sync.
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    pub fn with_focusable(mut self, v: bool) -> Self {
        self.focusable = v;
        self
    }

    /// Controlled selected (toggle) state — HTML `aria-pressed`.
    ///
    /// Sets the internal `selected` flag (driving the `:selected` style
    /// variant) **and** the `aria-pressed` state (driving `aria-pressed:*`
    /// class strings). Both fire together, so existing code using
    /// `selected:*` classes keeps working, and new code can use either
    /// `selected:*` or `aria-pressed:*` interchangeably.
    ///
    /// Renamed from `selected` so `StyledExt::selected(..)` remains
    /// callable for styling the `:selected` variant:
    ///
    /// ```
    /// button()
    ///     .with_selected(true)                                        // state
    ///     .selected(|s| s.bg(blue_600()))                             // `selected:*`
    ///     .style("aria-pressed:bg-blue-600 aria-pressed:text-white")  // `aria-pressed:*`
    /// ```
    ///
    /// If you want **only** the aria variant (no `selected:*`), call
    /// `with_aria_pressed(true)` from [`HasSemanticEvents`] directly
    /// instead.
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self.semantic_events.aria_state.pressed = v;
        self
    }

    pub fn no_default_style(mut self) -> Self {
        self.use_defaults = false;
        self
    }

    pub fn on_selected(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_selected = Some(cb.into());
        self
    }

    pub fn on_deselected(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_deselected = Some(cb.into());
        self
    }

    /// Build the `Div` that will actually render. It carries the button's
    /// style (UA + user), state flags (`disabled`, `selected`, `focusable`,
    /// `semantic_events` — including the aria state), and children; `Div`
    /// takes care of style inheritance, variant resolution (both native
    /// and aria-* variants), hover/press tracking, focus management, and
    /// scrolling.
    fn build_div(&self) -> Div {
        let mut div = Div::new()
            .with_disabled(self.disabled)
            .with_selected(self.selected)
            .with_focusable(self.focusable)
            .with_semantic_events(self.semantic_events.clone());

        // Layer the UA style *beneath* the user overrides. `Div::render`
        // will then merge the combined style on top of the inherited
        // ancestor context — matching the original Button fold chain:
        //
        //     inherited  →  UA  →  user.style
        //
        // (Because `merge_style` only copies fields that differ from
        // `Style::bare()`, pre-merging UA + user style here and then
        // merging the result on top of `inherited` inside `Div` produces
        // the exact same final value as the original three-step chain.)
        let style = if self.use_defaults {
            button_ua_style().merge_style(&self.style)
        } else {
            self.style.clone()
        };
        *div.get_style() = style;

        *div.children_mut() = self.children.clone();
        *div.key_mut() = self.key.clone();
        *div.event_handlers_mut() = self.event_handlers.clone();
        *div.accessibility_data_mut() = self.accessibility.clone();

        div
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledExt for Button {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}
impl ChildrenExt for Button {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}
impl KeyExt for Button {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}
impl EventHandlersExt for Button {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
impl AccessibilityExt for Button {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}
impl HasSemanticEvents for Button {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

fn button_ua_style() -> Style {
    let theme = active_theme();
    let mut style = Style::bare();
    style.background = theme
        .fill("surface")
        .or_else(|| theme.fill("background"))
        .unwrap_or_else(|| StyledColor::rgb(239, 239, 239).into());
    style.padding = Gaps::new(4.0, 12.0, 4.0, 12.0);
    style.radius = CornerRadius::new_all(4.0);
    style.cursor = CursorIcon::Pointer;
    style
}

impl Component for Button {
    fn render(&self) -> impl IntoElement {
        // All style inheritance, variant resolution (native + aria-*),
        // hover/press state, focus management, and (should you ever add
        // it) scrolling is handled by `Div`.
        let div = self.build_div();
        let mut el = div.build_element();

        // `aria-disabled` is treated as disabled for a11y attributes and
        // cursor purposes, matching how `.with_disabled(true)` behaves.
        // The native `disabled` flag already gates the pointer handlers
        // inside `Div::build_element`; this combined check makes an
        // explicit `.with_aria_disabled(true)` behave the same way.
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;

        // Layer the button-specific a11y attributes on top of the `Div`
        // output. `a11y_id` was already set by `Div::build_element`, so
        // these all attach to the same accessibility node.
        el = el
            .a11y_role(AccessibilityRole::Button)
            .a11y_enabled(!effectively_disabled)
            .a11y_focusable(!effectively_disabled && self.focusable);

        // Dispatch on_selected / on_deselected based on the current
        // (controlled) state. Wired as `on_all_press` so it fires for any
        // press event that reached the button.
        if !effectively_disabled && (self.on_selected.is_some() || self.on_deselected.is_some()) {
            let on_selected = self.on_selected.clone();
            let on_deselected = self.on_deselected.clone();
            let was_selected = self.selected;
            el = el.on_all_press(move |e: Event<PressEventData>| {
                if was_selected {
                    if let Some(cb) = &on_deselected {
                        cb.call(e);
                    }
                } else if let Some(cb) = &on_selected {
                    cb.call(e);
                }
            });
        }

        // Disabled state overrides whatever cursor the style requested.
        if effectively_disabled {
            el = el.cursor(CursorIcon::NotAllowed);
        }

        el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
