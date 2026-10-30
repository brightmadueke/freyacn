//! # Button component — shadcn/ui styling on top of Freya's `Div`.
//!
//! ## Composition
//!
//! `Button` is a thin styling layer on top of `Div`. Its base styling,
//! variants, and sizes all live inside `variants(...)` as a single
//! string literal + variant props — the macro generates the fully
//! merged `style` local:
//!
//! ```text
//! base style  →  variant.resolve()  →  size.resolve()  →  user overrides
//! ```
//!
//! Interaction (hover, press, focus, disabled) is handled entirely by
//! `Div`. `Button` does not own its own state — it just passes through
//! the caller's optional `on_hover` and `on_press` callbacks.
//!
//! ## Accessibility
//!
//! `Button` forwards the caller's `accessibility` extension and sets a
//! default `Role::Button` on the underlying `Div`, so screen readers
//! announce it as a button. Callers can add a label via
//! `Button().accessibility(AccessibilityData::new().alt("..."))`.
//!
//! `Button` does **not** expose per-field `a11y_*` props
//! (`a11y_expanded`, `a11y_hidden`, etc.). Those are meant for
//! disclosure triggers, helper elements, and similar, not for a button —
//! reach for `Div` directly when you need them.
//!
//! ## Interactive unit
//!
//! The `Button`'s underlying `Div` is the interactive element. Its
//! children — `Label`, `Icon`, badges — are non-interactive by default,
//! so the button's `hover:` / `active:` stylesheet stay applied when the
//! pointer moves onto its label or icon. Hover and press state live on
//! the button `Div` and propagate to descendants via `DivActiveState`.
//!
//! Callers adding their own children should not need to touch
//! `pointer-events` — `Label` and `Icon` are non-interactive out of the
//! box. If a child does need its own hover (e.g. a nested disclosure
//! control), give it `.pointer-events-auto()`.
//!
//! ## Optional `on_hover`
//!
//! `on_hover` is `Option<EventHandler<...>>` rather than
//! `EventHandler<...> = EventHandler::from(|_| {})`. An `EventHandler`
//! default closure would be reconstructed on every render, making
//! `Button`'s props compare as different each time and causing `Div` to
//! re-render unnecessarily. The `Option` keeps props stable when the
//! caller doesn't supply a handler.

use crate::prelude::*;

// ==============================================================
// Variants
// ==============================================================

/// Visual variant of a [`Button`].
///
/// Each variant only carries what is *specific* to it — colour, border,
/// background. Layout, typography, and disabled behaviour live in the
/// base style inside `variants("…", …)` on [`Button`].
#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[variant("bg-primary text-primary-foreground hover:bg-primary/80")]
    #[default]
    Default,

    #[variant(
        "border-border bg-background text-foreground hover:bg-muted \
         dark:border-input dark:bg-input/30 dark:hover:bg-input/50"
    )]
    Outline,

    #[variant("bg-secondary text-secondary-foreground hover:bg-secondary/80")]
    Secondary,

    #[variant("hover:bg-muted text-foreground dark:hover:bg-muted/50")]
    Ghost,

    #[variant(
        "bg-destructive/10 text-destructive hover:bg-destructive/20 \
         focus-visible:border-destructive/40 focus-visible:ring-destructive/20 \
         dark:bg-destructive/20 dark:hover:bg-destructive/30 \
         dark:focus-visible:ring-destructive/40"
    )]
    Destructive,

    #[variant("text-primary underline underline-offset-4")]
    Link,
}

/// Size of a [`Button`].
///
/// Each size only carries what is *specific* to it — height, padding,
/// gap, text size. Layout shared with every size lives in the base
/// style inside `variants("…", …)` on [`Button`].
#[variant]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[variant("h-8 gap-1.5 px-2.5")]
    #[default]
    Default,

    #[variant("h-6 gap-1 rounded-md px-2 text-xs")]
    Xs,

    #[variant("h-7 gap-1 rounded-md px-2.5 text-[0.8rem]")]
    Sm,

    #[variant("h-9 gap-1.5 px-2.5 text-sm")]
    Lg,

    #[variant("size-8")]
    Icon,

    #[variant("size-6 rounded-md")]
    IconXs,

    #[variant("size-7 rounded-md")]
    IconSm,

    #[variant("size-9")]
    IconLg,
}

// ==============================================================
// Component
// ==============================================================

#[component(
    props(
        disabled: bool = false,
        focusable: bool = true,
        on_hover: Option<EventHandler<Event<PointerEventData>>> = None,
    ),
    variants(
        "inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent \
         bg-clip-padding text-sm font-medium whitespace-nowrap transition-all outline-none \
         select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 \
         disabled:opacity-50",
        variant: ButtonVariant = ButtonVariant::Default,
        size:    ButtonSize    = ButtonSize::Default,
    ),
    ext(style, children, key, event_handlers, accessibility),
)]
#[derive(Clone, PartialEq)]
pub fn Button() {
    // The macro has already built the fully-merged `style` local:
    //   base style  +  variant.resolve()  +  size.resolve()  +  user overrides
    //
    // `style.clone()` produces the whole button style in one step.
    let mut div = Div()
        .children(children.clone())
        .event_handlers(event_handlers.clone())
        // Forward caller-supplied accessibility metadata.
        .accessibility(accessibility.clone())
        // Default role — announced as a button to assistive tech.
        .a11y_role(AccessibilityRole::Button)
        .merge_style(style.clone())
        .disabled(*disabled)
        .focusable(*focusable);

    if let Some(handler) = on_hover.clone() {
        div = div.on_hover(handler);
    }

    div
}
