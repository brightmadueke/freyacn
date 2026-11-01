# freyacn

> ⚠️ **Work in progress.** `freyacn` is under active development and its public
> API is **not yet stable**. Breaking changes may land in any release.
>
> **Status:** pre-alpha. Expect rough edges, missing features, and occasional
> doc/code drift. If you want to try it, use a `git` dependency pinned to a
> specific commit. If you find a bug or want to discuss the design, please
> open an issue.

A modern, shadcn/ui-inspired component library for
[Freya](https://github.com/marc2332/freya), built around a single styling
trait, a theme system, and a layered variant model.

`freyacn` is the runtime crate: it provides `Style`, `StyleExt`, `StyleVariant`,
the theme, the styled component primitives, and re-exports the
`#[component]`, `#[variant]`, `#[extensions]`, and `#[struct_fields]`
macros. It also re-exports Freya's own prelude, so you don't need a direct
`freya` dependency.

Built against **Freya 0.5.0-rc.10**.

> **Complete API reference:** see [`API.md`](../../API.md) at the workspace root.
> It documents every component, every `StyleExt` method, every macro, and the
> full theme API.

---

## Installation

```toml
[dependencies]
freyacn = "0.2.0-alpha.1"
```

> **Not published to crates.io yet.** Use a git dependency pinned to a
> specific commit for reproducible builds.

Then in your code:

```rust
use freyacn::prelude::*;
```

That single import brings in the macros, `Style` / `StyleExt`, every
component in the crate, `Theme`, `StateFlags`, and Freya's own prelude —
`EventHandler`, `Event`, `PointerEventData`, `LaunchConfig`, `WindowConfig`,
`State`, and `launch` are all in scope.

---

## Quick start

```rust,no_run
use freyacn::prelude::*;

#[component(
    props(tooltip: String),
    ext(children, style),
)]
#[derive(Clone, PartialEq)]
fn CustomButton(text: String) {
    Button()
        .merge_style(style.clone())
        .children(children.clone())
        .child(Label().text(text.clone()))
}

fn app() -> impl IntoElement {
    let theme = Theme::light().slate();
    use_init_cn_theme(theme);

    CustomButton()
        .text("Save")
        .variant(ButtonVariant::Destructive)
        .bg_card()
        .p_6()
}

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)));
}
```

---

## The two halves of the API

Everything in `freyacn` hangs off two abstractions:

| Abstraction    | Purpose                                                                                                                                                                                                                                |
|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`Style`**    | A plain data struct holding every styling field: background, foreground, typography, borders, effects, flex, sizing, spacing, positioning, overflow. All fields are `Option<T>` or `Vec<T>`, so `Style::default()` is the empty style. |
| **`StyleExt`** | A trait with one required method, `fn get_style(&mut self) -> &mut Style`. Every Tailwind-style helper is a provided method that mutates the returned `Style`.                                                                         |

`Style` implements its own trait, so bare `Style` values chain the same way components do:

```rust,no_run
use freyacn::prelude::*;

let s = Style::default()
    .bg_card()
    .text_card_foreground()
    .p_6()
    .corner_radius(12.0)
    .shadow_md()
    .style("font-semibold");
```

Any component participates by implementing one method:

```rust
use freyacn::prelude::*;

struct MyWidget {
    style: Style,
}

impl StyleExt for MyWidget {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}
```

Once that impl exists, every `StyleExt` helper becomes available on
`MyWidget`. There's also a `map_style` provided method that handles the
borrow dance for you:

```rust
fn get_style(&mut self) -> &mut Style { &mut self.style }
// Every provided helper is defined as `self.map_style(|s| s.some_setter(...))`.
```

### Interaction state is a `Div` concern

`Style` is pure data — it has no opinion about hover, focus, or disabled.
State-conditioned styles (`hover:bg-primary/80`, `focus-visible:ring-ring`,
`disabled:opacity-50`) are stored as pending layers inside `Style.states`
and folded at render time by whichever component owns the runtime
observation.

In practice that's always `Div`. Every other component in the crate —
`Button`, `AccordionTrigger`, `Label`, `Icon`, `Divider`, `Group`, and any
custom widget you build — passes its `Style` to a `Div` and lets `Div`
observe state. If you're writing a new component, the pattern is:

```rust,no_run
use freyacn::prelude::*;

Div()
    .merge_style(merged_style)
    .children(children.clone())
```

and you get `hover:` / `focus:` / `active:` / `disabled:` / `dark:`
support for free.

---

## Interactive units and pointer events

Freya's hit tester routes each pointer event to the **topmost interactive**
element under the pointer. When both a parent and a child are interactive,
moving the pointer onto the child fires `pointer_leave` on the parent,
which clears the parent's hover state — even though the pointer is still
visually inside the parent.

To make `hover:bg-primary/80` on a `Button` stay applied while the pointer
is over the button's own label, the crate follows one rule across every
component:

- **The outermost element of a widget is the interactive unit.**
  `Button`'s `Div` and `AccordionTrigger`'s `Div` are interactive.
- **Children are non-interactive by default.** `Label` and `Icon` both
  check `style.pointer_events.unwrap_or(false)` — they don't become
  hit-test targets unless the caller opts in.
- **Children react to the parent's state via `DivActiveState` context.**
  `Div` publishes its active `StateFlags` on every render; `Label` and any
  other state-sensitive leaf read those flags and merge them into their
  own state set.
- **Opt in with `.pointer-events-auto()` when a leaf genuinely needs its
  own hover.** A standalone `Label` in a list, an `Icon` that acts as a
  button, an inline `Div` with its own disclosure — opt those in
  individually:

```rust,no_run
use freyacn::prelude::*;

Label()
    .text("Hover me")
    .style("pointer-events-auto hover:underline cursor-pointer")
```

- **Containers do not inherit.** `Div`'s `inherit_state` prop defaults to
  `false`, so a hovered row doesn't light up every button inside it.
  `Label` sets it to `true` explicitly, so a label always follows its
  parent button's hover.

### Why not `on_pointer_over` / `on_pointer_out`?

Those bubble from children — `out` fires on the parent when the pointer
moves onto its own child. `on_pointer_enter` / `on_pointer_leave` fire
exactly once per true boundary crossing, matching CSS `:hover` semantics.
`Div` and `Label` use only the latter.

---

## The styling vocabulary

`StyleExt` exposes the full Tailwind subset Freya can render. Every helper
funnels through a base setter — `background(Color)`, `color(Color)`,
`corner_radius(…)`, `padding(…)`, `width(…)`, and so on — so you can drop
to the raw form whenever you have a concrete value.

### Colours

- **Palette** — every family × shade, as `bg_*` and `text_*`:
  `slate`, `gray`, `zinc`, `neutral`, `stone`, `mauve`, `olive`, `mist`,
  `taupe`, `red`, `orange`, `amber`, `yellow`, `lime`, `green`, `emerald`,
  `teal`, `cyan`, `sky`, `blue`, `indigo`, `violet`, `purple`, `fuchsia`,
  `pink`, `rose` — shades `50, 100, 200, …, 900, 950`.
- **Semantic tokens** — `background`, `foreground`, `primary`,
  `primary_foreground`, `secondary`, `secondary_foreground`, `muted`,
  `muted_foreground`, `accent`, `accent_foreground`, `destructive`,
  `destructive_foreground`, `card`, `card_foreground`, `popover`,
  `popover_foreground`, `border`, `input`, `ring`, `chart_1` … `chart_5`,
  the `sidebar_*` family, and `disabled` / `disabled_foreground`.
- **Literal** — `text_white`, `text_black`, `bg_white`, `bg_black`,
  `bg_transparent`, `text_transparent`.

### Typography

- Sizes: `text_xs` … `text_9xl`
- Weights: `font_thin`, `font_extralight`, `font_light`, `font_normal`,
  `font_medium`, `font_semibold`, `font_bold`, `font_extrabold`,
  `font_black_weight`
- Slant: `italic`, `not_italic`
- Families: `font_sans`, `font_serif`, `font_mono`
- Alignment: `text_left`, `text_center`, `text_right`, `text_justify`
- Decoration: `underline`, `overline`, `line_through`, `no_underline`, plus
  `decoration_color`, `decoration_thickness`, `underline_offset`
- Transform: `uppercase`, `lowercase`, `capitalize`, `normal_case`
- Extras: `whitespace_*`, `break_words`, `break_all`, `break_keep`,
  `text_ellipsis_set`, `text_clip`, `line_clamp`, list style helpers,
  `accent_color`, `caret_color`, `selection_color`, `placeholder_color`,
  `line_height`, `letter_spacing`, `text_indent`

### Borders

- Width: `border`, `border_0`, `border_2`, `border_4`, `border_8`, and the
  per-side `border_t` / `border_r` / `border_b` / `border_l` / `border_x` /
  `border_y`
- Colour: `border_primary`, `border_destructive`, every palette token
- Style: `border_solid`, `border_dashed`, `border_dotted`, `border_double`,
  `border_none`
- Divide: `divide_x`, `divide_y`, `divide_color`
- Corner radius: `rounded_none`, `rounded_sm`, `rounded`, `rounded_md`,
  `rounded_lg`, `rounded_xl`, `rounded_2xl`, `rounded_3xl`, `rounded_full`

`Div` renders per-side borders by splitting them across the flex main
axis and cross axis: main-axis sides become real child `rect`s inside the
visual, cross-axis sides become siblings in an opposite-direction wrapper.
Both paths render on transparent backgrounds — no `bg-*` needed.

> **`border-*` requires both width and colour.** `border-green-500` alone
> sets only the border colour and renders nothing (same as CSS). Use
> `border-2 border-green-500` — width *and* colour — or `.border_width(2.0)
> .border_color(c)` for the method form.

### Effects

- Opacity: `opacity_0`, `opacity_25`, `opacity_50`, `opacity_75`, `opacity_100`
- Shadows: `shadow_sm`, `shadow_md`, `shadow_lg`, `shadow_xl`, `shadow_2xl`,
  `shadow_none`
- Ring (Tailwind v4): `ring`, `ring_0` … `ring_8`, `ring_primary`,
  `ring_offset`, `inset_ring`, plus palette and semantic colour variants.
  Freya 0.5.0-rc.10 does not yet render ring primitives — tokens parse but
  are no-ops at paint time.
- Outline: `outline_width`, `outline_color`, `outline_offset`, `outline_none`
- Transforms: `rotate(deg)`, `scale(s)`, `transform_origin(o)`,
  `backdrop_blur(radius)` — blurs what's behind the element, matching CSS
  `backdrop-filter: blur()`

### Animations

`animate_spin`, `animate_ping`, `animate_pulse`, `animate_bounce`, and
`animate_none` map to the corresponding Tailwind utilities. `Div` starts
the animation and drives the paint-time transform each frame.

### Flex

- Direction: `flex_row`, `flex_col`
- Wrap: `flex_wrap_content`, `flex_wrap_spacing`
- Content mode: `content_flex`, `content_fit`, `content_normal`
- Sizing: `flex_1`, `grow`, `grow_0`, `shrink`, `shrink_0`, `shrink_1`
- Justify: `justify_start`, `justify_center`, `justify_end`,
  `justify_between`, `justify_around`, `justify_evenly`
- Cross axis: `items_start`, `items_center`, `items_end`
- Self: `self_start`, `self_center`, `self_end`
- Content align: `content_start`, `content_center`, `content_end`,
  `content_between`, `content_around`, `content_evenly`
- Gaps: `gap`, `gap_x`, `gap_y`, `space_x`, `space_y`
- Order: `order`, `order_first`, `order_last`, `order_none`

### Sizing

- Raw: `w`, `h`, `min_w`, `min_h`, `max_w`, `max_h`, `size`
- Percent: `w_percent`, `h_percent`
- Keywords: `w_full`, `w_auto`, `h_full`, `h_auto`, `size_full`
- Tailwind scale: `w_0` … `w_96`, `h_0` … `h_96`, and the min/max variants
- Aspect ratio: `aspect`, `aspect_square`, `aspect_video`, `aspect_auto`

### Spacing

`p_*`, `px_*`, `py_*`, `pt_*`, `pr_*`, `pb_*`, `pl_*` and `m_*`, `mx_*`,
`my_*`, `mt_*`, `mr_*`, `mb_*`, `ml_*` on the Tailwind scale (1 unit = 4 px),
including fractional steps (`p_0_5`, `p_1_5`, `p_2_5`, `p_3_5`).

### Positioning and overflow

`relative`, `absolute`, `global`, `top`, `right`, `bottom`, `left`, `inset`,
`inset_x`, `inset_y`, `z_index`, `overflow_none`, `overflow_clip`,
`overflow_scroll` (vertical), `overflow_scroll_x`, `overflow_scroll_xy`, and
the per-axis overflow helpers.

### Layout extras

`hidden`, `visible`, `invisible`, `sr_only`, `not_sr_only`,
`pointer_events_none`, `pointer_events_auto`, `select_none`, `select_text`,
`select_all`, `select_auto`, `resize`, `resize_none`, `resize_x`, `resize_y`,
`table_auto`, `table_fixed`, `border_collapse`, `border_separate`, `columns`,
`container`.

---

## Style strings

Every `StyleExt` implementor accepts a Tailwind-style style string:

```rust,no_run
use freyacn::prelude::*;

let _ = Style::default().style("p-4 bg-primary text-white shadow-md font-bold");
```

Unknown tokens are silently ignored, matching Tailwind's behaviour. The
parser understands the full vocabulary above — palette and semantic colours,
opacity modifiers (`/80`), spacing, sizing (including viewport `w-screen`),
typography, borders, rings (`ring`, `ring-offset-*`, `inset-ring-*`),
arbitrary bracket values (`text-[12.8px]`, `bg-[#ff0000]`), transforms (`rotate-*`, `scale-*`, `blur-*`), effects, flex,
positioning, overflow,
aspect ratio, order, space, divide, outline, list, accent/caret, animations (`animate-spin`, `animate-pulse`,
`animate-bounce`, `animate-ping`), and
state-conditional modifiers (`hover:`, `focus:`, `focus-visible:`,
`active:`, `disabled:`, `dark:`, `first:`, `last:`, `odd:`, `even:`,
`only:`, `not-hover:`, `group-hover:`, `has-hover:`).

Negative forms (`-m-4`, `-mt-2`, `-space-x-2`, `-translate-y-1`, `-top-4`,
`-rotate-45`, `-scale-90`) are supported wherever they make sense. `-*`
prefixes resolve to negative lengths before hitting the corresponding
positive helper.

For typo hunting, `style_debug` reports unrecognised tokens:

```rust,no_run
use freyacn::prelude::*;

let (style, unknown) = Style::default()
    .style_debug("p-4 bg-primary shdaow-md not-a-thing");

assert_eq!(unknown, vec!["shdaow-md", "not-a-thing"]);
```

### What is not representable in Freya today

CSS Grid layout, element-self blur (`filter: blur()`, as opposed to
`backdrop-filter: blur()` which works), and responsive viewport-width
prefixes (`sm:`, `md:`, `lg:`). For viewport responsiveness, Freya's
`use_viewport()` hook can be used directly in Rust code.

`Div` handles scroll (`overflow_scroll`), fixed positioning (`global`),
per-side borders, and pointer/focus/hover state natively.

---

## Merging

`Style` has explicit merging primitives used by the component macro to
combine variant styles, size styles, and user overrides.

```rust,no_run
use freyacn::prelude::*;

let base    = Style::default().p_4().bg_primary();
let variant = Style::default().p_6().bg_destructive();
let merged  = base.merge(variant);
// merged.padding    == Some(6·unit)  — variant won
// merged.background == destructive   — variant won
```

| Method                                                | Effect                                                        |
|-------------------------------------------------------|---------------------------------------------------------------|
| `Style::merge(self, other) -> Style`                  | `other` wins on every field it set. Stack fields concatenate. |
| `Style::or(self, defaults) -> Style`                  | `self` wins; `defaults` fills gaps.                           |
| `Style::merge_from(&mut self, other)` / `or_from`     | In-place variants.                                            |
| `Style::is_empty(&self) -> bool`                      | `true` iff the style equals the default.                      |
| `Style::with_active_states(self, flags) -> Style`     | Fold state-conditional styles matching `flags`.               |
| `StyleExt::merge_style(other)` / `or_style(defaults)` | Lift the above onto any component.                            |

### Padding and margin merge per side

`padding` and `margin` are `Gaps` values, which merge field-by-field rather
than wholesale. That means a variant that sets only `px-4` overrides the
horizontal sides and leaves the base class's `py-2` intact. The same logic
applies across `.merge(...)` and `.merge_style(...)`.

---

## Variants

`#[variant]` turns an enum into a style-resolving type.

```rust
use freyacn::prelude::*;

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[variant("bg-primary text-primary-foreground")]
    #[default]
    Default,

    #[variant("bg-destructive text-destructive-foreground")]
    Destructive,

    #[variant("border-2 border-primary bg-transparent text-primary")]
    Outline,
}
```

The macro generates:

- `impl MyEnum { fn style_str(&self) -> &'static str; }`
- `impl MyEnum { fn resolve(&self) -> freyacn::Style; }`
- `impl freyacn::Variant for MyEnum`
- `impl From<&str>` and `impl From<String>`, matching kebab-cased variant
  names with a fallback to the first declared variant.

Kebab conversion handles simple (`Md` → `md`), acronym (`XS` → `xs`), and
compound (`SecondaryDestructive` → `secondary-destructive`) names.

The `StyleVariant` trait itself is one method:

```rust
use freyacn::prelude::Style;

pub trait Variant {
    fn resolve(&self) -> Style;
}
```

`.resolve()` reads the active theme, so it must be called inside a Freya
render context. When a variant is used as a prop inside a component that
declares `variants(...)` and `ext(style)`, the component macro calls
`.resolve()` for you and merges the result into the render body's `style`
local.

---

## Extensions

Extensions opt a struct into freyacn's built-in bundles. Each name maps to a
field, an initialiser, a render-body local, and a trait impl.

| Name             | Trait                                | Field(s) added                                           | Trait method             |
|------------------|--------------------------------------|----------------------------------------------------------|--------------------------|
| `children`       | `freyacn::prelude::ChildrenExt`      | `children: Vec<Element>`                                 | `get_children`           |
| `key`            | `freyacn::prelude::KeyExt`           | `key: DiffKey`                                           | `write_key`              |
| `style`          | `freyacn::prelude::StyleExt`         | `style: Style`                                           | `get_style`              |
| `event_handlers` | `freyacn::prelude::EventHandlersExt` | `event_handlers: FxHashMap<EventName, EventHandlerType>` | `get_event_handlers`     |
| `accessibility`  | `freyacn::prelude::AccessibilityExt` | `accessibility: AccessibilityData`                       | `get_accessibility_data` |

Used inside `#[component]` via the `ext(...)` group, or standalone:

```rust
use freyacn::prelude::*;

#[extensions(children, key, style)]
#[derive(Clone, PartialEq, Default)]
pub struct PlainBox {
    // Custom user fields
}
```

Declaring `variants(...)` inside `#[component]` requires `ext(style)` so that
variant styles can be folded into the effective style.

### Freya 0.5 notes

`event_handlers` used to be `with_event_handlers`. The trait method is
`event_handlers` in 0.5, and every component in the crate is on the new
name. If you're porting a `#[extensions(event_handlers)]` struct from a
0.4-based project, update the call site accordingly.

`accessibility` is unchanged. `StyleExt` and `ChildrenExt` are unchanged.

---

## The macros

`freyacn` re-exports four proc-macros from `freyacn-macros`:

- **`#[component(...)]`** — turns a function into a component struct with
  props, extensions, and variants.
- **`#[variant]`** — turns an enum into a style-resolving variant type.
- **`#[struct_fields(...)]`** — adds internal storage fields (no setters,
  separate docs).
- **`#[extensions(...)]`** — standalone form of `ext(...)`.

The full grammar, field sources, collision rules, generated code, and every
compile error are documented in the [`freyacn-macros`](../freyacn-macros)
README.

### The `#[component]` groups at a glance

```rust,no_run
# use freyacn::prelude::*;
# #[variant]
# #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
# pub enum ButtonVariant { #[default] #[variant("")] Default }
# #[derive(Clone, Default, PartialEq)]
# pub struct MyTooltip { pub text: String }
#[component(
    props(    subtitle: String, badge: String = "new".to_string()),
    ext(children, style),
    variants( variant: ButtonVariant = ButtonVariant::Default),
)]
#[struct_fields( tooltip: MyTooltip, hover_count: usize = 0)]
#[derive(Clone, PartialEq)]
fn Card(name: String) {
    // fn params  → `name: &String`   (required, panics if unset)
    // props      → `subtitle: &Option<String>`, `badge: &String`
    // struct     → `tooltip: &Option<MyTooltip>`, `hover_count: &usize`
    // variants   → `variant: &ButtonVariant`
    // extensions → `children: &Vec<Element>`, `style: &Style`

    Div()
        .merge_style(style.clone())
        .children(children.clone())
        .child(Label().text(name.clone()))
}
```

The generated `CardComponent` has setters for props (`subtitle`, `badge`,
`variant`), a `new()`, a `Default` impl, a free function `Card()`, and a
companion macro `Card!(...)`.

---

## Components

`freyacn` ships the core primitives built with the macro engine and
Tailwind v4 styling.

### `Div`

The foundational, state-aware layout primitive. Wraps Freya's `rect()`,
forwards every `Style` field Freya can render, and owns all runtime state
observation — consumers never compute `StateFlags` themselves.

- **State observation**: `DARK`, `HOVER`, `ACTIVE`, `FOCUS`,
  `FOCUS_VISIBLE`, `DISABLED`, plus structural `FIRST` / `LAST` /
  `ODD` / `EVEN` / `ONLY` and `GROUP_HOVER` / `GROUP_FOCUS` from an
  ancestor `Group`.
- **Full style surface**: background, foreground, typography, font width,
  text height, text shadows, corner radius, borders (uniform and
  per-side), padding, margin, size constraints, position, overflow, flex
  layout, opacity, shadows, z-index via layer, text overflow, pointer
  events, interactive, rotate/scale/backdrop-blur/transform-origin, child
  offsets, animations, and accessibility.
- **Props**: `disabled: bool`, `focusable: bool`,
  `inherit_state: bool = false`, `custom_a11y_id`, `a11y_role`,
  `a11y_expanded`, `a11y_labelled_by`, `a11y_hidden`.
- **Interaction callbacks**: `on_hover: EventHandler<Event<PointerEventData>>`,
  `on_press: EventHandler<Event<PressEventData>>`.

```rust,no_run
use freyacn::prelude::*;

Div()
    .style("p-4 bg-card text-card-foreground rounded-lg border shadow-sm")
    .focusable(true)
    .on_hover(|_| println!("hovered"))
    .on_press(|_| println!("pressed"))
    .child(Label().text("Card content"))
```

Because `Div` owns state folding, any component that returns a `Div` gets
`hover:` / `focus:` / `active:` / `disabled:` support for free. Just hand
it the merged style and let it observe.

```rust,no_run
# use freyacn::prelude::*;
# #[component(
#     props(disabled: bool = false),
#     ext(style, children, key, event_handlers, accessibility),
# )]
# fn MyButton() {
Div()
    .merge_style(style.clone())
    .disabled(*disabled)
    .focusable(true)
    .accessibility(accessibility.clone())
    .event_handlers(event_handlers.clone())
    .children(children.clone())
# }
```

`Div` publishes its active `StateFlags` to descendants via the
`DivActiveState` context. State-sensitive children (`Label`, and any
component that reads `try_consume_context::<DivActiveState>()`) use it to
inherit the parent's hover/focus/active/disabled flags. `inherit_state`
controls whether `Div` itself inherits its own parent's flags; the default
is `false` so a hovered row doesn't light up every child.

`Div` also applies `Style::space_x` / `Style::space_y` as `margin-left` /
`margin-top` on every child except the first — matching Tailwind's
`space-x-*` / `space-y-*` semantics. Negative values (`-space-x-2`)
produce overlap.

### `Button`

The primary interactive component, composited on top of `Div`.

- **6 variants**: `Default`, `Outline`, `Secondary`, `Ghost`, `Destructive`,
  `Link`. Accepts string coercion: `.variant("outline")`.
- **8 sizes**: `Default` (h-8), `Xs` (h-6), `Sm` (h-7), `Lg` (h-9), `Icon`
  (size-8), `IconXs`, `IconSm`, `IconLg`.
- **Props**: `disabled: bool`, `focusable: bool`,
  `on_hover: Option<EventHandler<Event<PointerEventData>>>`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

`Button` forwards the caller's `accessibility` extension and sets
`Role::Button` on the underlying `Div` by default, so screen readers
announce it as a button. Add a label via
`Button().accessibility(AccessibilityData::new().alt("..."))`.

`Button` is a pure compositor: it builds a `Style` from base classes plus
the variant and size resolutions and hands it to `Div` via
`.merge_style(...)`. All state observation is `Div`'s job. Children of the
button (`Label`, `Icon`) are non-interactive by default, so the button's
`hover:` / `active:` styles stay applied when the pointer is over its own
label or icon.

### `Label`

A text primitive. Renders a `rect()` wrapper around Freya's `label()`,
applies text styles to the label directly, and box styles to the wrapper.
Owns its own hover / press / focus tracking, and inherits the parent
`Div`'s active flags via `DivActiveState` (with `inherit_state = true`
by default).

- **Props**: `text: String`, `disabled: bool`, `focusable: bool`,
  `inherit_state: bool = true`, `custom_a11y_id`, `a11y_role`,
  `a11y_labelled_by`, `a11y_hidden`.
- **Text styles**: colour, size, weight, slant, width, families, align,
  decoration, height, shadow, truncation / line-clamp, line-height, case
  transform.
- **Box styles**: sizes (including `min_*` / `max_*` for truncation),
  opacity, rotate / scale / backdrop-blur, positioning, events, a11y.
- **Extensions**: `style`, `event_handlers`, `accessibility`.

Non-interactive by default — a `Label` inside a `Button` or
`AccordionTrigger` reacts to the parent's hover without stealing it. Opt
in with `.pointer-events-auto()` for a standalone hoverable label.

### `Icon`

A current-colour SVG. Wraps Freya's `SvgViewer` in a `Div`. Defaults to
16×16 when no size is set; mirrors the other axis when only one is set.

- **Props**: `svg: Bytes`.
- **Extensions**: `style`, `accessibility`, `key`.

Non-interactive by default — an `Icon` inside a `Button` or
`AccordionTrigger` reacts to the parent's hover without stealing it. The
SVG uses `currentColor`, so it picks up the parent's text colour through
Freya's cascade. Opt in with `.pointer-events-auto()` for a standalone
hoverable icon.

```rust,no_run
use freyacn::prelude::*;

Button()
    .child(Icon().svg(chevron_down()).size_4())
    .child(Label().text("Open"))
```

### `Image`

A styled image primitive that mimics HTML's `<img>` and wraps Freya's
native `image()`. Reads from a file path, caches the decoded handle
keyed on `src`, and falls back gracefully when the file is missing.

- **Props**: `src: String`, `bytes: Option<Bytes>`, `alt: String`,
  `fit: ImageFit`, `sampling: SamplingMode`, `snap_to_grid: bool`,
  `blur: f32`.
- **Extensions**: `style`, `accessibility`, `event_handlers`, `key`.
- **Supported formats** (via `skia_safe::Image::from_encoded`): PNG,
  JPEG, WebP, GIF, BMP, ICO, WBMP. AVIF, HEIF, and TIFF when Skia is
  built with the corresponding codecs. SVG is not supported — rasterise
  first.
- **`object-fit` mapping**:

  | `ImageFit` | Freya `AspectRatio` | Freya `ImageCover` |
            |------------|---------------------|--------------------|
  | `Fill`     | `None` (stretch)    | `Fill`             |
  | `Cover`    | `Max` (cover, crop) | `Center`           |
  | `Contain`  | `Min` (fit inside)  | `Fill`             |
  | `None`     | `Fit` (natural)     | `Fill`             |

- **Style-string support**: `object-fill` / `object-cover` /
  `object-contain` / `object-none` / `object-scale-down` map to `ImageFit`
  and take precedence over the `fit` prop. `blur-*` tokens set
  `ImageData::blur` and take precedence over the `blur` prop.
  `rounded-*`, `shadow-*`, `border-*`, sizing, opacity, transforms,
  position, and z-index all forward to the underlying image element.

```rust,no_run
use freyacn::prelude::*;

Image()
    .src("assets/photo.webp")
    .alt("Ada Lovelace")
    .style("w-64 h-48 object-cover rounded-lg shadow-md blur-sm")
```

`bytes` bypasses the filesystem read but still decodes. If both `src`
and `bytes` are set, `bytes` wins. Missing or corrupt sources render a
muted placeholder sized to the caller's constraints.

### `Avatar`

A user/entity avatar built on top of `Image`, with an `AvatarFallback`
slot, size/shape variants, and optional badge support.

- **Props**: `src: String`, `bytes: Option<Bytes>`, `alt: String`.
- **Variants**: `size: AvatarSize`, `shape: AvatarShape`.
- **Struct fields**: `badge: Element` — set via `.badge(...)`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

`Avatar` renders its inner content in priority order:

1. **Image** — if `bytes` is set or `src` names a non-empty file.
2. **Children** — if any are present (typically an `AvatarFallback`).
3. **Empty muted box** — otherwise.

The image-existence check is cached on `src` via `use_state`, so the
filesystem is only touched when `src` actually changes.

- **`AvatarSize`**: `Xs` (24), `Md` (32, default), `Lg` (40), `Xl` (48),
  `Xxl` (64). Fallback text size scales with the variant.
- **`AvatarShape`**: `Circle` (`rounded-full`, default), `Rounded`
  (`rounded-lg`), `Square` (`rounded-none`).
- **`.fallback("AL")`** — shortcut for
  `.child(AvatarFallback().child(Label().text("AL")))`.
- **`.badge(...)`** — attaches an `AvatarBadge`; the avatar renders in a
  positioning wrapper sized to the visible avatar so the badge lands at
  the visible bottom-right corner.
- **`.bordered()`** — explicit ring on a standalone avatar. Inside an
  `AvatarGroup`, the ring comes from the group's padded wrapper.

```rust,no_run
use freyacn::prelude::*;

Avatar()
    .src("assets/ada.jpg")
    .alt("Ada Lovelace")
    .fallback("AL")
    .size(AvatarSize::Lg)
    .badge(AvatarBadge().style("bg-green-500"))
```

### `AvatarFallback`

The fallback slot for `Avatar`. Renders a muted, centered box that fills
the avatar. Use directly for full control over content, or via
`.fallback("AL")` on `Avatar` for the shortcut.

- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

### `AvatarGroup`

A horizontal row of overlapping avatars.

- **Props**: `spacing: f32 = -8.0`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

Each child is wrapped in a `Div` with 2px padding filled with
`theme.card` — that padded ring is the visual separator that makes
overlapping avatars readable. Non-first children also get
`.ml(spacing)` for overlap and `.z_index(i)` for stacking.

```rust,no_run
use freyacn::prelude::*;

AvatarGroup()
    .child(Avatar().src("a.jpg").alt("A").fallback("A").size(AvatarSize::Lg))
    .child(Avatar().src("b.jpg").alt("B").fallback("B").size(AvatarSize::Lg))
    .child(AvatarGroupCount().size(AvatarSize::Lg).count(5))
```

### `AvatarGroupCount`

The "+N more" slot in an `AvatarGroup`. Styled like an avatar (muted
background, centered, rounded) but meant to hold a short count string.

- **Variants**: `size: AvatarSize`, `shape: AvatarShape`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.
- **`.count(n)`** — shortcut for `.child(Label().text(format!("+{n}")))`.

### `AvatarBadge`

A small status indicator — the colored dot in the corner of an avatar.

- **Variants**: `size: AvatarBadgeSize` (`Sm` 8px, `Md` 10px default,
  `Lg` 12px, `Xl` 16px).
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

Positions absolutely at the bottom-right of its positioning context.
`Avatar` supplies that context when `.badge(...)` is used, so the badge
is a sibling of the clipped avatar image and not cut by its
`overflow-clip`. Default color is `bg-primary`; override with
`.style("bg-green-500")` or any bg token.

### `Divider`

A 1px separating line. Horizontal by default (spans the parent's width);
vertical (`DividerOrientation::Vertical`) spans the parent's height. Both
are `shrink-0` so flex siblings can't squeeze them.

Supports an optional centred label:

```rust,no_run
use freyacn::prelude::*;

Divider().label("OR");   // ───────────── OR ─────────────
```

- **Props**: `label: String`, `orientation: DividerOrientation`.
- **Extensions**: `style`, `accessibility`, `key`.

### `Group`

Wraps a subtree and provides its hover / focus state to descendants via
`GroupState`. Use it to enable `group-hover:` / `has-hover:` on any
descendant. The row tracks hover with `on_pointer_enter` /
`on_pointer_leave`; descendants read the flags from context.

- **Props**: none.
- **Extensions**: `style`, `children`, `accessibility`.

```rust,no_run
use freyacn::prelude::*;

Group()
    .style("flex flex-row items-center gap-3 p-4 rounded-lg border cursor-pointer")
    .child(Icon().svg(bell()).size_4())
    .child(Label().text("Hover this row").style("group-hover:underline group-hover:text-primary"))
```

### `Accordion`

A shadcn-shaped accordion. Root / Item / Trigger / Content composition,
single-open and multiple-open modes, per-item disabled, and animated
open/close on both click and single-mode auto-close.

- **Root props**: `type_: AccordionType`, `collapsible: bool`,
  `disabled: bool`, `default_value: Option<Vec<String>>`.
- **Item props**: `value: String`, `disabled: bool`.
- **Extensions**: `style`, `children`, `key`.

```rust,no_run
use freyacn::prelude::*;

Accordion()
    .type_(AccordionType::Single)
    .collapsible(true)
    .item("item-1", "Is it accessible?", "Yes — role=Button triggers.")
    .item("item-2", "Is it animated?", "Yes — linear height and chevron.")
```

Or via the compound API when you need custom trigger children:

```rust,no_run
use freyacn::prelude::*;

Accordion()
    .type_(AccordionType::Single)
    .child(
        AccordionItem()
            .value("item-1")
            .child(
                AccordionTrigger()
                    .style("flex-row items-center gap-3")
                    .child(Icon().svg(bell()).size_4())
                    .child(Label().text("Notifications").style("flex-1 hover:underline")),
            )
            .child(AccordionContent().child(Label().text("Body"))),
    )
```

The trigger `Div` is the interactive unit; its `Label` and
`AnimatedChevron` are non-interactive, so the trigger keeps its `hover:`
state across the whole row. `hover:underline` on the title label with
`flex-1` underlines the title whenever the pointer is anywhere on the row.

Height and chevron animations are linear and configurable via constants
at the top of `accordion.rs`.

### `cn!` macro

Utility macro for Tailwind-style class concatenation. Skips empty strings
and `None` options:

```rust
use freyacn::prelude::*;

let is_active = true;
let classes = cn!(
    "px-4 py-2 rounded",
    "bg-primary text-primary-foreground",
    if is_active { "ring-2 ring-ring" } else { "" },
    Some("shadow-md"),
    None::<&str>,
);
assert_eq!(
    classes,
    "px-4 py-2 rounded bg-primary text-primary-foreground ring-2 ring-ring shadow-md",
);
```

---

## Style layering at render time

When a component declares `variants(...)` alongside `ext(style)`, the
generated `render` body computes the effective style before user code runs:

```text
Style::default()
    .merge(variant.resolve())      // declared order
    .merge(size.resolve())
    .merge(self.style.clone())     // user overrides win
```

The render body then binds `style: &Style` to the merged value.

**Consequences:**

- **Overrides persist across variant changes.** `.variant("outline")`
  recomputes the base layer but leaves user intent intact. If a caller
  wrote `.bg_green_50()`, that override still wins.
- **Theme changes propagate automatically.** `.resolve()` reads
  `use_cn_theme()` on every render, not at construction. A dark-mode
  toggle re-renders the whole tree with the new theme.
- **State styles fold last.** Any pending `hover:` / `focus:` entries in
  the merged style are folded by `Div` at render time using runtime
  state.

---

## Theming

`Theme` holds every semantic colour plus a palette containing all Tailwind
colours. It's `Copy`, so passing it around is free.

```rust,no_run
use freyacn::prelude::*;

// Fluent entry points:
let theme = Theme::light().slate();           // light, slate neutral base
let theme = Theme::dark().stone().theme_color("rose");
let theme = Theme::dark().zinc();             // base default accent
let theme = Theme::from_base("zinc", true).theme_color("blue");

use_init_cn_theme(theme);
```

`.base_color(...)` resets `primary` / `accent` / `ring` to the new base's
defaults, so `.theme_color(...)` must come after it. Per-palette shortcuts (`.stone()`, `.mauve()`, `.olive()`,
`.mist()`, `.taupe()`, `.slate()`,
`.gray()`, `.zinc()`, `.neutral()`) skip the string.

Reading the theme from a component:

```rust,no_run
use freyacn::prelude::*;

fn MyComponent() -> impl IntoElement {
    // Bind the state first: `State::read()` returns a guard whose
    // lifetime is tied to `&self`, so the temporary can't be dropped
    // at the semicolon.
    let state = use_cn_theme();
    let theme = *state.read();

    // Theme carries is_dark plus every semantic token.
    let bg      = theme.background;
    let fg      = theme.foreground;
    let primary = theme.primary;

    Div().background(bg).color(fg).style("p-4")
}
```

`use_init_cn_theme` reuses an existing context if one exists. For a scoped
override that always creates a fresh context, use `use_provide_cn_theme`.
`get_cn_theme_or_default()` returns a `Readable<Theme>` and never panics —
useful for built-in components that should work without an explicit provider.

Every palette and semantic helper reads the theme at call time, so switching
light/dark or changing the base colour updates every component on the next
render without reconstruction.

---

## Compile-time safety

The `#[component]` macro refuses to compile on:

- Duplicate field names across `props`, `struct_fields`, `variants`, and
  function parameters.
- A field whose name matches an extension field from `ext(...)`.
- A variant prop without a default.
- `variants(...)` without `ext(style)`.
- The removed `required` keyword or a missing type on an attribute prop.
- An unknown group (only `props`, `ext`, `variants` are valid).
- A misplaced `#[struct_fields(...)]` (it must come after `#[component]`).

Each error names the offending item and points at the fix.

---

## Workspace

`freyacn` is part of a Cargo workspace:

| Crate                      | Purpose                                                                                                                                                                                            |
|----------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`freyacn`** (this crate) | Runtime library: `Style`, `StyleExt`, `StyleVariant`, theme, extensions, components. Re-exports all macros and Freya's prelude.                                                                    |
| `freyacn-macros`           | Proc-macros (`#[component]`, `#[variant]`, `#[extensions]`, `#[struct_fields]`). Internal — never depend on it directly.                                                                           |
| `freyacn-cli`              | Command-line tool for bundling and packaging `freyacn` apps. Its command surface mirrors the Tauri CLI — `dev`, `build`, `bundle`, `icon`, `info`, and friends — so familiar workflows carry over. |

### Examples

Every component has a runnable showcase under `examples/`:

```bash
cargo run --example div
cargo run --example button
cargo run --example icon
cargo run --example label
cargo run --example divider
cargo run --example image
cargo run --example avatar
cargo run --example accordion
```

Each showcase follows the same skeleton — `use_init_cn_theme` at the root,
an outer `h-screen overflow-scroll` `Div`, and a `max-w-2xl mx-auto` column
of `section(...)` cards — so it's easy to copy when writing a new one.

---

## Roadmap

- More components: `Card`, `Input`, `Select`, `Checkbox`, `Switch`,
  `Dialog`, `Tabs`, `Tooltip`, `Popover`, `Dropdown`.
- `freyacn-cli` — an initial `dev` / `build` / `bundle` flow, mirroring the
  Tauri CLI's ergonomics.
- Publish to crates.io.
- Theme overrides at the component level.
- Additional icon sets.
- `resolve_with(&Theme)` for theme-explicit variant resolution outside a
  render context.
- First-class support for the style-surface fields Freya doesn't yet render (rings, outlines, per-axis overflow) as soon
  as the underlying primitives
  land.
- Element-self blur (`filter: blur()`) — currently only `backdrop-filter:
  blur()` is renderable.

---

## License

MIT.

---

## Acknowledgements

- [Freya](https://github.com/marc2332/freya) — GUI framework.
- [shadcn/ui](https://ui.shadcn.com) — design inspiration.
- [Tailwind CSS](https://tailwindcss.com) — utility-first philosophy.