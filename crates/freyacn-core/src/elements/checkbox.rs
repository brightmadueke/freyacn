use super::div::{Div, div};
use crate::styling::colors::{blue_600, gray_100};
use crate::styling::has::{HasKind, use_has_signal_when};
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{Style, StyledColor, StyledExt, white};
use freya::components::SvgViewer;
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{Component, EventHandlers};
use freya_icons::lucide;

const DEFAULT_SIZE: f32 = 16.0;

/// An HTML `<input type="checkbox">`-like element.
///
/// ## Behavior
///
/// * Controlled: reads `.with_checked(bool)`, fires `.on_checked(...)` /
///   `.on_unchecked(...)`. The caller flips the prop.
/// * Activatable via pointer, Enter, Space, assistive technology.
/// * Focusable when enabled; disabled checkboxes show `NotAllowed` and
///   suppress pointer / activation callbacks.
///
/// ## Accessibility
///
/// Role is `CheckBox`. Checked state is exposed via the node's
/// `description` (`"checked"` / `"unchecked"`).
///
/// ## Aria state
///
/// `with_checked` / `with_disabled` mirror their native flags into the
/// component's `AriaState`, so `aria-checked:*` / `aria-disabled:*`
/// class strings fire alongside their native counterparts.
///
/// ## Composition
///
/// Built on top of `div()`. `Div` owns style resolution, event wiring,
/// a11y id, variant resolution, and `has:` context.
///
/// ## Shadows
///
/// `Checkbox` delegates to `Div`, so box shadows work end-to-end. Text
/// shadows cascade to any `Text` child via `InheritedStyle` — the
/// default `SvgViewer` icon ignores them (no glyphs).
///
/// ## Has: signaling
///
/// Signals [`HasKind::Checked`] when checked and
/// [`HasKind::Disabled`] when disabled, so a parent with
/// `has-[:checked]:` / `has-[:disabled]:` styles fires when this
/// checkbox matches. Subject to the one-frame delay and persistence
/// caveats of [`crate::styling::has`].
#[derive(PartialEq, Clone)]
pub struct Checkbox {
    style: Style,
    event_handlers: EventHandlers,
    key: DiffKey,
    accessibility: AccessibilityData,

    disabled: bool,
    checked: bool,
    size: f32,

    icon: Option<Element>,
    check_color: Option<StyledColor>,

    semantic_events: SemanticEvents,

    on_checked: Option<EventHandler<Event<PressEventData>>>,
    on_unchecked: Option<EventHandler<Event<PressEventData>>>,
}

/// Shorthand constructor: `checkbox().with_checked(true)`
pub fn checkbox() -> Checkbox {
    Checkbox::new()
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),

            disabled: false,
            checked: false,
            size: DEFAULT_SIZE,

            icon: None,
            check_color: None,

            semantic_events: SemanticEvents::default(),

            on_checked: None,
            on_unchecked: None,
        }
    }

    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self.semantic_events.aria_state.checked = v;
        self
    }

    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_element());
        self
    }

    pub fn check_color(mut self, color: impl Into<StyledColor>) -> Self {
        self.check_color = Some(color.into());
        self
    }

    pub fn on_checked(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_checked = Some(cb.into());
        self
    }

    pub fn on_unchecked(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_unchecked = Some(cb.into());
        self
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledExt for Checkbox {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Checkbox {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Checkbox {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Checkbox {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Checkbox {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

fn checkbox_ua_style(size: f32, effectively_disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(size);
    style.height = Size::px(size);
    style.background = gray_100().into();
    style.radius = CornerRadius::new_all(3.0);
    style.cursor = if effectively_disabled {
        CursorIcon::NotAllowed
    } else {
        CursorIcon::Pointer
    };
    style.main_align = Alignment::center();
    style.cross_align = Alignment::center();

    style = style.checked(|mut s| {
        s.background = blue_600().into();
        s
    });
    style = style.disabled(|mut s| {
        s.opacity = 0.5;
        s
    });

    style
}

fn default_check_icon(size: f32, color: StyledColor) -> impl IntoElement {
    SvgViewer::new(lucide::check())
        .width(Size::px(size * 0.7))
        .height(Size::px(size * 0.7))
        .stroke(color)
        .stroke_width(2.5)
}

impl Component for Checkbox {
    fn render(&self) -> impl IntoElement {
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;

        // Signal checked / disabled state to any ancestor with
        // `has-[:checked]:` / `has-[:disabled]:`.
        use_has_signal_when(HasKind::Checked, self.checked);
        use_has_signal_when(HasKind::Disabled, effectively_disabled);

        let mut div_el: Div = div()
            .with_checked(self.checked)
            .with_disabled(effectively_disabled)
            .with_semantic_events(self.semantic_events.clone());

        *div_el.get_style() =
            checkbox_ua_style(self.size, effectively_disabled).merge_style(&self.style);

        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        div_el = div_el.a11y_role(AccessibilityRole::CheckBox);

        let was_checked = self.checked;
        div_el = div_el.a11y_builder(move |node| {
            node.set_description(if was_checked { "checked" } else { "unchecked" });
        });

        if !effectively_disabled && (self.on_checked.is_some() || self.on_unchecked.is_some()) {
            let on_checked = self.on_checked.clone();
            let on_unchecked = self.on_unchecked.clone();
            let was_checked = self.checked;
            div_el = div_el.on_all_press(move |e: Event<PressEventData>| {
                if was_checked {
                    if let Some(cb) = &on_unchecked {
                        cb.call(e);
                    }
                } else if let Some(cb) = &on_checked {
                    cb.call(e);
                }
            });
        }

        if self.checked {
            let icon_el: Element = match &self.icon {
                Some(el) => el.clone(),
                None => default_check_icon(
                    self.size,
                    self.check_color.clone().unwrap_or_else(|| white()),
                )
                .into_element(),
            };
            div_el = div_el.child(icon_el);
        }

        div_el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
