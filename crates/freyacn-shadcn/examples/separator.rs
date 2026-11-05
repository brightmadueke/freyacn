//! shadcn Separator showcase.
//!
//! Run with:
//!
//! ```text
//! cargo run -p freyacn-shadcn --example separator
//! ```
//!
//! Points at `<workspace>/resources/themes/v1`.

use freya::prelude::*;
use freyacn_core::prelude::{IntoElement, div, text};
use freyacn_core::styling::StyledExt;
use freyacn_core::theme::load_themes_with_default;
use freyacn_shadcn::components::separator::{Separator, SeparatorOrientation};

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    // Themes live at `<workspace>/resources/themes/v1`. `CARGO_MANIFEST_DIR`
    // is baked in at compile time as the *crate* directory
    // (`<workspace>/crates/freyacn-shadcn`), so `../../` reaches the
    // workspace root regardless of the process's working directory.
    // This is the *relative* path from the crate — stable across machines
    // and CI, unlike a bare `"resources/themes/v1"` which would only
    // resolve if the process was launched from the workspace root.
    let themes_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("resources/themes/v1");

    // `load_themes_with_default` is itself a hook — it wraps its internals
    // in `use_hook`, so the directory is read once and cached across
    // re-renders. Do NOT wrap this call in another `use_hook`.
    load_themes_with_default(&themes_dir, "default-dark");

    div()
        .style("size-full bg-background")
        .child(separator_showcase())
        .scroll_y()
}

fn separator_showcase() -> impl IntoElement {
    div()
        .style("flex-col gap-8 p-8 max-w-3xl")
        // ── Header ──────────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-1")
                .child(text("Separator").style("text-3xl font-bold"))
                .child(
                    text("Horizontal and vertical dividers, decorative and semantic")
                        .style("text-sm text-gray-500"),
                ),
        )
        // ── Basic horizontal ────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Horizontal").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-col gap-4")
                        .child(text("Section above").style("text-sm"))
                        .child(Separator())
                        .child(text("Section below").style("text-sm")),
                ),
        )
        // ── Basic vertical ──────────────────────────────────────────
        //
        // Vertical separators need a *bounded* height to have a visible
        // 1px width. `items-stretch` (the default for `flex-row`) gives
        // the separator the row's full height via `h-full` in its variant
        // class string. If the parent uses `items-center` instead, the
        // separator collapses to zero height — pin it with an explicit
        // `.style("h-6")` or similar on the separator itself.
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Vertical").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-row items-center gap-3 h-6")
                        .child(text("Home").style("text-sm"))
                        .child(Separator().orientation(SeparatorOrientation::Vertical))
                        .child(text("Docs").style("text-sm"))
                        .child(Separator().orientation(SeparatorOrientation::Vertical))
                        .child(text("Pricing").style("text-sm")),
                ),
        )
        // ── In a card layout ────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-3")
                .child(text("Card layout").style("text-xs font-semibold text-gray-500"))
                .child(
                    div()
                        .style("flex-col rounded-lg border border-border bg-card p-4 gap-3")
                        .child(text("Card title").style("text-base font-semibold"))
                        .child(
                            text("A short description beneath the title, separated from the content below.")
                                .style("text-sm text-gray-500"),
                        )
                        .child(Separator())
                        .child(
                            div()
                                .style("flex-row items-center gap-2 text-sm")
                                .child(text("Body content"))
                                .child(Separator().orientation(SeparatorOrientation::Vertical))
                                .child(text("Another item")),
                        ),
                ),
        )
        // ── Decorative vs semantic ──────────────────────────────────
        //
        // `decorative = true` (default) hides the separator from screen
        // readers — it's purely a visual cue.
        //
        // `decorative = false` exposes it as a semantic boundary. Screen
        // readers announce it, and the surrounding content is expected
        // to be navigable as sections.
        .child(
            div()
                .style("flex-col gap-3")
                .child(
                    text("Decorative vs semantic")
                        .style("text-xs font-semibold text-gray-500"),
                )
                .child(
                    div()
                        .style("flex-col gap-4 p-4 rounded-lg border border-border")
                        .child(
                            text("Decorative (default) — no a11y role")
                                .style("text-xs font-medium"),
                        )
                        .child(Separator())
                        .child(
                            text("Semantic — role = Splitter, announced by screen readers")
                                .style("text-xs font-medium"),
                        )
                        .child(Separator().decorative(false))
                        .child(
                            text("(After a semantic separator, content is treated as a distinct section.)")
                                .style("text-xs text-gray-500"),
                        ),
                ),
        )
        // ── Custom styling ──────────────────────────────────────────
        //
        // `bg-border` is the base colour; a user `.style(..)` overrides
        // it. Margin and thickness overrides are common — an
        // emphasised divider, a coloured accent, or a thicker rule.
        .child(
            div()
                .style("flex-col gap-3")
                .child(
                    text("Custom styling")
                        .style("text-xs font-semibold text-gray-500"),
                )
                .child(
                    div()
                        .style("flex-col gap-4")
                        .child(
                            text("Default border colour, my-4 spacing:")
                                .style("text-xs text-gray-500"),
                        )
                        .child(text("Above").style("text-sm"))
                        .child(Separator().style("my-4"))
                        .child(text("Below").style("text-sm"))
                        .child(
                            text("Emphasised — 2px, primary colour:")
                                .style("text-xs text-gray-500 mt-2"),
                        )
                        .child(text("Above").style("text-sm"))
                        .child(Separator().style("h-[2px] bg-primary my-2"))
                        .child(text("Below").style("text-sm"))
                        .child(
                            text("Faded — muted colour:")
                                .style("text-xs text-gray-500 mt-2"),
                        )
                        .child(text("Above").style("text-sm"))
                        .child(Separator().style("bg-muted opacity-60 my-2"))
                        .child(text("Below").style("text-sm")),
                ),
        )
        // ── Orientation switch ──────────────────────────────────────
        //
        // Both orientations accept the same `.style(..)` overrides, so
        // a single `separator()` helper in your app can branch on a
        // prop without duplicating the style chain.
        .child(
            div()
                .style("flex-col gap-3")
                .child(
                    text("Orientation switch")
                        .style("text-xs font-semibold text-gray-500"),
                )
                .child(
                    div()
                        .style("flex-row items-center gap-3 h-6")
                        .child(text("Alpha").style("text-sm"))
                        .child(
                            Separator()
                                .orientation(SeparatorOrientation::Vertical)
                                .style("bg-primary/40"),
                        )
                        .child(text("Beta").style("text-sm"))
                        .child(
                            Separator()
                                .orientation(SeparatorOrientation::Vertical)
                                .style("bg-primary/40"),
                        )
                        .child(text("Gamma").style("text-sm")),
                ),
        )
}

// Suppress unused warnings for the imports pulled in by the module-level
// style helpers used above — the file works if you trim it down, but
// keeping the imports aligned with what the example exercises avoids
// surprise when copy-pasting subsets.
#[allow(dead_code)]
fn _uses_styled_ext() {
    // `StyledExt` is in scope from `use freyacn_core::styling::StyledExt;`
    // so `.style(..)`, `.bg(..)`, `.h(..)`, `.opacity(..)` resolve. This
    // shim exists purely to keep the import marked as used when the
    // example is trimmed down to a section.
}
