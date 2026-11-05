//! shadcn Input showcase.
//!
//! Run with:
//!
//! ```text
//! cargo run -p freyacn-shadcn --example input
//! ```

use freya::prelude::*;
use freyacn_core::elements::input::InputMode;
use freyacn_core::prelude::{IntoElement, div, text};
use freyacn_core::styling::{HasSemanticEvents, StyledExt};
use freyacn_core::theme::load_themes_with_default;
use freyacn_shadcn::components::button::Button;
use freyacn_shadcn::components::input::Input;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    let themes_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("resources/themes/v1");
    load_themes_with_default(&themes_dir, "default-dark");

    div()
        .style("size-full bg-background")
        .child(inputs_showcase())
        .scroll_y()
}

fn inputs_showcase() -> impl IntoElement {
    div()
        .style("flex-col gap-8 p-8 max-w-md")
        .child(
            div()
                .style("flex-col gap-1")
                .child(text("Input").style("text-3xl font-bold"))
                .child(
                    text("A single-line text input for forms, search, and credentials.")
                        .style("text-sm text-muted-foreground"),
                ),
        )
        .child(section(
            "Uncontrolled",
            "Just an input. Type and it works — no state wiring required.",
            Input().placeholder("you@example.com"),
        ))
        .child(controlled_example())
        .child(section(
            "Password",
            "Masks every character with •.",
            Input().mode(InputMode::password()).placeholder("Password"),
        ))
        .child(section(
            "Max length (10)",
            "The input stops accepting characters after 10.",
            Input().max_length(10usize).placeholder("Up to 10 chars"),
        ))
        .child(section(
            "Disabled",
            "Not focusable, not typeable. Cursor becomes NotAllowed on hover.",
            Input().value("Read-only value").with_disabled(true),
        ))
        .child(section(
            "Read-only",
            "Can be focused and selected, but not edited.",
            Input().value("user_3f8a9c2d").with_read_only(true),
        ))
        .child(section(
            "Invalid",
            "Red border signals a validation error via aria-invalid.",
            Input().value("not-an-email").with_aria_invalid(true),
        ))
        .child(form_example())
        .child(search_example())
}

fn section(
    title: &'static str,
    caption: &'static str,
    child: impl IntoElement,
) -> impl IntoElement {
    div()
        .style("flex-col gap-2")
        .child(text(title).style("text-sm font-semibold"))
        .child(text(caption).style("text-xs text-muted-foreground"))
        .child(child)
}

// ── Controlled ──────────────────────────────────────────────────────

fn controlled_example() -> impl IntoElement {
    let mut email = use_state(String::new);

    let current: String = (*email.read()).clone();
    let char_count: usize = current.chars().count();

    div()
        .style("flex-col gap-2")
        .child(text("Controlled").style("text-sm font-semibold"))
        .child(
            text("The parent owns the value. The counter updates as you type.")
                .style("text-xs text-muted-foreground"),
        )
        .child(
            Input()
                .value(current)
                .placeholder("you@example.com")
                .on_change(move |text: String| email.set(text)),
        )
        .child(
            text(format!(
                "{} character{}",
                char_count,
                if char_count == 1 { "" } else { "s" },
            ))
            .style("text-xs text-muted-foreground"),
        )
}

// ── Form with submit ────────────────────────────────────────────────

fn form_example() -> impl IntoElement {
    let mut email = use_state(String::new);
    let mut password = use_state(String::new);
    let mut submitted = use_state(String::new);

    let email_v: String = (*email.read()).clone();
    let password_v: String = (*password.read()).clone();
    let submitted_v: String = (*submitted.read()).clone();
    let can_submit: bool = !email_v.is_empty() && !password_v.is_empty();

    let email_for_btn = email.clone();
    let password_for_btn = password.clone();
    let mut submitted_for_btn = submitted.clone();

    div()
        .style("flex-col gap-2")
        .child(text("Form with submit").style("text-sm font-semibold"))
        .child(
            text("Press Enter in either field, or click Sign in.")
                .style("text-xs text-muted-foreground"),
        )
        .child(
            div()
                .style("flex-col gap-3")
                .child(
                    div()
                        .style("flex-col gap-1.5")
                        .child(text("Email").style("text-sm font-medium leading-none"))
                        .child(
                            Input()
                                .value(email_v)
                                .placeholder("m@example.com")
                                .on_change({
                                    let mut email = email.clone();
                                    move |text: String| email.set(text)
                                })
                                .on_submit({
                                    let mut submitted = submitted.clone();
                                    let password_for_submit = password.clone();
                                    let email_for_submit = email.clone();
                                    move |_text: String| {
                                        let em: String = (*email_for_submit.read()).clone();
                                        let pw: String = (*password_for_submit.read()).clone();
                                        if !em.is_empty() && !pw.is_empty() {
                                            submitted.set(format!(
                                                "Submitted: {} + (password, {} chars)",
                                                em,
                                                pw.chars().count(),
                                            ));
                                        }
                                    }
                                }),
                        ),
                )
                .child(
                    div()
                        .style("flex-col gap-1.5")
                        .child(text("Password").style("text-sm font-medium leading-none"))
                        .child(
                            Input()
                                .value(password_v)
                                .mode(InputMode::password())
                                .placeholder("Password")
                                .on_change({
                                    let mut password = password.clone();
                                    move |text: String| password.set(text)
                                })
                                .on_submit({
                                    let mut submitted = submitted.clone();
                                    let email_for_submit = email.clone();
                                    move |text: String| {
                                        let em: String = (*email_for_submit.read()).clone();
                                        if !em.is_empty() {
                                            submitted.set(format!(
                                                "Submitted: {} + (password, {} chars)",
                                                em,
                                                text.chars().count(),
                                            ));
                                        }
                                    }
                                }),
                        ),
                )
                .child(
                    Button()
                        .style("w-full")
                        .with_disabled(!can_submit)
                        .on_press(move |_| {
                            let em: String = (*email_for_btn.read()).clone();
                            let pw: String = (*password_for_btn.read()).clone();
                            if !em.is_empty() && !pw.is_empty() {
                                submitted_for_btn.set(format!(
                                    "Submitted: {} + (password, {} chars)",
                                    em,
                                    pw.chars().count(),
                                ));
                            }
                        })
                        .child(text("Sign in")),
                ),
        )
        .child(
            text(if submitted_v.is_empty() {
                "Not submitted yet.".to_string()
            } else {
                submitted_v
            })
            .style("text-xs text-muted-foreground"),
        )
}

// ── Search ──────────────────────────────────────────────────────────

fn search_example() -> impl IntoElement {
    let mut query = use_state(String::new);
    let current: String = (*query.read()).clone();
    let matches: Vec<&'static str> = filter_items(current.as_str());

    div()
        .style("flex-col gap-2")
        .child(text("Search").style("text-sm font-semibold"))
        .child(
            text("Filters a small list as you type. Clear the input to reset.")
                .style("text-xs text-muted-foreground"),
        )
        .child(
            Input()
                .value(current)
                .placeholder("Search…")
                .on_change(move |text: String| query.set(text)),
        )
        .child(
            div()
                .style("flex-col gap-1 mt-2")
                .children(matches.into_iter().map(|item: &'static str| {
                    div()
                        .style("px-3 py-1.5 rounded-md bg-muted text-sm")
                        .child(text(item))
                })),
        )
}

fn filter_items(query: &str) -> Vec<&'static str> {
    const ITEMS: &[&'static str] = &[
        "Buttons",
        "Cards",
        "Checkboxes",
        "Dialogs",
        "Dropdowns",
        "Forms",
        "Inputs",
        "Menus",
        "Popovers",
        "Radios",
        "Selects",
        "Separators",
        "Tabs",
        "Tooltips",
    ];

    if query.is_empty() {
        return ITEMS.to_vec();
    }

    let needle: String = query.to_lowercase();
    ITEMS
        .iter()
        .filter(|item| item.to_lowercase().contains(needle.as_str()))
        .copied()
        .collect()
}
