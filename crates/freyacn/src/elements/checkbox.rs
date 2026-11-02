use super::div::{Div, div};
use crate::styling::colors::{blue_600, gray_100};
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{Style, StyledColor, StyledExt, white};
use freya::components::SvgViewer;
use freya::icons::lucide;
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{Component, EventHandlers};

const DEFAULT_SIZE: f32 = 16.0;

/// An HTML `<input type="checkbox">`-like element.
///
/// ## Behavior
///
/// * Controlled: reads `.with_checked(bool)`, fires `.on_checked(...)` when
///   activated while unchecked, `.on_unchecked(...)` when activated while
///   checked. The caller flips the prop in the handler.
/// * Activatable via pointer, Enter, Space, and assistive technology
///   (via `.on_all_press` on the underlying `div`).
/// * Focusable when enabled; not focusable when disabled.
/// * Disabled checkboxes show `CursorIcon::NotAllowed` and suppress
///   pointer and activation callbacks.
///
/// ## Example
///
/// ```
/// let checked = use_state(|| false);
/// checkbox()
///     .with_checked(*checked.read())
///     .on_checked(move |_| checked.set(true))
///     .on_unchecked(move |_| checked.set(false))
/// ```
///
/// Variants:
///
/// ```
/// use crate::styling::colors::green_600;
/// checkbox().checked(|mut s| {
///     s.background = green_600().into();
///     s
/// })
/// ```
///
/// Custom icon / icon color:
///
/// ```
/// checkbox().icon(text("check").font_size(12.0))
/// checkbox().check_color(green_500())
/// ```
///
/// ## Accessibility
///
/// Role is `CheckBox`. Checked state is exposed through the node's
/// `description` (`"checked"` / `"unchecked"`) because
/// `accesskit::Toggled` isn't reachable in the current dependency tree.
/// The description supplements the label set via `.a11y_alt(..)`, so a
/// custom label is preserved alongside the state.
///
/// ## Composition
///
/// Built on top of `div()`. `div` owns style inheritance, pointer and
/// focus tracking, a11y id, and variant resolution (including `:checked`
/// via `.with_checked(..)`). `Checkbox` only carries UA defaults, forwards
/// semantic events (via `Div::with_semantic_events`), and adds the
/// activation handler + icon child.
///
/// ## Reset limitation
///
/// UA `width`, `height`, `radius`, and `cursor` cannot be reset via
/// `.width(..)` and friends because of `merge_style`'s "differs from
/// bare" rule. Use `.size(..)` to pick a non-default size.
///
/// ## No border field usage
///
/// The box is filled rather than outlined (unchecked = `gray_100`,
/// checked = `blue_600`). If you need a true border, wrap the checkbox
/// in your own `div()` with `.border_*()`.
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

    // ── state ──────────────────────────────────────────────────────

    /// Controlled checked state (HTML `checked`). Named `with_checked`
    /// so `StyledExt::checked(..)` remains callable for styling the
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

    /// Box side length in pixels. Defaults to `16.0`.
    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }

    // ── icon ───────────────────────────────────────────────────────

    /// Custom icon shown when checked. Replaces the default lucide
    /// `check` SVG.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_element());
        self
    }

    /// Color of the default check icon. Ignored when `.icon(..)` is set.
    /// Defaults to `white()` from the FreyaCN palette.
    pub fn check_color(mut self, color: impl Into<StyledColor>) -> Self {
        self.check_color = Some(color.into());
        self
    }

    // ── activation semantics ───────────────────────────────────────

    /// Fires when the box is activated while `checked == false`.
    pub fn on_checked(mut self, cb: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_checked = Some(cb.into());
        self
    }

    /// Fires when the box is activated while `checked == true`.
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

/// HTML `<input type="checkbox">` user-agent defaults.
fn checkbox_ua_style(size: f32, disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(size);
    style.height = Size::px(size);
    style.background = gray_100().into();
    style.radius = CornerRadius::new_all(3.0);
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

fn default_check_icon(size: f32, color: StyledColor) -> impl IntoElement {
    SvgViewer::new(lucide::check())
        .width(Size::px(size * 0.7))
        .height(Size::px(size * 0.7))
        .stroke(color)
        .stroke_width(2.5)
}

impl Component for Checkbox {
    fn render(&self) -> impl IntoElement {
        // `div` owns style resolution, event wiring, and a11y.
        let mut div_el: Div = div()
            .with_checked(self.checked)
            .with_disabled(self.disabled);

        *div_el.get_style() = checkbox_ua_style(self.size, self.disabled).merge_style(&self.style);

        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        div_el = div_el.a11y_role(AccessibilityRole::CheckBox);

        // `accesskit::Toggled` isn't reachable, so encode the checked
        // state in the node's description. `set_description` supplements
        // the label set via `.a11y_alt(..)` rather than replacing it, so
        // a custom label is preserved alongside the state.
        //
        // If a direct `accesskit` dependency is added later, this
        // collapses to `node.set_toggled(..)`:
        //
        //   div_el = div_el.a11y_builder(move |node| {
        //       node.set_toggled(if was_checked { Toggled::True } else { Toggled::False });
        //   });
        let was_checked = self.checked;
        div_el = div_el.a11y_builder(move |node| {
            node.set_description(if was_checked { "checked" } else { "unchecked" });
        });

        // Forward all 8 semantic callbacks in one line.
        div_el = div_el.with_semantic_events(self.semantic_events.clone());

        // Activation: on_checked / on_unchecked.
        if self.on_checked.is_some() || self.on_unchecked.is_some() {
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

        // Icon child when checked.
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
