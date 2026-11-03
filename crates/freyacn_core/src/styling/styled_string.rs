//! Inline Tailwind-style class strings for [`StyledExt`].
//!
//! `StyledString` lets you write component styling as a single string
//! rather than a chain of method calls:
//!
//! ```
//! text("Hello")
//!     .style("p-4 w-1/2 text-lg font-semibold text-blue-500/80 hover:bg-blue-100 -mt-2")
//! ```
//!
//! The alternative, equivalent form is the fluent API:
//!
//! ```
//! text("Hello")
//!     .p(16.0)
//!     .w_1_2()
//!     .text_lg()
//!     .font_semibold()
//!     .text_color(blue_500().opacity(0.8))
//!     .hover(|s| s.bg(blue_100()))
//!     .mt(-8.0)
//! ```
//!
//! Both produce the same `Style`. `StyledString` is useful when the class
//! list comes from data (theme files, user input, generated code) or when
//! you'd rather keep styling next to the markup in a single string.
//!
//! # Grammar
//!
//! One whitespace-separated token per class:
//!
//! ```text
//! class    := [variant ":"] ["-"] utility
//! variant  := hover | focus | focus-visible | focus-within | active
//!           | disabled | checked | selected
//! utility  := prefix "-" value | keyword
//! value    := scale number | fraction | keyword | "[" arbitrary "]" | color ["/" alpha]
//! ```
//!
//! - An optional leading `-` negates utilities that support it (margins,
//!   rotation, scale, z-index).
//! - An optional `<variant>:` prefix scopes the utility to an interaction
//!   state (`hover:bg-red-500`).
//! - Bracket values (`[...]`) escape otherwise-special characters such as
//!   `/`, `:` and whitespace in arbitrary values (`w-[calc(100%-2rem)]`,
//!   `gap-[ 10px ]`).
//!
//! # Class reference
//!
//! ## Layout
//!
//! | Class                         | Result                                           |
//! |-------------------------------|--------------------------------------------------|
//! | `flex-row` / `flex-col`       | `direction(Horizontal / Vertical)`               |
//! | `justify-start` … `justify-evenly` | `main_align(...)`                           |
//! | `items-start` / `items-center` / `items-end` | `cross_align(...)`                 |
//! | `absolute` / `relative`       | `position = absolute / stacked`                  |
//!
//! ## Spacing (padding / margin / gap)
//!
//! ```text
//! p-<n>  px-<n>  py-<n>  pt-<n>  pr-<n>  pb-<n>  pl-<n>
//! m-<n>  mx-<n>  my-<n>  mt-<n>  mr-<n>  mb-<n>  ml-<n>  -m-…  -mx-…
//! gap-<n>
//! ```
//!
//! `<n>` is the Tailwind spacing scale: `0` = 0px, `1` = 4px, `1.5` =
//! 6px, `4` = 16px, `px` = 1px, or an arbitrary value like `[13px]`.
//! Negative margins use a leading `-` (`-mt-2` = −8px).
//!
//! ## Dimensions
//!
//! ```text
//! w-<v>  h-<v>  size-<v>
//! min-w-<v>  min-h-<v>  max-w-<v>  max-h-<v>
//! ```
//!
//! `<v>` accepts:
//!
//! - The spacing scale (`w-4` = 16px)
//! - Fractions (`w-1/2` = 50%)
//! - Keywords (`w-full` = 100%, `w-screen` = 100%, `w-auto` = content, `w-fit` = content)
//! - Arbitrary values (`w-[300px]`, `w-[50%]`, `w-[ 300px ]`)
//! - For `max-w-*` only: the container scale (`max-w-xs` = 320px, `max-w-7xl` = 1280px)
//!
//! ## Colour
//!
//! ```text
//! bg-<color>          text-<color>          border-<color>
//! bg-<color>/<a>      text-<color>/<a>      border-<color>/<a>
//! ```
//!
//! `<color>` is a palette name (`blue-500`, `neutral-950`, `red-600`,
//! `white`, `black`, `transparent`) or an arbitrary hex string
//! (`[#ff0000]`, `[#fff]`, `[#ff000080]`).
//!
//! `<a>` is an alpha in `0..=100` (`/50` = 50% opacity). It **multiplies**
//! whatever alpha the source colour already has, matching Tailwind's
//! `bg-blue-500/50` convention for opaque colours.
//!
//! ## Typography
//!
//! ```text
//! text-xs  text-sm  text-base  text-md  text-lg  text-xl  text-2xl … text-9xl  text-[14px]
//! text-left  text-center  text-right  text-justify  text-start  text-end
//! text-clip  text-ellipsis
//! font-thin  font-extralight  font-light  font-normal  font-medium
//! font-semibold  font-bold  font-extrabold  font-black
//! font-sans  font-serif  font-mono
//! font-stretch-ultra-condensed … font-stretch-ultra-expanded
//! italic  not-italic
//! underline  overline  line-through  no-underline
//! tracking-tighter … tracking-widest  tracking-[0.05em]
//! leading-none … leading-loose  leading-[1.6]
//! text-shadow-none  text-shadow-2xs  text-shadow-xs  text-shadow-sm
//! text-shadow-md  text-shadow-lg
//! tabular-nums  slashed-zero  diagonal-fractions  small-caps
//! truncate
//! ```
//!
//! `tracking-*` is applied **after** any `text-*` font-size class, so
//! `"tracking-wide text-lg"` and `"text-lg tracking-wide"` produce
//! identical results (tracking is a multiplier of the font size).
//!
//! ## Effects
//!
//! ```text
//! opacity-0  opacity-5 … opacity-100  opacity-[0.42]
//! rotate-0  rotate-1 … rotate-180  rotate-[45deg]  -rotate-45
//! scale-0  scale-50 … scale-200  scale-[1.25]
//! scale-x-<n>  scale-y-<n>  -scale-…  -scale-x-…  -scale-y-…
//! origin-center  origin-top  origin-top-right  origin-right
//! origin-bottom-right  origin-bottom  origin-bottom-left
//! origin-left  origin-top-left
//! backdrop-blur  backdrop-blur-none  backdrop-blur-2xs … backdrop-blur-3xl
//! backdrop-blur-[10px]
//! overflow-hidden  overflow-clip  overflow-visible
//! cursor-<name>  cursor-none
//! ```
//!
//! ## Borders
//!
//! ```text
//! border               1px all edges
//! border-0 … border-8   width in px (0, 2, 4, 8 are the standard steps)
//! border-[3px]         arbitrary width
//! border-<color>       1px + colour (`border-red-500`, `border-[#f00]`)
//! border-<edge>        1px on that edge/axis (t/r/b/l/x/y)
//! border-<edge>-<n>    width on that edge (`border-t-2`, `border-x-4`)
//! border-<edge>-<col>  sets the border fill colour (`border-t-red-500`)
//! border-none          remove the border object
//! border-inner / border-outer / border-center   alignment
//! ```
//!
//! **Width and colour are independent ops.** `border-2 border-red-500` and
//! `border-red-500 border-2` produce identical results, because
//! `border-red-500` only touches the fill (creating a 1px border if none
//! exists) and `border-2` only touches the width.
//!
//! **Per-side colours aren't possible** in Freya — `Border` has a single
//! `Fill` shared by all sides. `border-t-red-500` sets that single fill,
//! so it effectively colours every visible side. The `t` edge is only
//! meaningful for width classes (`border-t-2`).
//!
//! ## Layers (z-index)
//!
//! ```text
//! z-10  z-[5]  z-auto  -z-10
//! z-overlay  z-overlay-3
//! ```
//!
//! Values map to Freya's `Layer`, which is **relative to the parent's
//! layer** — not a global CSS z-index. `z-overlay` / `z-overlay-N` produce
//! a layer above all siblings, which is what you want for modals and
//! dropdowns.
//!
//! Out-of-range values are **rejected**, not wrapped: `z-40000` and
//! `z-overlay-300` are dropped with a debug warning rather than silently
//! becoming wrong values.
//!
//! ## Images
//!
//! ```text
//! object-contain  object-cover  object-fill  object-none  object-scale-down
//! object-center  object-top-left  object-left-top
//! image-auto  image-pixelated  image-smooth
//! sampling-nearest  sampling-bilinear  sampling-trilinear
//! sampling-mitchell  sampling-catmull-rom
//! snap-to-grid  no-snap-to-grid
//! blur  blur-none  blur-2xs  blur-xs  blur-sm  blur-md  blur-lg
//! blur-xl  blur-2xl  blur-3xl  blur-[10px]
//! ```
//!
//! Image classes write to `Style`'s image fields, so variants work on
//! them too (`hover:blur-md`).
//!
//! # Variants
//!
//! The eight supported variants map 1:1 to the `StyledExt` methods of the
//! same name:
//!
//! | Class prefix      | Method              |
//! |-------------------|---------------------|
//! | `hover:`          | `Style::hover`      |
//! | `focus:`          | `Style::focus`      |
//! | `focus-visible:`  | `Style::focus_visible` |
//! | `focus-within:`   | `Style::focus_within`  |
//! | `active:`         | `Style::active`     |
//! | `disabled:`       | `Style::disabled`   |
//! | `checked:`        | `Style::checked`    |
//! | `selected:`       | `Style::selected`   |
//!
//! **Stacked variants are not supported.** `hover:focus:bg-red-500` fails
//! to parse: the splitter takes the *first* top-level `:` and treats the
//! remainder (`focus:bg-red-500`) as a single utility, which doesn't
//! match any prefix. Use the fluent API if you need combined states.
//!
//! # Arbitrary values
//!
//! Bracket values allow characters that would otherwise terminate the
//! token (`:`, `/`, whitespace inside brackets, `#`, …). They support:
//!
//! - **Pixels**: `[10px]`, `[3px]`
//! - **Rems**: `[1.5rem]` (converted as `1.5 × 16px`)
//! - **Unitless**: `[10]` (lenient — treated as `10px`; Tailwind requires
//!   a unit, but FreyaCN accepts both for backwards compatibility)
//! - **Percent** (dimensions only): `w-[50%]`
//! - **Hex colours**: `bg-[#ff0000]`, `text-[#fff]`, `border-[#ff000080]`
//!
//! **Interior whitespace is allowed.** The class splitter is bracket-aware,
//! so `[ 10px ]` and `[ calc(100% - 2rem) ]` are single tokens — the
//! leading/trailing whitespace inside the brackets is trimmed before
//! parsing.
//!
//! Arbitrary **text-shadows** (`text-shadow-[0_2px_4px_#000]`) are **not**
//! supported — only the named sizes. Parsing arbitrary CSS shadow syntax
//! would require a full colour + offset + blur parser.
//!
//! # Debug warnings
//!
//! Unknown classes are ignored, and in debug builds they're reported on
//! stderr:
//!
//! ```text
//! [StyledString] unknown or unsupported class `foo-bar`
//! ```
//!
//! This is a no-op in release builds.
//!
//! # Performance
//!
//! Parsing happens **once** per `StyledString::parse` call. The base ops
//! are applied as a `Vec<Op>`; the variant utilities are re-parsed on
//! each variant invocation because `Op` isn't `Send + Sync` (it holds
//! `Rc`-based Freya types). This is fine for short strings — a hover
//! handler re-parsing a dozen classes costs microseconds.

use crate::styling::colors::{self as c, StyledColor};
use crate::styling::style::Style;
use crate::styling::styled::StyledExt;
use freya_core::data::Overflow;
use freya_core::elements::image::{AspectRatio, ImageCover, SamplingMode};
use freya_core::prelude::{
    CursorIcon, FontFeature, FontSlant, FontWeight, FontWidth, Layer, TextAlign, TextDecoration,
    TextOverflow, TransformOrigin,
};
use std::borrow::Cow;
use std::sync::Arc;
use torin::prelude::{Alignment, Direction, Length};
use torin::size::Size;

// ============================================================================
// Public type
// ============================================================================

/// A parsed Tailwind-style class string.
///
/// Build one with [`StyledString::parse`], the `From<&str>` impl, or just
/// pass a string to [`StyledExt::style`], then apply it with
/// [`StyledString::apply`] or let [`StyledExt::style`] do it for you.
///
/// # Structure
///
/// Internally, `StyledString` holds two collections:
///
/// - `base: Vec<Op>` — utilities that apply unconditionally, in write
///   order (except `tracking-*`, which is sorted last — see
///   [`Op::is_tracking`]).
/// - `variants: Vec<(VariantKind, Arc<[String]>)>` — utilities grouped by
///   their state prefix. Each group is stored as raw strings because the
///   parsed `Op` isn't `Send + Sync`; the closure re-parses them when the
///   variant fires. `Arc<[String]>` avoids cloning the list on every
///   `apply()`.
///
/// # Examples
///
/// ## From a string literal
///
/// ```
/// # use freya_cn::styling::styled_string::StyledString;
/// let classes = StyledString::from("p-4 text-lg hover:bg-blue-100");
/// ```
///
/// ## Reuse across many elements
///
/// ```
/// # use freya_cn::styling::styled_string::StyledString;
/// # use freya_cn::styling::styled::StyledExt;
/// # use freya_cn::styling::style::Style;
/// let button_style = StyledString::from("px-4 py-2 rounded-md bg-blue-500 hover:bg-blue-600");
///
/// let style_a = button_style.apply(Style::default());
/// let style_b = button_style.apply(Style::default());
/// // `button_style` can be reused without re-parsing.
/// ```
///
/// ## Dynamic strings
///
/// `StyledString` accepts anything that converts into `String`, so
/// runtime-formatted class lists work:
///
/// ```
/// # use freya_cn::styling::styled_string::StyledString;
/// let size = "lg";
/// let classes = StyledString::from(format!("p-4 text-{size} font-semibold"));
/// ```
#[derive(Clone, Default)]
pub struct StyledString {
    /// Unconditional utilities, in write order except for `tracking-*`
    /// (sorted last). See [`Op::is_tracking`] for why.
    base: Vec<Op>,

    /// Variant utilities, grouped by kind. `Arc<[String]>` avoids cloning
    /// the list on every `apply()`. The `String`s are re-parsed inside the
    /// closure because `Op` isn't `Send + Sync`.
    variants: Vec<(VariantKind, Arc<[String]>)>,
}

impl StyledString {
    /// Parses a whitespace-separated list of Tailwind-style class names.
    ///
    /// Unknown classes and unsupported syntax are skipped (with a debug
    /// warning on stderr in `debug_assertions` builds). Empty input
    /// produces an empty `StyledString` whose `apply()` is a no-op.
    ///
    /// # Splitting
    ///
    /// Classes are split by [`split_classes`], which respects `[...]`
    /// brackets. `"gap-[ 10px ]"` is a single class (not three), and the
    /// interior whitespace is trimmed by the individual value parsers.
    ///
    /// # Ordering
    ///
    /// Base utilities are applied in the order they appear, with one
    /// exception: `tracking-*` classes are always applied **after**
    /// any `text-*` font-size classes, since tracking is a multiplier of
    /// the current font size. This makes `"tracking-wide text-lg"` and
    /// `"text-lg tracking-wide"` equivalent — see the
    /// `tracking_is_order_independent` test.
    ///
    /// # Variant grouping
    ///
    /// All classes with the same variant prefix are grouped into a single
    /// closure. `"hover:bg-red-500 hover:text-white"` registers **one**
    /// `hover` variant that applies both utilities, not two overlapping
    /// ones. The order within a group is preserved.
    ///
    /// # Example
    ///
    /// ```
    /// # use freya_cn::styling::styled_string::StyledString;
    /// let s = StyledString::parse("p-4 text-lg hover:bg-blue-100");
    /// assert!(!s.is_empty());
    ///
    /// let empty = StyledString::parse("not-a-class");
    /// assert!(empty.is_empty());
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut this = Self::default();
        let mut variant_classes: Vec<(VariantKind, Vec<String>)> = Vec::new();

        for class in split_classes(input) {
            let (variant, rest) = split_variant(class);
            let kind = match variant {
                None => None,
                Some(v) => match VariantKind::from_str(v) {
                    Some(k) => Some(k),
                    None => {
                        warn_unknown(class);
                        continue;
                    }
                },
            };
            let Some(op) = parse_utility(rest) else {
                warn_unknown(class);
                continue;
            };
            match kind {
                None => this.base.push(op),
                Some(k) => {
                    // `op` only validated the class; the closure re-parses
                    // the raw string because `Op` isn't `Send + Sync`.
                    drop(op);
                    match variant_classes.iter_mut().find(|(vk, _)| *vk == k) {
                        Some((_, classes)) => classes.push(rest.to_owned()),
                        None => variant_classes.push((k, vec![rest.to_owned()])),
                    }
                }
            }
        }

        this.variants = variant_classes
            .into_iter()
            .map(|(k, v)| (k, Arc::from(v.into_boxed_slice())))
            .collect();

        // `tracking-*` depends on the current font size, so always apply it
        // last regardless of write order.
        this.base.sort_by_key(Op::is_tracking);
        this
    }

    /// Returns `true` when **no** class was recognized.
    ///
    /// Useful for short-circuiting:
    ///
    /// ```
    /// # use freya_cn::styling::styled_string::StyledString;
    /// # use freya_cn::styling::styled::StyledExt;
    /// # use freya_cn::styling::style::Style;
    /// let s = StyledString::from("");
    /// let style = if s.is_empty() {
    ///     Style::default()
    /// } else {
    ///     s.apply(Style::default())
    /// };
    /// ```
    pub fn is_empty(&self) -> bool {
        self.base.is_empty() && self.variants.is_empty()
    }

    /// Applies every parsed class to `target` and returns it.
    ///
    /// Base utilities are applied first, then each variant is registered as
    /// a closure on the `Style`. The target's `StyleVariants` collection is
    /// what makes the variants fire later — this is why `target` must be a
    /// [`StyledExt`] implementor (only `Style` and things wrapping it
    /// qualify in practice).
    ///
    /// # Order guarantees
    ///
    /// - Base utilities apply in the order of [`Self::parse`], with the
    ///   `tracking-*` exception.
    /// - Variants apply **after** all base utilities, so `hover:bg-red-500`
    ///   overrides `bg-blue-500` on hover regardless of order in the
    ///   source string.
    /// - Variants fire in Freya's standard priority order when active
    ///   (`disabled` > `active` > `checked` > `selected` > `focus-within`
    ///   > `focus-visible` > `focus` > `hover`), independent of the order
    ///   they were registered.
    ///
    /// # Example
    ///
    /// ```
    /// # use freya_cn::styling::styled_string::StyledString;
    /// # use freya_cn::styling::style::Style;
    /// let style = StyledString::from("p-4 text-lg hover:bg-blue-100")
    ///     .apply(Style::default());
    /// assert_eq!(style.padding.top(), 16.0);
    /// ```
    pub fn apply<T: StyledExt>(&self, target: T) -> T {
        let mut t = apply_ops(target, &self.base);
        for (kind, classes) in &self.variants {
            let classes = Arc::clone(classes);
            let f = move |s: Style| {
                let mut ops: Vec<Op> = classes.iter().filter_map(|c| parse_utility(c)).collect();
                ops.sort_by_key(Op::is_tracking);
                apply_ops(s, &ops)
            };
            t = match kind {
                VariantKind::Hover => t.hover(f),
                VariantKind::Focus => t.focus(f),
                VariantKind::FocusVisible => t.focus_visible(f),
                VariantKind::FocusWithin => t.focus_within(f),
                VariantKind::Active => t.active(f),
                VariantKind::Disabled => t.disabled(f),
                VariantKind::Checked => t.checked(f),
                VariantKind::Selected => t.selected(f),
            };
        }
        t
    }
}

// ---- From impls ----

/// `From<&str>` — parses immediately.
impl From<&str> for StyledString {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

/// `From<String>` — parses immediately.
impl From<String> for StyledString {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}

/// `From<&String>` — parses immediately; useful in `for class in &vec` loops.
impl From<&String> for StyledString {
    fn from(s: &String) -> Self {
        Self::parse(s)
    }
}

/// `From<Cow<'static, str>>` — parses immediately. Compatible with strings
/// produced by `format!` and by `include_str!`-based static data.
impl From<Cow<'static, str>> for StyledString {
    fn from(s: Cow<'static, str>) -> Self {
        Self::parse(&s)
    }
}

/// Emit a debug-only warning for an unrecognized class.
///
/// Silent in release builds. Uses `eprintln!` directly (no logger
/// dependency) since this only fires during development.
fn warn_unknown(_class: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[StyledString] unknown or unsupported class `{_class}`");
}

// ============================================================================
// Splitting
// ============================================================================

/// Split a class list on whitespace, **respecting `[...]` brackets**.
///
/// `str::split_whitespace` breaks arbitrary values that contain spaces:
/// `"gap-[ 10px ]"` becomes three tokens (`"gap-["`, `"10px"`, `"]"`),
/// all of which fail to parse. This splitter skips whitespace while the
/// bracket depth is positive, so `[ 10px ]` stays part of the surrounding
/// class.
///
/// # Depth handling
///
/// Nested `[...]` are tracked correctly. An unmatched `]` is clamped at
/// zero — the same policy as [`split_variant`] — so a stray `]` doesn't
/// corrupt later splits.
///
/// # Examples
///
/// ```ignore
/// split_classes("p-4 text-lg")               → ["p-4", "text-lg"]
/// split_classes("gap-[ 10px ]")              → ["gap-[ 10px ]"]
/// split_classes("p-4  w-[ a b ]  text-lg")   → ["p-4", "w-[ a b ]", "text-lg"]
/// split_classes("p-4 ] text-lg")             → ["p-4", "]", "text-lg"]
/// ```
fn split_classes(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;

    for (i, ch) in input.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            c if c.is_whitespace() && depth == 0 => {
                if i > start {
                    out.push(&input[start..i]);
                }
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }

    if start < input.len() {
        out.push(&input[start..]);
    }
    out
}

// ============================================================================
// Variants
// ============================================================================

/// The interaction states a class can be scoped to (`hover:` prefix, …).
///
/// One variant per `StyledExt` state method. Each maps to a distinct
/// closure registration on the target style; Freya's variant priority
/// system decides which one wins when multiple are simultaneously active.
///
/// # Example
///
/// All eight can be used together without conflict:
///
/// ```
/// # use freya_cn::styling::styled_string::StyledString;
/// let s = StyledString::from(
///     "bg-white \
///      hover:bg-gray-50 \
///      focus:ring-2 \
///      focus-visible:ring-blue-500 \
///      active:scale-95 \
///      disabled:opacity-50 \
///      checked:bg-blue-500 \
///      selected:bg-blue-100"
/// );
/// assert!(!s.is_empty());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
enum VariantKind {
    /// `hover:`
    Hover,
    /// `focus:`
    Focus,
    /// `focus-visible:`
    FocusVisible,
    /// `focus-within:`
    FocusWithin,
    /// `active:`
    Active,
    /// `disabled:`
    Disabled,
    /// `checked:`
    Checked,
    /// `selected:`
    Selected,
}

impl VariantKind {
    /// Map the variant prefix (sans trailing `:`) to its kind.
    ///
    /// Returns `None` for unknown prefixes, which the caller treats as an
    /// unknown class. Notably, stacked variants (`hover:focus:bg-red-500`)
    /// fail here because the splitter takes only the first `:`, so the
    /// remainder starts with `focus:` which isn't a recognized variant
    /// prefix on its own after the split.
    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "hover" => Self::Hover,
            "focus" => Self::Focus,
            "focus-visible" => Self::FocusVisible,
            "focus-within" => Self::FocusWithin,
            "active" => Self::Active,
            "disabled" => Self::Disabled,
            "checked" => Self::Checked,
            "selected" => Self::Selected,
            _ => return None,
        })
    }
}

/// Split a class token into its variant prefix (if any) and utility part.
///
/// Splits on the **first** top-level `:` — one that isn't inside brackets.
/// Returns `(Some("hover"), "bg-red-500")` for `"hover:bg-red-500"`, and
/// `(None, "w-[calc(1:2)]")` for `"w-[calc(1:2)]"` (the `:` inside brackets
/// isn't a variant separator).
///
/// # Bracket handling
///
/// Depth tracking ensures `:` inside `[...]` isn't treated as a separator.
/// A stray unmatched `]` (depth would go negative) is clamped — this
/// prevents a malformed class like `"x]:hover:bg-red"` from hiding the
/// `hover:` variant behind a bogus depth.
///
/// # Example
///
/// ```ignore
/// split_variant("hover:bg-red-500")       → (Some("hover"), "bg-red-500")
/// split_variant("w-[calc(1:2)]")          → (None, "w-[calc(1:2)]")
/// split_variant("x]:hover:bg-red-500")    → (Some("x]"), "hover:bg-red-500")
/// split_variant("bg-red-500")             → (None, "bg-red-500")
/// ```
fn split_variant(class: &str) -> (Option<&str>, &str) {
    let mut depth = 0i32;
    for (i, ch) in class.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                // Clamp at zero: an unmatched `]` shouldn't hide later `:`
                // characters that should be treated as variant separators.
                if depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => return (Some(&class[..i]), &class[i + 1..]),
            _ => {}
        }
    }
    (None, class)
}

// ============================================================================
// Operations
// ============================================================================

/// Which side(s) a padding or margin utility applies to (`p-4` vs `pt-4`
/// vs `px-4`).
#[derive(Clone, Copy)]
enum Side {
    /// All four sides (`p-*`, `m-*`).
    All,
    /// Left + right (`px-*`, `mx-*`).
    X,
    /// Top + bottom (`py-*`, `my-*`).
    Y,
    /// Top only (`pt-*`, `mt-*`).
    T,
    /// Right only (`pr-*`, `mr-*`).
    R,
    /// Bottom only (`pb-*`, `mb-*`).
    B,
    /// Left only (`pl-*`, `ml-*`).
    L,
}

/// Which corner(s) a `rounded-*` utility applies to.
///
/// Unlike [`BorderEdge`], radius has per-corner variants (`tl`, `tr`, `br`,
/// `bl`) but no axis variants (`x` / `y`) — matching `StyledExt::radius_*`.
#[derive(Clone, Copy)]
enum Edge {
    /// All four corners (`rounded-md`).
    All,
    /// Top edge (`rounded-t-md`).
    T,
    /// Right edge (`rounded-r-md`).
    R,
    /// Bottom edge (`rounded-b-md`).
    B,
    /// Left edge (`rounded-l-md`).
    L,
    /// Top-left corner (`rounded-tl-md`).
    Tl,
    /// Top-right corner (`rounded-tr-md`).
    Tr,
    /// Bottom-right corner (`rounded-br-md`).
    Br,
    /// Bottom-left corner (`rounded-bl-md`).
    Bl,
}

/// Which edge(s) a `border-*` width utility applies to.
///
/// Unlike [`Edge`], borders distinguish `X` / `Y` (both sides of an axis)
/// from individual sides — matching `StyledExt::border_x` / `border_y`.
/// Borders have no per-corner variants; corners are painted implicitly
/// where edges meet.
///
/// Colours are **not** edge-scoped — Freya's `Border` has a single `Fill`,
/// so `border-t-red-500` sets the whole border's colour.
#[derive(Clone, Copy)]
enum BorderEdge {
    /// All four sides (`border`, `border-2`).
    All,
    /// Left + right (`border-x`, `border-x-2`).
    X,
    /// Top + bottom (`border-y`, `border-y-2`).
    Y,
    /// Top only (`border-t`, `border-t-2`).
    T,
    /// Right only (`border-r`, `border-r-2`).
    R,
    /// Bottom only (`border-b`, `border-b-2`).
    B,
    /// Left only (`border-l`, `border-l-2`).
    L,
}

/// Which dimension a `w-*` / `h-*` / `min-*` / `max-*` utility targets.
#[derive(Clone, Copy)]
enum Dim {
    /// `w-*`
    Width,
    /// `h-*`
    Height,
    /// `size-*` — sets both width and height.
    Both,
    /// `min-w-*`
    MinWidth,
    /// `min-h-*`
    MinHeight,
    /// `max-w-*`
    MaxWidth,
    /// `max-h-*`
    MaxHeight,
}

/// Which font stack a `font-*` class selects (`font-sans`, `font-serif`,
/// `font-mono`). Weight variants are separate `Op` variants.
#[derive(Clone, Copy)]
enum Family {
    /// `font-sans`
    Sans,
    /// `font-serif`
    Serif,
    /// `font-mono`
    Mono,
}

/// Size keyword for `text-shadow-*` utilities. `None` removes all shadows;
/// the others pick one of `StyledExt::text_shadow_*`'s presets.
#[derive(Clone, Copy)]
enum ShadowSize {
    /// `text-shadow-none` — clears all text shadows.
    None,
    /// `text-shadow-2xs`
    S2xs,
    /// `text-shadow-xs`
    Xs,
    /// `text-shadow-sm`
    Sm,
    /// `text-shadow-md`
    Md,
    /// `text-shadow-lg`
    Lg,
}

/// A single resolved style mutation.
///
/// Every parsed class turns into one `Op`. Applying a `StyledString`
/// reduces to folding these over the target style:
///
/// ```text
/// [Op::Padding(All, 16.0), Op::FontSize(18.0), Op::Bg(red_500)]
///     .fold(style, |s, op| op.apply(s))
/// ```
///
/// The enum exists to decouple parsing from mutation, so the parser can be
/// tested against the ops it produces rather than against fully-built
/// `Style` values.
///
/// # `Send + Sync`
///
/// `Op` holds `Rc`-based Freya types (`StyledColor` wraps `Fill`, which
/// wraps shaders). This means **`Op` is not `Send + Sync`**, which is why
/// variant classes are stored as strings and re-parsed inside the
/// `Send + Sync` closure that Freya requires.
#[derive(Clone)]
enum Op {
    // ---- paint ----
    /// `bg-*` — sets `Style::background`.
    Bg(StyledColor),
    /// `text-<color>` — sets `Style::text_color`.
    TextColor(StyledColor),
    /// `text-left` / `text-center` / … — sets `Style::text_align`.
    TextAlign(TextAlign),
    /// `text-clip` / `text-ellipsis` — sets `Style::text_overflow`.
    TextOverflow(TextOverflow),
    /// `text-xs` … `text-9xl` or `text-[Npx]` — sets `Style::font_size`.
    FontSize(f32),
    /// `font-<weight>` — sets `Style::font_weight`.
    FontWeight(FontWeight),
    /// `italic` / `not-italic` — sets `Style::font_style`.
    FontStyle(FontSlant),
    /// `font-stretch-*` — sets `Style::font_stretch`.
    FontStretch(FontWidth),
    /// `tabular-nums` / `small-caps` / … — appends to `Style::font_features`.
    FontFeature(FontFeature),
    /// `font-sans` / `font-serif` / `font-mono` — replaces `Style::font_families`.
    FontFamily(Family),
    /// `tracking-*` — sets `Style::letter_spacing` (multiplied by font size).
    Tracking(f32),
    /// `leading-*` — sets `Style::line_height`.
    LineHeight(f32),
    /// `text-shadow-*` — append or clear `Style::text_shadows`.
    TextShadow(ShadowSize),
    /// `underline` / `no-underline` / … — sets `Style::text_decoration`.
    TextDecoration(TextDecoration),
    /// `cursor-<name>` — sets `Style::cursor`.
    Cursor(CursorIcon),
    /// `cursor-none` — sets `Style::cursor_visible = false`.
    CursorHidden,
    /// `overflow-*` — sets `Style::overflow`.
    Overflow(Overflow),
    /// `backdrop-blur-*` — sets `Style::blur`.
    BackdropBlur(f32),
    /// `rotate-*` (and negated) — sets `Style::rotation`.
    Rotate(f32),
    /// `opacity-*` — sets `Style::opacity`.
    Opacity(f32),
    /// `scale-*` (and negated) — sets `Style::scale` uniformly.
    Scale(f32),
    /// `scale-x-*` (and negated) — sets `Style::scale.x` only.
    ScaleX(f32),
    /// `scale-y-*` (and negated) — sets `Style::scale.y` only.
    ScaleY(f32),
    /// `origin-*` — sets `Style::transform_origin`.
    Origin(TransformOrigin),

    // ---- radius ----
    /// `rounded-*` — sets `Style::radius` on the given edge(s).
    Radius(Edge, f32),

    // ---- border ----
    /// `border-<n>` — sets only the **width** on the given edge(s). Never
    /// touches the colour.
    BorderWidth(BorderEdge, f32),
    /// `border-<color>` — sets the border's **fill**. Creates a 1px border
    /// if none exists yet; preserves an existing width. Never touches the
    /// width of an already-created border.
    ///
    /// Not edge-scoped: Freya's `Border` has a single `Fill`, so a
    /// per-side colour isn't representable. `border-t-red-500` therefore
    /// colours the whole border.
    BorderColor(StyledColor),
    /// `border-none` — sets `Style::border = None`.
    BorderNone,
    /// `border-inner` — `Style::border` alignment = inner.
    BorderInner,
    /// `border-outer` — `Style::border` alignment = outer.
    BorderOuter,
    /// `border-center` — `Style::border` alignment = center.
    BorderCenter,

    // ---- spacing ----
    /// `p-*`, `px-*`, … — sets `Style::padding` on the given side(s).
    Padding(Side, f32),
    /// `m-*`, `mx-*`, … — sets `Style::margin` on the given side(s).
    Margin(Side, f32),

    // ---- layout ----
    /// `w-*` / `h-*` / `min-*` / `max-*` / `size-*` — sets the given
    /// dimension field.
    Dim(Dim, Size),
    /// `gap-*` — sets `Style::spacing`.
    Gap(f32),
    /// `flex-row` / `flex-col` — sets `Style::direction`.
    Direction(Direction),
    /// `justify-*` — sets `Style::main_align`.
    MainAlign(Alignment),
    /// `items-*` — sets `Style::cross_align`.
    CrossAlign(Alignment),
    /// `absolute` — sets `Style::position` to absolute.
    Absolute,
    /// `relative` / `static` — sets `Style::position` to stacked.
    Relative,
    /// `truncate` — combines `text-ellipsis` and `overflow-clip`.
    Truncate,

    // ---- layer (z-index) ----
    /// `z-*` — sets `Style::layer`. See [`z_index`] for the mapping.
    Layer(Layer),

    // ---- image ----
    /// `object-contain` / `object-cover` / … — sets `Style::object_fit`.
    ObjectFit(AspectRatio),
    /// `object-scale-down` — approximates CSS `scale-down` with contain.
    ObjectScaleDown,
    /// `object-center` / `object-top-left` — sets `Style::object_position`.
    ObjectPosition(ImageCover),
    /// `image-*` / `sampling-*` — sets `Style::sampling_mode`.
    Sampling(SamplingMode),
    /// `blur-*` — sets `Style::image_blur`.
    ImageBlur(f32),
    /// `snap-to-grid` / `no-snap-to-grid` — sets `Style::snap_to_grid`.
    SnapToGrid(bool),
}

impl Op {
    /// `true` for `Op::Tracking`, used to sort it last before applying.
    ///
    /// `tracking-*` multiplies the **current** font size, so it must run
    /// after any `text-*` (font size) utility — otherwise `"tracking-wide
    /// text-lg"` would compute tracking against the wrong font size. The
    /// sort in [`StyledString::parse`] guarantees this regardless of the
    /// order the classes were written in.
    fn is_tracking(&self) -> bool {
        matches!(self, Op::Tracking(_))
    }

    /// Negate this op (`-mt-4`, `-rotate-45`, `-scale-50`, `-z-10`).
    ///
    /// Returns `None` for ops that can't be negated. For `Layer::Relative`
    /// the negation uses `checked_neg()`, which rejects `i16::MIN` — the
    /// only value whose negation would overflow — so `-z-32768` is dropped
    /// rather than panicking.
    fn negated(self) -> Option<Op> {
        Some(match self {
            Op::Margin(s, v) => Op::Margin(s, -v),
            Op::Rotate(v) => Op::Rotate(-v),
            Op::Scale(v) => Op::Scale(-v),
            Op::ScaleX(v) => Op::ScaleX(-v),
            Op::ScaleY(v) => Op::ScaleY(-v),
            Op::Layer(Layer::Relative(n)) => Op::Layer(Layer::Relative(n.checked_neg()?)),
            _ => return None,
        })
    }

    /// Apply this op to a [`StyledExt`] target and return it.
    ///
    /// Each variant delegates to the corresponding `StyledExt` method —
    /// `Op::Padding(All, 16.0)` → `t.p(16.0)`, etc. Nothing bypasses the
    /// fluent API, so behaviour matches hand-written calls exactly.
    ///
    /// # Border width / colour independence
    ///
    /// `BorderWidth` and `BorderColor` are separate ops so they can be
    /// applied in any order: `"border-2 border-red-500"` and
    /// `"border-red-500 border-2"` both end with a 2px red border. If
    /// they were one op (`Border(edge, width, Some(color))`), the second
    /// appearance would clobber the first's width.
    fn apply<T: StyledExt>(self, t: T) -> T {
        match self {
            Op::Bg(col) => t.bg(col),
            Op::TextColor(col) => t.text_color(col),
            Op::TextAlign(a) => t.text_align(a),
            Op::TextOverflow(o) => t.text_overflow(o),
            Op::FontSize(s) => t.font_size(s),
            Op::FontWeight(w) => t.font_weight(w),
            Op::FontStyle(s) => t.font_style(s),
            Op::FontStretch(s) => t.font_stretch(s),
            Op::FontFeature(f) => t.font_feature(f),
            Op::FontFamily(f) => match f {
                Family::Sans => t.font_sans(),
                Family::Serif => t.font_serif(),
                Family::Mono => t.font_mono(),
            },
            Op::Tracking(em) => t.tracking(em),
            Op::LineHeight(l) => t.line_height(l),
            Op::TextShadow(s) => match s {
                ShadowSize::None => t.text_shadow_none(),
                ShadowSize::S2xs => t.text_shadow_2xs(),
                ShadowSize::Xs => t.text_shadow_xs(),
                ShadowSize::Sm => t.text_shadow_sm(),
                ShadowSize::Md => t.text_shadow_md(),
                ShadowSize::Lg => t.text_shadow_lg(),
            },
            Op::TextDecoration(d) => t.text_decoration(d),
            Op::Cursor(cur) => t.cursor(cur),
            Op::CursorHidden => t.cursor_none(),
            Op::Overflow(o) => t.overflow(o),
            Op::BackdropBlur(b) => t.backdrop_blur(b),
            Op::Rotate(r) => t.rotate(r),
            Op::Opacity(o) => t.opacity(o),
            Op::Scale(s) => t.scale(s),
            Op::ScaleX(s) => t.scale_x(s),
            Op::ScaleY(s) => t.scale_y(s),
            Op::Origin(o) => t.transform_origin(o),
            Op::Radius(e, v) => match e {
                Edge::All => t.radius(v),
                Edge::T => t.radius_t(v),
                Edge::R => t.radius_r(v),
                Edge::B => t.radius_b(v),
                Edge::L => t.radius_l(v),
                Edge::Tl => t.radius_tl(v),
                Edge::Tr => t.radius_tr(v),
                Edge::Br => t.radius_br(v),
                Edge::Bl => t.radius_bl(v),
            },
            Op::BorderWidth(edge, width) => match edge {
                BorderEdge::All => t.border_width(width),
                BorderEdge::X => t.border_x(width),
                BorderEdge::Y => t.border_y(width),
                BorderEdge::T => t.border_t(width),
                BorderEdge::R => t.border_r(width),
                BorderEdge::B => t.border_b(width),
                BorderEdge::L => t.border_l(width),
            },
            Op::BorderColor(c) => t.border_color(c),
            Op::BorderNone => t.border_none(),
            Op::BorderInner => t.border_inner(),
            Op::BorderOuter => t.border_outer(),
            Op::BorderCenter => t.border_center(),
            Op::Padding(s, v) => match s {
                Side::All => t.p(v),
                Side::X => t.px(v),
                Side::Y => t.py(v),
                Side::T => t.pt(v),
                Side::R => t.pr(v),
                Side::B => t.pb(v),
                Side::L => t.pl(v),
            },
            Op::Margin(s, v) => match s {
                Side::All => t.m(v),
                Side::X => t.mx(v),
                Side::Y => t.my(v),
                Side::T => t.mt(v),
                Side::R => t.mr(v),
                Side::B => t.mb(v),
                Side::L => t.ml(v),
            },
            Op::Dim(dim, size) => {
                let mut t = t;
                let s = t.get_style();
                match dim {
                    Dim::Width => s.width = size,
                    Dim::Height => s.height = size,
                    Dim::Both => {
                        s.width = size.clone();
                        s.height = size;
                    }
                    Dim::MinWidth => s.min_width = size,
                    Dim::MinHeight => s.min_height = size,
                    Dim::MaxWidth => s.max_width = size,
                    Dim::MaxHeight => s.max_height = size,
                }
                t
            }
            Op::Gap(v) => t.gap(v),
            Op::Direction(d) => t.direction(d),
            Op::MainAlign(a) => t.main_align(a),
            Op::CrossAlign(a) => t.cross_align(a),
            Op::Absolute => t.absolute(),
            Op::Relative => t.relative(),
            Op::Truncate => t.truncate(),
            Op::Layer(l) => t.layer(l),
            Op::ObjectFit(a) => t.object_fit(a),
            Op::ObjectScaleDown => t.object_scale_down(),
            Op::ObjectPosition(p) => t.object_position(p),
            Op::Sampling(m) => t.image_rendering(m),
            Op::ImageBlur(b) => t.image_blur(b),
            Op::SnapToGrid(true) => t.snap_to_grid(),
            Op::SnapToGrid(false) => t.no_snap_to_grid(),
        }
    }
}

/// Apply a slice of ops to a target in order. A thin wrapper around
/// `fold` so both base and variant code paths share the same iteration
/// pattern.
fn apply_ops<T: StyledExt>(target: T, ops: &[Op]) -> T {
    ops.iter().cloned().fold(target, |t, op| op.apply(t))
}

// ============================================================================
// Image utilities
// ============================================================================

/// Parse image-related utilities (`object-*`, `image-*`, `sampling-*`,
/// `snap-to-grid`, `blur-*`).
///
/// Blur names are checked before falling back to `blur-[...]`; the
/// fallback is what makes `blur-[10px]` and `blur-[1.5rem]` work. Note the
/// fallback only accepts the `blur-` prefix — `parse_image` is called from
/// [`parse_base`] for **every** class, so a bare `blur` (no dash) is
/// handled by the preceding match arm and a `blur-`-prefixed unknown
/// (`blur-foo`) is rejected.
fn parse_image(c: &str) -> Option<Op> {
    Some(match c {
        // object-fit
        "object-contain" => Op::ObjectFit(AspectRatio::Min),
        "object-cover" => Op::ObjectFit(AspectRatio::Max),
        "object-fill" => Op::ObjectFit(AspectRatio::None),
        "object-none" => Op::ObjectFit(AspectRatio::Fit),
        "object-scale-down" => Op::ObjectScaleDown,
        // object-position
        "object-center" => Op::ObjectPosition(ImageCover::Center),
        "object-top-left" | "object-left-top" => Op::ObjectPosition(ImageCover::Fill),
        // image-rendering
        "image-auto" => Op::Sampling(SamplingMode::Trilinear),
        "image-pixelated" => Op::Sampling(SamplingMode::Nearest),
        "image-smooth" => Op::Sampling(SamplingMode::Mitchell),
        "sampling-nearest" => Op::Sampling(SamplingMode::Nearest),
        "sampling-bilinear" => Op::Sampling(SamplingMode::Bilinear),
        "sampling-trilinear" => Op::Sampling(SamplingMode::Trilinear),
        "sampling-mitchell" => Op::Sampling(SamplingMode::Mitchell),
        "sampling-catmull-rom" => Op::Sampling(SamplingMode::CatmullRom),
        // grid snapping (Freya-specific)
        "snap-to-grid" => Op::SnapToGrid(true),
        "no-snap-to-grid" => Op::SnapToGrid(false),
        // filter blur
        "blur" => Op::ImageBlur(8.0),
        "blur-none" => Op::ImageBlur(0.0),
        "blur-2xs" => Op::ImageBlur(2.0),
        "blur-xs" => Op::ImageBlur(4.0),
        "blur-sm" => Op::ImageBlur(8.0),
        "blur-md" => Op::ImageBlur(12.0),
        "blur-lg" => Op::ImageBlur(16.0),
        "blur-xl" => Op::ImageBlur(24.0),
        "blur-2xl" => Op::ImageBlur(40.0),
        "blur-3xl" => Op::ImageBlur(64.0),
        other => Op::ImageBlur(arbitrary_px(other.strip_prefix("blur-")?)?),
    })
}

// ============================================================================
// Value helpers
// ============================================================================

/// Wrap a pixel value in a [`Size`].
fn px(v: f32) -> Size {
    Size::Pixels(Length::new(v))
}

/// Wrap a percent value in a [`Size`].
fn pct(v: f32) -> Size {
    Size::Percentage(Length::new(v))
}

/// Parse a non-negative decimal: `4`, `1.5`, `.5`.
///
/// Rejects negatives (`-` prefix), exponents (`1e3`), hex (`0x10`), and
/// anything with non-digit-non-dot characters. `.5` (leading dot) is
/// accepted; `1.` (trailing dot) is also accepted via Rust's `f32::parse`
/// — this matches Tailwind's leniency.
fn plain_num(s: &str) -> Option<f32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    s.parse().ok()
}

/// Parse a possibly-negative decimal: `-0.05`.
///
/// Equivalent to `plain_num` with an optional `-` prefix.
fn signed_num(s: &str) -> Option<f32> {
    match s.strip_prefix('-') {
        Some(rest) => plain_num(rest).map(|n| -n),
        None => plain_num(s),
    }
}

/// Strip outer `[...]`, returning the inner text unchanged.
///
/// The caller is responsible for trimming if they want to ignore interior
/// whitespace. Returns `None` if the input doesn't start with `[` and end
/// with `]`.
fn arbitrary(s: &str) -> Option<&str> {
    s.strip_prefix('[')?.strip_suffix(']')
}

/// Parse a bracket pixel value: `[13px]`, `[1.5rem]`, or (leniently) `[10]`.
///
/// # Accepted forms
///
/// | Input        | Output  | Note                                     |
/// |--------------|---------|------------------------------------------|
/// | `[13px]`     | 13.0    | Explicit pixels                          |
/// | `[1.5rem]`   | 24.0    | `1.5 × 16`                               |
/// | `[10]`       | 10.0    | Lenient — treated as `10px`              |
/// | `[ 10px ]`   | 10.0    | Interior whitespace trimmed              |
/// | `[10em]`     | None    | `em` / `%` / `vh` / … not supported      |
///
/// # Unitless leniency
///
/// Tailwind requires a unit inside brackets (`w-[10px]`, not `w-[10]`).
/// FreyaCN accepts both for backwards compatibility. To reject unitless
/// brackets, replace the final `plain_num(inner)` in this function with
/// `None`.
///
/// # `em` / `%`
///
/// These would require font-size or container context to resolve, which
/// the parser doesn't have. Use `w-[50%]` for percentages (handled by
/// [`size_value`]) or fall back to the fluent API for `em`-based sizing.
fn arbitrary_px(s: &str) -> Option<f32> {
    let inner = arbitrary(s)?.trim();
    if let Some(n) = inner.strip_suffix("px") {
        plain_num(n)
    } else if let Some(n) = inner.strip_suffix("rem") {
        plain_num(n).map(|n| n * 16.0)
    } else {
        plain_num(inner)
    }
}

/// Parse a bracket number: `[0.3]` or `[ -0.3 ]`.
fn arbitrary_num(s: &str) -> Option<f32> {
    signed_num(arbitrary(s)?.trim())
}

/// Parse a Tailwind spacing-scale value: `px` = 1px, `N` = N×4px, or
/// `[Npx]` for arbitrary pixels.
///
/// # Examples
///
/// ```text
/// spacing("px")    → Some(1.0)
/// spacing("1")     → Some(4.0)
/// spacing("1.5")   → Some(6.0)
/// spacing("4")     → Some(16.0)
/// spacing("[13px]")→ Some(13.0)
/// spacing("foo")   → None
/// ```
fn spacing(s: &str) -> Option<f32> {
    if s == "px" {
        Some(1.0)
    } else if let Some(n) = plain_num(s) {
        Some(n * 4.0)
    } else {
        arbitrary_px(s)
    }
}

/// Parse a percentage-ish value: `50` = 0.5, `[0.3]` = 0.3.
///
/// Used for opacity, scale, and similar 0..=1 values that are written as
/// percentages in the source class.
fn percent(s: &str) -> Option<f32> {
    plain_num(s).map(|n| n / 100.0).or_else(|| arbitrary_num(s))
}

/// Resolve a width / height / min / max value to a [`Size`].
///
/// # Accepted forms
///
/// | Input          | Output                                |
/// |----------------|---------------------------------------|
/// | `auto` / `fit` | `Size::Inner` (fit to content)         |
/// | `full`/`screen`| `Size::Percentage(100)`                |
/// | `1/2`          | `Size::Percentage(50)`                 |
/// | `[50%]`        | `Size::Percentage(50)`                 |
/// | `4`            | `Size::Pixels(16)`                     |
/// | `[13px]`       | `Size::Pixels(13)`                     |
/// | `[1.5rem]`     | `Size::Pixels(24)`                     |
///
/// Note: `screen` is treated identically to `full` because Freya's sizing
/// system has no viewport-relative `Size` variant — this matches the
/// `StyledExt::w_screen` / `h_screen` shortcuts, which also use
/// `Percentage(100)` as an approximation.
fn size_value(v: &str) -> Option<Size> {
    match v {
        "auto" | "fit" => return Some(Size::Inner),
        "full" | "screen" => return Some(pct(100.0)),
        _ => {}
    }
    if let Some((a, b)) = v.split_once('/') {
        let (a, b) = (plain_num(a)?, plain_num(b)?);
        if b == 0.0 {
            return None;
        }
        return Some(pct(a * 100.0 / b));
    }
    if let Some(n) = arbitrary(v).and_then(|i| i.strip_suffix('%')) {
        return plain_num(n).map(pct);
    }
    spacing(v).map(px)
}

/// Look up the Tailwind container-size scale (`max-w-xs` = 320px, …).
///
/// Only used by `max-w-*` — other utilities use [`size_value`] which
/// doesn't include these sizes. This matches Tailwind's behaviour, where
/// `w-xl` doesn't exist but `max-w-xl` does.
fn container_size(v: &str) -> Option<f32> {
    Some(match v {
        "xs" => 320.0,
        "sm" => 384.0,
        "md" => 448.0,
        "lg" => 512.0,
        "xl" => 576.0,
        "2xl" => 672.0,
        "3xl" => 768.0,
        "4xl" => 896.0,
        "5xl" => 1024.0,
        "6xl" => 1152.0,
        "7xl" => 1280.0,
        _ => return None,
    })
}

/// Resolve a colour name or arbitrary hex to a [`StyledColor`], applying an
/// optional `/NN` alpha suffix.
///
/// # Syntax
///
/// ```text
/// blue-500           → blue_500()
/// blue-500/50        → blue_500().opacity(0.5)
/// [#ff0000]          → StyledColor::rgb(255, 0, 0)
/// [#f00]             → StyledColor::rgb(255, 0, 0)
/// [#ff000080]        → StyledColor::argb(0x80, 255, 0, 0)
/// [#ff000080]/50     → StyledColor::argb(0x80, 255, 0, 0).opacity(0.5)
/// transparent        → StyledColor::rgb(0, 0, 0).opacity(0.0)
/// ```
///
/// # Alpha semantics
///
/// `/NN` **multiplies** the source alpha (matching Tailwind). This is the
/// same convention as `theme.rs`'s `with_alpha_pct` for palette functions
/// — a `/50` suffix halves whatever the underlying colour's alpha is.
fn color_value(v: &str) -> Option<StyledColor> {
    let (name, alpha) = match v.split_once('/') {
        Some((n, a)) => (n, Some(plain_num(a)? / 100.0)),
        None => (v, None),
    };
    let col = color_name_or_hex(name)?;
    Some(match alpha {
        Some(a) => col.opacity(a),
        None => col,
    })
}

/// Resolve a colour token — either a named palette colour or an arbitrary
/// bracketed hex.
///
/// Named colours are looked up in the big `color` match (see [`color`]).
/// Bracket colours are parsed via [`parse_hex_rgba`]. Anything else
/// returns `None`.
fn color_name_or_hex(name: &str) -> Option<StyledColor> {
    // Arbitrary hex: `bg-[#ff0000]`, `text-[#abc]`, `border-[#ff000080]`
    if let Some(inner) = arbitrary(name) {
        let (r, g, b, a) = parse_hex_rgba(inner.trim())?;
        return Some(StyledColor::argb(a, r, g, b));
    }
    color(name)
}

/// Parse a hex colour string into separate `(r, g, b, a)` bytes.
///
/// # Accepted forms
///
/// | Input        | Output                              |
/// |--------------|-------------------------------------|
/// | `#abc`       | (170, 187, 204, 255)                |
/// | `#abcd`      | (170, 187, 204, 221)                |
/// | `#aabbcc`    | (170, 187, 204, 255)                |
/// | `#aabbccdd`  | (170, 187, 204, 221)                |
/// | `abc` (no `#`) | same as `#abc`                    |
///
/// Returns `None` for wrong lengths, non-hex characters, or empty input.
fn parse_hex_rgba(hex: &str) -> Option<(u8, u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    let expand = |h: &str| -> String { h.chars().flat_map(|c| [c, c]).collect() };
    let (rgb, alpha) = match s.len() {
        3 => (expand(s), 255u8),
        4 => {
            let a = u8::from_str_radix(&expand(&s[3..]), 16).ok()?;
            (expand(&s[..3]), a)
        }
        6 => (s.to_string(), 255u8),
        8 => {
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            (s[..6].to_string(), a)
        }
        _ => return None,
    };
    if rgb.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    Some((r, g, b, alpha))
}

/// Look up a named palette colour (Tailwind scale + `white` / `black` /
/// `transparent`).
///
/// This is a long, flat `match` — every family/step is listed explicitly
/// so the compiler can turn it into a jump table and so adding a new
/// family is a mechanical change to one place. The parser
/// ([`lookup_palette_styled`] in `theme.rs`) uses a macro to generate the
/// same list; they must be kept in sync.
///
/// `transparent` is `StyledColor::rgb(0, 0, 0).opacity(0.0)` — a solid
/// black with zero alpha, **not** a `Fill::None`. See
/// `crate::styling::colors::transparent` for the rationale.
fn color(name: &str) -> Option<StyledColor> {
    Some(match name {
        "black" => c::black(),
        "mauve-50" => c::mauve_50(),
        "mauve-100" => c::mauve_100(),
        "mauve-200" => c::mauve_200(),
        "mauve-300" => c::mauve_300(),
        "mauve-400" => c::mauve_400(),
        "mauve-500" => c::mauve_500(),
        "mauve-600" => c::mauve_600(),
        "mauve-700" => c::mauve_700(),
        "mauve-800" => c::mauve_800(),
        "mauve-900" => c::mauve_900(),
        "mauve-950" => c::mauve_950(),
        "olive-50" => c::olive_50(),
        "olive-100" => c::olive_100(),
        "olive-200" => c::olive_200(),
        "olive-300" => c::olive_300(),
        "olive-400" => c::olive_400(),
        "olive-500" => c::olive_500(),
        "olive-600" => c::olive_600(),
        "olive-700" => c::olive_700(),
        "olive-800" => c::olive_800(),
        "olive-900" => c::olive_900(),
        "olive-950" => c::olive_950(),
        "mist-50" => c::mist_50(),
        "mist-100" => c::mist_100(),
        "mist-200" => c::mist_200(),
        "mist-300" => c::mist_300(),
        "mist-400" => c::mist_400(),
        "mist-500" => c::mist_500(),
        "mist-600" => c::mist_600(),
        "mist-700" => c::mist_700(),
        "mist-800" => c::mist_800(),
        "mist-900" => c::mist_900(),
        "mist-950" => c::mist_950(),
        "taupe-50" => c::taupe_50(),
        "taupe-100" => c::taupe_100(),
        "taupe-200" => c::taupe_200(),
        "taupe-300" => c::taupe_300(),
        "taupe-400" => c::taupe_400(),
        "taupe-500" => c::taupe_500(),
        "taupe-600" => c::taupe_600(),
        "taupe-700" => c::taupe_700(),
        "taupe-800" => c::taupe_800(),
        "taupe-900" => c::taupe_900(),
        "taupe-950" => c::taupe_950(),
        "white" => c::white(),
        "transparent" => c::transparent(),
        "slate-50" => c::slate_50(),
        "slate-100" => c::slate_100(),
        "slate-200" => c::slate_200(),
        "slate-300" => c::slate_300(),
        "slate-400" => c::slate_400(),
        "slate-500" => c::slate_500(),
        "slate-600" => c::slate_600(),
        "slate-700" => c::slate_700(),
        "slate-800" => c::slate_800(),
        "slate-900" => c::slate_900(),
        "slate-950" => c::slate_950(),
        "gray-50" => c::gray_50(),
        "gray-100" => c::gray_100(),
        "gray-200" => c::gray_200(),
        "gray-300" => c::gray_300(),
        "gray-400" => c::gray_400(),
        "gray-500" => c::gray_500(),
        "gray-600" => c::gray_600(),
        "gray-700" => c::gray_700(),
        "gray-800" => c::gray_800(),
        "gray-900" => c::gray_900(),
        "gray-950" => c::gray_950(),
        "zinc-50" => c::zinc_50(),
        "zinc-100" => c::zinc_100(),
        "zinc-200" => c::zinc_200(),
        "zinc-300" => c::zinc_300(),
        "zinc-400" => c::zinc_400(),
        "zinc-500" => c::zinc_500(),
        "zinc-600" => c::zinc_600(),
        "zinc-700" => c::zinc_700(),
        "zinc-800" => c::zinc_800(),
        "zinc-900" => c::zinc_900(),
        "zinc-950" => c::zinc_950(),
        "neutral-50" => c::neutral_50(),
        "neutral-100" => c::neutral_100(),
        "neutral-200" => c::neutral_200(),
        "neutral-300" => c::neutral_300(),
        "neutral-400" => c::neutral_400(),
        "neutral-500" => c::neutral_500(),
        "neutral-600" => c::neutral_600(),
        "neutral-700" => c::neutral_700(),
        "neutral-800" => c::neutral_800(),
        "neutral-900" => c::neutral_900(),
        "neutral-950" => c::neutral_950(),
        "stone-50" => c::stone_50(),
        "stone-100" => c::stone_100(),
        "stone-200" => c::stone_200(),
        "stone-300" => c::stone_300(),
        "stone-400" => c::stone_400(),
        "stone-500" => c::stone_500(),
        "stone-600" => c::stone_600(),
        "stone-700" => c::stone_700(),
        "stone-800" => c::stone_800(),
        "stone-900" => c::stone_900(),
        "stone-950" => c::stone_950(),
        "red-50" => c::red_50(),
        "red-100" => c::red_100(),
        "red-200" => c::red_200(),
        "red-300" => c::red_300(),
        "red-400" => c::red_400(),
        "red-500" => c::red_500(),
        "red-600" => c::red_600(),
        "red-700" => c::red_700(),
        "red-800" => c::red_800(),
        "red-900" => c::red_900(),
        "red-950" => c::red_950(),
        "orange-50" => c::orange_50(),
        "orange-100" => c::orange_100(),
        "orange-200" => c::orange_200(),
        "orange-300" => c::orange_300(),
        "orange-400" => c::orange_400(),
        "orange-500" => c::orange_500(),
        "orange-600" => c::orange_600(),
        "orange-700" => c::orange_700(),
        "orange-800" => c::orange_800(),
        "orange-900" => c::orange_900(),
        "orange-950" => c::orange_950(),
        "amber-50" => c::amber_50(),
        "amber-100" => c::amber_100(),
        "amber-200" => c::amber_200(),
        "amber-300" => c::amber_300(),
        "amber-400" => c::amber_400(),
        "amber-500" => c::amber_500(),
        "amber-600" => c::amber_600(),
        "amber-700" => c::amber_700(),
        "amber-800" => c::amber_800(),
        "amber-900" => c::amber_900(),
        "amber-950" => c::amber_950(),
        "yellow-50" => c::yellow_50(),
        "yellow-100" => c::yellow_100(),
        "yellow-200" => c::yellow_200(),
        "yellow-300" => c::yellow_300(),
        "yellow-400" => c::yellow_400(),
        "yellow-500" => c::yellow_500(),
        "yellow-600" => c::yellow_600(),
        "yellow-700" => c::yellow_700(),
        "yellow-800" => c::yellow_800(),
        "yellow-900" => c::yellow_900(),
        "yellow-950" => c::yellow_950(),
        "lime-50" => c::lime_50(),
        "lime-100" => c::lime_100(),
        "lime-200" => c::lime_200(),
        "lime-300" => c::lime_300(),
        "lime-400" => c::lime_400(),
        "lime-500" => c::lime_500(),
        "lime-600" => c::lime_600(),
        "lime-700" => c::lime_700(),
        "lime-800" => c::lime_800(),
        "lime-900" => c::lime_900(),
        "lime-950" => c::lime_950(),
        "green-50" => c::green_50(),
        "green-100" => c::green_100(),
        "green-200" => c::green_200(),
        "green-300" => c::green_300(),
        "green-400" => c::green_400(),
        "green-500" => c::green_500(),
        "green-600" => c::green_600(),
        "green-700" => c::green_700(),
        "green-800" => c::green_800(),
        "green-900" => c::green_900(),
        "green-950" => c::green_950(),
        "emerald-50" => c::emerald_50(),
        "emerald-100" => c::emerald_100(),
        "emerald-200" => c::emerald_200(),
        "emerald-300" => c::emerald_300(),
        "emerald-400" => c::emerald_400(),
        "emerald-500" => c::emerald_500(),
        "emerald-600" => c::emerald_600(),
        "emerald-700" => c::emerald_700(),
        "emerald-800" => c::emerald_800(),
        "emerald-900" => c::emerald_900(),
        "emerald-950" => c::emerald_950(),
        "teal-50" => c::teal_50(),
        "teal-100" => c::teal_100(),
        "teal-200" => c::teal_200(),
        "teal-300" => c::teal_300(),
        "teal-400" => c::teal_400(),
        "teal-500" => c::teal_500(),
        "teal-600" => c::teal_600(),
        "teal-700" => c::teal_700(),
        "teal-800" => c::teal_800(),
        "teal-900" => c::teal_900(),
        "teal-950" => c::teal_950(),
        "cyan-50" => c::cyan_50(),
        "cyan-100" => c::cyan_100(),
        "cyan-200" => c::cyan_200(),
        "cyan-300" => c::cyan_300(),
        "cyan-400" => c::cyan_400(),
        "cyan-500" => c::cyan_500(),
        "cyan-600" => c::cyan_600(),
        "cyan-700" => c::cyan_700(),
        "cyan-800" => c::cyan_800(),
        "cyan-900" => c::cyan_900(),
        "cyan-950" => c::cyan_950(),
        "sky-50" => c::sky_50(),
        "sky-100" => c::sky_100(),
        "sky-200" => c::sky_200(),
        "sky-300" => c::sky_300(),
        "sky-400" => c::sky_400(),
        "sky-500" => c::sky_500(),
        "sky-600" => c::sky_600(),
        "sky-700" => c::sky_700(),
        "sky-800" => c::sky_800(),
        "sky-900" => c::sky_900(),
        "sky-950" => c::sky_950(),
        "blue-50" => c::blue_50(),
        "blue-100" => c::blue_100(),
        "blue-200" => c::blue_200(),
        "blue-300" => c::blue_300(),
        "blue-400" => c::blue_400(),
        "blue-500" => c::blue_500(),
        "blue-600" => c::blue_600(),
        "blue-700" => c::blue_700(),
        "blue-800" => c::blue_800(),
        "blue-900" => c::blue_900(),
        "blue-950" => c::blue_950(),
        "indigo-50" => c::indigo_50(),
        "indigo-100" => c::indigo_100(),
        "indigo-200" => c::indigo_200(),
        "indigo-300" => c::indigo_300(),
        "indigo-400" => c::indigo_400(),
        "indigo-500" => c::indigo_500(),
        "indigo-600" => c::indigo_600(),
        "indigo-700" => c::indigo_700(),
        "indigo-800" => c::indigo_800(),
        "indigo-900" => c::indigo_900(),
        "indigo-950" => c::indigo_950(),
        "violet-50" => c::violet_50(),
        "violet-100" => c::violet_100(),
        "violet-200" => c::violet_200(),
        "violet-300" => c::violet_300(),
        "violet-400" => c::violet_400(),
        "violet-500" => c::violet_500(),
        "violet-600" => c::violet_600(),
        "violet-700" => c::violet_700(),
        "violet-800" => c::violet_800(),
        "violet-900" => c::violet_900(),
        "violet-950" => c::violet_950(),
        "purple-50" => c::purple_50(),
        "purple-100" => c::purple_100(),
        "purple-200" => c::purple_200(),
        "purple-300" => c::purple_300(),
        "purple-400" => c::purple_400(),
        "purple-500" => c::purple_500(),
        "purple-600" => c::purple_600(),
        "purple-700" => c::purple_700(),
        "purple-800" => c::purple_800(),
        "purple-900" => c::purple_900(),
        "purple-950" => c::purple_950(),
        "fuchsia-50" => c::fuchsia_50(),
        "fuchsia-100" => c::fuchsia_100(),
        "fuchsia-200" => c::fuchsia_200(),
        "fuchsia-300" => c::fuchsia_300(),
        "fuchsia-400" => c::fuchsia_400(),
        "fuchsia-500" => c::fuchsia_500(),
        "fuchsia-600" => c::fuchsia_600(),
        "fuchsia-700" => c::fuchsia_700(),
        "fuchsia-800" => c::fuchsia_800(),
        "fuchsia-900" => c::fuchsia_900(),
        "fuchsia-950" => c::fuchsia_950(),
        "pink-50" => c::pink_50(),
        "pink-100" => c::pink_100(),
        "pink-200" => c::pink_200(),
        "pink-300" => c::pink_300(),
        "pink-400" => c::pink_400(),
        "pink-500" => c::pink_500(),
        "pink-600" => c::pink_600(),
        "pink-700" => c::pink_700(),
        "pink-800" => c::pink_800(),
        "pink-900" => c::pink_900(),
        "pink-950" => c::pink_950(),
        "rose-50" => c::rose_50(),
        "rose-100" => c::rose_100(),
        "rose-200" => c::rose_200(),
        "rose-300" => c::rose_300(),
        "rose-400" => c::rose_400(),
        "rose-500" => c::rose_500(),
        "rose-600" => c::rose_600(),
        "rose-700" => c::rose_700(),
        "rose-800" => c::rose_800(),
        "rose-900" => c::rose_900(),
        "rose-950" => c::rose_950(),
        _ => return None,
    })
}

// ============================================================================
// Parser
// ============================================================================

/// Parse a single (variant-stripped) class into an [`Op`].
///
/// Handles the leading `-` negation prefix, then delegates to
/// [`parse_base`]. Negation is rejected for classes that don't support it
/// (only margins, rotation, scale, and relative z-index can be negated).
fn parse_utility(class: &str) -> Option<Op> {
    let (neg, base) = match class.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, class),
    };
    let op = parse_base(base)?;
    if neg { op.negated() } else { Some(op) }
}

/// Parse keyword-only utilities that have no `-<value>` suffix.
///
/// These are matched first in [`parse_base`] because none of them can be
/// confused with a `head-value` split. `truncate` is here rather than in
/// `parse_base`'s main match because it's a compound convenience op.
fn parse_static(c: &str) -> Option<Op> {
    Some(match c {
        "italic" => Op::FontStyle(FontSlant::Italic),
        "not-italic" => Op::FontStyle(FontSlant::Upright),
        "underline" => Op::TextDecoration(TextDecoration::Underline),
        "overline" => Op::TextDecoration(TextDecoration::Overline),
        "line-through" => Op::TextDecoration(TextDecoration::LineThrough),
        "no-underline" => Op::TextDecoration(TextDecoration::None),
        "truncate" => Op::Truncate,
        "tabular-nums" => Op::FontFeature(FontFeature::TABULAR_NUMBERS),
        "slashed-zero" => Op::FontFeature(FontFeature::SLASHED_ZERO),
        "diagonal-fractions" => Op::FontFeature(FontFeature::FRACTIONS),
        "small-caps" => Op::FontFeature(FontFeature::SMALL_CAPS),
        "flex-row" => Op::Direction(Direction::Horizontal),
        "flex-col" => Op::Direction(Direction::Vertical),
        "absolute" => Op::Absolute,
        "relative" | "static" => Op::Relative,
        "backdrop-blur" => Op::BackdropBlur(8.0),
        _ => return None,
    })
}

/// Dispatch a single class (already stripped of variant and negation) to
/// the appropriate parser.
///
/// # Ordering
///
/// 1. [`parse_static`] — keyword-only classes.
/// 2. [`parse_image`] — image-related classes.
/// 3. [`border_class`] — borders (needs its own dispatch because it has
///    edge + width + colour combinations).
/// 4. Multi-dash prefixes (`rounded-*`, `text-shadow-*`,
///    `backdrop-blur-*`, `font-stretch-*`, `min-w-*`, `min-h-*`,
///    `max-w-*`, `max-h-*`, `scale-x-*`, `scale-y-*`). These **must** come
///    before the generic `head-value` split, otherwise `rounded-tl-lg`
///    would be split as `("rounded", "tl-lg")` and fail.
/// 5. Generic `head-value` split (`p-4` → `("p", "4")`, etc.).
fn parse_base(c: &str) -> Option<Op> {
    if let Some(op) = parse_static(c) {
        return Some(op);
    }
    if let Some(op) = parse_image(c) {
        return Some(op);
    }
    if let Some(op) = border_class(c) {
        return Some(op);
    }

    // ---- multi-dash prefixes (must be checked before the generic split) ----
    if c == "rounded" {
        return rounded("");
    }
    if let Some(r) = c.strip_prefix("rounded-") {
        return rounded(r);
    }
    if let Some(v) = c.strip_prefix("text-shadow-") {
        return Some(Op::TextShadow(match v {
            "none" => ShadowSize::None,
            "2xs" => ShadowSize::S2xs,
            "xs" => ShadowSize::Xs,
            "sm" => ShadowSize::Sm,
            "md" => ShadowSize::Md,
            "lg" => ShadowSize::Lg,
            _ => return None,
        }));
    }
    if let Some(v) = c.strip_prefix("backdrop-blur-") {
        return backdrop_blur(v);
    }
    if let Some(v) = c.strip_prefix("font-stretch-") {
        return font_stretch(v);
    }
    if let Some(v) = c.strip_prefix("min-w-") {
        return Some(Op::Dim(Dim::MinWidth, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("min-h-") {
        return Some(Op::Dim(Dim::MinHeight, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("max-w-") {
        if let Some(px_v) = container_size(v) {
            return Some(Op::Dim(Dim::MaxWidth, px(px_v)));
        }
        return Some(Op::Dim(Dim::MaxWidth, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("max-h-") {
        return Some(Op::Dim(Dim::MaxHeight, size_value(v)?));
    }
    if let Some(v) = c.strip_prefix("scale-x-") {
        return percent(v).map(Op::ScaleX);
    }
    if let Some(v) = c.strip_prefix("scale-y-") {
        return percent(v).map(Op::ScaleY);
    }

    // ---- `head-value` ----
    let (head, v) = c.split_once('-')?;
    match head {
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" => {
            Some(Op::Padding(side(&head[1..])?, spacing(v)?))
        }
        "m" | "mx" | "my" | "mt" | "mr" | "mb" | "ml" => {
            Some(Op::Margin(side(&head[1..])?, spacing(v)?))
        }
        "w" => Some(Op::Dim(Dim::Width, size_value(v)?)),
        "h" => Some(Op::Dim(Dim::Height, size_value(v)?)),
        "size" => Some(Op::Dim(Dim::Both, size_value(v)?)),
        "gap" => spacing(v).map(Op::Gap),
        "bg" => color_value(v).map(Op::Bg),
        "text" => text(v),
        "font" => font(v),
        "tracking" => tracking(v),
        "leading" => leading(v),
        "cursor" => cursor(v),
        "z" => z_index(v),
        "overflow" => match v {
            "hidden" | "clip" => Some(Op::Overflow(Overflow::Clip)),
            "visible" => Some(Op::Overflow(Overflow::None)),
            _ => None,
        },
        "opacity" => percent(v).map(|o| Op::Opacity(o.clamp(0.0, 1.0))),
        "rotate" => {
            if v == "none" {
                Some(Op::Rotate(0.0))
            } else if let Some(n) = plain_num(v) {
                Some(Op::Rotate(n))
            } else {
                let inner = arbitrary(v)?;
                plain_num(inner.strip_suffix("deg").unwrap_or(inner)).map(Op::Rotate)
            }
        }
        "scale" => percent(v).map(Op::Scale),
        "origin" => origin(v),
        "justify" => Some(Op::MainAlign(match v {
            "start" => Alignment::Start,
            "center" => Alignment::Center,
            "end" => Alignment::End,
            "between" => Alignment::SpaceBetween,
            "around" => Alignment::SpaceAround,
            "evenly" => Alignment::SpaceEvenly,
            _ => return None,
        })),
        "items" => Some(Op::CrossAlign(match v {
            "start" => Alignment::Start,
            "center" => Alignment::Center,
            "end" => Alignment::End,
            _ => return None,
        })),
        _ => None,
    }
}

/// Map a padding/margin suffix (`""`, `"x"`, `"y"`, `"t"`, …) to a [`Side`].
///
/// The suffix is `head[1..]` for a `p`/`m` head — i.e. `""` for `p`,
/// `"x"` for `px`, `"t"` for `pt`, etc.
fn side(s: &str) -> Option<Side> {
    Some(match s {
        "" => Side::All,
        "x" => Side::X,
        "y" => Side::Y,
        "t" => Side::T,
        "r" => Side::R,
        "b" => Side::B,
        "l" => Side::L,
        _ => return None,
    })
}

/// Map a `rounded-*` edge suffix to an [`Edge`].
///
/// Only corner suffixes (`tl`, `tr`, `br`, `bl`) and side suffixes
/// (`t`, `r`, `b`, `l`) are recognized. `rounded-x` / `rounded-y` don't
/// exist in `StyledExt` and are rejected.
fn edge_of(s: &str) -> Option<Edge> {
    Some(match s {
        "t" => Edge::T,
        "r" => Edge::R,
        "b" => Edge::B,
        "l" => Edge::L,
        "tl" => Edge::Tl,
        "tr" => Edge::Tr,
        "br" => Edge::Br,
        "bl" => Edge::Bl,
        _ => return None,
    })
}

/// Map a `border-*` edge suffix to a [`BorderEdge`].
///
/// Unlike [`edge_of`], borders support `x` / `y` (axis) variants but no
/// corners — matching `StyledExt`'s `border_x` / `border_y` methods.
fn border_edge_of(s: &str) -> Option<BorderEdge> {
    Some(match s {
        "x" => BorderEdge::X,
        "y" => BorderEdge::Y,
        "t" => BorderEdge::T,
        "r" => BorderEdge::R,
        "b" => BorderEdge::B,
        "l" => BorderEdge::L,
        _ => return None,
    })
}

/// Parse `rounded`, `rounded-<size>`, `rounded-<edge>`, `rounded-<edge>-<size>`,
/// and arbitrary-radius forms.
///
/// # Examples
///
/// ```text
/// rounded              → 4px on all corners
/// rounded-md           → 6px on all corners
/// rounded-t            → 4px on the top edge
/// rounded-t-md         → 6px on the top edge
/// rounded-tl-lg        → 8px on the top-left corner
/// rounded-[6px]        → 6px on all corners (arbitrary)
/// rounded-tl-[6px]     → 6px on the top-left corner (arbitrary)
/// rounded-full         → 9999px on all corners (pill)
/// ```
///
/// # Ambiguity resolution
///
/// When `rest` contains a dash, the parser tries to interpret the prefix
/// as an edge first. If it isn't a recognized edge, the whole `rest` is
/// treated as a size (`rounded-[6px]` → `rest = "[6px]"` → no dash →
/// treated as a size). This lets arbitrary values (`rounded-[6px]`) work
/// without a special-case branch.
fn rounded(rest: &str) -> Option<Op> {
    if rest.is_empty() {
        return Some(Op::Radius(Edge::All, 4.0));
    }
    let (edge, size) = match rest.split_once('-') {
        Some((e, s)) => match edge_of(e) {
            Some(e) => (e, Some(s)),
            None => (Edge::All, Some(rest)),
        },
        None => match edge_of(rest) {
            Some(e) => (e, None),
            None => (Edge::All, Some(rest)),
        },
    };
    let radius = match size {
        None => 4.0,
        Some("none") => 0.0,
        Some("xs") => 2.0,
        Some("sm") => 4.0,
        Some("md") => 6.0,
        Some("lg") => 8.0,
        Some("xl") => 12.0,
        Some("2xl") => 16.0,
        Some("3xl") => 24.0,
        Some("4xl") => 32.0,
        Some("full") => 9999.0,
        Some(other) => arbitrary_px(other)?,
    };
    Some(Op::Radius(edge, radius))
}

/// Parse border utilities.
///
/// Accepts every combination of edge / width / colour that `StyledExt`
/// exposes. Width and colour produce **separate ops**
/// ([`Op::BorderWidth`] and [`Op::BorderColor`]), so they can appear in
/// either order in the class string and both will take effect on the same
/// border object. `border-2 border-red-500` and `border-red-500 border-2`
/// both end up as a 2px red border.
///
/// # Supported forms
///
/// ```text
/// border                    → 1px width, all edges
/// border-none               → remove border object
/// border-inner              → inner alignment
/// border-outer              → outer alignment
/// border-center             → center alignment
/// border-<width>            → 0, 2, 4, 8, [3px], …
/// border-<color>            → red-500, red-500/50, transparent, [#ff0000]
/// border-<edge>             → t, r, b, l, x, y  (1px width on that edge/axis)
/// border-<edge>-<width>     → border-t-2, border-x-4
/// border-<edge>-<color>     → fills the whole border (see below)
/// ```
///
/// # Per-side colours
///
/// Freya's `Border` has a **single** `Fill` shared by all four sides, so a
/// per-side colour isn't representable. `border-t-red-500` therefore sets
/// that shared fill — every visible side becomes red. The `t` prefix is
/// only meaningful for width classes.
///
/// # Ambiguity resolution
///
/// The function tries `border-<edge>-<tail>` before
/// `border-<width-or-colour>`. This matters for `border-t-2`: without the
/// edge-prefix branch, `border-t-2` would try to parse `"t-2"` as a width
/// (fails) or as a colour (fails), returning `None`. With the branch,
/// `head = "t"` matches `BorderEdge::T`, `tail = "2"` parses as width 2.
fn border_class(c: &str) -> Option<Op> {
    if c == "border" {
        return Some(Op::BorderWidth(BorderEdge::All, 1.0));
    }
    if c == "border-none" {
        return Some(Op::BorderNone);
    }
    if c == "border-inner" {
        return Some(Op::BorderInner);
    }
    if c == "border-outer" {
        return Some(Op::BorderOuter);
    }
    if c == "border-center" {
        return Some(Op::BorderCenter);
    }

    let rest = c.strip_prefix("border-")?;

    // `border-<edge>-<width>` or `border-<edge>-<color>`
    if let Some((head, tail)) = rest.split_once('-') {
        if let Some(edge) = border_edge_of(head) {
            // Width first (edge-scoped).
            if let Some(n) = plain_num(tail).or_else(|| arbitrary_px(tail)) {
                return Some(Op::BorderWidth(edge, n));
            }
            // Colour is **uniform across edges** (single `Fill` on
            // `Border`), so `border-t-red-500` colours the whole border.
            // Emitted as `BorderColor` so it doesn't touch any width
            // another class may have set.
            if let Some(col) = color_value(tail) {
                return Some(Op::BorderColor(col));
            }
            return None;
        }
    }

    // `border-<width>`
    if let Some(n) = plain_num(rest).or_else(|| arbitrary_px(rest)) {
        return Some(Op::BorderWidth(BorderEdge::All, n));
    }
    // `border-<color>`
    if let Some(col) = color_value(rest) {
        return Some(Op::BorderColor(col));
    }

    // `border-<edge>` alone (default 1px on that edge)
    if let Some(edge) = border_edge_of(rest) {
        return Some(Op::BorderWidth(edge, 1.0));
    }

    None
}

/// Parse `backdrop-blur-<size>` or `backdrop-blur-[Npx]`.
///
/// The blur radius is in pixels; named sizes match Tailwind's
/// `backdrop-blur-*` scale.
fn backdrop_blur(v: &str) -> Option<Op> {
    Some(Op::BackdropBlur(match v {
        "none" => 0.0,
        "2xs" => 2.0,
        "xs" => 4.0,
        "sm" => 8.0,
        "md" => 12.0,
        "lg" => 16.0,
        "xl" => 24.0,
        "2xl" => 40.0,
        "3xl" => 64.0,
        other => arbitrary_px(other)?,
    }))
}

/// Parse `z-<value>`.
///
/// # Supported forms
///
/// ```text
/// z-auto       → Layer::Relative(0)
/// z-<n>        → Layer::Relative(n)   (n in i16 range)
/// z-[n]        → Layer::Relative(n)   (n in i16 range)
/// z-overlay    → Layer::Overlay
/// z-overlay-<n> → Layer::OverlayLevel(n)  (n in u8 range)
/// ```
///
/// # Range handling
///
/// Values outside `i16::MIN..=i16::MAX` (relative) or `0..=255` (overlay
/// level) are **rejected**, not silently truncated. `z-40000` is dropped
/// with a debug warning rather than becoming `z--25536`; `z-overlay-300`
/// is dropped rather than becoming `z-overlay-44`. This is safer for
/// remote-input scenarios where a maliciously large value could otherwise
/// wrap to a "negative" layer.
///
/// # Why `z-auto` → `Relative(0)`
///
/// Freya's `Layer::Relative(0)` is the closest thing to CSS `z-index:
/// auto` — the element participates in normal stacking order without
/// being lifted above siblings.
fn z_index(v: &str) -> Option<Op> {
    match v {
        "auto" => return Some(Op::Layer(Layer::Relative(0))),
        "overlay" => return Some(Op::Layer(Layer::Overlay)),
        _ => {}
    }
    if let Some(level) = v.strip_prefix("overlay-") {
        let n = plain_num(level)?;
        if n.fract() != 0.0 {
            return None;
        }
        // Reject > u8::MAX instead of silently wrapping.
        let level = u8::try_from(n as i32).ok()?;
        return Some(Op::Layer(Layer::OverlayLevel(level)));
    }
    let n = plain_num(v).or_else(|| arbitrary(v).and_then(plain_num))?;
    if n.fract() != 0.0 {
        return None;
    }
    // Reject out-of-i16-range instead of silently wrapping.
    let n_i16 = i16::try_from(n as i32).ok()?;
    Some(Op::Layer(Layer::Relative(n_i16)))
}

/// Parse `text-<value>`: font size, alignment, overflow, arbitrary px, or
/// colour.
///
/// # Disambiguation order
///
/// 1. Named sizes (`xs`, `sm`, `base`, …, `9xl`)
/// 2. Alignment (`left`, `center`, …)
/// 3. Overflow (`clip`, `ellipsis`)
/// 4. Arbitrary pixels (`[14px]`) — checked **before** colour so
///    `text-[14px]` is a size, not an invalid hex
/// 5. Colour (named or `[#hex]`)
///
/// Note `md` is an alias for `base` (16px) — Tailwind uses `base`, but
/// FreyaCN's theme system uses `md`, so both are accepted to reduce
/// friction for users switching between the two.
fn text(v: &str) -> Option<Op> {
    let size = match v {
        "xs" => Some(12.0),
        "sm" => Some(14.0),
        "base" | "md" => Some(16.0),
        "lg" => Some(18.0),
        "xl" => Some(20.0),
        "2xl" => Some(24.0),
        "3xl" => Some(30.0),
        "4xl" => Some(36.0),
        "5xl" => Some(48.0),
        "6xl" => Some(60.0),
        "7xl" => Some(72.0),
        "8xl" => Some(96.0),
        "9xl" => Some(128.0),
        _ => None,
    };
    if let Some(s) = size {
        return Some(Op::FontSize(s));
    }
    match v {
        "left" => return Some(Op::TextAlign(TextAlign::Left)),
        "right" => return Some(Op::TextAlign(TextAlign::Right)),
        "center" => return Some(Op::TextAlign(TextAlign::Center)),
        "justify" => return Some(Op::TextAlign(TextAlign::Justify)),
        "start" => return Some(Op::TextAlign(TextAlign::Start)),
        "end" => return Some(Op::TextAlign(TextAlign::End)),
        "clip" => return Some(Op::TextOverflow(TextOverflow::Clip)),
        "ellipsis" => return Some(Op::TextOverflow(TextOverflow::Ellipsis)),
        _ => {}
    }
    if let Some(px_v) = arbitrary_px(v) {
        return Some(Op::FontSize(px_v));
    }
    color_value(v).map(Op::TextColor)
}

/// Parse `font-<value>`: weight or family.
///
/// Font *family* classes (`font-sans`, `font-serif`, `font-mono`) replace
/// the entire family stack. Font *weight* classes (`font-bold`, …) set the
/// weight. `font-stretch-*` is handled by [`font_stretch`] because it has
/// a multi-dash prefix.
fn font(v: &str) -> Option<Op> {
    Some(match v {
        "thin" => Op::FontWeight(FontWeight::THIN),
        "extralight" => Op::FontWeight(FontWeight::EXTRA_LIGHT),
        "light" => Op::FontWeight(FontWeight::LIGHT),
        "normal" => Op::FontWeight(FontWeight::NORMAL),
        "medium" => Op::FontWeight(FontWeight::MEDIUM),
        "semibold" => Op::FontWeight(FontWeight::SEMI_BOLD),
        "bold" => Op::FontWeight(FontWeight::BOLD),
        "extrabold" => Op::FontWeight(FontWeight::EXTRA_BOLD),
        "black" => Op::FontWeight(FontWeight::BLACK),
        "sans" => Op::FontFamily(Family::Sans),
        "serif" => Op::FontFamily(Family::Serif),
        "mono" => Op::FontFamily(Family::Mono),
        _ => return None,
    })
}

/// Parse `font-stretch-<value>`.
fn font_stretch(v: &str) -> Option<Op> {
    Some(Op::FontStretch(match v {
        "ultra-condensed" => FontWidth::ULTRA_CONDENSED,
        "extra-condensed" => FontWidth::EXTRA_CONDENSED,
        "condensed" => FontWidth::CONDENSED,
        "semi-condensed" => FontWidth::SEMI_CONDENSED,
        "normal" => FontWidth::NORMAL,
        "semi-expanded" => FontWidth::SEMI_EXPANDED,
        "extra-expanded" => FontWidth::EXTRA_EXPANDED,
        "ultra-expanded" => FontWidth::ULTRA_EXPANDED,
        _ => return None,
    }))
}

/// Parse `tracking-<value>`: letter spacing in `em`.
///
/// Arbitrary values accept an optional `em` suffix: `tracking-[0.05em]`
/// and `tracking-[0.05]` are equivalent. The value is stored in
/// `Op::Tracking`, which applies at `apply` time as `font_size × em` —
/// see [`Op::is_tracking`] for why that matters.
fn tracking(v: &str) -> Option<Op> {
    Some(Op::Tracking(match v {
        "tighter" => -0.05,
        "tight" => -0.025,
        "normal" => 0.0,
        "wide" => 0.025,
        "wider" => 0.05,
        "widest" => 0.1,
        other => {
            let inner = arbitrary(other)?.trim();
            signed_num(inner.strip_suffix("em").unwrap_or(inner))?
        }
    }))
}

/// Parse `leading-<value>`: line height as a unitless multiplier.
fn leading(v: &str) -> Option<Op> {
    Some(Op::LineHeight(match v {
        "none" => 1.0,
        "tight" => 1.25,
        "snug" => 1.375,
        "normal" => 1.5,
        "relaxed" => 1.625,
        "loose" => 2.0,
        other => arbitrary_num(other)?,
    }))
}

/// Parse `origin-<value>`: transform origin.
fn origin(v: &str) -> Option<Op> {
    Some(Op::Origin(match v {
        "center" => TransformOrigin::center(),
        "top" => TransformOrigin::top(),
        "top-right" => TransformOrigin::top_right(),
        "right" => TransformOrigin::right(),
        "bottom-right" => TransformOrigin::bottom_right(),
        "bottom" => TransformOrigin::bottom(),
        "bottom-left" => TransformOrigin::bottom_left(),
        "left" => TransformOrigin::left(),
        "top-left" => TransformOrigin::top_left(),
        _ => return None,
    }))
}

/// Parse `cursor-<value>`: mouse cursor icon.
///
/// `cursor-none` is special — it hides the cursor entirely via
/// `Style::cursor_visible = false` rather than selecting a specific icon.
fn cursor(v: &str) -> Option<Op> {
    if v == "none" {
        return Some(Op::CursorHidden);
    }
    Some(Op::Cursor(match v {
        "auto" | "default" => CursorIcon::Default,
        "context-menu" => CursorIcon::ContextMenu,
        "help" => CursorIcon::Help,
        "pointer" => CursorIcon::Pointer,
        "progress" => CursorIcon::Progress,
        "wait" => CursorIcon::Wait,
        "cell" => CursorIcon::Cell,
        "crosshair" => CursorIcon::Crosshair,
        "text" => CursorIcon::Text,
        "vertical-text" => CursorIcon::VerticalText,
        "alias" => CursorIcon::Alias,
        "copy" => CursorIcon::Copy,
        "move" => CursorIcon::Move,
        "no-drop" => CursorIcon::NoDrop,
        "not-allowed" => CursorIcon::NotAllowed,
        "grab" => CursorIcon::Grab,
        "grabbing" => CursorIcon::Grabbing,
        "e-resize" => CursorIcon::EResize,
        "n-resize" => CursorIcon::NResize,
        "ne-resize" => CursorIcon::NeResize,
        "nw-resize" => CursorIcon::NwResize,
        "s-resize" => CursorIcon::SResize,
        "se-resize" => CursorIcon::SeResize,
        "sw-resize" => CursorIcon::SwResize,
        "w-resize" => CursorIcon::WResize,
        "ew-resize" => CursorIcon::EwResize,
        "ns-resize" => CursorIcon::NsResize,
        "nesw-resize" => CursorIcon::NeswResize,
        "nwse-resize" => CursorIcon::NwseResize,
        "col-resize" => CursorIcon::ColResize,
        "row-resize" => CursorIcon::RowResize,
        "all-scroll" => CursorIcon::AllScroll,
        "zoom-in" => CursorIcon::ZoomIn,
        "zoom-out" => CursorIcon::ZoomOut,
        "dnd-ask" => CursorIcon::DndAsk,
        "all-resize" => CursorIcon::AllResize,
        _ => return None,
    }))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::styling::styled::StyledExt;
    use freya_core::prelude::{Color, Fill};

    fn s(classes: &str) -> Style {
        Style::default().style(classes)
    }

    #[test]
    fn spacing_and_negative_margin() {
        let st = s("p-4 -mt-2 mx-1.5");
        assert_eq!(st.padding.top(), 16.0);
        assert_eq!(st.margin.top(), -8.0);
        assert_eq!(st.margin.left(), 6.0);
    }

    #[test]
    fn gap_and_arbitrary() {
        assert_eq!(s("gap-2").spacing, Length::new(8.0));
        assert_eq!(s("gap-[13px]").spacing, Length::new(13.0));
        // Lenient unitless: `[10]` == `[10px]`.
        assert_eq!(s("gap-[10]").spacing, Length::new(10.0));
        // Interior whitespace allowed inside brackets (bracket-aware split).
        assert_eq!(s("gap-[ 10px ]").spacing, Length::new(10.0));
    }

    #[test]
    fn split_classes_is_bracket_aware() {
        assert_eq!(split_classes("p-4 text-lg"), vec!["p-4", "text-lg"]);
        assert_eq!(split_classes("gap-[ 10px ]"), vec!["gap-[ 10px ]"]);
        assert_eq!(
            split_classes("p-4  w-[ a b ]  text-lg"),
            vec!["p-4", "w-[ a b ]", "text-lg"]
        );
        // Stray `]` doesn't hide anything: it's just a token.
        assert_eq!(split_classes("p-4 ] text-lg"), vec!["p-4", "]", "text-lg"]);
        assert!(split_classes("").is_empty());
        assert!(split_classes("   ").is_empty());
    }

    #[test]
    fn dimensions() {
        assert_eq!(s("w-1/2").width, pct(50.0));
        assert_eq!(s("h-full").height, pct(100.0));
        assert_eq!(s("size-6").width, px(24.0));
        assert_eq!(s("max-w-xl").max_width, px(576.0));
    }

    #[test]
    fn tracking_is_order_independent() {
        let expected = Style::default().text_lg().tracking_widen();
        assert_eq!(s("tracking-wide text-lg"), expected);
        assert_eq!(s("text-lg tracking-wide"), expected);
    }

    #[test]
    fn image_classes() {
        let st = s("object-cover blur-md snap-to-grid");
        assert_eq!(st.image_blur, 12.0);
        assert!(st.snap_to_grid);
        assert_eq!(s("blur-[10px]").image_blur, 10.0);
        assert_eq!(s("object-center"), Style::default().object_center());
    }

    #[test]
    fn image_classes_work_in_variants() {
        assert!(!StyledString::parse("hover:blur-md").is_empty());
    }

    #[test]
    fn z_index_classes() {
        assert_eq!(s("z-10").layer, Layer::Relative(10));
        assert_eq!(s("-z-10").layer, Layer::Relative(-10));
        assert_eq!(s("z-[5]").layer, Layer::Relative(5));
        assert_eq!(s("z-auto").layer, Layer::Relative(0));
        assert_eq!(s("z-overlay").layer, Layer::Overlay);
        assert_eq!(s("z-overlay-3").layer, Layer::OverlayLevel(3));
    }

    #[test]
    fn z_index_rejects_out_of_range() {
        // Silently truncating would set wrong values; rejecting drops the
        // class instead (with a debug warning).
        assert!(StyledString::parse("z-40000").is_empty());
        assert!(StyledString::parse("z--40000").is_empty());
        assert!(StyledString::parse("z-overlay-300").is_empty());
        assert!(StyledString::parse("-z-32768").is_empty());
    }

    #[test]
    fn z_index_works_in_variants() {
        assert!(!StyledString::parse("hover:z-50").is_empty());
    }

    #[test]
    fn unknown_classes_are_ignored() {
        assert_eq!(s("definitely-not-a-class p-1"), s("p-1"));
    }

    #[test]
    fn bracket_colon_is_not_a_variant() {
        let (v, rest) = split_variant("w-[calc(1:2)]");
        assert!(v.is_none());
        assert_eq!(rest, "w-[calc(1:2)]");
    }

    #[test]
    fn stray_bracket_doesnt_swallow_variant_colon() {
        // `]` with no matching `[` should not make depth negative and
        // prevent `hover:` from being recognised.
        let (v, rest) = split_variant("x]:hover:bg-red-500");
        assert_eq!(v, Some("x]"));
        assert_eq!(rest, "hover:bg-red-500");
    }

    #[test]
    fn arbitrary_colors() {
        assert_eq!(
            s("bg-[#ff0000]").background,
            Fill::Color(Color::from_rgb(255, 0, 0))
        );
        assert_eq!(
            s("text-[#fff]").text_color,
            Fill::Color(Color::from_rgb(255, 255, 255))
        );
        // Short hex with alpha.
        let bg = s("bg-[#f00f]").background;
        assert_eq!(bg, Fill::Color(Color::from_argb(255, 255, 0, 0)));
    }

    #[test]
    fn border_classes() {
        // `border` = 1px all edges.
        let b = s("border").border.unwrap();
        assert_eq!(b.width.top, 1.0);

        // `border-2` = 2px all edges.
        let b = s("border-2").border.unwrap();
        assert_eq!(b.width.top, 2.0);

        // `border-t-4` = 4px top only.
        let b = s("border-t-4").border.unwrap();
        assert_eq!(b.width.top, 4.0);
        assert_eq!(b.width.right, 0.0);

        // `border-x-2` = 2px both left and right.
        let b = s("border-x-2").border.unwrap();
        assert_eq!(b.width.left, 2.0);
        assert_eq!(b.width.right, 2.0);

        // `border-red-500` = 1px + that colour.
        let b = s("border-red-500").border.unwrap();
        assert_eq!(b.width.top, 1.0);

        // `border-none` removes the border object.
        assert!(s("border-none").border.is_none());

        // `border-[3px]` arbitrary width.
        let b = s("border-[3px]").border.unwrap();
        assert_eq!(b.width.top, 3.0);
    }

    #[test]
    fn border_width_and_color_combine_in_any_order() {
        let a = s("border-2 border-red-500");
        let b = s("border-red-500 border-2");
        assert_eq!(a.border.unwrap().width.top, b.border.unwrap().width.top);
    }
}
