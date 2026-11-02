use crate::styling::apply::apply_to_rect;
use crate::styling::semantic_events::{
    HasSemanticEvents, SemanticEvents, attach_pointer_handlers, use_semantic_focus_events,
};
use crate::styling::style_variants::StyleVariant;
use crate::styling::{InheritedStyle, Style, StyledExt};
use freya::elements::image::{Image as ImagePrimitive, ImageHandle, image as image_primitive};
use freya::prelude::*;
use freya_core::prelude::{try_consume_context, use_provide_context};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(PartialEq, Clone)]
pub enum Src {
    Handle(ImageHandle),
    Bytes(Bytes),
    Path(PathBuf),
    Url(String),
}

impl From<ImageHandle> for Src {
    fn from(v: ImageHandle) -> Self {
        Src::Handle(v)
    }
}
impl From<Bytes> for Src {
    fn from(v: Bytes) -> Self {
        Src::Bytes(v)
    }
}
impl From<PathBuf> for Src {
    fn from(v: PathBuf) -> Self {
        Src::Path(v)
    }
}
impl From<&str> for Src {
    fn from(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            Src::Url(s.to_owned())
        } else {
            Src::Path(PathBuf::from(s))
        }
    }
}
impl From<String> for Src {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

#[derive(PartialEq, Clone)]
pub struct Image {
    src: Src,
    alt: String,
    fallback: Option<Element>,
    style: Style,
    event_handlers: EventHandlers,
    children: Vec<Element>,
    key: DiffKey,
    accessibility: AccessibilityData,
    disabled: bool,
    checked: bool,
    selected: bool,
    semantic_events: SemanticEvents,
}

pub fn image(src: impl Into<Src>) -> Image {
    Image::new(src)
}

impl Image {
    pub fn new(src: impl Into<Src>) -> Self {
        Self {
            src: src.into(),
            alt: String::new(),
            fallback: None,
            style: Style::bare(),
            event_handlers: EventHandlers::default(),
            children: Vec::new(),
            key: DiffKey::None,
            accessibility: AccessibilityData::default(),
            disabled: false,
            checked: false,
            selected: false,
            semantic_events: SemanticEvents::default(),
        }
    }

    pub fn src(mut self, src: impl Into<Src>) -> Self {
        self.src = src.into();
        self
    }
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }
    pub fn fallback(mut self, el: impl IntoElement) -> Self {
        self.fallback = Some(el.into_element());
        self
    }
    pub fn with_disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn with_checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }
    pub fn with_selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
}

impl StyledExt for Image {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}
impl ChildrenExt for Image {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}
impl KeyExt for Image {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}
impl EventHandlersExt for Image {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
impl AccessibilityExt for Image {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}
impl HasSemanticEvents for Image {
    fn semantic_events_mut(&mut self) -> &mut SemanticEvents {
        &mut self.semantic_events
    }
}

// ─── Loading (unchanged) ────────────────────────────────────────────────

fn decode(bytes: Bytes) -> Option<ImageHandle> {
    use freya::engine::prelude::{SkData, SkImage};
    let data = SkData::new_copy(&bytes);
    let img = SkImage::from_encoded(data)?;
    Some(ImageHandle::new(img, bytes))
}

fn fetch_and_decode(url: &str) -> Option<ImageHandle> {
    let mut resp = ureq::get(url).call().ok()?;
    let buf = resp
        .body_mut()
        .with_config()
        .limit(MAX_IMAGE_BYTES)
        .read_to_vec()
        .ok()?;
    decode(Bytes::from(buf))
}

async fn fetch_off_thread(url: String) -> Option<ImageHandle> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch_and_decode(&url));
    });
    loop {
        match rx.try_recv() {
            Ok(result) => return result,
            Err(mpsc::TryRecvError::Disconnected) => return None,
            Err(mpsc::TryRecvError::Empty) => {
                timer(std::time::Duration::from_millis(30)).await;
            }
        }
    }
}

fn resolve(src: &Src) -> Option<ImageHandle> {
    match src {
        Src::Handle(h) => Some(h.clone()),
        Src::Bytes(b) => decode(b.clone()),
        Src::Path(p) => decode(Bytes::from(std::fs::read(p).ok()?)),
        Src::Url(_) => None,
    }
}

// ─── Component ──────────────────────────────────────────────────────────

impl Component for Image {
    fn render(&self) -> impl IntoElement {
        let hovering = use_state(|| false);
        let pressing = use_state(|| false);
        let a11y_id = use_a11y();
        let focus = use_focus(a11y_id);

        let mut remote = use_state(|| None::<ImageHandle>);

        use_hook({
            let url = match &self.src {
                Src::Url(u) => Some(u.clone()),
                _ => None,
            };
            move || {
                spawn(async move {
                    let Some(url) = url else { return };
                    if let Some(h) = fetch_off_thread(url).await {
                        remote.set(Some(h));
                    }
                })
            }
        });

        let cache = use_hook(|| Rc::new(RefCell::new(None::<(Src, Option<ImageHandle>)>)));
        let handle = match &self.src {
            Src::Url(_) => remote.read().clone(),
            _ => {
                let mut c = cache.borrow_mut();
                match &*c {
                    Some((s, h)) if *s == self.src => h.clone(),
                    _ => {
                        let h = resolve(&self.src);
                        *c = Some((self.src.clone(), h.clone()));
                        h
                    }
                }
            }
        };

        // ── 1. Base style ───────────────────────────────────────────────
        let base = match try_consume_context::<InheritedStyle>() {
            Some(InheritedStyle(parent)) => parent.inheritable_subset().merge_style(&self.style),
            None => Style::default().merge_style(&self.style),
        };

        // ── 2. Live variants ────────────────────────────────────────────
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

        let mut live = Vec::with_capacity(8);
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
        if self.checked {
            live.push(StyleVariant::Checked);
        }
        if self.selected {
            live.push(StyleVariant::Selected);
        }

        let style = variants.apply_all(&live, base);

        // ── 3. Focus events (both branches) ─────────────────────────────
        use_semantic_focus_events(&self.semantic_events, focus_state);

        // ── 4. Provide to descendants (both branches) ───────────────────
        use_provide_context(|| InheritedStyle(style.clone()));

        // ── 5a. Fallback branch ─────────────────────────────────────────
        let Some(handle) = handle else {
            let mut el = apply_to_rect(rect(), &style).maybe_child(self.fallback.clone());

            *el.event_handlers_mut() = self.event_handlers.clone();
            *el.accessibility_data_mut() = self.accessibility.clone();

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

            return el.into_element();
        };

        // ── 5b. Loaded branch ───────────────────────────────────────────
        let mut el = apply_style(image_primitive(handle), &style);

        *el.event_handlers_mut() = self.event_handlers.clone();
        *el.accessibility_data_mut() = self.accessibility.clone();

        if !self.alt.is_empty() {
            el = el.a11y_alt(self.alt.clone());
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

        el.children(self.children.clone()).into_element()
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

fn apply_style(el: ImagePrimitive, s: &Style) -> ImagePrimitive {
    let mut el = el
        .width(s.width.clone())
        .height(s.height.clone())
        .min_width(s.min_width.clone())
        .min_height(s.min_height.clone())
        .max_width(s.max_width.clone())
        .max_height(s.max_height.clone())
        .padding(s.padding)
        .margin(s.margin)
        .direction(s.direction)
        .main_align(s.main_align.clone())
        .cross_align(s.cross_align.clone())
        .position(s.position.clone())
        .content(s.content.clone())
        .spacing(s.spacing.get())
        .offset_x(s.offset_x.get())
        .offset_y(s.offset_y.get())
        .corner_radius(s.radius)
        .overflow(s.overflow)
        .opacity(s.opacity)
        .rotation(s.rotation)
        .scale(s.scale)
        .transform_origin(s.transform_origin.clone())
        .backdrop_blur(s.blur)
        .layer(s.layer)
        .aspect_ratio(s.object_fit.clone())
        .image_cover(s.object_position.clone())
        .sampling_mode(s.sampling_mode.clone())
        .snap_to_grid(s.snap_to_grid)
        .blur(s.image_blur);

    if let Some(border) = &s.border {
        el = el.border(border.clone());
    }

    if s.cursor != CursorIcon::default() {
        el = el.cursor(s.cursor);
    }
    if !s.cursor_visible {
        el = el.cursor_visible(false);
    }
    el
}
