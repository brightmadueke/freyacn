//! Styling for Freya elements.
//!
//! Five layers, each independently useful:
//!
//! | Layer | Item | Purpose |
//! |---|---|---|
//! | Data | [`Style`] | One flat struct holding every paint / typography / border / shadow / layout attribute. |
//! | Builder | [`StyledExt`] | Fluent `self -> Self` methods on anything that exposes a `Style`. |
//! | Parser | [`StyledString`] | Tailwind-style class strings that fan out into the same `Style` mutations as the builder. |
//! | State | [`StyleVariant`] / [`StyleVariants`] | Pseudo-state patches (`:hover`, `:focus`, `aria-invalid`, …) stored *on* the style. |
//! | Palette | [`StyledColor`] and friends | Tailwind colours, gradients, shaders — everything convertible to [`freya_core::prelude::Fill`]. |
//!
//! # Supporting modules
//!
//! Two `pub` modules sit **underneath** the five layers above. They're not
//! part of the everyday user-facing API, but you'll touch them if you're
//! writing a new element:
//!
//! * [`apply`] — `Style` → primitive mapping. Three entry points:
//!   [`apply_to_rect`](apply::apply_to_rect) copies layout / paint /
//!   border / effects / shadows / inheritable text properties onto a
//!   `rect()`; [`apply_text_style`](apply::apply_text_style) does the
//!   text-only slice for any primitive implementing `TextStyleExt`
//!   (`Label`, `Paragraph`, and future text elements);
//!   [`HasFontLists`](apply::HasFontLists) +
//!   [`apply_font_lists`](apply::apply_font_lists) handle the
//!   `font_family` / `font_feature` / `text_shadow` loops that aren't
//!   identical across primitives.
//! * [`semantic_events`] — the shared semantic-event layer.
//!   [`SemanticEvents`] is the callback bundle every interactive element
//!   carries; [`HasSemanticEvents`] adds the fluent `.on_hover(..)` /
//!   `.on_focus(..)` setters;
//!   [`attach_pointer_handlers`](semantic_events::attach_pointer_handlers)
//!   and
//!   [`use_semantic_focus_events`](semantic_events::use_semantic_focus_events)
//!   do the actual wiring during render.
//!
//! Element code imports these from their module paths; only
//! [`HasSemanticEvents`] and [`SemanticEvents`] are re-exported from this
//! module (see the bottom of this file) because user code needs the trait
//! in scope to call the setters.
//!
//! # Mental model
//!
//! Freya itself is style-agnostic: each primitive has its own props
//! (`ButtonProps`, `TextProps`, …). FreyaCN sits on top of that with a
//! single shared [`Style`] whose fields you fold into a primitive's own
//! props, so a chain written once transfers unchanged to whatever element
//! renders it:
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
//!     .border_1()
//!     .border_color(neutral_700())
//!     .rounded_md()
//!     .shadow_md()
//!     .font_semibold()
//!     .hover(|s| s.bg(neutral_800()))
//!     .active(|s| s.bg(neutral_950()))
//!     .disabled(|s| s.opacity_50().cursor_not_allowed());
//!
//! // …or parse a Tailwind-style string into the same struct.
//! let s = Style::default().style(
//!     "bg-neutral-900 text-neutral-50 px-4 py-2 border border-neutral-700 \
//!      rounded-md shadow-md font-semibold hover:bg-neutral-800 active:bg-neutral-950 \
//!      disabled:opacity-50",
//! );
//! ```
//!
//! # Examples
//!
//! ## Style inheritance
//!
//! Only CSS-`inherit`-able properties flow from parent to child. Set a
//! text colour on an ancestor and every descendant inherits it; set
//! `padding` or `border` and neither does:
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::{gray_200, red_500};
//! use torin::gaps::Gaps;
//!
//! div()
//!     .text_color(red_500())
//!     .padding(Gaps::new(16.0, 0.0, 16.0, 0.0))
//!     .border_1()
//!     .border_color(gray_200())
//!     .child(
//!         // Inherits `red_500`; does *not* inherit the padding or border.
//!         div().child(text("red text, no padding, no border"))
//!     )
//! ```
//!
//! The inheritable slice lives in
//! [`Style::inheritable_subset`](crate::styling::Style::inheritable_subset):
//! `text_color`, `text_align`, all `font_*`, `letter_spacing`,
//! `line_height`, `text_shadows`, `text_decoration`, `cursor`,
//! `cursor_visible`. Everything else (`background`, `border`, `shadows`,
//! `padding`, `margin`, `width`, `radius`, `variants`, …) stays local to
//! the element that set it — box shadow and border both sit outside the
//! inherit chain, matching CSS. Text shadow *does* cascade, matching CSS.
//!
//! ## Using semantic events
//!
//! Every element that implements [`HasSemanticEvents`] accepts the same
//! eight callbacks — each fires **once per state transition**, not per
//! pointer tick:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! div()
//!     .on_hover(|_| println!("pointer entered"))
//!     .on_hover_end(|_| println!("pointer left"))
//!     .on_pressed(|_| println!("pointer down"))
//!     .on_released(|_| println!("pointer up"))
//!     .on_focus(|| println!("focused"))
//!     .on_blur(|| println!("blurred"))
//!     .on_focus_visible(|| println!("keyboard-focused"))
//!     .on_focus_visible_end(|| println!("no longer keyboard-focused"))
//!     .child(text("Hover me"))
//! ```
//!
//! Raw events that aren't covered by the semantic layer
//! (`.on_press`, `.on_key_down`, `.on_mouse_down`, `.on_wheel`, …) still
//! go through [`EventHandlersExt`](freya::prelude::EventHandlersExt).
//!
//! ## Writing a new element
//!
//! Elements follow a consistent 5-step recipe. This is the shape every
//! element in [`crate::elements`] follows:
//!
//! ```
//! use crate::styling::apply::apply_to_rect;
//! use crate::styling::semantic_events::{
//!     attach_pointer_handlers, use_semantic_focus_events,
//!     HasSemanticEvents, SemanticEvents,
//! };
//! use crate::styling::style_variants::StyleVariant;
//! use crate::styling::{InheritedStyle, Style, StyledExt};
//!
//! pub struct MyElement {
//!     style: Style,
//!     semantic_events: SemanticEvents,
//!     // …children, event_handlers, a11y data, state props…
//! }
//!
//! impl Component for MyElement {
//!     fn render(&self) -> impl IntoElement {
//!         // 1. Local reactive state for the live variant set.
//!         let hovering = use_state(|| false);
//!         let pressing = use_state(|| false);
//!         let a11y_id = use_a11y();
//!         let focus = use_focus(a11y_id);
//!
//!         // 2. Resolve base style: inherited ∪ user style.
//!         //    Ancestors provide `InheritedStyle` from their own render;
//!         //    the root falls back to the theme default.
//!         let base = match try_consume_context::<InheritedStyle>() {
//!             Some(InheritedStyle(parent)) => parent
//!                 .inheritable_subset()
//!                 .merge_style(&self.style),
//!             None => Style::default().merge_style(&self.style),
//!         };
//!
//!         // 3. Apply live variants on top of the base.
//!         let focus_state = focus();
//!         let mut live = Vec::new();
//!         if *hovering.read() { live.push(StyleVariant::Hover); }
//!         if *pressing.read() { live.push(StyleVariant::Active); }
//!         if matches!(focus_state, Focus::Keyboard | Focus::Pointer) {
//!             live.push(StyleVariant::Focus);
//!         }
//!         if focus_state == Focus::Keyboard {
//!             live.push(StyleVariant::FocusVisible);
//!         }
//!         let style = self.style.variants.apply_all(&live, base);
//!
//!         // 4. Fire focus events + provide resolved style to descendants.
//!         use_semantic_focus_events(&self.semantic_events, focus_state);
//!         use_provide_context(|| InheritedStyle(style.clone()));
//!
//!         // 5. Render primitive, copy style, wire pointer events.
//!         let mut el = apply_to_rect(rect(), &style);
//!         *el.event_handlers_mut() = self.event_handlers.clone();
//!         if !self.disabled {
//!             el = attach_pointer_handlers(
//!                 el,
//!                 &self.semantic_events,
//!                 hovering,
//!                 pressing,
//!                 /* needs_variant_tracking */ !live.is_empty(),
//!                 /* focus_on_press */      None,
//!             );
//!         }
//!         el.children(self.children.clone())
//!     }
//! }
//! ```
//!
//! The three internal calls separate responsibilities cleanly:
//!
//! | Helper | What it does |
//! |---|---|
//! | [`apply_to_rect`](apply::apply_to_rect) (or [`apply_text_style`](apply::apply_text_style)) | Copies `Style` fields onto a primitive. `apply_to_rect` also handles `Style::border` via `DecorationExt`, and `Style::shadows` / `Style::text_shadows` via `EffectExt`. |
//! | [`attach_pointer_handlers`](semantic_events::attach_pointer_handlers) | Wires the four pointer events (`enter`, `leave`, `down`, global `up`) that drive `:hover` / `:active` and fire `.on_hover(..)` / `.on_pressed(..)`. |
//! | [`use_semantic_focus_events`](semantic_events::use_semantic_focus_events) | Diffs focus state across renders and fires `.on_focus(..)` / `.on_blur(..)` / `.on_focus_visible(..)` exactly once per transition. |
//!
//! Elements that **don't** render children (e.g. `text`, `paragraph`,
//! `button`) skip step 4's `use_provide_context` — there's nothing below
//! them to consume the context.
//!
//! ## Theme defaults vs bare patches
//!
//! Step 2 above starts from `Style::default()` for the **theme layer**,
//! which is correct: theme defaults are what you get when no class string
//! touches a field.
//!
//! Components generated by `#[component]` follow a slightly different
//! shape. Their fold is:
//!
//! ```text
//! Style::default()                    ← theme layer
//!   .style(  base class string  )
//!   .merge_style(&variant.resolve())  ← variant patches
//!   .merge_style(&size.resolve())     ← size patches
//!   .merge_style(&self.style)         ← user overrides
//! ```
//!
//! `variant.resolve()`, `size.resolve()`, and the `styled` extension
//! field that `#[component]` generates all start from **`Style::bare()`**,
//! not `Style::default()`. This is deliberate and load-bearing:
//! `merge_style` copies any field that differs from `Style::bare()`, so
//! a `Style::default()`-based variant would leak the theme's
//! `text_color = foreground` and `background = background` into the fold
//! and silently overwrite whatever the class string set. Starting from
//! `bare()` makes each overlay a **sparse patch** — only fields the class
//! string touches are non-bare, so the theme defaults survive the merge.
//!
//! Concretely: a `Button` variant whose class string is
//! `bg-primary text-primary-foreground` renders with `text_color =
//! primaryForeground`, not `foreground`. If `resolve()` started from
//! `Style::default()`, the `foreground` value would come back on every
//! `merge_style` pass and clobber the variant's own class string.
//!
//! # Border
//!
//! [`Style::border`](crate::styling::Style) is `Option<Border>`. `None`
//! means "no border"; `Some(Border)` draws an outline with a per-side
//! width, a fill colour, and an alignment (inner / outer / center).
//!
//! The border field is **not inherited** — same as CSS. A child `div()`
//! inside a bordered parent starts with `border: None`.
//!
//! Two ways to set it:
//!
//! * **Raw** — `.border(Some(Border::new().fill(gray_200()).width(1.0)))`.
//!   Matches the field exactly; useful for conditionals.
//! * **Tailwind-style** — `.border_1().border_color(gray_200())`. The
//!   per-property helpers lazily create a `Border` if none exists, so
//!   order in the chain doesn't matter.
//!
//! Available helpers on [`StyledExt`]:
//!
//! | Helper | Tailwind | Effect |
//! |---|---|---|
//! | `.border_0()` / `_1()` / `_2()` / `_3()` / `_4()` / `_8()` | `border-{n}` | Width on all sides |
//! | `.border_t(..)` / `_r` / `_b` / `_l` | `border-t` / `-r` / `-b` / `-l` | Width per side |
//! | `.border_x(..)` / `_y(..)` | `border-x` / `-y` | Width on left+right / top+bottom |
//! | `.border_color(..)` | `border-{color}` | Border fill |
//! | `.border_inner()` / `_outer()` / `_center()` | *(freya-specific)* | `BorderAlignment` |
//! | `.border_none()` | `border-none` | Sets `border: None` outright |
//! | `.ring()` / `.ring_0()` … `.ring_3()` | `ring` / `ring-{n}` | Width on all sides *(see note)* |
//! | `.ring_color(..)` | `ring-{color}` | Border fill *(see note)* |
//!
//! **Ring is approximated as border.** Freya has no separate `ring`
//! field, so `ring-*` utilities map onto `Style::border` — `ring-[3px]`
//! becomes `border_width(3.0)` and `ring-ring/50` becomes
//! `border_color(ring @ 50%)`. Visually close to Tailwind v4's shadcn
//! pattern (`focus-visible:ring-[3px] focus-visible:ring-ring/50`), but
//! **not a true offset** — the "ring" and the "border" are the same
//! `Border` struct, so you can't have both a visible border and a
//! separate ring at once. If you need them independently, use a
//! wrapper `div()`, or (see the Shadows section) apply an offset shadow
//! for the ring effect.
//!
//! The `border` **does not inherit**, so it's local to the element that
//! set it — same rule as `padding`, `background`, and `radius`.
//!
//! Only primitives that implement `DecorationExt` render borders. In the
//! built-in element set, that's `div()`, `button()`, `checkbox()`,
//! `radio()`, `input()`, and `image()`. `text()`, `p()`, and `icon()`
//! ignore borders — wrap in a `div()` to outline text or SVG content.
//! See the [`crate::elements`] module docs for the full matrix.
//!
//! # Shadows
//!
//! `Style` carries two shadow vectors:
//!
//! * `Style::shadows: Vec<Shadow>` — **box shadows**, drawn around the
//!   element's frame (Tailwind `shadow-*`).
//! * `Style::text_shadows: Vec<TextShadow>` — **glyph shadows**, drawn
//!   behind the text layer only (Tailwind `text-shadow-*`).
//!
//! Both are layered: entries are painted in order, and the size presets
//! **replace** the vector while `.shadow(..)` / `.text_shadow(..)`
//! **append** to it. That mirrors CSS — `box-shadow: a, b` renders both
//! layers, but writing `box-shadow: a; box-shadow: b` in a stylesheet
//! leaves only `b`. Manual multi-layer shadows therefore use the base
//! methods:
//!
//! ```
//! div()
//!     .shadow(Shadow::new().y(2.0).blur(4.0).color(black().opacity(0.1)))
//!     .shadow(Shadow::new().y(8.0).blur(16.0).color(black().opacity(0.05)))
//! ```
//!
//! ## Box shadows
//!
//! | Helper | Tailwind | Effect |
//! |---|---|---|
//! | `.shadow(..)` | — | Append one shadow |
//! | `.shadow_none()` | `shadow-none` | Clear the vector |
//! | `.shadow_2xs()` / `_xs` / `_sm` / `_md` / `_lg` / `_xl` / `_2xl` | `shadow-{size}` | **Replace** with the Tailwind v4.1 preset |
//! | `.shadow_inner()` | `shadow-inner` | **Replace** with the inset preset |
//! | `.shadow_color(..)` | `shadow-{color}` | Recolour every layer in the vector |
//!
//! `shadow_color` is meant to be chained *after* a size, matching
//! Tailwind's `shadow-{size} shadow-{color}` pairing. It's a no-op when
//! the vector is empty.
//!
//! ## Text shadows
//!
//! | Helper | Tailwind | Effect |
//! |---|---|---|
//! | `.text_shadow(..)` | — | Append one shadow |
//! | `.text_shadow_none()` | `text-shadow-none` | Clear the vector |
//! | `.text_shadow_2xs()` / `_xs` / `_sm` / `_md` / `_lg` | `text-shadow-{size}` | **Replace** with the Tailwind v4.1 preset |
//! | `.text_shadow_color(..)` | `text-shadow-{color}` | Recolour every layer |
//!
//! ## The `blur_sigma` wrinkle
//!
//! Freya's `TextShadow::blur_sigma` is a **standard deviation**, not the
//! CSS `blur-radius` you write in `text-shadow:` shorthand. The SVG spec
//! (which Skia follows) defines `sigma = radius / 2`, so the text-shadow
//! presets halve the CSS radius before constructing the `TextShadow`.
//! `blur_sigma` values of `0.5`, `1.0`, `1.5`, `2.0`, `3.0`, `4.0`,
//! `6.0`, `10.0` therefore correspond to CSS radii of `1px`, `2px`,
//! `3px`, `4px`, `6px`, `8px`, `12px`, `20px` respectively.
//!
//! ## Rendering
//!
//! Only primitives that implement `EffectExt` render box shadows, and
//! only text primitives (`Label`, `Paragraph`) render text shadows.
//! `apply_to_rect` walks `Style::shadows` for you when rendering a
//! `rect()`; `apply_text_style` handles the `text_shadows` slice.
//! `button()`, `input()`, `checkbox()`, `radio()`, and `image()` all
//! forward `Style::shadows` to their inner `rect()` — text shadows on
//! those same elements land on their label child, if they have one.
//!
//! ## Inheritance
//!
//! Box shadow does **not** inherit (matching CSS) — a shadowed parent
//! does not shadow its children. Text shadow **does** inherit (also
//! matching CSS) via `Style::inheritable_subset`. `text_shadows` is in
//! the inheritable set; `shadows` is not.
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
//! The `aria-*` variants slot into the same priority ladder between
//! `selected` and `disabled`:
//!
//! ```text
//! dark < hover < focus < focus-visible < focus-within
//!   < selected < aria-selected < checked < aria-checked
//!   < active < aria-expanded < aria-pressed < aria-invalid
//!   < aria-required < aria-readonly < aria-hidden
//!   < aria-disabled < disabled
//! ```
//!
//! So `aria-invalid:*` beats `hover:*` (invalid styling persists while
//! hovering) but `disabled:*` still beats `aria-invalid:*`.
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
//! # Reset limitation
//!
//! [`Style::merge_style`](crate::styling::Style::merge_style) uses a
//! **"differs from `Style::bare()`"** test — a field is copied from the
//! argument only if it isn't at its bare default. That makes partial
//! overrides ergonomic (you only write the fields you care about) at the
//! cost of one sharp edge: **a field cannot be explicitly reset to its
//! bare value** by an argument that happens to set it to that value.
//!
//! ```
//! // `child` wants to force padding back to zero.
//! let parent = Style::bare().padding(Gaps::new(16.0, 16.0, 16.0, 16.0));
//! let child  = Style::bare().padding(Gaps::new(0.0, 0.0, 0.0, 0.0));
//! let merged = parent.merge_style(&child);
//! // `merged.padding` is still (16, 16, 16, 16) — child's zero doesn't win.
//! ```
//!
//! This bites `border` in a specific way: `Style::bare()` has
//! `border: None`. A `.border_none()` call **also** sets `None`, so it
//! can't override an existing border inside a variant — `None == None`
//! skips the merge. To clear a border in a `:hover` / `:focus` variant,
//! use `.border_0()` instead, which sets `Some(zero-width Border)` that
//! *does* differ from `None` and therefore wins:
//!
//! ```
//! div()
//!     .border_2().border_color(blue_500())
//!     .hover(|s| s.border_0())          // ✓ border visually disappears on hover
//!     .hover(|s| s.border_none())       // ✗ no-op — border stays
//! ```
//!
//! Shadow vectors have the same caveat in reverse: `Style::bare()` has
//! `shadows: vec![]` / `text_shadows: vec![]`, and every size preset
//! writes a **non-empty** vector, so presets and `.shadow(..)` merges
//! work cleanly. `.shadow_none()` / `.text_shadow_none()` — which write
//! `vec![]` — hit the same "differs from bare" trap as `.border_none()`
//! when used as a variant override. If you need to cancel a shadow
//! inside `:hover` / `:focus`, apply an empty-layer preset or re-render
//! the base style without the shadow.
//!
//! Every element that applies user-agent defaults (button, paragraph,
//! checkbox) exposes a `no_default_style()` escape hatch so users can
//! bypass the UA layer entirely and use their `Style` against the theme
//! defaults directly:
//!
//! ```
//! button().no_default_style().padding(Gaps::new(0.0, 0.0, 0.0, 0.0))
//! ```
//!
//! A full `StylePatch` type (parallel `Option<T>` fields) would fix this
//! properly, but it's a much larger change to the `Style` API and isn't
//! in scope.
//!
//! # String parser
//!
//! [`StyledString`] parses a whitespace-separated Tailwind-style class
//! string into the same `Style` mutations the fluent API performs, so
//! the two spellings are interchangeable — pick whichever reads better
//! at the call site. The parser also understands the grammar that the
//! fluent API can't express directly:
//!
//! * **Pseudo-state prefixes** which land as [`StyleVariants`] entries:
//!   `dark:`, `hover:`, `focus:`, `focus-visible:`, `focus-within:`,
//!   `active:`, `disabled:`, `checked:`, `selected:`, plus the full
//!   **aria family** — `aria-invalid:`, `aria-checked:`,
//!   `aria-disabled:`, `aria-expanded:`, `aria-pressed:`,
//!   `aria-readonly:`, `aria-required:`, `aria-selected:`,
//!   `aria-hidden:`. Multiple prefixes stack (`dark:aria-invalid:*`).
//! * **Theme tokens** in colour utilities. `bg-primary`,
//!   `text-primary-foreground`, `border-ring`, `ring-ring/50` resolve in
//!   three steps: the built-in palette, then the active theme's *fill*
//!   map, then the theme's *color* map. Kebab-case and camelCase are
//!   both accepted, so `text-primary-foreground` matches
//!   `primaryForeground` from the theme JSON.
//! * **Shadow utilities.** `shadow-{2xs,xs,sm,md,lg,xl,2xl,inner,none}`
//!   and `text-shadow-{2xs,xs,sm,md,lg,none}` map to the same presets
//!   as the fluent API. `shadow-{color}` recolours every active layer.
//! * **`ring-*` utilities** mapped onto the border — `ring-[3px]`
//!   becomes border width, `ring-ring/50` becomes border colour at 50 %.
//!   See the Border section for the "not a true offset ring" caveat.
//! * **Arbitrary values** (`w-[137px]`, `text-[12.8px]`, `bg-[#ff0000]`).
//! * **CSS custom properties** (`[--brand:#f00]`) and arbitrary
//!   properties (`[width:100px]`), exposed via [`StyledString`] custom
//!   properties so downstream renderers can thread them into their own
//!   systems.
//! * The `!` important marker — inert here, but accepted so strings
//!   copied from a Tailwind codebase don't warn.
//!
//! Unknown classes **panic** at parse time (with a message naming the
//! offending token and pointing at the likely cause). If you want
//! compile-time safety, add a `#[test]` that calls
//! [`StyledString::parse`] on each variant's `style_str()`:
//!
//! ```ignore
//! #[test]
//! fn all_button_classes_parse() {
//!     for v in [ButtonVariant::Default, ButtonVariant::Destructive /* … */] {
//!         StyledString::parse(v.style_str());
//!     }
//! }
//! ```
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
//! Border utilities use `border_*` (matching Tailwind's `border-*`),
//! **not** `b_*`. The `b_` prefix would collide visually with `bg_*`
//! and neither matches the Tailwind spec.
//!
//! Shadow utilities use `shadow_*` (matching Tailwind's `shadow-*`) and
//! `text_shadow_*` (matching Tailwind's `text-shadow-*`). The base
//! methods `.shadow(..)` / `.text_shadow(..)` shadow the preset names
//! by one underscore — `shadow_md()` is the preset, `shadow(..)` takes
//! a raw `Shadow`. Same for text.
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
//! Borders test the same way — the field is `Option<Border>`, so compare
//! `is_some()` / `is_none()` (or the whole `Border`, since it derives
//! `PartialEq`):
//!
//! ```
//! let base = Style::default().border_2().border_color(red_500());
//! let hovered = applied(base.clone(), &[StyleVariant::Hover]);
//!
//! // Border was cleared by the `:hover` variant (via `.border_0()`):
//! assert!(hovered.border.as_ref().map_or(false, |b| !b.is_visible()));
//! ```
//!
//! Shadows test the same way — the fields are `Vec<Shadow>` /
//! `Vec<TextShadow>`, so compare `.len()`, iterate, or compare the
//! whole vector (both types derive `PartialEq`):
//!
//! ```
//! let base = Style::default().shadow_md();
//! assert_eq!(base.shadows.len(), 2);  // shadow-md is two layers
//!
//! let hovered = applied(
//!     base.clone(),
//!     &[StyleVariant::Hover],
//! );
//! // Assuming the hover closure sets a different preset:
//! // assert_eq!(hovered.shadows.len(), 1);
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
//! * **True offset rings as `ring-*`.** Tailwind v4's `ring-*` paints
//!   an outline *outside* the border, on its own layer. `Style::shadows`
//!   now exists as a first-class field, so a **true ring is
//!   implementable** — a `0 0 0 {width} {colour}` shadow reproduces the
//!   effect precisely, and you can write it by hand today:
//!
//!   ```
//!   div().shadow(
//!       Shadow::new().spread(3.0).color(ring_color.opacity(0.5))
//!   )
//!   ```
//!
//!   `ring-*` *utilities* are still wired to `Style::border` (see the
//!   Border section), so the class-string and fluent spellings don't
//!   currently produce a shadow-based ring. Migrating `ring-*` to
//!   `Style::shadows` is the natural next step; it's held back only by
//!   the change in visual semantics for anyone relying on the current
//!   border-based approximation.
//!
//! # Examples
//!
//! ## Basic bordered card
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::{gray_200, gray_900};
//! use torin::gaps::Gaps;
//!
//! div()
//!     .border_1()
//!     .border_color(gray_200())
//!     .rounded_md()
//!     .padding(Gaps::new(16.0, 16.0, 16.0, 16.0))
//!     .child(text("Card content").text_color(gray_900()))
//! ```
//!
//! ## Elevated card with shadow
//!
//! The `shadow-*` presets mirror Tailwind v4.1, so a card that "floats"
//! is one method call:
//!
//! ```
//! use freyacn::prelude::*;
//! use torin::gaps::Gaps;
//!
//! div()
//!     .bg(white())
//!     .rounded_lg()
//!     .shadow_lg()
//!     .padding(Gaps::new(16.0, 16.0, 16.0, 16.0))
//!     .child(text("Elevated"))
//! ```
//!
//! ## Focus ring via shadow (true offset)
//!
//! `ring-*` utilities map onto `border`, which eats into padding. When
//! you need a ring that sits *outside* the element — the way a real
//! focus ring should — build it from a shadow instead:
//!
//! ```
//! use freyacn::prelude::*;
//! use crate::styling::colors::blue_500;
//!
//! button()
//!     .rounded_md()
//!     .focus_visible(|s| {
//!         s.shadow(
//!             Shadow::new()
//!                 .spread(3.0)
//!                 .color(blue_500().opacity(0.5)),
//!         )
//!     })
//!     .child("Click me")
//! ```
//!
//! ## Text shadow on a headline
//!
//! ```
//! use freyacn::prelude::*;
//!
//! text("Above the fold")
//!     .text_lg()
//!     .text_shadow_sm()
//! ```
//!
//! ## Focus ring via border
//!
//! `.border_2()` on `:focus-visible` is the simplest way to build an
//! accessible focus indicator. Pair with `.border_outer()` so the ring
//! sits *outside* the element's bounds (like a real focus ring) rather
//! than eating into its padding.
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
//! ## Tailwind `ring-*` on the class string
//!
//! Class strings accept `ring-*` even though it maps to `border` under
//! the hood — useful when copying shadcn's utilities verbatim:
//!
//! ```
//! use freyacn::prelude::*;
//!
//! button()
//!     .style(
//!         "bg-primary text-primary-foreground rounded-md shadow-md \
//!          focus-visible:ring-[3px] focus-visible:ring-ring/50 \
//!          aria-invalid:border-destructive aria-invalid:ring-destructive/20",
//!     )
//!     .child("Save")
//! ```
//!
//! ## Left accent stripe
//!
//! Per-side borders let you build stripes without a wrapper:
//!
//! ```
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

pub mod apply;
pub mod colors;
pub mod has;
pub mod semantic_events;
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
/// **border**, **shadows** (box + text), radius, and layout. `Default`
/// is "unstyled": transparent background, black text, `1.0` scale, no
/// border, empty shadow vectors, zero radius / padding / margin / gap —
/// i.e. what a bare Freya element looks like before any utility is
/// chained.
///
/// Re-exported here so primitives can construct a base style and merge
/// it into their own props without reaching into the `style` submodule
/// path. See [`style`] for the field-by-field breakdown.
pub use style::*;

/// Fluent [`StyledExt`] builder.
///
/// Every method is `self -> Self`, so utilities chain. The trait is
/// implemented for anything that exposes a `&mut Style` via
/// [`get_style`](styled::StyledExt::get_style), which is how it reaches
/// both the bare [`Style`] struct and component-owned style wrappers.
///
/// Besides layout / paint / typography helpers, `StyledExt` exposes the
/// **border** family — `.border_1()`, `.border_2()`, `.border_3()`,
/// `.border_t(..)`, `.border_color(..)`, `.border_inner()`, and friends
/// — which mutate `Style::border` directly, plus the **ring** family
/// (`.ring_3()`, `.ring_color(..)`) which approximates Tailwind's ring
/// utilities by writing to the same `Border` field. `.border(..)`
/// replaces the whole `Option<Border>`; the granular helpers lazily
/// create a `Border` if none exists yet.
///
/// The **shadow** family — `.shadow_2xs()` / `_xs` / `_sm` / `_md` /
/// `_lg` / `_xl` / `_2xl` / `_inner`, `.shadow_none()`, and
/// `.shadow_color(..)` — mutates `Style::shadows`. The base
/// `.shadow(..)` method appends a custom `Shadow`, while the size
/// presets replace the whole vector (matching Tailwind's
/// `box-shadow:` semantics). Same shape for `.text_shadow(..)` /
/// `.text_shadow_2xs()` … `.text_shadow_lg()` / `.text_shadow_none()` /
/// `.text_shadow_color(..)`, operating on `Style::text_shadows`.
///
/// The doc comment on [`styled`] carries the primitive-side recipe for
/// applying variants per render — read that before wiring `.hover(..)`
/// into a new element.
pub use styled::{StyledExt, ThemedExt};

/// Tailwind-style class-string parser.
///
/// [`StyledString::parse`] tokenises a whitespace-separated string of
/// Tailwind utilities, pseudo-state prefixes (`hover:`, `dark:`,
/// `aria-invalid:`, …), theme tokens (`bg-primary`,
/// `text-primary-foreground`), arbitrary bracketed values
/// (`w-[137px]`), shadow presets (`shadow-md`, `text-shadow-lg`), and
/// `ring-*` utilities (mapped to border under the hood), then applies
/// them through the same [`StyledExt`] chain the fluent API uses. Use
/// this when a class string is easier to read (or arrives from config)
/// than the equivalent builder call.
///
/// See the [`styled_string`] module docs for the full grammar and the
/// list of unsupported constructs.
pub use styled_string::StyledString;

/// Pseudo-state kinds and the container that stores their patches.
///
/// [`StyleVariant`] is the enum — the interactive states (`Dark`,
/// `Hover`, `Focus`, `FocusVisible`, `FocusWithin`, `Selected`,
/// `Checked`, `Active`, `Disabled`) plus the full aria family
/// (`AriaSelected`, `AriaChecked`, `AriaExpanded`, `AriaPressed`,
/// `AriaInvalid`, `AriaRequired`, `AriaReadonly`, `AriaHidden`,
/// `AriaDisabled`). [`StyleVariants`] is the per-style container those
/// patches live in. The container is `Clone`, cheap to clone (all
/// payloads are `Arc`'d), and compares by variant *kind* only — not by
/// closure identity — so freshly-allocated callbacks don't defeat
/// `Style: PartialEq` across renders.
///
/// See [`style_variants`] for [`StyleVariants::apply_all`]'s priority
/// ordering and the callback signature.
pub use style_variants::{StyleVariant, StyleVariants};

/// Shared semantic-event layer.
///
/// [`HasSemanticEvents`] gives every element that embeds a
/// [`SemanticEvents`] the fluent setters — `.on_hover(...)`,
/// `.on_hover_end(...)`, `.on_pressed(...)`, `.on_released(...)`,
/// `.on_focus(...)`, `.on_blur(...)`, `.on_focus_visible(...)`,
/// `.on_focus_visible_end(...)` — each of which fires once per state
/// transition, not on every pointer tick.
///
/// It also exposes the aria state builders — `.with_aria_invalid(bool)`,
/// `.with_aria_checked(bool)`, `.with_aria_disabled(bool)`, and so on —
/// which flip the corresponding `StyleVariant::Aria*` entry and (for
/// `disabled` / `checked`) mirror the native state flag. Elements that
/// implement `HasSemanticEvents` therefore accept `aria-*:*` class
/// strings out of the box:
///
/// ```ignore
/// input(value)
///     .with_aria_invalid(true)
///     .style("aria-invalid:border-red-500 aria-invalid:ring-red-500/20")
/// ```
///
/// Element authors additionally use
/// [`attach_pointer_handlers`](semantic_events::attach_pointer_handlers)
/// and [`use_semantic_focus_events`](semantic_events::use_semantic_focus_events)
/// during render; those aren't re-exported here because they're internal
/// plumbing, not user-facing API.
pub use semantic_events::{HasSemanticEvents, SemanticEvents};

pub use has::{
    HasContext, HasFlags, HasKind, try_use_has_context, use_has_signal, use_has_signal_when,
};
