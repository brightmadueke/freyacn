use super::div::{Div, div};
use crate::styling::apply::apply_text_style;
use crate::styling::colors::gray_400;
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{InheritedStyle, Style, StyledColor, StyledExt, black, white};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::elements::paragraph::ParagraphCursorExt;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{Component, EventHandlers, try_consume_context, use_state};
use std::borrow::Cow;
use torin::gaps::Gaps;
use torin::size::Size;

const DEFAULT_WIDTH: f32 = 200.0;

/// How the input renders its value.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// HTML `type="text"` — plain text.
    #[default]
    Text,
    /// HTML `type="password"` — mask each char with the given character.
    Password(char),
}

impl InputMode {
    pub fn password() -> Self {
        Self::Password('•')
    }
}

/// A single-line HTML `<input type="text">`-like element.
///
/// ## Behavior
///
/// * **Controlled** — reads `.value(String)`, fires `.on_change(handler)`
///   when the value changes. The parent updates the prop in response.
/// * **Focusable** on mount (unless `.with_disabled(true)`), on Tab, and
///   by clicking.
/// * Enter fires `.on_submit(handler)` with the current value.
/// * `:hover` / `:active` / `:focus` / `:focus-visible` variants and the
///   full semantic-event set (`.on_hover`, `.on_focus`, …) work via the
///   underlying `div()`.
///
/// ## Example
///
/// ```ignore
/// let value = use_state(String::new);
/// input(value.read().clone())
///     .placeholder("Type here")
///     .on_change({
///         let mut value = value;
///         move |text| value.set(text)
///     })
/// ```
///
/// Password:
///
/// ```ignore
/// input(*value.read())
///     .mode(InputMode::password())
///     .on_change(move |text| value.set(text))
/// ```
///
/// Disabled / read-only:
///
/// ```ignore
/// input("locked")
///     .with_disabled(true)
///
/// input(*value.read())
///     .with_read_only(true)   // focusable + cursor places, not editable
/// ```
///
/// ## Known limitations
///
/// Not implemented in this version (all require editor infrastructure
/// that doesn't fit the current `div` + `paragraph` composition):
///
/// * **Horizontal scrolling** — long text is clipped with ellipsis at the
///   input's right edge. Cursor beyond that edge is not visible.
/// * **Selection** (drag, shift-click, select-all) — cursor is placed by
///   clicking, but no range selection.
/// * **Clipboard** (Ctrl+C/V/X) — no `freya_edit` dependency, so no
///   copy/paste pipeline.
/// * **IME preedit** — typed characters append directly; no preedit
///   overlay for CJK composition.
///
/// ## Reset limitation
///
/// UA `width`, `padding`, `radius`, and `cursor` cannot be reset via
/// `.width(size)` and friends because of `merge_style`'s "differs from
/// bare" rule. Use explicit non-default values, or wrap in a `div()` for
/// full control.
#[derive(PartialEq, Clone)]
pub struct Input {
    style: Style,
    event_handlers: EventHandlers,
    key: DiffKey,
    accessibility: AccessibilityData,

    value: String,
    placeholder: Option<Cow<'static, str>>,
    mode: InputMode,
    max_length: Option<usize>,
    auto_focus: bool,
    disabled: bool,
    read_only: bool,

    on_change: Option<EventHandler<String>>,
    on_submit: Option<EventHandler<String>>,

    semantic_events: SemanticEvents,
}

/// Shorthand constructor: `input("initial text")`
pub fn input(value: impl Into<String>) -> Input {
    Input::new(value)
}

impl Input {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),

            value: value.into(),
            placeholder: None,
            mode: InputMode::default(),
            max_length: None,
            auto_focus: false,
            disabled: false,
            read_only: false,

            on_change: None,
            on_submit: None,

            semantic_events: SemanticEvents::default(),
        }
    }

    // ── state ──────────────────────────────────────────────────────

    /// Controlled value. Match with `.on_change(handler)`.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Placeholder shown when `value` is empty. HTML `placeholder`.
    pub fn placeholder(mut self, p: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder = Some(p.into());
        self
    }

    /// Text vs. password masking. Defaults to [`InputMode::Text`].
    pub fn mode(mut self, m: impl Into<InputMode>) -> Self {
        self.mode = m.into();
        self
    }

    /// HTML `maxlength` — maximum character count.
    pub fn max_length(mut self, n: impl Into<Option<usize>>) -> Self {
        self.max_length = n.into();
        self
    }

    /// HTML `autofocus` — request focus on mount.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// HTML `disabled` — not focusable, not editable, muted, cursor
    /// `NotAllowed`.
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    /// HTML `readonly` — focusable and cursor places, but not editable.
    pub fn with_read_only(mut self, v: bool) -> Self {
        self.read_only = v;
        self
    }

    // ── events ─────────────────────────────────────────────────────

    /// Fires with the new value whenever the user edits the text.
    pub fn on_change(mut self, cb: impl Into<EventHandler<String>>) -> Self {
        self.on_change = Some(cb.into());
        self
    }

    /// Fires with the current value when the user presses Enter.
    pub fn on_submit(mut self, cb: impl Into<EventHandler<String>>) -> Self {
        self.on_submit = Some(cb.into());
        self
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl StyledExt for Input {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Input {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Input {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Input {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Input {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

/// HTML `<input type="text">` UA defaults.
///
/// The `:focus` variant is registered with a no-op closure — it makes
/// `Div::render` treat the element as focusable, which is required for
/// Tab focus and click-to-focus.
fn input_ua_style(disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(DEFAULT_WIDTH);
    style.background = white().into();
    style.padding = Gaps::new(6.0, 8.0, 6.0, 8.0);
    style.radius = CornerRadius::new_all(4.0);
    style.overflow = Overflow::Clip;
    style.cursor = if disabled {
        CursorIcon::NotAllowed
    } else {
        CursorIcon::Text
    };

    // Force focusability — `Div::render` only marks the element
    // focusable when a `:focus`-family variant or focus callback is
    // registered.
    style = style.focus(|s| s);

    style = style.disabled(|mut s| {
        s.opacity = 0.5;
        s
    });

    style
}

/// Convert a UTF-16 index (paragraph's cursor space) to a byte offset
/// into `text` (Rust's `String` indexing).
fn utf16_to_byte(text: &str, utf16_index: usize) -> usize {
    let mut utf16_count = 0;
    for (byte_idx, ch) in text.char_indices() {
        if utf16_count >= utf16_index {
            return byte_idx;
        }
        utf16_count += ch.len_utf16();
    }
    text.len()
}

impl Component for Input {
    fn render(&self) -> impl IntoElement {
        let mut cursor = use_state(|| self.value.encode_utf16().count());
        let holder = use_state(ParagraphHolder::default);

        // ── 1. Base style ────────────────────────────────────────
        // bare < inherited < UA < user style
        let inherited = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset(),
            None => Style::default(),
        };
        let ua = input_ua_style(self.disabled);
        let raw_style = inherited.merge_style(&ua).merge_style(&self.style);

        // ── 2. Visible text + cursor ─────────────────────────────
        let display_placeholder = self.value.is_empty() && self.placeholder.is_some();

        let visible_text: Cow<str> = match self.mode {
            InputMode::Password(ch) => {
                let n = self.value.encode_utf16().count();
                Cow::Owned(ch.to_string().repeat(n))
            }
            InputMode::Text => {
                if display_placeholder {
                    self.placeholder.clone().unwrap_or_default()
                } else {
                    Cow::Borrowed(&self.value)
                }
            }
        };

        let display_cursor = if self.disabled || display_placeholder {
            None
        } else {
            let max = visible_text.encode_utf16().count();
            Some((*cursor.read()).min(max))
        };

        // ── 3. Key handling ──────────────────────────────────────
        let read_only = self.read_only || self.disabled;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let value_for_keys = self.value.clone();
        let max_len = self.max_length;

        let on_key_down = move |e: Event<KeyboardEventData>| {
            if read_only {
                return;
            }

            let mut text = value_for_keys.clone();
            let cur_utf16 = (*cursor.read()).min(text.encode_utf16().count());
            let cur_byte = utf16_to_byte(&text, cur_utf16);

            match &e.key {
                Key::Character(c) if !c.is_empty() => {
                    if c.chars().any(|ch| !ch.is_control()) {
                        if let Some(max) = max_len {
                            if text.chars().count() + c.chars().count() > max {
                                return;
                            }
                        }
                        text.insert_str(cur_byte, c);
                        cursor.set(cur_utf16 + c.encode_utf16().count());
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                Key::Named(NamedKey::Backspace) => {
                    if cur_byte > 0 {
                        let prev = text[..cur_byte].chars().last().unwrap();
                        text.drain(cur_byte - prev.len_utf8()..cur_byte);
                        cursor.set(cur_utf16 - prev.len_utf16());
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                Key::Named(NamedKey::Delete) => {
                    if cur_byte < text.len() {
                        let next = text[cur_byte..].chars().next().unwrap();
                        text.drain(cur_byte..cur_byte + next.len_utf8());
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    if cur_utf16 > 0 {
                        cursor.set(cur_utf16 - 1);
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    let total = text.encode_utf16().count();
                    if cur_utf16 < total {
                        cursor.set(cur_utf16 + 1);
                    }
                }
                Key::Named(NamedKey::Home) => cursor.set(0),
                Key::Named(NamedKey::End) => {
                    cursor.set(text.encode_utf16().count());
                }
                Key::Named(NamedKey::Enter) => {
                    if let Some(cb) = &on_submit {
                        cb.call(text);
                    }
                }
                _ => {}
            }
        };

        // ── 4. Click to position cursor ──────────────────────────
        let holder_for_press = holder.clone();
        let on_focus_press = move |e: Event<FocusPressEventData>| {
            if read_only {
                return;
            }
            let holder_ref = holder_for_press.peek();
            let holder_borrow = holder_ref.0.borrow();
            if let Some(inner) = holder_borrow.as_ref() {
                // `element_location()` already returns `CursorPoint`
                // (`Point2D<f64, ()>`), which is exactly what
                // `cursor_index_at_point` expects.
                let idx = inner
                    .paragraph
                    .cursor_index_at_point(e.element_location().to_tuple());
                cursor.set(idx);
            }
        };

        // ── 5. Cursor color via `StyledColor` ────────────────────
        // `paragraph().cursor_color(..)` takes `Into<Color>`.
        // `StyledColor::to_color()` resolves the solid color with any
        // opacity applied; `None` when the fill is a gradient, in
        // which case we fall back to black.
        let cursor_color: Color = StyledColor::from(raw_style.text_color.clone())
            .to_color()
            .unwrap_or(black().into());

        // ── 6. Paragraph child ───────────────────────────────────
        //
        // Using the `paragraph()` primitive directly rather than `p(..)`:
        // the input needs `holder`, `cursor_index`, and `cursor_color`
        // — APIs that the `p(..)` wrapper doesn't expose. If `p(..)`
        // gains `.holder(..)` / `.cursor_index(..)` / `.cursor_color(..)`,
        // this can switch to the wrapper.
        let mut para = paragraph()
            .holder(holder.read().clone())
            .min_width(Size::percent(100.0))
            .max_lines(1)
            .cursor_color(cursor_color);

        para = apply_text_style(para, &raw_style);

        if display_placeholder {
            para = para.color(gray_400());
        }
        if let Some(idx) = display_cursor {
            para = para.cursor_index(idx);
        }
        para = para.span(visible_text.to_string());

        if !read_only {
            para = para.on_focus_press(on_focus_press);
        }

        // ── 7. Outer Div — owns focus, hover, semantic events ────
        let mut div_el: Div = div()
            .with_disabled(self.disabled)
            .with_semantic_events(self.semantic_events.clone());

        *div_el.get_style() = raw_style;
        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        div_el = div_el
            .a11y_enabled(!self.disabled)
            .a11y_auto_focus(self.auto_focus)
            .a11y_role(AccessibilityRole::TextInput)
            .a11y_alt(self.value.clone());

        if !read_only {
            div_el = div_el.on_key_down(on_key_down);
        }

        div_el = div_el.child(para);

        div_el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
