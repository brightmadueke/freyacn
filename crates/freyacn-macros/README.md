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

2. **The macros emit paths to `freyacn` types by default.** The generated
   code refers to `::freyacn::prelude::Component`,
   `::freyacn::prelude::Style`, and so on. Without `freyacn` in the
   dependency graph, expansion would fail with "cannot find crate
   `freyacn`".

The only reasons to depend on `freyacn-macros` directly are:

- Forking the macro implementation itself.
- Writing a crate in the *same workspace* that the facade depends on (e.g. `freyacn-shadcn`) and using the
  `crate = <path>` override to emit
  paths to `freyacn_core` instead of `freyacn`. See
  [Crate path override](#crate-path-override).

For every other use case, use `freyacn`.

---

## The four attributes

| Attribute               | Purpose                                                                                                                                                                       |
|-------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `#[component(...)]`     | Turns a function into a component struct, a PascalCase constructor fn, and a companion macro. `props`, `ext`, and `variants` groups live inside it.                           |
| `#[variant]`            | Turns an enum into a style-resolving variant type with `From<&str>` / `From<String>`, `.style_str()`, `.resolve()`, and an `impl freyacn::prelude::ComponentVariant`.         |
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

    #[variant("border border-border bg-background text-foreground")]
    Outline,
}

#[component(
    props(badge: String = "new".to_string()),
    ext(styled, children, key, event_handlers, semantic_events),
    variants(
        "px-4 py-2 rounded-lg",
        variant: ButtonVariant = ButtonVariant::Default,
    ),
)]
#[allow(unused_variables)]
pub fn Button() {
    div()
        .merge_style(style.clone())
        .children(children.clone())
        .event_handlers(event_handlers.clone())
        .semantic_events(semantic_events.clone())
        .child(text("Save"))
}
```

> **No return type needed.** The macro generates the `impl IntoElement`
> signature for the component's `render` method. Write the function body as
> the expression you want rendered, and skip both the `-> impl IntoElement`
> annotation and any trailing `.into()`.

> **`#[allow(unused_variables)]`.** `variant` and other named props are
> bound as locals in the render body — you *can* reference them, but the
> common case (a shadcn-style wrapper around a primitive) only uses the
> merged `style` local. Add the allow to silence the warning.

> **Components don't have a `label` prop by default.** Content is passed
> as children — `.child(text(…))` — or via a `label: String` function
> parameter if you want `Button().label("Save")` (function parameters
> become required props).

Three equivalent ways to construct the component above:

```rust
Button().variant(ButtonVariant::Destructive);         // typed
Button().variant("destructive");                      // &str
Button!(variant = "destructive");                     // macro
```

For what `style`, `bg_primary`, `p_4`, and other `StyledExt` helpers do,
and for the class-string token catalogue, see the `freyacn` crate README.
This document covers only the *macro* surface.

---

## Crate path override

Every macro emits paths like `::freyacn::prelude::…`. That's correct for
user-facing code, which always has the facade available.

But **internal crates that the facade depends on** (e.g.
`freyacn-shadcn`) can't depend on `freyacn` — that's a dependency cycle.
For those, override the emitted path at the macro site:

```rust
// Inside freyacn-shadcn, which depends on freyacn-core but not freyacn.
use freyacn_macros::{component, variant};

#[variant(freyacn_core)]
pub enum ButtonVariant { /* … */ }

#[component(
    crate = freyacn_core,
    variants(…),
    ext(styled, children),
)]
pub fn Button() { /* … */ }

#[extensions(crate = freyacn_core, styled, children)]
pub struct Panel {
    /* … */
}
```

The generated code now references `freyacn_core::prelude::{…}` instead of
`freyacn::prelude::{…}`. The target crate must expose the same items (`Style`, `StyledExt`, `ComponentVariant`,
`Element`, `Component`,
`IntoElement`, the extension traits, …) under its `prelude` — `freyacn-core`
does.

`#[variant]`'s override accepts the bare path (`#[variant(freyacn_core)]`);
`#[component]` and `#[extensions]` use `crate = <path>` because they
already have named groups and the explicit form avoids ambiguity.

---

## `#[component(...)]`

Every component is written as a normal Rust function. The attribute groups
declare where each field comes from; the source decides its kind.

### Field sources

| Source                          | Kind          | Field type       | Init             | Body sees                   | Setter accepts              |
|---------------------------------|---------------|------------------|------------------|-----------------------------|-----------------------------|
| `fn Card(name: String)`         | **required**  | `Option<String>` | `None`           | `&String` (panics if unset) | `impl Into<String>`         |
| `props(name: String)`           | **optional**  | `Option<String>` | `None`           | `&Option<String>`           | `impl Into<Option<String>>` |
| `props(name: String = "x")`     | **defaulted** | `String`         | `"x"`            | `&String`                   | `impl Into<String>`         |
| `variants(name: T = default)`   | **variant**   | `T`              | `default`        | `&T`                        | `impl Into<T>`              |
| `#[struct_fields(name: T)]`     | **internal**  | `Option<T>`      | `None`           | `&Option<T>`                | *(no setter)*               |
| `#[struct_fields(name: T = x)]` | **internal**  | `T`              | `x`              | `&T`                        | *(no setter)*               |
| `ext(...)`                      | **extension** | *trait-specific* | *trait-specific* | *trait-specific*            | *(no setter)*               |

You **never** write `Option<T>` yourself for an optional prop or a struct
field — writing `name: String` is enough; the macro wraps it.

### Grammar

```text
#[component(
    crate = <path>,                            // optional
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
    text(format!("Hello, {name}!"))   // no `-> …`, no `.into()`
}
```

### Required props — function parameters

```rust
#[component]
fn Greeting(name: String) {
    text(format!("Hello, {name}!"))
}

Greeting().name("Ada");           // OK
// Greeting();                    // compiles, panics at render
// Greeting().name(None);         // type error — intentional
```

### Optional props — `props(name: T)`

```rust
#[component(props(subtitle: String))]
fn Heading() {
    match subtitle.as_ref() {
        Some(s) => text(s.clone()),
        None => div(),
    }
}

Heading().subtitle("hi");                       // &str  → Some("hi".into())
Heading().subtitle(String::from("hi"));         // String → Some
Heading().subtitle(Some("hi".to_string()));     // Option<String>
Heading().subtitle(None);                       // Option<String>
```

### Defaulted props — `props(name: T = expr)`

```rust
#[component(props(
    size:   f32    = 16.0,
    weight: String = "normal".to_string(),
))]
fn LabelText() {
    text(format!("{size}pt {weight}"))
}
```

### Variant props — `variants(name: T = default)`

A default is required: variant fields are stored as `T`, not `Option<T>`.
Setter takes `impl Into<T>`, which lets you pass the enum value or any type
`T: From<…>`. Pairs naturally with `#[variant]`-annotated enums.

```rust
#[component(
    variants(variant: ButtonVariant = ButtonVariant::Default),
    ext(styled, children),
)]
#[allow(unused_variables)]
fn Button() {
    div()
        .merge_style(style.clone())
        .children(children.clone())
}

b.variant(ButtonVariant::Destructive);   // typed
b.variant("destructive");                // &str
b.variant(String::from("destructive"));  // String
```

> **`variants(...)` requires `ext(styled)`.** The merged style is bound as a
> local called `style` in the render body — that requires the `styled`
> extension. If you declare variants without `ext(styled)`, the macro emits a
> targeted compile error. (Note: the extension is `styled`, not `style` —
> `style` names the field, `styled` names the extension.)

#### Base style — `variants("base classes", …)`

The first token inside `variants(...)` may be a string literal. It becomes
the bottom of the style merge chain — declared before every variant, so
variants and user overrides still win over it:

```rust
#[component(
    ext(styled, children),
    variants(
        "inline-flex items-center justify-center rounded-md \
         text-sm font-medium whitespace-nowrap",
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Default,
    ),
)]
#[allow(unused_variables)]
pub fn Button() {
    div()
        .merge_style(style.clone())
        .children(children.clone())
}
```

Rules:

* **At most one** string literal. A second one is a compile error.
* **Positioned first.** If you write `variants(variant: …)` first and then
  a string literal, the parser rejects the string as an unexpected token.
* **Requires `ext(styled)`.** The base lives in the same fold chain that
  merges the variants.
* **Optional.** When omitted, the merge chain is just
  `Style::default().merge_style(&variant.resolve())…`.

### Struct fields — `#[struct_fields(...)]`

Placed **below** `#[component]`. Same grammar as `props(...)`, but no setter
is generated and they are documented in a separate section. Use them to
store data your own trait impls need.

```rust
use freyacn::prelude::*;

#[component(props(subtitle: String))]
#[struct_fields(
    tooltip: MyTooltip,              // Option<MyTooltip> = None
    hover_count: usize = 0,          // usize = 0
)]
fn Card(name: String) {
    let _ = tooltip.as_ref().map(|t| t.text.as_str()).unwrap_or("");
    let _ = hover_count;
    div().child(text(name.clone()))
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
  convenience alias that doesn't require importing `ComponentVariant`.
* `impl freyacn::prelude::ComponentVariant for MyEnum` — the
  trait-based resolver.
* `impl From<&str> for MyEnum`
* `impl From<String> for MyEnum`

### Declaration

```rust
use freyacn::prelude::*;

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[variant("bg-primary text-primary-foreground hover:bg-primary/90")]
    #[default]
    Default,

    #[variant("bg-destructive text-white hover:bg-destructive/90")]
    Destructive,

    #[variant("border border-input bg-background hover:bg-accent")]
    Outline,
}
```

### Kebab-case conversion

| PascalCase             | Kebab                   |
|------------------------|-------------------------|
| `Default`              | `default`               |
| `Md`                   | `md`                    |
| `XS`                   | `xs`                    |
| `IconXs`               | `icon-xs`               |
| `SecondaryDestructive` | `secondary-destructive` |

### Fallback for unknown names

`From<&str>` falls back to the **first declared variant**. Put your natural
default first and mark it `#[default]` if you use `derive(Default)`.

### Resolving to a `Style`

`.resolve()` reads the active theme, parses the variant's style string, and
returns a `Style`. It must be called inside a Freya render context.

**Crucially, `resolve()` starts from `Style::bare()`, not
`Style::default()`.** This is the [Style::bare () invariant](#the-stylebare-invariant)
that makes variant class strings survive the merge into a component. See
that section for the full explanation.

When used as a prop inside a component that declares `variants(...)` and
`ext(styled)`, the macro calls `.resolve()` for you and merges the result
into the render body's `style` local — after the base style, before user
overrides.

You can also resolve manually:

```rust
let v = ButtonVariant::Destructive;
let s = v.resolve();
div().merge_style(s);
```

### The `ComponentVariant` trait

`#[variant]` generates `impl freyacn::prelude::ComponentVariant for MyEnum`.
The trait is:

```rust
pub trait ComponentVariant {
    fn resolve(&self) -> Style;
}
```

It exists so the component macro can merge variant styles without knowing
the concrete enum type — it just calls `ComponentVariant::resolve(&self.field)`.

(The trait was renamed from `Variant` — the shorter name collided with
`StyleVariant`, the pseudo-state enum. Same role, clearer name.)

### Testing your variants

Class strings are parsed at **runtime**, so a typo compiles but panics on
the first render. Guard against that with a parse test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use freyacn_core::styling::StyledString;

    #[test]
    fn all_variants_parse() {
        for v in [
            ButtonVariant::Default,
            ButtonVariant::Destructive,
            ButtonVariant::Outline,
        ] {
            StyledString::parse(v.style_str());
        }
    }
}
```

This turns a runtime panic into a `cargo test` failure and pins the
vocabulary of every variant's class string.

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

| Name              | Trait                                 | Field(s) added                                           | Trait method             |
|-------------------|---------------------------------------|----------------------------------------------------------|--------------------------|
| `children`        | `freyacn::prelude::ChildrenExt`       | `children: Vec<Element>`                                 | `children_mut`           |
| `key`             | `freyacn::prelude::KeyExt`            | `key: DiffKey`                                           | `key_mut`                |
| `styled`          | `freyacn::prelude::StyledExt`         | `style: Style`                                           | `get_style`              |
| `event_handlers`  | `freyacn::prelude::EventHandlersExt`  | `event_handlers: FxHashMap<EventName, EventHandlerType>` | `event_handlers_mut`     |
| `accessibility`   | `freyacn::prelude::AccessibilityExt`  | `accessibility: AccessibilityData`                       | `accessibility_data_mut` |
| `semantic_events` | `freyacn::prelude::HasSemanticEvents` | `semantic_events: SemanticEvents`                        | `semantic_events_mut`    |

Note the trait method names use the `*_mut` convention — `children_mut`,
`key_mut`, `event_handlers_mut`, `accessibility_data_mut`. `get_style` is
the exception because `StyledExt::get_style` is the accessor every fluent
method builds on; renaming it would break the builder API.

### On a hand-written struct

The attribute works on any struct with named fields. The struct must derive
or implement `Default` so the added fields can be initialised.

```rust
use freyacn::prelude::*;

#[extensions(children, key, styled, accessibility)]
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

impl ChildrenExt for PlainBox { /* children_mut */ }
impl KeyExt for PlainBox { /* key_mut */ }
impl StyledExt for PlainBox { /* get_style */ }
impl AccessibilityExt for PlainBox { /* accessibility_data_mut */ }
```

### Choosing a subset

Opt into just the extensions you need:

```rust
#[extensions(styled)]
#[derive(Clone, PartialEq, Default)]
pub struct Styled {
    pub label: String,
}
```

`Styled` gets a `style: Style` field and the `StyledExt` impl — but no
`children`, no `key`, no event handlers. Pay only for what you request.

Standalone `#[extensions(...)]` never accepts a base style — that's a
`#[component]` concept that lives inside `variants("…")`.

### Using `styled` to drive an element

The `styled` extension is what gives your struct the full `StyledExt`
surface — `.bg(..)`, `.p_4()`, `.style("…")`, `.hover(..)`, and so on.
`style` starts at `Style::bare()`, so it behaves as a **sparse patch**:
only the fields you explicitly set are non-bare. See the
[Style::bare () invariant](#the-stylebare-invariant) for why this matters.

### Using `semantic_events` on an interactive element

`semantic_events` adds `.on_hover(..)` / `.on_pressed(..)` / `.on_focus(..)`
and friends, plus the aria-state builders that drive the `aria-*:`
class families:

```rust
#[component(
    ext(styled, semantic_events, children),
)]
#[allow(unused_variables)]
pub fn Input() {
    // `.with_aria_invalid(bool)` flips the `aria-invalid:*` variant
    // at the primitive level.
    input(/* … */)
        .merge_style(style.clone())
        .semantic_events(semantic_events.clone())
        .children(children.clone())
}
```

The extension only **stores** the events and exposes the setters. The
render body is responsible for wiring them into the primitive — see the
`styling::semantic_events` module docs for the correct shape.

---

## The `Style::bare()` invariant

The most important runtime behaviour to understand is **how `style` is
built at render time** when a component has `variants(...)`. Two of the
three layers start from `Style::bare()`, and one starts from
`Style::default()`:

```text
effective style = Style::default()                    ← theme layer
                    .style(<base>)                    ← base class string
                    .merge_style(&variant.resolve())  ← starts from bare()
                    .merge_style(&size.resolve())     ← starts from bare()
                    .merge_style(&self.style)         ← starts from bare()
```

**Only the theme layer uses `Style::default()`.** `ComponentVariant::resolve()`
and `self.style` (the `styled` extension's field) both initialise at
`Style::bare()`, so they behave as *sparse patches* — only fields the layer
explicitly sets survive the merge.

Why does this matter? Because `Style::merge_style` copies a field from its
argument **only if that field differs from `Style::bare()`**. If
`resolve()` returned `Style::default()`, its `text_color` would be the
theme's `foreground` (not bare black), so `merge_style` would copy
`foreground` into the folded style — silently overwriting whatever the
variant's class string actually set (e.g. `text-primary-foreground`).

Concretely: a `ButtonVariant::Default` whose class string is
`bg-primary text-primary-foreground` renders with `text_color =
primaryForeground`. If `resolve()` started from `Style::default()`,
`foreground` would leak through the `ButtonSize::Default` and `.style(..)`
layers and overwrite it, and the button would render with the wrong text
colour.

**Consequences:**

* **Overrides persist across variant changes.** Calling `.variant("outline")`
  recomputes the base layer but leaves user intent intact. If a caller
  wrote `.bg(green_50())`, that override still wins.
* **Theme changes propagate automatically.** `.resolve()` reads the active
  theme on every render, not at construction. A dark-mode toggle re-renders
  the whole tree with the new theme.
* **The base comes first, so it is the least specific declaration.**
  Variants and user overrides always win over it — matching shadcn's
  `cn(base, variant, size, user)` layering.
* **Theme-seeded fields are preserved.** If no layer sets `font_size`,
  the theme's `font_size` survives the merge because no overlay patch has
  a non-bare `font_size` field.

---

## Attribute forwarding

Every attribute on the function other than `#[doc]`, `#[struct_fields(...)]`,
and the `#[component(...)]` groups is forwarded **verbatim** to the generated
struct. The macro does **not** bake in any derives.

```rust
#[component(props(label: String))]
#[derive(Clone, Debug, PartialEq, Hash)]
#[cfg(feature = "experimental")]
#[my_custom_macro(arg = 42)]
fn Widget() { /* … */ }
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
in the merge chain emitted for the `styled` extension.

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
    props(subtitle: String),
    ext(children, styled),
    variants(
        "inline-flex items-center rounded-lg",
        variant: ButtonVariant = ButtonVariant::Default,
    ),
)]
#[struct_fields(tooltip: MyTooltip, hover_count: usize = 0)]
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
    pub style: Style,                           // extension (bare patch)
}

impl ButtonComponent {
    pub fn new() -> Self { /* … */ }

    pub fn label(self, value: impl Into<String>) -> Self { /* … */ }
    pub fn subtitle(self, value: impl Into<Option<String>>) -> Self { /* … */ }
    pub fn variant(self, value: impl Into<ButtonVariant>) -> Self { /* … */ }
}

impl Default for ButtonComponent { /* delegates to new() */ }

impl freyacn::prelude::ChildrenExt for ButtonComponent { /* children_mut */ }
impl freyacn::prelude::StyledExt for ButtonComponent { /* get_style */ }

impl freyacn::prelude::Component for ButtonComponent {
    fn render(&self) -> impl freyacn::prelude::IntoElement {
        let children: &Vec<Element> = &self.children;

        // The `style` local is the merged result:
        //   theme defaults  →  base style  →  variant.resolve()  →  self.style
        //
        // `__freyacn_base` starts from `Style::default()` (theme layer).
        // Each overlay (`variant.resolve()` and `self.style`) starts from
        // `Style::bare()`, so only fields each layer explicitly set survive
        // the merge.
        let __freyacn_merged_style: Style = {
            let mut __freyacn_base = <Style as Default>::default();
            // Base style, when declared in `variants("...")`:
            __freyacn_base = StyledExt::style(
                __freyacn_base,
                "inline-flex items-center rounded-lg",
            );
            __freyacn_base = __freyacn_base.merge_style(
                &freyacn::prelude::ComponentVariant::resolve(&self.variant)
            );
            __freyacn_base.merge_style(&self.style)
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

When `variants(...)` has **no** base string, the `StyledExt::style(...)`
line is omitted — the merge chain is byte-identical to what it was before
the feature landed. When the component declares **no** variants at all,
the `styled` extension still emits the fast path (`let style = &self.style;`)
with no merge at all.

Every item carries generated documentation describing its role and kind —
`cargo doc` produces a complete reference for free. When a base style is
declared, it appears in the struct's doc under a dedicated "Base style"
section.

---

## Hook safety

`#[component]`'s body becomes `Component::render`'s body. **Hook functions
must be called unconditionally at the top of `render`** — Freya panics on
the second render if the slot count changes.

The rule:

```rust
// ✗ PANICS on the second render — the outer `use_hook` short-circuits,
//   the inner hooks never fire, and Freya complains about slot
//   misalignment.
fn app() -> impl IntoElement {
    use_hook(|| {
        load_theme_registry_from_dir(dir).set_active("…").unwrap();
    });
    // …
}

// ✓ `load_theme_registry_from_dir` is itself a hook that internally
//   wraps in `use_hook`. Call it directly.
fn app() -> impl IntoElement {
    load_theme_registry_from_dir(dir).set_active("…").unwrap();
    // …
}
```

Same rule as Freya's own components: no `use_state` / `use_hook` /
`use_a11y` / `use_focus` / `use_side_effect` inside `if` branches, `for`
loops, or nested `use_hook` closures.

---

## Compile errors

### Duplicate name across sources

```rust
#[component(props(name: String = "ada".to_string()))]
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
#[component(props(required name: String))]
fn Card(name: String) { /* … */ }
```

```
error: `required` is not a keyword. Function parameters are the only way to
       declare required props: `fn Card(name: String)`. Attribute props are
       optional (or defaulted with `= expr`).
```

### Collision with an extension field

```rust
#[component(props(children: Vec<Element>), ext(children))]
fn Broken() { /* … */ }
```

```
error: `children` collides with a field contributed by `ext(...)`.
       Extension fields are reserved by freyacn and cannot be shadowed by
       an attribute prop (`props(...)`).
```

### Variants without `ext(styled)`

```rust
#[component(
    variants(variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: `variants(...)` requires `styled` to also be listed in `ext(...)`,
       so the resolved variant styles have somewhere to be merged.
       Add `styled`, e.g. `ext(styled, children)`.
```

### Variant without a default

```rust
#[component(variants(variant: ButtonVariant), ext(styled))]
fn Button() { /* … */ }
```

```
error: variant prop `variant` requires a default, e.g.
       `variant: MyEnum = MyEnum::Default`
```

### Base style without `ext(styled)`

```rust
#[component(
    variants("p-4", variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: `variants("...")` base style requires `styled` to also be listed
       in `ext(...)`, so the base can be folded into the render body's
       style. Add `styled`, e.g. `ext(styled, children)`.
```

### Duplicate base style

```rust
#[component(
    ext(styled),
    variants("p-4", "p-6", variant: ButtonVariant = ButtonVariant::Default),
)]
fn Button() { /* … */ }
```

```
error: duplicate base-style string in `variants(...)`; only one is allowed
```

### Unknown group

```rust
#[component(styles(foo: String))]
fn Card() { /* … */ }
```

```
error: unknown group `styles`. Expected `props(...)`, `ext(...)`,
       or `variants(...)`. (The crate path is set with `crate = path`.)
```

### Unknown extension name

```rust
#[extensions(shadow)]
#[derive(Default)]
pub struct Widget;
```

```
error: Unsupported extension `shadow`. Valid names: `children`, `key`,
       `styled`, `event_handlers`, `accessibility`, `semantic_events`.
```

### Misplaced `#[struct_fields]`

```rust
#[struct_fields(tooltip: MyTooltip)]   // ← wrong order
#[component(props(name: String))]
fn Card(name: String) { /* … */ }
```

```
error: `#[struct_fields(...)]` must be placed **below** `#[component(...)]`
```

### Duplicate `crate = …`

```rust
#[component(crate = freyacn_core, crate = freyacn, ext(styled))]
fn Card() { /* … */ }
```

```
error: duplicate `crate = ...` argument
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

    #[variant("bg-destructive text-white hover:bg-destructive/90")]
    Destructive,

    #[variant("border border-input bg-background hover:bg-accent")]
    Outline,
}

#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[variant("h-8 px-3 text-xs")]
    Sm,
    #[variant("h-9 px-4 text-sm")]
    #[default]
    Md,
    #[variant("h-10 px-6 text-base")]
    Lg,
}

// ── Component ────────────────────────────────────────────────────────

/// A button with variant, size, optional tooltip and full styling.
#[component(
    props(
        tooltip: String,
        disabled: bool = false,
    ),
    ext(styled, children, key, event_handlers, accessibility, semantic_events),
    variants(
        // Base: shared layout, typography, disabled behaviour.
        "inline-flex items-center justify-center gap-2 \
         rounded-md font-medium whitespace-nowrap \
         disabled:opacity-50 disabled:cursor-not-allowed",
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Md,
    ),
)]
#[struct_fields(hover_count: usize = 0)]
#[allow(unused_variables)]
#[derive(Clone, PartialEq)]
pub fn Button() {
    // `style` = base + variant.resolve() + size.resolve() + user overrides.
    let tip = tooltip.as_deref().unwrap_or("");
    let tip: &'static str = Box::leak(tip.to_string().into_boxed_str());

    div()
        .merge_style(style.clone())
        .children(children.clone())
        .event_handlers(event_handlers.clone())
        .accessibility(accessibility.clone())
        .semantic_events(semantic_events.clone())
        .child(text(tip))
}

// ── Standalone extensions on a hand-written struct ──────────────────

/// A very small wrapper used for pure styling, no component.
#[extensions(styled)]
#[derive(Clone, PartialEq, Default)]
pub struct Chip {
    pub text: String,
}

impl Chip {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), ..Default::default() }
    }
}

// ── Usage ────────────────────────────────────────────────────────────

fn app() -> impl IntoElement {
    use_init_theme_registry(include_str!("../themes/default-dark.json"));

    let mut b = Button()
        .variant("destructive")       // base layer
        .size(ButtonSize::Lg)         // base layer
        .tooltip("Save the current document")
        .disabled(false)
        .bg(green_50())               // override — wins over the variant
        .p_6()                        // override — wins over the size
        .corner_radius(12.0)
        .style("font-semibold border-2 border-neutral-200")
        .child(text("Save"));

    b.hover_count = 3;

    // Change the variant after the fact. Base recomputes; overrides persist.
    b.variant("outline");

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

Because `ComponentVariant::resolve()` reads the active theme. Resolving at
construction time would freeze the theme — dark/light toggles, theme
switches, and any dynamic theming would be invisible to already-built
components. Rendering happens inside a Freya context, so it's the natural
place to read the theme.

The cost is one parse + one merge per variant per render, plus one parse
for the base style when present. For a typical button that's three parses
and four merges. It's cheap relative to the renderer's work. If it ever
shows up in a profile, memoizing per `(component, theme)` is
straightforward.

**Why does `resolve()` start from `Style::bare()`?**

Because `merge_style` copies any field that differs from `bare()`. If
`resolve()` started from `Style::default()`, its theme-seeded fields (`text_color = foreground`,
`background = background`, `font_size`, …)
would look "explicitly set" and be copied back, overwriting whatever the
variant's class string actually touched. The
[Style::bare () invariant](#the-stylebare-invariant) section covers this in
full.

**Why does the `styled` extension's `self.style` start from
`Style::bare()` too?**

Same reason. `.style("…")` / `.bg(..)` / `.p_4()` chain writes to
`self.style`. Starting from `Style::default()` would seed the field with
theme defaults on construction, and `merge_style(&self.style)` would copy
those seeded values back over the variant's class strings. Starting from
`bare()` keeps the field a **sparse patch** — only the fields the user
explicitly set are non-bare, so the merge only copies what the user
intended.

**Why does the base style live inside `variants(...)` and not `props(...)`?**

Because the base style is a *style* declaration, not a *prop* — it has no
storage field, no setter, and no read from the render body. It only exists
to seed the merge chain. Grouping it with `variants(...)` keeps the entire
style-resolution story in one syntactic place: base → variants → user
overrides. It also makes the dependency obvious: both the base and the
variants need `ext(styled)`, so the compile error fires from the same
block.

**Why does the base come *first* in the merge chain?**

So variants and user overrides can win over it. This mirrors shadcn's
`cn(base, variant, size, user)` layering: the base is the least specific
declaration and the user's `.bg(green_50())` is the most specific. If the
base came last, it would silently override every variant style — the
opposite of what a "base" means.

**Why do user overrides persist across variant changes?**

Because intent should not be discarded silently. If a caller explicitly
set `.bg(green_50())`, they meant it. Changing the variant only changes the *base* style; the override remains until the
caller changes it. This is
consistent with how shadcn/ui and similar layered systems behave — the
user's overrides always win.

**Why is `freyacn-macros` internal to `freyacn`?**

Because the macros emit paths that point at `freyacn` types (`::freyacn::prelude::Component`,
`::freyacn::prelude::Style`,
`::freyacn::prelude::ComponentVariant`, …). Exposing `freyacn-macros` as
an end-user dependency would mean users who pull it directly also have to
pull `freyacn`, and could end up with mismatched versions if they pin them
independently. Re-exporting from `freyacn` keeps the version surface a
single crate.

The `crate = <path>` override exists so internal crates that the facade
depends on (`freyacn-shadcn`, etc.) can use the macros without depending
on the facade.

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

* **`resolve_with(&Theme)`.** `ComponentVariant::resolve()` requires a
  Freya render context. A theme-explicit variant is planned for tests and
  out-of-render use. This requires threading the theme through the palette
  lookup machinery, which is a standalone change.
* **Auto-set position when a side is set.** Setting `.top(10.0)` without a
  position kind currently leaves the value stranded. A behaviour decision is
  pending.
* **More built-in extensions.** More bundles will land once the current
  surface stabilises.
* **Auto-wire semantic events.** The `semantic_events` extension currently
  only stores the callbacks; the render body must call
  `attach_pointer_handlers` / `use_semantic_focus_events` manually. An
  "auto-wire" mode that emits the plumbing from the macro would need a
  post-body emission slot the extension system doesn't currently have.

---

## Contributing

This crate moves fast. Before opening a PR, please open an issue describing
what you want to change and why — the API surface is still in flux, and a
quick design discussion will save everyone time.

---

## License

MIT.

[freyacn]: https://github.com/freyacn
[freya]: https://crates.io/crates/freya

```