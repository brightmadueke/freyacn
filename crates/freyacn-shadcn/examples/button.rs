//! shadcn Button showcase.
//!
//! Run with:
//!
//! ```text
//! cargo run -p freyacn-shadcn --example button
//! ```
//!
//! Points at `<workspace>/resources/themes/v1` by default. Override with
//! `FREYACN_THEMES_DIR=/path/to/themes cargo run -p freyacn-shadcn --example button`.

use freya::icons::lucide;
use freya::prelude::*;
use freyacn_core::elements::icon::icon;
use freyacn_core::prelude::{IntoElement, div, text};
use freyacn_core::styling::{HasSemanticEvents, StyledExt};
use freyacn_core::theme::load_themes_with_default;
use freyacn_shadcn::components::button::{Button, ButtonSize, ButtonVariant};

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    // `load_themes_with_default` is itself a hook — it wraps its
    // internals in `use_hook`, so the directory is read once and cached
    // across re-renders. Do NOT wrap this call in another `use_hook`:
    // the outer closure would short-circuit on the second render, skip
    // the inner hooks, and Freya would panic with "Hook functions must
    // follow these rules".
    //
    // It also activates `"default-dark"` as the initial theme in one
    // step, avoiding a separate `.set_active(..)` call (which returns
    // `Result` and would need `.expect(..)`). Panics if no theme in the
    // directory has the given id — for an example, that's the right
    // failure mode.
    load_themes_with_default(
        "/home/bright/RustroverProjects/freyacn/resources/themes/v1",
        "default-dark",
    );

    div()
        .style("size-full bg-background")
        .child(buttons_showcase())
        .scroll_y()
}

fn buttons_showcase() -> impl IntoElement {
    let click_count = use_state(|| 0u32);
    let locked = use_state(|| false);
    let toggled = use_state(|| false);
    let invalid = use_state(|| false);

    let click_v = *click_count.read();
    let locked_v = *locked.read();
    let toggled_v = *toggled.read();
    let invalid_v = *invalid.read();

    div()
        .style("flex-col gap-8 p-8 max-w-3xl")
        // ── Header ──────────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-1")
                .child(text("Button").style("text-3xl font-bold"))
                .child(
                    text("All shadcn variants, sizes, and states").style("text-sm text-gray-500"),
                ),
        )
        // ── Variants ────────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Variants").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child(Button().child(text("Default")))
                        .child(
                            Button()
                                .variant(ButtonVariant::Destructive)
                                .child(text("Destructive")),
                        )
                        .child(
                            Button()
                                .variant(ButtonVariant::Outline)
                                .child(text("Outline")),
                        ),
                )
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child(
                            Button()
                                .variant(ButtonVariant::Secondary)
                                .child(text("Secondary")),
                        )
                        .child(Button().variant(ButtonVariant::Ghost).child(text("Ghost")))
                        .child(Button().variant(ButtonVariant::Link).child(text("Link"))),
                ),
        )
        // ── Sizes ───────────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Sizes").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child(Button().size(ButtonSize::Xs).child(text("Extra small")))
                        .child(Button().size(ButtonSize::Sm).child(text("Small")))
                        .child(Button().size(ButtonSize::Default).child(text("Default")))
                        .child(Button().size(ButtonSize::Lg).child(text("Large"))),
                ),
        )
        // ── Icon-only sizes ─────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Icon").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child(Button().size(ButtonSize::IconXs).child(plus_icon()))
                        .child(Button().size(ButtonSize::IconSm).child(plus_icon()))
                        .child(Button().size(ButtonSize::Icon).child(plus_icon()))
                        .child(Button().size(ButtonSize::IconLg).child(plus_icon())),
                ),
        )
        // ── Icon + label composition ────────────────────────────────
        //
        // The `has-[svg]:px-N` in each non-icon size variant fires on
        // the second render after the icon mounts: the `Icon` signals
        // `HasKind::Svg`, the `Div` re-renders with `HasSvg` in its
        // live set, and the padding shrinks. Same visual result as
        // shadcn's `has-[>svg]:px-N` — see `styling::has` for the
        // mechanism and the one-frame caveat.
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("With icons").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child(Button().child(plus_icon()).child(text("New")))
                        .child(
                            Button()
                                .variant(ButtonVariant::Destructive)
                                .child(trash_icon())
                                .child(text("Delete")),
                        )
                        .child(
                            Button()
                                .variant(ButtonVariant::Outline)
                                .child(text("Download"))
                                .child(download_icon()),
                        ),
                ),
        )
        // ── States ──────────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("States").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child({
                            let mut toggled = toggled.clone();
                            Button()
                                .variant(if toggled_v {
                                    ButtonVariant::Default
                                } else {
                                    ButtonVariant::Outline
                                })
                                .with_selected(toggled_v)
                                .on_press(move |_| {
                                    let next = !*toggled.read();
                                    toggled.set(next);
                                })
                                .child(text(if toggled_v { "On" } else { "Toggle" }))
                        })
                        .child({
                            let mut locked = locked.clone();
                            Button()
                                .variant(ButtonVariant::Ghost)
                                .on_press(move |_| {
                                    let next = !*locked.read();
                                    locked.set(next);
                                })
                                .child(text(if locked_v { "Unlock" } else { "Lock" }))
                        })
                        .child(
                            Button()
                                .variant(ButtonVariant::Destructive)
                                .with_disabled(locked_v)
                                .child(text("Disabled")),
                        )
                        .child({
                            let mut invalid = invalid.clone();
                            Button()
                                .variant(ButtonVariant::Outline)
                                .with_aria_invalid(invalid_v)
                                .on_press(move |_| {
                                    let next = !*invalid.read();
                                    invalid.set(next);
                                })
                                .child(text(if invalid_v { "Invalid" } else { "Validate" }))
                        }),
                ),
        )
        // ── Interactive ─────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Interactive").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3")
                        .child({
                            let mut click_count = click_count.clone();
                            Button()
                                .on_press(move |_| {
                                    let next = *click_count.read() + 1;
                                    click_count.set(next);
                                })
                                .child(text("Click me"))
                        })
                        .child({
                            let mut click_count = click_count.clone();
                            Button()
                                .variant(ButtonVariant::Outline)
                                .on_press(move |_| {
                                    click_count.set(0);
                                })
                                .child(text("Reset"))
                        })
                        .child(
                            text(format!(
                                "{} click{}",
                                click_v,
                                if click_v == 1 { "" } else { "s" }
                            ))
                            .style("text-sm text-gray-500"),
                        ),
                ),
        )
}

// ── Icon helpers ────────────────────────────────────────────────

fn plus_icon() -> impl IntoElement {
    icon(lucide::plus()).size(16.0)
}

fn trash_icon() -> impl IntoElement {
    icon(lucide::trash()).size(16.0)
}

fn download_icon() -> impl IntoElement {
    icon(lucide::download()).size(16.0)
}
