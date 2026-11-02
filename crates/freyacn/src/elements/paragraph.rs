use crate::styling::apply::apply_text_style;
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledExt};
use freya::prelude::*;
use freya::prelude::{Paragraph as ParagraphPrimitive, paragraph as paragraph_primitive};
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Component, EventHandlers, try_consume_context, use_a11y, use_focus, use_provide_context,
    use_state,
};
use std::borrow::Cow;
use torin::gaps::Gaps;

#[derive(PartialEq, Clone)]
pub struct Paragraph {
    content: Vec<ParagraphContentItem>,
    style: Style,
    event_handlers: EventHandlers,
    accessibility: AccessibilityData,
    key: DiffKey,

    disabled: bool,
    focusable: bool,
    use_defaults: bool,

    max_lines: Option<usize>,
    line_height: Option<f32>,

    semantic_events: SemanticEvents,
}

#[derive(PartialEq, Clone)]
enum ParagraphContentItem {
    Span(Span<'static>),
    Child(Element),
}

/// Shorthand constructor: `p("Hello world")`
pub fn p(content: impl Into<Cow<'static, str>>) -> Paragraph {
    Paragraph::new(content)
}

impl Paragraph {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        let mut this = Self::empty();
        this.content
            .push(ParagraphContentItem::Span(Span::new(content.into())));
        this
    }

    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            accessibility: AccessibilityData::default(),
            key: DiffKey::None,

            disabled: false,
            focusable: false,
            use_defaults: true,

            max_lines: None,
            line_height: None,

            semantic_events: SemanticEvents::default(),
        }
    }

    // ── content ────────────────────────────────────────────────────

    pub fn span(mut self, span: impl Into<Span<'static>>) -> Self {
        self.content.push(ParagraphContentItem::Span(span.into()));
        self
    }

    pub fn spans_iter(mut self, spans: impl Iterator<Item = Span<'static>>) -> Self {
        for span in spans {
            self.content.push(ParagraphContentItem::Span(span));
        }
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.content
            .push(ParagraphContentItem::Child(child.into_element()));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        for child in children {
            self.content
                .push(ParagraphContentItem::Child(child.into_element()));
        }
        self
    }

    pub fn maybe_child(mut self, child: Option<impl IntoElement>) -> Self {
        if let Some(child) = child {
            self.content
                .push(ParagraphContentItem::Child(child.into_element()));
        }
        self
    }

    // ── paragraph-specific props ───────────────────────────────────

    pub fn max_lines(mut self, n: impl Into<Option<usize>>) -> Self {
        self.max_lines = n.into();
        self
    }

    pub fn line_height(mut self, h: impl Into<Option<f32>>) -> Self {
        self.line_height = h.into();
        self
    }

    // ── state ──────────────────────────────────────────────────────

    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn with_focusable(mut self, v: bool) -> Self {
        self.focusable = v;
        self
    }

    pub fn no_default_style(mut self) -> Self {
        self.use_defaults = false;
        self
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::empty()
    }
}

impl StyledExt for Paragraph {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Paragraph {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Paragraph {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Paragraph {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl HasSemanticEvents for Paragraph {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

fn paragraph_ua_style() -> Style {
    let mut style = Style::bare();
    style.margin = Gaps::new(16.0, 0.0, 16.0, 0.0);
    style
}

impl Component for Paragraph {
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
            let ua = paragraph_ua_style();
            inherited.merge_style(&ua).merge_style(&self.style)
        } else {
            inherited.merge_style(&self.style)
        };

        // ── 2. Live variants ────────────────────────────────────────
        let variants = &self.style.variants;
        let needs_hover_active_tracking =
            variants.has(StyleVariant::Hover) || variants.has(StyleVariant::Active);
        let wants_focus = self.focusable
            || variants.has(StyleVariant::Focus)
            || variants.has(StyleVariant::FocusVisible)
            || variants.has(StyleVariant::FocusWithin)
            || self.semantic_events.has_focus_callbacks();

        let focus_state = focus();
        let focused_kb = focus_state == Focus::Keyboard;
        let focused = matches!(focus_state, Focus::Keyboard | Focus::Pointer);

        let mut live = Vec::with_capacity(6);
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

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events ─────────────────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Provide to descendants ───────────────────────────────
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 5. Build the primitive from ordered content ─────────────
        let mut el = paragraph_primitive();
        for item in &self.content {
            match item {
                ParagraphContentItem::Span(span) => el = el.span(span.clone()),
                ParagraphContentItem::Child(child) => el = el.child(child.clone()),
            }
        }
        if let Some(n) = self.max_lines {
            el = el.max_lines(n);
        }

        // ── 6. Apply style + events + a11y ──────────────────────────
        let mut el = apply_style(el, &style);

        if let Some(h) = self.line_height {
            el = el.line_height(h);
        }

        *el.event_handlers_mut() = self.event_handlers.clone();

        // Preserve the primitive's `role = Paragraph` unless the user
        // set accessibility data explicitly.
        if self.accessibility != AccessibilityData::default() {
            *el.accessibility_data_mut() = self.accessibility.clone();
        }

        if wants_focus && !self.disabled {
            el = el.a11y_id(a11y_id).a11y_focusable(true);
        }

        if !self.disabled {
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

/// Copies a resolved `Style` onto a `paragraph()` primitive.
fn apply_style(el: ParagraphPrimitive, s: &Style) -> ParagraphPrimitive {
    let el = el
        // box model
        .width(s.width.clone())
        .height(s.height.clone())
        .min_width(s.min_width.clone())
        .min_height(s.min_height.clone())
        .max_width(s.max_width.clone())
        .max_height(s.max_height.clone())
        .padding(s.padding)
        .margin(s.margin)
        .position(s.position.clone())
        // effects
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotation(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        .layer(s.layer);

    // text style — shared with text.rs
    let mut el = apply_text_style(el, s);

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
