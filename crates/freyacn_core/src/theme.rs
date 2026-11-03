//! JSON-driven theming for FreyaCN, modeled on gpui-kit semantics.
//!
//! # Overview
//!
//! `Theme` is a resolved, typed view over a `.json` theme file. Loading
//! involves four steps, all handled by [`Theme::try_from_json_str`]:
//!
//! 1. Parse `ThemeConfig` (serde).
//! 2. Resolve `palette` overrides into concrete `Color`s.
//! 3. Resolve `semantic` + `colors` into `Fill`s, with iteration so
//!    tokens can reference each other.
//! 4. Build `list` / `menu` / `table` themes from their structured
//!    configs, then overlay any matching entries from `colors`.
//!
//! Any unresolvable reference **panics** at load time — the theme system
//! is meant to fail fast so typos don't leak into the UI as silent black
//! rectangles. Use the `try_*` variants for graceful error handling.
//!
//! # Quick start
//!
//! ```no_run
//! use freya_cn::theme::load_theme_registry;
//!
//! fn app() {
//!     let handle = load_theme_registry(include_str!("../themes/dark.json"));
//!     // ... build UI; components read the active theme automatically ...
//!     handle.set_active("dark").unwrap();
//! }
//! ```
//!
//! For a non-Freya context (tests, CLI tools) use [`Theme::try_from_json_str`]:
//!
//! ```no_run
//! use freya_cn::theme::Theme;
//!
//! let theme = Theme::try_from_json_str(include_str!("../themes/light.json"))
//!     .expect("theme should be valid");
//!
//! let bg = theme.background_fill();
//! let fg = theme.foreground();
//! ```
//!
//! # Discovering loaded themes
//!
//! Theme pickers, settings panels, and CLI tooling all need to enumerate
//! what's loaded without paying for a full `Arc<Theme>` clone per entry.
//! Use [`ThemeInfo`] for that.
//!
//! ## From a Freya component
//!
//! ```no_run
//! # use freya::prelude::*;
//! # use freya_cn::theme::{use_theme_handle, Appearance};
//! #[component]
//! fn ThemePicker() -> Element {
//!     let handle = use_theme_handle();
//!
//!     // Subscribing read: re-renders when the theme changes.
//!     let active = handle.active_info();
//!
//!     // Non-subscribing list, sorted by name — perfect for a picker.
//!     let infos = handle.theme_infos();
//!
//!     rsx! {
//!         rect {
//!             for info in infos {
//!                 {
//!                     let is_current = info.id == active.id;
//!                     let id = info.id.clone();
//!                     let handle = handle.clone();
//!                     rsx! {
//!                         rect {
//!                             key: "{id}",
//!                             onclick: move |_| { let _ = handle.set_active(&id); },
//!                             label {
//!                                 if is_current { "✓ {info.name}" } else { "  {info.name}" }
//!                             }
//!                         }
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Outside Freya
//!
//! Every `ThemeHandle` method also exists on [`ThemeRegistry`] — use it
//! directly in tests, CLI tools, or app init code that runs before Freya
//! mounts:
//!
//! ```no_run
//! use freya_cn::theme::ThemeRegistry;
//!
//! let registry = ThemeRegistry::new();
//! registry.load(include_str!("themes/light.json"));
//! registry.load(include_str!("themes/dark.json"));
//!
//! // Sorted, cheap metadata.
//! for info in registry.info_list() {
//!     println!("{} ({}) — {:?}", info.name, info.id, info.appearance);
//! }
//!
//! // Count, membership, active.
//! println!("{} themes loaded", registry.len());
//! if registry.contains("default-dark") {
//!     registry.set_active("default-dark").unwrap();
//! }
//! if let Some(info) = registry.active_info() {
//!     println!("active: {}", info.name);
//! }
//! ```
//!
//! ## Method map
//!
//! | Need                              | `ThemeRegistry`          | `ThemeHandle`            |
//! |-----------------------------------|--------------------------|--------------------------|
//! | Full themes (resolved tokens)     | `list()` / `get(id)`     | `themes()`               |
//! | Cheap metadata (one)              | `theme_info(id)`         | `theme_info(id)`         |
//! | Cheap metadata (all, sorted)      | `info_list()`            | `theme_infos()`          |
//! | Currently-active metadata         | `active_info()`          | `active_info()`          |
//! | Currently-active theme            | `active()`               | `current()`              |
//! | Currently-active ID               | `active_id()`            | `active_id()`            |
//! | Membership                        | `contains(id)`           | `contains(id)`           |
//! | `is_active(id)`                   | —                        | `is_active(id)`          |
//! | Count                             | `len()`                  | `theme_count()`          |
//! | Empty                             | `is_empty()`             | `is_empty()`             |
//! | All IDs                           | `ids()`                  | `theme_ids()`            |
//!
//! ## Stability of order
//!
//! `info_list()` / `theme_infos()` are **sorted by name (case-insensitive)
//! then id**, so picker output is stable across runs. `list()`,
//! `themes()`, `ids()`, and `theme_ids()` iterate the underlying
//! `HashMap` and have unspecified order — don't rely on it.
//!
//! ## Cost
//!
//! | Method                         | Clones per entry                              |
//! |--------------------------------|-----------------------------------------------|
//! | `info_list()` / `theme_infos()`| 4 strings + `Copy` of `appearance` (~200 B)   |
//! | `list()` / `themes()`          | Entire resolved theme: semantic map, colors, components JSON, structured list/menu/table themes |
//!
//! For a 10-theme picker, `theme_infos()` copies ~2 KB total; `themes()`
//! copies the full resolved state of every theme. Use `theme_infos()`
//! unless you actually need the resolved tokens.
//!
//! # Example theme file
//!
//! ```json
//! {
//!   "id": "my-app-dark",
//!   "name": "My App Dark",
//!   "appearance": "dark",
//!   "version": "1.0.0",
//!   "author": "you",
//!
//!   "palette": {
//!     "brand.500": "#6366f1",
//!     "brand.600": "#4f46e5"
//!   },
//!
//!   "semantic": {
//!     "background": "{neutral.950}",
//!     "foreground": "{neutral.50}",
//!     "primary": "{brand.500}",
//!     "primaryForeground": "{neutral.50}",
//!     "muted": "{neutral.900}",
//!     "mutedForeground": "{neutral.400}",
//!     "border": "{neutral.800}",
//!     "input": "{neutral.800}",
//!     "ring": "{brand.500}"
//!   },
//!
//!   "colors": {
//!     "button.primary.background": "{primary}",
//!     "button.primary.hover.background": "{brand.600}",
//!     "button.primary.foreground": "{primaryForeground}",
//!     "list.row.hover": "{muted}"
//!   },
//!
//!   "list": {
//!     "rowHeight": 36,
//!     "rowPaddingX": 12,
//!     "radius": 6
//!   },
//!
//!   "menu": {
//!     "itemPaddingX": 10,
//!     "itemPaddingY": 6
//!   },
//!
//!   "table": {
//!     "cellPaddingX": 12,
//!     "cellPaddingY": 8
//!   },
//!
//!   "radius": { "sm": 4, "md": 6, "lg": 8 },
//!   "spacing": { "xs": 4, "sm": 8, "md": 12, "lg": 16 },
//!   "typography": {
//!     "fontFamily": {
//!       "sans": "Inter, ui-sans-serif, system-ui",
//!       "mono": "JetBrains Mono, ui-monospace"
//!     },
//!     "fontSize": {
//!       "sm": [14, 20],
//!       "md": [16, 24],
//!       "lg": [18, 28]
//!     },
//!     "fontWeight": { "base": 400, "medium": 500, "bold": 700 }
//!   },
//!   "zIndex": { "0": 0, "10": 10, "20": 20, "overlay": 1000 },
//!   "motion": {
//!     "duration": { "fast": 150, "normal": 250 },
//!     "easing":   { "standard": "cubic-bezier(.4,0,.2,1)" }
//!   },
//!   "shadows": {
//!     "sm": "0 1px 2px rgba(0,0,0,.05)",
//!     "md": "0 4px 6px rgba(0,0,0,.1)"
//!   }
//! }
//! ```
//!
//! # Token reference syntax
//!
//! Inside string token values you can write:
//!
//! | Syntax                         | Meaning                                                    |
//! |--------------------------------|------------------------------------------------------------|
//! | `#rrggbb` / `#rrggbbaa`        | Hex, with optional alpha                                   |
//! | `#rgb` / `#rgba`               | Short hex                                                  |
//! | `{blue.500}`                   | Palette function from [`crate::styling::colors`]           |
//! | `{brand.500}`                  | Palette override (defined in `palette`)                    |
//! | `{semantic.primary}`           | Another semantic token (resolved iteratively)              |
//! | `{blue.500}/50`                | Any of the above, with 0–100 alpha **overriding** source   |
//! | `transparent`                  | `rgba(0,0,0,0)`                                            |
//!
//! Gradients use structured objects (see [`LinearSpec`], [`RadialSpec`],
//! [`ConicSpec`]) — their `stops[].color` accepts the same syntax.
//!
//! # Panic behaviour
//!
//! The following cause a panic at load time:
//!
//! - Malformed JSON.
//! - A palette override with an invalid hex string.
//! - A token referencing an unknown palette family or step.
//! - A token referencing an unknown `{semantic.*}` key.
//! - A cycle in `semantic` references.
//! - A structured `list` / `menu` / `table` / `colors` value that fails
//!   to resolve.
//! - A component token that looks like a color / radius / gradient but
//!   can't be resolved.
//!
//! All panics carry the offending JSON path (e.g.
//! ``at `list.rowHover`: unknown palette step: missing.500``), so fixing
//! a broken theme file is quick.

#![allow(clippy::too_many_arguments)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use freya::prelude::*;
use freya::prelude::{
    Readable, State, provide_context, try_consume_context, use_consume, use_hook,
};
use paste::paste;
use serde::{Deserialize, Serialize};

use crate::styling::colors::{
    StyledColor, black, blue_500, blue_600, blue_700, green_500, neutral_100, neutral_200,
    neutral_300, neutral_400, neutral_500, red_600, transparent, white, yellow_500,
};
use crate::styling::{Style, StyledExt};
use std::cell::RefCell;

// ---------------------------------------------------------------------
// Thread-local active theme
// ---------------------------------------------------------------------

thread_local! {
    /// The theme currently active on this thread. Read by
    /// [`Style::default`] and every `active_theme()`-based shortcut in
    /// [`crate::styling::styled`] (radius, spacing, fonts, z-index, etc.).
    ///
    /// Written by [`set_active_theme`], which itself is invoked by
    /// [`load_theme_registry`], [`load_theme_registry_from_dir`], and
    /// [`ThemeHandle::set_active`] / [`ThemeHandle::reload_dir`] — you
    /// rarely need to call it directly.
    ///
    /// Using a thread-local instead of a global keeps parallel test
    /// runners isolated, at the cost of needing the theme to be installed
    /// on any thread that builds UI (Freya guarantees this for you).
    static ACTIVE_THEME: RefCell<Option<Arc<Theme>>> = RefCell::new(None);
}

/// Install `theme` as the current thread's active theme.
///
/// Called automatically by:
///
/// - [`load_theme_registry`] — on first render.
/// - [`load_theme_registry_from_dir`] — on first render.
/// - [`ThemeHandle::set_active`] — on every theme switch.
/// - [`ThemeHandle::reload_dir`] — after re-reading the directory.
///
/// Any [`Style::default`] / `active_theme()`-based call **after** this
/// point picks up the new theme's tokens. UI already built on the current
/// frame keeps its old colours until the next render — Freya re-runs the
/// render function, which creates fresh `Style`s.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use freya_cn::theme::{set_active_theme, Theme};
///
/// let theme = Arc::new(Theme::try_from_json_str(include_str!("theme.json")).unwrap());
/// set_active_theme(theme);
/// ```
pub fn set_active_theme(theme: Arc<Theme>) {
    ACTIVE_THEME.with(|slot| *slot.borrow_mut() = Some(theme));
}

/// Read the current thread's active theme.
///
/// Every `active_theme()` call in [`crate::styling::styled`] uses this to
/// resolve `rounded_*`, `text_*`, `font_*`, `gap_*`, `z_*`, `p_*`, etc.
/// It's also what [`Style::default`] reads for `background` / `foreground`.
///
/// Falls back to a shared [`Theme::empty_light`] when no theme has been
/// installed (unit tests, headless runs, WASM initial frame, …). The
/// fallback is **not** cached — each call constructs a fresh `Arc`, so
/// installing a real theme at any time takes effect on the next call.
///
/// # Performance
///
/// One `Arc` clone + one `HashMap` lookup per call. Fine for typical UI
/// code. If you're building thousands of styled nodes per frame, hoist
/// `let theme = active_theme();` and use its `*_or(..)` helpers directly.
pub fn active_theme() -> Arc<Theme> {
    ACTIVE_THEME
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| Arc::new(Theme::empty_light()))
}

// ---------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------

/// Everything that can go wrong while parsing, resolving, or loading a
/// theme.
///
/// The `try_*` methods on [`Theme`], [`ThemeRegistry`], and
/// [`ThemeHandle`] return `Result<_, ThemeError>`. The non-`try`
/// variants panic with the same message — see the module docs for the
/// rationale.
///
/// # Example
///
/// ```no_run
/// use freya_cn::theme::{Theme, ThemeError};
///
/// match Theme::try_from_json_str(r#"{ "id": "x" }"#) {
///     Err(ThemeError::Json(e)) => eprintln!("bad json: {e}"),
///     Err(ThemeError::UnknownPaletteStep(s)) => eprintln!("unknown palette: {s}"),
///     Err(ThemeError::TokenAt { path, reason }) => {
///         eprintln!("`{path}` failed: {reason}");
///     }
///     Err(e) => eprintln!("theme error: {e}"),
///     Ok(theme) => { let _ = theme; }
/// }
/// ```
#[derive(Debug)]
pub enum ThemeError {
    /// Malformed JSON. Wraps the underlying `serde_json` error, which
    /// points at the exact line / column.
    Json(serde_json::Error),

    /// I/O failure while reading a theme file or directory.
    Io(std::io::Error),

    /// A `palette` override or hex literal contained an invalid hex
    /// string (wrong length, non-hex characters, empty).
    InvalidHex(String),

    /// A palette reference like `{blue.500}` had a malformed alpha
    /// suffix — e.g. `{blue.500}/abc`.
    InvalidPaletteRef(String),

    /// A palette reference like `{nope.500}` or `{blue.999}` used a
    /// family / step that doesn't exist.
    UnknownPaletteStep(String),

    /// [`ThemeRegistry::set_active`] was called with an ID that isn't
    /// registered.
    UnknownTheme(String),

    /// A token resolved to something that isn't a solid colour (usually
    /// a gradient) where the API required a solid `Color` — e.g.
    /// `{semantic.brand}` where `brand` is a gradient used as a
    /// `component_color`.
    NotASolidColor(String),

    /// A token at `path` (JSON dot-path like `list.rowHover` or
    /// `colors.button.primary.background`) failed to resolve for `reason`.
    ///
    /// This is the most common error shape; the inner `reason` is usually
    /// one of the variants above.
    TokenAt {
        path: String,
        reason: Box<ThemeError>,
    },

    /// The `semantic` map contains tokens that reference each other in a
    /// cycle, or reference keys that don't exist. The `Vec` lists the
    /// offending token names.
    Cycle(Vec<String>),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "json error: {e}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::InvalidHex(s) => write!(f, "invalid hex color: {s}"),
            Self::InvalidPaletteRef(s) => write!(f, "invalid palette reference: {s}"),
            Self::UnknownPaletteStep(s) => write!(f, "unknown palette step: {s}"),
            Self::UnknownTheme(s) => write!(f, "unknown theme id: {s}"),
            Self::NotASolidColor(s) => write!(f, "token is not a solid color: {s}"),
            Self::TokenAt { path, reason } => write!(f, "at `{path}`: {reason}"),
            Self::Cycle(tokens) => write!(
                f,
                "unresolvable semantic tokens (cycle or missing reference): {tokens:?}"
            ),
        }
    }
}
impl std::error::Error for ThemeError {}
impl From<serde_json::Error> for ThemeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
impl From<std::io::Error> for ThemeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// `true` for errors that mean "this token might resolve on the next pass
/// once `{semantic.*}` dependencies are known" — see
/// [`resolve_semantic_map`].
fn is_retryable(e: &ThemeError) -> bool {
    matches!(e, ThemeError::NotASolidColor(_))
}

// ---------------------------------------------------------------------
// Appearance + metadata
// ---------------------------------------------------------------------

/// Light vs dark, mirrors the `appearance` field of the theme JSON.
///
/// Currently informational only — the actual colours come from
/// `semantic.background` / `semantic.foreground` etc. Exposed so callers
/// can branch on it (e.g. swap icon assets, adjust shadows) without
/// duplicating the check against `theme.id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}

/// Lightweight metadata about a loaded theme.
///
/// Returned by [`Theme::metadata`], [`ThemeRegistry::theme_info`] /
/// [`ThemeRegistry::info_list`] / [`ThemeRegistry::active_info`], and the
/// equivalents on [`ThemeHandle`]. Cheaper than cloning an
/// `Arc<Theme>` when you only need to *display* a theme (pickers,
/// dropdowns, settings panels, CLI listings).
///
/// # Fields
///
/// | Field        | Source                       | Notes                                    |
/// |--------------|------------------------------|------------------------------------------|
/// | `id`         | `ThemeConfig::id`            | The stable key for [`set_active`](ThemeRegistry::set_active) |
/// | `name`       | `ThemeConfig::name`          | Display string — safe to localize in the picker |
/// | `appearance` | `ThemeConfig::appearance`    | `Light` / `Dark` — use for sun/moon icons |
/// | `version`    | `ThemeConfig::version`       | Semantic version, may be `""`             |
/// | `author`     | `ThemeConfig::author`        | `Option<String>`                          |
///
/// # Building one
///
/// Most callers get a `ThemeInfo` from a lookup; you rarely construct
/// one directly. When you do (e.g. in a test fixture), it's a plain
/// struct with public fields:
///
/// ```
/// use freya_cn::theme::{Appearance, ThemeInfo};
///
/// let info = ThemeInfo {
///     id: "default-dark".into(),
///     name: "FreyaCN Dark".into(),
///     appearance: Appearance::Dark,
///     version: "1.0.0".into(),
///     author: Some("FreyaCN".into()),
/// };
/// ```
///
/// # From an existing theme
///
/// [`Theme::metadata`] clones only the five fields it needs — the resolved
/// colours, tokens, and component themes are **not** cloned:
///
/// ```no_run
/// # use freya_cn::theme::Theme;
/// # let theme = Theme::try_from_json_str("{}").unwrap();
/// let info = theme.metadata();
/// assert_eq!(info.id, theme.id);
/// assert_eq!(info.name, theme.name);
/// assert_eq!(info.appearance, theme.appearance);
/// ```
///
/// # Persisting a user's choice
///
/// `ThemeInfo` is `Serialize` / `Deserialize`, so you can store the
/// selected theme's metadata to a config file and rehydrate it later
/// without parsing the full theme:
///
/// ```
/// # use freya_cn::theme::{Appearance, ThemeInfo};
/// let info = ThemeInfo {
///     id: "default-dark".into(),
///     name: "FreyaCN Dark".into(),
///     appearance: Appearance::Dark,
///     version: "1.0.0".into(),
///     author: None,
/// };
/// let json = serde_json::to_string(&info).unwrap();
/// let back: ThemeInfo = serde_json::from_str(&json).unwrap();
/// assert_eq!(info, back);
/// ```
///
/// To re-apply a saved choice, pass the `id` to
/// [`ThemeHandle::set_active`]:
///
/// ```no_run
/// # use freya_cn::theme::ThemeHandle;
/// # let handle: ThemeHandle = todo!();
/// # let saved_info: freya_cn::theme::ThemeInfo = todo!();
/// if handle.contains(&saved_info.id) {
///     handle.set_active(&saved_info.id).unwrap();
/// }
/// ```
///
/// # Rendering a picker
///
/// The typical shape:
///
/// ```no_run
/// # use freya_cn::theme::{ThemeHandle, ThemeInfo};
/// fn render_picker(handle: &ThemeHandle) -> Vec<String> {
///     let active = handle.active_info();
///     handle
///         .theme_infos()
///         .into_iter()
///         .map(|info| {
///             let marker = if info.id == active.id { "✓" } else { " " };
///             let icon = match info.appearance {
///                 freya_cn::theme::Appearance::Light => "☀",
///                 freya_cn::theme::Appearance::Dark => "☾",
///             };
///             format!("{marker} {icon} {} ({})", info.name, info.id)
///         })
///         .collect()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInfo {
    /// Stable identifier (matches `ThemeConfig::id`).
    pub id: String,
    /// Human-readable name shown in pickers.
    pub name: String,
    /// Light/dark marker — useful for badges or auto-selecting a default.
    pub appearance: Appearance,
    /// Semantic version string (may be empty).
    pub version: String,
    /// Author name, if the theme provides one.
    pub author: Option<String>,
}

impl ThemeInfo {
    /// Build a `ThemeInfo` from a [`Theme`] without cloning the whole
    /// thing.
    fn from_theme(t: &Theme) -> Self {
        Self {
            id: t.id.clone(),
            name: t.name.clone(),
            appearance: t.appearance,
            version: t.version.clone(),
            author: t.author.clone(),
        }
    }
}

// ---------------------------------------------------------------------
// Component state
// ---------------------------------------------------------------------

/// Interaction / value state a component can enter. Mirrors gpui-kit's
/// state set.
///
/// Used by [`Theme::state_fill`] / [`Theme::state_color`] /
/// [`Theme::state_radius`] to look up `{path}.{state}.{field}` with a
/// per-state fallback chain, before falling back to `{path}.{field}`.
///
/// # Fallback chain
///
/// The fallback chain lets a theme author write fewer keys — e.g. only
/// `focus` is needed for a keyboard-focusable control to also pick up
/// the visual treatment when `FocusVisible` is requested:
///
/// | Requested          | Falls back to |
/// |--------------------|---------------|
/// | `FocusVisible`     | `focus`       |
/// | `Pressed`          | `active`      |
/// | `Open`             | `active`      |
/// | `Indeterminate`    | `checked`     |
/// | everything else    | —             |
///
/// # Example
///
/// ```json
/// {
///   "components": {
///     "button": {
///       "primary": {
///         "background": "{primary}",
///         "hover": { "background": "{brand.600}" },
///         "active": { "background": "{brand.700}" },
///         "focus": { "ring": "{ring}" }
///       }
///     }
///   }
/// }
/// ```
///
/// With this config, `state_fill("button.primary", Hover, "background")`
/// returns `{brand.600}`, `state_fill(..., Pressed, "background")` falls
/// through to `active` → `{brand.700}`, and `state_fill(..., Base, ...)`
/// returns `{primary}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentState {
    /// No interaction — the default state.
    Base,
    /// Pointer is over the element.
    Hover,
    /// Element is being pressed (mouse down).
    Active,
    /// Element has focus.
    Focus,
    /// Element has focus via keyboard (subset of `Focus`).
    FocusVisible,
    /// Element is disabled / non-interactive.
    Disabled,
    /// Checkbox / radio / switch is on.
    Checked,
    /// Checkbox is in the indeterminate state (subset of `Checked`).
    Indeterminate,
    /// Item is selected (list, tab, etc.).
    Selected,
    /// Button-style press (subset of `Active`).
    Pressed,
    /// Popover / dropdown / accordion is expanded (subset of `Active`).
    Open,
}

impl ComponentState {
    /// JSON key for this state (`"hover"`, `"focus-visible"`, …).
    pub fn as_str(self) -> &'static str {
        match self {
            ComponentState::Base => "base",
            ComponentState::Hover => "hover",
            ComponentState::Active => "active",
            ComponentState::Focus => "focus",
            ComponentState::FocusVisible => "focus-visible",
            ComponentState::Disabled => "disabled",
            ComponentState::Checked => "checked",
            ComponentState::Indeterminate => "indeterminate",
            ComponentState::Selected => "selected",
            ComponentState::Pressed => "pressed",
            ComponentState::Open => "open",
        }
    }

    /// Alternative state names to try if this state has no explicit
    /// component token. See the type-level docs for the full table.
    pub fn fallback_chain(self) -> &'static [&'static str] {
        match self {
            ComponentState::Base
            | ComponentState::Hover
            | ComponentState::Active
            | ComponentState::Focus
            | ComponentState::Disabled
            | ComponentState::Checked
            | ComponentState::Selected => &[],
            ComponentState::FocusVisible => &["focus"],
            ComponentState::Indeterminate => &["checked"],
            ComponentState::Pressed => &["active"],
            ComponentState::Open => &["active"],
        }
    }
}

// ---------------------------------------------------------------------
// JSON shapes
// ---------------------------------------------------------------------

/// A single token value, in any of the JSON-supported shapes.
///
/// Parsed from `serde(untagged)`, so the JSON discriminator is the
/// object key (`linear` / `radial` / `conic`) or the bare string form.
///
/// # Examples
///
/// ```json
/// "{blue.500}"
/// "{semantic.primary}/50"
/// "#ff8800"
/// "transparent"
/// ```
///
/// Gradients:
///
/// ```json
/// { "linear": { "angle": 135, "stops": [
///     { "color": "{blue.500}", "at": 0 },
///     { "color": "{purple.500}", "at": 100 }
/// ]}}
/// ```
///
/// ```json
/// { "radial": { "center": [0.5, 0.5], "radius": 1.0, "stops": [ /* … */ ] }}
/// ```
///
/// ```json
/// { "conic": { "angle": 0, "center": [0.5, 0.5], "stops": [ /* … */ ] }}
/// ```
///
/// # Notes
///
/// - Radial / conic `center` and radial `radius` are parsed but
///   **currently ignored** — Freya's `RadialGradient` / `ConicGradient`
///   only accept stops. The fields are kept for forward compatibility.
/// - Gradient stops support the same string syntax as solid tokens,
///   including `{semantic.*}` references (resolved iteratively).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// Solid colour: `"#fff"`, `"{blue.500}"`, `"{neutral.950}/24"`, `"transparent"`.
    Color(String),
    /// Linear gradient (135° default if `angle` omitted).
    Linear { linear: LinearSpec },
    /// Radial gradient.
    Radial { radial: RadialSpec },
    /// Conic gradient.
    Conic { conic: ConicSpec },
}

/// A single gradient colour stop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stop {
    /// Token string — accepts the same syntax as `TokenValue::Color`.
    pub color: String,
    /// Position along the gradient, `0.0 ..= 100.0`.
    pub at: f32,
}

/// Linear gradient parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearSpec {
    /// Angle in degrees clockwise from "up". Defaults to `0`.
    #[serde(default)]
    pub angle: f32,
    /// Ordered list of colour stops.
    pub stops: Vec<Stop>,
}

/// Radial gradient parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialSpec {
    /// Relative centre `(x, y)` in `0.0 ..= 1.0`. Defaults to `(0.5, 0.5)`.
    ///
    /// **Currently ignored** — Freya's `RadialGradient` only accepts
    /// stops. Kept for forward compatibility.
    #[serde(default = "c05")]
    pub center: (f32, f32),
    /// Relative radius in `0.0 ..= 1.0`. Defaults to `1.0`.
    ///
    /// **Currently ignored** — see `center`.
    #[serde(default = "one")]
    pub radius: f32,
    /// Ordered list of colour stops.
    pub stops: Vec<Stop>,
}

/// Conic gradient parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConicSpec {
    /// Angle in degrees. Defaults to `0`.
    #[serde(default)]
    pub angle: f32,
    /// Relative centre `(x, y)` in `0.0 ..= 1.0`. Defaults to `(0.5, 0.5)`.
    ///
    /// **Currently ignored** — see [`RadialSpec::center`].
    #[serde(default = "c05")]
    pub center: (f32, f32),
    /// Ordered list of colour stops.
    pub stops: Vec<Stop>,
}

fn c05() -> (f32, f32) {
    (0.5, 0.5)
}
fn one() -> f32 {
    1.0
}

/// Typography tokens — font stacks, sizes, weights.
///
/// # Example
///
/// ```json
/// {
///   "fontFamily": {
///     "sans": "Inter, ui-sans-serif, system-ui",
///     "serif": "Georgia, Cambria, Times New Roman, serif",
///     "mono": "JetBrains Mono, ui-monospace"
///   },
///   "fontSize": {
///     "xs": [12, 16],
///     "sm": [14, 20],
///     "md": [16, 24],
///     "lg": [18, 28],
///     "xl": [20, 28]
///   },
///   "fontWeight": { "base": 400, "medium": 500, "bold": 700 }
/// }
/// ```
///
/// - `fontFamily` values are comma-separated stacks.
/// - `fontSize` values are `[size, line_height]` pairs (both in `px`).
///   `line_height` is currently informational only; the `text_*`
///   shortcuts use `size`.
/// - `fontWeight` values are CSS numbers (`100 ..= 900`).
///
/// Consumed by [`Theme::font_size_or`] / [`Theme::font_weight_or`] /
/// [`Theme::font_family_stack`], which drive the `text_*`, `font_*`
/// weight, and `font_sans`/`font_serif`/`font_mono` shortcuts in
/// [`crate::styling::styled`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypographyConfig {
    /// `key -> comma-separated stack`.
    #[serde(default)]
    pub font_family: HashMap<String, String>,
    /// `key -> (size_px, line_height_px)`.
    #[serde(default)]
    pub font_size: HashMap<String, (f32, f32)>,
    /// `key -> CSS weight (100..=900)`.
    #[serde(default)]
    pub font_weight: HashMap<String, u16>,
}

/// Motion tokens — durations and easing curves.
///
/// **Declarative only.** Freya's `Style` has no transition / animation
/// fields, so these values are exposed through
/// [`Theme::motion_duration`] / [`Theme::motion_easing`] for future use
/// or custom renderers, but nothing in
/// [`StyledExt`] consumes them.
///
/// # Example
///
/// ```json
/// {
///   "duration": { "fast": 150, "normal": 250, "slow": 400 },
///   "easing": {
///     "standard": "cubic-bezier(.4,0,.2,1)",
///     "ease-out": "cubic-bezier(0,0,.2,1)"
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionConfig {
    /// `key -> duration in milliseconds`.
    #[serde(default)]
    pub duration: HashMap<String, u32>,
    /// `key -> easing curve string` (CSS syntax, not parsed).
    #[serde(default)]
    pub easing: HashMap<String, String>,
}

// ---------------------------------------------------------------------
// Structured component configs (gpui-kit compatible)
// ---------------------------------------------------------------------

/// Structured `list` config. Overrides the defaults from
/// [`ListTheme`] when present; individual keys are optional.
///
/// Matching entries under `colors` (e.g. `"list.row.hover"`) override
/// the corresponding field here, so a minimal `list` block for
/// **metrics only** + `colors` for **paint** is a valid split.
///
/// # Example — full
///
/// ```json
/// {
///   "list": {
///     "background": "{background}",
///     "border": "{border}",
///     "rowBackground": "{background}",
///     "rowHover": "{muted}",
///     "rowSelected": "{accent}",
///     "rowActive": "{muted}",
///     "rowForeground": "{foreground}",
///     "rowSelectedForeground": "{accentForeground}",
///     "rowDisabledForeground": "{disabledForeground}",
///     "rowHeight": 36,
///     "rowPaddingX": 12,
///     "rowPaddingY": 6,
///     "radius": 6,
///     "gap": 0
///   }
/// }
/// ```
///
/// # Example — metrics only
///
/// ```json
/// {
///   "list": { "rowHeight": 40, "radius": 8 },
///   "colors": {
///     "list.row.hover": "{muted}",
///     "list.row.selected": "{accent}"
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListConfig {
    /// List container background.
    #[serde(default)]
    pub background: Option<TokenValue>,
    /// List container border colour.
    #[serde(default)]
    pub border: Option<TokenValue>,
    /// Row background (idle).
    #[serde(default)]
    pub row_background: Option<TokenValue>,
    /// Row background on hover.
    #[serde(default)]
    pub row_hover: Option<TokenValue>,
    /// Row background when selected.
    #[serde(default)]
    pub row_selected: Option<TokenValue>,
    /// Row background while pressed.
    #[serde(default)]
    pub row_active: Option<TokenValue>,
    /// Row text colour (idle).
    #[serde(default)]
    pub row_foreground: Option<TokenValue>,
    /// Row text colour when selected.
    #[serde(default)]
    pub row_selected_foreground: Option<TokenValue>,
    /// Row text colour when disabled.
    #[serde(default)]
    pub row_disabled_foreground: Option<TokenValue>,
    /// Row height in `px`. Defaults to `32`.
    #[serde(default)]
    pub row_height: Option<f32>,
    /// Row horizontal padding in `px`. Defaults to `12`.
    #[serde(default)]
    pub row_padding_x: Option<f32>,
    /// Row vertical padding in `px`. Defaults to `6`.
    #[serde(default)]
    pub row_padding_y: Option<f32>,
    /// Row corner radius in `px`. Defaults to `6`.
    #[serde(default)]
    pub radius: Option<f32>,
    /// Gap between rows in `px`. Defaults to `0`.
    #[serde(default)]
    pub gap: Option<f32>,
}

/// Structured `menu` config. Same split as [`ListConfig`]: this block
/// sets paint + metrics, `colors` entries override individual pieces.
///
/// # Example
///
/// ```json
/// {
///   "menu": {
///     "background": "{popover}",
///     "border": "{border}",
///     "itemHover": "{muted}",
///     "itemSelected": "{accent}",
///     "itemForeground": "{popoverForeground}",
///     "separator": "{border}",
///     "radius": 6,
///     "itemPaddingX": 10,
///     "itemPaddingY": 6
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfig {
    /// Menu container background.
    #[serde(default)]
    pub background: Option<TokenValue>,
    /// Menu container border colour.
    #[serde(default)]
    pub border: Option<TokenValue>,
    /// Item background on hover.
    #[serde(default)]
    pub item_hover: Option<TokenValue>,
    /// Item background when selected.
    #[serde(default)]
    pub item_selected: Option<TokenValue>,
    /// Item text colour.
    #[serde(default)]
    pub item_foreground: Option<TokenValue>,
    /// Separator line colour.
    #[serde(default)]
    pub separator: Option<TokenValue>,
    /// Menu corner radius in `px`. Defaults to `6`.
    #[serde(default)]
    pub radius: Option<f32>,
    /// Item horizontal padding in `px`. Defaults to `10`.
    #[serde(default)]
    pub item_padding_x: Option<f32>,
    /// Item vertical padding in `px`. Defaults to `6`.
    #[serde(default)]
    pub item_padding_y: Option<f32>,
}

/// Structured `table` config. Same split as [`ListConfig`].
///
/// # Example
///
/// ```json
/// {
///   "table": {
///     "background": "{card}",
///     "border": "{border}",
///     "headerBackground": "{muted}",
///     "headerForeground": "{mutedForeground}",
///     "rowHover": "{accent}",
///     "rowSelected": "{accent}",
///     "cellPaddingX": 12,
///     "cellPaddingY": 8,
///     "radius": 6
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableConfig {
    /// Table background.
    #[serde(default)]
    pub background: Option<TokenValue>,
    /// Table border colour.
    #[serde(default)]
    pub border: Option<TokenValue>,
    /// Header row background.
    #[serde(default)]
    pub header_background: Option<TokenValue>,
    /// Header row text colour.
    #[serde(default)]
    pub header_foreground: Option<TokenValue>,
    /// Row background on hover.
    #[serde(default)]
    pub row_hover: Option<TokenValue>,
    /// Row background when selected.
    #[serde(default)]
    pub row_selected: Option<TokenValue>,
    /// Cell horizontal padding in `px`. Defaults to `12`.
    #[serde(default)]
    pub cell_padding_x: Option<f32>,
    /// Cell vertical padding in `px`. Defaults to `8`.
    #[serde(default)]
    pub cell_padding_y: Option<f32>,
    /// Table corner radius in `px`. Defaults to `6`.
    #[serde(default)]
    pub radius: Option<f32>,
}

/// The full `.json` shape of a theme file.
///
/// Usually you'd hand a JSON string to [`Theme::try_from_json_str`]
/// directly — `ThemeConfig` exists so the intermediate state is
/// inspectable / testable, and so `serde_json::from_str::<ThemeConfig>`
/// can be used on its own (e.g. to round-trip a theme back to JSON).
///
/// See the module docs for a complete example JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    /// Stable identifier (`"my-app-dark"`). Referenced by
    /// [`ThemeRegistry::set_active`].
    pub id: String,
    /// Human-readable name shown in theme pickers.
    pub name: String,
    /// Light vs dark. Purely informational (see [`Appearance`]).
    pub appearance: Appearance,
    /// Semantic version string. Optional; defaults to `""`.
    #[serde(default)]
    pub version: String,
    /// Author name. Optional.
    #[serde(default)]
    pub author: Option<String>,

    /// Palette overrides on top of the built-in Tailwind scale.
    ///
    /// Key is `"group.step"` (`"brand.500"`), value is a hex string.
    /// Referenced from tokens as `{brand.500}`.
    #[serde(default)]
    pub palette: HashMap<String, String>,

    /// Core semantic tokens (`background`, `foreground`, `primary`, …).
    ///
    /// Can reference each other via `{semantic.other}` — resolution is
    /// iterative with cycle detection (see [`resolve_semantic_map`]).
    pub semantic: HashMap<String, TokenValue>,

    /// Flat dotted tokens, gpui-kit style: `"button.primary.background"`,
    /// `"list.row.hover"`, etc.
    ///
    /// Exposed through [`Theme::token_fill`] / [`Theme::token_color`] /
    /// [`Theme::token`]. Also transparently overlaid onto the structured
    /// `list` / `menu` / `table` themes when the key matches (e.g.
    /// `"list.row.hover"` → [`ListTheme::row_hover`]).
    #[serde(default)]
    pub colors: HashMap<String, TokenValue>,

    /// Structured list theme. See [`ListConfig`].
    #[serde(default)]
    pub list: ListConfig,
    /// Structured menu theme. See [`MenuConfig`].
    #[serde(default)]
    pub menu: MenuConfig,
    /// Structured table theme. See [`TableConfig`].
    #[serde(default)]
    pub table: TableConfig,

    /// Free-form nested component tokens, read by
    /// [`Theme::component_fill`] / [`Theme::component_color`] /
    /// [`Theme::component_radius`] and their state-aware wrappers.
    ///
    /// Any JSON object; not validated at load time (only when looked up).
    #[serde(default)]
    pub components: serde_json::Value,

    /// Named radius values (`"sm": 4`, `"lg": 12`, …). Drives the
    /// `rounded_*` shortcuts in [`crate::styling::styled`].
    #[serde(default)]
    pub radius: HashMap<String, f32>,
    /// Named spacing values (`"sm": 8`, `"lg": 16`, …). Drives the
    /// named `p_sm` / `gap_md` / … shortcuts.
    #[serde(default)]
    pub spacing: HashMap<String, f32>,
    /// Typography tokens. See [`TypographyConfig`].
    #[serde(default)]
    pub typography: TypographyConfig,

    /// **Declarative only.** Freya's `Style` has no box-shadow field, so
    /// these values are exposed via [`Theme::shadow`] for future use /
    /// SVG filters / custom rendering, but nothing in `StyledExt`
    /// consumes them.
    #[serde(default)]
    pub shadows: HashMap<String, String>,

    /// **Declarative only.** Freya's `Style` has no transition fields,
    /// so these are exposed via [`Theme::motion_duration`] /
    /// [`Theme::motion_easing`] but do not animate anything.
    #[serde(default)]
    pub motion: MotionConfig,

    /// Named z-index values. Drives the `z_*` shortcuts. Falls back to
    /// the Tailwind numeric scale (`"0"`, `"10"`, …) when a key is
    /// missing.
    #[serde(default, rename = "zIndex")]
    pub z_index: HashMap<String, i32>,
}

// ---------------------------------------------------------------------
// Contrast pairs
// ---------------------------------------------------------------------

/// `(foreground, background)` pairs checked by [`Theme::validate`] for
/// WCAG AA body-text contrast (4.5:1).
///
/// UI-only pairs (`border`, `ring`, `input`) are excluded because they
/// follow the non-text 3:1 rule, which is lower than what
/// `contrast_ratio` would flag here.
const CONTRAST_PAIRS: &[(&str, &str)] = &[
    ("background", "foreground"),
    ("card", "cardForeground"),
    ("popover", "popoverForeground"),
    ("primary", "primaryForeground"),
    ("secondary", "secondaryForeground"),
    ("muted", "mutedForeground"),
    ("accent", "accentForeground"),
    ("destructive", "destructiveForeground"),
    ("sidebar", "sidebarForeground"),
    ("sidebarPrimary", "sidebarPrimaryForeground"),
    ("sidebarAccent", "sidebarAccentForeground"),
    ("success", "successForeground"),
    ("warning", "warningForeground"),
    ("info", "infoForeground"),
    ("link", "background"),
];

// ---------------------------------------------------------------------
// Resolved component themes
// ---------------------------------------------------------------------

/// A resolved flat token that may be a solid colour or a gradient.
///
/// Returned by [`Theme::token`]. Use `color` when the caller needs a
/// single `Color` (e.g. text); use `background` when it needs the full
/// `Fill` (e.g. a background that might be a gradient).
///
/// # Example
///
/// ```no_run
/// # use freya_cn::theme::Theme;
/// # let theme = Theme::try_from_json_str("{}").unwrap();
/// if let Some(tok) = theme.token("button.primary.background") {
///     // Solid callers: tok.color
///     // Full-fidelity callers: tok.background
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeToken {
    /// Solid representative colour. For gradient tokens this is
    /// transparent (`rgba(0,0,0,0)`), matching what Freya's painter would
    /// render if given the raw `Color`.
    pub color: Color,
    /// Full resolved fill — solid or gradient.
    pub background: Fill,
}

/// Resolved list theme. Built from [`ThemeConfig::list`] + overlaid
/// `colors` entries. Accessible as [`Theme::list`].
///
/// # Example
///
/// ```no_run
/// # use freya_cn::theme::Theme;
/// # let theme = Theme::try_from_json_str("{}").unwrap();
/// let row_hover = theme.list.row_hover.clone();
/// let row_h = theme.list.row_height;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ListTheme {
    pub background: Fill,
    pub border: Fill,
    pub row_background: Fill,
    pub row_hover: Fill,
    pub row_selected: Fill,
    pub row_active: Fill,
    pub row_foreground: Fill,
    pub row_selected_foreground: Fill,
    pub row_disabled_foreground: Fill,
    pub row_height: f32,
    pub row_padding_x: f32,
    pub row_padding_y: f32,
    pub radius: f32,
    pub gap: f32,
}

/// Resolved menu theme. Built from [`ThemeConfig::menu`] + overlaid
/// `colors` entries. Accessible as [`Theme::menu`].
#[derive(Debug, Clone, PartialEq)]
pub struct MenuTheme {
    pub background: Fill,
    pub border: Fill,
    pub item_hover: Fill,
    pub item_selected: Fill,
    pub item_foreground: Fill,
    pub separator: Fill,
    pub radius: f32,
    pub item_padding_x: f32,
    pub item_padding_y: f32,
}

/// Resolved table theme. Built from [`ThemeConfig::table`] + overlaid
/// `colors` entries. Accessible as [`Theme::table`].
#[derive(Debug, Clone, PartialEq)]
pub struct TableTheme {
    pub background: Fill,
    pub border: Fill,
    pub header_background: Fill,
    pub header_foreground: Fill,
    pub row_hover: Fill,
    pub row_selected: Fill,
    pub cell_padding_x: f32,
    pub cell_padding_y: f32,
    pub radius: f32,
}

impl ListTheme {
    /// Build a [`Style`] for a list row in `state`.
    ///
    /// Applies the theme's background, foreground, height, padding, and
    /// radius. The row's *content* (text, icons, trailing widgets) is the
    /// caller's job — this only sets the row container's appearance.
    ///
    /// # State mapping
    ///
    /// | State                              | Background       | Foreground              |
    /// |------------------------------------|------------------|-------------------------|
    /// | `Base`                             | `row_background` | `row_foreground`        |
    /// | `Hover`                            | `row_hover`      | `row_foreground`        |
    /// | `Active` / `Pressed`               | `row_active`     | `row_selected_foreground` |
    /// | `Selected` / `Checked`             | `row_selected`   | `row_selected_foreground` |
    /// | `Disabled`                         | `row_background` | `row_disabled_foreground` |
    /// | `Focus` / `FocusVisible` / `Open`  | `row_background` | `row_foreground`        |
    pub fn row_style(&self, state: ComponentState) -> Style {
        let (bg, fg): (&Fill, &Fill) = match state {
            ComponentState::Hover => (&self.row_hover, &self.row_foreground),
            ComponentState::Active | ComponentState::Pressed => {
                (&self.row_active, &self.row_selected_foreground)
            }
            ComponentState::Selected | ComponentState::Checked => {
                (&self.row_selected, &self.row_selected_foreground)
            }
            ComponentState::Disabled => (&self.row_background, &self.row_disabled_foreground),
            _ => (&self.row_background, &self.row_foreground),
        };
        Style::default()
            .bg(StyledColor::from_fill(bg.clone()))
            .text_color(StyledColor::from_fill(fg.clone()))
            .h(self.row_height)
            .px(self.row_padding_x)
            .py(self.row_padding_y)
            .radius(self.radius)
    }
}

impl MenuTheme {
    /// Build a [`Style`] for a menu item in `state`.
    pub fn item_style(&self, state: ComponentState) -> Style {
        let bg = match state {
            ComponentState::Hover => &self.item_hover,
            ComponentState::Selected | ComponentState::Checked => &self.item_selected,
            _ => &self.background,
        };
        Style::default()
            .bg(StyledColor::from_fill(bg.clone()))
            .text_color(StyledColor::from_fill(self.item_foreground.clone()))
            .px(self.item_padding_x)
            .py(self.item_padding_y)
            .radius(self.radius)
    }
}

impl TableTheme {
    /// Style for the header row.
    pub fn header_style(&self) -> Style {
        Style::default()
            .bg(StyledColor::from_fill(self.header_background.clone()))
            .text_color(StyledColor::from_fill(self.header_foreground.clone()))
            .radius(self.radius)
    }

    /// Style for a body row in `state`.
    pub fn row_style(&self, state: ComponentState) -> Style {
        let bg = match state {
            ComponentState::Hover => &self.row_hover,
            ComponentState::Selected | ComponentState::Checked => &self.row_selected,
            _ => &self.background,
        };
        Style::default().bg(StyledColor::from_fill(bg.clone()))
    }

    /// Style for a single cell.
    pub fn cell_style(&self) -> Style {
        Style::default()
            .px(self.cell_padding_x)
            .py(self.cell_padding_y)
    }
}

// ---------------------------------------------------------------------
// Resolved theme
// ---------------------------------------------------------------------

/// A fully-resolved theme.
///
/// # Accessor families
///
/// - **Semantic** — [`fill`](Self::fill), [`color`](Self::color), plus
///   the named getters ([`background`](Self::background),
///   [`primary`](Self::primary), …, [`link_hover`](Self::link_hover)).
///   Each `*_fill` variant preserves gradients.
/// - **Flat tokens** — [`token_fill`](Self::token_fill),
///   [`token_color`](Self::token_color), [`token`](Self::token).
/// - **Component tokens** — [`component_fill`](Self::component_fill),
///   [`component_color`](Self::component_color),
///   [`component_radius`](Self::component_radius), with state-aware
///   wrappers [`state_fill`](Self::state_fill),
///   [`state_color`](Self::state_color),
///   [`state_radius`](Self::state_radius).
/// - **Scale shortcuts** — [`radius`](Self::radius),
///   [`radius_or`](Self::radius_or),
///   [`spacing_or`](Self::spacing_or),
///   [`font_size_or`](Self::font_size_or),
///   [`font_weight_or`](Self::font_weight_or),
///   [`font_family_stack`](Self::font_family_stack),
///   [`z_index_or`](Self::z_index_or).
/// - **Structured components** — [`list`](Self::list),
///   [`menu`](Self::menu), [`table`](Self::table).
/// - **Metadata** — [`metadata`](Self::metadata) (cheap [`ThemeInfo`]).
/// - **Declarative only** — [`shadow`](Self::shadow),
///   [`motion_duration`](Self::motion_duration),
///   [`motion_easing`](Self::motion_easing).
///
/// # Example
///
/// ```no_run
/// use freya_cn::theme::{Theme, ComponentState};
///
/// let theme = Theme::try_from_json_str(include_str!("theme.json")).unwrap();
///
/// // Semantic lookups.
/// let bg = theme.background_fill();
/// let fg = theme.foreground();
///
/// // Component + state lookups.
/// if let Some(fill) = theme.state_fill("button.primary", ComponentState::Hover, "background") {
///     // ...
/// }
///
/// // Structured list theme.
/// let hover_bg = theme.list.row_hover.clone();
/// let row_h = theme.list.row_height;
///
/// // Metadata.
/// let info = theme.metadata();
/// println!("{} ({})", info.name, info.id);
/// ```
///
/// # Construction
///
/// - [`from_json_str`](Self::from_json_str) / [`try_from_json_str`](Self::try_from_json_str)
///   parse a JSON string.
/// - [`from_config`](Self::from_config) / [`try_from_config`](Self::try_from_config)
///   build from an already-parsed [`ThemeConfig`].
/// - [`empty_light`](Self::empty_light) gives an inert fallback.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub appearance: Appearance,
    pub version: String,
    pub author: Option<String>,

    palette_overrides: HashMap<String, Color>,
    semantic: HashMap<String, Fill>,

    /// Flat dotted tokens resolved from [`ThemeConfig::colors`].
    pub colors: HashMap<String, Fill>,

    pub components: serde_json::Value,

    pub list: ListTheme,
    pub menu: MenuTheme,
    pub table: TableTheme,

    pub radius: HashMap<String, f32>,
    pub spacing: HashMap<String, f32>,
    pub typography: TypographyConfig,
    pub shadows: HashMap<String, String>,
    pub motion: MotionConfig,
    pub z_index: HashMap<String, i32>,
}

impl Theme {
    /// Parse and resolve a theme from a JSON string. **Panics** on invalid
    /// JSON or any unknown token — see the module-level *Panic behaviour*
    /// section.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use freya_cn::theme::Theme;
    ///
    /// let theme = Theme::from_json_str(include_str!("theme.json"));
    /// let fg = theme.foreground();
    /// ```
    pub fn from_json_str(s: &str) -> Self {
        Self::try_from_json_str(s).unwrap_or_else(|e| panic!("invalid theme JSON: {e}"))
    }

    /// Non-panicking variant of [`from_json_str`](Self::from_json_str).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use freya_cn::theme::Theme;
    ///
    /// match Theme::try_from_json_str(include_str!("theme.json")) {
    ///     Ok(theme) => println!("loaded {}", theme.name),
    ///     Err(e) => eprintln!("theme failed: {e}"),
    /// }
    /// ```
    pub fn try_from_json_str(s: &str) -> Result<Self, ThemeError> {
        let cfg: ThemeConfig = serde_json::from_str(s)?;
        Self::try_from_config(cfg)
    }

    /// Build a theme from a [`ThemeConfig`]. **Panics** on any unknown
    /// token (bad hex, unknown palette step, cycle, bad structured
    /// `list` / `menu` / `table`, …).
    ///
    /// Use this when you've already parsed the JSON yourself (e.g. via
    /// `serde_json::from_value`) or built a `ThemeConfig` in code.
    pub fn from_config(cfg: ThemeConfig) -> Self {
        Self::try_from_config(cfg).unwrap_or_else(|e| panic!("invalid theme: {e}"))
    }

    /// Non-panicking variant of [`from_config`](Self::from_config).
    pub fn try_from_config(cfg: ThemeConfig) -> Result<Self, ThemeError> {
        let mut palette_overrides = HashMap::new();
        for (path, hex) in &cfg.palette {
            let color = parse_hex(hex).map_err(|e| ThemeError::TokenAt {
                path: format!("palette.{path}"),
                reason: Box::new(e),
            })?;
            palette_overrides.insert(path.clone(), color);
        }

        let semantic = resolve_semantic_map(&cfg.semantic, &palette_overrides)?;
        let colors = resolve_colors(&cfg.colors, &palette_overrides, &semantic)?;

        let list = build_list_theme(&cfg.list, &palette_overrides, &semantic, &colors)?;
        let menu = build_menu_theme(&cfg.menu, &palette_overrides, &semantic, &colors)?;
        let table = build_table_theme(&cfg.table, &palette_overrides, &semantic, &colors)?;

        Ok(Self {
            id: cfg.id,
            name: cfg.name,
            appearance: cfg.appearance,
            version: cfg.version,
            author: cfg.author,
            palette_overrides,
            semantic,
            colors,
            components: cfg.components,
            list,
            menu,
            table,
            radius: cfg.radius,
            spacing: cfg.spacing,
            typography: cfg.typography,
            shadows: cfg.shadows,
            motion: cfg.motion,
            z_index: cfg.z_index,
        })
    }

    // ---- Metadata ----

    /// Lightweight metadata — see [`ThemeInfo`].
    ///
    /// Clones the five string fields it needs and nothing else. Use this
    /// when building a list of themes for a picker; use the `Arc<Theme>`
    /// itself when you need the resolved colours / tokens.
    ///
    /// # Cost
    ///
    /// One `String::clone` per string field (`id`, `name`, `version`,
    /// `author`) plus a `Copy` of `appearance`. Doesn't clone the
    /// `semantic` map, resolved `colors`, structured `list` / `menu` /
    /// `table` themes, or `components` JSON. For a typical theme this is
    /// ~200 bytes.
    ///
    /// # Example — build a picker entry
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// let info = theme.metadata();
    ///
    /// println!("ID:       {}", info.id);
    /// println!("Name:     {}", info.name);
    /// println!("Version:  {}", info.version);
    /// println!("Author:   {}", info.author.as_deref().unwrap_or("(none)"));
    ///
    /// let appearance = match info.appearance {
    ///     freya_cn::theme::Appearance::Light => "light",
    ///     freya_cn::theme::Appearance::Dark => "dark",
    /// };
    /// println!("Mode:     {appearance}");
    /// ```
    ///
    /// # Example — round-trip through `ThemeInfo`
    ///
    /// ```no_run
    /// # use freya_cn::theme::{Appearance, Theme, ThemeInfo};
    /// # let theme = Theme::try_from_json_str(r#"{"id":"x","name":"X","appearance":"dark","semantic":{}}"#).unwrap();
    /// let info: ThemeInfo = theme.metadata();
    /// assert_eq!(info.id, theme.id);
    /// assert_eq!(info.name, theme.name);
    /// assert_eq!(info.appearance, Appearance::Dark);
    /// ```
    ///
    /// # Example — compare two themes by metadata
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let a = Theme::try_from_json_str("{}").unwrap();
    /// # let b = Theme::try_from_json_str("{}").unwrap();
    /// let same_mode = a.metadata().appearance == b.metadata().appearance;
    /// ```
    pub fn metadata(&self) -> ThemeInfo {
        ThemeInfo::from_theme(self)
    }

    // ---- Semantic lookups ----

    /// Look up a semantic token by key, returning the full `Fill`
    /// (which may be a gradient).
    ///
    /// Returns `None` if the key is absent. Gradient tokens are returned
    /// intact; use [`color`](Self::color) if you specifically need a
    /// solid `Color`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// if let Some(fill) = theme.fill("primary") {
    ///     // paint something
    /// }
    /// ```
    pub fn fill(&self, key: &str) -> Option<Fill> {
        self.semantic.get(key).cloned()
    }

    /// Look up a semantic token as a solid [`Color`].
    ///
    /// Returns `None` if the key is absent **or** if the token is a
    /// gradient. Use [`fill`](Self::fill) or the `*_fill` getters if you
    /// need to support gradients.
    pub fn color(&self, key: &str) -> Option<Color> {
        match self.semantic.get(key)? {
            Fill::Color(c) => Some(*c),
            _ => None,
        }
    }

    // ---- Flat token lookups ----

    /// Look up a flat `colors` token by dotted path
    /// (`"button.primary.background"`), returning the full `Fill`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// if let Some(fill) = theme.token_fill("button.primary.background") {
    ///     // paint something
    /// }
    /// ```
    pub fn token_fill(&self, path: &str) -> Option<Fill> {
        self.colors.get(path).cloned()
    }

    /// Look up a flat token as a solid [`Color`]. Returns `None` for
    /// gradients or missing keys.
    pub fn token_color(&self, path: &str) -> Option<Color> {
        match self.colors.get(path)? {
            Fill::Color(c) => Some(*c),
            _ => None,
        }
    }

    /// Look up a flat token as a [`ThemeToken`] — both the solid colour
    /// and the full fill. See [`ThemeToken`] for usage.
    pub fn token(&self, path: &str) -> Option<ThemeToken> {
        let background = self.colors.get(path)?.clone();
        let color = match &background {
            Fill::Color(c) => *c,
            _ => Color::from_argb(0, 0, 0, 0),
        };
        Some(ThemeToken { color, background })
    }

    // ---- Scale shortcuts ----

    /// Look up `radius.<key>`, falling back to `8.0`. Prefer
    /// [`radius_or`](Self::radius_or) when the fallback matters — this
    /// exists for compatibility with older code.
    pub fn radius(&self, key: &str) -> f32 {
        self.radius.get(key).copied().unwrap_or(8.0)
    }

    /// Look up `radius.<key>`, returning `fallback` if undefined.
    ///
    /// Used by every `rounded_*` shortcut in
    /// [`crate::styling::styled`] via `active_theme().radius_or("sm", 4.0)`.
    pub fn radius_or(&self, key: &str, fallback: f32) -> f32 {
        self.radius.get(key).copied().unwrap_or(fallback)
    }

    /// Look up `typography.fontSize.<key>` (a `(size, line_height)` pair),
    /// trying each key in order.
    ///
    /// The multiple-key form lets `text_md()` accept either `fontSize.md`
    /// or `fontSize.base` — `&["md", "base"]`.
    pub fn font_size_or(&self, keys: &[&str], fallback: f32) -> f32 {
        keys.iter()
            .filter_map(|k| self.typography.font_size.get(*k))
            .map(|(s, _)| *s)
            .next()
            .unwrap_or(fallback)
    }

    /// Look up `typography.fontWeight.<key>`, trying each key in order.
    /// Returns a CSS weight (`100 ..= 900`).
    pub fn font_weight_or(&self, keys: &[&str], fallback: u16) -> u16 {
        keys.iter()
            .filter_map(|k| self.typography.font_weight.get(*k))
            .copied()
            .next()
            .unwrap_or(fallback)
    }

    /// Split `typography.fontFamily.<key>` into a `Vec<Cow>` suitable
    /// for [`Style::font_families`].
    ///
    /// Returns `None` if the key is absent **or** if the value is empty
    /// after trimming.
    pub fn font_family_stack(&self, key: &str) -> Option<Vec<Cow<'static, str>>> {
        let raw = self.typography.font_family.get(key)?;
        let stack: Vec<Cow<'static, str>> = raw
            .split(',')
            .map(|f| f.trim())
            .filter(|f| !f.is_empty())
            .map(|f| Cow::Owned(f.to_owned()))
            .collect();
        if stack.is_empty() { None } else { Some(stack) }
    }

    /// Look up `spacing.<key>`, returning `fallback` if undefined.
    ///
    /// Drives the named `p_sm` / `gap_md` / `m_lg` / … shortcuts.
    pub fn spacing_or(&self, key: &str, fallback: f32) -> f32 {
        self.spacing.get(key).copied().unwrap_or(fallback)
    }

    /// Look up `zIndex.<key>`, returning `fallback` if undefined.
    ///
    /// Drives the `z_*` shortcuts. Falls back to the Tailwind numeric
    /// scale (`"0"`, `"10"`, …).
    pub fn z_index_or(&self, key: &str, fallback: i32) -> i32 {
        self.z_index.get(key).copied().unwrap_or(fallback)
    }

    /// Raw shadow string, e.g. `"0 4px 6px rgba(0,0,0,.1)"`.
    ///
    /// **Declarative only.** Freya's `Style` has no box-shadow field, so
    /// nothing consumes this today. Kept for future use / SVG filters.
    pub fn shadow(&self, key: &str) -> Option<&str> {
        self.shadows.get(key).map(String::as_str)
    }

    /// Motion duration in milliseconds, e.g. `Some(250)` for `"normal"`.
    ///
    /// **Declarative only** — Freya's `Style` has no transition fields.
    pub fn motion_duration(&self, key: &str) -> Option<u32> {
        self.motion.duration.get(key).copied()
    }

    /// Motion easing curve string, e.g. `Some("cubic-bezier(.4,0,.2,1)")`.
    ///
    /// **Declarative only** — see [`motion_duration`](Self::motion_duration).
    pub fn motion_easing(&self, key: &str) -> Option<&str> {
        self.motion.easing.get(key).map(String::as_str)
    }

    /// Look up `spacing.<key>`, falling back to `0.0`. Prefer
    /// [`spacing_or`](Self::spacing_or) when the fallback matters.
    pub fn spacing(&self, key: &str) -> f32 {
        self.spacing.get(key).copied().unwrap_or(0.0)
    }

    // ---- Component token access (nested `components` JSON) ----

    /// Raw access to the `components` JSON tree by dotted path.
    ///
    /// Most callers should use the typed [`component_fill`](Self::component_fill) /
    /// [`component_color`](Self::component_color) /
    /// [`component_radius`](Self::component_radius) helpers instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// let raw = theme.component("button.primary.background");
    /// if let Some(v) = raw {
    ///     println!("{}", v);
    /// }
    /// ```
    pub fn component(&self, path: &str) -> Option<&serde_json::Value> {
        let mut cur = &self.components;
        for part in path.split('.') {
            cur = cur.get(part)?;
        }
        Some(cur)
    }

    /// Resolve a component token to a solid [`Color`].
    ///
    /// Returns `None` if the path doesn't exist or its value isn't a
    /// colour token (e.g. a radius key, a nested object). **Panics** if
    /// the value looks like a colour token but can't be resolved (bad
    /// palette step, missing semantic key, invalid hex, gradient where a
    /// solid is required).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// if let Some(fg) = theme.component_color("button.primary.foreground") {
    ///     // use as text colour
    /// }
    /// ```
    pub fn component_color(&self, path: &str) -> Option<Color> {
        let v = self.component(path)?;
        let s = v.as_str()?;
        if !looks_like_color_token(s) {
            return None;
        }
        Some(
            resolve_color(s, &self.palette_overrides, &self.semantic).unwrap_or_else(|e| {
                panic!("component token `{path}` has invalid color `{s}`: {e}")
            }),
        )
    }

    /// Resolve a component token to a [`Fill`] — solid or gradient.
    ///
    /// Returns `None` if the path doesn't exist or its value isn't a
    /// colour / gradient token. **Panics** if the value looks like a
    /// token but can't be resolved.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// if let Some(bg) = theme.component_fill("button.primary.background") {
    ///     // paint as background
    /// }
    /// ```
    pub fn component_fill(&self, path: &str) -> Option<Fill> {
        let v = self.component(path)?;

        if let Some(s) = v.as_str() {
            if !looks_like_color_token(s) {
                return None;
            }
            return Some(Fill::Color(
                resolve_color(s, &self.palette_overrides, &self.semantic).unwrap_or_else(|e| {
                    panic!("component token `{path}` has invalid color `{s}`: {e}")
                }),
            ));
        }

        // Try to parse as a gradient token.
        if let Ok(tv) = serde_json::from_value::<TokenValue>(v.clone()) {
            return Some(
                resolve_fill(&tv, &self.palette_overrides, &self.semantic).unwrap_or_else(|e| {
                    panic!("component token `{path}` has invalid gradient: {e}")
                }),
            );
        }

        // Not a colour string and not a valid gradient token — return None.
        None
    }

    /// Resolve a component token to a radius value.
    ///
    /// Returns `None` if the path doesn't exist or its value isn't a
    /// radius reference. **Panics** if the value looks like a radius key
    /// but isn't present in the theme's `radius` map.
    ///
    /// # Example
    ///
    /// ```json
    /// { "components": { "button": { "radius": "lg" } } }
    /// ```
    ///
    /// With `"radius": { "lg": 8 }` in the theme, `component_radius("button.radius")`
    /// returns `Some(8.0)`.
    pub fn component_radius(&self, path: &str) -> Option<f32> {
        let v = self.component(path)?;
        if let Some(n) = v.as_f64() {
            return Some(n as f32);
        }
        if let Some(s) = v.as_str() {
            if !looks_like_radius_key(s) {
                return None;
            }
            return Some(*self.radius.get(s).unwrap_or_else(|| {
                panic!("component token `{path}` references unknown radius `{s}`")
            }));
        }
        None
    }

    // ---- State-aware accessors ----

    /// Look up a component token with per-state fallback.
    ///
    /// Resolution order:
    ///
    /// 1. `{path}.{state}.{field}` — exact state match.
    /// 2. `{path}.{alt}.{field}` for each `alt` in
    ///    [`ComponentState::fallback_chain`] — e.g. `focus-visible` falls
    ///    back to `focus`.
    /// 3. `{path}.{field}` — base value.
    ///
    /// Returns `None` if none of the three levels resolve.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::{Theme, ComponentState};
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// let hover = theme.state_fill("button.primary", ComponentState::Hover, "background");
    /// let pressed = theme.state_fill("button.primary", ComponentState::Pressed, "background");
    /// // `pressed` falls back to `button.primary.active.background` if
    /// // `button.primary.pressed.background` is absent.
    /// ```
    pub fn state_fill(&self, path: &str, state: ComponentState, field: &str) -> Option<Fill> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(f) = self.component_fill(&with_state) {
                return Some(f);
            }
            for alt in state.fallback_chain() {
                let with_alt = format!("{}.{}.{}", path, alt, field);
                if let Some(f) = self.component_fill(&with_alt) {
                    return Some(f);
                }
            }
        }
        self.component_fill(&format!("{}.{}", path, field))
    }

    /// Like [`state_fill`](Self::state_fill) but returns a solid
    /// [`Color`]. Returns `None` for gradient tokens at any level.
    pub fn state_color(&self, path: &str, state: ComponentState, field: &str) -> Option<Color> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(c) = self.component_color(&with_state) {
                return Some(c);
            }
            for alt in state.fallback_chain() {
                let with_alt = format!("{}.{}.{}", path, alt, field);
                if let Some(c) = self.component_color(&with_alt) {
                    return Some(c);
                }
            }
        }
        self.component_color(&format!("{}.{}", path, field))
    }

    /// Like [`state_fill`](Self::state_fill) but returns a radius value.
    pub fn state_radius(&self, path: &str, state: ComponentState, field: &str) -> Option<f32> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(r) = self.component_radius(&with_state) {
                return Some(r);
            }
            for alt in state.fallback_chain() {
                let with_alt = format!("{}.{}.{}", path, alt, field);
                if let Some(r) = self.component_radius(&with_alt) {
                    return Some(r);
                }
            }
        }
        self.component_radius(&format!("{}.{}", path, field))
    }

    // ---- Validation ----

    /// Soft-validate the theme, returning human-readable warning strings.
    ///
    /// Called automatically from [`ThemeRegistry::load`]; the warnings are
    /// printed to stderr. Nothing is rejected — a theme is still usable
    /// with missing tokens or low contrast, it just may look bad.
    ///
    /// Checks performed:
    ///
    /// - All required semantic tokens (`background`, `foreground`,
    ///   `primary`, `primaryForeground`, …) are present.
    /// - Every `(fg, bg)` pair in [`CONTRAST_PAIRS`] meets WCAG AA
    ///   (4.5:1) — **only** for solid tokens; gradient tokens are
    ///   silently skipped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::Theme;
    /// # let theme = Theme::try_from_json_str("{}").unwrap();
    /// for warning in theme.validate() {
    ///     eprintln!("theme warning: {warning}");
    /// }
    /// ```
    pub fn validate(&self) -> Vec<String> {
        const REQUIRED: &[&str] = &[
            "background",
            "foreground",
            "card",
            "cardForeground",
            "popover",
            "popoverForeground",
            "primary",
            "primaryForeground",
            "secondary",
            "secondaryForeground",
            "muted",
            "mutedForeground",
            "accent",
            "accentForeground",
            "destructive",
            "destructiveForeground",
            "border",
            "input",
            "ring",
        ];
        let mut warnings = Vec::new();

        for key in REQUIRED {
            if !self.semantic.contains_key(*key) {
                warnings.push(format!("missing required semantic token: {key}"));
            }
        }

        for (fg_key, bg_key) in CONTRAST_PAIRS {
            if let (Some(a), Some(b)) = (self.color(fg_key), self.color(bg_key)) {
                let ratio = contrast_ratio(a, b);
                if ratio < 4.5 {
                    warnings.push(format!(
                        "contrast {} vs {}: {:.2} < 4.5",
                        fg_key, bg_key, ratio
                    ));
                }
            }
        }

        warnings
    }

    /// Inert fallback used when [`active_theme`] is called before any
    /// theme has been installed.
    ///
    /// Has:
    ///
    /// - No semantic tokens (so getters return their palette fallbacks).
    /// - No flat `colors` tokens.
    /// - Full default [`ListTheme`] / [`MenuTheme`] / [`TableTheme`].
    /// - Tailwind's radius scale (`none` … `full`, plus `default`).
    /// - Empty spacing / typography / shadows / motion / z-index maps.
    ///
    /// Safe to hand to components — nothing panics, everything renders
    /// with generic defaults. Callers that want a real theme install one
    /// via [`set_active_theme`].
    pub fn empty_light() -> Self {
        Self {
            id: "fallback-light".into(),
            name: "Fallback Light".into(),
            appearance: Appearance::Light,
            version: "0".into(),
            author: None,
            palette_overrides: HashMap::new(),
            semantic: HashMap::new(),
            colors: HashMap::new(),
            components: serde_json::Value::Null,
            list: default_list_theme(),
            menu: default_menu_theme(),
            table: default_table_theme(),
            radius: [
                ("none", 0.0),
                ("xs", 2.0),
                ("sm", 4.0),
                ("md", 6.0),
                ("lg", 8.0),
                ("xl", 12.0),
                ("2xl", 16.0),
                ("3xl", 24.0),
                ("4xl", 32.0),
                ("full", 9999.0),
                ("default", 4.0),
            ]
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect(),
            spacing: HashMap::new(),
            typography: TypographyConfig::default(),
            shadows: HashMap::new(),
            motion: MotionConfig::default(),
            z_index: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------
// Heuristics for component-token values
// ---------------------------------------------------------------------

/// `true` when `s` looks like it was intended as a colour token.
///
/// Used to decide whether a component-token value that fails resolution
/// should panic (yes — clearly a colour) or be silently treated as
/// non-colour data (no — radius key, nested object, arbitrary text).
///
/// Recognises: bare hex (`#abc`, `abcdef`), braced refs (`{blue.500}`,
/// `{semantic.x}/50`), and `transparent`.
fn looks_like_color_token(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if s.eq_ignore_ascii_case("transparent") {
        return true;
    }
    if s.starts_with('#') {
        return true;
    }
    if s.starts_with('{') && s.ends_with('}') {
        return true;
    }
    let len = s.len();
    (len == 3 || len == 4 || len == 6 || len == 8) && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// `true` when `s` looks like a radius key (`sm`, `md`, `2xl`, …).
fn looks_like_radius_key(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

// ---------------------------------------------------------------------
// Default component themes
// ---------------------------------------------------------------------

/// Default [`ListTheme`] — generic black-on-transparent with neutral
/// hover / selection. Used when the theme doesn't specify `list` fields
/// or their overlays.
fn default_list_theme() -> ListTheme {
    let bg = Fill::Color(transparent().to_color().unwrap());
    let fg = Fill::Color(black().to_color().unwrap());
    ListTheme {
        background: bg.clone(),
        border: Fill::Color(neutral_300().to_color().unwrap()),
        row_background: bg.clone(),
        row_hover: Fill::Color(neutral_100().to_color().unwrap()),
        row_selected: Fill::Color(neutral_200().to_color().unwrap()),
        row_active: Fill::Color(neutral_200().to_color().unwrap()),
        row_foreground: fg.clone(),
        row_selected_foreground: fg,
        row_disabled_foreground: Fill::Color(neutral_400().to_color().unwrap()),
        row_height: 32.0,
        row_padding_x: 12.0,
        row_padding_y: 6.0,
        radius: 6.0,
        gap: 0.0,
    }
}

/// Default [`MenuTheme`] — transparent container, neutral hover.
fn default_menu_theme() -> MenuTheme {
    let bg = Fill::Color(transparent().to_color().unwrap());
    let fg = Fill::Color(black().to_color().unwrap());
    MenuTheme {
        background: bg.clone(),
        border: Fill::Color(neutral_300().to_color().unwrap()),
        item_hover: Fill::Color(neutral_100().to_color().unwrap()),
        item_selected: Fill::Color(neutral_200().to_color().unwrap()),
        item_foreground: fg,
        separator: Fill::Color(neutral_200().to_color().unwrap()),
        radius: 6.0,
        item_padding_x: 10.0,
        item_padding_y: 6.0,
    }
}

/// Default [`TableTheme`] — transparent container, neutral header.
fn default_table_theme() -> TableTheme {
    let bg = Fill::Color(transparent().to_color().unwrap());
    let fg = Fill::Color(black().to_color().unwrap());
    TableTheme {
        background: bg.clone(),
        border: Fill::Color(neutral_300().to_color().unwrap()),
        header_background: Fill::Color(neutral_100().to_color().unwrap()),
        header_foreground: fg.clone(),
        row_hover: Fill::Color(neutral_100().to_color().unwrap()),
        row_selected: Fill::Color(neutral_200().to_color().unwrap()),
        cell_padding_x: 12.0,
        cell_padding_y: 8.0,
        radius: 6.0,
    }
}

// ---------------------------------------------------------------------
// Resolvers for structured component themes
// ---------------------------------------------------------------------

/// Resolve a single `Option<TokenValue>` field of a structured component
/// config, returning `fallback` when absent and wrapping errors with the
/// dotted JSON `path`.
fn resolve_opt_fill(
    value: &Option<TokenValue>,
    semantic: &HashMap<String, Fill>,
    overrides: &HashMap<String, Color>,
    fallback: Fill,
    path: &str,
) -> Result<Fill, ThemeError> {
    match value {
        Some(tv) => resolve_fill(tv, overrides, semantic).map_err(|e| ThemeError::TokenAt {
            path: path.into(),
            reason: Box::new(e),
        }),
        None => Ok(fallback),
    }
}

/// Resolve the entire flat `colors` map, wrapping errors with
/// `colors.{key}` paths.
fn resolve_colors(
    raw: &HashMap<String, TokenValue>,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<HashMap<String, Fill>, ThemeError> {
    let mut out = HashMap::with_capacity(raw.len());
    for (path, tv) in raw {
        let fill = resolve_fill(tv, overrides, semantic).map_err(|e| ThemeError::TokenAt {
            path: format!("colors.{path}"),
            reason: Box::new(e),
        })?;
        out.insert(path.clone(), fill);
    }
    Ok(out)
}

/// Build a [`ListTheme`] from its structured config, then overlay any
/// matching entries under `colors` (which take precedence).
///
/// Panics on unresolvable tokens via `ThemeError::TokenAt { path:
/// "list.…" }`.
fn build_list_theme(
    cfg: &ListConfig,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
    colors: &HashMap<String, Fill>,
) -> Result<ListTheme, ThemeError> {
    let fallback = default_list_theme();

    let mut theme = ListTheme {
        background: resolve_opt_fill(
            &cfg.background,
            semantic,
            overrides,
            fallback.background,
            "list.background",
        )?,
        border: resolve_opt_fill(
            &cfg.border,
            semantic,
            overrides,
            fallback.border,
            "list.border",
        )?,
        row_background: resolve_opt_fill(
            &cfg.row_background,
            semantic,
            overrides,
            fallback.row_background,
            "list.rowBackground",
        )?,
        row_hover: resolve_opt_fill(
            &cfg.row_hover,
            semantic,
            overrides,
            fallback.row_hover,
            "list.rowHover",
        )?,
        row_selected: resolve_opt_fill(
            &cfg.row_selected,
            semantic,
            overrides,
            fallback.row_selected,
            "list.rowSelected",
        )?,
        row_active: resolve_opt_fill(
            &cfg.row_active,
            semantic,
            overrides,
            fallback.row_active,
            "list.rowActive",
        )?,
        row_foreground: resolve_opt_fill(
            &cfg.row_foreground,
            semantic,
            overrides,
            fallback.row_foreground,
            "list.rowForeground",
        )?,
        row_selected_foreground: resolve_opt_fill(
            &cfg.row_selected_foreground,
            semantic,
            overrides,
            fallback.row_selected_foreground,
            "list.rowSelectedForeground",
        )?,
        row_disabled_foreground: resolve_opt_fill(
            &cfg.row_disabled_foreground,
            semantic,
            overrides,
            fallback.row_disabled_foreground,
            "list.rowDisabledForeground",
        )?,
        row_height: cfg.row_height.unwrap_or(fallback.row_height),
        row_padding_x: cfg.row_padding_x.unwrap_or(fallback.row_padding_x),
        row_padding_y: cfg.row_padding_y.unwrap_or(fallback.row_padding_y),
        radius: cfg.radius.unwrap_or(fallback.radius),
        gap: cfg.gap.unwrap_or(fallback.gap),
    };

    if let Some(fill) = colors.get("list.background") {
        theme.background = fill.clone();
    }
    if let Some(fill) = colors.get("list.border") {
        theme.border = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.background") {
        theme.row_background = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.hover") {
        theme.row_hover = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.selected") {
        theme.row_selected = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.active") {
        theme.row_active = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.foreground") {
        theme.row_foreground = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.selected.foreground") {
        theme.row_selected_foreground = fill.clone();
    }
    if let Some(fill) = colors.get("list.row.disabled.foreground") {
        theme.row_disabled_foreground = fill.clone();
    }

    Ok(theme)
}

/// Build a [`MenuTheme`] from its structured config, then overlay
/// matching `colors` entries (which take precedence).
fn build_menu_theme(
    cfg: &MenuConfig,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
    colors: &HashMap<String, Fill>,
) -> Result<MenuTheme, ThemeError> {
    let fallback = default_menu_theme();

    let mut theme = MenuTheme {
        background: resolve_opt_fill(
            &cfg.background,
            semantic,
            overrides,
            fallback.background,
            "menu.background",
        )?,
        border: resolve_opt_fill(
            &cfg.border,
            semantic,
            overrides,
            fallback.border,
            "menu.border",
        )?,
        item_hover: resolve_opt_fill(
            &cfg.item_hover,
            semantic,
            overrides,
            fallback.item_hover,
            "menu.itemHover",
        )?,
        item_selected: resolve_opt_fill(
            &cfg.item_selected,
            semantic,
            overrides,
            fallback.item_selected,
            "menu.itemSelected",
        )?,
        item_foreground: resolve_opt_fill(
            &cfg.item_foreground,
            semantic,
            overrides,
            fallback.item_foreground,
            "menu.itemForeground",
        )?,
        separator: resolve_opt_fill(
            &cfg.separator,
            semantic,
            overrides,
            fallback.separator,
            "menu.separator",
        )?,
        radius: cfg.radius.unwrap_or(fallback.radius),
        item_padding_x: cfg.item_padding_x.unwrap_or(fallback.item_padding_x),
        item_padding_y: cfg.item_padding_y.unwrap_or(fallback.item_padding_y),
    };

    if let Some(fill) = colors.get("menu.background") {
        theme.background = fill.clone();
    }
    if let Some(fill) = colors.get("menu.border") {
        theme.border = fill.clone();
    }
    if let Some(fill) = colors.get("menu.item.hover") {
        theme.item_hover = fill.clone();
    }
    if let Some(fill) = colors.get("menu.item.selected") {
        theme.item_selected = fill.clone();
    }
    if let Some(fill) = colors.get("menu.item.foreground") {
        theme.item_foreground = fill.clone();
    }
    if let Some(fill) = colors.get("menu.separator") {
        theme.separator = fill.clone();
    }

    Ok(theme)
}

/// Build a [`TableTheme`] from its structured config, then overlay
/// matching `colors` entries (which take precedence).
fn build_table_theme(
    cfg: &TableConfig,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
    colors: &HashMap<String, Fill>,
) -> Result<TableTheme, ThemeError> {
    let fallback = default_table_theme();

    let mut theme = TableTheme {
        background: resolve_opt_fill(
            &cfg.background,
            semantic,
            overrides,
            fallback.background,
            "table.background",
        )?,
        border: resolve_opt_fill(
            &cfg.border,
            semantic,
            overrides,
            fallback.border,
            "table.border",
        )?,
        header_background: resolve_opt_fill(
            &cfg.header_background,
            semantic,
            overrides,
            fallback.header_background,
            "table.headerBackground",
        )?,
        header_foreground: resolve_opt_fill(
            &cfg.header_foreground,
            semantic,
            overrides,
            fallback.header_foreground,
            "table.headerForeground",
        )?,
        row_hover: resolve_opt_fill(
            &cfg.row_hover,
            semantic,
            overrides,
            fallback.row_hover,
            "table.rowHover",
        )?,
        row_selected: resolve_opt_fill(
            &cfg.row_selected,
            semantic,
            overrides,
            fallback.row_selected,
            "table.rowSelected",
        )?,
        cell_padding_x: cfg.cell_padding_x.unwrap_or(fallback.cell_padding_x),
        cell_padding_y: cfg.cell_padding_y.unwrap_or(fallback.cell_padding_y),
        radius: cfg.radius.unwrap_or(fallback.radius),
    };

    if let Some(fill) = colors.get("table.background") {
        theme.background = fill.clone();
    }
    if let Some(fill) = colors.get("table.border") {
        theme.border = fill.clone();
    }
    if let Some(fill) = colors.get("table.header.background") {
        theme.header_background = fill.clone();
    }
    if let Some(fill) = colors.get("table.header.foreground") {
        theme.header_foreground = fill.clone();
    }
    if let Some(fill) = colors.get("table.row.hover") {
        theme.row_hover = fill.clone();
    }
    if let Some(fill) = colors.get("table.row.selected") {
        theme.row_selected = fill.clone();
    }

    Ok(theme)
}

// ---------------------------------------------------------------------
// Named semantic getters
// ---------------------------------------------------------------------

/// Unwrap a palette [`StyledColor`] into a [`Color`].
///
/// Every palette function in [`crate::styling::colors`] returns a solid
/// colour, so this never panics. Exists to keep the `getters!` macro
/// from repeating `.to_color().expect(..)` on every fallback.
#[inline]
fn palette_solid(c: StyledColor) -> Color {
    c.to_color()
        .expect("theme getter fallback must be a solid palette color")
}

/// Generate `fn key() -> Color` and `fn key_fill() -> Fill` getters for
/// each semantic token.
///
/// Each getter resolves the semantic token if the theme defines it, and
/// otherwise falls back to the given palette function. The `_fill`
/// variant preserves gradients; the plain variant returns the solid
/// colour (or the palette fallback for gradient tokens — see [`Theme::color`]).
macro_rules! getters {
    ($($name:ident => $key:literal | $fallback:expr),* $(,)?) => {
        impl Theme {
            $(
                /// Semantic colour getter. Falls back to the palette
                /// value if the theme doesn't define the token.
                #[inline] pub fn $name(&self) -> Color {
                    self.color($key)
                        .unwrap_or_else(|| palette_solid($fallback))
                }
                paste! {
                    /// Semantic fill getter — preserves gradients. Falls
                    /// back to the palette value as a solid fill if the
                    /// theme doesn't define the token.
                    #[inline] pub fn [<$name _fill>](&self) -> Fill {
                        self.fill($key)
                            .unwrap_or_else(|| Fill::Color(palette_solid($fallback)))
                    }
                }
            )*
        }
    };
}

getters! {
    background                  => "background"                  | transparent(),
    foreground                  => "foreground"                  | black(),
    card                        => "card"                        | transparent(),
    card_foreground             => "cardForeground"              | black(),
    popover                     => "popover"                     | transparent(),
    popover_foreground          => "popoverForeground"           | black(),
    primary                     => "primary"                     | black(),
    primary_foreground          => "primaryForeground"           | white(),
    secondary                   => "secondary"                   | transparent(),
    secondary_foreground        => "secondaryForeground"         | black(),
    muted                       => "muted"                       | transparent(),
    muted_foreground            => "mutedForeground"             | neutral_500(),
    accent                      => "accent"                      | transparent(),
    accent_foreground           => "accentForeground"            | black(),
    destructive                 => "destructive"                 | red_600(),
    destructive_foreground      => "destructiveForeground"       | white(),
    border                      => "border"                      | neutral_300(),
    input                       => "input"                       | neutral_300(),
    ring                        => "ring"                        | blue_500(),
    disabled                    => "disabled"                    | transparent(),
    disabled_foreground         => "disabledForeground"          | neutral_400(),
    sidebar                     => "sidebar"                     | transparent(),
    sidebar_foreground          => "sidebarForeground"           | black(),
    sidebar_primary             => "sidebarPrimary"              | black(),
    sidebar_primary_foreground  => "sidebarPrimaryForeground"    | white(),
    sidebar_accent              => "sidebarAccent"               | transparent(),
    sidebar_accent_foreground   => "sidebarAccentForeground"     | black(),
    sidebar_border              => "sidebarBorder"               | neutral_300(),
    sidebar_ring                => "sidebarRing"                 | blue_500(),
    chart_1                     => "chart1"                      | neutral_500(),
    chart_2                     => "chart2"                      | neutral_500(),
    chart_3                     => "chart3"                      | neutral_500(),
    chart_4                     => "chart4"                      | neutral_500(),
    chart_5                     => "chart5"                      | neutral_500(),
    success                     => "success"                     | green_500(),
    success_foreground          => "successForeground"           | white(),
    warning                     => "warning"                     | yellow_500(),
    warning_foreground          => "warningForeground"           | black(),
    info                        => "info"                        | blue_500(),
    info_foreground             => "infoForeground"              | white(),
    placeholder                 => "placeholder"                 | neutral_400(),
    link                        => "link"                        | blue_600(),
    link_hover                  => "linkHover"                   | blue_700(),
}

// ---------------------------------------------------------------------
// Palette lookup
// ---------------------------------------------------------------------

/// Resolve a `{group.step}` palette reference against the built-in
/// Tailwind scale exported by [`crate::styling::colors`].
///
/// Returns `None` for unknown families / steps. `{white}` / `{black}`
/// are special-cased: the step part is ignored (`{white.anything}` →
/// `white()`).
fn lookup_palette_styled(path: &str) -> Option<StyledColor> {
    let (group, step) = path.split_once('.')?;

    match (group, step) {
        ("white", _) => return Some(white()),
        ("black", _) => return Some(black()),
        _ => {}
    }

    macro_rules! try_family {
        ($fam:ident) => {
            paste! {
                match step {
                    "50"  => Some(crate::styling::colors::[<$fam _ 50>]()),
                    "100" => Some(crate::styling::colors::[<$fam _ 100>]()),
                    "200" => Some(crate::styling::colors::[<$fam _ 200>]()),
                    "300" => Some(crate::styling::colors::[<$fam _ 300>]()),
                    "400" => Some(crate::styling::colors::[<$fam _ 400>]()),
                    "500" => Some(crate::styling::colors::[<$fam _ 500>]()),
                    "600" => Some(crate::styling::colors::[<$fam _ 600>]()),
                    "700" => Some(crate::styling::colors::[<$fam _ 700>]()),
                    "800" => Some(crate::styling::colors::[<$fam _ 800>]()),
                    "900" => Some(crate::styling::colors::[<$fam _ 900>]()),
                    "950" => Some(crate::styling::colors::[<$fam _ 950>]()),
                    _ => None,
                }
            }
        };
    }

    match group {
        "stone" => try_family!(stone),
        "mauve" => try_family!(mauve),
        "olive" => try_family!(olive),
        "mist" => try_family!(mist),
        "taupe" => try_family!(taupe),
        "slate" => try_family!(slate),
        "gray" => try_family!(gray),
        "zinc" => try_family!(zinc),
        "neutral" => try_family!(neutral),
        "red" => try_family!(red),
        "orange" => try_family!(orange),
        "amber" => try_family!(amber),
        "yellow" => try_family!(yellow),
        "lime" => try_family!(lime),
        "green" => try_family!(green),
        "emerald" => try_family!(emerald),
        "teal" => try_family!(teal),
        "cyan" => try_family!(cyan),
        "sky" => try_family!(sky),
        "blue" => try_family!(blue),
        "indigo" => try_family!(indigo),
        "violet" => try_family!(violet),
        "purple" => try_family!(purple),
        "fuchsia" => try_family!(fuchsia),
        "pink" => try_family!(pink),
        "rose" => try_family!(rose),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Resolution helpers
// ---------------------------------------------------------------------

/// Parse a hex string (`#rrggbb`, `#rrggbbaa`, `#rgb`, `#rgba`, with or
/// without `#`) into a [`Color`].
fn parse_hex(s: &str) -> Result<Color, ThemeError> {
    let (r, g, b, a) =
        crate::utils::parse_hex_rgba(s).ok_or_else(|| ThemeError::InvalidHex(s.into()))?;
    Ok(Color::from_argb(a, r, g, b))
}

/// Apply an optional `/NN` alpha suffix to an already-solid [`Color`],
/// **overwriting** its existing alpha.
fn apply_alpha_pct(c: Color, pct: Option<f32>) -> Color {
    match pct {
        Some(p) => {
            let a = (p.clamp(0.0, 100.0) / 100.0 * 255.0).round() as u8;
            c.with_a(a)
        }
        None => c,
    }
}

/// Resolve a [`StyledColor`] palette function to a [`Color`], applying
/// the optional `/NN` alpha suffix. Same "overwrite" semantics as
/// [`apply_alpha_pct`] so `{blue.500}/50` and `"#3b82f680"`-derived
/// `{…}/50` behave identically (Tailwind convention).
fn with_alpha_pct(styled: StyledColor, pct: Option<f32>) -> Result<Color, ThemeError> {
    let c = styled
        .to_color()
        .ok_or_else(|| ThemeError::NotASolidColor("gradient".into()))?;
    Ok(match pct {
        Some(p) => {
            let a = (p.clamp(0.0, 100.0) / 100.0 * 255.0).round() as u8;
            c.with_a(a)
        }
        None => c,
    })
}

/// Resolve a single colour token string (with optional `/NN` alpha
/// suffix).
///
/// Handles: `transparent`, `{semantic.*}`, `{palette.override}`,
/// `{family.step}`, and raw hex. See the module-level *Token reference
/// syntax* table for the full grammar.
///
/// # Errors
///
/// - [`ThemeError::InvalidPaletteRef`] — `/NN` suffix that isn't a
///   number.
/// - [`ThemeError::NotASolidColor`] — `{semantic.*}` that resolves to a
///   gradient.
/// - [`ThemeError::UnknownPaletteStep`] — `{family.step}` not found in
///   the built-in palette or `palette` overrides.
/// - [`ThemeError::InvalidHex`] — malformed hex literal.
pub fn resolve_color(
    s: &str,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<Color, ThemeError> {
    let s = s.trim();

    if s.eq_ignore_ascii_case("transparent") {
        return Ok(Color::from_argb(0, 0, 0, 0));
    }

    if let Some(inner) = s.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
        let (path, alpha_pct) = match inner.split_once('/') {
            Some((p, a)) => (
                p,
                Some(
                    a.parse::<f32>()
                        .map_err(|_| ThemeError::InvalidPaletteRef(s.into()))?,
                ),
            ),
            None => (inner, None),
        };

        if let Some(key) = path.strip_prefix("semantic.") {
            let base = match semantic.get(key) {
                Some(Fill::Color(c)) => *c,
                _ => return Err(ThemeError::NotASolidColor(path.into())),
            };
            return Ok(apply_alpha_pct(base, alpha_pct));
        }

        if let Some(c) = overrides.get(path) {
            return Ok(apply_alpha_pct(*c, alpha_pct));
        }

        let styled = lookup_palette_styled(path)
            .ok_or_else(|| ThemeError::UnknownPaletteStep(path.into()))?;
        return with_alpha_pct(styled, alpha_pct);
    }

    parse_hex(s)
}

/// Resolve every gradient stop in `stops` to `(Color, at)` tuples.
///
/// Stops support the same string grammar as solid tokens, including
/// `{semantic.*}` and `{family.step}` references.
fn build_stops(
    stops: &[Stop],
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<Vec<(Color, f32)>, ThemeError> {
    stops
        .iter()
        .map(|s| Ok((resolve_color(&s.color, overrides, semantic)?, s.at)))
        .collect()
}

/// Resolve a [`TokenValue`] (solid or gradient) into a [`Fill`].
///
/// # Errors
///
/// Propagates any error from [`resolve_color`] / [`build_stops`].
pub fn resolve_fill(
    token: &TokenValue,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<Fill, ThemeError> {
    match token {
        TokenValue::Color(s) => Ok(Fill::Color(resolve_color(s, overrides, semantic)?)),
        TokenValue::Linear { linear } => {
            let mut g = LinearGradient::new().angle(linear.angle);
            for (color, at) in build_stops(&linear.stops, overrides, semantic)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
        TokenValue::Radial { radial } => {
            let _ = (radial.center, radial.radius);
            let mut g = RadialGradient::new();
            for (color, at) in build_stops(&radial.stops, overrides, semantic)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
        TokenValue::Conic { conic } => {
            let _ = conic.center;
            let mut g = ConicGradient::new().angle(conic.angle);
            for (color, at) in build_stops(&conic.stops, overrides, semantic)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
    }
}

/// Compute the WCAG contrast ratio between two colours.
///
/// Returns a number in `1.0 ..= 21.0`; `4.5` is the WCAG AA threshold
/// for body text, `3.0` for large text and UI elements.
fn contrast_ratio(a: Color, b: Color) -> f32 {
    fn lum(c: Color) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }
    let (l1, l2) = {
        let x = lum(a);
        let y = lum(b);
        if x > y { (x, y) } else { (y, x) }
    };
    (l1 + 0.05) / (l2 + 0.05)
}

/// Resolve the `semantic` map in dependency order, so tokens can
/// reference each other (`"muted": "{semantic.foreground}/50"`).
///
/// # Algorithm
///
/// Repeatedly try to resolve every unresolved token against the
/// partially-built map:
///
/// - Tokens that resolve are inserted into the result map.
/// - Tokens whose only remaining dependency is another not-yet-resolved
///   token fail with [`ThemeError::NotASolidColor`] and are retried on
///   the next pass.
/// - A full pass with zero progress means the remaining tokens form a
///   cycle or reference a missing key — reported as
///   [`ThemeError::Cycle`].
/// - Any other error (bad hex, unknown palette step, …) aborts
///   immediately with [`ThemeError::TokenAt`] naming the offending token.
///
/// # Complexity
///
/// `O(n * d)` where `n` is the number of tokens and `d` the longest
/// dependency chain. Typical themes have `n < 50` and `d < 3`, so this
/// runs in microseconds.
fn resolve_semantic_map(
    raw: &HashMap<String, TokenValue>,
    overrides: &HashMap<String, Color>,
) -> Result<HashMap<String, Fill>, ThemeError> {
    let mut resolved: HashMap<String, Fill> = HashMap::with_capacity(raw.len());

    loop {
        if resolved.len() == raw.len() {
            return Ok(resolved);
        }

        let mut progress = false;
        for (key, val) in raw {
            if resolved.contains_key(key) {
                continue;
            }
            match resolve_fill(val, overrides, &resolved) {
                Ok(fill) => {
                    resolved.insert(key.clone(), fill);
                    progress = true;
                }
                // `{semantic.X}` not yet resolved → retry next pass.
                Err(e) if is_retryable(&e) => {}
                // Any other error is a genuine problem — surface with path.
                Err(e) => {
                    return Err(ThemeError::TokenAt {
                        path: format!("semantic.{key}"),
                        reason: Box::new(e),
                    });
                }
            }
        }

        if !progress {
            let mut stuck: Vec<String> = raw
                .keys()
                .filter(|k| !resolved.contains_key(*k))
                .cloned()
                .collect();
            stuck.sort();
            return Err(ThemeError::Cycle(stuck));
        }
    }
}

// ---------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------

/// In-memory store of loaded themes, keyed by their `id`.
///
/// Usually created and managed by [`load_theme_registry`] /
/// [`load_theme_registry_from_dir`], which also install the initial
/// theme and provide a [`ThemeHandle`] to Freya's context. Use
/// `ThemeRegistry` directly for testing, CLI tooling, or custom app
/// wiring.
///
/// # Discovery
///
/// The registry exposes both full [`Arc<Theme>`](Theme) access
/// ([`get`](Self::get), [`list`](Self::list), [`active`](Self::active))
/// and cheap metadata access ([`theme_info`](Self::theme_info),
/// [`info_list`](Self::info_list), [`active_info`](Self::active_info))
/// via [`ThemeInfo`]. Prefer the `info_*` methods for UI lists — they
/// avoid cloning the whole theme.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use freya_cn::theme::ThemeRegistry;
///
/// let registry = ThemeRegistry::new();
/// registry.load(include_str!("theme.json"));
/// registry.load(include_str!("theme-dark.json"));
///
/// registry.set_active("theme-dark").unwrap();
///
/// // Full theme.
/// let active = registry.active().unwrap();
/// println!("active theme: {}", active.name);
///
/// // Just metadata.
/// for info in registry.info_list() {
///     println!("{} ({})", info.name, info.id);
/// }
/// ```
#[derive(Default)]
pub struct ThemeRegistry {
    themes: RwLock<HashMap<String, Arc<Theme>>>,
    active_id: RwLock<Option<String>>,
}

impl ThemeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a theme from JSON. **Panics** on invalid JSON or any unknown
    /// token.
    ///
    /// If the registry was empty, the newly loaded theme becomes active.
    /// Otherwise the previously-active theme stays active. Validation
    /// warnings are printed to stderr.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let registry = ThemeRegistry::new();
    /// registry.load(include_str!("themes/dark.json"));
    /// ```
    pub fn load(&self, json: &str) -> Arc<Theme> {
        self.try_load(json)
            .unwrap_or_else(|e| panic!("failed to load theme: {e}"))
    }

    /// Non-panicking variant of [`load`](Self::load).
    pub fn try_load(&self, json: &str) -> Result<Arc<Theme>, ThemeError> {
        let cfg: ThemeConfig = serde_json::from_str(json)?;
        let theme = Arc::new(Theme::try_from_config(cfg)?);
        for w in theme.validate() {
            eprintln!("theme '{}': {}", theme.id, w);
        }
        let id = theme.id.clone();
        self.themes
            .write()
            .unwrap()
            .insert(id.clone(), theme.clone());
        let mut active = self.active_id.write().unwrap();
        if active.is_none() {
            *active = Some(id);
        }
        Ok(theme)
    }

    /// Load every `.json` in `dir`. **Panics** on I/O errors or any
    /// unknown token in any file.
    ///
    /// Files are loaded in filesystem order; the first one becomes active
    /// if the registry was empty. Non-`.json` files are ignored.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let registry = ThemeRegistry::new();
    /// let n = registry.load_dir(std::path::Path::new("themes"));
    /// println!("loaded {n} themes");
    /// ```
    pub fn load_dir(&self, dir: &std::path::Path) -> usize {
        self.try_load_dir(dir)
            .unwrap_or_else(|e| panic!("failed to load theme directory: {e}"))
    }

    /// Non-panicking variant of [`load_dir`](Self::load_dir).
    pub fn try_load_dir(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        let mut count = 0usize;
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let s = std::fs::read_to_string(&path)?;
                self.try_load(&s)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Re-read every `.json` in `dir` and update the registry in place.
    /// **Panics** on I/O errors or any unknown token.
    ///
    /// New files are added; existing IDs are overwritten; removed files
    /// are **not** pruned (call [`remove`](Self::remove) first if you
    /// need to prune). Uses the same semantics as
    /// [`load_dir`](Self::load_dir), so hot-reload in a running app is
    /// supported via [`ThemeHandle::reload_dir`].
    pub fn reload_dir(&self, dir: &std::path::Path) -> usize {
        self.try_reload_dir(dir)
            .unwrap_or_else(|e| panic!("failed to reload theme directory: {e}"))
    }

    /// Non-panicking variant of [`reload_dir`](Self::reload_dir).
    pub fn try_reload_dir(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        self.try_load_dir(dir)
    }

    // ---- Retrieval ----

    /// Look up a registered theme by ID.
    pub fn get(&self, id: &str) -> Option<Arc<Theme>> {
        self.themes.read().unwrap().get(id).cloned()
    }

    /// The currently-active theme, if any.
    pub fn active(&self) -> Option<Arc<Theme>> {
        let id = self.active_id.read().unwrap().clone()?;
        self.get(&id)
    }

    /// Set the active theme by ID.
    ///
    /// Returns [`ThemeError::UnknownTheme`] if the ID isn't registered.
    /// Does **not** install the theme into the thread-local — callers who
    /// need that (i.e. Freya components) should use
    /// [`ThemeHandle::set_active`] instead, which does both.
    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        if !self.themes.read().unwrap().contains_key(id) {
            return Err(ThemeError::UnknownTheme(id.into()));
        }
        *self.active_id.write().unwrap() = Some(id.into());
        Ok(())
    }

    /// Remove a theme from the registry.
    ///
    /// If the removed theme was active, `active_id` is cleared. Returns
    /// the removed theme if it existed.
    pub fn remove(&self, id: &str) -> Option<Arc<Theme>> {
        let t = self.themes.write().unwrap().remove(id);
        if let Some(t) = &t {
            let mut active = self.active_id.write().unwrap();
            if active.as_deref() == Some(t.id.as_str()) {
                *active = None;
            }
        }
        t
    }

    /// All registered themes. Order is unspecified (HashMap iteration).
    pub fn list(&self) -> Vec<Arc<Theme>> {
        self.themes.read().unwrap().values().cloned().collect()
    }

    /// All registered theme IDs. Order is unspecified.
    pub fn ids(&self) -> Vec<String> {
        self.themes.read().unwrap().keys().cloned().collect()
    }

    // ---- Discovery / metadata ----

    /// Number of loaded themes.
    ///
    /// O(1) — reads the map's length without iterating.
    ///
    /// # Example — empty vs populated
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let reg = ThemeRegistry::new();
    /// assert_eq!(reg.len(), 0);
    ///
    /// reg.load(include_str!("themes/light.json"));
    /// assert_eq!(reg.len(), 1);
    ///
    /// reg.load(include_str!("themes/dark.json"));
    /// assert_eq!(reg.len(), 2);
    /// ```
    ///
    /// # Example — guard against empty pickers
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// if reg.len() == 0 {
    ///     eprintln!("warning: no themes loaded, UI will use fallback colours");
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.themes.read().unwrap().len()
    }

    /// `true` when no themes are loaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let mut_hint = ThemeRegistry::new();
    /// assert!(mut_hint.is_empty());
    ///
    /// // Guard a hot-reload directory that hasn't been created yet.
    /// if mut_hint.is_empty() {
    ///     eprintln!("no themes loaded yet — install a fallback");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.themes.read().unwrap().is_empty()
    }

    /// `true` when `id` is registered.
    ///
    /// Unlike [`get`](Self::get), this doesn't clone the `Arc<Theme>` —
    /// use it as a fast existence check (e.g. before calling
    /// [`set_active`](Self::set_active) on user input).
    ///
    /// # Example — validate before switching
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// // `set_active` would error on unknown IDs anyway, but this lets
    /// // you avoid the error path (and the `Arc<Theme>` clone inside).
    /// if reg.contains("default-dark") {
    ///     reg.set_active("default-dark").unwrap();
    /// }
    /// ```
    ///
    /// # Example — guard a reload that skips the current theme
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// # let user_id = "user-custom";
    /// if !reg.contains(user_id) {
    ///     eprintln!("theme `{user_id}` was removed; falling back to default");
    ///     reg.set_active("default-light").unwrap();
    /// }
    /// ```
    pub fn contains(&self, id: &str) -> bool {
        self.themes.read().unwrap().contains_key(id)
    }

    /// The registry's tracked active ID, if any.
    ///
    /// This is the ID last passed to [`set_active`](Self::set_active) or
    /// the ID of the first-loaded theme. It may differ from what's
    /// currently *rendered* if the caller went through
    /// [`ThemeHandle::set_active`] instead of `ThemeRegistry::set_active`
    /// (the handle keeps its own reactive state). For most callers,
    /// prefer [`ThemeHandle::active_id`].
    ///
    /// # Example — inspect the registry without Freya
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let reg = ThemeRegistry::new();
    /// reg.load(include_str!("themes/light.json"));
    /// reg.load(include_str!("themes/dark.json"));
    ///
    /// // First-loaded theme becomes active.
    /// assert_eq!(reg.active_id().as_deref(), Some("default-light"));
    ///
    /// reg.set_active("default-dark").unwrap();
    /// assert_eq!(reg.active_id().as_deref(), Some("default-dark"));
    /// ```
    ///
    /// # Example — persist the current ID before exiting
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// if let Some(id) = reg.active_id() {
    ///     std::fs::write("last-theme.txt", id).ok();
    /// }
    /// ```
    pub fn active_id(&self) -> Option<String> {
        self.active_id.read().unwrap().clone()
    }

    /// Lightweight metadata for a single theme.
    ///
    /// Same as `get(id).map(|t| t.metadata())` but clearly indicates
    /// intent. Returns `None` for unknown IDs.
    ///
    /// # Example — single lookup
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let reg = ThemeRegistry::new();
    /// reg.load(include_str!("themes/light.json"));
    ///
    /// if let Some(info) = reg.theme_info("default-light") {
    ///     println!("{} — v{} ({:?})", info.name, info.version, info.appearance);
    /// }
    /// ```
    ///
    /// # Example — avoid a full theme clone in a list
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// // BAD: clones the entire resolved theme (semantic map, colors,
    /// // components JSON, …) just to print its name.
    /// for theme in reg.list() {
    ///     println!("{}", theme.name);
    /// }
    ///
    /// // GOOD: clones five small fields.
    /// for info in reg.info_list() {
    ///     println!("{}", info.name);
    /// }
    /// ```
    pub fn theme_info(&self, id: &str) -> Option<ThemeInfo> {
        self.themes.read().unwrap().get(id).map(|t| t.metadata())
    }

    /// Lightweight metadata for every loaded theme, **sorted by `name`
    /// (case-insensitive), then `id`**.
    ///
    /// The sort makes this suitable for theme pickers directly — no
    /// caller-side ordering required, and the order is stable across runs
    /// regardless of `HashMap` iteration order.
    ///
    /// # Example — print a sorted list
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// let reg = ThemeRegistry::new();
    /// reg.load(include_str!("themes/light.json"));
    /// reg.load(include_str!("themes/dark.json"));
    ///
    /// for info in reg.info_list() {
    ///     println!("{} ({})", info.name, info.id);
    /// }
    /// // FreyaCN Dark (default-dark)
    /// // FreyaCN Light (default-light)
    /// ```
    ///
    /// # Example — filter by appearance
    ///
    /// ```no_run
    /// # use freya_cn::theme::{Appearance, ThemeRegistry};
    /// # let reg = ThemeRegistry::new();
    /// let dark_themes: Vec<_> = reg
    ///     .info_list()
    ///     .into_iter()
    ///     .filter(|i| i.appearance == Appearance::Dark)
    ///     .collect();
    /// ```
    ///
    /// # Example — search by name (case-insensitive substring)
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// # let query = "dark";
    /// let needle = query.to_lowercase();
    /// let matches: Vec<_> = reg
    ///     .info_list()
    ///     .into_iter()
    ///     .filter(|i| i.name.to_lowercase().contains(&needle))
    ///     .collect();
    /// ```
    ///
    /// # Example — build a display label
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// let labels: Vec<String> = reg
    ///     .info_list()
    ///     .iter()
    ///     .map(|i| format!("{} — {}", i.name, i.id))
    ///     .collect();
    /// ```
    pub fn info_list(&self) -> Vec<ThemeInfo> {
        let themes = self.themes.read().unwrap();
        let mut out: Vec<ThemeInfo> = themes.values().map(|t| t.metadata()).collect();
        out.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        out
    }

    /// Lightweight metadata for the currently-active theme, if any.
    ///
    /// Returns `None` if no theme has been loaded yet, or after the
    /// active theme was removed via [`remove`](Self::remove).
    ///
    /// # Example — status line
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// match reg.active_info() {
    ///     Some(info) => println!("Active: {} ({:?})", info.name, info.appearance),
    ///     None => println!("No theme active"),
    /// }
    /// ```
    ///
    /// # Example — persist on switch
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeRegistry;
    /// # let reg = ThemeRegistry::new();
    /// reg.set_active("default-dark").unwrap();
    /// if let Some(info) = reg.active_info() {
    ///     std::fs::write("last-theme-id.txt", &info.id).ok();
    /// }
    /// ```
    pub fn active_info(&self) -> Option<ThemeInfo> {
        self.active().map(|t| t.metadata())
    }

    // ---- Static constructors ----

    /// Parse a theme without registering it. **Panics** on invalid JSON
    /// or any unknown token.
    ///
    /// Convenience wrapper for [`Theme::from_json_str`].
    pub fn from_json(json: &str) -> Theme {
        Theme::from_json_str(json)
    }

    /// Non-panicking variant of [`from_json`](Self::from_json).
    pub fn try_from_json(json: &str) -> Result<Theme, ThemeError> {
        Theme::try_from_json_str(json)
    }

    /// Build a registry from a directory. **Panics** on I/O errors or any
    /// unknown token.
    pub fn from_dir(dir: &std::path::Path) -> Self {
        Self::try_from_dir(dir).unwrap_or_else(|e| panic!("failed to load theme directory: {e}"))
    }

    /// Non-panicking variant of [`from_dir`](Self::from_dir).
    pub fn try_from_dir(dir: &std::path::Path) -> Result<Self, ThemeError> {
        let registry = Self::new();
        registry.try_load_dir(dir)?;
        Ok(registry)
    }

    /// Insert an already-built [`Theme`] into the registry.
    ///
    /// If the registry was empty, the inserted theme becomes active.
    pub fn insert(&self, theme: Theme) -> Arc<Theme> {
        let arc = Arc::new(theme);
        self.themes
            .write()
            .unwrap()
            .insert(arc.id.clone(), arc.clone());
        let mut active = self.active_id.write().unwrap();
        if active.is_none() {
            *active = Some(arc.id.clone());
        }
        arc
    }

    /// Alias for [`reload_dir`](Self::reload_dir).
    pub fn reload(&self, dir: &std::path::Path) -> usize {
        self.reload_dir(dir)
    }
}

// ---------------------------------------------------------------------
// Freya context integration
// ---------------------------------------------------------------------

/// Reactive handle to the currently-active theme.
///
/// Returned by [`load_theme_registry`] / [`load_theme_registry_from_dir`]
/// and provided into Freya's context so descendants can reach it via
/// [`use_theme_handle`]. Combines:
///
/// - [`registry`](Self::registry) — the full set of loaded themes.
/// - [`active`](Self::active) — Freya's `State<Arc<Theme>>`, which
///   components read via [`use_cn_theme`] / [`use_theme`].
///
/// # Discovery
///
/// `ThemeHandle` mirrors the registry's discovery methods
/// ([`theme_infos`](Self::theme_infos), [`active_info`](Self::active_info),
/// [`is_active`](Self::is_active), [`contains`](Self::contains), …) so
/// pickers and settings panels don't need to reach through `registry`
/// directly.
///
/// # Example
///
/// ```no_run
/// # use freya_cn::theme::*;
/// # use freya::prelude::*;
/// #[component]
/// fn App() -> Element {
///     let handle = load_theme_registry(include_str!("themes/light.json"));
///     rsx! {
///         button {
///             onclick: move |_| { let _ = handle.set_active("dark"); },
///             "Switch to dark"
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct ThemeHandle {
    /// The full registry of loaded themes.
    pub registry: Arc<ThemeRegistry>,
    /// Freya's reactive state pointing at the active theme.
    pub active: State<Arc<Theme>>,
}

impl ThemeHandle {
    // ---- Switching themes ----

    /// Switch the active theme to `id`.
    ///
    /// Updates:
    ///
    /// 1. The registry's `active_id`.
    /// 2. The thread-local `ACTIVE_THEME` (read by every `rounded_*`,
    ///    `text_*`, etc. shortcut).
    /// 3. Freya's `State<Arc<Theme>>` — subscribes every component that
    ///    calls [`use_cn_theme`] / [`use_theme`].
    ///
    /// Returns [`ThemeError::UnknownTheme`] if `id` isn't registered.
    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        self.registry.set_active(id)?;
        if let Some(t) = self.registry.active() {
            set_active_theme(t.clone());
            let mut active = self.active;
            active.set(t);
        }
        Ok(())
    }

    /// Re-read every `.json` in `dir` and refresh the active theme.
    ///
    /// New files are added, existing IDs are overwritten, removed files
    /// are **not** pruned. If the active theme's ID disappears, the
    /// existing state remains pointed at the old (in-memory) version.
    ///
    /// **Panics** on I/O errors or any unknown token in any reloaded
    /// file. Useful for hot-reloading theme files during development.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freya_cn::theme::*;
    /// # let handle: ThemeHandle = todo!();
    /// let n = handle.reload_dir(std::path::Path::new("themes"));
    /// println!("reloaded {n} themes");
    /// ```
    pub fn reload_dir(&self, dir: &std::path::Path) -> usize {
        let n = self.registry.reload_dir(dir);
        let prev_id = self.active.read().id.clone();
        if let Some(t) = self.registry.get(&prev_id) {
            set_active_theme(t.clone());
            let mut active = self.active;
            active.set(t);
        }
        n
    }

    // ---- Reading the currently-active theme ----

    /// Clone the currently-active theme out of Freya's reactive state.
    ///
    /// **Subscribes** the calling component: re-renders whenever
    /// [`set_active`](Self::set_active) or [`reload_dir`](Self::reload_dir)
    /// changes the theme. This is the reactive form of
    /// [`crate::theme::use_theme`] — same result, but usable from a handle
    /// you already hold.
    ///
    /// # Example — read resolved colours
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let theme = handle.current();
    /// let bg = theme.background_fill();
    /// let fg = theme.foreground();
    /// let primary = theme.primary_fill();
    /// // …use bg / fg / primary in a rect
    /// ```
    ///
    /// # Example — branch on appearance
    ///
    /// ```no_run
    /// # use freya_cn::theme::{Appearance, ThemeHandle};
    /// # let handle: ThemeHandle = todo!();
    /// let icon = match handle.current().appearance {
    ///     Appearance::Light => "☀",
    ///     Appearance::Dark => "☾",
    /// };
    /// ```
    ///
    /// # Example — trigger re-render on switch
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// fn render_label(handle: &ThemeHandle) -> String {
    ///     // Subscribes — the calling component re-renders whenever
    ///     // `set_active` changes the theme.
    ///     handle.current().name.clone()
    /// }
    /// ```
    pub fn current(&self) -> Arc<Theme> {
        self.active.read().clone()
    }

    /// ID of the currently-rendered theme.
    ///
    /// Reads from Freya's reactive state (what's actually on screen)
    /// rather than the registry's tracked `active_id`. Prefer this over
    /// [`ThemeRegistry::active_id`] in UI code.
    ///
    /// **Subscribes** — re-renders on theme switch.
    ///
    /// # Example — show the current ID
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// println!("Now showing: {}", handle.active_id());
    /// ```
    ///
    /// # Example — conditional rendering
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// if handle.active_id() == "default-dark" {
    ///     // render the sun icon (to switch back to light)
    /// }
    /// ```
    pub fn active_id(&self) -> String {
        self.active.read().id.clone()
    }

    /// Lightweight metadata for the currently-rendered theme.
    ///
    /// **Subscribes** — re-renders on theme switch. This is the method a
    /// theme picker should call once per render to determine which entry
    /// gets the checkmark.
    ///
    /// # Example — picker with checkmark
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let active = handle.active_info();
    ///
    /// for info in handle.theme_infos() {
    ///     let label = if info.id == active.id {
    ///         format!("✓ {}", info.name)
    ///     } else {
    ///         format!("  {}", info.name)
    ///     };
    ///     println!("{label}");
    /// }
    /// ```
    ///
    /// # Example — status bar text
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let info = handle.active_info();
    /// println!("Theme: {} v{}", info.name, info.version);
    /// ```
    pub fn active_info(&self) -> ThemeInfo {
        self.active.read().metadata()
    }

    /// `true` when `id` is the currently-rendered theme.
    ///
    /// Handy for theme-picker checkmarks. **Subscribes** — re-renders on
    /// theme switch.
    ///
    /// # Example — checkmark per entry
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// for info in handle.theme_infos() {
    ///     let marker = if handle.is_active(&info.id) { "●" } else { "○" };
    ///     println!("{marker} {}", info.name);
    /// }
    /// ```
    ///
    /// # Example — conditional styling
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// # let id = "default-dark";
    /// let bg = if handle.is_active(id) {
    ///     "rgba(0,0,0,0.1)"
    /// } else {
    ///     "transparent"
    /// };
    /// ```
    pub fn is_active(&self, id: &str) -> bool {
        self.active_id() == id
    }

    // ---- Enumerating loaded themes ----

    /// All loaded themes as `Arc<Theme>`.
    ///
    /// Same as [`ThemeRegistry::list`], but accessible from the handle.
    /// **Order is unspecified** (HashMap iteration) — for a stable UI
    /// list, use [`theme_infos`](Self::theme_infos) instead.
    ///
    /// **Does not subscribe** — call it from a component that also reads
    /// [`current`](Self::current) if you need re-renders.
    ///
    /// # Example — access resolved tokens for all themes
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// for theme in handle.themes() {
    ///     // Full resolved theme — can read `.primary()`, `.list.row_hover`, …
    ///     println!("{}: {}", theme.name, theme.primary());
    /// }
    /// ```
    ///
    /// # Example — filter by semantic token value
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let dark_by_bg: Vec<_> = handle
    ///     .themes()
    ///     .into_iter()
    ///     .filter(|t| t.background().r() < 128)
    ///     .collect();
    /// ```
    ///
    /// # Example — avoid when you only need metadata
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// // BAD: clones the entire resolved theme per entry.
    /// for theme in handle.themes() {
    ///     println!("{}", theme.name);
    /// }
    ///
    /// // GOOD: clones five small fields per entry.
    /// for info in handle.theme_infos() {
    ///     println!("{}", info.name);
    /// }
    /// ```
    pub fn themes(&self) -> Vec<Arc<Theme>> {
        self.registry.list()
    }

    /// All loaded themes as lightweight [`ThemeInfo`], sorted by name.
    ///
    /// **Does not subscribe** to registry changes — call it inside a
    /// component that also reads `use_cn_theme()` (or uses
    /// [`current`](Self::current)) if you need re-renders on
    /// [`reload_dir`](Self::reload_dir). For most UIs this is fine: the
    /// theme list is stable after initial load, and
    /// [`set_active`](Self::set_active) already updates the reactive
    /// state that pickers read via [`active_info`](Self::active_info).
    ///
    /// # Example — full picker render
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// fn render_picker(handle: &ThemeHandle) -> Vec<String> {
    ///     // Subscribing read for re-renders on switch.
    ///     let active = handle.active_info();
    ///
    ///     // Non-subscribing list — cheap, stable order.
    ///     handle
    ///         .theme_infos()
    ///         .into_iter()
    ///         .map(|info| {
    ///             let marker = if info.id == active.id { "✓" } else { " " };
    ///             format!("{marker} {} ({})", info.name, info.id)
    ///         })
    ///         .collect()
    /// }
    /// ```
    ///
    /// # Example — count by appearance
    ///
    /// ```no_run
    /// # use freya_cn::theme::{Appearance, ThemeHandle};
    /// # let handle: ThemeHandle = todo!();
    /// let dark = handle
    ///     .theme_infos()
    ///     .iter()
    ///     .filter(|i| i.appearance == Appearance::Dark)
    ///     .count();
    /// println!("{dark} dark theme(s) available");
    /// ```
    ///
    /// # Example — search input
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// # let query = "solarized";
    /// let needle = query.to_lowercase();
    /// let results: Vec<_> = handle
    ///     .theme_infos()
    ///     .into_iter()
    ///     .filter(|i| {
    ///         i.name.to_lowercase().contains(&needle)
    ///             || i.id.to_lowercase().contains(&needle)
    ///     })
    ///     .collect();
    /// ```
    pub fn theme_infos(&self) -> Vec<ThemeInfo> {
        self.registry.info_list()
    }

    /// IDs of every loaded theme (unspecified order).
    ///
    /// Same as [`ThemeRegistry::ids`]. For a sorted list, use
    /// [`theme_infos`](Self::theme_infos).
    ///
    /// # Example — feed a native `<select>`
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let options: Vec<(String, String)> = handle
    ///     .theme_ids()
    ///     .into_iter()
    ///     .map(|id| {
    ///         let name = handle.theme_info(&id).map(|i| i.name).unwrap_or(id.clone());
    ///         (id, name)
    ///     })
    ///     .collect();
    /// ```
    ///
    /// # Example — check whether every file in a directory loaded
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let loaded: std::collections::HashSet<_> = handle.theme_ids().into_iter().collect();
    /// for expected in ["default-light", "default-dark"] {
    ///     assert!(loaded.contains(expected), "missing theme: {expected}");
    /// }
    /// ```
    pub fn theme_ids(&self) -> Vec<String> {
        self.registry.ids()
    }

    /// Metadata for a single theme by ID.
    ///
    /// Same as `handle.registry.theme_info(id)` but accessible from the
    /// handle. Returns `None` for unknown IDs.
    ///
    /// # Example — tooltip content
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// # let id = "default-dark";
    /// if let Some(info) = handle.theme_info(id) {
    ///     let tooltip = match info.author {
    ///         Some(a) => format!("{} — by {}", info.name, a),
    ///         None => info.name.clone(),
    ///     };
    ///     println!("{tooltip}");
    /// }
    /// ```
    ///
    /// # Example — validate a saved preference
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// # let saved_id = "user-custom";
    /// if handle.theme_info(saved_id).is_none() {
    ///     eprintln!("saved theme `{saved_id}` no longer exists");
    /// }
    /// ```
    pub fn theme_info(&self, id: &str) -> Option<ThemeInfo> {
        self.registry.theme_info(id)
    }

    /// `true` when `id` is registered.
    ///
    /// # Example — safe switch
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// # let user_input = "default-dark";
    /// if handle.contains(user_input) {
    ///     handle.set_active(user_input).unwrap();
    /// } else {
    ///     eprintln!("unknown theme: {user_input}");
    /// }
    /// ```
    pub fn contains(&self, id: &str) -> bool {
        self.registry.contains(id)
    }

    /// Number of loaded themes.
    ///
    /// # Example — show a badge
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// let count = handle.theme_count();
    /// println!("{count} theme(s) available");
    /// ```
    pub fn theme_count(&self) -> usize {
        self.registry.len()
    }

    /// `true` when no themes are loaded.
    ///
    /// # Example — guard the picker
    ///
    /// ```no_run
    /// # use freya_cn::theme::ThemeHandle;
    /// # let handle: ThemeHandle = todo!();
    /// if handle.is_empty() {
    ///     eprintln!("no themes loaded — install at least one via load_theme_registry");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }
}

/// Load a theme from a JSON string and install it as the app's active
/// theme.
///
/// Must be called inside a Freya component — uses [`use_hook`] so the
/// registry and state survive re-renders. Subsequent calls to
/// [`use_cn_theme`] / [`use_theme`] pick up the active theme.
///
/// Also provides both `State<Arc<Theme>>` and [`ThemeHandle`] into
/// Freya's context, so descendants can call [`use_theme_handle`] to
/// reach the registry (theme pickers, settings panels, …).
///
/// **Panics** on invalid JSON or any unknown token.
///
/// # Example
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::load_theme_registry;
/// #[component]
/// fn App() -> Element {
///     let handle = load_theme_registry(include_str!("themes/light.json"));
///     rsx! { /* … */ }
/// }
/// ```
pub fn load_theme_registry(initial_json: &str) -> ThemeHandle {
    use_hook(|| {
        let registry = Arc::new(ThemeRegistry::new());
        let theme = registry.load(initial_json);
        set_active_theme(theme.clone());
        let active = State::create(theme);
        provide_context(active);

        let handle = ThemeHandle { registry, active };
        // Also provide the handle, so any descendant can call
        // `use_theme_handle()` and reach the registry (theme pickers,
        // settings panels, CLI introspection).
        provide_context(handle.clone());
        handle
    })
}

/// Alias for [`load_theme_registry`] matching hook-style naming conventions.
pub use load_theme_registry as use_init_theme_registry;

/// Load every `.json` file in `dir` and install one of them as the
/// active theme.
///
/// Like [`load_theme_registry`] but loads from a directory. The first
/// file (filesystem order) becomes active. **Panics** on I/O errors,
/// empty directory, or any unknown token.
///
/// # Example
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::load_theme_registry_from_dir;
/// #[component]
/// fn App() -> Element {
///     let handle = load_theme_registry_from_dir(
///         std::path::Path::new("themes"),
///     );
///     rsx! { /* … */ }
/// }
/// ```
pub fn load_theme_registry_from_dir(dir: &std::path::Path) -> ThemeHandle {
    use_hook(|| {
        let registry = Arc::new(ThemeRegistry::new());
        registry.load_dir(dir);
        let active = registry.active().expect("no theme was loaded");
        set_active_theme(active.clone());
        let state = State::create(active);
        provide_context(state);

        let handle = ThemeHandle {
            registry,
            active: state,
        };
        provide_context(handle.clone());
        handle
    })
}

/// Subscribe to the currently active theme.
///
/// Re-renders the calling component whenever
/// [`ThemeHandle::set_active`] / [`ThemeHandle::reload_dir`] changes the
/// theme. Panics if called outside a component tree — use
/// [`get_cn_theme_or_default`] for a non-panicking variant.
///
/// # Example
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::use_cn_theme;
/// #[component]
/// fn MyCard() -> Element {
///     let theme = use_cn_theme();
///     let bg = theme.read().card_fill();
///     rsx! { rect { background: "{bg:?}" } }
/// }
/// ```
pub fn use_cn_theme() -> State<Arc<Theme>> {
    use_consume::<State<Arc<Theme>>>()
}

/// Convenience: clone the active theme out of the reactive state.
///
/// Same as `use_cn_theme().read().clone()` but shorter. Subscribes to
/// the state for you, so the calling component re-renders on theme
/// switches.
///
/// # Example
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::use_theme;
/// #[component]
/// fn MyButton() -> Element {
///     let theme = use_theme();
///     let bg = theme.primary_fill();
///     rsx! { rect { background: "{bg:?}" } }
/// }
/// ```
pub fn use_theme() -> Arc<Theme> {
    use_cn_theme().read().clone()
}

/// Consume the [`ThemeHandle`] provided by
/// [`load_theme_registry`] / [`load_theme_registry_from_dir`].
///
/// Any component below the provider can call this to reach the full
/// registry — useful for theme pickers, settings panels, and anything
/// that needs to enumerate or switch themes.
///
/// Panics if called outside a component tree that called
/// `load_theme_registry*`. Use [`try_consume_context`] directly if you
/// need a non-panicking variant.
///
/// # Example — a full picker component
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::{use_theme_handle, Appearance, ThemeInfo};
/// #[component]
/// fn ThemePicker() -> Element {
///     let handle = use_theme_handle();
///
///     // Subscribes — re-renders on `set_active`.
///     let active = handle.active_info();
///     // Cheap list, stable order.
///     let infos = handle.theme_infos();
///
///     rsx! {
///         rect {
///             direction: "column",
///             gap: "4",
///             for info in infos {
///                 {
///                     let is_current = info.id == active.id;
///                     let icon = match info.appearance {
///                         Appearance::Light => "☀",
///                         Appearance::Dark => "☾",
///                     };
///                     let label = format!(
///                         "{} {} {}",
///                         if is_current { "✓" } else { " " },
///                         icon,
///                         info.name,
///                     );
///                     let id = info.id.clone();
///                     let handle = handle.clone();
///                     rsx! {
///                         rect {
///                             key: "{id}",
///                             padding: "8",
///                             background: if is_current { "rgba(0,0,0,0.1)" } else { "transparent" },
///                             onclick: move |_| { let _ = handle.set_active(&id); },
///                             label { "{label}" }
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// # Example — settings panel that persists the choice
///
/// ```no_run
/// # use freya::prelude::*;
/// # use freya_cn::theme::use_theme_handle;
/// #[component]
/// fn ThemeSettings() -> Element {
///     let handle = use_theme_handle();
///     let active = handle.active_info();
///
///     let on_switch = move |id: String| {
///         if handle.contains(&id) {
///             let _ = handle.set_active(&id);
///             // Persist the ID so the next launch restores it.
///             let _ = std::fs::write("theme.txt", &id);
///         }
///     };
///
///     rsx! {
///         label { "Current theme: {active.name} v{active.version}" }
///         label { "Author: {active.author.clone().unwrap_or_default()}" }
///         // …list rendered here, calling `on_switch(id)` per entry…
///     }
/// }
/// ```
pub fn use_theme_handle() -> ThemeHandle {
    use_consume::<ThemeHandle>()
}

/// Read the active theme without requiring a Freya context, falling back
/// to [`Theme::empty_light`] when none is provided.
///
/// Use this in components that may be rendered outside a Freya provider
/// (tests, isolated previews). Subscribes when a provider is present.
pub fn get_cn_theme_or_default() -> Readable<Arc<Theme>> {
    try_consume_context::<State<Arc<Theme>>>()
        .map(|s| s.into())
        .unwrap_or_else(|| Arc::new(Theme::empty_light()).into())
}
