//! # Image Showcase
//!
//! Demonstrates the path-based `Image()` — `.src("path/to/photo.png")`
//! reads, decodes, caches, and renders, matching HTML `<img src="…">`.
//!
//! Covers: default sizing, all four `object-fit` modes both as a
//! kebab-case `.fit("…")` string and as tailwind-style `object-*`
//! tokens in `style("…")`, rounded corners, borders, shadows, sizing,
//! all five sampling modes, blur (both as a prop and as `blur-*`
//! tokens), snap-to-grid, clickable images, embedded bytes via
//! `include_bytes!`, a missing-file fallback, and a long list to force
//! the page to scroll.
//!
//! ## Setup
//!
//! The demo looks for these files relative to the crate root. Any PNG
//! or JPEG works; if they're missing, the fallback placeholder renders
//! and the showcase still runs.
//!
//! ```text
//! crates/freyacn/assets/sample.jpg
//! crates/freyacn/assets/sample2.jpg
//! ```
//!
//! Run with:
//!
//! ```bash
//! cargo run --example image_showcase
//! ```

use freyacn::prelude::*;

// ==============================================================
// Demo assets
// ==============================================================
//
// Point these at any PNG or JPEG on disk. If the file is missing, the
// `Image()` fallback renders a muted placeholder — the showcase still
// runs, it just shows empty boxes.
//
// Change these to absolute paths or to files you know exist on your
// machine (e.g. "/home/you/Pictures/photo.jpg") for a real demo.

const SAMPLE_PATH: &str = "/home/bright/RustroverProjects/freyacn/crates/freyacn/assets/sample.jpg";
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
    Div().style("h-[1px] w-full bg-border")
}

fn cell(caption: &str, sample: impl IntoElement) -> impl IntoElement {
    Div()
        .style("flex flex-col gap-2 p-3 rounded-md border border-border bg-muted/40")
        .child(
            Label()
                .text(caption)
                .style("text-[10px] font-mono text-muted-foreground"),
        )
        .child(sample)
}

// ==============================================================
// Sections
// ==============================================================

fn basic_section() -> impl IntoElement {
    section(
        "Basic",
        "The smallest possible usage — a path and nothing else. The image renders at its intrinsic size.",
        Image().src(SAMPLE_PATH).alt("Sample image"),
    )
}

/// `.fit()` takes a kebab-case string — `"fill"`, `"cover"`,
/// `"contain"`, `"none"`. Unknown values fall back to `"fill"`, which
/// is also the default when `fit` is omitted.
fn fit_section() -> impl IntoElement {
    section(
        "Object fit",
        "`.fit()` takes a kebab-case string — fill, cover, contain, none. Unknown values fall back to fill. The conversion to `ImageFit` happens inside the component body.",
        Div()
            .style("flex flex-col gap-6")
            .child(
                Div()
                    .style("flex flex-row flex-wrap gap-4 items-start")
                    .child(cell(
                        "fill (stretch)",
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("fill")
                            .style("w-32 h-20 rounded-md border border-border"),
                    ))
                    .child(cell(
                        "cover (crop, centre)",
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("cover")
                            .style("w-32 h-20 rounded-md border border-border"),
                    ))
                    .child(cell(
                        "contain (fit inside)",
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("contain")
                            .style("w-32 h-20 rounded-md border border-border bg-muted"),
                    ))
                    .child(cell(
                        "none (natural)",
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("none")
                            .style("w-32 h-20 rounded-md border border-border bg-muted"),
                    )),
            )
            // Unknown fallback
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("Unknown value falls back to fill — .fit(\"banana\")")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("banana")
                            .style("w-32 h-20 rounded-md border border-border"),
                    ),
            ),
    )
}

/// The new tailwind-style `object-*` tokens. These live in `stylesheet`
/// and take precedence over the `fit` prop because the style string is
/// the more specific declaration.
fn fit_token_section() -> impl IntoElement {
    section(
        "Object fit — tailwind token",
        "`object-fill` / `object-cover` / `object-contain` / `object-none` / `object-scale-down` in a `style(\"…\")` string. The token wins over the `fit` prop.",
        Div()
            .style("flex flex-col gap-4")
            .child(
                Div()
                    .style("flex flex-row flex-wrap gap-4 items-start")
                    .child(cell(
                        "object-fill",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md border border-border object-fill"),
                    ))
                    .child(cell(
                        "object-cover",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md border border-border object-cover"),
                    ))
                    .child(cell(
                        "object-contain",
                        Image().src(SAMPLE_PATH).style(
                            "w-32 h-20 rounded-md border border-border bg-muted object-contain",
                        ),
                    ))
                    .child(cell(
                        "object-none",
                        Image().src(SAMPLE_PATH).style(
                            "w-32 h-20 rounded-md border border-border bg-muted object-none",
                        ),
                    ))
                    .child(cell(
                        "object-scale-down",
                        Image().src(SAMPLE_PATH).style(
                            "w-32 h-20 rounded-md border border-border bg-muted object-scale-down",
                        ),
                    )),
            )
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("Token wins over prop — .fit(\"fill\").style(\"object-cover\")")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("fill")
                            .style("w-32 h-20 rounded-md border border-border object-cover"),
                    ),
            ),
    )
}

fn radius_section() -> impl IntoElement {
    section(
        "Corner radius",
        "`Image` inherits `DecorationExt::corner_radius` directly.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-end")
            .child(cell(
                "rounded-md",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-24 h-24 rounded-md"),
            ))
            .child(cell(
                "rounded-xl",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-24 h-24 rounded-xl"),
            ))
            .child(cell(
                "rounded-full",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-24 h-24 rounded-full"),
            )),
    )
}

fn border_shadow_section() -> impl IntoElement {
    section(
        "Borders and shadows",
        "`DecorationExt::border` and `DecorationExt::shadow` work on `Image` directly.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start p-4")
            .child(
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-40 h-24 rounded-lg border-2 border-primary"),
            )
            .child(
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-40 h-24 rounded-lg border border-border shadow-lg"),
            )
            .child(
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-40 h-24 rounded-lg shadow-2xl"),
            ),
    )
}

fn sizing_section() -> impl IntoElement {
    section(
        "Sizing",
        "Size the image directly. `width` and `height` come from the `ContainerSizeExt` blanket impl.",
        Div()
            .style("flex flex-col gap-4")
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("w-full h-32 (fills width)")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("cover")
                            .style("w-full h-32 rounded-lg border border-border"),
                    ),
            )
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("w-48 h-48 (square)")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .fit("cover")
                            .style("w-48 h-48 rounded-lg border border-border"),
                    ),
            ),
    )
}

fn sampling_section() -> impl IntoElement {
    section(
        "Sampling modes",
        "All five of Freya's `SamplingMode` variants. `Trilinear` is Freya's default.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start")
            .child(cell(
                "Nearest (pixel art)",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .sampling(SamplingMode::Nearest)
                    .style("w-32 h-20 rounded-md border border-border"),
            ))
            .child(cell(
                "Bilinear",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .sampling(SamplingMode::Bilinear)
                    .style("w-32 h-20 rounded-md border border-border"),
            ))
            .child(cell(
                "Trilinear (default)",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .sampling(SamplingMode::Trilinear)
                    .style("w-32 h-20 rounded-md border border-border"),
            ))
            .child(cell(
                "Mitchell",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .sampling(SamplingMode::Mitchell)
                    .style("w-32 h-20 rounded-md border border-border"),
            ))
            .child(cell(
                "CatmullRom",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .sampling(SamplingMode::CatmullRom)
                    .style("w-32 h-20 rounded-md border border-border"),
            )),
    )
}

fn blur_prop_section() -> impl IntoElement {
    section(
        "Blur — typed prop",
        "Element-self blur — CSS `filter: blur()`. Unlike `Rect`, `Image` supports this directly.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start")
            .child(cell(
                "no blur",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .style("w-32 h-20 rounded-md"),
            ))
            .child(cell(
                "blur(2.0)",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .blur(2.0_f32)
                    .style("w-32 h-20 rounded-md"),
            ))
            .child(cell(
                "blur(4.0)",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("cover")
                    .blur(4.0_f32)
                    .style("w-32 h-20 rounded-md"),
            )),
    )
}

/// `blur-*` tokens in a style string feed `ImageData::blur` and win
/// over the `blur` prop. Radius values are the ones configured in
/// `stylesheet`: `sm`=4, `md`=12, `lg`=16, `xl`=24.
fn blur_token_section() -> impl IntoElement {
    section(
        "Blur — tailwind token",
        "`blur-sm` / `blur-md` / `blur-lg` / `blur-xl` in a `style(\"…\")` string. The token wins over the `blur` prop.",
        Div()
            .style("flex flex-col gap-4")
            .child(
                Div()
                    .style("flex flex-row flex-wrap gap-4 items-start")
                    .child(cell(
                        "no blur",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md object-cover"),
                    ))
                    .child(cell(
                        "blur-sm (4)",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md object-cover blur-sm"),
                    ))
                    .child(cell(
                        "blur-md (12)",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md object-cover blur-md"),
                    ))
                    .child(cell(
                        "blur-lg (16)",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md object-cover blur-lg"),
                    ))
                    .child(cell(
                        "blur-xl (24)",
                        Image()
                            .src(SAMPLE_PATH)
                            .style("w-32 h-20 rounded-md object-cover blur-xl"),
                    )),
            )
            .child(
                Div()
                    .style("flex flex-col gap-2")
                    .child(
                        Label()
                            .text("Token wins over prop — .blur(0.0).style(\"blur-md\")")
                            .style("text-xs font-mono text-muted-foreground"),
                    )
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .blur(0.0_f32)
                            .style("w-32 h-20 rounded-md object-cover blur-md"),
                    ),
            ),
    )
}

fn snap_section() -> impl IntoElement {
    section(
        "Snap to grid",
        "Freya's `Image` defaults to `snap_to_grid = false`. Opt in for pixel-art scaling.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start")
            .child(cell(
                "snap_to_grid(false) — default",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .style("w-32 h-20 rounded-md border border-border"),
            ))
            .child(cell(
                "snap_to_grid(true)",
                Image()
                    .src(SAMPLE_PATH)
                    .fit("fill")
                    .snap_to_grid(true)
                    .style("w-32 h-20 rounded-md border border-border"),
            )),
    )
}

fn embedded_bytes_section() -> impl IntoElement {
    section(
        "Embedded bytes",
        "`bytes(...)` bypasses the filesystem and decodes directly — useful for `include_bytes!`-embedded assets.",
        // If you have an `assets/logo.png` in the crate, `include_bytes!`
        // would look like:
        //
        //     .bytes(Bytes::from_static(include_bytes!("../assets/logo.png")))
        //
        // For the demo we read from the same path as `src` so the
        // section works even without an embedded asset.
        Image()
            .bytes(std::fs::read(SAMPLE_PATH).map(Bytes::from).ok())
            .fit("cover")
            .style("w-48 h-32 rounded-lg border border-border"),
    )
}

fn missing_file_section() -> impl IntoElement {
    section(
        "Missing file",
        "When `src` doesn't exist or fails to decode, `Image()` renders a muted placeholder sized to the caller's constraints — the layout doesn't collapse.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start")
            .child(
                Image()
                    .src("this/path/does/not/exist.png")
                    .style("w-32 h-20 rounded-md border border-border"),
            )
            .child(
                Image()
                    .src("")
                    .style("w-32 h-20 rounded-md border border-border"),
            ),
    )
}

fn clickable_section() -> impl IntoElement {
    section(
        "Clickable images",
        "`Image` implements `EventHandlersExt`, so `on_press` and `on_hover` fire on the image directly.",
        Div()
            .style("flex flex-row flex-wrap gap-4")
            .child(
                Image()
                    .src(SAMPLE_PATH)
                    .alt("Thumbnail 1")
                    .fit("cover")
                    .style("w-24 h-24 rounded-lg border border-border cursor-pointer")
                    .on_press(|_| println!("thumbnail 1 pressed")),
            )
            .child(
                Image()
                    .src(SAMPLE_PATH_2)
                    .alt("Thumbnail 2")
                    .fit("cover")
                    .style("w-24 h-24 rounded-lg border border-border cursor-pointer")
                    .on_press(|_| println!("thumbnail 2 pressed")),
            ),
    )
}

fn inside_div_section() -> impl IntoElement {
    section(
        "Inside a `Div`",
        "`Image` has no background, so wrap it in a `Div` when you want padding + a background behind it.",
        Div()
            .style("flex flex-row flex-wrap gap-4 items-start")
            .child(
                Div().style("p-2 bg-muted rounded-xl").child(
                    Image()
                        .src(SAMPLE_PATH)
                        .alt("Framed image")
                        .fit("cover")
                        .style("w-40 h-24 rounded-lg"),
                ),
            )
            .child(
                Div()
                    .style("p-1 bg-primary/10 rounded-full border border-primary/40")
                    .child(
                        Image()
                            .src(SAMPLE_PATH)
                            .alt("Avatar")
                            .fit("cover")
                            .style("w-20 h-20 rounded-full"),
                    ),
            ),
    )
}

fn gallery_section() -> impl IntoElement {
    section(
        "Gallery",
        "A grid of clickable thumbnails. Each `Image` caches its own decoded handle keyed on `src`.",
        Div()
            .style("grid-cols-4 gap-3 w-full")
            .children((1..=8).map(|i| {
                Image()
                    .src(SAMPLE_PATH)
                    .alt(format!("Thumbnail {i}"))
                    .style(
                        "w-full h-20 rounded-md border border-border cursor-pointer object-cover",
                    )
                    .on_press(move |_| println!("gallery {i} pressed"))
            })),
    )
}

fn long_list_section() -> impl IntoElement {
    section(
        "A long list",
        "Enough images to force the page to scroll.",
        Div()
            .style("flex flex-col gap-2")
            .children((0..30).map(|i| {
                Div()
                .style(
                    "flex flex-row items-center gap-3 p-2 rounded-md border border-border bg-card",
                )
                .child(
                    Image()
                        .src(SAMPLE_PATH)
                        .alt(format!("Item {i}"))
                        .style("w-12 h-12 rounded-md object-cover"),
                )
                .child(Label().text(format!("Image item {}", i + 1)))
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
                                .text("Image")
                                .style("text-3xl font-bold text-foreground"),
                        )
                        .child(
                            Label()
                                .text("`Image().src(\"path/to/photo.png\")` — reads, decodes, caches, and renders. HTML `<img>` semantics on top of Freya 0.5.0-rc.10.")
                                .style("text-base text-muted-foreground"),
                        ),
                )
                .child(divider())
                .child(basic_section())
                .child(fit_section())
                .child(fit_token_section())
                .child(radius_section())
                .child(border_shadow_section())
                .child(sizing_section())
                .child(sampling_section())
                .child(blur_prop_section())
                .child(blur_token_section())
                .child(snap_section())
                .child(embedded_bytes_section())
                .child(missing_file_section())
                .child(clickable_section())
                .child(inside_div_section())
                .child(gallery_section())
                .child(divider())
                .child(long_list_section()),
        )
}

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)));
}
