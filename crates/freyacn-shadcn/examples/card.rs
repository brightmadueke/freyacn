//! shadcn Card showcase.
//!
//! Mirrors shadcn/ui's Card examples: login form, notifications, team
//! members, and subscription plan.
//!
//! Run with:
//!
//! ```text
//! cargo run -p freyacn-shadcn --example card
//! ```

use freya::prelude::*;
use freyacn_core::elements::input::InputMode;
use freyacn_core::prelude::{IntoElement, div, text};
use freyacn_core::styling::StyledExt;
use freyacn_core::theme::load_themes_with_default;
use freyacn_shadcn::components::button::{Button, ButtonVariant};
use freyacn_shadcn::components::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use freyacn_shadcn::components::input::Input;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    // `<workspace>/resources/themes/v1`, resolved from the crate dir at
    // compile time so the example works regardless of the process CWD.
    let themes_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("resources/themes/v1");

    load_themes_with_default(&themes_dir, "default-dark");

    div()
        .style("size-full bg-background")
        .child(cards_showcase())
        .scroll_y()
}

fn cards_showcase() -> impl IntoElement {
    div()
        .style("flex-col gap-8 p-8 items-center")
        // ── Page header ─────────────────────────────────────────────
        .child(
            div()
                .style("flex-col gap-1 items-center")
                .child(text("Card").style("text-3xl font-bold"))
                .child(
                    text("Displays a card with header, content, and footer.")
                        .style("text-sm text-muted-foreground"),
                ),
        )
        // ── Row 1: Login + Notifications ────────────────────────────
        .child(
            div()
                .style("flex-row gap-6 items-start")
                .child(login_card())
                .child(notifications_card()),
        )
        // ── Row 2: Team + Subscription ──────────────────────────────
        .child(
            div()
                .style("flex-row gap-6 items-start")
                .child(team_members_card())
                .child(subscription_card()),
        )
}

// ── Login ───────────────────────────────────────────────────────────

fn login_card() -> impl IntoElement {
    let mut email = use_state(String::new);
    let mut password = use_state(String::new);

    Card()
        .style("w-[380px]")
        .child(
            CardHeader()
                .style("flex-row justify-between items-start")
                .child(
                    div()
                        .style("flex-col gap-1.5")
                        .child(CardTitle().child(text("Login to your account")))
                        .child(
                            CardDescription()
                                .child(text("Enter your email below to login to your account")),
                        ),
                )
                .child(CardAction().child(
                    text("Sign Up").style("text-primary text-sm font-medium cursor-pointer"),
                )),
        )
        .child(
            CardContent().child(
                div()
                    .style("flex-col gap-4")
                    .child(
                        div()
                            .style("flex-col gap-2")
                            .child(text("Email").style("text-sm font-medium leading-none"))
                            .child(
                                Input()
                                    .value(email.read().to_string())
                                    .placeholder("m@example.com")
                                    .on_change(move |text| email.set(text)),
                            ),
                    )
                    .child(
                        div()
                            .style("flex-col gap-2")
                            .child(
                                div()
                                    .style("flex-row items-center justify-between")
                                    .child(
                                        text("Password").style("text-sm font-medium leading-none"),
                                    )
                                    .child(
                                        text("Forgot your password?")
                                            .style("text-xs text-muted-foreground cursor-pointer"),
                                    ),
                            )
                            .child(
                                Input()
                                    .value(password.read().to_string())
                                    .mode(InputMode::password())
                                    .on_change(move |text| password.set(text)),
                            ),
                    ),
            ),
        )
        .child(
            CardFooter()
                .style("flex-col gap-2")
                .child(Button().style("w-full").child(text("Login")))
                .child(
                    Button()
                        .variant(ButtonVariant::Outline)
                        .style("w-full")
                        .child(text("Login with Google")),
                ),
        )
}

// ── Notifications ───────────────────────────────────────────────────

fn notifications_card() -> impl IntoElement {
    Card()
        .style("w-[380px]")
        .child(
            CardHeader()
                .style("flex-row justify-between items-start")
                .child(
                    div()
                        .style("flex-col gap-1.5")
                        .child(CardTitle().child(text("Notifications")))
                        .child(CardDescription().child(text("You have 3 unread messages."))),
                )
                .child(CardAction().child(text("🔔").style("text-lg"))),
        )
        .child(
            CardContent().child(
                div()
                    .style("flex-col gap-4")
                    .child(notification_item(
                        "OM",
                        "Your call has been confirmed.",
                        "1 hour ago",
                    ))
                    .child(notification_item(
                        "OM",
                        "You have a new message!",
                        "1 hour ago",
                    ))
                    .child(notification_item(
                        "OM",
                        "Your subscription is expiring soon!",
                        "2 hours ago",
                    )),
            ),
        )
        .child(
            CardFooter().child(
                Button()
                    .variant(ButtonVariant::Outline)
                    .style("w-full")
                    .child(text("Mark all as read")),
            ),
        )
}

fn notification_item(
    initials: &'static str,
    message: &'static str,
    time: &'static str,
) -> impl IntoElement {
    div()
        .style("flex-row items-start gap-3")
        .child(
            div()
                .style("size-9 rounded-full bg-secondary items-center justify-center")
                .child(text(initials).style("text-xs font-medium")),
        )
        .child(
            div()
                .style("flex-col gap-1")
                .child(text(message).style("text-sm font-medium leading-snug"))
                .child(text(time).style("text-xs text-muted-foreground")),
        )
}

// ── Team Members ────────────────────────────────────────────────────

fn team_members_card() -> impl IntoElement {
    Card()
        .style("w-[380px]")
        .child(
            CardHeader()
                .child(CardTitle().child(text("Team Members")))
                .child(CardDescription().child(text("Invite your team to collaborate."))),
        )
        .child(
            CardContent().child(
                div()
                    .style("flex-col gap-4")
                    .child(team_member("SC", "Sarah Chen", "sarah@example.com"))
                    .child(team_member("MR", "Marcus Rodriguez", "marcus@example.com"))
                    .child(team_member("EW", "Emily Watson", "emily@example.com")),
            ),
        )
        .child(
            CardFooter().child(
                Button()
                    .variant(ButtonVariant::Outline)
                    .style("w-full")
                    .child(text("Invite Member")),
            ),
        )
}

fn team_member(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
) -> impl IntoElement {
    div()
        .style("flex-row items-center gap-3")
        .child(
            div()
                .style("size-9 rounded-full bg-secondary items-center justify-center")
                .child(text(initials).style("text-xs font-medium")),
        )
        .child(
            div()
                .style("flex-col gap-0.5")
                .child(text(name).style("text-sm font-medium leading-none"))
                .child(text(email).style("text-xs text-muted-foreground")),
        )
}

// ── Subscription ────────────────────────────────────────────────────

fn subscription_card() -> impl IntoElement {
    Card()
        .style("w-[380px]")
        .child(
            CardHeader()
                .child(CardTitle().child(text("Pro Plan")))
                .child(CardDescription().child(text("Everything you need to scale."))),
        )
        .child(
            CardContent().child(
                div()
                    .style("flex-col gap-4")
                    .child(
                        div()
                            .style("flex-row items-end gap-1")
                            .child(text("$29").style("text-4xl font-bold leading-none"))
                            .child(text("/month").style("text-sm text-muted-foreground pb-1")),
                    )
                    .child(
                        div()
                            .style("flex-col gap-2 text-sm")
                            .child(feature("Unlimited projects"))
                            .child(feature("Advanced analytics"))
                            .child(feature("Priority support"))
                            .child(feature("Custom domains")),
                    ),
            ),
        )
        .child(CardFooter().child(Button().style("w-full").child(text("Upgrade to Pro"))))
}

fn feature(label: &'static str) -> impl IntoElement {
    div()
        .style("flex-row items-center gap-2")
        .child(text("✓").style("text-primary font-bold"))
        .child(text(label))
}
