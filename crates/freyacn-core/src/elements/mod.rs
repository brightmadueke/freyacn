//! Built-in UI elements.
//!
//! | Element | Constructor | HTML analogue |
//! |---|---|---|
//! | [`Div`] | [`div()`] | `<div>` |
//! | [`Text`] | [`text(...)`](text()) | `<span>` (leaf text) |
//! | [`Paragraph`] | [`p(...)`](p()) | `<p>` |
//! | [`Image`] | [`image(...)`](image()) | `<img>` |
//! | [`Icon`] | [`icon(...)`](icon()) | `<svg>` |
//! | [`Button`] | [`button()`] | `<button>` |
//! | [`Checkbox`] | [`checkbox()`] | `<input type="checkbox">` |
//! | [`Radio`] | [`radio()`] | `<input type="radio">` |
//! | [`Input`] | [`input(...)`](input()) | `<input type="text">` |
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
//! [`Style`](crate::styling::Style) has an `Option<Border>` field, so borders
//! are a first-class style property. Support depends on the primitive the
//! element renders:
//!
//! | Element | Supports `.border_*()` | Renderer |
//! |---|---|---|
//! | `div()` | ✓ | `apply_to_rect` → `DecorationExt` |
//! | `button()` | ✓ | `apply_to_rect` |
//! | `checkbox()` | ✓ | built on `div()` |
//! | `radio()` | ✓ | built on `div()` |
//! | `input(...)` | ✓ | built on `div()` |
//! | `image(...)` | ✓ | local `apply_style` |
//! | `icon(...)` | ✗ | `SvgViewer` has no `DecorationExt` |
//! | `text(...)` | ✗ | `Label` has no `DecorationExt` |
//! | `p(...)` | ✗ | `Paragraph` has no `DecorationExt` |
//!
//! For `text()`, `p()`, and `icon()`, wrap in a `div()` — the `div` owns
//! the border, the child lives inside:
//!
//! ```
//! div()
//!     .border_1()
//!     .border_color(gray_200())
//!     .child(text("Hello"))
//! ```
//!
//! Border does **not** inherit. Available Tailwind-style utilities:
//! `.border_1()` / `_2()` / `_4()` / `_8()` / `_0()`, `.border_t(..)` /
//! `_r` / `_b` / `_l` for per-side, `.border_x(..)` / `_y(..)` for opposite
//! pairs, `.border_color(..)`, plus freya-specific `.border_inner()` /
//! `_outer()` / `_center()` for `BorderAlignment`.
//!
//! ## Scrolling
//!
//! Freya's `Overflow` enum only has `None` and `Clip` — no `Scroll`. To fill
//! that gap, `div()` wraps its children in a `ScrollView` when scrolling is
//! enabled, while keeping the outer `rect()` (and all its styling) intact:
//!
//! | Method | Effect |
//! |---|---|
//! | `.scroll_y()` | Vertical scroll, internal controller, scrollbar shown |
//! | `.scroll_x()` | Horizontal scroll, internal controller, scrollbar shown |
//! | `.with_scroll_controller(c)` | External controller for `scroll_to_x` / `scroll_to_y` |
//! | `.show_scrollbar(bool)` | Toggle the scrollbar (default `true`) |
//! | `.hide_scrollbar()` | Shorthand for `.show_scrollbar(false)` |
//!
//! The outer `Div`'s size bounds the viewport — set `.height(..)` /
//! `.max_height(..)` for `scroll_y()`, `.width(..)` / `.max_width(..)` for
//! `scroll_x()`. Without a bound, the viewport sizes to content and no
//! scroll happens.
//!
//! ```
//! div()
//!     .height(Size::px(200.0))
//!     .scroll_y()
//!     .children(items)
//! ```
//!
//! Non-scrolling `div()`s behave exactly as before — the internal
//! `ScrollController` is allocated (hooks run unconditionally) but unused.
//! `input()` uses the same pattern internally for horizontal scrolling.
//!
//! ## Icon colour inheritance
//!
//! `Icon` maps [`Style::text_color`](crate::styling::Style::text_color) to
//! [`SvgViewer::color`](freya::components::SvgViewer::color), which is the
//! SVG's `currentColor` slot. Lucide, heroicons, and most modern icon sets
//! paint their shapes with `stroke="currentColor"` or `fill="currentColor"`,
//! so an `icon(..)` inside any element with an effective text colour takes
//! that exact colour without per-icon configuration:
//!
//! ```ignore
//! Button::new()
//!     .variant(ButtonVariant::Default)   // class: `text-primary-foreground`
//!     .child(icon(lucide::plus()))       // icon inherits primaryForeground
//! ```
//!
//! For explicit overrides, `.color(..)` sets the whole SVG's `currentColor`
//! and `.fill(..)` / `.stroke(..)` set the shapes' paint directly. `.fill`
//! / `.stroke` bypass `currentColor`, which is the right choice for
//! two-tone icon sets.
//!
//! ```ignore
//! icon(lucide::heart())
//!     .fill(red_500())
//!     .stroke(white())
//!     .stroke_width(1.5)
//!     .size(24.0)
//! ```
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
//! * `icon(...)` — `16px` square (`width` and `height` both `16px`);
//!   `currentColor` set from the inherited text colour. Override with
//!   `.size(n)` / `.w(n).h(n)`.
//! * `p(...)` — `1em` top/bottom margin (`16px` at default font size).
//!
//! These mirror what browsers give `<button>`, `<input type="checkbox">`,
//! `<input type="radio">`, `<input type="text">`, `<svg>`, and `<p>`.
//! `div()`, `text(...)`, and `image(...)` have no UA defaults — they merge
//! directly against inherited + user style, matching `<div>`, `<span>`, and
//! `<img>`.
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
//! Icons are **decorative by default** and do not fire activation events.
//! Use a `button()` around the icon if you need presses, or reach for
//! `HasSemanticEvents` directly if you want hover-only behaviour on a bare
//! icon.
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
//! ```
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
//! ## Accessibility
//!
//! Every interactive element sets an appropriate `AccessibilityRole`
//! (`Button`, `CheckBox`, `RadioButton`, `TextInput`, `PasswordInput`).
//!
//! Checked state on `checkbox()` / `radio()` is exposed through the node's
//! **description** (`"checked"` / `"unchecked"`, `"selected"` / `"not
//! selected"`) rather than `set_toggled`, because `accesskit::Toggled`
//! isn't reachable in the current dependency tree. The description
//! supplements a label set via `.a11y_alt(..)` — it doesn't replace it, so
//! custom labels survive.
//!
//! `input()` sets its `a11y_alt` to either the real value (text mode) or a
//! fixed-length mask (password mode). The raw password never flows through
//! `a11y_alt`.
//!
//! `icon(..)` forwards `.a11y_alt(..)` / `.a11y_role(..)` from the builder
//! to the underlying `SvgViewer`, so a meaningful icon (e.g. a standalone
//! logo) can be announced. Decorative icons inside a labelled container
//! need no alt — pass through `a11y_alt("")` to hide from AT, or leave
//! unset.
//!
//! ## Known limitations
//!
//! ### Implementation approximations
//!
//! * **`checkbox()` / `radio()` use fill approximations** for their boxes
//!   and rings (predating `Style::border`). User `.border_*()` on them is
//!   applied by the underlying `div()`, but the filled visual dominates.
//!   Refactor to `.border(..)` is a small follow-up.
//! * **`a11y_checked` via description** — see the Accessibility section.
//!   The proper `set_toggled` call needs a direct `accesskit` dependency.
//! * **`:focus-within` is approximated** as "self focused" — a `div`
//!   containing a focused child doesn't match `:focus-within`. Needs
//!   descendant-focus tracking the framework doesn't expose.
//! * **`icon(..)` has no box-model overrides** — `SvgViewer` exposes
//!   `width` / `height` / `margin` / `position` but not `padding` /
//!   `min_*` / `max_*` / `border`. Wrap in a `div()` if you need them.
//!
//! ### HTML parity gaps
//!
//! * **`radio()`** — no `name`-based grouping (caller uses `use_state`
//!   to drive single-selection), no roving tabindex (each radio is
//!   independently tabbable). Matches the standard React / Vue / Dioxus
//!   pattern.
//! * **`input(...)`** — `type="email"` / `"url"` / `"tel"` / `"number"` /
//!   `"search"` all behave like `type="text"`. Only `type="text"` and
//!   `type="password"` are distinguished. Password mode opts out of
//!   selection because mask ↔ value misalign on emoji.
//! * **`input(...)`** — clipboard shortcuts (Ctrl+C/V/X) and selection
//!   (drag, shift+arrow, Ctrl+A) **are implemented** — this paragraph
//!   exists to point at the remaining gaps: no right-click context menu,
//!   no undo/redo history, no drag-autoscroll.
//!
//! ### Reset limitation
//!
//! [`Style::merge_style`](crate::styling::Style::merge_style) uses a
//! "differs from `Style::bare()`" test, so a field whose inherited value
//! happens to equal its bare default cannot be reset by a child setting it
//! back to that default. Every element that applies user-agent defaults
//! exposes a `no_default_style()` escape hatch. This also affects `border`:
//! use `.border_0()` (not `.border_none()`) to visually clear a border
//! inside a variant.
//!
//! # Examples
//!
//! ## Container + text
//!
//! ```
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
//! ## Scrolling
//!
//! Vertical list bounded to 200px:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! div()
//!     .height(Size::px(200.0))
//!     .scroll_y()
//!     .spacing(4.0)
//!     .children(items.iter().map(|item| text(item)))
//! ```
//!
//! Horizontal tag row:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! div()
//!     .width(Size::px(300.0))
//!     .scroll_x()
//!     .hide_scrollbar()
//!     .spacing(8.0)
//!     .children(tags.iter().map(|t| tag(t)))
//! ```
//!
//! Programmatic scroll with a controller:
//!
//! ```
//! use freyacn::prelude::*;
//! use freya::components::{ScrollConfig, use_scroll_controller};
//!
//! let controller = use_scroll_controller(ScrollConfig::default);
//!
//! div()
//!     .height(Size::px(200.0))
//!     .with_scroll_controller(controller)
//!     .children(items)
//!
//! // From a button handler, say:
//! // controller.scroll_to_y(0);
//! ```
//!
//! ## Border
//!
//! Basic bordered card:
//!
//! ```
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
//! Focus ring via border:
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::{blue_500, gray_300};
//!
//! button()
//!     .border_1()
//!     .border_color(gray_300())
//!     .focus_visible(|s| {
//!         s.border_2()
//!             .border_color(blue_500())
//!             .border_outer()
//!     })
//!     .child("Click me")
//! ```
//!
//! ## Rich text
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::{blue_600, gray_900};
//!
//! p("Read the ")
//!     .span(Span::new("documentation").color(blue_600()))
//!     .span(Span::new(" before you start."))
//!     .text_color(gray_900())
//! ```
//!
//! ## Icon
//!
//! Icons inherit text colour from their container — no per-icon override
//! needed inside a `Button`, `Div`, or `Text`:
//!
//! ```
//! use freyacn::prelude::*;
//! use freya::icons::lucide;
//! use crate::styling::colors::blue_600;
//!
//! // Inside a button — icon picks up the button's text colour.
//! Button::new()
//!     .child(icon(lucide::plus()))
//!     .child(text("New"))
//!
//! // Standalone with explicit colour / size.
//! icon(lucide::heart())
//!     .color(blue_600())
//!     .size(24.0)
//!
//! // Two-tone — bypass `currentColor` entirely.
//! icon(lucide::star())
//!     .fill(blue_600())
//!     .stroke(white())
//!     .stroke_width(1.5)
//! ```
//!
//! Icon inside a bordered card (border goes on a `div`, not the icon):
//!
//! ```
//! div()
//!     .border_1()
//!     .border_color(gray_200())
//!     .rounded_md()
//!     .p_2()
//!     .child(icon(lucide::info()))
//! ```
//!
//! ## Button (basic + toggle)
//!
//! ```
//! use freyacn::prelude::*;
//!
//! button()
//!     .on_press(|_| println!("clicked"))
//!     .child("Submit")
//!
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
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::green_600;
//!
//! let accepted = use_state(|| false);
//! checkbox()
//!     .with_checked(*accepted.read())
//!     .on_checked(move |_| accepted.set(true))
//!     .on_unchecked(move |_| accepted.set(false))
//!     .checked(|s| s.background(green_600().into()))
//! ```
//!
//! ## Radio
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::green_600;
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
//! Custom color — pair `.checked(...)` (the ring) with `.dot_color(...)`
//! (the inner dot):
//!
//! ```
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
//! ```
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
//! ```
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
//! ```
//! input("admin").with_disabled(true)
//! input(*value.read()).with_read_only(true)
//! input(*code.read()).max_length(6).on_change(move |text| code.set(text))
//! ```
//!
//! ## Image
//!
//! ```
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
//! Elements inherit text style and compose naturally:
//!
//! ```
//! use freyacn::prelude::*;
//! use freya::icons::lucide;
//! use crate::styling::colors::{gray_200, gray_500, gray_900, red_600};
//! use torin::gaps::Gaps;
//!
//! let accepted = use_state(|| false);
//! let email = use_state(String::new);
//!
//! div()
//!     .text_color(gray_900())
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
//!             .child(icon(lucide::check()))
//!             .child("Continue")
//!     )
//! ```

pub mod button;
pub mod checkbox;
pub mod div;
pub mod icon;
pub mod image;
pub mod input;
pub mod paragraph;
pub mod radio;
pub mod text;

pub use button::*;
pub use checkbox::*;
pub use div::*;
pub use icon::*;
pub use image::*;
pub use input::*;
pub use paragraph::*;
pub use radio::*;
pub use text::*;
