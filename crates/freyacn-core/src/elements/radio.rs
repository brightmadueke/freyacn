use super::div::{Div, div};
use crate::styling::colors::{blue_600, gray_300, transparent};
use crate::styling::has::{HasKind, use_has_signal_when};
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{Style, StyledColor, StyledExt};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::IntoElement;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{Component, EventHandlers};
use torin::gaps::Gaps;
use torin::size::Size;

const DEFAULT_SIZE: f32 = 16.0;
const RING_WIDTH: f32 = 2.0;

/// An HTML `<input type="radio">`-like element.
///
/// ## Behavior
///
/// * Controlled: reads `.with_checked(bool)`, fires `.on_checked(...)`
///   when activated while unchecked. **Activating an already-checked
///   radio is a no-op** — HTML semantics.
/// * Focusable when enabled; disabled radios suppress pointer callbacks.
///
/// ## Grouping
///
/// Single-selection across radios is not implemented — caller owns the
/// state via `use_state`.
///
/// ## Accessibility
///
/// Role is `RadioButton`. Selected state is exposed via `description`
/// (`"selected"` / `"not selected"`).
///
/// ## Aria state
///
/// `with_checked` / `with_disabled` mirror their native flags.
///
/// ## Composition
///
/// Built on top of `div()`.
///
/// ## Shadows
///
/// Box shadows render on the outer rect (the ring circle) via `Div`.
/// Text shadows don't paint (no glyph content on any of the inner
/// `Div`s).
///
/// ## Has: signaling
///
/// Signals [`HasKind::Checked`] when checked and
/// [`HasKind::Disabled`] when disabled. Subject to the one-frame delay
/// and persistence caveats of [`crate::styling::has`].
#[derive(PartialEq, Clone)]
pub struct Radio {
    style: Style,
    event_handlers: EventHandlers,
    key: DiffKey,
    accessibility: AccessibilityData,

    disabled: bool,
    checked: bool,
    size: f32,

    dot_color: Option<StyledColor>,

    semantic_events: SemanticEvents,
    on_checked: Option<EventHandler<Event<PressEventData>>>,
}

pub fn radio() -> Radio {
    Radio::new()
}

impl Radio {
    pub fn new() -> Self {
        Self {
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),

            disabled: false,
            checked: false,
            size: DEFAULT_SIZE,

            dot_color: None,

            semantic_events: SemanticEvents::default(),
            on_checked: None,
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

    pub fn dot_color(mut self, color: impl Into<StyledColor>) -> Self {
        self.dot_color = Some(color.into());
        self
    }

    pub fn on_checked(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_checked = Some(cb.into());
        self
    }
}

impl Default for Radio {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledExt for Radio {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Radio {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Radio {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Radio {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Radio {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

fn radio_ua_style(size: f32, effectively_disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(size);
    style.height = Size::px(size);
    style.background = gray_300().into();
    style.radius = CornerRadius::new_all(9999.0);
    style.padding = Gaps::new_all(RING_WIDTH);
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

impl Component for Radio {
    fn render(&self) -> impl IntoElement {
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;

        // Signal checked / disabled state to any ancestor with `has-*:`
        // styles.
        use_has_signal_when(HasKind::Checked, self.checked);
        use_has_signal_when(HasKind::Disabled, effectively_disabled);

        let mut div_el: Div = div()
            .with_checked(self.checked)
            .with_disabled(effectively_disabled);

        *div_el.get_style() =
            radio_ua_style(self.size, effectively_disabled).merge_style(&self.style);

        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        div_el = div_el.a11y_role(AccessibilityRole::RadioButton);

        let was_checked = self.checked;
        div_el = div_el.a11y_builder(move |node| {
            node.set_description(if was_checked {
                "selected"
            } else {
                "not selected"
            });
        });

        div_el = div_el.with_semantic_events(self.semantic_events.clone());

        if !effectively_disabled && self.on_checked.is_some() {
            let on_checked = self.on_checked.clone();
            let was_checked = self.checked;
            div_el = div_el.on_all_press(move |e: Event<PressEventData>| {
                if !was_checked {
                    if let Some(cb) = &on_checked {
                        cb.call(e);
                    }
                }
            });
        }

        let inner_size = (self.size - 2.0 * RING_WIDTH).max(0.0);
        let mut inner_style = Style::bare();
        inner_style.width = Size::px(inner_size);
        inner_style.height = Size::px(inner_size);
        inner_style.background = transparent().into();
        inner_style.radius = CornerRadius::new_all(9999.0);
        inner_style.main_align = Alignment::center();
        inner_style.cross_align = Alignment::center();

        let mut inner = div();
        *inner.get_style() = inner_style;

        if self.checked {
            let dot_size = self.size * 0.55;
            let dot_color = self.dot_color.clone().unwrap_or_else(|| blue_600());
            let mut dot_style = Style::bare();
            dot_style.width = Size::px(dot_size);
            dot_style.height = Size::px(dot_size);
            dot_style.background = dot_color.into();
            dot_style.radius = CornerRadius::new_all(9999.0);

            let mut dot = div();
            *dot.get_style() = dot_style;
            inner = inner.child(dot);
        }

        div_el = div_el.child(inner);
        div_el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
