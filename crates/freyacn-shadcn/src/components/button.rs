use freyacn_core::elements::button;
use freyacn_core::prelude::{AccessibilityExt, ChildrenExt, EventHandlersExt, StyledExt};
use freyacn_core::styling::HasSemanticEvents;
use freyacn_macros::{component, variant};

#[variant(freyacn_core)]
#[derive(Default, PartialEq, Clone)]
pub enum ButtonVariant {
    #[default]
    #[variant("bg-primary text-primary-foreground hover:bg-primary/90")]
    Default,

    #[variant(
        "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40"
    )]
    Destructive,

    #[variant(
        "border bg-background hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50"
    )]
    Outline,

    #[variant("bg-secondary text-secondary-foreground hover:bg-secondary/80")]
    Secondary,

    #[variant("hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50")]
    Ghost,

    #[variant("text-primary hover:underline")]
    Link,
}

#[variant(freyacn_core)]
#[derive(Default, PartialEq, Clone)]
pub enum ButtonSize {
    #[default]
    #[variant("h-9 px-4 py-2")]
    Default,

    #[variant("h-6 gap-1 rounded-md px-2 text-xs")]
    Xs,

    #[variant("h-8 gap-1.5 rounded-md px-3")]
    Sm,

    #[variant("h-10 rounded-md px-6")]
    Lg,

    #[variant("size-9")]
    Icon,

    #[variant("size-6 rounded-md")]
    IconXs,

    #[variant("size-8")]
    IconSm,

    #[variant("size-10")]
    IconLg,
}

#[component(
    crate = freyacn_core,
    variants(
        "flex-row items-center justify-center gap-2 rounded-md text-sm font-medium \
         focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 \
         disabled:opacity-50 \
         aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
         dark:aria-invalid:ring-destructive/40",
        variant: ButtonVariant = ButtonVariant::Default,
        size: ButtonSize = ButtonSize::Default,
    ),
    props(
        with_disabled: bool = false,
        with_focusable: bool = false,
        with_selected: bool = false,
    ),
    ext(styled, event_handlers, children, accessibility, semantic_events)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn Button() {
    button()
        .merge_style(style.clone())
        .children(children.clone())
        .event_handlers(event_handlers.clone())
        .accessibility(accessibility.clone())
        .semantic_events(semantic_events.clone())
        .with_disabled(*with_disabled)
        .with_focusable(*with_focusable)
        .with_selected(*with_selected)
}
