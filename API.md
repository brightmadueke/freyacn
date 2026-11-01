## Table of contents

1. [Overview](#1-overview)
2. [Quick reference — "I want to…"](#2-quick-reference--i-want-to)
3. [Style system](#3-style-system)
    - 3.1 [`Style` struct](#31-style-struct)
    - 3.2 [`StyleExt` trait](#32-styleext-trait)
    - 3.3 [Style merging](#33-style-merging)
    - 3.4 [Style strings](#34-style-strings)
    - 3.5 [`StateFlags` and `StateCondition`](#35-stateflags-and-statecondition)
4. [Theme system](#4-theme-system)
    - 4.1 [`Theme` struct](#41-theme-struct)
    - 4.2 [`Colors` palette](#42-colors-palette)
    - 4.3 [Building themes](#43-building-themes)
    - 4.4 [Theme hooks](#44-theme-hooks)
5. [Utilities](#5-utilities)
    - 5.1 [`cn!` macro](#51-cn-macro)
    - 5.2 [`utils` module](#52-utils-module)
6. [Patterns and recipes](#6-patterns-and-recipes)
7. [Gotchas and pitfalls](#7-gotchas-and-pitfalls)

---

## 1. Overview

`freyacn` is a styling and theming toolkit for [Freya](https://github.com/marc2332/freya), targeting **Freya
0.5.0-rc.10**.

It is built around three primitives:

| Primitive      | Purpose                                                                                                                                 |
|----------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| **`Style`**    | A plain data struct holding every styling field (background, typography, borders, spacing, flex, …). `Style::default()` is empty.       |
| **`StyleExt`** | A trait with one required method, `get_style()`. Provides ~500 chainable methods: `bg_primary()`, `p_4()`, `text_sm()`, `flex_col()`, … |
| **`Theme`**    | A resolved, copyable set of semantic tokens (`background`, `primary`, `card`, …) plus the full Tailwind palette.                        |

> **Note**: The crate no longer ships a component layer or a `#[component]` / `#[variant]` macro system. `Style`,
> `StyleExt`, and `Theme` compose directly with Freya's own elements. See [§9.3](#93-implementing-styleext) for
> wiring up your own component struct.

### Installation

```toml
[dependencies]
freyacn = "0.2.0-alpha.1"
```

```rust
use freyacn::prelude::*;
```

The prelude re-exports:

- `Style`, `StyleExt`, `StyledString`, `StyledColor`
- `Theme`, `ThemeRegistry`, `ThemeHandle`, theme hooks (`use_init_cn_theme`, `use_cn_theme`,
  `use_provide_cn_theme`, `get_cn_theme_or_default`)
- The `cn!` macro
- All of Freya's own prelude (`EventHandler`, `Event`, `LaunchConfig`, `WindowConfig`, `State`, `launch`, …)
- All Freya lucide icons (`info()`, `chevron_down()`, `check()`, `alert_circle()`, …)
- `EventHandlerType`, `EventName`, `FxHashMap`
- Utility helpers (`color_with_alpha`, `blend_colors`, `get_color_from_fill`, `apply_opacity`)

**Freya is a transitive dependency** — you never add it to your `Cargo.toml` and never write `use freya::prelude::*`.

---

## 2. Quick reference — "I want to…"

| Goal                                | Use                                                                                           |
|-------------------------------------|-----------------------------------------------------------------------------------------------|
| Set the app theme                   | `use_init_cn_theme(Theme::dark().neutral())`                                                  |
| Read a theme token                  | `let s = use_cn_theme(); let bg = s.read().background;`                                       |
| Lay out children vertically         | `Style::default().flex_col().gap(8.0)`                                                        |
| Lay out children horizontally       | `Style::default().flex_row().items_center()`                                                  |
| Center content                      | `Style::default().flex_row().justify_center().items_center()`                                 |
| Give a box padding, radius, surface | `Style::default().p(16.0).corner_radius(8.0).background(card)`                                |
| Add a border                        | `Style::default().style("border border-border")` — **always pair width + colour**             |
| Tailwind-style string shorthand     | `Style::default().style("p-4 bg-card rounded-lg")`                                            |
| Conditional class strings           | `cn!("base", if active { "active" } else { "" })`                                             |
| Typo-check a style string           | `let (s, unknown) = Style::default().style_debug("p-4 fizzbuzz");`                            |
| Build your own widget               | Implement `StyleExt` for a struct that owns a `Style` (see [§9.3](#93-implementing-styleext)) |

---

## 3. Style system

### 3.1 `Style` struct

`Style` is a plain data container — no component logic lives on it. Every field is `Option<T>`, `Vec<T>`, or a similar
container, so `Style::default()` is empty.

```rust
use freyacn::prelude::*;

let s = Style::default ()
.background(Color::RED)
.p_4()
.corner_radius(8.0);
```

`Style` fields are public. You'll rarely construct a `Style` by hand — the `StyleExt` methods exist for that — but
you'll often inspect one when integrating with Freya elements.

#### Field reference

| Category                 | Fields (all `Option<T>` unless noted)                                                                                                                                                                                                                                                                          |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Background & surface** | `background`, `bg_repeat`, `bg_size`, `bg_attachment`, `bg_clip`                                                                                                                                                                                                                                               |
| **Borders**              | `border_width`, `border_top_width`, `border_right_width`, `border_bottom_width`, `border_left_width`, `border_color`, `border_style`, `divide_x`, `divide_y`, `divide_color`, `border_collapse`                                                                                                                |
| **Corner radius**        | `corner_radius`                                                                                                                                                                                                                                                                                                |
| **Shadows & rings**      | `shadows: Vec<Shadow>`, `ring_width`, `ring_color`, `ring_offset`, `ring_offset_color`, `inset_ring_width`, `inset_ring_color`, `outline_width`, `outline_color`, `outline_offset`                                                                                                                             |
| **Text**                 | `text_color`, `font_size`, `font_slant`, `font_weight`, `font_families: Vec<Cow<'static, str>>`, `font_stretch`, `text_align`, `text_decoration`, `text_transform`, `line_height`, `letter_spacing`, `text_indent`, `whitespace`, `word_break`, `text_ellipsis`, `line_clamp`, `text_shadows: Vec<TextShadow>` |
| **Text decoration**      | `decoration_color`, `decoration_thickness`, `underline_offset`                                                                                                                                                                                                                                                 |
| **Text extras**          | `selection_color`, `accent_color`, `caret_color`, `placeholder_color`, `list_style_type`, `list_style_position`                                                                                                                                                                                                |
| **Visibility & effects** | `opacity`, `visibility`, `blur` (backdrop), `pointer_events`, `user_select`, `resize`, `sr_only`                                                                                                                                                                                                               |
| **Transform**            | `rotate`, `scale`, `transform_origin`, `offset_x`, `offset_y`                                                                                                                                                                                                                                                  |
| **Box**                  | `padding`, `margin`, `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height`, `visible_width`, `visible_height`, `aspect_ratio`                                                                                                                                                                |
| **Flex / layout**        | `direction`, `main_align`, `cross_align`, `align_self`, `align_content`, `content`, `spacing`, `gap_x`, `gap_y`, `space_x`, `space_y`, `flex_shrink`, `flex_grow`, `order`                                                                                                                                     |
| **Grid**                 | `grid_cols`, `grid_rows`, `col_span`, `row_span`                                                                                                                                                                                                                                                               |
| **Positioning**          | `position_kind`, `top`, `right`, `bottom`, `left`, `z_index`                                                                                                                                                                                                                                                   |
| **Overflow & scroll**    | `overflow`, `overflow_x`, `overflow_y`, `scroll_axis`, `scrollbar`                                                                                                                                                                                                                                             |
| **Media**                | image fields written by `object-*`, `blur-*`, `sampling-*` tokens (see [`StyledString`](#34-style-strings))                                                                                                                                                                                                    |
| **Misc**                 | `columns`, `container`, `table_layout`                                                                                                                                                                                                                                                                         |
| **Conditional styles**   | `states: Vec<StateStyle>`                                                                                                                                                                                                                                                                                      |
| **Animation**            | `animation: Option<AnimationSpec>`, `transition: Option<TransitionSpec>`                                                                                                                                                                                                                                       |

#### Materialisation helpers

```rust
impl Style {
    pub fn to_border(&self) -> Option<Border>;      // fold border fields into a Freya Border
    pub fn to_position(&self) -> Option<Position>;  // fold position_kind + insets into a Position
    pub fn is_empty(&self) -> bool;                 // true iff Style::default()
}
```

#### Local enums

`Style` uses several freyacn-defined enums:

```rust
pub enum TextTransform { None, Uppercase, Lowercase, Capitalize }
pub enum BorderStyle { Solid, Dashed, Dotted, Double, None }
pub enum Visibility { Visible, Hidden, Collapse }
pub enum ObjectFit { Fill, Contain, Cover, None, ScaleDown }
pub enum UserSelect { None, Text, All, Auto }
pub enum Resize { None, Both, Horizontal, Vertical }
pub enum Whitespace { Normal, Nowrap, Pre, PreLine, PreWrap, BreakSpaces }
pub enum WordBreak { Normal, Words, All, Keep }
pub enum TextEllipsis { Clip, Ellipsis }
pub enum ListStyleType { None, Disc, Decimal }
pub enum ListStylePosition { Inside, Outside }
pub enum BackgroundRepeat { Repeat, NoRepeat, RepeatX, RepeatY }
pub enum BackgroundSize { Auto, Cover, Contain }
pub enum BackgroundAttachment { Fixed, Local, Scroll }
pub enum BackgroundClip { Border, Padding, Content, Text }
pub enum BorderCollapse { Collapse, Separate }
pub enum TableLayout { Auto, Fixed }
pub enum PositionKind { Relative, Absolute, Global }
pub enum ScrollAxis { Vertical, Horizontal, Both }
pub enum AnimationName { Spin, Ping, Pulse, Bounce, None }
```

---

### 3.2 `StyleExt` trait

The main API for styling. In this crate, `Style` itself implements `StyleExt`; you can implement it for your own
widgets to get all ~500 provided methods.

**Required method:**

```rust
pub trait StyleExt: Sized {
    fn get_style(&mut self) -> &mut Style;
}
```

Implement this for any struct, and all ~500 provided methods become available:

```rust
struct MyWidget {
    style: Style
}

impl StyleExt for MyWidget {
    fn get_style(&mut self) -> &mut Style { &mut self.style }
}

let w = MyWidget { style: Style::default () }
.bg_primary()
.p_4()
.text_white();
```

#### Base setters

Concrete-value setters. Most other methods funnel through these.

```rust
fn background(self, color: Color) -> Self;
fn color(self, color: Color) -> Self;
fn corner_radius(self, radius: impl Into<CornerRadius>) -> Self;
fn border_width(self, width: f32) -> Self;
fn border_color(self, color: Color) -> Self;
fn border_style(self, style: BorderStyle) -> Self;
fn font_size(self, size: impl Into<FontSize>) -> Self;
fn font_weight(self, weight: impl Into<FontWeight>) -> Self;
fn font_slant(self, slant: impl Into<FontSlant>) -> Self;
fn font_family(self, family: impl Into<Cow<'static, str>>) -> Self;
fn text_align(self, align: impl Into<TextAlign>) -> Self;
fn text_decoration(self, decoration: impl Into<TextDecoration>) -> Self;
fn text_transform(self, transform: TextTransform) -> Self;
fn line_height(self, value: f32) -> Self;
fn letter_spacing(self, value: f32) -> Self;
fn text_indent(self, value: f32) -> Self;
fn opacity(self, opacity: f32) -> Self;
fn shadow(self, shadow: impl Into<Shadow>) -> Self;
fn ring_width(self, width: f32) -> Self;
fn ring_color(self, color: Color) -> Self;
fn ring_offset(self, offset: f32) -> Self;
fn ring_offset_color(self, c: Color) -> Self;
fn inset_ring_width(self, width: f32) -> Self;
fn inset_ring_color(self, c: Color) -> Self;
fn padding(self, gaps: impl Into<Gaps>) -> Self;
fn margin(self, gaps: impl Into<Gaps>) -> Self;
fn width(self, size: impl Into<Size>) -> Self;
fn height(self, size: impl Into<Size>) -> Self;
fn min_width(self, size: impl Into<Size>) -> Self;
fn min_height(self, size: impl Into<Size>) -> Self;
fn max_width(self, size: impl Into<Size>) -> Self;
fn max_height(self, size: impl Into<Size>) -> Self;
fn visible_width(self, size: impl Into<VisibleSize>) -> Self;
fn visible_height(self, size: impl Into<VisibleSize>) -> Self;
fn z_index(self, z: i32) -> Self;
fn overflow(self, overflow: Overflow) -> Self;
fn overflow_x(self, o: Overflow) -> Self;
fn overflow_y(self, o: Overflow) -> Self;
fn direction(self, dir: Direction) -> Self;
fn main_align(self, align: Alignment) -> Self;
fn cross_align(self, align: Alignment) -> Self;
fn align_self(self, align: Alignment) -> Self;
fn align_content(self, align: Alignment) -> Self;
fn content(self, content: Content) -> Self;
fn spacing(self, spacing: f32) -> Self;
fn gap_x(self, size: f32) -> Self;
fn gap_y(self, size: f32) -> Self;
fn space_x(self, px: f32) -> Self;
fn space_y(self, px: f32) -> Self;
fn shadow(self, s: impl Into<Shadow>) -> Self;
fn text_shadow(self, shadow: impl Into<TextShadow>) -> Self;
fn font_width(self, width: impl Into<FontWidth>) -> Self;
fn text_height(self, height: impl Into<TextHeightBehavior>) -> Self;
fn rotate(self, degrees: f32) -> Self;
fn scale(self, scale: impl Into<Scale>) -> Self;
fn transform_origin(self, origin: impl Into<TransformOrigin>) -> Self;
fn blur(self, radius: f32) -> Self;      // backdrop blur
fn offset_x(self, px: f32) -> Self;
fn offset_y(self, px: f32) -> Self;
fn object_fit(self, fit: ObjectFit) -> Self;
```

#### Color helpers

```rust
// Literal colors
fn text_white(self) -> Self;
fn text_black(self) -> Self;
fn bg_white(self) -> Self;
fn bg_black(self) -> Self;
fn bg_transparent(self) -> Self;
fn text_transparent(self) -> Self;

// Palette colours: {bg,text}_{family}_{shade}
// Families: stone, mauve, olive, mist, taupe, slate, gray, zinc, neutral,
//           red, orange, amber, yellow, lime, green, emerald, teal, cyan,
//           sky, blue, indigo, violet, purple, fuchsia, pink, rose
// Shades:   50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950
fn bg_slate_500(self) -> Self;
fn text_red_600(self) -> Self;
// … 26 families × 11 shades × 2 prefixes = 572 methods

// Semantic tokens: {bg,text}_{token}
// Tokens: background, foreground, primary, primary_foreground, secondary,
//         secondary_foreground, muted, muted_foreground, accent,
//         accent_foreground, destructive, destructive_foreground, card,
//         card_foreground, popover, popover_foreground, border, input,
//         ring, chart_1..chart_5, sidebar_*, disabled, disabled_foreground
fn bg_primary(self) -> Self;
fn text_muted_foreground(self) -> Self;
fn bg_card(self) -> Self;
// …

// Ring-prefixed variants (same families and tokens)
fn ring_primary(self) -> Self;
fn ring_offset_primary(self) -> Self;
fn inset_ring_primary(self) -> Self;
fn ring_slate_500(self) -> Self;      // etc.
```

#### Typography

```rust
// Font size
fn text_xs(self) -> Self;      // 12
fn text_sm(self) -> Self;      // 14
fn text_base(self) -> Self;    // 16
fn text_lg(self) -> Self;      // 18
fn text_xl(self) -> Self;      // 20
fn text_2xl(self) -> Self;     // 24
fn text_3xl(self) -> Self;     // 30
fn text_4xl(self) -> Self;     // 36
fn text_5xl(self) -> Self;     // 48
fn text_6xl(self) -> Self;     // 60
fn text_7xl(self) -> Self;     // 72
fn text_8xl(self) -> Self;     // 96
fn text_9xl(self) -> Self;     // 128

// Font weight
fn font_thin(self) -> Self;
fn font_extralight(self) -> Self;
fn font_light(self) -> Self;
fn font_normal(self) -> Self;
fn font_medium(self) -> Self;
fn font_semibold(self) -> Self;
fn font_bold(self) -> Self;
fn font_extrabold(self) -> Self;
fn font_black_weight(self) -> Self;

// Style
fn italic(self) -> Self;
fn not_italic(self) -> Self;

// Family
fn font_sans(self) -> Self;
fn font_serif(self) -> Self;
fn font_mono(self) -> Self;

// Alignment
fn text_left(self) -> Self;
fn text_center(self) -> Self;
fn text_right(self) -> Self;
fn text_justify(self) -> Self;

// Decoration
fn underline(self) -> Self;
fn overline(self) -> Self;
fn line_through(self) -> Self;
fn no_underline(self) -> Self;
fn decoration_color(self, c: Color) -> Self;
fn decoration_thickness(self, px: f32) -> Self;
fn underline_offset(self, px: f32) -> Self;

// Case
fn uppercase(self) -> Self;
fn lowercase(self) -> Self;
fn capitalize(self) -> Self;
fn normal_case(self) -> Self;

// Whitespace & break
fn whitespace_normal(self) -> Self;
fn whitespace_nowrap(self) -> Self;
fn whitespace_pre(self) -> Self;
fn whitespace_pre_line(self) -> Self;
fn whitespace_pre_wrap(self) -> Self;
fn whitespace_break_spaces(self) -> Self;
fn break_normal(self) -> Self;
fn break_words(self) -> Self;
fn break_all(self) -> Self;
fn break_keep(self) -> Self;

// Truncation
fn text_ellipsis_set(self) -> Self;
fn text_clip(self) -> Self;
fn line_clamp(self, n: u32) -> Self;

// Other
fn accent_color(self, c: Color) -> Self;
fn caret_color(self, c: Color) -> Self;
fn selection_color(self, c: Color) -> Self;
fn placeholder_color(self, c: Color) -> Self;
```

#### Borders & radius

```rust
// Widths
fn border(self) -> Self;      // 1px
fn border_0(self) -> Self;
fn border_2(self) -> Self;
fn border_4(self) -> Self;
fn border_8(self) -> Self;

// Colours
fn border_primary(self) -> Self;
fn border_destructive(self) -> Self;
// Also `border_{family}_{shade}` for every palette entry, e.g. `border_slate_500()`

// Styles
fn border_solid(self) -> Self;
fn border_dashed(self) -> Self;
fn border_dotted(self) -> Self;
fn border_double(self) -> Self;
fn border_none(self) -> Self;

// Per-side widths
fn border_t(self, w: f32) -> Self;
fn border_r(self, w: f32) -> Self;
fn border_b(self, w: f32) -> Self;
fn border_l(self, w: f32) -> Self;
fn border_x(self, w: f32) -> Self;
fn border_y(self, w: f32) -> Self;

// Corner radius
fn rounded_none(self) -> Self;   // 0
fn rounded_sm(self) -> Self;     // 2
fn rounded(self) -> Self;        // 4
fn rounded_md(self) -> Self;     // 6
fn rounded_lg(self) -> Self;     // 8
fn rounded_xl(self) -> Self;     // 12
fn rounded_2xl(self) -> Self;    // 16
fn rounded_3xl(self) -> Self;    // 24
fn rounded_full(self) -> Self;   // 9999

// Divide (for split borders between children)
fn divide_x(self, px: f32) -> Self;
fn divide_y(self, px: f32) -> Self;
fn divide_color(self, c: Color) -> Self;
```

**Important:** `border_2()` sets only the *width*. Pair it with `border_color` (or use `border border-border` in a style
string). A border with width but no colour is invisible.

#### Effects

```rust
fn opacity_0(self) -> Self;
fn opacity_25(self) -> Self;
fn opacity_50(self) -> Self;
fn opacity_75(self) -> Self;
fn opacity_100(self) -> Self;

fn shadow_sm(self) -> Self;
fn shadow_md(self) -> Self;
fn shadow_lg(self) -> Self;
fn shadow_xl(self) -> Self;
fn shadow_2xl(self) -> Self;
fn shadow_none(self) -> Self;

fn ring(self) -> Self;      // ring_width(1)
fn ring_0(self) -> Self;
fn ring_1(self) -> Self;
fn ring_2(self) -> Self;
fn ring_3(self) -> Self;
fn ring_4(self) -> Self;
fn ring_6(self) -> Self;
fn ring_8(self) -> Self;

fn ring_offset_0 / 1 / 2 / 4 / 8( self ) -> Self;
fn ring_offset_color(self, c: Color) -> Self;

fn inset_ring(self) -> Self;
fn inset_ring_0 / 1 / 2 / 4 / 8( self ) -> Self;
fn inset_ring_width(self, w: f32) -> Self;
fn inset_ring_color(self, c: Color) -> Self;

fn ring_white / black/transparent( self ) -> Self;
fn ring_offset_white / black( self ) -> Self;
fn inset_ring_white / black( self ) -> Self;

fn outline_width(self, px: f32) -> Self;
fn outline_color(self, c: Color) -> Self;
fn outline_offset(self, px: f32) -> Self;
fn outline_none(self) -> Self;
```

**Note:** `ring-*` tokens parse and store values, but Freya 0.5.0-rc.10 has no ring renderer. Use `border-*` or
`shadow-*` for visible rings.

#### Flex

```rust
fn flex_row(self) -> Self;
fn flex_col(self) -> Self;
fn flex_wrap_content(self) -> Self;
fn flex_wrap_spacing(self, spacing: f32) -> Self;
fn content_flex(self) -> Self;
fn content_fit(self) -> Self;
fn content_normal(self) -> Self;
fn flex_1(self) -> Self;
fn grow(self) -> Self;
fn grow_0(self) -> Self;
fn shrink(self) -> Self;
fn shrink_0(self) -> Self;
fn shrink_1(self) -> Self;
fn flex_shrink(self, value: f32) -> Self;
fn grow_factor(self, value: f32) -> Self;

fn justify_start / center/end/between/around/evenly( self ) -> Self;
fn items_start / center/end( self ) -> Self;
fn self_start / center/end( self ) -> Self;
fn content_start / center/end/between/around/evenly( self ) -> Self;
fn gap(self, size: f32) -> Self;
fn order(self, n: i32) -> Self;
fn order_first / last/none( self ) -> Self;
```

#### Sizing

```rust
// Raw
fn w(self, px: f32) -> Self;
fn h(self, px: f32) -> Self;
fn min_w(self, px: f32) -> Self;
fn min_h(self, px: f32) -> Self;
fn max_w(self, px: f32) -> Self;
fn max_h(self, px: f32) -> Self;
fn size(self, px: f32) -> Self;

// Percentages & keywords
fn w_percent(self, pct: f32) -> Self;
fn h_percent(self, pct: f32) -> Self;
fn w_full(self) -> Self;
fn w_auto(self) -> Self;
fn h_full(self) -> Self;
fn h_auto(self) -> Self;
fn size_full(self) -> Self;

// Tailwind scale (0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 5, 6, 7, 8, 9, 10,
//                 11, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56,
//                 60, 64, 72, 80, 96)
fn w_0(), w_0_5(), w_1(), … fn w_96(self) -> Self;
fn h_0(), h_0_5(), … fn h_96(self) -> Self;
fn min_w_ *, min_h_*, max_w_*, max_h_*, size_*;

// Aspect ratio
fn aspect(self, ratio: f32) -> Self;
fn aspect_square(self) -> Self;
fn aspect_video(self) -> Self;
fn aspect_auto(self) -> Self;
```

**Scaling:** 1 Tailwind unit = 4px. So `w_4()` = 16px, `p_2()` = 8px.

#### Spacing

Same Tailwind scale as sizing.

```rust
fn p_0…96( self ) -> Self; fn px_ *; fn py_ *; fn pt_ *; fn pr_ *; fn pb_ *; fn pl_ *;
fn m_0…96( self ) -> Self; fn mx_ *; fn my_ *; fn mt_ *; fn mr_ *; fn mb_ *; fn ml_ *;
```

**Per-side merging:** `p_2 px_4` produces 8px vertical, 16px horizontal — `px` overrides only left/right.

#### Positioning

```rust
fn relative(self) -> Self;
fn absolute(self) -> Self;
fn global(self) -> Self;    // fixed
fn top(self, value: f32) -> Self;
fn right(self, value: f32) -> Self;
fn bottom(self, value: f32) -> Self;
fn left(self, value: f32) -> Self;
fn inset(self, value: f32) -> Self;
fn inset_x(self, value: f32) -> Self;
fn inset_y(self, value: f32) -> Self;
```

#### Overflow & scroll

```rust
fn overflow_none(self) -> Self;
fn overflow_clip(self) -> Self;
fn overflow_scroll(self) -> Self;       // vertical
fn overflow_scroll_x(self) -> Self;
fn overflow_scroll_xy(self) -> Self;
fn scroll_axis(self, axis: ScrollAxis) -> Self;
fn scrollbar(self, show: bool) -> Self;
fn overflow_x_clip / none( self ) -> Self;
fn overflow_y_clip / none( self ) -> Self;
```

**Important:** Use `overflow_clip()`, not `overflow_hidden` (which doesn't parse). `overflow_hidden` in a style string
also doesn't parse — the parser only knows `overflow-clip` / `overflow-none` / `overflow-scroll*`.

#### Layout extras

```rust
fn hidden(self) -> Self;            // visibility: Collapse
fn visible(self) -> Self;
fn invisible(self) -> Self;         // visibility: Hidden
fn visibility(self, v: Visibility) -> Self;
fn sr_only(self) -> Self;
fn not_sr_only(self) -> Self;

fn pointer_events_none(self) -> Self;
fn pointer_events_auto(self) -> Self;
fn select_none / text/all/ auto ( self ) -> Self;
fn resize / none/x/y( self ) -> Self;

fn table_auto / fixed( self ) -> Self;
fn border_collapse / separate( self ) -> Self;
fn columns(self, n: u32) -> Self;
fn container(self) -> Self;
```

#### Animations & transitions

```rust
fn animate_spin(self) -> Self;
fn animate_ping(self) -> Self;
fn animate_pulse(self) -> Self;
fn animate_bounce(self) -> Self;
fn animate_none(self) -> Self;

fn transition_all(self) -> Self;
fn transition_colors(self) -> Self;
fn transition_opacity(self) -> Self;
fn transition_transform(self) -> Self;
fn transition_none(self) -> Self;
```

`transition_*` is stored but not currently rendered. Animation specs are useful when consumed by a rendering layer that
reads `Style::animation`.

#### Object-fit

```rust
fn object_fit(self, fit: ObjectFit) -> Self;
fn object_fill(self) -> Self;
fn object_cover(self) -> Self;
fn object_contain(self) -> Self;
fn object_none(self) -> Self;
fn object_scale_down(self) -> Self;
```

#### Merging

```rust
fn merge_style(self, other: Style) -> Self;
fn or_style(self, defaults: Style) -> Self;
fn state_style(self, cond: StateCondition, style: Style) -> Self;
fn with_active_states(self, active: StateFlags) -> Self;
```

#### Style strings

```rust
fn style(self, styles: impl AsRef<str>) -> Self;
```

Parses a Tailwind-style token string. See [§3.4](#34-style-strings).

---

### 3.3 Style merging

`Style` has explicit merging primitives for combining base styles, variant styles, and user overrides.

```rust
impl Style {
    pub fn merge(self, other: Style) -> Style;
    pub fn or(self, defaults: Style) -> Style;
    pub fn merge_from(&mut self, other: Style);
    pub fn or_from(&mut self, defaults: Style);
    pub fn is_empty(&self) -> bool;
    pub fn with_active_states(self, active: StateFlags) -> Style;
}
```

| Method               | Behaviour                                                                                                  |
|----------------------|------------------------------------------------------------------------------------------------------------|
| `merge(self, other)` | `other` wins on every field it set. Stack fields (`shadows`, `font_families`, `text_shadows`) concatenate. |
| `or(self, defaults)` | `self` wins; `defaults` fills gaps only.                                                                   |

**Special cases:**

- **`padding` and `margin`** merge *per side*. A variant that sets only `px_4` doesn't wipe the base's `py_2`.
- **`states`** (state-conditional styles) concatenate.

**Typical layering** (when writing your own component):

```text
Style::default()
    .style("<base>")             // base layer
    .merge(variant_style)        // variant layer
    .merge(user_style)           // user overrides — always win
```

**Consequences:**

- **Overrides persist across re-computation.** Recomputing a base layer doesn't wipe the caller's `.bg_green_50()`.
- **Base is least specific; user overrides are most specific.** Matches shadcn's `cn(base, variant, size, user)`
  layering.

---

### 3.4 Style strings

Every `StyleExt` implementor accepts a Tailwind-style token string:

```rust
Style::default ().style("p-4 bg-card rounded-lg shadow-sm hover:shadow-md")
```

Unknown tokens are silently ignored (matches Tailwind).

#### Token categories

**Layout:** `flex`, `inline-flex`, `flex-row`, `flex-col`, `flex-wrap`, `flex-1`, `grow`, `grow-0`, `shrink`,
`shrink-0`, `shrink-1`, `content-*`, `justify-*`, `items-*`, `self-*`, `gap-*`, `w-*`, `h-*`, `min-w-*`, `min-h-*`,
`max-w-*`, `max-h-*`, `size-*`, `grid`, `grid-cols-1…12`, `col-span-*`, `row-span-*`.

**Color:** `bg-<family>-<shade>`, `text-<family>-<shade>`, `border-<family>-<shade>`, `ring-<family>-<shade>`, every
semantic token (`bg-primary`, `text-muted-foreground`, `border-input`, `ring-ring`, `bg-sidebar-accent`, …). Opacity
modifiers: `bg-primary/50`, `text-red-500/[0.37]`. Arbitrary hex: `bg-[#ff0000]`, `text-[#rgb]`, `border-[#rrggbbaa]`.
`color-mix()` is also supported: `bg-[color-mix(in_oklch,...)]`.

**Typography:** `text-xs`…`text-9xl`, `font-thin`…`font-black`, `italic`, `not-italic`, `underline`, `overline`,
`line-through`, `no-underline`, `uppercase`, `lowercase`, `capitalize`, `normal-case`, `text-left/center/right/justify`,
`leading-*`, `tracking-*`, `truncate`, `text-ellipsis`, `text-clip`, `line-clamp-*`, `whitespace-*`, `break-*`,
`text-[12.8px]`.

**Border, radius, effects:** `border`, `border-0/2/4/8`, `border-t/r/b/l/x/y`, `border-<colour>`,
`border-solid/dashed/dotted/double/none`, `rounded`, `rounded-none/sm/md/lg/xl/2xl/3xl/full`, `rounded-[10px]`,
`shadow-sm/md/lg/xl/2xl/none`, `opacity-*`.

**Ring tokens:** `ring`, `ring-0/1/2/3/4/6/8`, `ring-<colour>`, `ring-offset-*`, `inset-ring`. **These parse but don't
render** — Freya has no ring primitive.

**Position, overflow:** `relative`, `absolute`, `global`, `top-*`, `right-*`, `bottom-*`, `left-*`, `inset-*`,
`inset-x-*`, `inset-y-*`, `z-*`, `overflow-clip`, `overflow-none`, `overflow-scroll`, `overflow-scroll-x`,
`overflow-scroll-xy`, `overflow-x-*`, `overflow-y-*`.

**Transforms:** `rotate-*`, `-rotate-*`, `scale-*`, `-scale-*`, `translate-x-*`, `-translate-x-*`, `translate-y-*`,
`-translate-y-*`, `blur-*`, `origin-*`.

**Object fit:** `object-fill`, `object-cover`, `object-contain`, `object-none`, `object-scale-down`.

**Animations:** `animate-spin`, `animate-ping`, `animate-pulse`, `animate-bounce`, `animate-none`, `transition-*`.

#### Negative forms

`-m-4`, `-mt-2`, `-space-x-2`, `-translate-y-1`, `-top-4`, `-rotate-45`, `-scale-90`, `-inset-x-2`. Every negative token
resolves to a negative length before hitting the corresponding positive helper.

#### State prefixes

Chainable: `dark:hover:bg-slate-800`, `focus-visible:ring-2`, `disabled:opacity-50`, `active:bg-muted/80`.

Recognised prefixes: `dark`, `hover`, `focus`, `focus-visible`, `focus-within`, `active`, `disabled`, `visited`,
`checked`, `target`, `first`, `last`, `odd`, `even`, `only`, `group-hover`, `group-focus`, `group-active`, `has-hover`,
`has-focus`, `has-active`, `not-<state>` (e.g. `not-hover:opacity-100`).

**Fold rules:** State tokens parse into `Style::states` as `(StateCondition, Style)` pairs. When your own widget owns
runtime state, it computes the active `StateFlags` at render time and calls `with_active_states()` — one-way fold, later
matching entries win. See [§3.5](#35-stateflags-and-statecondition).

#### Arbitrary values

Every length token accepts `[<value>]` brackets. Units understood: `px`, `rem`, `em`, `pt`, `%`, `vw`/`vh` (for sizes).

```rust
Style::default ().style("w-[137px] text-[12.8px] gap-[1.5rem] bg-[#ff0000]")
```

#### `style_debug` — typo hunting

```rust
let (style, unknown) = Style::default ().style_debug("p-4 shdaow-md not-a-thing");
assert_eq!(unknown, vec!["shdaow-md", "not-a-thing"]);
```

#### What does NOT work (parses but doesn't render)

| Token                                      | Reason                                         |
|--------------------------------------------|------------------------------------------------|
| `ring-*`, `ring-offset-*`, `inset-ring-*`  | Freya has no ring primitive                    |
| `outline-*`                                | Same — parsed, not rendered                    |
| `overflow-hidden`                          | Not a token — use `overflow-clip`              |
| `grid-cols-[0_1fr]`, variable grid columns | Only equal columns supported                   |
| `has-[>svg]:*`, `[&>svg]:*`                | No descendant selectors                        |
| `sm:`, `md:`, `lg:`                        | No viewport breakpoints — use `use_viewport()` |
| `backdrop-blur` as token                   | Use `.blur(radius)` method or `blur-*`         |

---

### 3.5 `StateFlags` and `StateCondition`

State conditions for conditional styles. If you're writing a widget that owns its own hover/focus/active state, read
this section.

```rust
pub struct StateFlags(u32);

impl StateFlags {
    pub const NONE: Self;
    pub const DARK: Self;
    pub const HOVER: Self;
    pub const FOCUS: Self;
    pub const FOCUS_VISIBLE: Self;
    pub const FOCUS_WITHIN: Self;
    pub const ACTIVE: Self;
    pub const DISABLED: Self;
    pub const VISITED: Self;
    pub const CHECKED: Self;
    pub const TARGET: Self;
    pub const FIRST: Self;
    pub const LAST: Self;
    pub const ODD: Self;
    pub const EVEN: Self;
    pub const ONLY: Self;
    pub const GROUP_HOVER: Self;
    pub const GROUP_FOCUS: Self;
    pub const GROUP_ACTIVE: Self;

    pub const fn or(self, other: Self) -> Self;
    pub const fn is_subset_of(self, other: Self) -> bool;
    pub const fn intersects(self, other: Self) -> bool;
    pub const fn is_empty(self) -> bool;
}
```

```rust
pub struct StateCondition {
    pub positive: StateFlags,
    pub negative: StateFlags,
}

impl StateCondition {
    pub const fn none() -> Self;
    pub const fn pos(flag: StateFlags) -> Self;
    pub const fn neg(flag: StateFlags) -> Self;
    pub const fn and(self, other: Self) -> Self;
    pub const fn is_empty(&self) -> bool;
    pub fn matches(&self, active: StateFlags) -> bool;
}
```

`Style::with_active_states(flags)` folds every `(StateCondition, Style)` pair where `cond.matches(flags)`.

**Manual fold example** (for widgets that don't use the style-string state mechanism):

```rust
let mut active = StateFlags::NONE;
if theme.is_dark        { active = active.or(StateFlags::DARK); }
if * hovering.read()    { active = active.or(StateFlags::HOVER); }
if focus().is_focused() { active = active.or(StateFlags::FOCUS_VISIBLE); }
let folded = style.with_active_states(active);
```

---

## 4. Theme system

### 4.1 `Theme` struct

```rust
pub struct Theme {
    pub is_dark: bool,
    pub colors: Colors,

    // Core
    pub background: Color,
    pub foreground: Color,

    // Primary
    pub primary: Color,
    pub primary_foreground: Color,

    // Secondary
    pub secondary: Color,
    pub secondary_foreground: Color,

    // Muted
    pub muted: Color,
    pub muted_foreground: Color,

    // Accent
    pub accent: Color,
    pub accent_foreground: Color,

    // Destructive
    pub destructive: Color,
    pub destructive_foreground: Color,

    // Card
    pub card: Color,
    pub card_foreground: Color,

    // Popover
    pub popover: Color,
    pub popover_foreground: Color,

    // Borders & inputs
    pub border: Color,
    pub input: Color,
    pub ring: Color,

    // Charts
    pub chart_1: Color,
    pub chart_2: Color,
    pub chart_3: Color,
    pub chart_4: Color,
    pub chart_5: Color,

    // Sidebar
    pub sidebar: Color,
    pub sidebar_foreground: Color,
    pub sidebar_primary: Color,
    pub sidebar_primary_foreground: Color,
    pub sidebar_accent: Color,
    pub sidebar_accent_foreground: Color,
    pub sidebar_border: Color,
    pub sidebar_ring: Color,

    // States
    pub disabled: Color,
    pub disabled_foreground: Color,

    // Radius
    pub radius_sm: f32,   // 6
    pub radius_md: f32,   // 8
    pub radius_lg: f32,   // 10
    pub radius_xl: f32,   // 14
}
```

`Theme` is `Copy` — passing it around is free.

---

### 4.2 `Colors` palette

`Colors` is the full Tailwind color palette, available at `Theme::colors`.

```rust
pub struct Colors {
    pub white: Color,
    pub black: Color,

    // 26 families × 11 shades (50, 100, 200, …, 900, 950)
    pub stone_50: Color,
    pub stone_100: Color,
    … pub stone_950: Color,
    pub mauve_*,
    pub olive_*,
    pub mist_*,
    pub taupe_*,
    pub slate_*,
    pub gray_*,
    pub zinc_*,
    pub neutral_*,
    pub red_*,
    pub orange_*,
    pub amber_*,
    pub yellow_*,
    pub lime_*,
    pub green_*,
    pub emerald_*,
    pub teal_*,
    pub cyan_*,
    pub sky_*,
    pub blue_*,
    pub indigo_*,
    pub violet_*,
    pub purple_*,
    pub fuchsia_*,
    pub pink_*,
    pub rose_*,
}

impl Colors {
    pub fn new() -> Self;
}
```

Access: `theme.colors.slate_500`, `theme.colors.blue_400`.

The palette is also exposed as free functions from `crate::styling::colors`, each returning a `StyledColor`:
`colors::blue_500()`, `colors::neutral_950()`, `colors::white()`, `colors::transparent()`, etc.

---

### 4.3 Building themes

```rust
impl Theme {
    // Entry points
    pub fn light() -> Self;              // neutral base, light mode
    pub fn dark() -> Self;               // neutral base, dark mode
    pub fn from_base(base: &str, dark: bool) -> Self;

    // Fluent modifiers
    pub fn base_color(self, base: &str) -> Self;
    pub fn theme_color(self, accent: &str) -> Self;

    // Base shortcuts
    pub fn stone(self) -> Self;
    pub fn mauve(self) -> Self;
    pub fn olive(self) -> Self;
    pub fn mist(self) -> Self;
    pub fn taupe(self) -> Self;
    pub fn slate(self) -> Self;
    pub fn gray(self) -> Self;
    pub fn zinc(self) -> Self;
    pub fn neutral(self) -> Self;

    // Radius overrides
    pub fn radius_sm(self, value: f32) -> Self;
    pub fn radius_md(self, value: f32) -> Self;
    pub fn radius_lg(self, value: f32) -> Self;
    pub fn radius_xl(self, value: f32) -> Self;
    pub fn radius(self, value: f32) -> Self;   // derives all four steps
}
```

**Usage:**

```rust
// Default neutral dark theme
let theme = Theme::dark();

// Slate base, blue accent
let theme = Theme::dark().base_color("slate").theme_color("blue");

// Shorthand
let theme = Theme::dark().stone().theme_color("rose");

// Full neutral
let theme = Theme::light().neutral();
```

**Order matters:** `.base_color(...)` **resets** `primary` / `accent` / `ring` to the base palette's defaults. Always
chain `.theme_color(...)` after `.base_color(...)` / `.stone()` / `.zinc()`.

**Theme values (default neutral):**

| Token              | Light              | Dark               |
|--------------------|--------------------|--------------------|
| `background`       | `#ffffff`          | `rgb(10,10,10)`    |
| `foreground`       | `rgb(10,10,10)`    | `rgb(250,250,250)` |
| `card`             | `#ffffff`          | `rgb(23,23,23)`    |
| `popover`          | `#ffffff`          | `rgb(38,38,38)`    |
| `primary`          | `rgb(23,23,23)`    | `rgb(229,229,229)` |
| `muted`            | `rgb(245,245,245)` | `rgb(38,38,38)`    |
| `muted_foreground` | `rgb(115,115,115)` | `rgb(161,161,161)` |
| `border`           | `rgb(229,229,229)` | `white/10%`        |
| `input`            | `rgb(229,229,229)` | `white/15%`        |
| `ring`             | `rgb(161,161,161)` | `rgb(115,115,115)` |
| `destructive`      | `rgb(231,0,11)`    | `rgb(255,100,103)` |

These match shadcn/ui v4's `globals.css` byte-for-byte for the `neutral` base.

---

### 4.4 Theme hooks

```rust
// Provide a theme (reuses existing context if present, updates in-place otherwise)
pub fn use_init_cn_theme(theme: Theme) -> State<Theme>;

// Provide a theme in a new subtree (always creates a new context)
pub fn use_provide_cn_theme(theme: Theme) -> State<Theme>;

// Read the theme (panics if no provider)
pub fn use_cn_theme() -> State<Theme>;

// Read the theme with a fallback (never panics)
pub fn get_cn_theme_or_default() -> Readable<Theme>;
```

**Reading theme values — the state-binding rule:**

```rust
// ❌ WRONG: temporary State<Theme> is dropped before `read()` returns
let bg = use_cn_theme().read().background;

// ✅ CORRECT: bind the State to a local first
let state = use_cn_theme();
let bg = state.read().background;
```

`State::read()` returns a guard whose lifetime is tied to `&self`. If the `State` is a temporary, the guard dangles and
you get a compile error (or, with bad luck, a runtime panic).

---

## 5. Utilities

### 5.1 `cn!` macro

Concatenates class-name fragments, skipping empties and `None`.

```rust
cn!("px-4", "py-2", "rounded")
// → "px-4 py-2 rounded"

cn!("btn", if active { "btn-active" } else { "" })
// → "btn btn-active" (or "btn" when inactive)

cn!("input", Some("input-lg"), None::<&str>)
// → "input input-lg"

cn!()   // → ""

cn!("a", "b",)   // trailing comma OK → "a b"

// Nesting works
let inner = cn!("text-sm", "font-bold");
cn!("card", inner, "p-4")
// → "card text-sm font-bold p-4"
```

**Accepted argument types** (all implement `CnArg`):

- `&str`, `String`, `Cow<'_, str>`
- `&T`, `&mut T` where `T: CnArg`
- `Option<T>` where `T: CnArg` — `None` is skipped
- `Vec<T>`, `[T]`, `[T; N]` where `T: CnArg`
- A previous `cn!(...)` result

**Custom fragments:** implement `CnArg` for your own types:

```rust
impl CnArg for MyFragment {
    fn write_to(&self, out: &mut String) {
        if !out.is_empty() { out.push(' '); }
        out.push_str(&self.to_string());
    }
}
```

**Typical usage with `StyledString`:**

```rust
let classes = cn!("p-4 bg-card rounded-lg", if active { "ring-2 ring-primary" } else { "" });
Style::default ().style(classes)
```

### 5.2 `utils` module

Helper functions exported from `freyacn::utils` and `freyacn::prelude`:

```rust
// Replace colour's alpha with the given value (0.0–1.0)
pub fn color_with_alpha(color: Color, alpha: f32) -> Color;

// Linear blend between two colours
pub fn blend_colors(base: Color, blend: Color, ratio: f32) -> Color;

// Extract a solid Color from a Fill (returns TRANSPARENT for gradients)
pub fn get_color_from_fill(fill: Fill) -> Color;

// Multiply a colour's alpha channel
pub fn apply_opacity(c: Color, factor: f32) -> Color;
```

---

## 6. Patterns and recipes

### 6.1 Style layering

```text
Style::default()
    .style("<base>")              // base layer
    .merge(variant_style)         // variant layer
    .merge(user_style)            // user overrides — always win
```

**Implications:**

- Recomputing a base layer doesn't wipe the caller's overrides.
- Base is least specific; user overrides are most specific.

### 6.2 Custom extension trait

Because `StyleExt` is a trait with one required method, you can add your own chainable helpers on top for any
`StyleExt` implementor — `Style` included:

```rust
use freyacn::prelude::*;

pub trait CardExt {
    fn elevated(self) -> Self;
    fn outlined(self) -> Self;
}

impl<T: StyleExt> CardExt for T {
    fn elevated(self) -> Self {
        self.shadow_md().bg_card()
    }
    fn outlined(self) -> Self {
        self.style("border border-border")
    }
}

// Any StyleExt implementor gets the new methods
Style::default ().elevated().p_4()
Style::default ().outlined().p_4()
```

### 6.3 Implementing `StyleExt`

The canonical way to build a styling-friendly widget is a struct that owns a `Style` and delegates to it:

```rust
use freyacn::prelude::*;

#[derive(Clone, Default)]
pub struct MyWidget {
    pub style: Style,
    pub label: String,
}

impl StyleExt for MyWidget {
    fn get_style(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl MyWidget {
    pub fn render(&self) -> impl IntoElement {
        // Materialise the style against Freya's primitives here.
        freya::prelude::rect()
            .width(self.style.width.clone().unwrap_or_default())
            .height(self.style.height.clone().unwrap_or_default())
        // …
    }
}

// Usage
let w = MyWidget::default ()
.style("p-4 bg-card rounded-lg border border-border hover:bg-muted")
.label;
```

Because `Style` is a plain data struct, you can also apply styles directly and inspect them later:

```rust
let s = Style::default ().style("p-4 bg-card hover:bg-muted");
// or
let s = Style::default ().p_4().bg_card();
```

### 6.4 Materialising a `Style`

`Style` exposes helpers to fold the flat field soup into Freya's structured types:

```rust
impl Style {
    pub fn to_border(&self) -> Option<Border>;
    pub fn to_position(&self) -> Option<Position>;
    pub fn is_empty(&self) -> bool;
}
```

Use these when wiring `Style` into your own `IntoElement` implementation.

### 6.5 Responsive behavior

Freya has no breakpoint system. Use `use_viewport()` in Rust code and branch:

```rust
let size = use_viewport();
let is_wide = size.read().width > 768.0;

Style::default ().style( if is_wide { "flex-row" } else { "flex-col" })
```

### 6.6 Folding active states manually

If your widget owns hover/focus/active detection, compute the flags at render time and fold:

```rust
let mut active = StateFlags::NONE;
if theme.is_dark        { active = active.or(StateFlags::DARK); }
if * hovering.read()    { active = active.or(StateFlags::HOVER); }
if focus().is_focused() { active = active.or(StateFlags::FOCUS_VISIBLE); }

let resolved = my_widget.style.clone().with_active_states(active);
```

---

## 7. Gotchas and pitfalls

### Style parsing

| Symptom                                          | Cause                                                   | Fix                                                                                                                   |
|--------------------------------------------------|---------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| `overflow-hidden` does nothing                   | Not a token — parser only knows `overflow-clip`         | Use `.style("overflow-clip")` or `.overflow_clip()`                                                                   |
| `border-green-500` alone renders nothing         | Set colour but no width                                 | Use `border border-green-500` or `.border_width(2.0).border_color(c)`                                                 |
| `ring-2 ring-primary` parses but no ring appears | Freya has no ring primitive                             | Use `border-*` or `shadow-*` for visible rings                                                                        |
| Negative class `-ml-2` not applied               | Wasn't parsed before `0.2.0-alpha.1`                    | Update; current parser handles `-m-*`, `-translate-*`, `-rotate-*`, `-scale-*`, `-inset-*`, `-top-*`, `-left-*`, etc. |
| `grid-cols-3` doesn't render as a grid           | Parser only records `grid_cols`; rendering is up to you | Read `Style::grid_cols` / `Style::col_span` in your own component renderer                                            |

### Theming

| Symptom                                  | Cause                                                | Fix                                                                         |
|------------------------------------------|------------------------------------------------------|-----------------------------------------------------------------------------|
| `bg-card` looks transparent              | `card == background` in theme                        | Update to current theme; `card` is now one shade above `background`         |
| `bg-muted` indistinct from `bg-card`     | Wrong hierarchy in old theme                         | Current theme has `background < card < muted`                               |
| `use_cn_theme().read()` fails to compile | `State::read()` guard outlives the temporary `State` | `let state = use_cn_theme(); let v = state.read().X;`                       |
| Theme doesn't update on `.base_color()`  | Chained after `.theme_color()` — base resets accent  | Always chain `.base_color(...)` / `.stone()` **before** `.theme_color(...)` |

### Styles

| Symptom                                              | Cause                                                          | Fix                                                                                              |
|------------------------------------------------------|----------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| Style strings in a variant are not applied to leaves | `Style` is per-struct; text styles don't cascade automatically | Read `Style::text_color` / `font_size` / … in your widget's renderer and forward to the leaf     |
| Absolutely positioned children escape their parent   | Freya wraps every child in `WithPosition`                      | Use a `Position::Stacked` wrapper explicitly, or size the parent and rely on its `overflow-clip` |
| Animation appears to "jump then stick"               | Front-loaded easing on short durations                         | Use linear (default) — Freya's `use_animation` linear is fine                                    |

---

## Reference card

### Imports

```rust
use freyacn::prelude::*;
// Brings in Freya prelude, lucide icons, Style/StyleExt/StyledString/StyledColor,
// Theme/ThemeRegistry/ThemeHandle, theme hooks, cn!, and utility helpers.
```

### Common idioms

```rust
// Theme
use_init_cn_theme(Theme::dark().neutral().theme_color("blue"));
let theme_state = use_cn_theme();
let bg = theme_state.read().background;

// Layout (write directly into a Style, then forward to Freya primitives)
let s = Style::default ()
.flex_col()
.gap(8.0)
.p(16.0);

// Card-like surface
let s = Style::default ()
.style("bg-card text-card-foreground border border-border rounded-lg shadow-sm p-6");

// Tailwind string with state tokens
let s = Style::default ()
.style("cursor-pointer hover:bg-muted rounded-md p-3");

// Merge layering
let s = Style::default ()
.style("p-4 bg-card")
.merge(Style::default ().bg_green_50());  // user override wins

// Conditional classes
let classes = cn!("p-4 bg-card", if active { "ring-2 ring-primary" } else { "" });
let s = Style::default ().style(classes);

// Fold state manually inside a custom widget
let active = StateFlags::HOVER;
let s = my_widget.style.clone().with_active_states(active);

// Read the palette
let blue = freyacn::styling::colors::blue_500();
```