use crate::styling::apply::apply_text_style;
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledExt};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Component, EventHandlers, try_consume_context, use_a11y, use_focus, use_state,
};
use std::borrow::Cow;

/// A run of styled text, styled like an HTML `<span>`.
///
/// ## Composition
///
/// Renders a leaf `label()` primitive — no container. Style inheritance
/// comes from the nearest ancestor that provided `InheritedStyle` (any
/// `Div`, `Paragraph`, `Text`, or other component that calls
/// `use_provide_context`); this leaf does not provide one back, since it
/// has no descendants.
///
/// ## Variants
///
/// All `StyleVariant`s work as on `Div` — `hover:*`, `focus:*`,
/// `disabled:*`, and the full `aria-*` family. Because there is no `Div`
/// in between, the aria state is pushed into the live-variant set
/// directly in this component's `render`.
///
/// ## Aria state
///
/// `with_disabled`, `with_checked`, and `with_selected` mirror their
/// native flags into the component's `AriaState`, so both class families
/// fire from a single setter:
///
/// ```
/// text("Read-only note")
///     .with_disabled(true)
///     .style("disabled:text-gray-400 aria-disabled:cursor-not-allowed")
/// ```
///
/// ```
/// text("Selected item")
///     .with_selected(true)
///     .style("selected:font-semibold aria-selected:font-semibold")
/// ```
///
/// Additional aria states with no native equivalent are set through the
/// [`HasSemanticEvents`] builders:
///
/// ```
/// text("Required")
///     .with_aria_required(true)
///     .style("aria-required:text-red-500 aria-required:after:content-['*']")
/// ```
///
/// A direct `.with_aria_disabled(true)` also disables the text for
/// pointer-handler and focus purposes — the render body unions the aria
/// flags into `effectively_disabled`.
#[derive(PartialEq, Clone)]
pub struct Text {
    content: Cow<'static, str>,
    max_lines: Option<usize>,
    style: Style,
    event_handlers: EventHandlers,
    key: DiffKey,
    accessibility: AccessibilityData,
    disabled: bool,
    checked: bool,
    selected: bool,
    semantic_events: SemanticEvents,
}

/// Shorthand constructor: `text("Hello").style("text-lg font-semibold hover:text-blue-500")`
pub fn text(content: impl Into<Cow<'static, str>>) -> Text {
    Text::new(content)
}

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content: content.into(),
            max_lines: None,
            // `Style::bare()` — see `Div::new`.
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),
            disabled: false,
            checked: false,
            selected: false,
            semantic_events: SemanticEvents::default(),
        }
    }

    pub fn max_lines(mut self, n: impl Into<Option<usize>>) -> Self {
        self.max_lines = n.into();
        self
    }

    /// Disable the text.
    ///
    /// Sets the native `disabled` flag **and** the `aria-disabled` state.
    /// Both `disabled:*` and `aria-disabled:*` class strings fire, and
    /// pointer handlers are suppressed (text is not focusable by
    /// default, so `wants_focus` remains false unless the user registers
    /// a `:focus` variant or a focus callback).
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    /// Controlled checked state.
    ///
    /// Sets the native `checked` flag **and** the `aria-checked` state,
    /// so both `checked:*` and `aria-checked:*` class strings fire.
    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self.semantic_events.aria_state.checked = v;
        self
    }

    /// Controlled selected state.
    ///
    /// Sets the native `selected` flag **and** the `aria-selected` state,
    /// so both `selected:*` and `aria-selected:*` class strings fire.
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self.semantic_events.aria_state.selected = v;
        self
    }
}

impl StyledExt for Text {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Text {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Text {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Text {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Text {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

impl Component for Text {
    fn render(&self) -> impl IntoElement {
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        // Native `disabled` and `aria-disabled` are treated uniformly:
        // either one disables the text for pointer-handler purposes and
        // (via `StyleVariant::Disabled` / `StyleVariant::AriaDisabled`)
        // the corresponding class strings. `with_disabled` syncs the
        // pair, but a caller using `.with_aria_disabled(true)` directly
        // bypasses that sync, so this union keeps behaviour consistent.
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;

        // ── 1. Base style ───────────────────────────────────────────
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

        let mut live = Vec::with_capacity(12);
        if effectively_disabled {
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

        // ── aria-* state → live variants ────────────────────────────
        // `Text` renders a leaf `label()` directly (no `Div`), so the
        // aria variants must be pushed here. Every `aria-*:` class
        // registered on this text (e.g. `aria-invalid:text-red-500`)
        // only fires when the matching `StyleVariant::Aria*` is present
        // in `live`.
        live.extend(self.semantic_events.aria_state.active_variants());

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events ─────────────────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Render ───────────────────────────────────────────────
        // `Text` renders a leaf `label()`, so no `use_provide_context`
        // for descendants — there are none to consume it.
        let mut el =
            apply_style(label().text(self.content.clone()), &style).max_lines(self.max_lines);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        if wants_focus && !effectively_disabled {
            el = el.a11y_id(a11y_id).a11y_focusable(true);
        }

        if !effectively_disabled {
            el = attach_pointer_handlers(
                el,
                &self.semantic_events,
                hovering,
                pressing,
                needs_hover_active_tracking,
                None,
            );
        }

        el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

/// Copies a resolved `Style` onto a `label()`.
///
/// `Label` has no background, border, corner radius, direction or
/// alignment, so those `Style` fields are ignored. Wrap the text in a
/// `div()` for them.
fn apply_style(el: Label, s: &Style) -> Label {
    let el = el
        // box
        .width(s.width.clone())
        .height(s.height.clone())
        .min_width(s.min_width.clone())
        .min_height(s.min_height.clone())
        .max_width(s.max_width.clone())
        .max_height(s.max_height.clone())
        .padding(s.padding)
        .margin(s.margin)
        .position(s.position.clone())
        // effects (note `rotation`, not `rotate`, on `Label`)
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotation(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        .layer(s.layer);

    // text style — shared with paragraph.rs
    let mut el = apply_text_style(el, s);

    // 0.0 is our "unset" value, so only override when a leading class was used.
    if s.line_height > 0.0 {
        el = el.line_height(s.line_height);
    }

    if s.cursor != CursorIcon::default() {
        el = el.cursor(s.cursor);
    }
    if !s.cursor_visible {
        el = el.cursor_visible(false);
    }
    el
}
