//! Built-in UI elements.
//!
//! | Element | Constructor | HTML analogue |
//! |---|---|---|
//! | [`Div`] | [`div()`] | `<div>` |
//! | [`Text`] | [`text(...)`] | `<span>` (leaf text) |
//! | [`Paragraph`] | [`p(...)`] | `<p>` |
//! | [`Image`] | [`image(...)`] | `<img>` |
//! | [`Button`] | [`button()`] | `<button>` |
//! | [`Checkbox`] | [`checkbox()`] | `<input type="checkbox">` |
//! | [`Radio`] | [`radio()`] | `<input type="radio">` |
//! | [`Input`] | [`input(...)`] | `<input type="text">` |
//!
//! Every element in this module follows the same conventions, so once you
//! know one you know them all.
//!
//! ## Style cascade
//!
//! Each element resolves its final [`Style`](crate::styling::Style) from a
//! fixed-priority merge at render time:
//!
//! ```text
//! theme defaults
//!   < ancestor's inheritable subset          (via `InheritedStyle` context)
//!     < user-agent defaults                 (element-specific, e.g. `<button>`)
//!       < user overrides                    (the builder calls on the element)
//!         < live variants                   (`:hover`, `:active`, `:focus`, …)
//! ```
//!
//! Inheritance flows **top-down through context**, not through the DOM tree.
//! Every element that renders children calls
//! [`use_provide_context`](freya::prelude::use_provide_context) with its
//! resolved style, and children call
//! [`try_consume_context`](freya::prelude::try_consume_context) to read it.
//! Only CSS-`inherit`-able properties (text color, `font-*`, `text-*`,
//! `letter-spacing`, `line-height`, `text-decoration`, `cursor`) flow
//! through the inherited slice — a `padding`, `background`, or `border` set
//! on a parent does **not** cascade.
//!
//! See [`Style::inheritable_subset`](crate::styling::Style::inheritable_subset)
//! and [`Style::merge_style`](crate::styling::Style::merge_style) for the
//! mechanics.
//!
//! ## Border support
//!
//! [`Style`](crate::styling::Style) now has an `Option<Border>` field, so
//! borders are a first-class style property. Support depends on the
//! primitive the element renders:
//!
//! | Element | Supports `.border_*()` | Renderer |
//! |---|---|---|
//! | `div()` | ✓ | `apply_to_rect` → `DecorationExt` |
//! | `button()` | ✓ | `apply_to_rect` |
//! | `checkbox()` | ✓ | built on `div()` |
//! | `radio()` | ✓ | built on `div()` |
//! | `input(...)` | ✓ | built on `div()` |
//! | `image(...)` | ✓ | local `apply_style` (applies `Border` via `DecorationExt`) |
//! | `text(...)` | ✗ | `Label` has no `DecorationExt` |
//! | `p(...)` | ✗ | `Paragraph` has no `DecorationExt` |
//!
//! For `text()` and `p()`, wrap in a `div()` — the `div` owns the border,
//! the text lives inside:
//!
//! ```ignore
//! div()
//!     .border_1()
//!     .border_color(gray_200())
//!     .child(text("Hello"))
//! ```
//!
//! This matches HTML: text nodes can't be bordered, only the boxes that
//! contain them.
//!
//! Border does **not** inherit. A child `div()` inside a bordered parent
//! starts with `border: None` — set borders on the element that should
//! paint the outline.
//!
//! Available Tailwind-style utilities: `.border_1()` / `_2()` / `_4()` /
//! `_8()` / `_0()`, `.border_t(..)` / `_r` / `_b` / `_l` for per-side,
//! `.border_x(..)` / `_y(..)` for opposite pairs, `.border_color(..)`,
//! plus freya-specific `.border_inner()` / `_outer()` / `_center()` for
//! [`BorderAlignment`](freya::prelude::BorderAlignment). See
//! [`StyledExt`](crate::styling::StyledExt) for the full set.
//!
//! ## Element-specific defaults
//!
//! Following HTML, some elements ship user-agent defaults that differ from
//! the framework base:
//!
//! * `button()` — themed background, `4px 12px` padding, `4px` corner
//!   radius, pointer cursor.
//! * `checkbox()` — `16px` box, `gray_100` fill, `3px` radius, pointer
//!   cursor; checked state swaps fill to `blue_600` and shows a white
//!   lucide `check`.
//! * `radio()` — `16px` circle, `gray_300` ring, `2px` ring width,
//!   pointer cursor; checked state swaps the ring to `blue_600` and shows
//!   a `blue_600` dot at `55%` of the diameter.
//! * `input(...)` — `200px` width, white fill, `6px 8px` padding, `4px`
//!   radius, clipped overflow, `Text` cursor (or `NotAllowed` when
//!   disabled).
//! * `p(...)` — `1em` top/bottom margin (`16px` at default font size).
//!
//! These mirror what browsers give `<button>`, `<input type="checkbox">`,
//! `<input type="radio">`, `<input type="text">`, and `<p>`. `div()`,
//! `text(...)`, and `image(...)` have no UA defaults — they merge directly
//! against inherited + user style, matching `<div>`, `<span>`, and `<img>`.
//!
//! Because
//! [`merge_style`](crate::styling::Style::merge_style) cannot distinguish
//! "left at default" from "explicitly set to the default", UA defaults on
//! `padding` / `margin` / `radius` / `cursor` cannot be reset via
//! `.padding(0.0)` etc. Every element with UA defaults exposes
//! `.no_default_style()` to bypass them entirely.
//!
//! ## Semantic events
//!
//! Pointer and focus events are exposed twice:
//!
//! 1. **Raw** events via
//!    [`EventHandlersExt`](freya::prelude::EventHandlersExt): `.on_press`,
//!    `.on_mouse_down`, `.on_key_down`, `.on_pointer_enter`, and so on.
//!    These map 1:1 to the underlying element's event surface.
//!
//! 2. **Semantic** events via
//!    [`HasSemanticEvents`](crate::styling::HasSemanticEvents):
//!    `.on_hover`, `.on_hover_end`, `.on_pressed`, `.on_released`,
//!    `.on_focus`, `.on_blur`, `.on_focus_visible`, `.on_focus_visible_end`.
//!    These fire **once per state transition**, not on every pointer tick.
//!
//! Activation events are exposed only by the elements that own an
//! activation state:
//!
//! * `button()` — `.on_selected` / `.on_deselected`
//! * `checkbox()` — `.on_checked` / `.on_unchecked`
//! * `radio()` — `.on_checked` only (HTML semantics: clicking a checked
//!   radio is a no-op, so there is nothing to uncheck)
//! * `input(...)` — `.on_change` / `.on_submit` (edit / Enter)
//!
//! ## Controlled state
//!
//! Interactive state (`.with_selected(..)`, `.with_checked(..)`,
//! `.with_disabled(..)`, `.with_read_only(..)`) is **controlled**: the
//! element reads the prop, renders, and fires a callback; the caller is
//! responsible for flipping the prop in response. There is no internal
//! mutable state exposed to the user.
//!
//! The `with_` prefix on state props exists so `StyledExt`'s variant
//! methods (`.selected(..)`, `.checked(..)`, `.disabled(..)`) stay
//! callable for styling the same state:
//!
//! ```ignore
//! let selected = use_state(|| false);
//! button()
//!     .with_selected(*selected.read())                  // state
//!     .selected(|s| s.background(accent().into()))      // :selected variant
//!     .on_selected(move |_| selected.set(true))
//!     .on_deselected(move |_| selected.set(false))
//!     .child("Toggle")
//! ```
//!
//! Without the prefix, `.selected(bool)` and `.selected(Fn)` would collide
//! and method resolution would silently pick the inherent one.
//!
//! ## Known limitations
//!
//! Some elements approximate HTML because their implementation predates a
//! framework feature or because the underlying primitive doesn't expose
//! what's needed. These are documented here so contributors don't file
//! them as bugs.
//!
//! ### Checkbox and radio still use fill approximations
//!
//! `Style` gained a `border` field, so a real 1px outline is available to
//! any `DecorationExt` primitive. `checkbox()` and `radio()` currently
//! predate that and still paint their boxes/rings by **filling** their
//! parent `div()`:
//!
//! * **`checkbox()`** — box is filled (`gray_100` unchecked / `blue_600`
//!   checked) rather than outlined.
//! * **`radio()`** — the ring is approximated with three nested `div()`s:
//!   an outer filled circle, a transparent inner circle (the "hole"), and
//!   a small dot shown when checked.
//!
//! Both are visually correct and interactive, but they don't consume the
//! `Style::border` field — so a user override like
//! `.border_1().border_color(gray_400())` on a `checkbox()` or `radio()`
//! **is applied by the underlying `div()`**, but the element's own filled
//! visual may still dominate. Refactoring `checkbox.rs` / `radio.rs` to
//! use `.border(..)` for the outline is the natural next step (small —
//! replace the fill/3-div nesting with a single border draw).
//!
//! ### Focus hack for `input()`
//!
//! `Div` only marks an element focusable when a `:focus`-family variant
//! or a focus callback is registered. Elements that need focusability
//! without a focus callback (e.g. `input()`) register a **no-op** `:focus`
//! variant to force the flag:
//!
//! ```ignore
//! style = style.focus(|s| s);  // registers :focus → Div sees focusable
//! ```
//!
//! This is invisible at runtime but is a smell. A `Div::with_focusable(bool)`
//! setter would remove it; until then, any focusable element that doesn't
//! otherwise customize `:focus` carries this line.
//!
//! ### `:focus-within` approximation
//!
//! `div()` treats `:focus-within` as "self focused" — a `div` containing
//! a focused child does **not** match `:focus-within`. Proper support
//! needs descendant-focus tracking that `use_focus` doesn't currently
//! expose.
//!
//! ### HTML parity gaps
//!
//! * **`radio()`** — no `name`-based grouping (caller uses `use_state`
//!   to drive single-selection), no roving tabindex (each radio is
//!   independently tabbable). Matches the standard React / Vue / Dioxus
//!   pattern.
//! * **`input(...)`** — no selection (drag / shift-click / select-all),
//!   no clipboard (Ctrl+C/V/X), no IME preedit, no horizontal scrolling.
//!   Long text clips with ellipsis; the cursor beyond that edge is not
//!   visible.
//! * **`radio()` / `checkbox()`** — `a11y_checked` isn't wired through
//!   `AccessibilityData`; the role (`RadioButton` / `CheckBox`) is set
//!   but the checked state isn't announced separately. A future
//!   `a11y_builder(|node| node.set_checked(..))` call would complete it.
//! * **`input(...)`** — `type="email"` / `"url"` / `"tel"` / `"number"` /
//!   `"search"` all behave like `type="text"`. Only `type="text"` and
//!   `type="password"` are distinguished.
//!
//! ## Reset limitation
//!
//! [`Style::merge_style`](crate::styling::Style::merge_style) uses a
//! "differs from `Style::bare()`" test, so a field whose inherited value
//! happens to equal its bare default cannot be reset by a child setting it
//! back to that default. Every element that applies user-agent defaults
//! exposes a `no_default_style()` escape hatch for cases where this matters
//! (`padding: 0`, `margin: 0`, `radius: 0`, `cursor: default`).
//!
//! This also affects `border`: `Style::bare()` has `border: None`. A
//! `.border_none()` call sets `None`, which **doesn't** override an
//! existing border inside a variant — `None == None` skips the merge. To
//! visually clear a border in a `:hover` / `:focus` variant, use
//! `.border_0()` (sets `Some(zero-width Border)`), which does differ from
//! `None` and therefore wins.
//!
//! # Examples
//!
//! ## Container + text
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::{gray_50, gray_900};
//! use torin::gaps::Gaps;
//!
//! div()
//!     .background(gray_50())
//!     .padding(Gaps::new(16.0, 16.0, 16.0, 16.0))
//!     .rounded_md()
//!     .child(text("Hello, world").text_color(gray_900()))
//! ```
//!
//! ## Border
//!
//! Basic bordered card:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::gray_200;
//! use torin::gaps::Gaps;
//!
//! div()
//!     .border_1()
//!     .border_color(gray_200())
//!     .rounded_md()
//!     .padding(Gaps::new(16.0, 16.0, 16.0, 16.0))
//!     .child(text("Card content"))
//! ```
//!
//! Left accent stripe (per-side):
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::blue_500;
//! use torin::gaps::Gaps;
//!
//! div()
//!     .border_l(4.0)
//!     .border_color(blue_500())
//!     .padding(Gaps::new(12.0, 16.0, 12.0, 16.0))
//!     .child(text("Important note"))
//! ```
//!
//! Focus ring on a button:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::{blue_500, gray_300};
//!
//! button()
//!     .border_1()
//!     .border_color(gray_300())
//!     .focus(|s| s.border_2().border_color(blue_500()).border_outer())
//!     .child("Click me")
//! ```
//!
//! Border on an image:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::gray_300;
//!
//! image("avatar.png")
//!     .width(Size::px(48.0))
//!     .height(Size::px(48.0))
//!     .rounded_full()
//!     .border_2()
//!     .border_color(gray_300())
//! ```
//!
//! ## Rich text
//!
//! `p(..)` takes ordered spans and children, so styled runs and inline
//! elements interleave at the point they were added:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::{blue_600, gray_900};
//!
//! p("Read the ")
//!     .span(Span::new("documentation").color(blue_600()))
//!     .span(Span::new(" before you start."))
//!     .text_color(gray_900())
//! ```
//!
//! ## Button (basic + toggle)
//!
//! ```ignore
//! use freyacn::prelude::*;
//!
//! // Basic
//! button()
//!     .on_press(|_| println!("clicked"))
//!     .child("Submit")
//!
//! // Toggle (controlled)
//! let selected = use_state(|| false);
//! button()
//!     .with_selected(*selected.read())
//!     .on_selected(move |_| selected.set(true))
//!     .on_deselected(move |_| selected.set(false))
//!     .child(if *selected.read() { "On" } else { "Off" })
//! ```
//!
//! ## Checkbox
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::green_600;
//!
//! let accepted = use_state(|| false);
//! checkbox()
//!     .with_checked(*accepted.read())
//!     .on_checked(move |_| accepted.set(true))
//!     .on_unchecked(move |_| accepted.set(false))
//!     .checked(|s| s.background(green_600().into()))   // :checked style
//!     // optional custom icon:
//!     // .icon(text("check").font_size(12.0))
//! ```
//!
//! ## Radio
//!
//! HTML radio grouping is caller-driven (there is no `name` attribute).
//! Selection state lives in `use_state`:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use torin::prelude::Direction;
//!
//! #[derive(Clone, PartialEq)]
//! enum Size { Small, Medium, Large }
//!
//! let size = use_state(|| Size::Small);
//!
//! div()
//!     .direction(Direction::Vertical)
//!     .spacing(8.0)
//!     .child(
//!         radio()
//!             .with_checked(*size.read() == Size::Small)
//!             .on_checked(move |_| size.set(Size::Small))
//!     )
//!     .child(
//!         radio()
//!             .with_checked(*size.read() == Size::Medium)
//!             .on_checked(move |_| size.set(Size::Medium))
//!     )
//!     .child(
//!         radio()
//!             .with_checked(*size.read() == Size::Large)
//!             .on_checked(move |_| size.set(Size::Large))
//!     )
//! ```
//!
//! Custom color — pair `.checked(..)` (the ring) with `.dot_color(..)`
//! (the inner dot) so both stay consistent:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::green_600;
//!
//! radio()
//!     .with_checked(true)
//!     .checked(|s| s.background(green_600().into()))
//!     .dot_color(green_600())
//! ```
//!
//! ## Input
//!
//! Controlled text input:
//!
//! ```ignore
//! use freyacn::prelude::*;
//!
//! let name = use_state(String::new);
//! input(name.read().clone())
//!     .placeholder("Your name")
//!     .on_change(move |text| name.set(text))
//! ```
//!
//! Password + Enter-to-submit:
//!
//! ```ignore
//! use freyacn::prelude::*;
//!
//! let password = use_state(String::new);
//! input(*password.read())
//!     .mode(InputMode::password())
//!     .placeholder("••••••••")
//!     .on_change(move |text| password.set(text))
//!     .on_submit(|value| println!("submitted: {value}"))
//! ```
//!
//! Disabled / read-only / max length:
//!
//! ```ignore
//! input("admin").with_disabled(true)
//! input(*value.read()).with_read_only(true)
//! input(*code.read()).max_length(6).on_change(move |text| code.set(text))
//! ```
//!
//! Input with a focus border (default UA border plus user overrides):
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::{blue_500, gray_300};
//!
//! input(*value.read())
//!     .placeholder("Search…")
//!     .on_change(move |text| value.set(text))
//!     .border_1()
//!     .border_color(gray_300())
//!     .focus(|s| s.border_color(blue_500()))
//! ```
//!
//! ## Image
//!
//! ```ignore
//! use freyacn::prelude::*;
//!
//! image("assets/logo.png")
//!     .alt("Company logo")
//!     .width(Size::px(120.0))
//!     .height(Size::px(40.0))
//!     .fallback(text("Loading…"))
//! ```
//!
//! ## Composing elements
//!
//! Elements inherit text style and compose naturally — no glue code
//! required:
//!
//! ```ignore
//! use freyacn::prelude::*;
//! use crate::styling::colors::{gray_200, gray_500, gray_900, red_600};
//! use torin::gaps::Gaps;
//!
//! let accepted = use_state(|| false);
//! let email = use_state(String::new);
//!
//! div()
//!     .text_color(gray_900())          // inherited by text() below
//!     .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
//!     .border_1()
//!     .border_color(gray_200())
//!     .rounded_md()
//!     .child(
//!         p("Sign up for the newsletter.")
//!             .text_color(gray_500())
//!     )
//!     .child(
//!         input(email.read().clone())
//!             .placeholder("you@example.com")
//!             .on_change(move |text| email.set(text))
//!     )
//!     .child(
//!         div()
//!             .spacing(8.0)
//!             .direction(Direction::Horizontal)
//!             .child(
//!                 checkbox()
//!                     .with_checked(*accepted.read())
//!                     .on_checked(move |_| accepted.set(true))
//!                     .on_unchecked(move |_| accepted.set(false))
//!             )
//!             .child(
//!                 text("I agree to the terms")
//!                     .text_color(if *accepted.read() { gray_500() } else { red_600() })
//!             )
//!     )
//!     .child(
//!         button()
//!             .with_disabled(!*accepted.read())
//!             .on_press(|_| println!("submitted"))
//!             .child("Continue")
//!     )
//! ```

pub mod button;
pub mod checkbox;
pub mod div;
pub mod image;
pub mod input;
pub mod paragraph;
pub mod radio;
pub mod text;

pub use button::*;
pub use checkbox::*;
pub use div::*;
pub use image::*;
pub use input::*;
pub use paragraph::*;
pub use radio::*;
pub use text::*;
