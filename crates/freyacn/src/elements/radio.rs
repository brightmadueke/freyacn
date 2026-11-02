use super::div::{Div, div};
use crate::styling::colors::{blue_600, gray_300, transparent};
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
///   when activated while unchecked. **Unlike a checkbox, activating a
///   checked radio is a no-op** — HTML radios cannot be unchecked by
///   clicking, and neither can this one. There is no `.on_unchecked`.
/// * Focusable when enabled; not focusable when disabled.
/// * Disabled radios show `CursorIcon::NotAllowed` and suppress
///   activation callbacks.
///
/// ## Example
///
/// ```ignore
/// let selected = use_state(|| "a");
/// radio()
///     .with_checked(*selected.read() == "a")
///     .on_checked(move |_| selected.set("a"))
/// ```
///
/// ## Grouping
///
/// HTML radios with the same `name` form a group where selecting one
/// deselects the others. **This is not implemented** — the caller owns
/// the single-selection logic via `use_state`:
///
/// ```ignore
/// let selected = use_state(|| "small");
///
/// div()
///     .direction(Direction::Vertical)
///     .child(
///         radio()
///             .with_checked(*selected.read() == "small")
///             .on_checked(move |_| selected.set("small"))
///     )
///     .child(
///         radio()
///             .with_checked(*selected.read() == "medium")
///             .on_checked(move |_| selected.set("medium"))
///     )
/// ```
///
/// Roving tabindex / arrow-key navigation within a group is also not
/// implemented — each radio is independently tabbable.
///
/// ## Restyling the ring
///
/// `.checked(|s| s.background(green_600().into()))` recolors the **ring**
/// when checked. Because the **dot** defaults to `blue_600`, you'll need
/// `.dot_color(green_600())` alongside it for a fully consistent radio:
///
/// ```ignore
/// radio()
///     .checked(|s| s.background(green_600().into()))
///     .dot_color(green_600())
/// ```
///
/// This mirrors HTML's `accent-color` — one color for the whole control.
///
/// ## Composition
///
/// Built on top of `div()`. `div` owns:
///
/// * style inheritance (`try_consume_context` / `use_provide_context`),
/// * pointer and hover tracking for `:hover` and `:active`,
/// * focus tracking for `:focus` and `:focus-visible`,
/// * a11y id and focusability,
/// * variant resolution, including `:checked` via `.with_checked(..)`.
///
/// `Radio` only carries the UA defaults, forwards its own semantic
/// events (via `Div::with_semantic_events`), and adds the ring / hole /
/// dot children.
///
/// ## Reset limitation
///
/// UA `width`, `height`, `radius`, and `cursor` cannot be reset via
/// `.width(..)` and friends because of `merge_style`'s "differs from
/// bare" rule. Use `.size(..)` to pick a non-default size.
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

/// Shorthand constructor: `radio().with_checked(true)`
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

    // ── state ──────────────────────────────────────────────────────

    /// Controlled checked state. Named `with_checked` so
    /// `StyledExt::checked(..)` remains callable for styling the
    /// `:checked` variant.
    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }

    /// HTML `disabled`.
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    /// Radio diameter in pixels. Defaults to `16.0`.
    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }

    /// Color of the inner dot when checked. Ignored when unchecked.
    /// Defaults to `blue_600()` from the FreyaCN palette — pair with
    /// `.checked(|s| s.background(same_color))` to keep the ring and
    /// dot consistent.
    pub fn dot_color(mut self, color: impl Into<StyledColor>) -> Self {
        self.dot_color = Some(color.into());
        self
    }

    // ── activation semantics ───────────────────────────────────────

    /// Fires when the radio is activated while unchecked. Clicking an
    /// already-checked radio is a no-op (HTML semantics).
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

/// HTML `<input type="radio">` user-agent defaults.
///
/// The `:checked` variant swaps the outer ring fill from `gray_300` to
/// `blue_600`; the caller can override with `.checked(..)`.
fn radio_ua_style(size: f32, disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(size);
    style.height = Size::px(size);
    style.background = gray_300().into();
    style.radius = CornerRadius::new_all(9999.0); // full circle
    style.padding = Gaps::new_all(RING_WIDTH);
    style.cursor = if disabled {
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
        // ── 1. Base div — owns style resolution, events, a11y ───────
        let mut div_el: Div = div()
            .with_checked(self.checked)
            .with_disabled(self.disabled);

        *div_el.get_style() = radio_ua_style(self.size, self.disabled).merge_style(&self.style);

        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        div_el = div_el.a11y_role(AccessibilityRole::RadioButton);
        div_el = div_el.with_semantic_events(self.semantic_events.clone());

        // ── 2. Activation ───────────────────────────────────────────
        if self.on_checked.is_some() {
            let on_checked = self.on_checked.clone();
            let was_checked = self.checked;
            div_el = div_el.on_all_press(move |e: Event<PressEventData>| {
                // HTML semantics: activating an already-checked radio is a no-op.
                if !was_checked {
                    if let Some(cb) = &on_checked {
                        cb.call(e);
                    }
                }
            });
        }

        // ── 3. Inner "hole" — transparent, so the parent bg shows ──
        // The outer `div`'s 2px padding exposes the ring color around this.
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

        // ── 4. Dot (only when checked) ──────────────────────────────
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
