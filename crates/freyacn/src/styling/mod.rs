//! Styling for Freya elements.
//!
//! Five layers, each independently useful:
//!
//! | Layer | Item | Purpose |
//! |---|---|---|
//! | Data | [`Style`] | One flat struct holding every paint / typography / layout attribute. |
//! | Builder | [`StyledExt`] | Fluent `self -> Self` methods on anything that exposes a `Style`. |
//! | Parser | [`StyledString`] | Tailwind-style class strings that fan out into the same `Style` mutations as the builder. |
//! | State | [`StyleVariant`] / [`StyleVariants`] | Pseudo-state patches (`:hover`, `:focus`, …) stored *on* the style. |
//! | Palette | [`StyledColor`] and friends | Tailwind colours, gradients, shaders — everything convertible to [`freya_core::prelude::Fill`]. |
//!
//! # Mental model
//!
//! Freya itself is style-agnostic: each primitive has its own props
//! (`ButtonProps`, `TextProps`, …). FreyaCN sits on top of that with a
//! single shared [`Style`] whose fields you fold into a primitive's
//! own props, so a chain written once transfers unchanged to whatever
//! element renders it:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! // Either build the style fluently…
//! let s = Style::default()
//!     .bg(neutral_900())
//!     .text_color(neutral_50())
//!     .px_4()
//!     .py_2()
//!     .rounded_md()
//!     .font_semibold()
//!     .hover(|s| s.bg(neutral_800()))
//!     .active(|s| s.bg(neutral_950()))
//!     .disabled(|s| s.opacity_50().cursor_not_allowed());
//!
//! // …or parse a Tailwind-style string into the same struct.
//! let s = Style::default().style(
//!     "bg-neutral-900 text-neutral-50 px-4 py-2 rounded-md font-semibold \
//!      hover:bg-neutral-800 active:bg-neutral-950 disabled:opacity-50",
//! );
//! ```
//!
//! # The one invariant
//!
//! **Rebuild the base [`Style`] from props every render; apply
//! [`StyleVariants`] to a *clone* for that pass only.** Never write an
//! applied style back into state. Two consequences fall out of this:
//!
//! * Ending a variant is automatic — the next render simply stops
//!   including it in the live set, and the base values are visible again.
//!   There is no "un-hover" callback; the exit is the absence of the entry.
//! * Applied styles are pure values with respect to
//!   `(base_style, live_kinds)`, so variant wiring can be unit-tested by
//!   calling [`StyleVariants::apply_all`] directly.
//!
//! This is the difference between a button that flickers on hover and a
//! button that doesn't. If you take only one thing from these docs, take
//! this paragraph.
//!
//! See [`styled`] for the full explainer and a copy-pasteable
//! primitive-side example.
//!
//! # Precedence
//!
//! When several variants are live at once, they are applied in a fixed
//! order defined by [`StyleVariant::priority`] — `disabled` wins over
//! `active`, which wins over `focus`, which wins over `hover` — so a
//! disabled button never paints its hover colour even for one frame.
//! Callers don't need to sort their live set; [`StyleVariants::apply_all`]
//! does it.
//!
//! Registered order and listed order both stop mattering as soon as
//! [`StyleVariants::apply_all`] runs. That's deliberate: it means your
//! renderer can hand the framework its live-state vector in the natural
//! "check each bool in turn" order and still get deterministic overrides.
//!
//! # The `disabled` gate
//!
//! Precedence is not enough on its own. Your renderer must **also** gate
//! `hover` / `focus` / `active` off at the source when disabled —
//! otherwise the hover closure still runs (and still paints) on the
//! frame the pointer crosses a disabled element, before `Disabled`'s
//! higher priority gets a chance to overwrite the same field. The
//! primitive-side recipe in [`styled`] shows the correct shape.
//!
//! # String parser
//!
//! [`StyledString`] parses a whitespace-separated Tailwind-style class
//! string into the same `Style` mutations the fluent API performs, so
//! the two spellings are interchangeable — pick whichever reads better
//! at the call site. The parser also understands the grammar that the
//! fluent API can't express directly:
//!
//! * Pseudo-state prefixes (`hover:`, `focus-visible:`, `disabled:`, …)
//!   which land as [`StyleVariants`] entries.
//! * Arbitrary values (`w-[137px]`, `text-[12.8px]`, `bg-[#ff0000]`).
//! * CSS custom properties (`[--brand:#f00]`) and arbitrary properties
//!   (`[width:100px]`), exposed via [`StyledString`] custom properties
//!   so downstream renderers can thread them into their own systems.
//! * The `!` important marker — inert here, but accepted so strings
//!   copied from a Tailwind codebase don't warn.
//!
//! See the [`styled_string`] module docs for the full grammar and the
//! list of what is deliberately *not* supported.
//!
//! # Naming
//!
//! Utilities follow Tailwind, including the `.5` half-step and `n/5`
//! fraction forms. The two are deliberately told apart by a single
//! character:
//!
//! * `*_p5` — decimal half-step. `w_1_p5()` = `w-1.5` = `6px`.
//! * `*_n_5` — fraction. `w_1_5()` = `w-1/5` = `20%`.
//!
//! `w_5()` (no underscore between prefix and digit) is Tailwind's
//! `w-5` = `20px`. All three are distinct identifiers and the compiler
//! will catch a mix-up, but reviewers should read the `p` and the extra
//! `_5` as load-bearing. If you're coming from Tailwind, the mental
//! translation is:
//!
//! | Tailwind | FreyaCN |
//! |---|---|
//! | `w-5`   | `w_5()`      |
//! | `w-1.5` | `w_1_p5()`   |
//! | `w-1/5` | `w_1_5()`    |
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
//! No rendering, no signals — just function calls.
//!
//! # Non-goals
//!
//! Not in this layer, and not planned here:
//!
//! * **Group / peer variants** (`group-hover`, `peer-checked`) — these
//!   need shared ancestor state and don't fit the per-element
//!   [`StyleVariants`] container.
//! * **Descendant / sibling combinators** — same reason.
//! * **Transitions / animations** — a separate layer; the natural
//!   extension point is to interpolate between two applied snapshots.
//! * **Focus rings** — [`Style`] has no ring field yet; primitives that
//!   need one currently build it from `bg` / `radius` on an overlay
//!   child. A shared ring API is future work.

pub mod colors;
pub mod style;
pub mod style_variants;
pub mod styled;
pub mod styled_string;

/// Tailwind colour palette, gradients, shaders, and the [`StyledColor`]
/// type that unifies them.
///
/// Every colour helper in this module returns a [`StyledColor`], which
/// converts into [`freya_core::prelude::Fill`] on assignment — that's
/// what lets [`bg`](styled::StyledExt::bg) and
/// [`text_color`](styled::StyledExt::text_color) accept both plain
/// solids and gradients without overloads.
///
/// See [`colors`] for the full palette and the [`black`] / [`white`]
/// primitives.
pub use colors::{StyledColor, black, white};

/// The [`Style`] data struct and its `Default` impl.
///
/// One flat struct covering paint, typography, interaction, effects,
/// radius, and layout. `Default` is "unstyled": transparent background,
/// black text, `1.0` scale, zero radius / padding / margin / gap — i.e.
/// what a bare Freya element looks like before any utility is chained.
///
/// Re-exported here so primitives can construct a base style and merge
/// it into their own props without reaching into the `style` submodule
/// path. See [`style`] for the field-by-field breakdown.
pub use style::Style;

/// Fluent [`StyledExt`] builder.
///
/// Every method is `self -> Self`, so utilities chain. The trait is
/// implemented for anything that exposes a `&mut Style` via
/// [`get_style`](styled::StyledExt::get_style), which is how it reaches
/// both the bare [`Style`] struct and component-owned style wrappers.
///
/// The doc comment on [`styled`] carries the primitive-side recipe for
/// applying variants per render — read that before wiring `.hover(..)`
/// into a new element.
pub use styled::{StyledExt, ThemedExt};

/// Tailwind-style class-string parser.
///
/// [`StyledString::parse`] tokenises a whitespace-separated string of
/// Tailwind utilities, pseudo-state prefixes, and arbitrary bracketed
/// values, then applies them through the same [`StyledExt`] chain the
/// fluent API uses. Use this when a class string is easier to read (or
/// arrives from config) than the equivalent builder call.
///
/// See the [`styled_string`] module docs for the full grammar and the
/// list of unsupported constructs.
pub use styled_string::StyledString;

/// Pseudo-state kinds and the container that stores their patches.
///
/// [`StyleVariant`] is the enum (`Hover`, `Focus`, `Active`, …);
/// [`StyleVariants`] is the per-style container those patches live in.
/// The container is `Clone`, cheap to clone (all payloads are `Arc`'d),
/// and compares by variant *kind* only — not by closure identity — so
/// freshly-allocated callbacks don't defeat `Style: PartialEq` across
/// renders.
///
/// See [`style_variants`] for [`StyleVariants::apply_all`]'s priority
/// ordering and the callback signature.
pub use style_variants::{StyleVariant, StyleVariants};
