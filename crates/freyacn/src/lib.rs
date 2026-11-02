//! > ⚠️ **Pre-release.** The public API is not yet stable. Breaking
//! > changes may land in any `0.x` release.
//!
//! Styling and theming for [Freya](https://github.com/marc2332/freya) UIs.
//!
//! FreyaCN gives Freya a small design system: a JSON-driven theme, a
//! Tailwind-like utility vocabulary, and a set of pre-built elements that
//! all speak the same styling language. Every visual decision flows
//! through one flat [`Style`] struct, so a rule written once works
//! everywhere — on a bare `div()`, on a `button()`, or on a custom element
//! you write yourself.
//!
//! [TOC]
//!
//! # Philosophy
//!
//! Freya itself is deliberately style-agnostic: each primitive has its own
//! props (`ButtonProps`, `TextProps`, …) and there is no shared styling
//! surface between them. That's the right foundation for a rendering
//! engine, but it makes building a coherent UIs tedious — every element
//! re-invents its own way to say "16px of padding" or "1px border in
//! gray-200".
//!
//! FreyaCN sits on top with two orthogonal ideas:
//!
//! 1. **Theme-first.** Every visual decision — color, spacing, radius,
//!    typography, motion — resolves through a [`Theme`] loaded at runtime
//!    from JSON. Styles name a *token* (`{neutral.950}`), never a hex
//!    code. Change the JSON, change the whole app.
//!
//! 2. **Tailwind-inspired.** Utilities are composable, either as fluent
//!    builder methods (`.px_4()`, `.rounded_md()`, `.hover(..)`) or as
//!    parsed class strings (`.style("px-4 rounded-md hover:bg-muted")`).
//!    Same [`Style`] under the hood either way; pick whichever reads
//!    better at the call site.
//!
//! The result is that a rule written once transfers unchanged to every
//! element that inherits from the same base — no per-primitive adaptation,
//! no re-inventing `padding` on every new component.
//!
//! # Quick start
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
//!     .style("p-4 rounded-lg bg-card hover:bg-muted border border-border");
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
//! For a real element tree see [`elements`] — every element constructor
//! (`div()`, `text(..)`, `button()`, …) accepts the same styling chain.
//!
//! # Layers
//!
//! Six layers, each independently useful:
//!
//! | Layer   | Item | What it does |
//! |---------|------|--------------|
//! | Theme   | [`Theme`], [`ThemeRegistry`], [`ThemeHandle`], [`ComponentState`] | JSON design tokens; palette refs, gradients, component- and state-scoped tokens; thread-local active theme. |
//! | Data    | [`Style`] | One flat struct covering every paint / typography / border / layout attribute. |
//! | Builder | [`StyledExt`] | Fluent `self -> Self` utilities on anything that owns a `Style`; `ThemedExt` adds token-path helpers. |
//! | Parser  | [`StyledString`] | Tailwind class strings that fan into the same `Style` mutations. |
//! | State   | [`StyleVariant`] / [`StyleVariants`] | Pseudo-state patches (`hover:`, `focus:`, …) stored *on* the style. |
//! | Palette | [`StyledColor`] and friends | Tailwind colors, gradients, shaders — all convertible to [`Fill`]. |
//!
//! Two supporting modules sit underneath the six layers and are only
//! interesting if you're writing a new element — see [`styling`] for the
//! full breakdown.
//!
//! # Theming
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
//! ## Token syntax
//!
//! - **Palette references** — `{blue.500}`, `{neutral.950}`, `{red.600}`
//!   resolve against the full Tailwind scale exported by
//!   [`crate::styling::colors`]. Any name from the palette is valid.
//! - **Alpha suffixes** — `{neutral.50}/10` is white at 10% opacity.
//!   Combine any palette reference with `/0` through `/100`.
//! - **Gradients** — `linear`, `radial`, and `conic` are first-class
//!   tokens. See [`crate::theme`] for the JSON shape.
//! - **Components** — arbitrary nested paths (`button.primary.background`,
//!   `card.header.border`). Resolved lazily with `component_color` /
//!   `component_fill`; no schema validation on the leaf keys.
//! - **States** — `hover`, `active`, `focus`, `focus-visible`, `disabled`,
//!   `checked`, `selected`, `open`, `closed`, `expanded`, `collapsed`,
//!   `highlighted`, `dragging` are reserved. Each falls back to the base
//!   token when the state isn't present. Precedence among live states is
//!   the caller's responsibility (see [Precedence](#precedence-and-the-disabled-gate)).
//!
//! ## Loading
//!
//! [`ThemeRegistry`] supports:
//!
//! - Loading from a JSON string via [`ThemeRegistry::load`].
//! - Loading from a directory of `*.json` files via [`ThemeRegistry::load_dir`].
//! - Hot reload by re-reading the directory via [`ThemeRegistry::reload_dir`].
//! - Lookup by id via [`ThemeRegistry::get`], active-theme selection via
//!   [`ThemeRegistry::set_active`], and enumeration via
//!   [`ThemeRegistry::list`] / [`ThemeRegistry::ids`].
//!
//! ## Reactivity
//!
//! [`use_init_theme_registry`] wraps the registry in a [`ThemeHandle`] and
//! provides it (plus a `State<Arc<Theme>>` of the active theme) to the
//! current subtree. Switching themes updates **both** the thread-local
//! active theme (so `Style::default()` picks up new values) **and** the
//! `State` subscribers (so anything reading `use_cn_theme()` re-renders).
//!
//! This dual update is why theme switches feel instant: element styles are
//! derived from `Style::default()` at render time, so an active-theme swap
//! propagates through the normal rebuild-on-render path with no extra
//! wiring.
//!
//! # Styling
//!
//! Every element, every custom `Component`, every bare `Style` — they all
//! share the same builder surface:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! div()
//!     .bg(neutral_900())
//!     .text_color(neutral_50())
//!     .px_4()
//!     .py_2()
//!     .border_1()
//!     .border_color(neutral_700())
//!     .rounded_md()
//!     .hover(|s| s.bg(neutral_800()))
//!     .active(|s| s.bg(neutral_950()))
//!     .focus_visible(|s| s.border_color(blue_500()))
//!     .disabled(|s| s.opacity_50().cursor_not_allowed())
//!     .child("Click me")
//! ```
//!
//! The utility vocabulary is documented in three places:
//!
//! - [`styling`] — the full explainer, the string-parser grammar, the
//!   primitive-side recipe for custom elements.
//! - [`StyledExt`] — every builder method, with per-method docs.
//! - [`crate::styling::colors`] — the full Tailwind palette as functions.
//!
//! ## Border
//!
//! `Style::border` is `Option<Border>`. It's **not inherited** (matches
//! CSS — child elements start borderless), and it renders on any primitive
//! that implements `DecorationExt`. In the built-in element set, that's
//! `div`, `button`, `checkbox`, `radio`, `input`, and `image`; `text` and
//! `p` ignore it — wrap them in a `div()` to outline text content.
//!
//! Utilities:
//!
//! | Helper | Tailwind | Effect |
//! |---|---|---|
//! | `.border_0()` / `_1()` / `_2()` / `_4()` / `_8()` | `border-{n}` | Width on all sides |
//! | `.border_t(..)` / `_r` / `_b` / `_l` | `border-t` / `-r` / `-b` / `-l` | Width per side |
//! | `.border_x(..)` / `_y(..)` | `border-x` / `-y` | Width on left+right / top+bottom |
//! | `.border_color(..)` | `border-{color}` | Border fill |
//! | `.border_inner()` / `_outer()` / `_center()` | *(freya-specific)* | `BorderAlignment` |
//! | `.border_none()` | `border-none` | Sets `border: None` outright |
//!
//! One sharp edge: `.border_none()` doesn't clear an existing border
//! inside a variant (both sides are `None`, so the merge skips it). Use
//! `.border_0()` (zero-width `Some(Border)`) to visually clear a border in
//! a `:hover` / `:focus` variant. See [Reset
//! limitation](crate::styling#reset-limitation).
//!
//! # Elements
//!
//! [`elements`] ships one-to-one HTML analogues:
//!
//! | Element | Constructor | HTML |
//! |---|---|---|
//! | `Div` | `div()` | `<div>` |
//! | `Text` | `text(..)` | `<span>` |
//! | `Paragraph` | `p(..)` | `<p>` |
//! | `Image` | `image(..)` | `<img>` |
//! | `Button` | `button()` | `<button>` |
//! | `Checkbox` | `checkbox()` | `<input type="checkbox">` |
//! | `Radio` | `radio()` | `<input type="radio">` |
//! | `Input` | `input(..)` | `<input type="text">` |
//!
//! Every element resolves its style through the same cascade — theme
//! defaults, then ancestor's inheritable subset, then UA defaults, then
//! user overrides, then live variants. Inheritance flows top-down through
//! `InheritedStyle` context, not through the DOM tree; only CSS-`inherit`-
//! able properties cascade. See [`elements`] for the full cascade diagram
//! and per-element UA defaults.
//!
//! Every interactive element (button, checkbox, radio, input) implements
//! [`HasSemanticEvents`] for the shared eight callbacks, and exposes
//! element-specific activation events (`.on_selected`, `.on_checked`,
//! `.on_change`, …) where activation state exists.
//!
//! # The one invariant
//!
//! **Rebuild the base [`Style`] from props every render; apply
//! [`StyleVariants`] to a *clone* for that pass only.** Never write an
//! applied style back into state.
//!
//! Two consequences fall out of this:
//!
//! - Ending a variant is automatic — the next render simply stops including
//!   it in the live set, and the base values are visible again. There is no
//!   "un-hover" callback; the exit is the absence of the entry.
//! - Applied styles are pure values with respect to
//!   `(base_style, live_kinds)`, so variant wiring is unit-testable by
//!   calling `StyleVariants::apply_all` directly (no rendering, no
//!   signals).
//!
//! This is the difference between a button that flickers on hover and a
//! button that doesn't. If you take only one thing from these docs, take
//! this paragraph.
//!
//! # Precedence and the disabled gate
//!
//! When several variants are live at once, they are applied in a fixed
//! order defined by [`StyleVariant::priority`]:
//!
//! ```text
//! disabled > active > checked > selected
//!          > focus_within > focus_visible > focus > hover
//! ```
//!
//! Callers don't sort their live set; `StyleVariants::apply_all` does.
//! Registered order and listed order both stop mattering once it runs, so
//! a renderer can hand the framework its live-state vector in the natural
//! "check each bool in turn" order and still get deterministic overrides.
//!
//! Precedence alone isn't enough. The renderer must **also** gate
//! `hover` / `focus` / `active` off at the source when disabled —
//! otherwise the hover closure still paints on the frame the pointer
//! crosses a disabled element, before `Disabled`'s higher priority gets a
//! chance to overwrite the same field. The built-in elements do this; the
//! primitive-side recipe in [`styling`] shows the correct shape for
//! custom elements.
//!
//! # Semantic events
//!
//! Pointer and focus events are exposed twice:
//!
//! 1. **Raw** via [`EventHandlersExt`](freya::prelude::EventHandlersExt) —
//!    `.on_press`, `.on_mouse_down`, `.on_key_down`, `.on_pointer_enter`,
//!    and so on. These map 1:1 to the underlying element's event surface.
//!
//! 2. **Semantic** via [`HasSemanticEvents`] — the same eight callbacks on
//!    every interactive element:
//!
//!    | Callback | Fires when |
//!    |---|---|
//!    | `.on_hover(..)` | Pointer enters |
//!    | `.on_hover_end(..)` | Pointer leaves |
//!    | `.on_pressed(..)` | Pointer goes down |
//!    | `.on_released(..)` | Pointer is released (anywhere) |
//!    | `.on_focus(..)` | Element gains focus |
//!    | `.on_blur(..)` | Element loses focus |
//!    | `.on_focus_visible(..)` | Focus becomes keyboard-originated |
//!    | `.on_focus_visible_end(..)` | Focus is no longer keyboard-originated |
//!
//!    These fire **once per state transition**, not on every pointer tick —
//!    no event-throttling on the caller side.
//!
//! Activation events are element-specific because only interactive
//! elements own activation state:
//!
//! - `button()` — `.on_selected` / `.on_deselected`
//! - `checkbox()` — `.on_checked` / `.on_unchecked`
//! - `radio()` — `.on_checked` only (HTML: clicking a checked radio is a
//!   no-op)
//! - `input(..)` — `.on_change` / `.on_submit`
//!
//! See [`elements`] for the full per-element matrix.
//!
//! # String parser
//!
//! [`StyledString`] parses a whitespace-separated Tailwind-style string
//! into the same `Style` mutations the fluent API performs, so the two
//! spellings are interchangeable:
//!
//! ```
//! // These produce identical styles:
//! Style::default().bg(neutral_900()).px_4().hover(|s| s.bg(neutral_800()));
//! Style::default().style("bg-neutral-900 px-4 hover:bg-neutral-800");
//! ```
//!
//! The parser also understands grammar the fluent API can't express
//! directly:
//!
//! - Pseudo-state prefixes (`hover:`, `focus-visible:`, `disabled:`, …)
//!   which land as `StyleVariants` entries.
//! - Arbitrary values (`w-[137px]`, `text-[12.8px]`, `bg-[#ff0000]`).
//! - CSS custom properties (`[--brand:#f00]`) and arbitrary properties
//!   (`[width:100px]`), exposed via `StyledString` custom properties so
//!   downstream renderers can thread them into their own systems.
//! - The `!` important marker — inert here, but accepted so strings
//!   copied from a Tailwind codebase don't warn.
//!
//! See [`styling`] for the full grammar and the list of what is
//! deliberately *not* supported.
//!
//! # Naming
//!
//! Utilities follow Tailwind, including the `.5` half-step and `n/5`
//! fraction forms. The two are told apart by a single character:
//!
//! - `*_p5` — decimal half-step. `w_1_p5()` = `w-1.5` = `6px`.
//! - `*_n_5` — fraction. `w_1_5()` = `w-1/5` = `20%`.
//!
//! `w_5()` (no underscore between prefix and digit) is Tailwind's
//! `w-5` = `20px`. All three are distinct identifiers; the compiler
//! catches a mix-up, but reviewers should read the `p` and the extra `_5`
//! as load-bearing.
//!
//! Border utilities use `border_*`, **not** `b_*` — `b_*` would collide
//! visually with `bg_*` and neither matches the Tailwind spec.
//!
//! State props on elements use a `with_` prefix (`.with_checked(..)`,
//! `.with_selected(..)`, `.with_disabled(..)`, `.with_read_only(..)`) so
//! `StyledExt`'s variant methods (`.checked(..)`, `.selected(..)`,
//! `.disabled(..)`) stay callable for styling the same state. Without the
//! prefix, `.checked(bool)` and `.checked(Fn)` would collide and method
//! resolution would silently pick the inherent one.
//!
//! # Testing
//!
//! Because applied styles are pure, the recommended unit-test shape is:
//!
//! ```
//! fn applied(base: Style, live: &[StyleVariant]) -> Style {
//!     base.variants.apply_all(live, base.clone())
//! }
//!
//! let base = Style::default().bg(black());
//! assert_eq!(applied(base.clone(), &[StyleVariant::Hover]).background,
//!            red().into());
//! assert_eq!(applied(base.clone(), &[]).background,
//!            black().into());
//! ```
//!
//! Borders test the same way (`Option<Border>` derives `PartialEq`):
//!
//! ```
//! let base = Style::default().border_2().border_color(red_500());
//! let hovered = applied(base.clone(), &[StyleVariant::Hover]);
//! assert!(hovered.border.as_ref().map_or(false, |b| !b.is_visible()));
//! ```
//!
//! Themes test by constructing them explicitly — `Style::themed(&theme)`
//! bypasses the thread-local active theme, so tests never touch ambient
//! state:
//!
//! ```
//! let theme = Theme::from_json(include_str!("../themes/default-dark.json")).unwrap();
//! let base = Style::themed(&theme);
//! assert_eq!(base.text_color, theme.fill("foreground").unwrap());
//! ```
//!
//! No rendering, no signals — just function calls.
//!
//! # Cheatsheet: from Tailwind
//!
//! If you're porting a Tailwind design, the mental translation is:
//!
//! | Tailwind | FreyaCN |
//! |---|---|
//! | `w-5` | `.w_5()` |
//! | `w-1.5` | `.w_1_p5()` |
//! | `w-1/5` | `.w_1_5()` |
//! | `px-4 py-2` | `.px_4().py_2()` |
//! | `bg-neutral-900` | `.bg(neutral_900())` |
//! | `text-neutral-50` | `.text_color(neutral_50())` |
//! | `border` | `.border_1()` |
//! | `border-2 border-gray-200` | `.border_2().border_color(gray_200())` |
//! | `border-l-4` | `.border_l(4.0)` |
//! | `rounded-lg` | `.rounded_lg()` |
//! | `font-semibold` | `.font_semibold()` |
//! | `hover:bg-neutral-800` | `.hover(\|s\| s.bg(neutral_800()))` |
//! | `focus-visible:ring-2` | `.focus_visible(\|s\| s.border_2())` |
//! | `disabled:opacity-50` | `.disabled(\|s\| s.opacity_50())` |
//! | `md:flex` | *(not supported — no responsive variants)* |
//! | `group-hover:..` | *(not supported — no group variants)* |
//!
//! For anything beyond single-element utilities, use the string parser
//! (`.style("...")`) — it handles pseudo-state prefixes, arbitrary
//! values, and CSS custom properties directly.
//!
//! # Non-goals
//!
//! Not in this crate, and not planned:
//!
//! - **Responsive breakpoints** (`md:`, `lg:`, …). Freya's layout is
//!   window-sized; responsive behaviour is expressed in layout code
//!   (`use_viewport_size()` + conditionals), not in the style vocabulary.
//! - **Group / peer variants** (`group-hover`, `peer-checked`). These need
//!   shared ancestor state that doesn't fit per-element `StyleVariants`.
// - **Descendant / sibling combinators.** Same reason.
//! - **Transitions / animations.** A separate layer; the natural extension
//!   point is to interpolate between two applied snapshots.
//! - **Outline offset rings** (Tailwind's `ring`). `Style::border` gives
//!   simple `:focus-visible` outlines; a true offset ring needs its own
//!   field.
//!
//! # FAQ
//!
//! **How is this different from using Tailwind CSS directly?**
//! Tailwind CSS targets the browser. FreyaCN targets Freya elements. Same
//! mental vocabulary, completely different rendering path — there's no
//! CSS engine, no class-string mutation on the DOM, no cascade resolution
//! at paint time. The "cascade" here is the `Style` merge described in
//! [`elements`], evaluated once per render.
//!
//! **Can I use FreyaCN without a theme?**
//! Yes. `Style::bare()` gives you every field at its default (transparent
//! background, black text, no border, no padding). `Style::default()` (the
//! `Default` impl) applies the thread-local active theme on top of `bare()`.
//! If you never call `use_init_theme_registry`, `active_theme()` returns
//! a fallback that behaves like `bare()`.
//!
//! **How do I reset a field to its default?**
//! Because `merge_style` uses a "differs from `Style::bare()`" test, a
//! field whose value equals its bare default **cannot override** an
//! ancestor's non-default value. This bites `padding: 0`, `margin: 0`,
//! `radius: 0`, `cursor: default`, and `border: None` in variants. For
//! borders, use `.border_0()` (not `.border_none()`); for the others,
//! either set a small non-default value or use `.no_default_style()` on
//! the element to skip UA defaults. See [Reset
//! limitation](crate::styling#reset-limitation) for the full story and
//! the long-term fix (`StylePatch` with `Option<T>` fields).
//!
//! **Why does `with_` prefix every state prop?**
//! So `StyledExt`'s variant methods — `.checked(..)`, `.selected(..)`,
//! `.disabled(..)` — remain callable on the same element. Without the
//! prefix, `checkbox().checked(true)` and `checkbox().checked(|s| ..)`
//! would collide, and Rust resolves inherent methods before trait methods
//! — so the styling call would silently break.
//!
//! **Can I write my own element?**
//! Yes — see the 5-step recipe in [`styling`]. It's about 60 lines of
//! boilerplate for a basic element. Use `Div` (`with_semantic_events`,
//! `with_checked`, `with_disabled`) as the outer box, `apply_to_rect` /
//! `apply_text_style` to copy `Style` onto a primitive, and
//! `attach_pointer_handlers` / `use_semantic_focus_events` for events.
//!
//! **Where do I put theme files?**
//! Anywhere `include_str!` can reach. The convention is `themes/*.json`
//! at the crate root (matches `include_str!("../themes/default-dark.json")`).
//!
//! # Where to look next
//!
//! - **[`theme`]** — JSON theme format, palette, component/state resolution,
//!   hot reload.
//! - **[`styling`]** — the full styling explainer, variant precedence,
//!   string-parser grammar, primitive-side element recipe.
//! - **[`elements`]** — the pre-built elements, cascade diagram, per-element
//!   defaults, known limitations.
//! - **[`StyledExt`]** — every builder method, with examples.
//! - **[`crate::styling::colors`]** — the full Tailwind palette as functions.
//!
//! [TOC]: #table-of-contents
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
//! [`HasSemanticEvents`]: crate::styling::HasSemanticEvents
//! [`elements`]: crate::elements
//! [`styling`]: crate::styling
//! [`theme`]: crate::theme
//! [`Fill`]: freya_core::prelude::Fill
//!
//! [`ThemeRegistry::load`]: crate::theme::ThemeRegistry::load
//! [`ThemeRegistry::load_dir`]: crate::theme::ThemeRegistry::load_dir
//! [`ThemeRegistry::reload_dir`]: crate::theme::ThemeRegistry::reload_dir
//! [`ThemeRegistry::get`]: crate::theme::ThemeRegistry::get
//! [`ThemeRegistry::set_active`]: crate::theme::ThemeRegistry::set_active
//! [`ThemeRegistry::list`]: crate::theme::ThemeRegistry::list
//! [`ThemeRegistry::ids`]: crate::theme::ThemeRegistry::ids
//! [`use_init_theme_registry`]: crate::theme::use_init_theme_registry
//! [`StyleVariant::priority`]: crate::styling::StyleVariant::priority

extern crate self as freyacn;

pub mod cn;
pub mod elements;
pub mod styling;
pub mod theme;
pub mod utils;
mod variants;

/// Everything you need to build a FreyaCN UI.
///
/// One import covers:
///
/// - **Elements** — `div()`, `text(..)`, `p(..)`, `image(..)`, `button()`,
///   `checkbox()`, `radio()`, `input(..)` and their types.
/// - **Styling** — `Style`, `StyledExt`, `StyledString`, `StyledColor`,
///   `StyleVariant`, `StyleVariants`, `InheritedStyle`.
/// - **Semantic events** — `HasSemanticEvents`, `SemanticEvents`.
/// - **Theme** — every type from [`crate::theme`].
/// - **Freya icons** — `lucide` module (not its contents, to avoid
///   colliding with `image()` / `radio()` element constructors).
/// - **Utils** — everything from [`crate::utils`].
///
/// Freya's own prelude is *not* re-exported here; import it explicitly if
/// you're mixing FreyaCN elements with raw Freya primitives:
///
/// ```
/// use freyacn::prelude::*;
/// use freyacn::base::*;   // freya::prelude::*
/// ```
///
/// Or just `use freya::prelude::*;` at the top of the file — it's the same
/// thing.
pub mod prelude {
    #[doc(inline)]
    pub use crate::cn;

    // ---- styling ----
    #[doc(inline)]
    pub use crate::styling::{
        HasSemanticEvents, InheritedStyle, SemanticEvents, Style, StyleVariant, StyleVariants,
        StyledColor, StyledExt, StyledString, ThemedExt,
    };

    // ---- elements ----
    #[doc(inline)]
    pub use crate::elements::*;

    // ---- theme ----
    #[doc(inline)]
    pub use crate::theme::*;

    // ---- utilities ----
    #[doc(inline)]
    pub use crate::utils::*;

    // ---- Freya icons ----
    #[doc(inline)]
    pub use freya::icons::lucide;

    // ---- Freya event plumbing ----
    #[doc(inline)]
    pub use freya_core::element::EventHandlerType;
    #[doc(inline)]
    pub use freya_core::events::name::EventName;

    // ---- misc ----
    #[doc(inline)]
    pub use rustc_hash::FxHashMap;
}

/// Re-export of [`freya::prelude`].
///
/// Useful when you want Freya's primitives (`rect`, `label`, `Size`,
/// `Color`, …) in scope *without* pulling in FreyaCN's elements and
/// styling. Most code should use [`prelude`] instead; `base` exists for
/// the rare case where a name conflict forces explicit disambiguation.
///
/// ```
/// use freyacn::prelude::*;
/// use freyacn::base::{rect, Size};   // pick specific Freya primitives
/// ```
pub mod base {
    #[doc(inline)]
    pub use freya::prelude::*;
}
