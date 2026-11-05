use crate::styling::apply::{apply_text_style, apply_to_rect};
use crate::styling::colors::{black, gray_400};
use crate::styling::has::{HasKind, use_has_signal_when};
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledColor, StyledExt};
use freya::components::{ScrollConfig, ScrollController, ScrollView, use_scroll_controller};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::elements::paragraph::ParagraphCursorExt;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Component, EventHandlers, try_consume_context, use_a11y, use_focus, use_side_effect, use_state,
};
use std::borrow::Cow;
use torin::gaps::Gaps;
use torin::prelude::Area;
use torin::size::Size;

const DEFAULT_WIDTH: f32 = 200.0;

/// What kind of value the input represents.
///
/// Mirrors the HTML `<input type="...">` attribute. Drives:
///
/// - **Character filtering** — `Tel` and `Number` restrict the accepted
///   characters on both typing and paste. `Text`, `Email`, `Url`,
///   `Search`, and `Password` accept everything.
/// - **A11y role** — `Password(_)` uses `PasswordInput`; everything else
///   uses `TextInput`. Accesskit has no dedicated `Email` / `Tel` /
///   `Number` / `Search` roles, so the only differentiation is password
///   masking.
/// - **Display masking** — `Password(ch)` renders every character as
///   `ch`.
///
/// `Email`, `Tel`, `Url`, and `Search` are semantically distinct but
/// behave identically to `Text` in this implementation. They exist so
/// `style_variant(...)` calls can branch on the type, and so future
/// enhancements (URL validation, email keyboard hints on mobile, search
/// clear buttons) have a place to live.
///
/// **Validation is not enforced by type.** Use `min_length`,
/// `max_length`, `required`, and `is_valid()` to build validation
/// semantics — matching HTML, where `<input type="email">` will happily
/// accept `"notanemail"` until the form is submitted.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// HTML `type="text"` — plain text.
    #[default]
    Text,
    /// HTML `type="password"` — mask each char with the given character.
    Password(char),
    /// HTML `type="email"` — plain text, no character filtering.
    Email,
    /// HTML `type="tel"` — filters to digits and phone symbols
    /// (`+`, `-`, space, `(`, `)`, `.`, `*`, `#`).
    Tel,
    /// HTML `type="url"` — plain text, no character filtering.
    Url,
    /// HTML `type="number"` — filters to digits, `+`, `-`, `.`, `e`, `E`.
    /// Does not validate numeric grammar (multiple dots are accepted,
    /// matching browser behaviour — validation is the caller's job).
    Number,
    /// HTML `type="search"` — plain text, no character filtering.
    /// Kept distinct from `Text` so callers can branch on it; the
    /// native clear-button behaviour is not implemented.
    Search,
}

impl InputMode {
    pub fn password() -> Self {
        Self::Password('•')
    }

    /// `true` when the mode masks the value for display and a11y.
    pub fn is_masked(self) -> bool {
        matches!(self, InputMode::Password(_))
    }

    /// The character used for masking, if any.
    pub fn mask_char(self) -> Option<char> {
        match self {
            InputMode::Password(ch) => Some(ch),
            _ => None,
        }
    }

    /// Whether `ch` is allowed to appear in this input.
    ///
    /// Only `Tel` and `Number` filter; the rest accept all characters.
    /// Whitespace-only characters are filtered out separately (see
    /// `on_key_down`) — this predicate is purely type-driven.
    pub fn allows_char(self, ch: char) -> bool {
        match self {
            InputMode::Text
            | InputMode::Email
            | InputMode::Url
            | InputMode::Search
            | InputMode::Password(_) => true,
            InputMode::Tel => ch.is_ascii_digit() || "+- ()*#.".contains(ch),
            InputMode::Number => ch.is_ascii_digit() || "+-eE.".contains(ch),
        }
    }
}

/// A validation failure from [`Input::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    /// The value is empty and `required(true)` was set.
    Required,
    /// The value has fewer characters than `min_length`.
    TooShort { min: usize, actual: usize },
    /// The value has more characters than `max_length`.
    ///
    /// In practice this can only happen via programmatic `.value(..)`
    /// that already exceeds the limit — normal typing and paste are
    /// blocked by the key handler.
    TooLong { max: usize, actual: usize },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Required => write!(f, "value is required"),
            ValidationError::TooShort { min, actual } => {
                write!(f, "value is too short ({actual} chars, minimum {min})")
            }
            ValidationError::TooLong { max, actual } => {
                write!(f, "value is too long ({actual} chars, maximum {max})")
            }
        }
    }
}

/// A single-line HTML `<input type="text">`-like element.
///
/// ## Behavior
///
/// * **Controlled** — reads `.value(String)`, fires `.on_change(..)`.
/// * **Focusable**, selectable, clipboard-aware, IME-aware.
/// * Enter fires `.on_submit(..)`.
/// * `read_only(true)` — focusable, selectable, copyable; typing,
///   Backspace, Delete, Ctrl+X, Ctrl+V do nothing.
/// * `max_length(n)` — rejects character insertion (and clips pasted
///   text) once the resulting length would exceed `n`.
/// * `min_length(n)` — **validation only** (matches HTML `<input minlength>`
///   semantics). Does not block backspace or paste. Use `.validate()` /
///   `.is_valid()` to check.
/// * `required(true)` — **validation only**. Same semantics.
///
/// ## Types
///
/// See [`InputMode`]. `Tel` and `Number` filter characters on typing
/// and paste. `Password` masks the display. The rest accept everything.
///
/// ## Validation
///
/// ```rust,ignore
/// let v = use_state(String::new);
/// let text = (*v.read()).clone();
///
/// let el = input(text.clone())
///     .required(true)
///     .min_length(3)
///     .max_length(64)
///     .with_aria_invalid(!input(text.clone()).required(true).min_length(3).max_length(64).is_valid())
///     .on_change(move |t| v.set(t));
/// ```
///
/// `validate()` returns `Option<ValidationError>` with a specific
/// reason; `is_valid()` is the boolean form.
///
/// ## Caret
///
/// A text-insertion caret is rendered at the current cursor position
/// whenever the input is focused and enabled. It shows even on empty
/// inputs with placeholders. It's a static bar, not blinking.
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
    min_length: Option<usize>,
    required: bool,
    auto_focus: bool,
    disabled: bool,
    read_only: bool,

    on_change: Option<EventHandler<String>>,
    on_submit: Option<EventHandler<String>>,

    semantic_events: SemanticEvents,
}

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
            min_length: None,
            required: false,
            auto_focus: false,
            disabled: false,
            read_only: false,

            on_change: None,
            on_submit: None,

            semantic_events: SemanticEvents::default(),
        }
    }

    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn placeholder(mut self, p: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    pub fn mode(mut self, m: impl Into<InputMode>) -> Self {
        self.mode = m.into();
        self
    }
    /// HTML `maxlength` — the input refuses to accept more than `n`
    /// characters. Character count is Unicode scalar values.
    pub fn max_length(mut self, n: impl Into<Option<usize>>) -> Self {
        self.max_length = n.into();
        self
    }
    /// HTML `minlength` — validation only. Does **not** prevent the user
    /// from deleting below `n`. Matches browser behaviour; call
    /// `.is_valid()` to check.
    pub fn min_length(mut self, n: impl Into<Option<usize>>) -> Self {
        self.min_length = n.into();
        self
    }
    /// HTML `required` — validation only. Fails when `.value("")`.
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }
    pub fn with_read_only(mut self, v: bool) -> Self {
        self.read_only = v;
        self.semantic_events.aria_state.readonly = v;
        self
    }
    pub fn on_change(mut self, cb: impl Into<EventHandler<String>>) -> Self {
        self.on_change = Some(cb.into());
        self
    }
    pub fn on_submit(mut self, cb: impl Into<EventHandler<String>>) -> Self {
        self.on_submit = Some(cb.into());
        self
    }

    /// Check the current `.value(..)` against `required`, `min_length`,
    /// and `max_length`. Returns `None` when valid, `Some(reason)`
    /// otherwise.
    ///
    /// Reads the snapshot of `.value(..)` at construction time — the
    /// input itself is controlled, so call this on a fresh
    /// `input(new_value)` in the parent's render to get current
    /// validation state.
    pub fn validate(&self) -> Option<ValidationError> {
        let len = self.value.chars().count();
        if self.required && len == 0 {
            return Some(ValidationError::Required);
        }
        if let Some(min) = self.min_length {
            if len < min {
                return Some(ValidationError::TooShort { min, actual: len });
            }
        }
        if let Some(max) = self.max_length {
            if len > max {
                return Some(ValidationError::TooLong { max, actual: len });
            }
        }
        None
    }

    /// Boolean form of [`validate`](Self::validate).
    pub fn is_valid(&self) -> bool {
        self.validate().is_none()
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

fn input_ua_style(effectively_disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(DEFAULT_WIDTH);
    style.background = Fill::default();
    style.padding = Gaps::new(6.0, 8.0, 6.0, 8.0);
    style.radius = CornerRadius::new_all(4.0);
    style.overflow = Overflow::Clip;
    style.cursor = if effectively_disabled {
        CursorIcon::NotAllowed
    } else {
        CursorIcon::Text
    };
    style.direction = Direction::Horizontal;
    style.main_align = Alignment::Start;
    style.cross_align = Alignment::center();
    style
}

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

fn selection_char_count(text: &str, sel: Option<(usize, usize)>) -> usize {
    match sel {
        Some((start, end)) => {
            let sb = utf16_to_byte(text, start);
            let eb = utf16_to_byte(text, end);
            text[sb..eb].chars().count()
        }
        None => 0,
    }
}

fn follow_cursor(
    holder: State<ParagraphHolder>,
    text: &str,
    cursor_index: usize,
    text_align: TextAlign,
    padding: Gaps,
    para_area: State<Area>,
    viewport: State<Area>,
    mut scroll: ScrollController,
) {
    let viewport_area = *viewport.read();
    if viewport_area.width() == 0.0 {
        return;
    }
    let peeked = holder.peek();
    let holder_borrow = (*peeked).0.borrow();
    let Some(inner) = holder_borrow.as_ref() else {
        return;
    };
    let cursor_rect = inner.paragraph.cursor_rect(text, cursor_index, text_align);
    let scale = inner.scale_factor as f32;
    let cursor_x = cursor_rect.left / scale;
    let visible_cursor_x = cursor_x + padding.left();
    let visible_start_x = viewport_area.min_x() - para_area.read().min_x();
    let visible_end_x = visible_start_x + viewport_area.width();
    if visible_cursor_x < visible_start_x {
        scroll.scroll_to_x(-cursor_x as i32);
    } else if visible_cursor_x > visible_end_x {
        scroll.scroll_to_x(-(visible_cursor_x + padding.right() - viewport_area.width()) as i32);
    }
}

/// Filter a pasted string to the characters allowed by `mode`.
///
/// Control characters (newlines, tab) are always stripped — inputs are
/// single-line.
fn filter_paste(pasted: &str, mode: InputMode) -> String {
    pasted
        .chars()
        .filter(|ch| !ch.is_control())
        .filter(|ch| mode.allows_char(*ch))
        .collect()
}

impl Component for Input {
    fn render(&self) -> impl IntoElement {
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        let mut cursor = use_state(|| self.value.encode_utf16().count());
        let mut anchor = use_state(|| None::<usize>);
        let mut is_dragging = use_state(|| false);
        let mut pressing = use_state(|| false);
        let mut hovering = use_state(|| false);
        let holder = use_state(ParagraphHolder::default);
        let mut preedit = use_state(|| None::<String>);
        let scroll_controller = use_scroll_controller(ScrollConfig::default);
        let mut para_area = use_state(Area::default);
        let mut viewport_area = use_state(Area::default);

        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;
        let effective_read_only =
            self.read_only || self.semantic_events.aria_state.readonly || effectively_disabled;

        use_has_signal_when(HasKind::Disabled, effectively_disabled);

        let inherited = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset(),
            None => Style::default(),
        };
        let ua = input_ua_style(effectively_disabled);
        let raw_style = inherited.merge_style(&ua).merge_style(&self.style);

        let focus_state = focus();
        let focused = focus_state != Focus::Not;
        let focused_kb = focus_state == Focus::Keyboard;

        let mut live: Vec<StyleVariant> = Vec::with_capacity(12);
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
        live.extend(self.semantic_events.aria_state.active_variants());
        live.extend(self.semantic_events.has_state.active_variants());

        let styled = raw_style.variants.apply_all(&live, raw_style.clone());

        let cursor_icon = if effectively_disabled {
            CursorIcon::NotAllowed
        } else {
            CursorIcon::Text
        };

        let display_placeholder =
            self.value.is_empty() && self.placeholder.is_some() && preedit.read().is_none();

        let committed_utf16 = self.value.encode_utf16().count();
        let cursor_utf16 = (*cursor.read()).min(committed_utf16);
        let cursor_byte = utf16_to_byte(&self.value, cursor_utf16);

        let selection_enabled = !matches!(self.mode, InputMode::Password(_));
        let selection_range: Option<(usize, usize)> = if selection_enabled {
            match (*anchor.read(), *cursor.read()) {
                (Some(a), c) if a != c => Some((a.min(c), a.max(c))),
                _ => None,
            }
        } else {
            None
        };

        let (before_value, after_value) = self.value.split_at(cursor_byte);
        let (before_display, after_display) = match self.mode.mask_char() {
            Some(ch) => (
                ch.to_string().repeat(before_value.encode_utf16().count()),
                ch.to_string().repeat(after_value.encode_utf16().count()),
            ),
            None => (before_value.to_string(), after_value.to_string()),
        };

        let preedit_masked: Option<String> =
            preedit
                .read()
                .as_ref()
                .map(|p| match self.mode.mask_char() {
                    Some(ch) => ch.to_string().repeat(p.chars().count()),
                    None => p.clone(),
                });
        let preedit_utf16_len = preedit_masked
            .as_ref()
            .map(|p| p.encode_utf16().count())
            .unwrap_or(0);

        let display_cursor = if effectively_disabled || !focused {
            None
        } else if preedit_masked.is_some() {
            Some(cursor_utf16 + preedit_utf16_len)
        } else {
            Some(cursor_utf16)
        };

        // ── Key handler ────────────────────────────────────────────
        let read_only = effective_read_only;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let value_for_keys = self.value.clone();
        let max_len = self.max_length;
        let sel_enabled = selection_enabled;
        let mode = self.mode;

        let on_key_down = move |e: Event<KeyboardEventData>| {
            if matches!(e.key, Key::Named(NamedKey::Tab)) {
                return;
            }
            if effectively_disabled {
                return;
            }

            let mut text = value_for_keys.clone();
            let cur_utf16 = (*cursor.read()).min(text.encode_utf16().count());
            let cur_byte = utf16_to_byte(&text, cur_utf16);

            let sel: Option<(usize, usize)> = if sel_enabled {
                match (*anchor.read(), *cursor.read()) {
                    (Some(a), c) if a != c => Some((a.min(c), a.max(c))),
                    _ => None,
                }
            } else {
                None
            };

            let delete_sel = |text: &mut String, sel: Option<(usize, usize)>| -> usize {
                match sel {
                    Some((start, end)) => {
                        let s = utf16_to_byte(text, start);
                        let e = utf16_to_byte(text, end);
                        text.drain(s..e);
                        start
                    }
                    None => cur_utf16,
                }
            };

            match &e.key {
                Key::Named(NamedKey::ArrowLeft) => {
                    let extend = sel_enabled && e.modifiers.shift();
                    let new_cursor = cur_utf16.saturating_sub(1);
                    if extend {
                        if (*anchor.read()).is_none() {
                            anchor.set(Some(cur_utf16));
                        }
                    } else {
                        anchor.set(None);
                    }
                    cursor.set(new_cursor);
                    return;
                }
                Key::Named(NamedKey::ArrowRight) => {
                    let extend = sel_enabled && e.modifiers.shift();
                    let total = text.encode_utf16().count();
                    let new_cursor = (cur_utf16 + 1).min(total);
                    if extend {
                        if (*anchor.read()).is_none() {
                            anchor.set(Some(cur_utf16));
                        }
                    } else {
                        anchor.set(None);
                    }
                    cursor.set(new_cursor);
                    return;
                }
                Key::Named(NamedKey::Home) => {
                    if sel_enabled && e.modifiers.shift() {
                        if (*anchor.read()).is_none() {
                            anchor.set(Some(cur_utf16));
                        }
                    } else {
                        anchor.set(None);
                    }
                    cursor.set(0);
                    return;
                }
                Key::Named(NamedKey::End) => {
                    if sel_enabled && e.modifiers.shift() {
                        if (*anchor.read()).is_none() {
                            anchor.set(Some(cur_utf16));
                        }
                    } else {
                        anchor.set(None);
                    }
                    cursor.set(text.encode_utf16().count());
                    return;
                }
                Key::Character(c)
                    if e.modifiers.ctrl() && c.eq_ignore_ascii_case("a") && sel_enabled =>
                {
                    anchor.set(Some(0));
                    cursor.set(text.encode_utf16().count());
                    return;
                }
                Key::Character(c) if e.modifiers.ctrl() && c.eq_ignore_ascii_case("c") => {
                    let copy = match sel {
                        Some((s, end)) => {
                            let sb = utf16_to_byte(&text, s);
                            let eb = utf16_to_byte(&text, end);
                            text[sb..eb].to_string()
                        }
                        None => text.clone(),
                    };
                    if !copy.is_empty() {
                        let _ = Clipboard::set(copy);
                    }
                    return;
                }
                Key::Named(NamedKey::Enter) => {
                    if let Some(cb) = &on_submit {
                        cb.call(text);
                    }
                    return;
                }
                _ => {}
            }

            if read_only {
                return;
            }

            match &e.key {
                Key::Character(c) if e.modifiers.ctrl() && c.eq_ignore_ascii_case("x") => {
                    let copy = match sel {
                        Some((s, end)) => {
                            let sb = utf16_to_byte(&text, s);
                            let eb = utf16_to_byte(&text, end);
                            text[sb..eb].to_string()
                        }
                        None => text.clone(),
                    };
                    if !copy.is_empty() {
                        let _ = Clipboard::set(copy);
                    }
                    let new_cur = if sel.is_some() {
                        delete_sel(&mut text, sel)
                    } else {
                        text.clear();
                        0
                    };
                    cursor.set(new_cur);
                    anchor.set(None);
                    if let Some(cb) = &on_change {
                        cb.call(text);
                    }
                }
                Key::Character(c) if e.modifiers.ctrl() && c.eq_ignore_ascii_case("v") => {
                    if let Ok(pasted) = Clipboard::get() {
                        // Filter by type, then clip to remaining
                        // `max_length` room.
                        let filtered = filter_paste(&pasted, mode);
                        if filtered.is_empty() {
                            return;
                        }
                        let to_insert = match max_len {
                            Some(max) => {
                                let current = text.chars().count();
                                let sel_chars = selection_char_count(&text, sel);
                                let after_delete = current.saturating_sub(sel_chars);
                                let room = max.saturating_sub(after_delete);
                                if room == 0 {
                                    return;
                                }
                                filtered.chars().take(room).collect::<String>()
                            }
                            None => filtered,
                        };
                        let new_cur = delete_sel(&mut text, sel);
                        let insert_byte = utf16_to_byte(&text, new_cur);
                        text.insert_str(insert_byte, &to_insert);
                        cursor.set(new_cur + to_insert.encode_utf16().count());
                        anchor.set(None);
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                Key::Character(c) if !c.is_empty() => {
                    // Filter the typed character(s) by type. A single
                    // key event usually carries one char, but IME /
                    // dead keys can carry several — filter each.
                    let filtered: String = c
                        .chars()
                        .filter(|ch| !ch.is_control())
                        .filter(|ch| mode.allows_char(*ch))
                        .collect();
                    if filtered.is_empty() {
                        return;
                    }
                    if let Some(max) = max_len {
                        let current = text.chars().count();
                        let sel_chars = selection_char_count(&text, sel);
                        let after_delete = current.saturating_sub(sel_chars);
                        if after_delete + filtered.chars().count() > max {
                            // If inserting would exceed max, trim to
                            // fit rather than rejecting the whole key.
                            let room = max.saturating_sub(after_delete);
                            if room == 0 {
                                return;
                            }
                            let new_cur = delete_sel(&mut text, sel);
                            let to_insert: String = filtered.chars().take(room).collect();
                            let insert_byte = utf16_to_byte(&text, new_cur);
                            text.insert_str(insert_byte, &to_insert);
                            cursor.set(new_cur + to_insert.encode_utf16().count());
                            anchor.set(None);
                            if let Some(cb) = &on_change {
                                cb.call(text);
                            }
                            return;
                        }
                    }
                    let new_cur = delete_sel(&mut text, sel);
                    let insert_byte = utf16_to_byte(&text, new_cur);
                    text.insert_str(insert_byte, &filtered);
                    cursor.set(new_cur + filtered.encode_utf16().count());
                    anchor.set(None);
                    if let Some(cb) = &on_change {
                        cb.call(text);
                    }
                }
                Key::Named(NamedKey::Backspace) => {
                    // Note: `min_length` does NOT block backspace.
                    // Matches HTML `<input minlength>` — deleting below
                    // the minimum is allowed, validation fails, caller
                    // decides what to do.
                    let new_cur = if sel.is_some() {
                        delete_sel(&mut text, sel)
                    } else if cur_byte > 0 {
                        let prev = text[..cur_byte].chars().last().unwrap();
                        text.drain(cur_byte - prev.len_utf8()..cur_byte);
                        cur_utf16 - prev.len_utf16()
                    } else {
                        cur_utf16
                    };
                    cursor.set(new_cur);
                    anchor.set(None);
                    if let Some(cb) = &on_change {
                        cb.call(text);
                    }
                }
                Key::Named(NamedKey::Delete) => {
                    let new_cur = if sel.is_some() {
                        delete_sel(&mut text, sel)
                    } else if cur_byte < text.len() {
                        let next = text[cur_byte..].chars().next().unwrap();
                        text.drain(cur_byte..cur_byte + next.len_utf8());
                        cur_utf16
                    } else {
                        cur_utf16
                    };
                    cursor.set(new_cur);
                    anchor.set(None);
                    if let Some(cb) = &on_change {
                        cb.call(text);
                    }
                }
                _ => {}
            }
        };

        let on_rect_focus_press = {
            let holder = holder.clone();
            let para_area = para_area.clone();
            let mut cursor = cursor.clone();
            let mut anchor = anchor.clone();
            let mut is_dragging = is_dragging.clone();
            move |e: Event<FocusPressEventData>| {
                if effectively_disabled {
                    return;
                }
                a11y_id.request_focus();
                is_dragging.set(true);
                let peeked = holder.peek();
                let holder_borrow = (*peeked).0.borrow();
                if let Some(inner) = holder_borrow.as_ref() {
                    let area = *para_area.read();
                    let loc = e.global_location().to_tuple();
                    let relative = (loc.0 - area.min_x() as f64, loc.1 - area.min_y() as f64);
                    let idx = inner.paragraph.cursor_index_at_point(relative);
                    anchor.set(Some(idx));
                    cursor.set(idx);
                }
            }
        };

        let on_para_focus_press = {
            let holder = holder.clone();
            let mut cursor = cursor.clone();
            let mut anchor = anchor.clone();
            let mut is_dragging = is_dragging.clone();
            move |e: Event<FocusPressEventData>| {
                if effectively_disabled {
                    return;
                }
                a11y_id.request_focus();
                is_dragging.set(true);
                let peeked = holder.peek();
                let holder_borrow = (*peeked).0.borrow();
                if let Some(inner) = holder_borrow.as_ref() {
                    let idx = inner
                        .paragraph
                        .cursor_index_at_point(e.element_location().to_tuple());
                    anchor.set(Some(idx));
                    cursor.set(idx);
                }
            }
        };

        let on_global_pointer_move = {
            let holder = holder.clone();
            let para_area = para_area.clone();
            let mut cursor = cursor.clone();
            let is_dragging = is_dragging.clone();
            move |e: Event<PointerEventData>| {
                if !*is_dragging.read() || effectively_disabled {
                    return;
                }
                let peeked = holder.peek();
                let holder_borrow = (*peeked).0.borrow();
                if let Some(inner) = holder_borrow.as_ref() {
                    let area = *para_area.read();
                    let loc = e.global_location().to_tuple();
                    let relative = (loc.0 - area.min_x() as f64, loc.1 - area.min_y() as f64);
                    let idx = inner.paragraph.cursor_index_at_point(relative);
                    cursor.set(idx);
                }
            }
        };

        let on_global_pointer_up = {
            let mut is_dragging = is_dragging.clone();
            let mut pressing = pressing.clone();
            move |_: Event<PointerEventData>| {
                is_dragging.set_if_modified(false);
                pressing.set_if_modified(false);
            }
        };

        let on_pointer_enter = {
            let mut hovering = hovering.clone();
            move |_: Event<PointerEventData>| {
                hovering.set(true);
            }
        };
        let on_pointer_leave = {
            let mut hovering = hovering.clone();
            move |_: Event<PointerEventData>| {
                hovering.set(false);
            }
        };
        let on_pointer_down = {
            let mut pressing = pressing.clone();
            move |_: Event<PointerEventData>| {
                pressing.set(true);
            }
        };
        let on_ime_preedit = {
            let mut preedit = preedit.clone();
            let mode_for_preedit = self.mode;
            move |e: Event<ImePreeditEventData>| {
                let raw = e.data().text.clone();
                // Filter the preedit text to the allowed characters so
                // masked / number / tel inputs don't show disallowed
                // chars during composition.
                let filtered: String = raw
                    .chars()
                    .filter(|ch| !ch.is_control())
                    .filter(|ch| mode_for_preedit.allows_char(*ch))
                    .collect();
                preedit.set(if filtered.is_empty() {
                    None
                } else {
                    Some(filtered)
                });
            }
        };

        let cursor_color: Color = StyledColor::from(styled.text_color.clone())
            .to_color()
            .unwrap_or(black().into());

        let mut para = paragraph()
            .holder(holder.read().clone())
            .min_width(Size::percent(100.0))
            .max_lines(1)
            .padding(Gaps::new_all(0.0))
            .cursor_color(cursor_color)
            .cursor(cursor_icon)
            .on_sized({
                let mut para_area = para_area.clone();
                move |e: Event<SizedEventData>| {
                    para_area.set_if_modified(e.area);
                }
            });

        para = apply_text_style(para, &styled);

        if display_placeholder {
            para = para.color(gray_400());
            let placeholder = self.placeholder.clone().unwrap_or_default();
            para = para.span(placeholder.to_string());
        } else {
            para = para.span(before_display);
            if let Some(p) = &preedit_masked {
                para = para.span(Span::new(p.clone()).text_decoration(TextDecoration::Underline));
            }
            para = para.span(after_display);

            if let Some((s, e)) = selection_range {
                para = para.highlights(vec![(s, e)]);
            }
        }

        if let Some(idx) = display_cursor {
            para = para.cursor_index(idx);
        }

        if !effectively_disabled {
            para = para.on_focus_press(on_para_focus_press);
        }

        let visible_text_for_scroll: String = match self.mode.mask_char() {
            Some(ch) => ch.to_string().repeat(self.value.encode_utf16().count()),
            None => self.value.clone(),
        };
        let cursor_for_scroll = cursor_utf16;
        let padding_for_scroll = styled.padding;
        let text_align_for_scroll = styled.text_align;
        let holder_for_scroll = holder.clone();
        let para_area_for_scroll = para_area.clone();
        let viewport_for_scroll = viewport_area.clone();
        let scroll_for_effect = scroll_controller;

        use_side_effect(move || {
            follow_cursor(
                holder_for_scroll,
                visible_text_for_scroll.as_str(),
                cursor_for_scroll,
                text_align_for_scroll,
                padding_for_scroll,
                para_area_for_scroll,
                viewport_for_scroll,
                scroll_for_effect,
            );
        });

        let scroll_view = ScrollView::new_controlled(scroll_controller)
            .direction(Direction::Horizontal)
            .show_scrollbar(false)
            .width(Size::flex(1.0))
            .height(Size::Inner)
            .on_sized({
                let mut viewport_area = viewport_area.clone();
                move |e: Event<SizedEventData>| {
                    viewport_area.set_if_modified(e.area);
                }
            })
            .child(para);

        let mut rect_el = apply_to_rect(rect(), &styled);
        rect_el = rect_el.cursor(cursor_icon);

        *rect_el.event_handlers_mut() = self.event_handlers.clone();
        *rect_el.accessibility_data_mut() = self.accessibility.clone();

        let a11y_role = match self.mode {
            InputMode::Password(_) => AccessibilityRole::PasswordInput,
            _ => AccessibilityRole::TextInput,
        };

        let a11y_value: String = match self.mode.mask_char() {
            Some(ch) if !self.value.is_empty() => {
                ch.to_string().repeat(self.value.chars().count().min(8))
            }
            _ => self.value.clone(),
        };

        let mut rect_el = rect_el
            .a11y_id(a11y_id)
            .a11y_enabled(!effectively_disabled)
            .a11y_focusable(!effectively_disabled)
            .a11y_auto_focus(self.auto_focus)
            .a11y_role(a11y_role)
            .a11y_alt(a11y_value)
            .on_pointer_enter(on_pointer_enter)
            .on_pointer_leave(on_pointer_leave);

        if !effectively_disabled {
            rect_el = rect_el
                .on_key_down(on_key_down)
                .on_ime_preedit(on_ime_preedit)
                .on_pointer_down(on_pointer_down)
                .on_global_pointer_up(on_global_pointer_up)
                .on_global_pointer_move(on_global_pointer_move)
                .on_focus_press(on_rect_focus_press);
        }

        rect_el.child(scroll_view)
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
