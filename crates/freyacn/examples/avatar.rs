//! # Avatar Showcase
//!
//! Runnable example covering the full `Avatar` surface — circular by
//! default, in-group `bg-background` ring + subtle shadow + per-index
//! z-stacking, badges, missing files, embedded bytes, clickable
//! avatars, and a long list to force scrolling.
//!
//! ## Setup
//!
//! ```text
//! crates/freyacn/assets/sample.jpg
//! crates/freyacn/assets/sample2.jpg
//! ```
//!
//! Run with:
//!
//! ```bash
//! cargo run --example avatar_showcase
//! ```

use freyacn::prelude::*;

const SAMPLE_PATH: &str =
    "/home/bright/RustroverProjects/freyacn/crates/freyacn/assets/sample.jpg";
const SAMPLE_PATH_2: &str =
    "/home/bright/RustroverProjects/freyacn/crates/freyacn/assets/sample2.jpg";

// ==============================================================
// Layout helpers
// ==============================================================

fn section(title: &str, description: &str, body: impl IntoElement) -> impl IntoElement {
    Div()
        .style("flex flex-col gap-4 p-6 bg-card rounded-xl border border-border shadow-sm")
        .child(
            Label()
                .text(title)
                .style("text-lg font-semibold text-card-foreground"),
        )
        .child(
            Label()
                .text(description)
                .style("text-sm text-muted-foreground"),
        )
        .child(body)
}

fn divider() -> impl IntoElement {
    Div().style("h-px w-full bg-border")
}

fn cell(caption: &str, sample: impl IntoElement) -> impl IntoElement {
    Div()
        .style("flex flex-col items-center gap-2 p-3 rounded-md border border-border bg-muted/40")
        .child(sample)
        .child(
            Label()
                .text(caption)
                .style("text-[10px] font-mono text-muted-foreground"),
        )
}

// ==============================================================
// Sections
// ==============================================================

fn basic_section() -> impl IntoElement {
    section(
        "Basic",
        "Circular by default. Image with initials fallback, image-only, fallback-only, empty.",
        Div()
            .style("flex flex-row flex-wrap items-end gap-4")
            .child(cell(
                "src + alt + fallback",
                Avatar().src(SAMPLE_PATH).alt("Ada Lovelace").fallback("AL"),
            ))
            .child(cell(
                "src only",
                Avatar().src(SAMPLE_PATH).alt("Ada Lovelace"),
            ))
            .child(cell("fallback only", Avatar().fallback("JD")))
            .child(cell("fully empty", Avatar())),
    )
}

fn sizes_section() -> impl IntoElement {
    section(
        "Sizes",
        "`AvatarSize` scales the box and the fallback text together.",
        Div()
            .style("flex flex-row flex-wrap items-end gap-6")
            .child(cell(
                "Xs (24)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("A")
                    .size(AvatarSize::Xs),
            ))
            .child(cell(
                "Md (32)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("A")
                    .size(AvatarSize::Md),
            ))
            .child(cell(
                "Lg (40)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("A")
                    .size(AvatarSize::Lg),
            ))
            .child(cell(
                "Xl (48)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("A")
                    .size(AvatarSize::Xl),
            ))
            .child(cell(
                "Xxl (64)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("A")
                    .size(AvatarSize::Xxl),
            )),
    )
}

fn shapes_section() -> impl IntoElement {
    section(
        "Shapes",
        "`AvatarShape` controls rounding. Content clips to the same shape.",
        Div()
            .style("flex flex-row flex-wrap items-end gap-6")
            .child(cell(
                "Circle (default)",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Xl),
            ))
            .child(cell(
                "Rounded",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Xl)
                    .shape(AvatarShape::Rounded),
            ))
            .child(cell(
                "Square",
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Xl)
                    .shape(AvatarShape::Square),
            )),
    )
}

fn fallback_section() -> impl IntoElement {
    section(
        "Fallback — shortcut vs explicit",
        "`.fallback(\"AL\")` is sugar for `.child(AvatarFallback().child(Label().text(\"AL\")))`.",
        Div()
            .style("flex flex-row flex-wrap items-center gap-4")
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Lg),
            )
            .child(Avatar().fallback("GH").size(AvatarSize::Lg))
            .child(
                Avatar()
                    .child(AvatarFallback().child(Label().text("JD")))
                    .size(AvatarSize::Lg),
            )
            .child(
                Avatar()
                    .child(
                        AvatarFallback()
                            .style("bg-primary text-primary-foreground")
                            .child(Label().text("CU")),
                    )
                    .size(AvatarSize::Lg)
                    .shape(AvatarShape::Rounded),
            ),
    )
}

fn missing_image_section() -> impl IntoElement {
    section(
        "Missing image",
        "A bad `src` falls through to the fallback child (or empty muted box).",
        Div()
            .style("flex flex-row flex-wrap items-center gap-4")
            .child(
                Avatar()
                    .src("this/path/does/not/exist.png")
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Lg),
            )
            .child(Avatar().src("").fallback("GH").size(AvatarSize::Lg))
            .child(
                Avatar()
                    .src("this/path/does/not/exist.png")
                    .size(AvatarSize::Lg),
            ),
    )
}

fn embedded_bytes_section() -> impl IntoElement {
    section(
        "Embedded bytes",
        "`bytes(...)` bypasses the file check — useful for `include_bytes!` or fetched data.",
        Div()
            .style("flex flex-row flex-wrap items-center gap-4")
            .child(
                Avatar()
                    .bytes(std::fs::read(SAMPLE_PATH).map(Bytes::from).ok())
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Lg),
            )
            .child(
                Avatar()
                    .bytes(std::fs::read(SAMPLE_PATH_2).map(Bytes::from).ok())
                    .alt("Grace")
                    .fallback("GH")
                    .size(AvatarSize::Lg)
                    .shape(AvatarShape::Rounded),
            )
            .child(Avatar().bytes(None).fallback("JD").size(AvatarSize::Lg)),
    )
}

fn styled_section() -> impl IntoElement {
    section(
        "Styled wrapper",
        "`bordered()` adds the same 2px `theme.background` ring + subtle shadow used inside a group.",
        Div()
            .style("flex flex-row flex-wrap items-center gap-6")
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Xl)
                    .style("border-4 border-green-500"),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Xl)
                    .style("border-4 border-primary"),
            )
            .child(Avatar().fallback("GH").size(AvatarSize::Xl).bordered())
            .child(Avatar().fallback("JD").size(AvatarSize::Xl).bordered()),
    )
}

fn clickable_section() -> impl IntoElement {
    section(
        "Clickable",
        "`Avatar` implements `EventHandlersExt` — `on_press` fires on image or fallback.",
        Div()
            .style("flex flex-row flex-wrap items-center gap-4")
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Thumbnail 1")
                    .fallback("A")
                    .size(AvatarSize::Lg)
                    .style("cursor-pointer")
                    .on_press(|_| println!("avatar 1 pressed")),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH_2)
                    .alt("Thumbnail 2")
                    .fallback("B")
                    .size(AvatarSize::Lg)
                    .style("cursor-pointer")
                    .on_press(|_| println!("avatar 2 pressed")),
            )
            .child(
                Avatar()
                    .fallback("C")
                    .size(AvatarSize::Lg)
                    .style("cursor-pointer")
                    .on_press(|_| println!("fallback avatar pressed")),
            ),
    )
}

fn badge_colors_section() -> impl IntoElement {
    section(
        "Badges — colors",
        "The badge sits at the bottom-right of the visible avatar. Default color is `bg-primary`.",
        Div()
            .style("flex flex-row flex-wrap items-center gap-6")
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Online")
                    .fallback("ON")
                    .size(AvatarSize::Lg)
                    .badge(AvatarBadge().style("bg-green-500")),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH_2)
                    .alt("Away")
                    .fallback("AW")
                    .size(AvatarSize::Lg)
                    .badge(AvatarBadge().style("bg-amber-400")),
            )
            .child(
                Avatar()
                    .fallback("OF")
                    .size(AvatarSize::Lg)
                    .badge(AvatarBadge().style("bg-red-500")),
            )
            .child(
                Avatar()
                    .fallback("BU")
                    .size(AvatarSize::Lg)
                    .badge(AvatarBadge().style("bg-muted-foreground")),
            ),
    )
}

fn badge_sizes_section() -> impl IntoElement {
    section(
        "Badges — sizes",
        "`AvatarBadgeSize` scales the dot — Sm with Md avatars, Xl with Xxl avatars.",
        Div()
            .style("flex flex-row flex-wrap items-end gap-6")
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Small")
                    .fallback("SM")
                    .size(AvatarSize::Md)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Sm)
                            .style("bg-green-500"),
                    ),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Medium")
                    .fallback("MD")
                    .size(AvatarSize::Lg)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Md)
                            .style("bg-green-500"),
                    ),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Large")
                    .fallback("LG")
                    .size(AvatarSize::Xl)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Lg)
                            .style("bg-green-500"),
                    ),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("XLarge")
                    .fallback("XL")
                    .size(AvatarSize::Xxl)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Xl)
                            .style("bg-amber-400")
                            .child(Label().text("!").style("text-[8px]")),
                    ),
            ),
    )
}

fn group_default_section() -> impl IntoElement {
    section(
        "AvatarGroup — default overlap + ring + shadow",
        "Each avatar gets a 2px `bg-background` ring AND a subtle shadow. Later avatars stack on top via `.z_index(index)`, so rings and shadows cut into and drop onto the previous avatar.",
        Div()
            .style("flex flex-col gap-6")
            .child(
                AvatarGroup()
                    .child(
                        Avatar()
                            .src(SAMPLE_PATH)
                            .alt("Ada")
                            .fallback("AL")
                            .size(AvatarSize::Lg),
                    )
                    .child(
                        Avatar()
                            .src(SAMPLE_PATH_2)
                            .alt("Grace")
                            .fallback("GH")
                            .size(AvatarSize::Lg),
                    )
                    .child(Avatar().fallback("JD").size(AvatarSize::Lg))
                    .child(Avatar().fallback("+5").size(AvatarSize::Lg)),
            )
            .child(
                AvatarGroup()
                    .child(
                        Avatar()
                            .src(SAMPLE_PATH)
                            .alt("Ada")
                            .fallback("A")
                            .size(AvatarSize::Md),
                    )
                    .child(
                        Avatar()
                            .src(SAMPLE_PATH_2)
                            .alt("Grace")
                            .fallback("B")
                            .size(AvatarSize::Md),
                    )
                    .child(Avatar().fallback("C").size(AvatarSize::Md))
                    .child(AvatarGroupCount().size(AvatarSize::Md).count(3)),
            ),
    )
}

fn group_spacing_section() -> impl IntoElement {
    section(
        "AvatarGroup — spacing",
        "Override overlap with `.spacing(...)`. Z-index stacking, rings, and shadows still apply.",
        Div()
            .style("flex flex-col gap-6")
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("spacing(4.0) — 4px gap")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        AvatarGroup()
                            .spacing(4.0)
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH)
                                    .alt("A")
                                    .fallback("A")
                                    .size(AvatarSize::Lg),
                            )
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH_2)
                                    .alt("B")
                                    .fallback("B")
                                    .size(AvatarSize::Lg),
                            )
                            .child(Avatar().fallback("C").size(AvatarSize::Lg)),
                    ),
            )
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("spacing(-8.0) — default overlap")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        AvatarGroup()
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH)
                                    .alt("A")
                                    .fallback("A")
                                    .size(AvatarSize::Lg),
                            )
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH_2)
                                    .alt("B")
                                    .fallback("B")
                                    .size(AvatarSize::Lg),
                            )
                            .child(Avatar().fallback("C").size(AvatarSize::Lg)),
                    ),
            )
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("spacing(-14.0) — heavy overlap")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        AvatarGroup()
                            .spacing(-14.0)
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH)
                                    .alt("A")
                                    .fallback("A")
                                    .size(AvatarSize::Lg),
                            )
                            .child(
                                Avatar()
                                    .src(SAMPLE_PATH_2)
                                    .alt("B")
                                    .fallback("B")
                                    .size(AvatarSize::Lg),
                            )
                            .child(Avatar().fallback("C").size(AvatarSize::Lg)),
                    ),
            ),
    )
}

fn group_with_badges_section() -> impl IntoElement {
    section(
        "AvatarGroup + badges",
        "Badges, rings, shadows, and z-stacking all compose.",
        AvatarGroup()
            .child(
                Avatar()
                    .src(SAMPLE_PATH)
                    .alt("Ada")
                    .fallback("AL")
                    .size(AvatarSize::Lg)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Sm)
                            .style("bg-green-500"),
                    ),
            )
            .child(
                Avatar()
                    .src(SAMPLE_PATH_2)
                    .alt("Grace")
                    .fallback("GH")
                    .size(AvatarSize::Lg)
                    .badge(
                        AvatarBadge()
                            .size(AvatarBadgeSize::Sm)
                            .style("bg-amber-400"),
                    ),
            )
            .child(
                Avatar()
                    .fallback("JD")
                    .size(AvatarSize::Lg)
                    .badge(AvatarBadge().size(AvatarBadgeSize::Sm).style("bg-red-500")),
            )
            .child(AvatarGroupCount().size(AvatarSize::Lg).count(5)),
    )
}

fn inside_div_section() -> impl IntoElement {
    section(
        "Inside a Div",
        "Avatars compose with text.",
        Div()
            .style("flex flex-col gap-3")
            .child(
                Div()
                    .style("flex flex-row items-center gap-3 p-3 rounded-md border border-border bg-card")
                    .child(
                        Avatar()
                            .src(SAMPLE_PATH)
                            .alt("Ada")
                            .fallback("AL")
                            .size(AvatarSize::Lg)
                            .badge(AvatarBadge().style("bg-green-500")),
                    )
                    .child(
                        Div()
                            .style("flex flex-col")
                            .child(Label().text("Ada Lovelace").style("text-sm font-medium"))
                            .child(
                                Label()
                                    .text("Mathematician")
                                    .style("text-xs text-muted-foreground"),
                            ),
                    ),
            )
            .child(
                Div()
                    .style("flex flex-row items-center gap-3 p-3 rounded-md border border-border bg-card")
                    .child(
                        Avatar()
                            .fallback("GH")
                            .size(AvatarSize::Lg)
                            .shape(AvatarShape::Rounded),
                    )
                    .child(
                        Div()
                            .style("flex flex-col")
                            .child(
                                Label().text("Grace Hopper").style("text-sm font-medium"),
                            )
                            .child(
                                Label()
                                    .text("Computer scientist")
                                    .style("text-xs text-muted-foreground"),
                            ),
                    ),
            ),
    )
}

fn long_list_section() -> impl IntoElement {
    section(
        "A long list",
        "Enough rows to force the page to scroll.",
        Div()
            .style("flex flex-col gap-2")
            .children((0..30).map(|i| {
                let initials = match i % 4 {
                    0 => "AL",
                    1 => "GH",
                    2 => "JD",
                    _ => "MK",
                };
                let badge_color = match i % 3 {
                    0 => "bg-green-500",
                    1 => "bg-amber-400",
                    _ => "bg-muted-foreground",
                };
                Div()
                .style(
                    "flex flex-row items-center gap-3 p-2 rounded-md border border-border bg-card",
                )
                .child(
                    Avatar()
                        .src(if i % 2 == 0 {
                            SAMPLE_PATH
                        } else {
                            SAMPLE_PATH_2
                        })
                        .alt(format!("User {i}"))
                        .fallback(initials)
                        .size(AvatarSize::Md)
                        .badge(AvatarBadge().size(AvatarBadgeSize::Sm).style(badge_color)),
                )
                .child(Label().text(format!("User {i}")))
                .into_element()
            })),
    )
}

// ==============================================================
// App
// ==============================================================

fn app() -> impl IntoElement {
    use_init_cn_theme(Theme::light().neutral().theme_color("neutral"));

    Div()
        .style("flex flex-col w-full h-screen bg-background overflow-scroll text-foreground")
        .child(
            Div()
                .style("flex flex-col gap-8 max-w-3xl w-full mx-auto p-8")
                .child(
                    Div()
                        .style("flex flex-col gap-2")
                        .child(
                            Label()
                                .text("Avatar")
                                .style("text-3xl font-bold text-foreground"),
                        )
                        .child(
                            Label()
                                .text("`Avatar().src(\"photo.jpg\").fallback(\"AL\")` — circular by default, badges in the corner, 2px `bg-background` ring + subtle shadow + z-stacking inside an AvatarGroup.")
                                .style("text-base text-muted-foreground"),
                        ),
                )
                .child(divider())
                .child(basic_section())
                .child(sizes_section())
                .child(shapes_section())
                .child(fallback_section())
                .child(missing_image_section())
                .child(embedded_bytes_section())
                .child(styled_section())
                .child(clickable_section())
                .child(badge_colors_section())
                .child(badge_sizes_section())
                .child(group_default_section())
                .child(group_spacing_section())
                .child(group_with_badges_section())
                .child(inside_div_section())
                .child(divider())
                .child(long_list_section()),
        )
}

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)));
}
