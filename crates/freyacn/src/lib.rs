//! # FreyaCN
//!
//! Styling and theming for [Freya](https://github.com/marc2332/freya) UIs.
//!
//! FreyaCN is built on two ideas:
//!
//! 1. **Theme-first** — every visual decision (color, spacing, radius,
//!    typography, motion) is resolved through a [`Theme`] loaded at runtime
//!    from JSON. Styles name a *token*, never a hex code.
//! 2. **Tailwind-inspired** — utilities are composable, either as fluent
//!    builder methods on [`StyledExt`] or as parsed class strings via
//!    [`StyledString`]. Same `Style` underneath either way.
//!
//! ## Layers
//!
//! | Layer   | Item | What it does |
//! |---------|------|--------------|
//! | Theme   | [`Theme`], [`ThemeRegistry`], [`ThemeHandle`], [`ComponentState`] | JSON design tokens; palette refs, gradients, component- and state-scoped tokens; thread-local active theme. |
//! | Data    | [`Style`] | One flat struct covering every paint / typography / layout attribute. |
//! | Builder | [`StyledExt`] | Fluent `self -> Self` utilities on anything that owns a `Style`; `ThemedExt` adds token-path helpers. |
//! | Parser  | [`StyledString`] | Tailwind class strings that fan into the same `Style` mutations. |
//! | State   | [`StyleVariant`] / [`StyleVariants`] | Pseudo-state patches (`hover:`, `focus:`, …) stored *on* the style. |
//! | Palette | [`StyledColor`] and friends | Tailwind colors, gradients, shaders — all convertible to [`Fill`]. |
//!
//! ## Quick start
//!
//! ```
//! use freyacn::prelude::*;
//!
//! // 1. Install a theme from JSON. The registry validates it, installs
//! //    it as the thread-local active theme, and provides a reactive
//! //    `State<Arc<Theme>>` to the current subtree.
//! let themes = use_init_theme_registry(include_str!("../themes/default-dark.json"));
//!
//! // 2. Every element now starts from the theme's baseline — no
//! //    per-element wiring required.
//! let style = Style::default()
//!     .style("p-4 rounded-lg bg-card hover:bg-muted");
//!
//! // 3. Read a token anywhere below the provider.
//! let state = use_cn_theme();
//! let bg    = state.read().background();       // Color
//! let bg_f  = state.read().background_fill();  // Fill (solid or gradient)
//!
//! // 4. Or ask the theme for a specific component token with state fallback.
//! let hover = state.read()
//!     .state_fill("button.primary", ComponentState::Hover, "background");
//!
//! // 5. Switch themes at runtime.
//! themes.set_active("default-light").unwrap();
//! ```
//!
//! ## Theming
//!
//! A theme is a single JSON document. Only `id`, `name`, `appearance`, and
//! `semantic` are required; everything else is optional with sensible
//! defaults.
//!
//! ```
//! {
//!   "id": "default-dark",
//!   "name": "Default Dark",
//!   "appearance": "dark",
//!   "semantic": {
//!     "background": "{neutral.950}",
//!     "foreground": "{neutral.50}",
//!     "primary":    "{neutral.50}",
//!     "border":     "{neutral.50}/10"
//!   },
//!   "components": {
//!     "button": {
//!       "primary": {
//!         "background": "{semantic.primary}",
//!         "hover": {
//!             "background": "{neutral.200}"
//!         }
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! - **Palette references** — `{blue.500}`, `{neutral.950}`, `{red.600}` resolve
//!   against the full Tailwind scale exported by [`crate::styling::colors`].
//! - **Alpha suffixes** — `{neutral.50}/10` is white at 10 % opacity.
//! - **Gradients** — `linear`, `radial`, and `conic` are first-class tokens.
//! - **Components** — arbitrary nested paths (`button.primary.background`);
//!   resolved lazily with `component_color` / `component_fill`.
//! - **States** — `hover`, `active`, `focus`, `focus-visible`, `disabled`,
//!   `checked`, `selected`, `open`, … fall back to the base token.
//!   Precedence is the caller's responsibility.
//!
//! `ThemeRegistry` supports loading from a JSON string, a directory, or
//! programmatically, and re-reading the directory for hot reload. `ThemeHandle`
//! adds the reactive half: switching themes updates both the thread-local
//! active theme (so `Style::default()` picks up new values) and the
//! `State<Arc<Theme>>` subscribers.
//!
//! ## The one invariant
//!
//! **Rebuild the base [`Style`] from props every render; apply
//! [`StyleVariants`] to a *clone* for that pass only.** Never write an
//! applied style back into state. Variant teardown is the absence of an
//! entry in the next render's live set — there is no "un-hover" callback,
//! and precedence between overlapping variants is fixed by
//! [`styling::StyleVariant::priority`].
//!
//! See [`styling`] for the full explainer, the primitive-side recipe, and
//! the parser grammar.
//!
//! [`Theme`]: crate::theme::Theme
//! [`ThemeRegistry`]: crate::theme::ThemeRegistry
//! [`ThemeHandle`]: crate::theme::ThemeHandle
//! [`ComponentState`]: crate::theme::ComponentState
//! [`StyledExt`]: crate::styling::StyledExt
//! [`StyledString`]: crate::styling::StyledString
//! [`StyledColor`]: crate::styling::StyledColor
//! [`Style`]: crate::styling::Style
//! [`StyleVariant`]: crate::styling::StyleVariant
//! [`StyleVariants`]: crate::styling::StyleVariants
//! [`Fill`]: freya_core::prelude::Fill

extern crate self as freyacn;

pub mod cn;
pub mod styling;
pub mod theme;
pub mod utils;

pub mod prelude {
    #[doc(inline)]
    pub use crate::cn;

    // ---- styling ----
    #[doc(inline)]
    pub use crate::styling::{
        Style, StyleVariant, StyleVariants, StyledColor, StyledExt, StyledString, ThemedExt,
    };

    // ---- theme ----
    #[doc(inline)]
    pub use crate::theme::*;

    // ---- utilities ----
    #[doc(inline)]
    pub use crate::utils::*;

    // ---- Freya ----
    #[doc(inline)]
    pub use freya::icons::lucide::{image, radio, timer, *};
    #[doc(inline)]
    pub use freya_core::element::EventHandlerType;
    #[doc(inline)]
    pub use freya_core::events::name::EventName;

    // ---- misc ----
    #[doc(inline)]
    pub use rustc_hash::FxHashMap;
}

pub mod base {
    #[doc(inline)]
    pub use freya::prelude::*;
}
