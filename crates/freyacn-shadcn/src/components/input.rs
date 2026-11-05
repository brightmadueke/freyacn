//! shadcn Input — styled text input.
//!
//! Wraps [`freyacn_core::elements::input::Input`] with shadcn's base
//! style and **uncontrolled-by-default** semantics.
//!
//! ## Types
//!
//! All the `InputMode` variants are supported — `Text`, `Password`,
//! `Email`, `Tel`, `Url`, `Number`, `Search`. `Tel` and `Number` filter
//! characters on typing and paste; `Password` masks the display. See
//! [`freyacn_core::elements::input::InputMode`] for details.
//!
//! ```rust,ignore
//! Input().mode(InputMode::Email).placeholder("you@example.com")
//! Input().mode(InputMode::Number).placeholder("0")
//! Input().mode(InputMode::password()).placeholder("Password")
//! ```
//!
//! ## Validation
//!
//! `min_length`, `max_length`, and `required` are supported. `max_length`
//! blocks typing past the limit (matching HTML). `min_length` and
//! `required` are validation-only — pass the current value to a fresh
//! `Input()` to inspect `.validate()` / `.is_valid()` on the parent side:
//!
//! ```rust,ignore
//! let v = use_state(String::new);
//! let text = (*v.read()).clone();
//! let valid = Input().value(text.clone()).min_length(3).is_valid();
//!
//! Input()
//!     .value(text)
//!     .min_length(3)
//!     .with_aria_invalid(!valid)
//!     .on_change(move |t| v.set(t))
//! ```
//!
//! ## Class mapping
//!
//! See the shadcn/ui `Input` reference for the full list. Highlights:
//!
//! - `bg-transparent` inherits the parent's background (card / page).
//! - `aria-invalid:border-destructive` uses a solid destructive border
//!   (the `ring-*` variant is dropped — ring maps to border in FreyaCN,
//!   so keeping both would overwrite one with the other).
//! - `dark:bg-input/30` gives a subtle tint in dark themes.

use freya::prelude::{EventHandler, WritableUtils, use_side_effect, use_state};
use freyacn_core::elements::input::{InputMode, input};
use freyacn_core::prelude::{AccessibilityExt, EventHandlersExt, StyledExt};
use freyacn_core::styling::HasSemanticEvents;
use freyacn_macros::component;

#[component(
    crate = freyacn_core,
    variants(
         "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30 \
        focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 \
        aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 "
    ),
    props(
        value: String = String::new(),
        placeholder: String = String::new(),
        mode: InputMode = InputMode::Text,
        max_length: Option<usize> = None,
        min_length: Option<usize> = None,
        required: bool = false,
        auto_focus: bool = false,
        with_disabled: bool = false,
        with_read_only: bool = false,
        on_change: EventHandler<String> = EventHandler::from(|_: String| {}),
        on_submit: EventHandler<String> = EventHandler::from(|_: String| {}),
    ),
    ext(styled, event_handlers, accessibility, semantic_events)
)]
#[allow(unused_variables)]
#[derive(PartialEq, Clone)]
pub fn Input() {
    // ── Internal value tracking ─────────────────────────────────────
    let mut internal = use_state(|| (*value).clone());
    let mut last_prop = use_state(|| (*value).clone());
    let mut last_state = use_state(|| (*value).clone());

    if *last_prop.peek() != *value {
        let v = (*value).clone();
        internal.set_if_modified(v.clone());
        last_prop.set_if_modified(v.clone());
        last_state.set_if_modified(v);
    }

    use_side_effect({
        let internal = internal.clone();
        let mut last_state = last_state.clone();
        let cb = on_change.clone();
        move || {
            let current: String = (*internal.read()).clone();
            if current != *last_state.peek() {
                last_state.set_if_modified(current.clone());
                cb.call(current);
            }
        }
    });

    let displayed: String = (*internal.read()).clone();

    // ── Build the core input ───────────────────────────────────────
    let mut el = input(displayed)
        .merge_style(style.clone())
        .event_handlers(event_handlers.clone())
        .accessibility(accessibility.clone())
        .semantic_events(semantic_events.clone())
        .with_disabled(*with_disabled)
        .with_read_only(*with_read_only)
        .required(*required)
        .auto_focus(*auto_focus)
        .mode(*mode)
        .on_change({
            let mut internal = internal.clone();
            let user_cb = on_change.clone();
            move |text: String| {
                internal.set(text.clone());
                user_cb.call(text);
            }
        })
        .on_submit(on_submit.clone());

    if !placeholder.is_empty() {
        el = el.placeholder((*placeholder).clone());
    }

    if let Some(n) = *max_length {
        el = el.max_length(n);
    }

    if let Some(n) = *min_length {
        el = el.min_length(n);
    }

    el
}
