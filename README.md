# FreyaCN

> ⚠️ **Work in progress.** FreyaCN is under active development and its public
> API is **not yet stable**. Breaking changes may land in any release.
>
> **Status:** pre-alpha. Expect rough edges, missing features, and occasional
> doc/code drift. If you want to try it, use a `git` dependency pinned to a
> specific commit. If you find a bug or want to discuss the design, please
> open an issue.

A modern, shadcn/ui-inspired component library for
[Freya](https://github.com/marc2332/freya), built with Rust, Tailwind-style
extension traits, and a proc-macro layer that turns plain functions and enums
into theme-aware, prop-driven components.

**Targets Freya `0.5.0-rc.10`.**

> **Complete API reference:** [`API.md`](./API.md) — every component, every
> `StyleExt` method, every macro, and the full theme system, with examples.

---

## Overview

**FreyaCN** brings the elegance and consistency of
[shadcn/ui](https://ui.shadcn.com) to Freya, a Rust-based GUI framework. It
provides theme-aware, highly composable components with a Tailwind-inspired
API, plus the macro machinery to author new components in plain Rust.

The library is built around five core ideas:

1. **Theme-first** — every component reads the active theme via
   `use_cn_theme()`. Switching light/dark or changing the base colour updates
   every component automatically, without a recompile.
2. **Tailwind-inspired extensions** — one trait (`StyleExt`) exposes the
   entire styling vocabulary. Familiar utility names (`p_4()`, `bg_primary()`,
   `flex_col()`, `text_xl()`, …) map directly to Freya's renderer.
3. **Macro-driven authoring** — `#[component]` turns a plain function into a
   fully-typed component struct with props, extensions, and variants.
   `#[variant]` turns an enum into a style-resolving type. No boilerplate, no
   manual prop plumbing.
4. **shadcn/ui semantics** — component variants, sizes, and tokens follow the
   design language of shadcn/ui, making them immediately recognisable.
5. **Single interactive unit per widget** — one element per component owns
   pointer/focus/disabled state and the rest of the tree reacts to it
   through context. This matches CSS `:hover` semantics and is the only
   pattern that keeps `hover:` / `active:` styles stable as the pointer
   moves across a widget's own children.

---

## Workspace layout

The project is the **`freyacn`** Cargo workspace. Every crate
lives in this repository.

| Crate            | Path                     | Purpose                                                                               |
|------------------|--------------------------|---------------------------------------------------------------------------------------|
| `freyacn`        | `crates/freyacn/`        | Runtime library: `Style`, `StyleExt`, `Variant`, theme, components, extension traits. |
| `freyacn-macros` | `crates/freyacn-macros/` | Proc-macros: `#[component]`, `#[variant]`, `#[extensions]`, `#[struct_fields]`.       |
| `freyacn-cli`    | `crates/freyacn-cli/`    | Command-line tool for scaffolding, developing, building, and bundling FreyaCN apps.   |

`freyacn` re-exports every macro from `freyacn-macros`, so users depend on
`freyacn` only. `freyacn-macros` exists as a separate crate because proc-macro
code must compile in a distinct target, but it is **not** meant to be used
directly.

---

## Features

- **Full theming** — light/dark modes, custom base palettes (slate, stone,
  neutral, zinc, gray, …), and shadcn-style `base_color` / `theme_color`
  configuration through a fluent `Theme::light()` / `Theme::dark()` builder.
- **One styling trait** — `StyleExt` provides hundreds of provided methods
  across background, foreground, typography, borders, effects, flexbox,
  sizing, spacing, positioning, and more. Every helper is chainable.
- **Macro-driven components** — `#[component]` handles props, extensions, and
  variants in one attribute. Each field source has a distinct contract, and
  the generated struct carries full rustdoc.
- **Variant system** — `#[variant]` turns an enum into a `Style` resolver with
  string parsing (`From<&str>` / `From<String>`). Component variants merge
  into the effective style at render time.
- **Layered style resolution** — variant styles, size styles, and user
  overrides compose with deterministic `merge` / `or` semantics. Overrides
  persist across variant changes; theme updates propagate without
  reconstruction.
- **State-aware `Div`** — the foundational primitive observes pointer, focus,
  disabled, and dark-mode state, then folds `hover:` / `focus:` / `active:` /
  `disabled:` / `dark:` layers automatically. Any component that returns a
  `Div` inherits this for free. Structural selectors (`first:`, `last:`,
  `odd:`, `even:`, `only:`), `not-hover:`, and `group-hover:` all fold
  through the same pipeline.
- **Thirteen core components** — `Div`, `Button`, `Label`, `Icon`, `Divider`,
  `Group`, `Accordion`, `Image`, `Avatar`, `AvatarFallback`, `AvatarGroup`,
  `AvatarGroupCount`, `AvatarBadge`. Each ships with a runnable example.
- **Interactive-unit pattern** — children of an interactive widget are
  non-interactive by default and react to the parent's state via
  `DivActiveState` context. `Label` and `Icon` inherit hover automatically;
  opt in to self-hover per leaf with `.pointer-events-auto()`.
- **style-string API** — `style("p-4 bg-primary hover:shadow-md")` parses
  Tailwind-style tokens. `style_debug` reports unrecognised tokens for typo
  hunting. Negative forms (`-m-4`, `-space-x-2`, `-translate-y-1`, `-top-4`)
  are supported.
- **Animations** — `animate-spin`, `animate-ping`, `animate-pulse`,
  `animate-bounce`, `animate-none` are parsed end-to-end and driven by
  Freya's `use_animation`.
- **Accessibility** — focus, keyboard navigation, and ARIA via Freya's
  accessibility traits, wired through the `accessibility` extension.
  `Button` sets `Role::Button` by default; `AccordionTrigger` sets
  `Role::Button` + `aria-expanded`; `Label` and content regions link
  trigger-to-panel ids.
- **Composable** — every component implements `ChildrenExt`, so nesting is
  natural.
- **Performant** — leverages Freya's diffing and layout engine; `Div` only
  subscribes to its own animation tick, so unrelated `Div`s don't re-render
  during an animation.

---

## Installation

```toml
[dependencies]
freyacn = "0.2.0-alpha.1"
```

> **Not published to crates.io yet.** Use a git dependency pinned to a
> specific commit for reproducible builds.

`freyacn` re-exports Freya's own prelude through `freyacn::prelude`, so you
don't need a direct `freya` dependency.

In `main.rs`:

```rust,no_run
use freyacn::prelude::*;

fn app() -> impl IntoElement {
    let theme = Theme::light().slate();
    use_init_cn_theme(theme);

    Button()
        .child(Label().text("Click me"))
        .variant(ButtonVariant::Default)
        .size(ButtonSize::Default)
}

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)));
}
```

---

## Quick start

```rust,no_run
use freyacn::prelude::*;

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[variant("bg-primary text-primary-foreground px-4 py-2 rounded-lg")]
    #[default]
    Default,

    #[variant("bg-destructive text-destructive-foreground px-4 py-2 rounded-lg")]
    Destructive,

    #[variant("border-2 border-primary bg-transparent text-primary px-4 py-2 rounded-lg")]
    Outline,
}

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[variant("text-xs px-2 py-1")]
    Sm,
    #[variant("text-sm px-4 py-2")]
    #[default]
    Default,
    #[variant("text-base px-6 py-3")]
    Lg,
}

#[component(
    props(tooltip: String),
    ext(children, style),
    variants(
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Default,
    ),
)]
#[derive(Clone, PartialEq)]
fn CustomButton(text: String) {
    Div()
        .merge_style(style.clone())
        .children(children.clone())
        .child(Label().text(text.clone()))
}
```

Three equivalent ways to construct it:

```rust,no_run
# use freyacn::prelude::*;
# #[variant] #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum ButtonVariant { #[default] #[variant("")] Default, #[variant("")] Destructive }
# #[variant] #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum ButtonSize { #[default] #[variant("")] Default }
# #[component(ext(style), variants(variant: ButtonVariant = ButtonVariant::Default, size: ButtonSize = ButtonSize::Default))]
# #[derive(Clone, PartialEq)] fn CustomButton(text: String) { Div() }
CustomButton().text("Save").variant(ButtonVariant::Destructive);   // typed
CustomButton().text("Save").variant("destructive");                // &str
CustomButton!(text = "Save", variant = "destructive");             // macro
```

---

## The `#[component(...)]` macro

Every component is written as a plain Rust function. The attribute declares
the component's public API, internal storage, and variants.

### Groups

```text
#[component(
    props(    prop1: Type1, prop2: Type2 = default_expr, … ),
    ext(      extension1, extension2, … ),
    variants( variant: EnumType = EnumType::Default, … ),
)]
```

Each group is optional and may appear in any order.

### Four field sources

| Source                        | Kind      | Field type         | Initial value  | Body sees                   | Setter accepts              |
|-------------------------------|-----------|--------------------|----------------|-----------------------------|-----------------------------|
| `fn Card(name: String)`       | required  | `Option<String>`   | `None`         | `&String` (panics if unset) | `impl Into<String>`         |
| `props(name: String)`         | optional  | `Option<String>`   | `None`         | `&Option<String>`           | `impl Into<Option<String>>` |
| `props(name: String = "x")`   | defaulted | `String`           | `"x"`          | `&String`                   | `impl Into<String>`         |
| `variants(name: T = default)` | variant   | `T`                | `default`      | `&T`                        | `impl Into<T>`              |
| `#[struct_fields(name: T)]`   | internal  | `T` or `Option<T>` | `None` / —     | `&T` / `&Option<T>`         | *(no setter)*               |
| `ext(...)`                    | extension | trait-specific     | trait-specific | trait-specific              | *(no setter)*               |

### Required props — function parameters

```rust
#[component]
fn Greeting(name: String) {
    Label().text(format!("Hello, {name}!"))
}

Greeting().name("Ada");           // OK
// Greeting();                    // compiles, panics at render
// Greeting().name(None);         // type error — intentional
```

### Optional props — `props(name: T)`

Never write `Option<T>` yourself; declaring `name: String` in `props(...)` is
enough, and the macro wraps it.

```rust
#[component(props(subtitle: String))]
fn Heading(subtitle: String) {
    match subtitle {
        Some(s) => Label().text(s.as_str()),
        None => Div(),
    }
}
```

The setter takes `impl Into<Option<T>>`, which std's blanket
`From<T> for Option<T>` makes ergonomic — bare `T`, `Some(T)`, and `None` all
work.

### Defaulted props — `props(name: T = expr)`

```rust
#[component(props(
    size:   f32    = 16.0,
    weight: String = "normal".to_string(),
))]
fn Text(size: f32, weight: String) {
    Label().text(format!("{}pt {}", size, weight))
}
```

### Variant props — `variants(name: T = default)`

A default is mandatory — variant fields are stored as `T`, not `Option<T>`.
Pairs with `#[variant]`-annotated enums.

```rust
#[component(
    variants(variant: ButtonVariant = ButtonVariant::Default),
    ext(style),
)]
fn Button() {
    Div().merge_style(style.clone())
}

b.variant(ButtonVariant::Destructive);   // typed
b.variant("destructive");                // &str
b.variant(String::from("destructive"));  // String
```

> **`variants(...)` requires `ext(style)`.** The merged style is bound as a
> local called `style` in the render body, which needs the `style` extension.
> Missing `ext(style)` is a targeted compile error.

### Struct fields — `#[struct_fields(...)]`

Declared **below** `#[component]`. Same grammar as `props(...)`, but no
setter, separate documentation, and the fields exist purely so user-written
trait impls have somewhere to store data.

```rust
#[component(props(subtitle: String))]
#[struct_fields(
    tooltip: MyTooltip,              // Option<MyTooltip>
    hover_count: usize = 0,          // usize
)]
fn Card(name: String) {
    let _ = tooltip.as_ref().map(|t| t.text.as_str()).unwrap_or("");
    let _ = hover_count;
    Div().child(Label().text(name.as_str()))
}
```

### Extension fields — `ext(...)`

See [`#[extensions(...)]`](#extensions) for the full list. Each name maps to
a field, an initialiser, a render binding, and a trait impl.

### Return type

The component function does **not** need a return type or a trailing
`.into()`. The macro wraps the body in a `Component::render` method whose
signature is fixed.

### Attribute forwarding

Every attribute on the function other than `#[doc]`, `#[struct_fields(...)]`,
and the `#[component(...)]` groups is forwarded **verbatim** to the generated
struct. The macro does not bake in any derives — you decide.

### Collision rules

- **Duplicate names anywhere** (across `props`, `struct_fields`, `variants`,
  fn params) — compile error naming both sources.
- **A field whose name matches an extension field** — compile error, because
  extension fields are reserved by freyacn.

---

## The `#[variant]` macro

Applied to an enum, generates:

- `impl MyEnum { fn style_str(&self) -> &'static str; }`
- `impl MyEnum { fn resolve(&self) -> freyacn::Style; }` — convenience alias.
- `impl freyacn::Variant for MyEnum` — the trait-based resolver.
- `impl From<&str>` and `impl From<String>`, matching kebab-cased variant
  names with a fallback to the first declared variant.

### Declaration

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

### Kebab-case conversion

| PascalCase             | Kebab                   |
|------------------------|-------------------------|
| `Default`              | `default`               |
| `Md`                   | `md`                    |
| `XS`                   | `xs`                    |
| `SecondaryDestructive` | `secondary-destructive` |

### Resolving to a `Style`

`.resolve()` reads the active theme and parses the variant's style string. It
must be called inside a Freya render context. When used as a prop inside a
component that declares `variants(...)` and `ext(style)`, the component macro
calls `.resolve()` for you and merges the result into the render body's
`style` local.

The `Variant` trait exists so the component macro can merge variant styles
without knowing the concrete enum type:

```rust
pub trait Variant {
    fn resolve(&self) -> Style;
}
```

---

## Style layering

When a component declares `variants(...)` alongside `ext(style)`, the
generated `render` body computes the effective style before user code runs:

```text
Style::default()
    .merge(variant.resolve())      // declared order
    .merge(size.resolve())
    .merge(self.style.clone())     // user overrides win
```

The render body then binds `style: &Style` to the merged value.

### Consequences

**Overrides persist across variant changes.** Calling `.variant("outline")`
on a button whose caller previously set `.bg_green_50()` recomputes the base
layer but leaves user intent intact — the override still wins.

**Every `StyleExt` call is an override.**

```rust
Button()
.variant(ButtonVariant::Default)
.p_6()              // override padding — wins over the variant's px-2.5
.text_xl()          // override font size
.style("font-bold text-white")
```

### Merging primitives

`Style::merge(other)` produces a new `Style` where:

- **Scalar fields** (`Option<T>`) → `other.field.or(self.field)`. Set values
  in `other` win; unset values leave `self` alone.
- **Stack fields** (`shadows`, `font_families`, `text_shadows`) →
  concatenated.
- **`Gaps` fields** (`padding`, `margin`) → merged per side, so a variant
  setting only `px-4` doesn't wipe the base class's `py-2`.

There is also `Style::or(defaults)` for the inverse (fill gaps only), plus
`merge_from` / `or_from` for in-place mutation.

### State folding

`Div` folds `hover:` / `focus:` / `active:` / `disabled:` / `dark:` layers at
render time. If you build a primitive that doesn't go through `Div`, compute
the flag set yourself and call `with_active_states`:

```rust
let mut active = StateFlags::NONE;
if theme.is_dark        { active = active.or(StateFlags::DARK); }
if * hovering.read() == HOVERED { active = active.or(StateFlags::HOVER); }
if focus().is_focused() { active = active.or(StateFlags::FOCUS_VISIBLE); }

let folded = style.with_active_states(active);
```

---

## Interactive units and pointer events

Freya's hit tester routes each pointer event to the **topmost interactive**
element under the pointer. When both a parent and a child are interactive,
moving the pointer onto the child fires `pointer_leave` on the parent, which
clears the parent's hover state — even though the pointer is still visually
inside the parent.

The whole crate follows one rule to avoid that:

**The outermost element of a widget is the interactive unit. Its children are
non-interactive and react to the parent's state via context.**

Concretely:

| Rule                                                                                                                                                                       | Where it lives                                |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------|
| `Div` is interactive by default and publishes its active `StateFlags` via `DivActiveState` context.                                                                        | `div.rs`                                      |
| `Label` and `Icon` default to `pointer_events = false`. Hover is inherited from the parent `Div` via `DivActiveState`.                                                     | `label.rs`, `icon.rs`                         |
| Containers don't inherit their parent's state — `inherit_state` on `Div` defaults to `false`, so a hovered row doesn't light up every button inside it.                    | `div.rs`                                      |
| Leaves that follow the parent (labels) opt in with `inherit_state = true`.                                                                                                 | `label.rs`                                    |
| Hover / press state is tracked via `on_pointer_enter` / `on_pointer_leave` (not `over` / `out`, which bubble from children).                                               | `div.rs`, `label.rs`, `group.rs`, `button.rs` |
| Hover state uses `State<u8>` with sentinel values (`IDLE = 1`, `HOVERED = 2`) mutated with `*state.write() = v`, so the reactive store never skips a `true → false` write. | `div.rs`, `label.rs`                          |

If a child genuinely needs its own hover (a standalone `Label` in a list, an
`Icon` acting as a button, an inline disclosure control), opt in per leaf:

```rust,no_run
Label()
    .text("Hover me")
    .style("pointer-events-auto hover:underline cursor-pointer")
```

Every showcase in the repo follows the pattern — the accordion's title
`Label` is non-interactive and underlines via inherited hover, not
`pointer-events-auto`.

---

## `StyleExt` reference

`Style` is the data container. `StyleExt` is the trait that provides the
method surface. `Style` implements its own trait, so bare `Style` values
chain the same way components do.

### Base setters

`background`, `color`, `corner_radius`, `border_width`, `border_color`,
`border_style`, `font_size`, `font_weight`, `font_slant`, `font_family`,
`text_align`, `text_decoration`, `text_transform`, `line_height`,
`letter_spacing`, `text_indent`, `opacity`, `shadow`, `ring_width`,
`ring_color`, `ring_offset`, `padding`, `margin`, `width`, `height`,
`min_width`, `min_height`, `max_width`, `max_height`, `position`, `z_index`,
`overflow`, `direction`, `main_align`, `cross_align`, `align_self`,
`align_content`, `content`, `spacing`, `gap_x`, `gap_y`, `order`, `visibility`,
`user_select`, `resize`, `pointer_events`, `aspect`, `columns`, `table_layout`,
`border_collapse`, `list_style_type`, `list_style_position`, `object_fit`.

### Palette colours

Every family × shade, exposed as `bg_*` and `text_*`:

- Neutrals — `slate`, `gray`, `zinc`, `neutral`, `stone`, `mauve`, `olive`,
  `mist`, `taupe`
- Accents — `red`, `orange`, `amber`, `yellow`, `lime`, `green`, `emerald`,
  `teal`, `cyan`, `sky`, `blue`, `indigo`, `violet`, `purple`, `fuchsia`,
  `pink`, `rose`

Shades: `50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950`.

### Semantic tokens

`background`, `foreground`, `primary`, `primary_foreground`, `secondary`,
`secondary_foreground`, `muted`, `muted_foreground`, `accent`,
`accent_foreground`, `destructive`, `destructive_foreground`, `card`,
`card_foreground`, `popover`, `popover_foreground`, `border`, `input`, `ring`,
`chart_1` … `chart_5`, plus the `sidebar_*` family and `disabled` /
`disabled_foreground`.

### Literal colours

`text_white`, `text_black`, `bg_white`, `bg_black`, `bg_transparent`,
`text_transparent`.

### Typography

Sizes (`text_xs` … `text_9xl`), weights (`font_thin`, `font_extralight`,
`font_light`, `font_normal`, `font_medium`, `font_semibold`, `font_bold`,
`font_extrabold`, `font_black_weight`), slant (`italic`, `not_italic`),
families (`font_sans`, `font_serif`, `font_mono`), alignment (`text_left`,
`text_center`, `text_right`, `text_justify`), decoration (`underline`,
`overline`, `line_through`, `no_underline`, `decoration_color`,
`decoration_thickness`, `underline_offset`), transform (`uppercase`,
`lowercase`, `capitalize`, `normal_case`), extras (`whitespace_*`,
`break_words`, `break_all`, `break_keep`, `text_ellipsis_set`, `text_clip`,
`line_clamp`, list style helpers, `accent_color`, `caret_color`,
`selection_color`, `placeholder_color`, `line_height`, `letter_spacing`,
`text_indent`).

### Borders

Width (`border`, `border_0`, `border_2`, `border_4`, `border_8`, and per-side
`border_t` / `border_r` / `border_b` / `border_l` / `border_x` / `border_y`),
colour (`border_primary`, `border_destructive`, and palette tokens), style (`border_solid`, `border_dashed`,
`border_dotted`, `border_double`,
`border_none`), and per-side divide utilities (`divide_x`, `divide_y`,
`divide_color`).

**Per-side borders render on transparent backgrounds.** `Div` splits them
across the flex main axis (rendered as child rects inside the visual) and the
cross axis (rendered as siblings in an opposite-direction wrapper). No
`bg-*` needed.

> **`border-*` needs both width and colour.** `border-green-500` alone sets
> only the border colour and renders nothing (same as CSS). Use
> `border-2 border-green-500`, or the method form
> `.border_width(2.0).border_color(c)`.

### Corner radius

`rounded_none`, `rounded_sm`, `rounded`, `rounded_md`, `rounded_lg`,
`rounded_xl`, `rounded_2xl`, `rounded_3xl`, `rounded_full`.

### Effects

Opacity (`opacity_0`, `opacity_25`, `opacity_50`, `opacity_75`,
`opacity_100`), shadows (`shadow_sm`, `shadow_md`, `shadow_lg`, `shadow_xl`,
`shadow_2xl`, `shadow_none`), ring (`ring`, `ring_0` … `ring_8`,
`ring_primary`, `ring_offset`, `inset_ring`), outline (`outline_width`,
`outline_color`, `outline_offset`, `outline_none`), transforms (`rotate`,
`scale`, `transform_origin`, `blur`).

> **Ring tokens parse but do not render.** Freya 0.5.0-rc.10 has no ring
> primitive; `ring-*` / `ring-offset-*` / `inset-ring-*` set
> `Style::ring_width` and store the value, but nothing is painted. Use
> `border-*` or `shadow-*` for the visual equivalent until the primitive
> lands.

`blur(radius)` sets the element's **backdrop** blur (CSS
`backdrop-filter: blur()`), not element-self blur. Freya's renderer exposes
`Rect::backdrop_blur` and no element-self equivalent.

### Animations

`animate_spin`, `animate_ping`, `animate_pulse`, `animate_bounce`,
`animate_none`. `Div` starts the animation on mount and drives the
corresponding paint-time transform each frame.

### Flex

Direction (`flex_row`, `flex_col`), wrapping (`flex_wrap_content`,
`flex_wrap_spacing`), content mode (`content_flex`, `content_fit`,
`content_normal`), flex sizing (`flex_1`, `grow`, `grow_0`, `shrink`,
`shrink_0`, `shrink_1`), justification (`justify_start`, `justify_center`,
`justify_end`, `justify_between`, `justify_around`, `justify_evenly`),
cross-axis (`items_start`, `items_center`, `items_end`), self alignment,
content alignment, gaps (`gap`, `gap_x`, `gap_y`, `space_x`, `space_y`), and
`order`.

`space_x` / `space_y` apply `margin-left` / `margin-top` on every child
except the first — matching Tailwind's `space-x-*` / `space-y-*`. Negative
values (`-space-x-2`) produce overlap.

### Sizing

Raw (`w`, `h`, `min_w`, `min_h`, `max_w`, `max_h`, `size`), percentages,
keywords (`w_full`, `w_auto`, `h_full`, `h_auto`, `size_full`), Tailwind
scale (`w_0` … `w_96`, plus `min_w_*` … `max_h_*`), aspect ratio.

### Spacing

`p_*`, `px_*`, `py_*`, `pt_*`, `pr_*`, `pb_*`, `pl_*` and `m_*`, `mx_*`,
`my_*`, `mt_*`, `mr_*`, `mb_*`, `ml_*` on the Tailwind scale (1 unit = 4 px),
including fractional steps (`p_0_5`, `p_1_5`, `p_2_5`, `p_3_5`).

### Positioning

`relative`, `absolute`, `global`, `top`, `right`, `bottom`, `left`, `inset`,
`inset_x`, `inset_y`, `z_index`.

### Overflow

`overflow_none`, `overflow_clip`, `overflow_scroll`, `overflow_scroll_x`,
`overflow_scroll_xy`, and the per-axis clip / none helpers.

> **Use `overflow-clip`, not `overflow-hidden`.** The token parser recognises
> `overflow-clip` / `overflow-none` / `overflow-scroll*`. `overflow-hidden`
> is not a token and parses to nothing.

### Object fit

`object_fit`, `object_fill`, `object_cover`, `object_contain`,
`object_none`, `object_scale_down` map to the `ObjectFit` enum. Consumed by
the `Image` component.

### Layout extras

`hidden`, `visible`, `invisible`, `sr_only`, `not_sr_only`,
`pointer_events_none`, `pointer_events_auto`, `select_none`, `select_text`,
`select_all`, `select_auto`, `resize`, `resize_none`, `resize_x`, `resize_y`,
`table_auto`, `table_fixed`, `border_collapse`, `border_separate`, `columns`,
`container`.

### Merging

- `Style::merge(self, other) -> Style` — `other` wins on every field it set.
- `Style::or(self, defaults) -> Style` — `self` wins; `defaults` fills gaps.
- `Style::merge_from(&mut self, other)` / `or_from` — in-place variants.
- `Style::with_active_states(self, flags) -> Style` — fold state-conditional
  styles matching `flags`.
- `StyleExt::merge_style(other)` / `or_style(defaults)` — lift the above onto
  any component.

---

## Style-string API

For developers who prefer Tailwind's own syntax, every `StyleExt` implementor
accepts a style string:

```rust,no_run
# use freyacn::prelude::*;
let _ = Style::default().style("p-4 bg-primary text-white shadow-md font-bold");
```

The parser understands the full Tailwind vocabulary that Freya can render —
palette and semantic colours, opacity modifiers (`/80`), spacing, sizing (including `w-screen`), typography, borders,
rings (`ring`, `ring-offset-*`,
`inset-ring-*`), arbitrary bracket values (`text-[12.8px]`,
`bg-[#ff0000]`), transforms (`rotate-*`, `scale-*`, `blur-*`), effects, flex,
positioning, overflow, aspect ratio, order, space, divide, outline, list,
accent/caret, animations (`animate-spin`, `animate-pulse`, `animate-bounce`,
`animate-ping`), and state-conditional modifiers (`hover:`, `focus:`,
`focus-visible:`, `active:`, `disabled:`, `dark:`, `first:`, `last:`,
`odd:`, `even:`, `only:`, `not-hover:`, `group-hover:`, `has-hover:`).

Negative forms work everywhere they make sense: `-m-4`, `-mt-2`,
`-space-x-2`, `-translate-y-1`, `-top-4`, `-rotate-45`, `-scale-90`.
Unknown tokens are silently ignored (matching Tailwind's behaviour).

### `style_debug`

When you want to know which tokens had no effect — for typo hunting or test
assertions — use `style_debug`:

```rust,no_run
# use freyacn::prelude::*;
let (style, unknown) = Style::default()
    .style_debug("p-4 bg-primary shdaow-md not-a-thing");

assert_eq!(unknown, vec!["shdaow-md", "not-a-thing"]);
```

### What is not representable in Freya today

- **CSS Grid layout** — `grid-cols-*` is parsed but renders as flex wrap.
- **Element-self blur** — `filter: blur()` has no Freya equivalent. Only
  `backdrop-filter: blur()` works (`backdrop_blur` on `Rect`).
- **Ring primitives** — `ring-*`, `ring-offset-*`, `inset-ring-*` parse and
  store, but Freya has no ring renderer. Use `border-*` or `shadow-*`.
- **Responsive viewport-width prefixes** (`sm:`, `md:`, `lg:`) — no
  breakpoint system in Freya's renderer. Use `use_viewport()` in Rust code.
- **Per-axis overflow-x / overflow-y that differ** — `Overflow` is a single
  value.

Everything else in the vocabulary renders. Per-side borders, animations,
scroll containers, fixed positioning, and pointer/focus/hover state all work
end-to-end.

---

### `cn!` macro

Utility macro for Tailwind-style class concatenation, skipping empty values
and `None` options:

```rust
# use freyacn::prelude::*;
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

## `#[extensions(...)]`

Extensions opt a struct into freyacn's built-in bundles. Each name maps to a
field, an initialiser, a render-body local, and a trait impl.

| Name             | Trait                                | Field(s) added                                           | Trait method             |
|------------------|--------------------------------------|----------------------------------------------------------|--------------------------|
| `children`       | `freyacn::prelude::ChildrenExt`      | `children: Vec<Element>`                                 | `get_children`           |
| `key`            | `freyacn::prelude::KeyExt`           | `key: DiffKey`                                           | `write_key`              |
| `style`          | `freyacn::prelude::StyleExt`         | `style: Style`                                           | `get_style`              |
| `event_handlers` | `freyacn::prelude::EventHandlersExt` | `event_handlers: FxHashMap<EventName, EventHandlerType>` | `get_event_handlers`     |
| `accessibility`  | `freyacn::prelude::AccessibilityExt` | `accessibility: AccessibilityData`                       | `get_accessibility_data` |

> **Freya 0.5 note.** `EventHandlersExt::with_event_handlers` was renamed to
> `event_handlers`. All components in the workspace are on the new name; if
> you're porting a struct from a 0.4-based project, update the call site
> accordingly.

Standalone usage:

```rust,no_run
use freyacn::prelude::*;

#[extensions(children, key, style, accessibility)]
#[derive(Clone, PartialEq, Default)]
pub struct PlainBox;
```

---

## Components

`freyacn` ships thirteen core primitives, built with the macro engine and
Tailwind v4 styling. Each one has a runnable example under `examples/`.

### `Div`

The foundational, state-aware layout primitive. Wraps Freya's `rect()`,
forwards every `Style` field Freya can render, and owns all runtime state
observation — consumers never need to compute `StateFlags` themselves.

- **State observation**: `DARK`, `HOVER`, `ACTIVE`, `FOCUS`,
  `FOCUS_VISIBLE`, `DISABLED`, plus structural `FIRST` / `LAST` / `ODD` /
  `EVEN` / `ONLY` and `GROUP_HOVER` / `GROUP_FOCUS` from an ancestor
  `Group`.
- **Full style surface**: background, foreground, typography, font width,
  text height, text shadows, corner radius, borders (uniform and per-side),
  padding, margin, size constraints, position, overflow, flex layout,
  opacity, shadows, z-index via layer, text overflow, pointer events,
  interactive, rotate / scale / backdrop-blur / transform-origin, child
  offsets, animations, and accessibility.
- **Props**: `disabled`, `focusable`, `inherit_state` (default `false`),
  `custom_a11y_id`, `a11y_role`, `a11y_expanded`, `a11y_labelled_by`,
  `a11y_hidden`.
- **Interaction callbacks**: `on_hover`, `on_press`.

```rust,no_run
use freyacn::prelude::*;

Div()
    .style("p-4 bg-card text-card-foreground rounded-lg border shadow-sm")
    .focusable(true)
    .on_hover(|_| println!("hovered"))
    .on_press(|_| println!("pressed"))
    .child(Label().text("Card content"))
```

Any component that returns a `Div` gets `hover:` / `focus:` / `active:` /
`disabled:` support for free. `Div` publishes its active `StateFlags` via
`DivActiveState` context, so `Label` and any other state-sensitive leaf can
react to the parent's state without being interactive themselves.

`Div` also applies `Style::space_x` / `Style::space_y` as `margin-left` /
`margin-top` on every child except the first — matching Tailwind's
`space-x-*` / `space-y-*` semantics.

### `Button`

The primary interactive component, composited on top of `Div`.

- **6 variants**: `Default`, `Outline`, `Secondary`, `Ghost`, `Destructive`,
  `Link`. Accepts string coercion: `.variant("outline")`.
- **8 sizes**: `Default` (h-8), `Xs` (h-6), `Sm` (h-7), `Lg` (h-9), `Icon`
  (size-8), `IconXs`, `IconSm`, `IconLg`.
- **Props**: `disabled`, `focusable`,
  `on_hover: Option<EventHandler<Event<PointerEventData>>>`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

`Button` forwards the caller's `accessibility` extension and sets
`Role::Button` on the underlying `Div` by default, so screen readers announce
it as a button. Add a label via
`Button().accessibility(AccessibilityData::new().alt("..."))`.

Content is passed as children, matching the shadcn/ui API:

```rust,no_run
use freyacn::prelude::*;

Button()
    .child(Label().text("Delete item"))
    .variant(ButtonVariant::Destructive)
    .size(ButtonSize::Lg)
    .on_press(|_| println!("Deleted!"));
```

Children of the button (`Label`, `Icon`) are non-interactive by default, so
the button's `hover:` / `active:` styles stay applied when the pointer is
over its own label or icon.

### `Label`

A text primitive. Renders a `rect()` wrapper around Freya's `label()`, applies
text styles to the label directly, and box styles to the wrapper. Owns its
own hover / press / focus tracking, and inherits the parent `Div`'s active
flags via `DivActiveState`.

- **Props**: `text`, `disabled`, `focusable`, `inherit_state` (default
  `true`), `custom_a11y_id`, `a11y_role`, `a11y_labelled_by`, `a11y_hidden`.
- **Text styles**: colour, size, weight, slant, width, families, align,
  decoration, height, shadow, truncation / line-clamp, line-height, case
  transform.
- **Box styles**: sizes (including `min_*` / `max_*`), opacity, rotate /
  scale / backdrop-blur, positioning, events, a11y.
- **Extensions**: `style`, `event_handlers`, `accessibility`.

Non-interactive by default — a `Label` inside a `Button` or
`AccordionTrigger` reacts to the parent's hover without stealing it. Opt in
with `.pointer-events-auto()` for a standalone hoverable label:

```rust,no_run
Label()
    .text("Hover me")
    .style("pointer-events-auto hover:underline cursor-pointer")
```

### `Icon`

A current-colour SVG. Wraps Freya's `SvgViewer` in a `Div`. Defaults to 16×16
when no size is set; mirrors the other axis when only one is set.

- **Props**: `svg: Bytes`.
- **Extensions**: `style`, `accessibility`, `key`.

Non-interactive by default, so an `Icon` inside a `Button` or
`AccordionTrigger` reacts to the parent's hover without stealing it. The SVG
uses `currentColor`, so it picks up the parent's text colour through Freya's
cascade. Opt in with `.pointer-events-auto()` for a standalone hoverable
icon.

### `Image`

A styled image primitive that mimics HTML's `<img>` and wraps Freya's native
`image()`. Reads from a file path or pre-read bytes, caches the decoded
handle keyed on `src`, and falls back gracefully when the file is missing.

- **Props**: `src: String`, `bytes: Option<Bytes>`, `alt: String`,
  `fit: ImageFit`, `sampling: SamplingMode`, `snap_to_grid: bool`,
  `blur: f32`.
- **Extensions**: `style`, `accessibility`, `event_handlers`, `key`.
- **Supported formats** (via `skia_safe::Image::from_encoded`): PNG, JPEG,
  WebP, GIF, BMP, ICO, WBMP. AVIF, HEIF, and TIFF when Skia is built with
  the corresponding codecs. SVG is not supported — rasterise first.
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

`bytes` bypasses the filesystem read but still decodes. If both `src` and
`bytes` are set, `bytes` wins. Missing or corrupt sources render a muted
placeholder sized to the caller's constraints.

### `Avatar`

A user/entity avatar built on top of `Image`, with an `AvatarFallback` slot,
size/shape variants, and optional badge support.

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
- **`.bordered()`** — explicit padded ring on a standalone avatar. Inside an
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

The fallback slot for `Avatar`. Renders a muted, centered box that fills the
avatar. Use directly for full control over content, or via
`.fallback("AL")` on `Avatar` for the shortcut.

- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

### `AvatarGroup`

A horizontal row of overlapping avatars.

- **Props**: `spacing: f32 = -8.0`.
- **Extensions**: `style`, `children`, `key`, `event_handlers`,
  `accessibility`.

Each child is wrapped in a `Div` with 2px padding filled with `theme.card` —
that padded ring is the visual separator that makes overlapping avatars
readable. Non-first children also get `.ml(spacing)` for overlap and
`.z_index(i)` for stacking.

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
`Avatar` supplies that context when `.badge(...)` is used, so the badge is a
sibling of the clipped avatar image and not cut by its `overflow-clip`.
Default color is `bg-primary`; override with `.style("bg-green-500")` or any
bg token.

### `Divider`

A 1px separating line. Horizontal by default (spans the parent's width);
vertical (`DividerOrientation::Vertical`) spans the parent's height. Both are
`shrink-0` so flex siblings can't squeeze them.

Supports an optional centred label:

```rust,no_run
use freyacn::prelude::*;

Divider().label("OR");   // ───────────── OR ─────────────
```

- **Props**: `label`, `orientation`.
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
    .child(
        Label()
            .text("Hover this row")
            .style("group-hover:underline group-hover:text-primary"),
    )
```

### `Accordion`

A shadcn-shaped accordion. Root / Item / Trigger / Content composition,
single-open and multiple-open modes, per-item disabled, and animated
open/close on both click and single-mode auto-close.

- **Root props**: `type_`, `collapsible`, `disabled`,
  `default_value: Option<Vec<String>>`.
- **Item props**: `value`, `disabled`.
- **Extensions**: `style`, `children`, `key`.

The simple API:

```rust,no_run
use freyacn::prelude::*;

Accordion()
    .type_(AccordionType::Single)
    .collapsible(true)
    .item("item-1", "Is it accessible?", "Yes — role=Button triggers.")
    .item("item-2", "Is it animated?", "Yes — linear height and chevron.")
```

The compound API when you need custom trigger children:

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

The trigger `Div` is the interactive unit; its `Label` and internal chevron
are non-interactive, so the trigger keeps its `hover:` state across the
whole row. `hover:underline` on the title label with `flex-1` underlines the
title whenever the pointer is anywhere on the row.

Height and chevron animations are **linear** (front-loaded easing curves read
as "jumped then stuck" at 200–300ms) and durations are constants at the top
of `accordion.rs`.

Accessibility: `AccordionTrigger` sets `Role::Button` and `aria-expanded`;
`AccordionContent` sets `Role::Region` and is hidden while closed.
`AccordionItem` allocates trigger and content ids via `use_a11y()` and links
them with `a11y_labelled_by`, so screen readers can navigate trigger to
panel.

---

## Theming

The `Theme` struct holds every semantic colour (`background`, `foreground`,
`primary`, …) plus a `Colors` palette containing all Tailwind colours. It's
`Copy`, so passing it around is free.

### Initialising a theme

```rust,no_run
# use freyacn::prelude::*;
// Fluent light or dark theme with default palette:
let theme = Theme::light().slate();
use_init_cn_theme(theme);

// Or chainable fluent configuration:
let theme = Theme::dark().stone().theme_color("rose");
use_init_cn_theme(theme);

// Or with runtime palette names:
let theme = Theme::from_base("zinc", true).theme_color("blue");
use_init_cn_theme(theme);
```

`.base_color(...)` resets `primary` / `accent` / `ring` to the new base's
defaults, so `.theme_color(...)` must come after it. Per-palette shortcuts (`.stone()`, `.mauve()`, `.olive()`,
`.mist()`, `.taupe()`, `.slate()`,
`.gray()`, `.zinc()`, `.neutral()`) skip the string.

### Reading the theme inside a component

```rust,no_run
# use freyacn::prelude::*;
// Bind the State first: in Freya 0.5, `State::read()` returns a guard
// whose lifetime is tied to `&self`, so a temporary `State<Theme>`
// can't be dropped at the semicolon.
let state = use_cn_theme();
let bg      = state.read().background;
let primary = state.read().primary;
```

All palette and semantic helpers read the theme at call time, so switching
themes updates every component on the next render without reconstruction.

`use_init_cn_theme` reuses an existing context if one exists. For a scoped
override that always creates a fresh context, use `use_provide_cn_theme`.
`get_cn_theme_or_default()` returns a `Readable<Theme>` and never panics —
useful for built-in components that should work without an explicit provider.

---

## The CLI — `freyacn-cli`

`freyacn-cli` is the workspace tool for developing, building, and shipping
FreyaCN apps. Its command surface mirrors the Tauri CLI:

```bash
freyacn-cli dev            # run in dev mode with hot-reload
freyacn-cli build          # produce a release build
freyacn-cli bundle         # package the app for the current platform
freyacn-cli icon icon.png  # generate platform icons
freyacn-cli info           # print environment and version info
```

Additional subcommands will land as the workspace tool matures. The CLI is
part of the same workspace as the runtime, so versions are kept in lockstep.

---

## Customisation

Components are builders. Extend them or build your own with the same
primitives.

```rust
use freyacn::prelude::*;

#[component(
    props(label: String),
    ext(style, children),
)]
fn PrimaryButton(label: String) {
    Div()
        .merge_style(style.clone())
        .children(children.clone())
        .child(Label().text(label.as_str()))
}
```

Use `#[struct_fields(...)]` to add internal storage for your own trait impls:

```rust
#[component(props(label: String))]
#[struct_fields(tooltip: MyTooltip)]
fn TooltippedLabel(label: String) { /* … */ }

impl TooltipExt for TooltippedLabelComponent {
    fn set_tooltip(&mut self, text: impl Into<String>) {
        let tip = self.tooltip.get_or_insert_with(MyTooltip::default);
        tip.text = text.into();
    }
}
```

---

## Examples

Every component has a runnable showcase under
`crates/freyacn/examples/` in the `freyacn`:

```bash
cargo run --example div_showcase
cargo run --example button_showcase
cargo run --example icon_showcase
cargo run --example label_showcase
cargo run --example divider_showcase
cargo run --example image_showcase
cargo run --example avatar_showcase
cargo run --example accordion_showcase
```

Each showcase follows the same skeleton — `use_init_cn_theme` at the root, an
outer `h-screen overflow-scroll` `Div`, and a `max-w-2xl mx-auto` column of
`section(...)` cards — so it's easy to copy when writing a new one.

The accordion showcase includes an **inheritance** section that puts the same
accordion on three different backgrounds (`bg-muted`, `bg-background`,
`bg-secondary`) to demonstrate that items inherit their container's surface
and don't paint a background of their own.

The image showcase covers all four `object-fit` modes, every `SamplingMode`
variant, blur, snap-to-grid, embedded bytes, a missing-file fallback, and a
long list to exercise scroll.

The avatar showcase covers sizes, shapes, the `.fallback()` shortcut,
explicit `AvatarFallback`, missing sources, embedded bytes, badges in all
sizes and colours, `AvatarGroup` with default and custom spacing, and a
`AvatarGroup + badges` composition.

---

## Roadmap

- [ ] More components: `Card`, `Input`, `Select`, `Checkbox`, `Switch`,
  `Dialog`, `Tabs`, `Tooltip`, `Popover`, `Dropdown`.
- [x] `Div`, `Button`, `Label`, `Icon`, `Divider`, `Group`, `Accordion`,
  `Image`, `Avatar`, `AvatarFallback`, `AvatarGroup`, `AvatarGroupCount`,
  `AvatarBadge`.
- [ ] `freyacn-cli` — a `dev` / `build` / `bundle` / `icon` flow matching the
  Tauri CLI's ergonomics.
- [ ] Publish `freyacn`, `freyacn-macros`, and `freyacn-cli` to crates.io.
- [ ] Theme overrides at the component level.
- [ ] Additional icon sets.
- [ ] Storybook-style interactive docs.
- [ ] `resolve_with(&Theme)` for theme-explicit variant resolution outside a
  render context.
- [ ] First-class support for the style-surface fields Freya doesn't yet
  render (rings, outlines) as soon as the underlying primitives land.
- [ ] Element-self blur (`filter: blur()`) once Freya exposes a
  `Rect::blur` equivalent.

**Done since 0.2.0-alpha.1:**

- Freya 0.5.0-rc.10 port.
- Per-side borders render on transparent backgrounds.
- Animations (`animate-*`).
- `first:` / `last:` / `odd:` / `even:` / `only:`, `not-hover:`,
  `group-hover:`, `has-hover:` structural and group selectors.
- Interactive-units architecture (single interactive element per widget,
  children react via `DivActiveState`).
- Accordion with single/multiple/collapsible modes, per-item disabled,
  animated open/close on click **and** single-mode auto-close.
- `Image` component with path caching, bytes input, all `object-fit` modes,
  all sampling modes, blur, snap-to-grid, and graceful missing-file fallback.
- `Avatar` family: `Avatar`, `AvatarFallback`, `AvatarGroup`,
  `AvatarGroupCount`, `AvatarBadge`, with size/shape variants, fallback
  shortcut, automatic in-group ring, and per-item z-stacking.
- Negative style-string tokens (`-m-4`, `-space-x-2`, `-translate-y-1`,
  `-top-4`, `-rotate-45`, `-scale-90`).
- `space_x` / `space_y` rendered by `Div` (margin-left / margin-top on every
  child except the first).
- `object-fit` tokens (`object-fill`, `object-cover`, `object-contain`,
  `object-none`, `object-scale-down`).

---

## Migration from 0.1

The 0.2 release is a substantial rewrite. The most visible changes:

| Before (0.1)                                                                                              | After (0.2)                                |
|-----------------------------------------------------------------------------------------------------------|--------------------------------------------|
| `BackgroundExt`, `ForegroundExt`, `BorderExt`, `SpacingExt`, `SizingExt`, `EffectsExt`, `CornerRadiusExt` | One `StyleExt` trait                       |
| `fg_*` prefix                                                                                             | `text_*` prefix                            |
| Manual `impl MyComponent { fn background(self, …) … }`                                                    | `#[component]` macro                       |
| Hand-rolled variant matching in `render`                                                                  | `#[variant]` enums + `variants(...)` group |
| `#[extensions(...)]` on `#[component]`                                                                    | `ext(...)` group inside `#[component]`     |
| Raw `rect()` in render bodies                                                                             | Styled `Div()` component                   |
| `.class("…")`                                                                                             | `.style("…")`                              |
| `Theme::base_color(name, dark)`                                                                           | `Theme::light()` / `Theme::dark()`         |

Custom impls that previously implemented multiple extension traits should
implement `StyleExt` once and return `&mut Style`.

### 0.2.0-alpha.1 → current (Freya 0.5 port)

- `EventHandlersExt::with_event_handlers` → `event_handlers`.
- `State::read()` returns a guard tied to `&self`. Bind the state to a local
  before reading: `let state = use_cn_theme(); state.read()...`.
- `Rect::blur` removed. Use `backdrop_blur` for `backdrop-filter: blur()`.
- `CornerRadius`'s fields are private. Use `.is_round()` or per-corner
  accessors.
- `Size::fill()` removed. Use `Size::percent(100.0)`.
- `image`, `radio`, `timer` were added to both `freya::prelude` and
  `freya::icons::lucide`. The prelude re-exports the lucide versions
  explicitly to resolve the ambiguity.
- Interactive-unit pattern: `Div`'s `inherit_state` defaults to `false`;
  `Label` and `Icon` default to non-interactive; opt in with
  `pointer-events-auto()` per leaf.

---

## Contributing

Contributions are welcome. Before opening a PR, please open an issue
describing what you want to change and why — the public API is still in flux,
and a short design discussion will save everyone time.

Areas where help is especially useful:

- New components and variants.
- Style-string parser coverage — new Tailwind tokens Freya can render.
- Documentation and runnable examples.
- Testing infrastructure for proc-macro output.

### Guidelines

- Every new component gets a runnable example under `examples/` that follows
  the standard `section(...)` skeleton.
- Components that own interaction follow the interactive-units pattern: the
  outermost element is interactive, children are not, hover is inherited
  through `DivActiveState`.
- Never call `use_cn_theme().read()` inline — bind the state first.
- `Rect`/`Label` inherent methods come from Freya's traits; `StyleExt`
  methods come from freyacn's trait. If both offer the same name, prefer the
  `Style`-based one for consistency with the rest of the crate.
- Use `overflow-clip` (not `overflow-hidden`) in style strings — the parser
  only recognises the former.
- Prefer `border_width(...).border_color(...)` method calls over
  `border-*` tokens when you need a border to render reliably.

---

## License

MIT.

---

## Acknowledgements

- [Freya](https://github.com/marc2332/freya) — for the GUI framework.
- [shadcn/ui](https://ui.shadcn.com) — for design inspiration.
- [Tailwind CSS](https://tailwindcss.com) — for the utility-first philosophy.