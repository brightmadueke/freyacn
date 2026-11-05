A JSON-driven design system for [Freya](https://github.com/marc2332/freya) desktop UIs. FreyaCN bundles three layers
that share one `Style` type and one inheritance chain: runtime-loaded JSON themes, a Tailwind-inspired utility
vocabulary, and HTML-like elements.

**Targets Freya `0.5.0-rc.13`.** Pre-release — the public API may break between `0.x` releases. Pin to a specific tag or
commit if you need stability.

## Features

- **Themes as JSON.** Every colour, spacing, radius, typography value, gradient, shadow, motion curve, and z-index lives
  in a JSON file loaded at runtime. Swap the file, change the whole UI.
- **Tailwind-style utilities.** Compose `Style` with fluent methods (`.bg(blue_500()).p_4()`) or with parsed class
  strings (`.style("bg-primary hover:bg-primary/90")`). Both produce the same value.
- **HTML-like elements.** `div`, `text`, `p`, `button`, `input`, `image`, `icon`, and friends, pre-wired to the style
  cascade.
- **Per-component tokens with state.** Themes can define `button.primary.hover.background`-style entries resolved
  through dotted paths, with a built-in state fallback chain.
- **Structural `has:` variants.** `has-[svg]:`, `has-[img]:`, `has-[:checked]:`, `has-[:selected]:`, `has-[:disabled]:`
  fire when a descendant matches — via automatic signals from `Icon` / `Image` / `Checkbox` / `Radio` / `Input`, or
  explicitly via `.with_has_svg(cond)` and friends.
- **Optional component presets.** `shadcn` and `material3` feature flags pull in pre-built component sets that plug into
  the same `Style` / theme / inheritance model.
- **Macros.** `#[component]`, `#[variant]`, `#[extensions]`, `#[struct_fields]`, and `cn!` for the common
  component-authoring patterns.

## Installation

### From crates.io (latest published version)

```toml
[dependencies]
freyacn = "0.2.2"
```

The current release is **0.2.2**. The next release (**0.2.3**) will target Freya `0.5.0-rc.13`.

### From a release tag (recommended for reproducibility)

Every published release is tagged on the repository (`v0.2.2`, `v0.2.3`, …). Pin to a tag to lock in an exact version,
including any commit that hasn't propagated to crates.io yet:

```toml
[dependencies]
freyacn = { git = "https://github.com/brightmadueke/freyacn", tag = "v0.2.2" }
```

Swap `v0.2.2` for whatever tag you want to target — e.g. `v0.2.3` once the next release is cut.

### From a specific commit

```toml
[dependencies]
freyacn = { git = "https://github.com/brightmadueke/freyacn", rev = "<sha>" }
```

Use this when you need an exact point-in-history that hasn't been tagged, or to bisect a regression.

### Cargo features

Core FreyaCN — themes, styling, elements, macros — is always present. Optional **component presets** are enabled
per-feature:

| Feature     | Enables                                                              | Default |
|-------------|----------------------------------------------------------------------|---------|
| `shadcn`    | shadcn/ui component set — reachable at `freyacn::shadcn`             | off     |
| `material3` | Material Design 3 component set — at `freyacn::material3`            | off     |
| `all`       | Both presets (`shadcn` + `material3`)                                | off     |
| `full`      | Both presets (`shadcn` + `material3`) — currently identical to `all` | off     |

```toml
freyacn = { version = "0.2.2", features = ["all"] }
# or, pinned to a tag:
freyacn = { git = "https://github.com/brightmadueke/freyacn", tag = "v0.2.2", features = ["all"] }
```

Individual flags compose — `features = ["shadcn"]` pulls in only shadcn, `features = ["material3"]` only Material 3.
See [Presets](#presets) for what each one provides.

## Quick start

```rust
use freyacn::prelude::*;

#[component]
fn App() {
    // Load `themes/default-dark.json`, install it as the active theme,
    // and provide a `ThemeHandle` to the subtree.
    let _themes = load_theme("themes/default-dark.json");

    div()
        .bg_background()
        .text_foreground()
        .p_6()
        .child(text("Hello, FreyaCN!"))
}
```

The four loaders — `load_theme`, `load_theme_from_json`, `load_themes`, `load_themes_with_default` — all parse the JSON,
install the chosen theme as the thread-local active theme, and provide both a reactive `State<Arc<Theme>>` and a
`ThemeHandle` into Freya's context. Descendants read them with `use_theme()` / `use_cn_theme()` / `use_theme_handle()`.

**Return types.** Plain functions returning elements use `-> impl IntoElement` and end with the element expression — no
`.into()`. Functions annotated with `#[component]` skip the return type entirely; the macro generates `impl IntoElement`
for you. All examples below follow this convention.

## Usage

### Loading a theme

Four entry points, depending on where the JSON lives and how many themes you ship.

**One file on disk** — pass a path (or anything that converts to one: `&str`, `String`, `&Path`, `PathBuf`):

```rust
let themes = load_theme("themes/default-dark.json");
```

**Embedded at compile time** — use `include_str!` with `load_theme_from_json`:

```rust
let themes = load_theme_from_json(include_str!("../themes/default-dark.json"));
```

**A directory of `.json` files** — auto-activates the first one in filesystem order:

```rust
let themes = load_themes("themes");
```

**A directory, with a specific theme active** — loads every file, then activates the one whose `id` matches:

```rust
let themes = load_themes_with_default("themes", "default-dark");
```

All four must be called inside a Freya component (they use `use_hook`). For non-Freya contexts — tests, CLI tooling,
previews — use `Theme::try_from_json_str` or `ThemeRegistry` directly:

```rust
use freyacn::theme::Theme;

let theme = Theme::try_from_json_str(include_str!("themes/dark.json")) ?;
let bg = theme.background_fill();
```

```rust
use freyacn::theme::ThemeRegistry;

let registry = ThemeRegistry::new();
registry.load(include_str!("themes/dark.json"));
registry.set_active("dark") ?;
```

### Reading tokens

```rust
use freyacn::prelude::*;

#[component]
fn Card() {
    let t = use_theme();  // Arc<Theme>

    div()
        .bg(t.card_fill())                // Fill (solid or gradient)
        .text_color(t.card_foreground())  // Color
        .radius(t.radius("lg"))           // f32
        .p(t.spacing("4"))                // f32
}
```

Named getters are generated for the standard semantic set — `background`, `foreground`, `card`, `primary`, `muted`,
`destructive`, `border`, `ring`, `link`, and so on. Each has a matching `_fill` variant that preserves gradients.

For tokens not covered by a named getter, use `t.color("key")` / `t.fill("key")`.

### Styling: builder methods

```rust
let style = Style::default ()
.bg(blue_500())
.text_color(cyan_50())
.p_4()
.rounded_md();
```

`Style::default()` seeds `background` and `text_color` from the active theme, so new elements inherit the theme baseline
without any explicit call. Explicit `.bg(..)` / `.text_color(..)` override that default.

`Style::bare()` gives a theme-independent `Style` (used internally as the patch layer, and useful for tests).
`Style::themed(&theme)` builds one from an explicit theme.

### Styling: class strings

```rust
let style = Style::default ().style(
"bg-primary text-primary-foreground p-4 rounded-md hover:bg-primary/90",
);
```

Colour utilities resolve their name in three steps:

1. The built-in Tailwind palette (`blue-500`, `neutral-950`, `white`, …).
2. The active theme's `semantic` map (`primary`, `primary-foreground`, `background`, …).
3. The active theme's border-only map (`ring`, `border`, `input`).

Class names are kebab-case; theme keys are camelCase. Both spellings resolve, so `text-primary-foreground` finds
`primaryForeground` in the theme JSON.

### State variants

Prefix a class with a state, or chain a builder method:

```rust
// Class string
Style::default ().style(
"bg-primary hover:bg-primary/90 \
     focus-visible:ring-[3px] focus-visible:ring-ring/50 \
     aria-invalid:border-destructive",
)

// Fluent
Style::default ()
.bg(blue_500())
.hover( | s| s.bg(blue_700()))
.focus( | s| s.bg(blue_600()))
```

Available prefixes: `dark:`, `hover:`, `focus:`, `focus-visible:`, `focus-within:`, `active:`, `disabled:`, `checked:`,
`selected:`, `aria-invalid:`, `aria-checked:`, `aria-disabled:`, `aria-expanded:`, `aria-pressed:`, `aria-readonly:`,
`aria-required:`, `aria-selected:`, `aria-hidden:`, plus the structural `has-*` family — `has-[svg]:`, `has-[img]:`,
`has-[:checked]:`, `has-[:selected]:`, `has-[:disabled]:`.

Prefixes compose left to right: `dark:aria-invalid:bg-red-500` fires only when both are active. Precedence between
overlapping variants is fixed — see the `styling` module docs for the full ordering.

`aria-*:` variants only fire when the corresponding state is set. Interactive elements implement `HasSemanticEvents`,
which exposes `.with_aria_invalid(..)`, `.with_aria_checked(..)`, `.with_aria_disabled(..)`, `.with_aria_expanded(..)`,
`.with_aria_pressed(..)`, `.with_aria_readonly(..)`, `.with_aria_required(..)`, `.with_aria_selected(..)`,
`.with_aria_hidden(..)`:

```rust
input(value)
.with_aria_invalid( ! form_is_valid)
.style("aria-invalid:border-destructive")
```

`with_disabled` / `with_checked` / `with_selected` set both the native flag and the aria state in one call.

### `has:` variants

`has-*:` classes fire when a descendant matches a signal:

- `has-[svg]:` — contains an `Icon`
- `has-[img]:` — contains an `Image`
- `has-[:checked]:` — contains a checked `Checkbox` / `Radio`
- `has-[:selected]:` — contains a selected descendant
- `has-[:disabled]:` — contains a disabled descendant

Two ways to drive them:

**Automatic signaling** — a matching child (`Icon` emits `Svg`, `Image` emits `Img`, `Checkbox` / `Radio` emit `Checked`
when checked) signals the nearest `has:`-aware ancestor:

```rust
button()
.style("has-[svg]:pl-3")
.child(icon(lucide::check()))
.child(text("Continue"))
```

**Explicit `.with_has_*()`** — synchronous, no one-frame delay, resets when `false`:

```rust
button()
.with_has_svg(show_icon)
.has_svg( | s| s.pl_3())
.maybe_child(show_icon.then( | | icon(lucide::check())))
```

Automatic signaling carries three caveats: one-frame delay on mount, no unmount cleanup (use explicit `.with_has_*()`for
conditional children), and immediate-children-only unless intermediates also declare `has:` classes. See the
`styling::has` module docs for the full mechanism.

### Component tokens

Themes can carry free-form `components` JSON, read through state-aware dotted-path accessors:

```json
"components": {
  "button": {
    "primary": {
      "background": "{semantic.primary}",
      "foreground": "{semantic.primaryForeground}",
      "hover": {
        "background": "{neutral.200}"
      },
      "active": {
        "background": "{neutral.300}"
      }
    }
  }
}
```

```rust
let t = use_theme();
t.state_fill("button.primary", ComponentState::Hover, "background");
```

`state_fill` tries `button.primary.hover.background` first, then the state fallback chain (`focus-visible` → `focus`,
`pressed` → `active`, `open` → `active`, `indeterminate` → `checked`), then the base `button.primary.background`.
`state_color` and `state_radius` behave identically.

Raw and typed accessors for a specific path are available as `component`, `component_color`, `component_fill`, and
`component_radius`.

### Shadows

Box-shadow and text-shadow presets mirror Tailwind v4.1. Size presets **replace** the current shadow layers (matching
CSS `box-shadow:` / `text-shadow:` semantics); use the base `.shadow(..)` / `.text_shadow(..)` methods to append custom
layers.

```rust
// Box shadow
div()
.shadow_sm()                  // small
.shadow_lg()                  // large — replaces `sm`
.shadow_color(blue_500());    // re-tint

// Custom multi-layer box shadow
div()
.shadow(Shadow::new().y(2.0).blur(4.0).color(black().opacity(0.1)))
.shadow(Shadow::new().y(8.0).blur(16.0).color(black().opacity(0.05)));

// Text shadow
text("Hi").text_shadow_md();
text("Hi").text_shadow_lg().text_shadow_color(primary());
```

`shadow_color` / `text_shadow_color` recolor every currently-registered layer — they're meant to be chained *after* a
size preset, matching Tailwind's `shadow-{size} shadow-{color}` pairing. Both require a **solid** tint and panic on
gradients.

**Per-element support:**

| Element                              | Box shadow            | Text shadow            |
|--------------------------------------|-----------------------|------------------------|
| `div`, `button`, `checkbox`, `radio` | ✅                    | ❌ (no glyphs)         |
| `input`                              | ✅ (outer `Div`)      | ✅ (inner `paragraph`) |
| `text`, `p`                          | ❌ (no `.shadow(..)`) | ✅                     |
| `icon`, `image`                      | ❌                    | ❌                     |

Wrap in a `div().shadow_*()` when an element doesn't support the shadow you need.

> Freya's `TextShadow::blur_sigma` is a **standard deviation**, not the CSS `blur-radius`. The presets apply
> `sigma = radius / 2` (the SVG spec conversion), so they match Tailwind's visual output.

## Theme schema

A theme is a single JSON document. Four fields are required; everything else is optional and falls back to sensible
defaults.

### Required fields

| Field        | Type                      | Notes                                           |
|--------------|---------------------------|-------------------------------------------------|
| `id`         | `string`                  | Stable identifier; used by `set_active(id)`.    |
| `name`       | `string`                  | Display name shown in pickers.                  |
| `appearance` | `"light"` \| `"dark"`     | Informational; drives sun/moon icons, branches. |
| `semantic`   | `map<string, TokenValue>` | Core semantic tokens (see below).               |

`semantic` should contain the standard set — `background`, `foreground`, `card`, `cardForeground`, `popover`,
`popoverForeground`, `primary`, `primaryForeground`, `secondary`, `secondaryForeground`, `muted`, `mutedForeground`,
`accent`, `accentForeground`, `destructive`, `destructiveForeground`, `border`, `input`, `ring`. Missing entries produce
validation warnings on stderr but never fail the load. The bundled `themes/default-dark.json` and
`themes/default-light.json` include the full recommended set and are the best starting point.

### Optional fields

| Field        | Type                             | Default | Notes                                                                            |
|--------------|----------------------------------|---------|----------------------------------------------------------------------------------|
| `version`    | `string`                         | `""`    | Semantic version string.                                                         |
| `author`     | `string`                         | none    | Displayed in pickers/tooltips.                                                   |
| `palette`    | `map<"family.step", hex>`        | `{}`    | Overrides the built-in Tailwind palette.                                         |
| `colors`     | `map<"dotted.path", TokenValue>` | `{}`    | Flat component tokens. Overlaid onto `list`/`menu`/`table`.                      |
| `components` | any JSON                         | `null`  | Free-form nested tokens. Not validated at load time.                             |
| `list`       | `ListConfig`                     | `{}`    | Structured list theme (metrics + paint).                                         |
| `menu`       | `MenuConfig`                     | `{}`    | Structured menu theme.                                                           |
| `table`      | `TableConfig`                    | `{}`    | Structured table theme.                                                          |
| `radius`     | `map<string, f32>`               | `{}`    | Named radius scale (`"sm": 4`, `"lg": 8`, …).                                    |
| `spacing`    | `map<string, f32>`               | `{}`    | Named spacing scale (`"1": 4`, `"4": 16`, …).                                    |
| `typography` | `TypographyConfig`               | `{}`    | Font stacks, sizes, weights (see below).                                         |
| `shadows`    | `map<string, string>`            | `{}`    | Named CSS shadow strings. Declarative — see Notes.                               |
| `motion`     | `MotionConfig`                   | `{}`    | Duration (`{ "fast": 150 }`) + easing (`{ "standard": "…" }`). Declarative only. |
| `zIndex`     | `map<string, i32>`               | `{}`    | Named z-indices (`"modal": 1000`).                                               |

### `TokenValue`

Every colour value accepts one of these forms:

| Form                         | JSON                                                                     |
|------------------------------|--------------------------------------------------------------------------|
| Hex                          | `"#ff8800"`, `"#fff"`, `"#fff8"` (with alpha)                            |
| Palette reference            | `"{blue.500}"`                                                           |
| Palette reference with alpha | `"{neutral.50}/10"` — `/NN` is 0–100, overwrites source alpha            |
| Semantic reference           | `"{semantic.primary}"` (valid inside `semantic`, `components`, `colors`) |
| Transparent                  | `"transparent"`                                                          |
| Linear gradient              | `{ "linear": { "angle": 135, "stops": [Stop, …] } }`                     |
| Radial gradient              | `{ "radial": { "center": [0.5, 0.5], "radius": 1.0, "stops": […] } }`    |
| Conic gradient               | `{ "conic": { "angle": 0, "center": [0.5, 0.5], "stops": […] } }`        |

A `Stop` is `{ "color": TokenValue, "at": 0.0..=100.0 }`. Gradient `center` and `radius` are parsed but currently
ignored — Freya's gradient types only consume angle + stops.

### `TypographyConfig`

```json
{
  "fontFamily": {
    "sans": "Inter, system-ui, sans-serif",
    "mono": "JetBrains Mono"
  },
  "fontSize": {
    "sm": [
      12,
      16
    ],
    "base": [
      16,
      24
    ],
    "lg": [
      18,
      28
    ]
  },
  "fontWeight": {
    "normal": 400,
    "medium": 500,
    "bold": 700
  }
}
```

`fontSize` values are `[size_px, line_height_px]` pairs. `line_height` is parsed but the `text_*` shortcuts currently
only use `size`.

### `ListConfig` / `MenuConfig` / `TableConfig`

Each structured block accepts optional paint + metric keys. Individual `colors` entries (`"list.row.hover"`,
`"menu.item.selected"`, …) override the corresponding block field, so a minimal block for metrics + `colors` for paint
is a valid split.

| Config  | Paint keys                                                                                                                                         | Metric keys                                                                      |
|---------|----------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| `list`  | `background`, `border`, `rowBackground`, `rowHover`, `rowSelected`, `rowActive`, `rowForeground`, `rowSelectedForeground`, `rowDisabledForeground` | `rowHeight` (32), `rowPaddingX` (12), `rowPaddingY` (6), `radius` (6), `gap` (0) |
| `menu`  | `background`, `border`, `itemHover`, `itemSelected`, `itemForeground`, `separator`                                                                 | `radius` (6), `itemPaddingX` (10), `itemPaddingY` (6)                            |
| `table` | `background`, `border`, `headerBackground`, `headerForeground`, `rowHover`, `rowSelected`                                                          | `cellPaddingX` (12), `cellPaddingY` (8), `radius` (6)                            |

### Full example

```json
{
  "id": "my-app-dark",
  "name": "My App Dark",
  "appearance": "dark",
  "version": "1.0.0",
  "author": "you",
  "palette": {
    "brand.500": "#7c3aed",
    "brand.600": "#6d28d9"
  },
  "semantic": {
    "background": "{neutral.950}",
    "foreground": "{neutral.50}",
    "primary": "{brand.500}",
    "primaryForeground": "{neutral.50}",
    "muted": "{neutral.900}",
    "mutedForeground": "{neutral.400}",
    "border": "{neutral.50}/10",
    "ring": "{brand.500}"
  },
  "components": {
    "button": {
      "primary": {
        "background": "{semantic.primary}",
        "foreground": "{semantic.primaryForeground}",
        "hover": {
          "background": "{brand.600}"
        },
        "active": {
          "background": "{brand.600}"
        }
      }
    }
  },
  "radius": {
    "sm": 4,
    "md": 6,
    "lg": 8
  },
  "spacing": {
    "1": 4,
    "2": 8,
    "4": 16
  },
  "typography": {
    "fontFamily": {
      "sans": "Inter, system-ui, sans-serif"
    },
    "fontSize": {
      "sm": [
        12,
        16
      ],
      "base": [
        16,
        24
      ]
    },
    "fontWeight": {
      "normal": 400,
      "medium": 500,
      "bold": 700
    }
  },
  "zIndex": {
    "modal": 1000,
    "tooltip": 1100
  }
}
```

## Switching themes

```rust
let themes = use_theme_handle();
themes.set_active("default-light") ?;   // returns Result<(), ThemeError>
```

`ThemeHandle::set_active` updates three things together:

- The thread-local active theme, so the next `Style::default()` reads the new colours.
- The reactive `State<Arc<Theme>>`, so any component subscribed via `use_cn_theme()` re-renders.
- The registry's internal active id.

All three are required: `Style::default()` seeds new elements with the right baseline; `use_cn_theme()` gives existing
elements a reason to rebuild their style.

To enumerate available themes (for a picker, settings panel, etc.):

```rust
for info in themes.theme_infos() {
// info.id, info.name, info.appearance, info.version, info.author
}
```

`theme_infos()` is sorted by name, then id, and clones only five small fields per theme — cheap enough to call every
render.

> The lower-level `ThemeRegistry::set_active` only updates the registry's active id. Use `ThemeHandle::set_active`inside
> components.

## Hot reload

```rust
themes.reload_dir("themes");
```

Re-reads every `.json` in the directory, replaces entries with matching ids, and refreshes the active theme if it
changed on disk. New files are added; removed files are not pruned (call `registry.remove(id)` first if you need that).

There is no watcher thread. Wire `reload_dir` to `notify`, a keyboard shortcut, or your dev-tool of choice.

## Extensions

Two traits extend `Style`: `StyledExt` (numeric + keyword shortcuts) and `ThemedExt` (theme-path helpers). Both work on
any element, and every method returns `Self` so they chain.

### `StyledExt` — Tailwind-shaped shortcuts

Every numeric scale accepts the Tailwind steps (`0`, `1`, `2`, `3`, `4`, `6`, `8`, …) and the shorthand keywords
(`full`, `auto`, `px`, `screen`). All of these also work as class strings.

| Category           | Methods                                                                                                                                                                                                                                                                                                                                                                                              |
|--------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Spacing            | `p_N`, `px_N`, `py_N`, `pt_N`, `pr_N`, `pb_N`, `pl_N`, `m_N`, `mx_N`, `my_N`, `mt_N`, `mr_N`, `mb_N`, `ml_N`, `gap_N`, `gap_x_N`, `gap_y_N`                                                                                                                                                                                                                                                          |
| Sizing             | `w_N`, `h_N`, `size_N`, `min_w_N`, `max_w_N`, `min_h_N`, `max_h_N`, `w_full`, `h_full`, `w_auto`, `h_auto`, `w_px`, `h_px`, `w_screen`, `h_screen`                                                                                                                                                                                                                                                   |
| Layout             | `flex`, `flex_col`, `flex_row`, `flex_wrap`, `flex_nowrap`, `justify_start`, `justify_end`, `justify_center`, `justify_between`, `justify_around`, `justify_evenly`, `items_start`, `items_end`, `items_center`, `items_baseline`, `items_stretch`                                                                                                                                                   |
| Radius             | `rounded_{none,xs,sm,md,lg,xl,2xl,3xl,full}`, `rounded_t_*`, `rounded_r_*`, `rounded_b_*`, `rounded_l_*`, `rounded_tl_*`, `rounded_tr_*`, `rounded_br_*`, `rounded_bl_*`                                                                                                                                                                                                                             |
| Border             | `border`, `border_0`, `border_2`, `border_4`, `border_8`, `border_[Npx]`, `border_n_*`, `border_s_*`, `border_e_*`, `border_w_*`                                                                                                                                                                                                                                                                     |
| Ring               | `ring`, `ring_0`, `ring_1`, `ring_2`, `ring_4`, `ring_8`, `ring_[Npx]` (maps to border — see Notes)                                                                                                                                                                                                                                                                                                  |
| Typography         | `text_{xs,sm,base,lg,xl,2xl,3xl,…}`, `font_{thin,light,normal,medium,semibold,bold,extrabold,black}`, `font_sans`, `font_serif`, `font_mono`, `italic`, `not_italic`, `underline`, `line_through`, `uppercase`, `lowercase`, `capitalize`, `normal_case`, `truncate`, `text_left`, `text_center`, `text_right`                                                                                       |
| Effects            | `opacity_{0,5,10,20,25,30,40,50,60,70,75,80,90,95,100}`, `rotate_{0,1,2,3,6,12,45,90,180}`, `rotate_neg_{1,2,3,6,12,45,90,180}`, `scale_{0,50,75,90,95,100,105,110,125,150,200}`, `blur_{none,2xs,xs,sm,md,lg,xl,2xl,3xl}`, `backdrop_blur_{none,2xs,xs,sm,md,lg,xl,2xl,3xl}`, `overflow_{none,clip}`                                                                                                |
| Shadows            | `shadow_{2xs,xs,sm,md,lg,xl,2xl,inner}`, `shadow_none`, `shadow_color(..)`                                                                                                                                                                                                                                                                                                                           |
| Text shadows       | `text_shadow_{2xs,xs,sm,md,lg}`, `text_shadow_none`, `text_shadow_color(..)`                                                                                                                                                                                                                                                                                                                         |
| Z-index            | `z_0`, `z_10`, `z_20`, `z_30`, `z_40`, `z_50`, `z_auto`                                                                                                                                                                                                                                                                                                                                              |
| Colours (raw)      | `bg_<family>_<step>`, `text_<family>_<step>`, `border_<family>_<step>` — every Tailwind palette family                                                                                                                                                                                                                                                                                               |
| Colours (semantic) | `bg_<token>`, `text_<token>`, `border_<token>` — `bg_background`, `text_foreground`, `bg_primary`, `text_primary_foreground`, `border_border`, `ring_ring`, …                                                                                                                                                                                                                                        |
| Variants           | `dark(…)`, `hover(…)`, `focus(…)`, `focus_visible(…)`, `focus_within(…)`, `active(…)`, `disabled(…)`, `checked(…)`, `selected(…)`, `has_svg(…)`, `has_img(…)`, `has_checked(…)`, `has_selected(…)`, `has_disabled(…)`, `aria_invalid(…)`, `aria_checked(…)`, `aria_disabled(…)`, `aria_expanded(…)`, `aria_pressed(…)`, `aria_readonly(…)`, `aria_required(…)`, `aria_selected(…)`, `aria_hidden(…)` |

### `ThemedExt` — theme-path helpers

Resolve a token path against an `Arc<Theme>` without unwrapping `Option` at every call site:

```rust
let t = use_theme();

div()
.bg_token( & t, "primary")             // semantic → Fill
.text_token( & t, "primaryForeground") // semantic → Fill
.radius_token( & t, "lg")              // radius map → f32
.p_token( & t, "4")                    // spacing map → f32
.gap_token( & t, "2")
.hover({
let t = t.clone();
move | s | s.bg_component( & t, "button.primary", ComponentState::Hover, "background")
})
```

`*_token` variants read the semantic map and panic on missing keys (fail-fast: a typo should not silently render a black
box). `*_component` variants read the nested `components` tree with the same per-state fallback as `Theme::state_fill`.

The variant closures capture the theme by `Arc` (cheap — the handle is already shared), so they always see the current
theme when the closure runs.

## Elements

```rust
use freyacn::prelude::*;

#[component]
fn Toolbar() {
    div()
        .style("flex-col gap-2 p-4")
        .child(text("Title").style("text-lg font-bold"))
        .child(
            button()
                .style("bg-primary text-primary-foreground p-2 rounded-md")
                .child(text("Continue")),
        )
}
```

Elements share the same `Style`, variant, and inheritance model as everything else, so a utility that works on a raw
`div` works on a `button` here.

If you need a plain helper function rather than a `#[component]`, give it `-> impl IntoElement` and end with the element
expression:

```rust
fn title(label: &str) -> impl IntoElement {
    text(label).style("text-lg font-bold")
}
```

## Presets

Two optional component sets ship as feature-gated sibling crates. Each re-exports through the `freyacn` facade at its
own module path, and each brings its own default theme on top of the standard FreyaCN JSON themes.

### shadcn

Enabled by the `shadcn` feature. Re-exports `freyacn_shadcn` — pre-built shadcn/ui components, styled through FreyaCN's
theme system so they inherit the active theme's tokens (`primary`, `primaryForeground`, `muted`, `border`, …) without
per-component configuration.

```rust
use freyacn::prelude::*;
use freyacn::shadcn::{Button, Card};

#[component]
fn App() {
    let _themes = load_theme("themes/default-dark.json");
    Card()
        .child(text("Welcome"))
        .child(Button().child(text("Continue")))
}
```

The shadcn component set follows the same `Style` / variant / inheritance model as everything else, so all the utilities
documented above work on shadcn components too — `.style("hover:bg-primary/90")`, `.shadow_md()`, `.has-[svg]:pl-3`, and
so on.

### material3

Enabled by the `material3` feature. Re-exports `freyacn_material3` — Material Design 3 components with the MD3 tonal
palette (`primaryContainer`, `surfaceVariant`, `onSurface`, …) exposed as semantic tokens in the corresponding theme.

```rust
use freyacn::prelude::*;
use freyacn::material3::FilledButton;

#[component]
fn App() {
    let _themes = load_theme("themes/default-dark.json");
    FilledButton::default().child(text("Continue"))
}
```

The same style utilities, the same theme layering, and the same inheritance rules apply to Material 3 components.

### Combining presets

`features = ["all"]` (or `features = ["full"]`) enables both. The two sets are independent — shadcn components don't
need material3 enabled, and vice versa — so you can pull in only what you use and keep compile times down.

## Macros

### `#[variant]`

Turns an enum into a class-string provider. Each variant carries a `#[variant("…")]` attribute holding a class string,
and the generated `style_str()` returns it. `resolve()` folds the class string onto `Style::bare()` so variants compose
as sparse patches.

```rust
use freyacn::prelude::*;

#[variant]
#[derive(Clone, Copy, Default)]
enum ButtonVariant {
    #[default]
    #[variant("bg-primary text-primary-foreground hover:bg-primary/90")]
    Default,
    #[variant("bg-destructive text-destructive-foreground hover:bg-destructive/90")]
    Destructive,
    #[variant("bg-secondary text-secondary-foreground hover:bg-secondary/80")]
    Secondary,
}

ButtonVariant::Default.style_str();
// → "bg-primary text-primary-foreground hover:bg-primary/90"

ButtonVariant::Default.resolve();
// → Style starting from Style::bare(), with the class string folded on top
```

Implement `ComponentVariant` (which `#[variant]` does) to consume variants from `#[component]` below.

### `#[component]`

Generates a component function that folds its arguments into a single `Style`, merges variant and size on top, and
returns `impl IntoElement`. The `ext(…)` attribute adds extra parameters — `ext(children)` gives the component Freya's
`children` parameter, which it can forward with `.children(children.clone())`.

```rust
#[component]
fn ActionButton(label: String) {
    button()
        .style("px-4 py-2 rounded-md bg-primary text-primary-foreground")
        .child(text(label))
}

// With `ext(children)` and a variant argument:
#[component(ext(children))]
fn VariantButton(variant: ButtonVariant) {
    button()
        .style("px-4 py-2 rounded-md")
        .merge_style(variant.resolve())
        .children(children.clone())
}
```

The fold order is: `Style::default()` (theme baseline) → variant → size → `self.style` (whatever the caller chained).
See "How the layers fit" for why each layer starts where it does, and for how the variant list accumulates across that
chain.

### `#[struct_fields]`

**`#[struct_fields]` is not a standalone attribute.** It's a modifier for `#[component]` that injects extra fields into
the generated props struct. It must be placed **below** `#[component(...)]` — attributes are processed top-to-bottom,
and `#[component]` consumes `#[struct_fields]` to build the struct. Placing it above produces a compile error with a
helpful message.

```rust
#[component]
#[struct_fields(tooltip: Option<MyTooltip>, on_dismiss: EventHandler<()>)]
fn Card(title: String) {
    // `self.tooltip` and `self.on_dismiss` are available inside the
    // component body alongside the declared parameters.
}
```

Each entry follows `field_name: Type` syntax. Multiple fields are comma-separated. Callers construct the component with
`.tooltip(..)` / `.on_dismiss(..)` builder methods generated from the injected fields, on top of the ones derived from
the function parameters.

Combining with `ext(…)`:

```rust
#[component(ext(children))]
#[struct_fields(tooltip: Option<MyTooltip>)]
fn Badge(label: String) {
    div()
        .style("inline-flex items-center rounded-full px-2 py-0.5")
        .children(children.clone())
        .child(text(label))
}
```

### `cn!`

Joins class strings, skipping `None` / `false` entries:

```rust
let classes = cn!(
    "px-4 py-2",
    active.then_some("bg-primary"),
    None::<&str>,
    "rounded-md",
);
// → "px-4 py-2 bg-primary rounded-md"
```

Accepts anything that implements `CnArg`: string literals, `Option<&str>`, `bool`, `&str`, `String`.

## How the layers fit

Everything flows through one `Style` type and one rule:

**`Style::default()` is the theme layer. `Style::bare()` is the patch layer.**

- `Style::default()` seeds `background` and `text_color` from the active theme.
- `Style::bare()` starts empty.
- `merge_style` copies a scalar field (colour, padding, border, …) only if it differs from `bare()`, so patches stay
  sparse.
- `merge_style` **accumulates** variants — every layer in the fold contributes its own `hover:` / `focus-visible:` /
  `has-[svg]:` / `aria-invalid:` registrations, and later layers override earlier ones only when they register the
  *same* condition.
- `#[variant]` and `.style("…")` fold onto `Style::bare()`.
- `#[component]`'s fold starts from `Style::default()` and merges variant, size, and `self.style` on top.

Starting the overlay layers from `bare()` is what lets a variant that only sets `text_color` merge cleanly over a theme
default that also sets `text_color` — the variant wins, because it's a non-bare patch and the theme default is a bare
field.

**Variant accumulation is what makes multi-layer components work.** A shadcn `Button` registers variants in three
places:

- The base class string: `focus-visible:ring-[3px]`, `disabled:opacity-50`, `aria-invalid:border-destructive`,
  `dark:aria-invalid:ring-destructive/40`.
- `ButtonVariant::Default`: `hover:bg-primary/90`.
- `ButtonSize::Default`: `has-[svg]:px-3`.

Before the fix, each successive `merge_style` **replaced** the variant list, so only the last non-empty pass survived —
the `ButtonSize` `HasSvg` patch wiped out `Hover`, `FocusVisible`, `Disabled`, `AriaInvalid`, and the `Dark` compound.
`Div::build_element` then saw `variants.has(Hover) == false`, skipped wiring pointer handlers, and hover never fired.
With `StyleVariants::extend`, all four layers contribute, `has(Hover)` is `true`, pointer handlers are wired, and every
variant fires as expected.

## Notes

- `ring-*` utilities map onto Freya's `Border`; there is no separate ring model. You cannot have both a visible border
  and a separate ring on the same element — wrap in a `div` if you need both, or use a spread shadow
  (`Shadow::new().spread(3.0)`) for a true offset ring.
- `radial` and `conic` gradients accept `center` and `radius` fields for forward compatibility, but Freya's gradient
  types currently ignore them.
- The `shadows` JSON block holds CSS shadow strings and is **declarative** — nothing in `StyledExt` reads it. Apply box
  shadows and text shadows with `.shadow_*()` / `.text_shadow_*()`, which use the Tailwind v4.1 presets directly.
- `motion` tokens are parsed and exposed through `theme.motion_duration(..)` and `theme.motion_easing(..)`, but nothing
  in `StyledExt` consumes them yet.
- `get_theme_or_default()` is the non-panicking form of `use_theme()`: it falls back to a cached `Theme::empty_light()`
  when no provider is in scope.
- `has:` variants have documented limitations — one-frame delay on mount, no unmount cleanup, immediate-children-only
  for grandparents. Use `.with_has_*()` for conditional children or anywhere the delay is visible. See the`styling::has`
  module docs for details. If `has-*:` classes aren't firing at all, check that the element's style registered the
  variant (via `.style("has-[svg]:…")` or `.has_svg(…)`) — this is now independent of whether other layers also
  registered variants, but you still need at least one `has-*:` class on the element for the mechanism to engage.
- `all` and `full` are currently identical. If you intend them to diverge (e.g. `full` = `all` + heavyweight
  integrations), change the Cargo.toml first — the README table will track it.

## License

MIT.

## Acknowledgements

- [Freya](https://github.com/marc2332/freya) — GUI framework.
- [Tailwind CSS](https://tailwindcss.com) — utility-first approach.
- [gpui-kit](https://github.com/zed-industries/zed) — theme-token model (`semantic` → `components` → `state` with
  fallback).