# `freyacn-macros`

> ⚠️ **Work in progress.** This crate is under active development and its
> public API is **not yet stable**. Breaking changes may land in any release.
> The shapes shown below are the current target, not a compatibility promise.
> Expect rough edges, missing features, and doc/code drift.

Procedural macros for the [`freyacn`][freyacn] crate. They turn plain
functions and enums into styled, prop-driven components for
[Freya][freya], with complete generated documentation.

Everything described here is available through the prelude:

```rust
use freyacn::prelude::*;
```

The prelude re-exports the macros and the runtime surface they emit — you
do **not** need a direct `freyacn-macros` dependency.

---

## Relationship to `freyacn`

**`freyacn-macros` is an internal dependency of `freyacn`.** It is not meant
to be used on its own, for two reasons:

1. **`freyacn` re-exports every macro.** Everything documented here is
   available through the prelude:
   ```rust
   use freyacn::prelude::*;
   // component, variant, struct_fields, extensions
   ```
   Adding `freyacn-macros` to your `Cargo.toml` alongside `freyacn` would
   pull in the same code twice and, worse, risk pulling *different*
   versions.

2. **The macros emit paths to `freyacn` types.** The generated code refers
   to `::freyacn::prelude::Component`, `::freyacn::prelude::Style`, and so
   on. Without `freyacn` in the dependency graph, expansion would fail with
   "cannot find crate `freyacn`".

The only reason to depend on `freyacn-macros` directly is if you are
forking the macro implementation itself. For every other use case, use
`freyacn`.

---

## The four attributes

| Attribute               | Purpose                                                                                                                                                                       |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `#[component(...)]`     | Turns a function into a component struct, a PascalCase constructor fn, and a companion macro. `props`, `ext`, and `variants` groups live inside it.                           |
| `#[variant]`            | Turns an enum into a style-resolving variant type with `From<&str>` / `From<String>`, `.style_str()`, `.resolve()`, and an `impl freyacn::prelude::Variant`.                  |
| `#[struct_fields(...)]` | Placed below `#[component]`; adds internal storage fields the user's own trait impls manage. No setters are generated.                                                        |
| `#[extensions(...)]`    | Standalone attribute that opts any struct into freyacn's built-in extension bundles. The `ext(...)` group inside `#[component]` is the same feature for generated components. |

---

## Installation

```toml
[dependencies]
freyacn = "0.2"
```

That's the whole list. `freyacn-macros` is a transitive dependency, not a
direct one.

```rust
use freyacn::prelude::*;
```

---

## Quick start

```rust
use freyacn::prelude::*;

/// Visual variants for a button.
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

#[component(
    props( badge: String = "new".to_string()),
    ext(style, children, key, event_handlers),
    variants(
        "px-4 py-2 rounded-lg",
        variant: ButtonVariant = ButtonVariant::Default,
    ),
)]
pub fn Button() {
    Div()
        .children(children.clone())
        .event_handlers(event_handlers.clone())
        .merge_style(style.clone())
        .child(label().text("Save"))
}
```

> **No return type needed.** The macro generates the `impl IntoElement`
> signature for the component's `render` method. Write the function body as
> the expression you want rendered, and skip both the `-> impl IntoElement`
> annotation and any trailing `.into()`.

> **Components don't have a `label` prop by default.** Content is passed as
> children — `.child(label().text(...))` — or via a `label: String` function
> parameter if you want `Button().label("Save")` (function parameters become
> required props).

Three equivalent ways to construct the component above:

```rust
Button().variant(ButtonVariant::Destructive);         // typed
Button().variant("destructive");                      // &str
Button!(variant = "destructive");                     // macro
```

For what `style`, `bg_primary`, `p_4`, and other `StyleExt` helpers do, and
for the class-string token catalogue, see the `freyacn` crate README. This
document covers only the *macro* surface.

---

## `#[component(...)]`

Every component is written as a normal Rust function. The attribute groups
declare where each field comes from; the source decides its kind.

### Four field sources

| Source                          | Kind          | Field type       | Init             | Body sees                   | Setter accepts              |
|---------------------------------|---------------|------------------|------------------|-----------------------------|-----------------------------|
| `fn Card(name: String)`         | **required**  | `Option<String>` | `None`           | `&String` (panics if unset) | `impl Into<String>`         |
| `props(name: String)`           | **optional**  | `Option<String>` | `None`           | `&Option<String>`           | `impl Into<Option<String>>` |
| `props(name: String = "x")`     | **defaulted** | `String`         | `"x"`            | `&String`                   | `impl Into<String>`         |
| `variants(name: T = default)`   | **variant**   | `T`              | `default`        | `&T`                        | `impl Into<T>`              |
| `#[struct_fields(name: T)]`     | **internal**  | `Option<T>`      | `None`           | `&Option<T>`                | *(no setter)*               |
| `#[struct_fields(name: T = x)]` | **internal**  | `T`              | `x`              | `&T`                        | *(no setter)*               |
| `ext(children, style, …)`       | **extension** | *trait-specific* | *trait-specific* | *trait-specific*            | *(no setter)*               |

You **never** write `Option<T>` yourself for an optional prop or a struct
field — writing `name: String` is enough; the macro wraps it.

### Grammar

```text
#[component(
    props(    prop1: Type1, prop2: Type2 = default_expr, … ),
    ext(      extension1, extension2, … ),
    variants( [ "base style string", ]
              variant: EnumType = EnumType::Default,
              … ),
)]
```

Groups may appear in any order, and each is optional. `props` accepts
comma-separated declarations of the shape:

```text
field := ident ":" type [ "=" expr ]
```

`variants` accepts the same shape, preceded by an optional string literal
for the base style:

```text
variants_list := [ string_literal "," ]
                 field { "," field } [ "," ]
```

At most **one** base-style string per `variants(...)` group. A second one is
a compile error.

### Return type

The component function does **not** need a return type annotation, and does **not** need a trailing `.into()`. The macro
wraps the body in a
`Component::render` method that returns `impl IntoElement`, so the last
expression of the body just needs to implement `IntoElement`:

```rust
#[component]
fn Greeting(name: String) {
    label().text(format!("Hello, {name}!"))   // no `-> …`, no `.into()`
}
```

### Required props — function parameters

```rust
#[component]
fn Greeting(name: String) {
    label().text(format!("Hello, {name}!"))
}

Greeting().name("Ada");           // OK
// Greeting();                    // compiles, panics at render
// Greeting().name(None);         // type error — intentional
```

### Optional props — `props(name: T)`

```rust
#[component(props( subtitle: String))]
fn Heading(subtitle: String) {
    match subtitle {
        Some(s) => label().text(s.as_str()),
        None => Div(),
    }
}

h.subtitle("hi");                       // &str  → Some("hi".into())
h.subtitle(String::from("hi"));         // String → Some
h.subtitle(Some("hi".to_string()));     // Option<String>
h.subtitle(None);                       // Option<String>
```

### Defaulted props — `props(name: T = expr)`

```rust
#[component(props(
    size:   f32    = 16.0,
    weight: String = "normal".to_string(),
))]
fn Text(size: f32, weight: String) {
    label().text(format!("{}pt {}", size, weight))
}
```

### Variant props — `variants(name: T = default)`

A default is required: variant fields are stored as `T`, not `Option<T>`.
Setter takes `impl Into<T>`, which lets you pass the enum value or any type
`T: From<…>`. Pairs naturally with `#[variant]`-annotated enums.

```rust
#[component(
    variants( variant: ButtonVariant = ButtonVariant::Default),
    ext(style, children),
)]
fn Button() {
    Div()
        .merge_style(style.clone())
        .children(children.clone())
}

b.variant(ButtonVariant::Destructive);   // typed
b.variant("destructive");                // &str
b.variant(String::from("destructive"));  // String
```

> **`variants(...)` requires `ext(style)`.** The merged style is bound as a
> local called `style` in the render body — that requires the `style`
> extension. If you declare variants without `ext(style)`, the macro emits a
> targeted compile error.

#### Base style — `variants("base classes", …)`

The first token inside `variants(...)` may be a string literal. It becomes
the bottom of the style merge chain — declared before every variant, so
variants and user overrides still win over it:

```rust
#[component(
    ext(style, children),
    variants(
        "inline-flex items-center justify-center rounded-md \
         text-sm font-medium whitespace-nowrap transition-colors",
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Default,
    ),
)]
pub fn Button() {
    Div().merge_style(style.clone()).children(children.clone())
}
```

Rules:

* **At most one** string literal. A second one is a compile error.
* **Positioned first.** If you write `variants(variant: …)` first and then
  a string literal, the parser rejects the string as an unexpected token.
* **Requires `ext(style)`.** The base lives in the same fold chain that
  merges the variants.
* **Optional.** When omitted, the merge chain is exactly what it was before:
  `Style::default().merge(variant.resolve())…`.

### Struct fields — `#[struct_fields(...)]`

Placed **below** `#[component]`. Same grammar as `props(...)`, but no setter
is generated and they are documented in a separate section. Use them to
store data your own trait impls need.

```rust
use freyacn::prelude::*;

#[component(props( subtitle: String))]
#[struct_fields(
    tooltip: MyTooltip,              // Option<MyTooltip> = None
    hover_count: usize = 0,          // usize = 0
)]
fn Card(name: String) {
    let _ = tooltip.as_ref().map(|t| t.text.as_str()).unwrap_or("");
    let _ = hover_count;
    Div().child(label().text(name.as_str()))
}

// Your own trait — plain Rust, no macro.
#[derive(Default)]
pub struct MyTooltip {
    pub text: String,
    pub visible: bool,
}

pub trait TooltipExt {
    fn set_tooltip(&mut self, text: impl Into<String>);
}

impl TooltipExt for CardComponent {
    fn set_tooltip(&mut self, text: impl Into<String>) {
        let tip = self.tooltip.get_or_insert_with(MyTooltip::default);
        tip.text = text.into();
    }
}

let mut card = Card().name("Ada");
card.set_tooltip("Click to edit");
card.hover_count = 7;
```

### Extension fields — `ext(...)`

See [`#[extensions(...)]`](#extensions-standalone-on-any-struct) for the
full list. Names inside `ext(...)` are the same names that
`#[extensions(...)]` accepts.

---

## `#[variant]`

Applied to an enum, generates:

* `impl MyEnum { fn style_str(&self) -> &'static str; }`
* `impl MyEnum { fn resolve(&self) -> freyacn::prelude::Style; }` —
  convenience alias that doesn't require importing `StyleVariant`.
* `impl freyacn::prelude::Variant for MyEnum` — the trait-based resolver.
* `impl From<&str> for MyEnum`
* `impl From<String> for MyEnum`

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

### Fallback for unknown names

`From<&str>` falls back to the **first declared variant**. Put your natural
default first and mark it `#[default]` if you use `derive(Default)`.

### Resolving to a `Style`

`.resolve()` reads the active theme and parses the variant's style string.
It must be called inside a Freya render context. When used as a prop inside
a component that declares `variants(...)` and `ext(style)`, the macro calls
`.resolve()` for you and merges the result into the render body's `style`
local — after the base style, before user overrides.

You can also resolve manually:

```rust
let v = ButtonVariant::Destructive;
let s = v.resolve();
Div().merge_style(s);
```

### The `StyleVariant` trait

`#[variant]` generates `impl freyacn::prelude::Variant for MyEnum`. The
trait is:

```rust
pub trait Variant {
    fn resolve(&self) -> Style;
}
```

It exists so the component macro can merge variant styles without knowing
the concrete enum type — it just calls `Variant::resolve(&self.field)`.

---

## `#[extensions(...)]` — standalone on any struct

The `#[extensions(...)]` attribute (and its `ext(...)` sibling inside
`#[component]`) opt a struct into freyacn's built-in **extension bundles**.
Each name maps to a bundle of:

* a field to add to the struct,
* an initialiser for that field,
* a render-body local binding, and
* an `impl XxxExt for YourStruct` block.

### Supported names

| Name             | Trait                                | Field(s) added                                           | Trait method             |
|------------------|--------------------------------------|----------------------------------------------------------|--------------------------|
| `children`       | `freyacn::prelude::ChildrenExt`      | `children: Vec<Element>`                                 | `get_children`           |
| `key`            | `freyacn::prelude::KeyExt`           | `key: DiffKey`                                           | `write_key`              |
| `style`          | `freyacn::prelude::StyleExt`         | `style: Style`                                           | `get_style`              |
| `event_handlers` | `freyacn::prelude::EventHandlersExt` | `event_handlers: FxHashMap<EventName, EventHandlerType>` | `get_event_handlers`     |
| `accessibility`  | `freyacn::prelude::AccessibilityExt` | `accessibility: AccessibilityData`                       | `get_accessibility_data` |

### On a hand-written struct

The attribute works on any struct with named fields. The struct must derive
or implement `Default` so the added fields can be initialised.

```rust
use freyacn::prelude::*;

#[extensions(children, key, style, accessibility)]
#[derive(Clone, PartialEq, Default)]
pub struct PlainBox;
```

After expansion, `PlainBox` has:

```rust
pub struct PlainBox {
    pub children: Vec<Element>,
    pub key: DiffKey,
    pub style: Style,
    pub accessibility: AccessibilityData,
}

impl ChildrenExt for PlainBox { /* get_children */ }
impl KeyExt for PlainBox { /* write_key */ }
impl StyleExt for PlainBox { /* get_style */ }
impl AccessibilityExt for PlainBox { /* get_accessibility_data */ }
```

### Choosing a subset

Opt into just the extensions you need:

```rust
#[extensions(style)]
#[derive(Clone, PartialEq, Default)]
pub struct Styled {
    pub label: String,
}
```

`Styled` gets a `style: Style` field and the `StyleExt` impl — but no
`children`, no `key`, no event handlers. Pay only for what you request.

Standalone `#[extensions(...)]` never accepts a base style — that's a
`#[component]` concept that lives inside `variants("…")`.

---

## Style layering

The most important runtime behaviour to understand is **how `style` is
built at render time** when a component has `variants(...)`:

```text
effective style = Style::default()
                    .style(<base>)                // optional, from variants("...")
                    .merge(variant.resolve())     // declared order
                    .merge(size.resolve())
                    .merge(self.style.clone())    // user overrides win
```

The base style string is the optional leading argument to `variants(...)`;
if omitted, that step is skipped. Every `StyleExt` method called on the
component writes to the *override layer* (`self.style`). Base and variant
styles are computed fresh from the theme at render time and merged
underneath.

**Consequences:**

* **Overrides persist across variant changes.** Calling `.variant("outline")`
  recomputes the base layer but leaves user intent intact. If a caller wrote
  `.bg_green_50()`, that override still wins.
* **Theme changes propagate automatically.** `.resolve()` reads the active
  theme on every render, not at construction. A dark-mode toggle re-renders
  the whole tree with the new theme.
* **The base comes first, so it is the least specific declaration.** Variants
  and user overrides always win over it — matching shadcn's
  `cn(base, variant, size, user)` layering.

For the actual merging rules (`Style::merge` vs `Style::or`, `Gaps`
per-side merging, `with_active_states`), see the `freyacn` crate README.

---

## Attribute forwarding

Every attribute on the function other than `#[doc]`, `#[struct_fields(...)]`,
and the `#[component(...)]` groups is forwarded **verbatim** to the generated
struct. The macro does **not** bake in any derives.

```rust
#[component(props( label: String))]
#[derive(Clone, Debug, PartialEq, Hash)]
#[cfg(feature = "experimental")]
#[my_custom_macro(arg = 42)]
fn Widget(label: String) { /* … */ }
```

The generated struct carries the same `#[derive(...)]`, `#[cfg(...)]`, and
custom macro, in the same order. Doc comments on the function become the
struct's docs; a generated summary (listing props, struct fields, base
style, and extensions) is appended after them.

Three attributes are **not** forwarded, because the macro handles them
itself:

* `#[doc]` — re-emitted first, so it isn't duplicated.
* `#[struct_fields(...)]` — consumed and turned into storage fields.
* `#[component(...)]` groups — consumed and turned into the struct's shape.

---

## Field ordering in the generated struct

```text
┌── props(...)           attribute props
├── struct_fields(...)   internal storage
├── variants(...)        variant props
├── fn params            required props
└── ext(...)             extension fields
```

The base-style string inside `variants(...)` is not a field — it only lives
in the merge chain emitted for the `style` extension.

## Collision rules

* **Duplicate names anywhere** (across `props`, `struct_fields`, `variants`,
  fn params) → **compile error** naming both sources.
* **A field whose name matches an extension field** from `ext(...)` or
  `#[extensions(...)]` → **compile error**, because extension fields are
  reserved by freyacn.

---

## Generated code

For:

```rust
#[component(
    props( subtitle: String),
    ext(children, style),
    variants(
        "inline-flex items-center rounded-lg",
        variant: ButtonVariant = ButtonVariant::Default,
    ),
)]
#[struct_fields( tooltip: MyTooltip, hover_count: usize = 0)]
#[derive(Clone, Debug, PartialEq)]
fn Button(label: String) { /* … */ }
```

the macro emits:

```rust
#[derive(Clone, Debug, PartialEq)]              // ← forwarded from the user
pub struct ButtonComponent {
    pub subtitle: Option<String>,               // prop
    pub tooltip: Option<MyTooltip>,             // struct field
    pub hover_count: usize,                     // struct field
    pub variant: ButtonVariant,                 // variant prop
    pub label: Option<String>,                  // required prop
    pub children: Vec<Element>,                 // extension
    pub style: Style,                           // extension
}

impl ButtonComponent {
    pub fn new() -> Self { /* … */ }

    pub fn label(self, value: impl Into<String>) -> Self { /* … */ }
    pub fn subtitle(self, value: impl Into<Option<String>>) -> Self { /* … */ }
    pub fn variant(self, value: impl Into<ButtonVariant>) -> Self { /* … */ }
}

impl Default for ButtonComponent { /* delegates to new() */ }

impl freyacn::prelude::ChildrenExt for ButtonComponent { /* … */ }
impl freyacn::prelude::StyleExt for ButtonComponent { /* … */ }

impl freyacn::prelude::Component for ButtonComponent {
    fn render(&self) -> impl freyacn::prelude::IntoElement {
        let children: &Vec<Element> = &self.children;

        // The `style` local is the merged result:
        //   base style  +  variant.resolve()  +  self.style (user overrides)
        let __freyacn_merged_style: Style = {
            let mut __freyacn_base = Style::default();
            // Base style, when declared in `variants("...")`:
            __freyacn_base = StyleExt::style(
                __freyacn_base,
                "inline-flex items-center rounded-lg",
            );
            __freyacn_base = __freyacn_base.merge(
                freyacn::prelude::Variant::resolve(&self.variant)
            );
            __freyacn_base.merge(self.style.clone())
        };
        let style: &Style = &__freyacn_merged_style;

        let subtitle: &Option<String> = &self.subtitle;
        let tooltip: &Option<MyTooltip> = &self.tooltip;
        let hover_count: &usize = &self.hover_count;
        let variant: &ButtonVariant = &self.variant;

        let label: &String = self.label.as_ref()
            .expect("required prop `label` was not set before render");

        /* original function body */
        // The last expression is the return value; no `.into()` required.
    }
}

pub fn Button() -> ButtonComponent { ButtonComponent::new() }

#[macro_export]
macro_rules! Button {
    ($($key:ident = $val:expr),* $(,)?) => {
        ButtonComponent::new()$(.$key($val))*
    };
}
```

When `variants(...)` has **no** base string, the `StyleExt::style(...)`
line is omitted — the merge chain is byte-identical to what it was before
the feature landed. When the component declares **no** variants at all,
the `style` extension still emits the fast path (`let style = &self.style;`)
with no merge at all.

Every item carries generated documentation describing its role and kind —
`cargo doc` produces a complete reference for free. When a base style is
declared, it appears in the struct's doc under a dedicated "Base style"
section.

---

## Compile errors

### Duplicate name across sources

```rust
#[component(props( name: String = "ada".to_string()))]
fn Card(name: String) { /* … */ }
```

```
error: `name` is declared more than once (a function parameter and an
       attribute prop (`props(...)`)). Each field name may be used by
       exactly one declaration.
```

### Missing type

```rust
#[component(props(name))]
fn Card() { /* … */ }
```

```
error: missing type — every field must declare its type, e.g. `name: String`.
       (Required props are declared via the function signature instead.)
```

### `required` keyword

```rust
#[component(props( required name: String))]
fn Card(name: String) { /* … */ }
```

```
error: `required` is not a keyword. Function parameters are the only way to
       declare required props: `fn Card(name: String)`. Attribute props are
       optional (or defaulted with `= expr`).
```

### Collision with an extension field

```rust
#[component(props( children: Vec<Element>), ext(children))]
fn Broken() { /* … */ }
```

```
error: `children` collides with a field contributed by `ext(...)`.
       Extension fields are reserved by freyacn and cannot be shadowed by
       an attribute prop (`props(...)`).
```

### Variants without `ext(style)`

```rust
#[component(
    variants( variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: `variants(...)` requires `style` to also be listed in `ext(...)`,
       so the resolved variant styles have somewhere to be merged.
       Add `style`, e.g. `ext(style, children)`.
```

### Variant without a default

```rust
#[component(variants( variant: ButtonVariant), ext(style))]
fn Button() { /* … */ }
```

```
error: variant prop `variant` requires a default, e.g.
       `variant: MyEnum = MyEnum::Default`
```

### Base style without `ext(style)`

```rust
#[component(
    variants("p-4", variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: `variants("...")` base style requires `style` to also be listed
       in `ext(...)`, so the base can be folded into the render body's
       style. Add `style`, e.g. `ext(style, children)`.
```

### Duplicate base style

```rust
#[component(
    ext(style),
    variants("p-4", "p-6", variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: duplicate base-style string in `variants(...)`; only one is allowed
```

### Unknown group

```rust
#[component(styles( foo: String))]
fn Card() { /* … */ }
```

```
error: unknown group `styles`. Expected `props(...)`, `ext(...)`,
       or `variants(...)`.
```

### Unknown extension name

```rust
#[extensions(shadow)]
#[derive(Default)]
pub struct Widget;
```

```
error: Unsupported extension `shadow`. Valid names: `children`, `key`,
       `style`, `event_handlers`, `accessibility`.
```

### Misplaced `#[struct_fields]`

```rust
#[struct_fields(tooltip: MyTooltip)]   // ← wrong order
#[component(props( name: String))]
fn Card(name: String) { /* … */ }
```

```
error: `#[struct_fields(...)]` must be placed **below** `#[component(...)]`
```

---

## Full worked example

```rust
use freyacn::prelude::*;

// ── Variant enums ────────────────────────────────────────────────────

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[variant("bg-primary text-primary-foreground hover:bg-primary/90")]
    #[default]
    Default,

    #[variant("bg-destructive text-destructive-foreground hover:bg-destructive/90")]
    Destructive,

    #[variant("border border-border bg-background hover:bg-muted")]
    Outline,
}

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[variant("h-8 px-3 text-xs")]    Sm,
    #[variant("h-9 px-4 text-sm")]
    #[default] Md,
    #[variant("h-10 px-6 text-base")] Lg,
}

// ── Component ────────────────────────────────────────────────────────

/// A button with variant, size, optional tooltip and full styling.
#[component(
    props(
        tooltip: String,
        disabled: bool = false,
    ),
    ext(style, children, key, event_handlers, accessibility),
    variants(
        // Base: shared layout, typography, disabled behaviour.
        "inline-flex items-center justify-center gap-2 \
         rounded-md font-medium whitespace-nowrap transition-colors \
         disabled:pointer-events-none disabled:opacity-50",
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Md,
    ),
)]
#[struct_fields( hover_count: usize = 0)]
#[derive(Clone, PartialEq)]
pub fn Button() {
    // `style` = base style + variant.resolve() + size.resolve() + user overrides.
    let tip = tooltip.as_deref().unwrap_or("");

    Div()
        .merge_style(style.clone())
        .disabled(*disabled)
        .focusable(true)
        .accessibility(accessibility.clone())
        .event_handlers(event_handlers.clone())
        .children(children.clone())
        .child(label().text(tip))
}

// ── Standalone extensions on a hand-written struct ──────────────────

/// A very small wrapper used for pure styling, no component.
#[extensions(style)]
#[derive(Clone, PartialEq, Default)]
pub struct Chip {
    pub text: String,
}

impl Chip {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), ..Default::default() }
    }
}

fn render_chip(chip: Chip) -> impl IntoElement {
    let chip = chip.bg_slate_100().text_slate_800().px_2().py_1();
    label().text(chip.text.as_str())
}

// ── Usage ────────────────────────────────────────────────────────────

fn app() -> impl IntoElement {
    use_init_cn_theme(Theme::light().base_color("slate").theme_color("blue"));

    let mut b = Button()
        .variant("destructive")       // base layer
        .size(ButtonSize::Lg)         // base layer
        .tooltip("Save the current document")
        .disabled(false)
        .bg_green_50()                // override — wins over the variant
        .p_6()                        // override — wins over the size
        .corner_radius(12.0)
        .shadow_md()
        .style("font-semibold border-2 border-slate-200")
        .child(label().text("Save"));

    b.hover_count = 3;

    // Change the variant after the fact. Base recomputes; overrides persist.
    b.variant("outline");

    let chip = Chip::new("Beta");
    let _ = render_chip(chip);

    b
}
```

---

## Design notes

**Why does each source decide the prop kind?**

* *Is this part of the component's public API?* — `props(...)`,
  `variants(...)`, and fn params: **yes**. They get setters and appear in
  the docs as props.
* *Is this internal storage the user manages?* — `#[struct_fields(...)]`:
  **no**. No setters, documented separately.
* *Is this framework-provided machinery?* — `ext(...)` /
  `#[extensions(...)]`: **no**. Reserved by freyacn, checked for collisions,
  documented as extensions.

Collapsing them would force every declaration to be ambiguous about intent,
and make "no setter" and "reserved name" impossible to express.

**Why doesn't the component function need a return type?**

Because the macro is the one that emits the `Component::render` method, and
`render` already has a fixed signature — `fn render(&self) -> impl
IntoElement`. The user's body is spliced in as the body of `render`, so the
return type comes from the surrounding signature, not from the function the
user wrote. That also means no `.into()` is needed: the last expression of
the body is coerced to `impl IntoElement` by the standard `impl Trait`
return mechanism.

**Why does the style merge happen at render time, not construction time?**

Because `Variant::resolve()` reads the active theme. Resolving at
construction time would freeze the theme — dark/light toggles, theme
switches, and any dynamic theming would be invisible to already-built
components. Rendering happens inside a Freya context, so it's the natural
place to read the theme.

The cost is one parse + one merge per variant per render, plus one parse for
the base style when present. For a typical button that's three parses and
four merges. It's cheap relative to the renderer's work. If it ever shows up
in a profile, memoizing per `(component, theme)` is straightforward.

**Why does the base style live inside `variants(...)` and not `props(...)`?**

Because the base style is a *style* declaration, not a *prop* — it has no
storage field, no setter, and no read from the render body. It only exists
to seed the merge chain. Grouping it with `variants(...)` keeps the entire
style-resolution story in one syntactic place: base → variants → user
overrides. It also makes the dependency obvious: both the base and the
variants need `ext(style)`, so the compile error fires from the same block.

**Why does the base come *first* in the merge chain?**

So variants and user overrides can win over it. This mirrors shadcn's
`cn(base, variant, size, user)` layering: the base is the least specific
declaration and the user's `.bg_green_50()` is the most specific. If the
base came last, it would silently override every variant style — the
opposite of what a "base" means.

**Why do user overrides persist across variant changes?**

Because intent should not be discarded silently. If a caller explicitly
set `.bg_green_50()`, they meant it. Changing the variant only changes the *base* style; the override remains until the
caller changes it. This is
consistent with how shadcn/ui and similar layered systems behave — the
user's overrides always win.

**Why is `freyacn-macros` internal to `freyacn`?**

Because the macros emit paths that point at `freyacn` types (`::freyacn::prelude::Component`,
`::freyacn::prelude::Style`,
`::freyacn::prelude::Variant`, …). Exposing `freyacn-macros` as an end-user
dependency would mean users who pull it directly also have to pull
`freyacn`, and could end up with mismatched versions if they pin them
independently. Re-exporting from `freyacn` keeps the version surface a
single crate.

**Why `impl Into<Option<T>>` for optional setters?**

Because std already gives us `From<T> for Option<T>` and `From<T> for T`,
so a single signature accepts a bare `T` (auto-wrapped), an `Option<T>`
(identity), and `None` — with no runtime overhead and no ambiguity.

**Why a compile error instead of a silent override for name collisions?**

Silent override hides a real bug: the user declared the same thing twice,
or tried to shadow an extension field. Refusing to compile is the loudest,
earliest signal.

**Why are extension fields appended last?**

Props, variants, and struct fields are what the user wrote; extensions are
opt-in machinery. Putting user-written declarations first keeps the common
case at the top of the struct.

**Why generate a free function *and* a macro with the same name?**

They live in different Rust namespaces (value vs macro), so there's no
conflict. The function is the zero-argument fast path; the macro adds
named-argument construction.

**Why forward user attributes instead of baking in derives?**

Because the right set of derives depends on how the component is used: a
keyed child needs `PartialEq`; one stored in a collection needs `Clone`;
one inspected during debugging needs `Debug`. Baking any of them in forces
users to accept them and hides the cost. Forwarding keeps the macro
agnostic — the user writes what they want and sees the exact set of traits
the struct implements in its source.

**Why is there a stub `#[struct_fields]` proc-macro?**

Because attribute macros are processed top-to-bottom. If
`#[struct_fields(...)]` is placed above `#[component(...)]`, the compiler
reaches `struct_fields` first — and without a stub, it errors with "cannot
find attribute". The stub produces a friendly message. When the ordering is
correct, `#[component]` consumes the attribute before the compiler ever
sees the stub, so the stub never fires.

---

## Roadmap

The following items are planned but not yet implemented:

* **`resolve_with(&Theme)`.** `.resolve()` requires a Freya render context.
  A theme-explicit variant is planned for tests and out-of-render use. This
  requires threading the theme through the palette lookup machinery, which
  is a standalone change.
* **Auto-set position when a side is set.** Setting `.top(10.0)` without a
  position kind currently leaves the value stranded. A behaviour decision is
  pending.
* **More built-in extensions.** More bundles will land once the current
  surface stabilises.

---

## Contributing

This crate is unstable and moves fast. Before opening a PR, please open an
issue describing what you want to change and why — the API surface is still
in flux, and a quick design discussion will save everyone time.

---

## License

MIT.

[freyacn]: https://github.com/freyacn
[freya]: https://crates.io/crates/freya