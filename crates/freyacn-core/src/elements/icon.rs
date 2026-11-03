//! `Icon` — an SVG element built on [`SvgViewer`] that participates in the
//! same style / variant / inheritance system as `Div`, `Text`, `Image`, …
//!
//! Lucide icons (and most modern icon sets) use `stroke="currentColor"`,
//! so the SVG's paint is fully driven by [`SvgViewer::color`]. `Icon`
//! wires `Style::text_color` into that slot, so an icon inherits its
//! colour from the nearest ancestor that provided an [`InheritedStyle`] —
//! typically the `Button` / `Div` / `Text` wrapping it.
//!
//! ```ignore
//! Button::new()
//!     .child(icon(lucide::heart()))   // inherits Button's text colour
//! ```

use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style::Style;
use crate::styling::style_variants::StyleVariant;
use crate::styling::styled::StyledExt;
use crate::styling::{InheritedStyle, StyledColor};
use freya::components::SvgViewer;
use freya::prelude::*;
use freya_core::diff_key::DiffKey;
use freya_core::element::{Element, IntoElement};
use freya_core::integration::{
    AccessibilityData, AccessibilityExt, ChildrenExt, ComponentKey, EventHandlersExt, KeyExt,
};
use freya_core::prelude::{
    Color, Component, CursorIcon, EventHandlers, Fill, try_consume_context, use_a11y, use_focus,
    use_provide_context, use_state,
};
use torin::prelude::Size;

/// An SVG icon.
///
/// ## Defaults
///
/// - **Size**: `16px × 16px` (matching shadcn's `size-4`). Override with
///   `.size(n)` / `.w(n).h(n)`.
/// - **Colour**: inherits from the nearest ancestor `InheritedStyle`
///   via `.color(text_color)`. Override with `.color(..)`,
///   `.fill(..)`, or `.stroke(..)`.
/// - **Stroke width**: whatever the SVG specifies (lucide default 2.0).
///   Override with `.stroke_width(w)`.
///
/// ## Composition
///
/// `Icon` renders a bare `SvgViewer` — no wrapper `Div`. Style
/// inheritance, variant resolution, focus / pointer tracking, and a11y
/// all flow through the same machinery as `Div`.
///
/// ## Style inheritance
///
/// `Style::text_color` maps to `SvgViewer::color()`, which is the SVG's
/// `currentColor` slot. Any ancestor that provides an `InheritedStyle`
/// (every `Div`, `Button`, `Text`, `Paragraph`, …) contributes its
/// effective text colour, so an icon inside a shadcn `Button` with
/// `text-primary-foreground` takes that exact colour with zero
/// per-icon code.
#[derive(PartialEq, Clone)]
pub struct Icon {
    source: ImageSource,
    style: Style,
    event_handlers: EventHandlers,
    children: Vec<Element>,
    key: DiffKey,
    accessibility: AccessibilityData,

    // Raw paint overrides. `None` = inherit from `Style`.
    fill: Option<StyledColor>,
    stroke: Option<StyledColor>,
    stroke_width: Option<f32>,

    // State flags — drive `disabled:*` / `checked:*` / `selected:*`
    // and the `aria-*` variant families.
    disabled: bool,
    checked: bool,
    selected: bool,

    semantic_events: SemanticEvents,
}

/// Shorthand constructor: `icon(lucide::heart())`
pub fn icon(source: impl Into<ImageSource>) -> Icon {
    Icon::new(source)
}

impl Icon {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        // Default size 16×16 (shadcn `size-4`). `Style::bare()` is the
        // "no theme" base, matching how `Div` and `Image` start.
        let mut style = Style::bare();
        style.width = Size::px(16.0);
        style.height = Size::px(16.0);

        Self {
            source: source.into(),
            style,
            event_handlers: EventHandlers::default(),
            children: Vec::new(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),

            fill: None,
            stroke: None,
            stroke_width: None,

            disabled: false,
            checked: false,
            selected: false,

            semantic_events: SemanticEvents::default(),
        }
    }

    // ── size ───────────────────────────────────────────────────────

    /// Square size in pixels. Equivalent to `.w(n).h(n)`.
    pub fn size(mut self, v: f32) -> Self {
        self.style.width = Size::px(v);
        self.style.height = Size::px(v);
        self
    }

    // ── paint ──────────────────────────────────────────────────────

    /// Override the SVG's `currentColor` slot.
    ///
    /// By default `Icon` fills this from `Style::text_color` (the
    /// inherited text colour), so most callers don't need this — use it
    /// when the icon should diverge from the surrounding text.
    pub fn color(mut self, color: impl Into<StyledColor>) -> Self {
        self.style.text_color = color.into().into();
        self
    }

    /// Override the SVG shapes' `fill` attribute. Rare for line icons.
    pub fn fill(mut self, color: impl Into<StyledColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    /// Override the SVG shapes' `stroke` attribute. Bypasses
    /// `currentColor` — useful for two-tone icon sets.
    pub fn stroke(mut self, color: impl Into<StyledColor>) -> Self {
        self.stroke = Some(color.into());
        self
    }

    /// Override the SVG's stroke width.
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = Some(w);
        self
    }

    // ── state ──────────────────────────────────────────────────────

    /// Disable the icon.
    ///
    /// Sets the native flag **and** the `aria-disabled` state, so both
    /// `disabled:*` and `aria-disabled:*` class families fire from a
    /// single call.
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    /// Controlled checked state.
    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self.semantic_events.aria_state.checked = v;
        self
    }

    /// Controlled selected state.
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self.semantic_events.aria_state.selected = v;
        self
    }
}

// ── Trait impls — mirrors `Div`, `Button`, `Image` ─────────────────

impl StyledExt for Icon {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl KeyExt for Icon {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl EventHandlersExt for Icon {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl AccessibilityExt for Icon {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl ChildrenExt for Icon {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl HasSemanticEvents for Icon {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

// ── Component ──────────────────────────────────────────────────────

impl Component for Icon {
    fn render(&self) -> impl IntoElement {
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        // `disabled` and `aria-disabled` are treated uniformly — either
        // one disables the icon for pointer tracking, focusability, and
        // the `disabled:*` / `aria-disabled:*` class families.
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;

        // ── 1. Base style ────────────────────────────────────────
        let base = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset().merge_style(&self.style),
            None => Style::default().merge_style(&self.style),
        };

        // ── 2. Live variants ─────────────────────────────────────
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

        let mut live = Vec::with_capacity(10);
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
        live.extend(self.semantic_events.aria_state.active_variants());

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events + inheritance ────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 4. Build the primitive ───────────────────────────────
        let mut el = SvgViewer::new(self.source.clone());

        // currentColor ← Style::text_color. Lucide / heroicons use
        // `stroke="currentColor"`, so this is what makes an icon inside
        // a Button inherit the button's text colour.
        let icon_color: Color = match &style.text_color {
            Fill::Color(c) => *c,
            // Gradient text colour has no single solid representative —
            // fall back to black. Callers with gradient icons should use
            // `.fill(..)` / `.stroke(..)` directly.
            _ => Color::from_argb(255, 0, 0, 0),
        };
        el = el.color(icon_color);

        // Explicit paint overrides.
        if let Some(f) = &self.fill {
            if let Some(c) = f.to_color() {
                el = el.fill(c);
            }
        }
        if let Some(s) = &self.stroke {
            if let Some(c) = s.to_color() {
                el = el.stroke(c);
            }
        }
        if let Some(w) = self.stroke_width {
            el = el.stroke_width(w);
        }

        // ── 5. Style → primitive ─────────────────────────────────
        el = apply_style(el, &style);

        // ── 6. A11y + pointer tracking ───────────────────────────
        let can_focus = !effectively_disabled && wants_focus;

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        el = el.a11y_id(a11y_id);
        if can_focus {
            el = el.a11y_focusable(true);
        }

        if !effectively_disabled {
            let focus_on_press = if can_focus { Some(a11y_id) } else { None };
            el = attach_pointer_handlers(
                el,
                &self.semantic_events,
                hovering,
                pressing,
                needs_hover_active_tracking,
                focus_on_press,
            );
        }

        if !self.children.is_empty() {
            el = el.children(self.children.clone());
        }

        el
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

/// Map a resolved [`Style`] onto a [`SvgViewer`] primitive.
///
/// Only the `SvgViewer` methods that exist on the current `freya` version
/// are used: `ContainerSizeExt` (width / height), `ContainerPositionExt`
/// (margin / position), `ContainerWithContentExt` (flow), `EffectExt`
/// (effects + cursor), `ImageExt` (image). Padding / min / max are not
/// reachable from `SvgViewer` and are skipped — icons are fixed-size,
/// wrap in a `div` if you need box-model overrides.
fn apply_style(el: SvgViewer, s: &Style) -> SvgViewer {
    let mut el = el
        // size
        .width(s.width.clone())
        .height(s.height.clone())
        // position
        .margin(s.margin)
        .position(s.position.clone())
        // flow
        .direction(s.direction)
        .main_align(s.main_align.clone())
        .cross_align(s.cross_align.clone())
        .spacing(s.spacing.get())
        .offset_x(s.offset_x.get())
        .offset_y(s.offset_y.get())
        .content(s.content.clone())
        // effects
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotation(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        // image
        .aspect_ratio(s.object_fit.clone())
        .image_cover(s.object_position.clone())
        .sampling_mode(s.sampling_mode.clone())
        .snap_to_grid(s.snap_to_grid)
        .blur(s.image_blur);

    // Cursor — `CursorIcon::default()` means "don't set", matching how
    // `Div` treats it in `styled.rs`.
    if s.cursor != CursorIcon::default() {
        el = el.cursor(s.cursor);
    }
    if !s.cursor_visible {
        el = el.cursor_visible(false);
    }

    el
}
