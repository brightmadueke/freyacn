//! `Icon` — an SVG element built on [`SvgViewer`] that participates in the
//! same style / variant / inheritance system as `Div`, `Text`, `Image`, …
//!
//! ## Shadows
//!
//! `SvgViewer` (Freya `0.5.0-rc.13`) doesn't expose `.shadow(..)`. Both
//! `.shadow_*()` (box) and `.text_shadow_*()` (glyph) are silently
//! dropped. Wrap in a `div().shadow_md()` for a shadowed icon.
//!
//! ## Has: signaling
//!
//! Every `Icon` signals `HasKind::Svg` to its nearest
//! [`HasContext`](crate::styling::has::HasContext), so an ancestor with
//! `has-[svg]:` styles fires when this icon is present. Because of the
//! one-frame delay described in `styling::has`, the ancestor updates on
//! the *second* render after this icon mounts.

use crate::styling::has::{HasKind, use_has_signal, use_has_signal_when};
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
/// - **Size**: `16px × 16px`. Override with `.size(n)`.
/// - **Colour**: inherits from the nearest ancestor `InheritedStyle`
///   via `.color(text_color)`. Override with `.color(..)` / `.fill(..)` /
///   `.stroke(..)`.
/// - **Stroke width**: whatever the SVG specifies. Override with
///   `.stroke_width(w)`.
///
/// ## Composition
///
/// Renders a bare `SvgViewer`. Style inheritance, variant resolution,
/// focus tracking, and a11y flow through the same machinery as `Div`.
///
/// ## Style inheritance
///
/// `Style::text_color` maps to `SvgViewer::color()`, which is the SVG's
/// `currentColor` slot. Any ancestor providing an `InheritedStyle`
/// contributes its effective text colour.
///
/// ## Shadows
///
/// Neither shadow vector renders on an `Icon` — see the module doc.
/// Wrap in a `div()` for a shadow.
///
/// ## Has: signaling
///
/// Signals [`HasKind::Svg`] to the nearest ancestor `HasContext` so
/// `has-[svg]:` fires on that ancestor. Signals [`HasKind::Disabled`]
/// when the icon is disabled. Both are subject to the one-frame delay
/// and persistence caveats of [`crate::styling::has`].
#[derive(PartialEq, Clone)]
pub struct Icon {
    source: ImageSource,
    style: Style,
    event_handlers: EventHandlers,
    children: Vec<Element>,
    key: DiffKey,
    accessibility: AccessibilityData,

    fill: Option<StyledColor>,
    stroke: Option<StyledColor>,
    stroke_width: Option<f32>,

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

    pub fn size(mut self, v: f32) -> Self {
        self.style.width = Size::px(v);
        self.style.height = Size::px(v);
        self
    }

    pub fn color(mut self, color: impl Into<StyledColor>) -> Self {
        let sc: StyledColor = color.into();
        self.style.text_color = sc.to_fill();
        self
    }

    pub fn fill(mut self, color: impl Into<StyledColor>) -> Self {
        self.fill = Some(color.into());
        self
    }

    pub fn stroke(mut self, color: impl Into<StyledColor>) -> Self {
        self.stroke = Some(color.into());
        self
    }

    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = Some(w);
        self
    }

    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self.semantic_events.aria_state.disabled = v;
        self
    }

    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self.semantic_events.aria_state.checked = v;
        self
    }

    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self.semantic_events.aria_state.selected = v;
        self
    }
}

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

impl Component for Icon {
    fn render(&self) -> impl IntoElement {
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        // Signal to the nearest `HasContext` that this is an SVG.
        // Fires `has-[svg]:` classes on the ancestor.
        use_has_signal(HasKind::Svg);

        // Signal disabled state too.
        let effectively_disabled = self.disabled || self.semantic_events.aria_state.disabled;
        use_has_signal_when(HasKind::Disabled, effectively_disabled);

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
        live.extend(self.semantic_events.aria_state.active_variants());
        live.extend(self.semantic_events.has_state.active_variants());

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events + inheritance ────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 4. Build the primitive ───────────────────────────────
        let mut el = SvgViewer::new(self.source.clone());

        let icon_color: Color = match &style.text_color {
            Fill::Color(c) => *c,
            _ => Color::from_argb(255, 0, 0, 0),
        };
        el = el.color(icon_color);

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

fn apply_style(el: SvgViewer, s: &Style) -> SvgViewer {
    let mut el = el
        .width(s.width.clone())
        .height(s.height.clone())
        .margin(s.margin)
        .position(s.position.clone())
        .direction(s.direction)
        .main_align(s.main_align.clone())
        .cross_align(s.cross_align.clone())
        .spacing(s.spacing.get())
        .offset_x(s.offset_x.get())
        .offset_y(s.offset_y.get())
        .content(s.content.clone())
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotation(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        .aspect_ratio(s.object_fit.clone())
        .image_cover(s.object_position.clone())
        .sampling_mode(s.sampling_mode.clone())
        .snap_to_grid(s.snap_to_grid)
        .blur(s.image_blur);

    if s.cursor != CursorIcon::default() {
        el = el.cursor(s.cursor);
    }
    if !s.cursor_visible {
        el = el.cursor_visible(false);
    }

    el
}
