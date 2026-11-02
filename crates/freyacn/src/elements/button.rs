use crate::styling::apply::apply_to_rect;
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledColor, StyledExt};
use crate::theme::active_theme;
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{try_consume_context, use_provide_context};
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

    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn with_focusable(mut self, v: bool) -> Self {
        self.focusable = v;
        self
    }

    /// Controlled selected state (HTML `aria-pressed`).
    ///
    /// Renamed from `selected` so `StyledExt::selected(..)` remains
    /// callable for styling the `:selected` variant:
    ///
    /// ```ignore
    /// button()
    ///     .with_selected(true)                                // state
    ///     .selected(|s| s.background(blue_600().into()))      // variant
    /// ```
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
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
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        // ── 1. Base style ───────────────────────────────────────────
        let inherited = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset(),
            None => Style::default(),
        };
        let base = if self.use_defaults {
            let ua = button_ua_style();
            inherited.merge_style(&ua).merge_style(&self.style)
        } else {
            inherited.merge_style(&self.style)
        };

        // ── 2. Live variants ────────────────────────────────────────
        let variants = &self.style.variants;
        let needs_hover_active_tracking =
            variants.has(StyleVariant::Hover) || variants.has(StyleVariant::Active);

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
        if self.selected {
            live.push(StyleVariant::Selected);
        }

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events ─────────────────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Provide to descendants ───────────────────────────────
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 5. Render ───────────────────────────────────────────────
        let can_focus = !self.disabled && self.focusable;

        let mut el = apply_to_rect(rect(), &style)
            .a11y_id(a11y_id)
            .a11y_role(AccessibilityRole::Button)
            .a11y_enabled(!self.disabled)
            .a11y_focusable(can_focus);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

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

            if self.on_selected.is_some() || self.on_deselected.is_some() {
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
        }

        if self.disabled {
            el = el.cursor(CursorIcon::NotAllowed);
        }

        el.children(self.children.clone())
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
