//! Procedural macros for the [`freyacn-core`] crate.
//!
//! This crate provides three attribute macros — [`component`],
//! [`variant`], [`extensions`] — that turn plain Rust functions and
//! enums into the styled, prop-driven building blocks freyacn-core
//! uses, plus a [`struct_fields`] stub that fires as a compile error
//! when that attribute is placed in the wrong position.
//!
//! All three macros share:
//!
//! - **A configurable crate path.** `#[component(crate = freyacn_core, …)]`,
//!   `#[variant(freyacn_core)]`, `#[extensions(crate = freyacn_core, …)]`.
//!   Defaults to `::freyacn` (the facade). See [Crate path
//!   override](#crate-path-override).
//! - **The `Style::bare()` invariant.** Every `Style` they emit starts
//!   from `Style::bare()`, not `Style::default()`, so class-string
//!   overlays can't accidentally overwrite theme-seeded fields. See
//!   [The `Style::bare()` invariant](#the-stylebare-invariant).
//! - **No runtime overhead.** Everything is macro-expanded at compile
//!   time; the only runtime work is the class-string parse that
//!   `#[variant]`'s generated `resolve()` performs (which the
//!   `#[component]` fold caches via `StyleVariants`).
//!
//! [`freyacn-core`]: https://crates.io/crates/freyacn
//!
//! # At a glance
//!
//! | Macro | Input | Generates |
//! |---|---|---|
//! | [`#[component]`](macro@component) | A function | `{Name}Component` struct + builder setters + `Component` impl + companion macro |
//! | [`#[variant]`](macro@variant) | An enum | `impl ComponentVariant` + `style_str()` + `From<&str>` / `From<String>` |
//! | [`#[extensions]`](macro@extensions) | A struct | Extra fields + trait impls (`ChildrenExt`, `StyledExt`, `KeyExt`, …) |
//! | [`#[struct_fields]`](macro@struct_fields) | — | Compile error |
//!
//! # Crate path override
//!
//! Every macro emits paths like `::freyacn::prelude::Style`. That's the
//! right default for user-facing code, which always has the facade
//! available. But **internal** crates that the facade itself depends on
//! (e.g. `freyacn-shadcn`) can't depend on the facade — that's a
//! dependency cycle. For those, override the path at the macro site:
//!
//! ```ignore
//! // Inside freyacn-shadcn, which depends on freyacn-core but not freyacn.
//! use freyacn_macros::{component, variant};
//!
//! #[variant(freyacn_core)]
//! pub enum ButtonVariant { /* … */ }
//!
//! #[component(
//!     crate = freyacn_core,
//!     variants("…", variant: ButtonVariant = ButtonVariant::Default),
//!     ext(styled, children),
//! )]
//! pub fn Button() { /* … */ }
//! ```
//!
//! The generated code references `freyacn_core::prelude::{…}` instead of
//! `freyacn::prelude::{…}`. The target crate must expose the same items
//! (`Style`, `StyledExt`, `ComponentVariant`, `Element`, `Component`,
//! `IntoElement`, the extension traits, etc.) under its `prelude`.
//!
//! # The `Style::bare()` invariant
//!
//! The `Style` that a `#[component]` fold produces is built by merging
//! several layers:
//!
//! ```text
//! Style::default()                              ← theme layer  (`#[component]`)
//!   .style(  base class string  )
//!   .merge_style(&variant.resolve())            ← variant patches  (`#[variant]`)
//!   .merge_style(&size.resolve())               ← size patches     (`#[variant]`)
//!   .merge_style(&self.style)                   ← user overrides   (`#[extensions(styled)]`)
//! ```
//!
//! Two of those layers must start from **`Style::bare()`**, not
//! `Style::default()`:
//!
//! - `#[variant]`'s `ComponentVariant::resolve()` — starts from `bare()`,
//!   then folds the class string.
//! - `#[extensions(styled)]`'s `self.style` field — initialised to
//!   `bare()`, then gets whatever the caller chained with `.style("…")` /
//!   `.bg(..)` / etc.
//!
//! Only the **theme layer** — the initial `Style::default()` inside
//! `#[component]`'s fold — starts from theme defaults, and that's
//! correct: the theme is what you get when no class string touches a
//! field.
//!
//! **Why does this matter?** [`Style::merge_style`] copies a field from
//! the argument only if that field differs from `Style::bare()`. If
//! `variant.resolve()` returned `Style::default()`, its `text_color`
//! would be the theme's `foreground` (not bare black), so the merge
//! would copy `foreground` into the folded style — silently overwriting
//! whatever the variant's class string set (e.g.
//! `text-primary-foreground`).
//!
//! Concretely: a `ButtonVariant::Default` whose class string is
//! `bg-primary text-primary-foreground` renders with `text_color =
//! primaryForeground`. If `resolve()` started from `Style::default()`,
//! `foreground` would leak through the `ButtonSize::Default` and
//! `.style(..)` layers and overwrite it, and the button would render
//! with the wrong text colour. Starting from `bare()` keeps each layer
//! a **sparse patch** — only fields the layer explicitly sets survive
//! the merge.
//!
//! # `#[component]`
//!
//! Turns a function into a `Component` struct. The function's **body**
//! becomes `render()`'s body — everything the body `return`s must
//! implement `IntoElement`. The function's **parameters** become required
//! props. Attribute props, struct fields, and variant props add to the
//! struct.
//!
//! ## Grammar
//!
//! ```ignore
//! #[component(
//!     crate = <path>,                          // optional
//!     props(                                   // optional, repeatable
//!         <name>: <Type>,
//!         <name>: <Type> = <Expr>,
//!         …
//!     ),
//!     ext(                                     // optional, repeatable
//!         <extension_name>,
//!         …
//!     ),
//!     variants(                                // optional, at most once
//!         "<base class string>",
//!         <name>: <EnumType> = <EnumType>::Variant,
//!         …
//!     ),
//! )]
//! fn <fn_name>(<required_param>: <Type>, …) -> impl IntoElement { <body> }
//! ```
//!
//! Inside the function, `#[struct_fields(<name>: <Type> [= <Expr>], …)]`
//! can declare additional storage fields. Those don't get setters — they
//! exist for internal state the caller manages via traits.
//!
//! ## Prop sources
//!
//! The macro collects props from **four** sources, in this order. Duplicate
//! names across sources are a compile error:
//!
//! | Source | Required? | Storage | Render body sees | Getter |
//! |---|---|---|---|---|
//! | Function parameters | Yes | `Option<T>` | `&T` (panics if unset) | `.name(impl Into<T>)` |
//! | `props(name: T)` | No | `Option<T>` | `&Option<T>` | `.name(impl Into<Option<T>>)` |
//! | `props(name: T = expr)` | No | `T` | `&T` | `.name(impl Into<T>)` |
//! | `#[struct_fields(name: T)]` | No | `Option<T>` | `&Option<T>` | *(none)* |
//! | `#[struct_fields(name: T = expr)]` | No | `T` | `&T` | *(none)* |
//! | `variants(name: MyEnum = expr)` | No | `MyEnum` | `&MyEnum` | `.name(impl Into<MyEnum>)` |
//!
//! ## Generated code shape
//!
//! For a function named `card`:
//!
//! ```ignore
//! // 1. The generated struct.
//! #[derive(PartialEq, Clone)]
//! pub struct CardComponent {
//!     // props fields — one per prop from every source
//!     // + extension fields (children, style, key, event_handlers,
//!     //                    accessibility, semantic_events, …)
//! }
//!
//! // 2. `new()` — every field at its initial value.
//! impl CardComponent {
//!     pub fn new() -> Self { /* … */ }
//!
//!     // 3. Builder setters — one per prop (except struct_fields).
//!     pub fn <prop_name>(mut self, value: impl Into<…>) -> Self { /* … */ }
//! }
//!
//! // 4. Default impl.
//! impl ::core::default::Default for CardComponent {
//!     fn default() -> Self { Self::new() }
//! }
//!
//! // 5. Free constructor.
//! pub fn Card() -> CardComponent { CardComponent::new() }
//!
//! // 6. Extension trait impls (ChildrenExt, StyledExt, …).
//!
//! // 7. Component impl — the body lives here.
//! impl ::freyacn::prelude::Component for CardComponent {
//!     fn render(&self) -> impl ::freyacn::prelude::IntoElement {
//!         // extension locals
//!         let style: &Style = …;
//!         let children: &Vec<Element> = …;
//!         // …one per extension
//!
//!         // prop locals
//!         let title: &String = …;
//!         // …one per prop
//!
//!         // user body
//!         #body
//!     }
//!
//!     fn render_key(&self) -> DiffKey { … }
//! }
//!
//! // 8. Companion macro for construction.
//! #[macro_export]
//! macro_rules! Card {
//!     ($($key:ident = $val:expr),* $(,)?) => {
//!         CardComponent::new()$(.$key($val))*
//!     };
//! }
//! ```
//!
//! ## The `variants(…)` group
//!
//! The first string literal is a **base class string** — it's applied
//! right after `Style::default()`, so it sits beneath variant / size /
//! user overrides in the merge. Pass `""` (or omit the string) for no
//! base style.
//!
//! The remaining entries are **variant props** — enum-typed props with
//! required defaults. Each one is resolved at render time and merged into
//! the fold above the base, below user overrides, in declaration order.
//!
//! `variants(… non-empty)` **requires** `ext(styled)` — the merged
//! style needs somewhere to live in the render body.
//!
//! ```ignore
//! #[component(
//!     variants(
//!         "inline-flex items-center gap-2",     // base
//!         variant: ButtonVariant = ButtonVariant::Default,
//!         size:    ButtonSize    = ButtonSize::Default,
//!     ),
//!     ext(styled),
//! )]
//! fn Button() { /* … */ }
//! ```
//!
//! ## The `ext(…)` group
//!
//! Requests one or more **extension bundles**. See [`#[extensions]`](macro@extensions)
//! for the supported names. The common shadcn set:
//!
//! ```ignore
//! ext(styled, children, event_handlers, accessibility, semantic_events)
//! ```
//!
//! Each extension:
//!
//! - Adds a field to the struct (`style`, `children`, `event_handlers`, …).
//! - Initialises it in `new()`.
//! - Adds a local binding in `render()` (`let style: &Style = …`).
//! - Implements the corresponding trait (`StyledExt`, `ChildrenExt`, …).
//!
//! ## Warning suppression
//!
//! `#[component]` always binds every prop as a local in `render()` —
//! `variant`, `size`, and the rest — so callers can reference them from
//! the body. But the common case (shadcn `Button` here) only uses the
//! merged `style` local. That produces `unused_variables` warnings on
//! `variant` / `size`.
//!
//! Silence with `#[allow(unused_variables)]` on the function. The macro
//! emits `#[allow(unused_variables)]` on the generated `render()` too, so
//! the warning wouldn't fire there, but the *user-visible* function still
//! gets it from the bindings emitted at the top of `render()`.
//!
//! ```ignore
//! #[component(/* … */)]
//! #[allow(unused_variables)]
//! pub fn Button() { /* … */ }
//! ```
//!
//! ## Hook safety
//!
//! The body of `#[component]` is `render()` — hook functions like
//! `use_state` / `use_hook` / `use_a11y` / `use_focus` must be called
//! unconditionally at the top. **Never** wrap another hook's call in
//! `use_hook`:
//!
//! ```ignore
//! // ✗ PANICS on the second render — the outer hook short-circuits,
//! //   the inner hooks never fire, and Freya complains about slot
//! //   misalignment.
//! fn app() -> impl IntoElement {
//!     use_hook(|| {
//!         load_theme_registry_from_dir(dir).set_active("…").unwrap();
//!     });
//!     // …
//! }
//!
//! // ✓ `load_theme_registry_from_dir` is itself a hook that internally
//! //   wraps in `use_hook`. Call it directly.
//! fn app() -> impl IntoElement {
//!     load_theme_registry_from_dir(dir).set_active("…").unwrap();
//!     // …
//! }
//! ```
//!
//! Same rule as Freya's own components.
//!
//! # `#[variant]`
//!
//! Turns an enum into a `ComponentVariant`. Each variant carries a
//! Tailwind class string via `#[variant("…")]`. The generated `resolve()`
//! parses that string into a `Style` on first use (via `StyledString`)
//! and returns it — starting from `Style::bare()`.
//!
//! ## Grammar
//!
//! ```ignore
//! #[variant]                    // → ::freyacn path
//! #[variant(freyacn_core)]      // → explicit path
//! pub enum <Name> {
//!     #[variant("<class string>")]
//!     <Variant>,
//!     …
//! }
//! ```
//!
//! ## Generated code shape
//!
//! ```ignore
//! impl <Name> {
//!     /// The raw class string for the current variant.
//!     pub fn style_str(&self) -> &'static str {
//!         match self {
//!             Self::Variant1 => "…",
//!             // …
//!         }
//!     }
//!
//!     /// Convenience alias for `ComponentVariant::resolve(self)`.
//!     pub fn resolve(&self) -> Style {
//!         <Self as ComponentVariant>::resolve(self)
//!     }
//! }
//!
//! impl ::freyacn::prelude::ComponentVariant for <Name> {
//!     fn resolve(&self) -> Style {
//!         StyledExt::style(Style::bare(), self.style_str())
//!     }
//! }
//!
//! impl From<&str> for <Name> { /* kebab-case match, first variant = fallback */ }
//! impl From<String> for <Name> { /* delegates to From<&str> */ }
//! ```
//!
//! The `From<&str>` impl matches **kebab-case** names — `ButtonVariant::Default`
//! becomes `"default"`, `IconXs` becomes `"icon-xs"`. Unknown names fall
//! back to the first declared variant.
//!
//! ## Common shadcn pattern
//!
//! ```ignore
//! #[variant(freyacn_core)]
//! #[derive(Default, PartialEq, Clone)]
//! pub enum ButtonVariant {
//!     #[default]
//!     #[variant("bg-primary text-primary-foreground hover:bg-primary/90")]
//!     Default,
//!
//!     #[variant("bg-destructive text-white hover:bg-destructive/90")]
//!     Destructive,
//!
//!     #[variant("border bg-background hover:bg-accent")]
//!     Outline,
//!
//!     #[variant("bg-secondary text-secondary-foreground")]
//!     Secondary,
//!
//!     #[variant("hover:bg-accent hover:text-accent-foreground")]
//!     Ghost,
//!
//!     #[variant("text-primary hover:underline")]
//!     Link,
//! }
//! ```
//!
//! Class strings can use anything `StyledString` supports — theme tokens,
//! pseudo-state prefixes, the aria family, arbitrary values, `ring-*`,
//! etc. See `freyacn_core::styling::StyledString` for the full grammar.
//!
//! ## Testing
//!
//! Class strings are parsed at **runtime**, so a typo compiles but panics
//! on the first render. Guard against that with a parse test:
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use freyacn_core::styling::StyledString;
//!
//!     #[test]
//!     fn all_variants_parse() {
//!         for v in [ButtonVariant::Default, ButtonVariant::Destructive /* … */] {
//!             StyledString::parse(v.style_str());
//!         }
//!     }
//! }
//! ```
//!
//! # `#[extensions]`
//!
//! Opts a hand-written struct into a freyacn-core **extension bundle**.
//! Same mechanism `#[component]`'s `ext(…)` group uses, but for structs
//! you wrote yourself.
//!
//! Used standalone when you want a struct that participates in the
//! ChildrenExt / StyledExt / KeyExt / EventHandlersExt ecosystem without
//! being generated from a function.
//!
//! ## Grammar
//!
//! ```ignore
//! #[extensions(<name>, <name>, …)]              // → ::freyacn path
//! #[extensions(crate = freyacn_core, <name>, …)] // → explicit path
//! pub struct <Name> { /* named fields */ }
//! ```
//!
//! ## Supported extensions
//!
//! | Name | Field added | Trait impl | Local binding in `Component::render` (if you use `#[component]`) |
//! |---|---|---|---|
//! | `children` | `children: Vec<Element>` | `ChildrenExt` | `children: &Vec<Element>` |
//! | `key` | `key: DiffKey` | `KeyExt` | `key: &DiffKey` |
//! | `styled` | `style: Style` | `StyledExt` | `style: &Style` (sparse patch) |
//! | `event_handlers` | `event_handlers: FxHashMap<…>` | `EventHandlersExt` | `event_handlers: &FxHashMap<…>` |
//! | `accessibility` | `accessibility: AccessibilityData` | `AccessibilityExt` | `accessibility: &AccessibilityData` |
//! | `semantic_events` | `semantic_events: SemanticEvents` | `HasSemanticEvents` | `semantic_events: &SemanticEvents` |
//!
//! `styled` is initialised to `Style::bare()`, not `Style::default()`,
//! so it behaves as a sparse patch. See [The `Style::bare()`
//! invariant](#the-stylebare-invariant).
//!
//! ## Example
//!
//! ```ignore
//! use freyacn_macros::extensions;
//!
//! #[extensions(children, key, styled, event_handlers)]
//! #[derive(PartialEq, Clone)]
//! pub struct MyBox {
//!     // any fields you already wanted
//!     pub label: String,
//!     // extension fields are appended automatically:
//!     //   children, key, style, event_handlers
//! }
//!
//! // Now the struct accepts:
//! //   .child(..)          — ChildrenExt
//! //   .key(..)            — KeyExt
//! //   .style("…")         — StyledExt
//! //   .on_press(..)       — EventHandlersExt
//! ```
//!
//! # `#[struct_fields]`
//!
//! A stub macro that fires when `#[struct_fields(…)]` is placed in the
//! wrong position. `#[struct_fields(…)]` is only valid **inside** a
//! `#[component]` function — anywhere else, the stub emits a compile
//! error with the correct placement instructions.
//!
//! You shouldn't need to use this directly; it exists so a misplaced
//! attribute produces a helpful error instead of "unknown attribute".
//!
//! # Integration with the theme system
//!
//! Generated `Component::render` bodies read the theme through the usual
//! `Style::default()` path — that's the theme layer in the fold.
//! `#[variant]`'s `resolve()` and the `styled` extension's `self.style`
//! start from `Style::bare()`, so they only override fields the class
//! string or user chain actually sets.
//!
//! `#[component]` **doesn't** install a theme — call
//! `load_theme_registry_from_dir(..)` (or `load_theme_registry(..)` /
//! `set_active_theme(..)`) once at the app root, before any component
//! that uses theme tokens renders.
//!
//! # Feature flags
//!
//! None. Every macro is unconditionally enabled.
//!
//! # Testing your macros
//!
//! Three test shapes cover almost everything:
//!
//! 1. **Class-string parse test** — catches typos in `#[variant("…")]` at
//!    `cargo test` time instead of first render. See the `#[variant]`
//!    section above.
//! 2. **Generated-code shape test** — `cargo expand` on a test component,
//!    or a small `#[test]` that asserts the struct's `PartialEq` impl
//!    behaves. Rarely necessary if the parse test passes.
//! 3. **Hook safety test** — a minimal component rendered twice; if the
//!    hook order is wrong, the second render panics with the hook-misuse
//!    message. Worth adding to any component that calls more than
//!    `use_state` once.

mod component;
mod extension;
mod struct_fields;
mod variant;

use proc_macro::TokenStream;

use crate::component::parse_component;
use crate::extension::parse_extensions;
use crate::variant::parse_variant;

/// Turn a function into a `Component` struct.
///
/// See the [crate-level docs](crate) for the full grammar, the
/// generated code shape, the prop source table, and gotchas around
/// `#[allow(unused_variables)]` and hook safety.
///
/// # Minimal example
///
/// ```ignore
/// use freyacn_macros::component;
///
/// #[component]
/// fn greeting(name: String) {
///     div().child(text(format!("Hello, {name}!")))
/// }
///
/// // Now `GreetingComponent` exists:
/// let g = greeting("world".into()).into_element();
/// ```
///
/// # Full example
///
/// ```ignore
/// #[component(
///     crate = freyacn_core,
///     variants(
///         "inline-flex items-center gap-2 rounded-md text-sm font-medium",
///         variant: ButtonVariant = ButtonVariant::Default,
///         size:    ButtonSize    = ButtonSize::Default,
///     ),
///     props(
///         with_disabled: bool = false,
///         with_focusable: bool = true,
///     ),
///     ext(styled, children, event_handlers, accessibility, semantic_events),
/// )]
/// #[allow(unused_variables)]
/// pub fn Button() {
///     button()
///         .merge_style(style.clone())
///         .children(children.clone())
///         .event_handlers(event_handlers.clone())
///         .accessibility(accessibility.clone())
///         .semantic_events(semantic_events.clone())
///         .with_disabled(*with_disabled)
///         .with_focusable(*with_focusable)
/// }
/// ```
///
/// # Errors
///
/// The macro emits a compile error (not a panic) when:
///
/// - A prop is declared twice across `props` / `struct_fields` / `variants` / fn params.
/// - A `variants` entry lacks a default.
/// - `variants(…)` is non-empty but `ext(styled)` is missing.
/// - An `ext(…)` name is unknown.
/// - A `crate = …` argument is duplicated.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_component(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Opt a struct into a freyacn-core extension bundle.
///
/// See the [crate-level docs](crate) for the list of supported
/// extension names, what each one adds, and the `crate = <path>`
/// override.
///
/// # Example
///
/// ```ignore
/// use freyacn_macros::extensions;
///
/// #[extensions(children, key, styled, event_handlers)]
/// #[derive(PartialEq, Clone)]
/// pub struct Panel {
///     pub title: String,
/// }
///
/// // Now accepts `.child(..)`, `.key(..)`, `.style("…")`, `.on_press(..)`.
/// ```
///
/// # Errors
///
/// The macro emits a compile error when:
///
/// - The target isn't a struct with named fields.
/// - An extension name is unknown.
/// - A `crate = …` argument is duplicated.
#[proc_macro_attribute]
pub fn extensions(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_extensions(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Turn an enum into a `ComponentVariant`.
///
/// Every variant must carry a `#[variant("<class string>")]` attribute.
/// See the [crate-level docs](crate) for the generated code shape, the
/// kebab-case `From<&str>` behaviour, and the parse-test recipe.
///
/// # Minimal example
///
/// ```ignore
/// use freyacn_macros::variant;
///
/// #[variant]
/// #[derive(Default, PartialEq, Clone)]
/// pub enum Tone {
///     #[default]
///     #[variant("text-neutral-900")]
///     Neutral,
///
///     #[variant("text-red-600")]
///     Danger,
/// }
/// ```
///
/// # With a crate path override
///
/// ```ignore
/// // Inside freyacn-shadcn, which depends on freyacn-core but not freyacn:
/// #[variant(freyacn_core)]
/// pub enum ButtonVariant { /* … */ }
/// ```
///
/// # Errors
///
/// The macro emits a compile error when:
///
/// - A variant is missing its `#[variant(..)]` attribute.
/// - A variant has duplicate `#[variant(..)]` attributes.
/// - The enum has no variants.
/// - The `#[variant(..)]` argument isn't a string literal.
#[proc_macro_attribute]
pub fn variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse_variant(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Stub for `#[struct_fields(...)]` when placed above `#[component(...)]`.
///
/// `#[struct_fields(…)]` is only valid **inside** a `#[component]`
/// function body's attribute list. Anywhere else — on a function, a
/// struct, an impl block — this stub fires and emits a compile error
/// explaining the correct placement.
///
/// You shouldn't call this macro directly. It exists so a misplaced
/// attribute produces a helpful error instead of "cannot find attribute".
#[proc_macro_attribute]
pub fn struct_fields(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    struct_fields::stub().into()
}
