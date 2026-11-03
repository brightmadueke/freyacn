use super::div::{Div, div};
use crate::styling::apply::apply_text_style;
use crate::styling::colors::gray_400;
use crate::styling::semantic_events::{HasSemanticEvents, SemanticEvents};
use crate::styling::{InheritedStyle, Style, StyledColor, StyledExt, black, white};
use freya::components::{ScrollConfig, ScrollController, ScrollView, use_scroll_controller};
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::elements::paragraph::ParagraphCursorExt;
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Component, EventHandlers, try_consume_context, use_side_effect, use_state,
};
use std::borrow::Cow;
use torin::gaps::Gaps;
use torin::prelude::Area;
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
/// * **Focusable** on mount (unless disabled), on Tab, and by clicking.
/// * **Selection** — click-drag to select, shift+arrow to extend, Ctrl+A
///   to select all. Password mode opts out because the mask length may
///   differ from the value length in UTF-16.
/// * **Horizontal scrolling** — long values scroll so the cursor is
///   always visible.
/// * **Clipboard** — Ctrl+C copies the selection (or whole value), Ctrl+X
///   cuts, Ctrl+V pastes at the cursor. Respects `.max_length(n)`.
/// * **IME** — preedit composes inline with an underline, honoring mask.
/// * Enter fires `.on_submit(handler)`.
///
/// ## Example
///
/// ```
/// let value = use_state(String::new);
/// input(value.read().clone())
///     .placeholder("Type here")
///     .on_change(move |text| value.set(text))
/// ```
///
/// ## Accessibility
///
/// Role is `TextInput`, or `PasswordInput` when `.mode(m)` is a
/// `Password`. The a11y value is either the real text (text mode) or a
/// fixed-length mask (password mode) — the raw password never flows
/// through `a11y_alt`.
///
/// ## Aria state
///
/// `with_disabled` and `with_read_only` mirror their native flags into
/// the component's `AriaState`, so `aria-disabled:*` and
/// `aria-readonly:*` class strings fire alongside their native
/// counterparts from a single setter:
///
/// ```
/// input(value)
///     .with_disabled(true)
///     .style("disabled:opacity-50 aria-disabled:cursor-not-allowed")
/// ```
///
/// Additional aria states with no native equivalent are set through the
/// [`HasSemanticEvents`] builders:
///
/// ```
/// input(value)
///     .with_aria_invalid(true)
///     .with_aria_required(true)
///     .style("
///         aria-invalid:border-red-500
///         aria-required:border-amber-500
///     ")
/// ```
///
/// Calling `.with_aria_disabled(true)` / `.with_aria_readonly(true)`
/// directly also disables the input's editing logic — the render body
/// unions the aria flags into `effectively_disabled` / `effective_read_only`.
///
/// ## Known limitations
///
/// * `e.modifiers.ctrl()` / `.shift()` — if `Modifiers` exposes these as
///   public fields instead of methods, swap the calls. One-character fix.
/// * Password mode disables selection because mask ↔ value misalignment
///   on emoji would land highlights at the wrong offsets.
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

    pub fn max_length(mut self, n: impl Into<Option<usize>>) -> Self {
        self.max_length = n.into();
        self
    }

    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// Disable the input.
    ///
    /// Sets the native `disabled` flag **and** the `aria-disabled` state.
    /// Both `disabled:*` and `aria-disabled:*` class strings fire, and
    /// the input is removed from the focus order and stops accepting
    /// keyboard input.
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    /// Make the input read-only.
    ///
    /// Sets the native `read_only` flag **and** the `aria-readonly` state.
    /// Both `readonly:*` (via `Style::read_only` — not used here, so the
    /// native read-only gate suppresses editing directly) and
    /// `aria-readonly:*` class strings fire, and the keyboard handler
    /// short-circuits.
    pub fn with_read_only(mut self, v: bool) -> Self {
        self.read_only = v;
        self.semantic_events.aria_state.readonly = v;
        self
    }

    // ── events ─────────────────────────────────────────────────────

    pub fn on_change(mut self, cb: impl Into<EventHandler<String>>) -> Self {
        self.on_change = Some(cb.into());
        self
    }

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

/// HTML `<input type="text">` UA defaults. No phantom `:focus` — focus
/// comes from `Div::with_focusable`.
fn input_ua_style(effectively_disabled: bool) -> Style {
    let mut style = Style::bare();
    style.width = Size::px(DEFAULT_WIDTH);
    style.background = white().into();
    style.padding = Gaps::new(6.0, 8.0, 6.0, 8.0);
    style.radius = CornerRadius::new_all(4.0);
    style.overflow = Overflow::Clip;
    style.cursor = if effectively_disabled {
        CursorIcon::NotAllowed
    } else {
        CursorIcon::Text
    };
    style = style.disabled(|mut s| {
        s.opacity = 0.5;
        s
    });
    style
}

/// Convert a UTF-16 index (paragraph's cursor space) to a byte offset.
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

/// If the cursor is outside the visible viewport, scroll so it comes
/// back into view. Ported from the reference `Input::follow_cursor`
/// implementation.
///
/// `ScrollController` is `Copy` and stores its position in internal
/// `State`, so `scroll_to_x` on a copy mutates the shared state —
/// `follow_cursor` can take it by value and be called from an `Fn`
/// closure.
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

    let holder_ref = holder.peek();
    let holder_borrow = holder_ref.0.borrow();
    let Some(inner) = holder_borrow.as_ref() else {
        return;
    };

    let cursor_rect = inner.paragraph.cursor_rect(text, cursor_index, text_align);
    let scale = inner.scale_factor as f32;
    let cursor_x = cursor_rect.left / scale;
    let visible_cursor_x = cursor_x + padding.left();

    // All f32 — `Area` fields and skia rect fields are both f32.
    let visible_start_x = viewport_area.min_x() - para_area.read().min_x();
    let visible_end_x = visible_start_x + viewport_area.width();

    if visible_cursor_x < visible_start_x {
        scroll.scroll_to_x(-cursor_x as i32);
    } else if visible_cursor_x > visible_end_x {
        scroll.scroll_to_x(-(visible_cursor_x + padding.right() - viewport_area.width()) as i32);
    }
}

impl Component for Input {
    fn render(&self) -> impl IntoElement {
        let mut cursor = use_state(|| self.value.encode_utf16().count());
        let mut anchor = use_state(|| None::<usize>);
        let mut is_dragging = use_state(|| false);
        let holder = use_state(ParagraphHolder::default);
        let mut preedit = use_state(|| None::<String>);
        let scroll_controller = use_scroll_controller(ScrollConfig::default);
        let para_area = use_state(Area::default);
        let viewport_area = use_state(Area::default);

        // Native `disabled` / `read_only` and their aria counterparts are
        // treated uniformly: any of them being set engages the
        // corresponding gate. `with_disabled` / `with_read_only` sync the
        // pair, but a caller using `.with_aria_disabled(true)` /
        // `.with_aria_readonly(true)` directly bypasses that sync, so
        // this union keeps behaviour consistent.
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;
        let effective_read_only =
            self.read_only || self.semantic_events.aria_state.readonly || effectively_disabled;

        // ── 1. Base style ────────────────────────────────────────
        let inherited = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset(),
            None => Style::default(),
        };
        let ua = input_ua_style(effectively_disabled);
        let raw_style = inherited.merge_style(&ua).merge_style(&self.style);

        // ── 2. Visible text + cursor + selection ────────────────
        let display_placeholder =
            self.value.is_empty() && self.placeholder.is_some() && preedit.read().is_none();

        let committed_utf16 = self.value.encode_utf16().count();
        let cursor_utf16 = (*cursor.read()).min(committed_utf16);
        let cursor_byte = utf16_to_byte(&self.value, cursor_utf16);

        let selection_enabled = matches!(self.mode, InputMode::Text);
        let selection_range: Option<(usize, usize)> = if selection_enabled {
            match (*anchor.read(), *cursor.read()) {
                (Some(a), c) if a != c => Some((a.min(c), a.max(c))),
                _ => None,
            }
        } else {
            None
        };

        let (before_value, after_value) = self.value.split_at(cursor_byte);
        let (before_display, after_display) = match self.mode {
            InputMode::Password(ch) => (
                ch.to_string().repeat(before_value.encode_utf16().count()),
                ch.to_string().repeat(after_value.encode_utf16().count()),
            ),
            InputMode::Text => (before_value.to_string(), after_value.to_string()),
        };

        let preedit_masked: Option<String> = preedit.read().as_ref().map(|p| match self.mode {
            InputMode::Password(ch) => ch.to_string().repeat(p.chars().count()),
            InputMode::Text => p.clone(),
        });

        let preedit_utf16_len = preedit_masked
            .as_ref()
            .map(|p| p.encode_utf16().count())
            .unwrap_or(0);

        let display_cursor = if effectively_disabled || display_placeholder {
            None
        } else if preedit_masked.is_some() {
            Some(cursor_utf16 + preedit_utf16_len)
        } else {
            Some(cursor_utf16)
        };

        // ── 3. Key handling ──────────────────────────────────────
        let read_only = effective_read_only;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let value_for_keys = self.value.clone();
        let max_len = self.max_length;
        let password_mode = matches!(self.mode, InputMode::Password(_));
        let sel_enabled = selection_enabled;

        let on_key_down = move |e: Event<KeyboardEventData>| {
            if read_only {
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
                // ── Clipboard ────────────────────────────────────
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
                }
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
                        let new_cur = delete_sel(&mut text, sel);
                        let mut inserted = pasted;
                        if let Some(max) = max_len {
                            let room = max.saturating_sub(text.chars().count());
                            if room == 0 {
                                return;
                            }
                            inserted = inserted.chars().take(room).collect();
                        }
                        let insert_byte = utf16_to_byte(&text, new_cur);
                        text.insert_str(insert_byte, &inserted);
                        cursor.set(new_cur + inserted.encode_utf16().count());
                        anchor.set(None);
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                Key::Character(c)
                    if e.modifiers.ctrl() && c.eq_ignore_ascii_case("a") && sel_enabled =>
                {
                    anchor.set(Some(0));
                    cursor.set(text.encode_utf16().count());
                }
                // ── Character input ──────────────────────────────
                Key::Character(c) if !c.is_empty() => {
                    if c.chars().any(|ch| !ch.is_control()) {
                        let new_cur = delete_sel(&mut text, sel);
                        if let Some(max) = max_len {
                            if text.chars().count() + c.chars().count() > max {
                                return;
                            }
                        }
                        let insert_byte = utf16_to_byte(&text, new_cur);
                        text.insert_str(insert_byte, c);
                        cursor.set(new_cur + c.encode_utf16().count());
                        anchor.set(None);
                        if let Some(cb) = &on_change {
                            cb.call(text);
                        }
                    }
                }
                // ── Editing ──────────────────────────────────────
                Key::Named(NamedKey::Backspace) => {
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
                // ── Navigation ───────────────────────────────────
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
                }
                Key::Named(NamedKey::Enter) => {
                    if let Some(cb) = &on_submit {
                        cb.call(text);
                    }
                }
                _ => {}
            }
        };

        // ── 4. Click / drag to position cursor + select ─────────
        let holder_for_press = holder.clone();
        let on_focus_press = move |e: Event<FocusPressEventData>| {
            if read_only {
                return;
            }
            let holder_ref = holder_for_press.peek();
            let holder_borrow = holder_ref.0.borrow();
            if let Some(inner) = holder_borrow.as_ref() {
                let idx = inner
                    .paragraph
                    .cursor_index_at_point(e.element_location().to_tuple());
                anchor.set(Some(idx));
                cursor.set(idx);
                is_dragging.set(true);
            }
        };

        let holder_for_move = holder.clone();
        let para_area_for_move = para_area.clone();
        let on_global_pointer_move = move |e: Event<PointerEventData>| {
            if !*is_dragging.read() || read_only {
                return;
            }
            let holder_ref = holder_for_move.peek();
            let holder_borrow = holder_ref.0.borrow();
            if let Some(inner) = holder_borrow.as_ref() {
                let area = *para_area_for_move.read();
                let loc = e.global_location().to_tuple();
                // `loc` is (f64, f64); `Area::min_x/y` are f32 — cast.
                let relative = (loc.0 - area.min_x() as f64, loc.1 - area.min_y() as f64);
                let idx = inner.paragraph.cursor_index_at_point(relative);
                cursor.set(idx);
            }
        };

        let on_global_pointer_up = move |_: Event<PointerEventData>| {
            is_dragging.set_if_modified(false);
        };

        // ── 5. IME preedit ───────────────────────────────────────
        let on_ime_preedit = move |e: Event<ImePreeditEventData>| {
            let text = e.data().text.clone();
            preedit.set(if text.is_empty() { None } else { Some(text) });
        };

        // ── 6. Cursor color ──────────────────────────────────────
        let cursor_color: Color = StyledColor::from(raw_style.text_color.clone())
            .to_color()
            .unwrap_or(black().into());

        // ── 7. Paragraph child ───────────────────────────────────
        let mut para = paragraph()
            .holder(holder.read().clone())
            .min_width(Size::percent(100.0))
            .max_lines(1)
            .cursor_color(cursor_color)
            .on_sized({
                let mut para_area = para_area.clone();
                move |e: Event<SizedEventData>| {
                    para_area.set_if_modified(e.area);
                }
            });

        para = apply_text_style(para, &raw_style);

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

        if !read_only {
            para = para.on_focus_press(on_focus_press);
        }

        // ── 8. Wrap in a horizontal ScrollView ───────────────────
        let visible_text_for_scroll: String = match self.mode {
            InputMode::Password(ch) => ch.to_string().repeat(self.value.encode_utf16().count()),
            InputMode::Text => self.value.clone(),
        };
        let cursor_for_scroll = cursor_utf16;
        let padding_for_scroll = raw_style.padding;
        let text_align_for_scroll = raw_style.text_align;
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

        // ── 9. Outer Div — owns focus, hover, semantic events ────
        // `Div` receives the semantic events (including `aria_state`) so
        // it can push the active `StyleVariant::Aria*` entries into its
        // live-variant set and fire `aria-disabled:*`,
        // `aria-readonly:*`, `aria-invalid:*`, `aria-required:*`, and the
        // other aria conditions in `raw_style.variants`.
        let mut div_el: Div = div()
            .with_disabled(effectively_disabled)
            .with_semantic_events(self.semantic_events.clone())
            .with_focusable(!effectively_disabled);

        *div_el.get_style() = raw_style;
        *div_el.event_handlers_mut() = self.event_handlers.clone();
        *div_el.accessibility_data_mut() = self.accessibility.clone();

        let a11y_role = match self.mode {
            InputMode::Password(_) => AccessibilityRole::PasswordInput,
            InputMode::Text => AccessibilityRole::TextInput,
        };

        let a11y_value: String = if password_mode {
            if self.value.is_empty() {
                String::new()
            } else {
                match self.mode {
                    InputMode::Password(ch) => {
                        ch.to_string().repeat(self.value.chars().count().min(8))
                    }
                    InputMode::Text => String::new(),
                }
            }
        } else {
            self.value.clone()
        };

        div_el = div_el
            .a11y_enabled(!effectively_disabled)
            .a11y_auto_focus(self.auto_focus)
            .a11y_role(a11y_role)
            .a11y_alt(a11y_value);

        if !read_only {
            div_el = div_el.on_key_down(on_key_down);
        }
        div_el = div_el.on_ime_preedit(on_ime_preedit);

        if selection_enabled && !read_only {
            div_el = div_el
                .on_global_pointer_move(on_global_pointer_move)
                .on_global_pointer_up(on_global_pointer_up);
        }

        div_el = div_el.child(scroll_view);

        div_el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
