//! `has:` variant tracking — child-to-parent signaling.
//!
//! Tailwind's `has-[...]` fires when an element contains a descendant
//! matching the selector. Freya has no DOM tree or selector engine, so
//! arbitrary selectors (`has-[.foo]`, `has-[data-x]`) can't work. The
//! semantic-child cases — `has-[svg]`, `has-[img]`, `has-[:checked]`,
//! `has-[:selected]`, `has-[:disabled]` — do work, via a flag bag
//! carried in context.
//!
//! # How it works
//!
//! 1. A parent whose style declares any `has:` variant creates a
//!    reactive [`HasFlags`] bag and provides it via [`HasContext`].
//! 2. Children that match a signal — [`Icon`](crate::elements::Icon),
//!    [`Image`](crate::elements::Image), a checked
//!    [`Checkbox`](crate::elements::Checkbox) / [`Radio`](crate::elements::Radio) —
//!    set or clear the matching flag on the parent's bag via
//!    [`use_has_signal`] / [`use_has_signal_when`].
//! 3. The parent re-renders, reads its flags, and pushes
//!    `StyleVariant::Has*` entries into the live-variant set.
//! 4. Matching `has-*:` classes fire.
//!
//! Flags are tracked **per-owner** (each child that signals gets a
//! unique id), so two children with the same kind don't conflict. The
//! parent's active set is the union of every child's active signals.
//!
//! # Limitations
//!
//! - **One-frame delay on mount.** The child signals *after* its first
//!   render, so the parent's `has:` styles paint on the *next* frame.
//!   This is fundamental to the reactive-context pattern: the parent
//!   has finished its render by the time the child's effect fires.
//! - **No unmount cleanup.** Freya has no unmount hook, so a
//!   long-lived child keeps its flag set. For conditional icons /
//!   images, use `.with_has_svg(cond)` explicitly — that path goes
//!   through `SemanticEvents::has_state` and updates synchronously
//!   with no delay and no persistence issue.
//! - **Immediate children only, unless intermediates opt in.** A flag
//!   set by a grandchild propagates to an ancestor only if the
//!   intermediate element also has `has:` classes of its own — which
//!   makes it provide its own [`HasContext`] and re-signal upward.
//!   For arbitrary-depth tracking, use explicit `.with_has_svg(cond)`.

use crate::styling::style_variants::StyleVariant;
use freya::prelude::*;
use std::cell::Cell;
use std::collections::{BTreeSet, HashSet};

/// The kind of "has" a child signals to its ancestors.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum HasKind {
    /// Contains an `<svg>` (an [`Icon`](crate::elements::Icon)).
    Svg,
    /// Contains an [`Image`](crate::elements::Image).
    Img,
    /// Contains a checked checkbox or radio.
    Checked,
    /// Contains a selected element.
    Selected,
    /// Contains a disabled element.
    Disabled,
}

impl HasKind {
    /// Map to the corresponding [`StyleVariant`].
    pub fn to_variant(self) -> StyleVariant {
        match self {
            HasKind::Svg => StyleVariant::HasSvg,
            HasKind::Img => StyleVariant::HasImg,
            HasKind::Checked => StyleVariant::HasChecked,
            HasKind::Selected => StyleVariant::HasSelected,
            HasKind::Disabled => StyleVariant::HasDisabled,
        }
    }

    /// Parse a selector string from `has-[<selector>]:` or `has-<name>:`.
    ///
    /// Accepts:
    ///
    /// - `[svg]` / `[img]` — bracketed bare tag names.
    /// - `[:checked]` / `[:selected]` / `[:disabled]` — pseudo-class forms.
    /// - `svg` / `img` — unbracketed, fluent-friendly.
    pub fn from_selector(selector: &str) -> Option<Self> {
        let trimmed = selector.trim();
        let inner = trimmed
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(trimmed)
            .trim();
        let inner = inner.strip_prefix(':').unwrap_or(inner);
        Some(match inner {
            "svg" => HasKind::Svg,
            "img" => HasKind::Img,
            "checked" => HasKind::Checked,
            "selected" => HasKind::Selected,
            "disabled" => HasKind::Disabled,
            _ => return None,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────
// Owner ids — one per `use_has_signal` caller
// ─────────────────────────────────────────────────────────────────────

thread_local! {
    static NEXT_HAS_OWNER_ID: Cell<u64> = const { Cell::new(0) };
}

fn next_owner_id() -> u64 {
    NEXT_HAS_OWNER_ID.with(|c| {
        let id = c.get();
        c.set(id.wrapping_add(1));
        id
    })
}

// ─────────────────────────────────────────────────────────────────────
// HasFlags — reactive bag shared between parent and descendants
// ─────────────────────────────────────────────────────────────────────

/// A reactive bag of per-owner [`HasKind`] flags, shared between a
/// parent element and its descendants.
///
/// Each child signals with a stable **owner id** (obtained once via
/// `use_hook(next_owner_id)`). The bag stores `(kind, owner)` pairs, so
/// two children can independently signal the same kind without
/// conflicting: the effective active set is any `kind` with at least
/// one live owner.
///
/// Cheap to clone — clones share the same underlying `State`.
#[derive(Clone)]
pub struct HasFlags {
    state: State<HashSet<(HasKind, u64)>>,
}

impl HasFlags {
    /// Create a new flag bag tied to the current component's reactive
    /// scope. **Must be called inside `render`** — uses `use_state`.
    pub fn new() -> Self {
        Self {
            state: use_state(HashSet::<(HasKind, u64)>::new),
        }
    }

    /// Add `(kind, owner)`. No-op if already present (`set_if_modified`
    /// short-circuits on an unchanged value, so re-firing the effect on
    /// every render can't loop).
    pub fn set(&mut self, kind: HasKind, owner: u64) {
        let mut next = (*self.state.peek()).clone();
        next.insert((kind, owner));
        self.state.set_if_modified(next);
    }

    /// Remove `(kind, owner)`. No-op if absent.
    pub fn clear(&mut self, kind: HasKind, owner: u64) {
        let mut next = (*self.state.peek()).clone();
        if next.remove(&(kind, owner)) {
            self.state.set_if_modified(next);
        }
    }

    /// `true` if any owner is currently signaling `kind`.
    pub fn contains(&self, kind: HasKind) -> bool {
        self.state.read().iter().any(|(k, _)| *k == kind)
    }

    /// Every active `HasKind` (as a [`StyleVariant`]) — deduped across
    /// owners, in stable order.
    ///
    /// **Subscribes** to state changes, so the owning component
    /// re-renders when a child signals.
    pub fn active_variants(&self) -> Vec<StyleVariant> {
        let kinds: BTreeSet<HasKind> = self.state.read().iter().map(|(k, _)| *k).collect();
        kinds.into_iter().map(|k| k.to_variant()).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────
// HasContext — the context key
// ─────────────────────────────────────────────────────────────────────

/// Context wrapper that lets children signal [`HasKind`] flags to the
/// nearest ancestor whose style declares a `has:` variant.
#[derive(Clone)]
pub struct HasContext(pub HasFlags);

impl HasContext {
    /// `provide_context(self)` — small ergonomic shim so callers don't
    /// have to import `provide_context` alongside `HasContext`.
    pub fn provide(self) {
        provide_context(self);
    }
}

impl PartialEq for HasContext {
    fn eq(&self, _: &Self) -> bool {
        // Same bag → equal. Flag updates propagate via `State` subscribers,
        // not via context identity. Treating all contexts as equal avoids
        // triggering re-renders merely because a child re-rendered.
        true
    }
}

/// Read the nearest ancestor [`HasContext`], or `None` if no ancestor
/// declared a `has:` variant.
pub fn try_use_has_context() -> Option<HasContext> {
    try_consume_context::<HasContext>()
}

// ─────────────────────────────────────────────────────────────────────
// Signalers — the API children call
// ─────────────────────────────────────────────────────────────────────

/// Mark this element as matching `kind` for every ancestor that provides
/// a [`HasContext`].
///
/// Runs as a side effect. Registers **once per component** — the effect
/// re-runs on parent re-renders only if `kind` is somehow reactive
/// (it isn't, so this is effectively "fire once and stay set").
///
/// For conditional signals, use [`use_has_signal_when`].
pub fn use_has_signal(kind: HasKind) {
    use_has_signal_when(kind, true);
}

/// Like [`use_has_signal`] but conditional on `active`.
///
/// **Correctly re-runs when `active` flips.** `active` is wrapped in
/// `use_reactive` so the effect subscribes to it: `false → true` calls
/// `HasFlags::set`, `true → false` calls `HasFlags::clear`. Both paths
/// use this caller's stable owner id, so multiple children signalling
/// the same kind are tracked independently.
///
/// # Implementation note
///
/// `use_side_effect` registers the effect **once** via `use_hook`
/// (see the Freya `effect.rs` source). The effect's closure captures
/// handles, not values — `ctx: Option<HasContext>` (a handle to the
/// parent's `State`), `active: State<bool>` (from `use_reactive`),
/// `owner: u64` (a `Copy` value from `use_hook`). Reading `active`
/// inside the effect subscribes to it; that's what makes the effect
/// re-fire on toggle. Capturing the bare `active: bool` value would
/// freeze it at the first-render value and never re-fire.
pub fn use_has_signal_when(kind: HasKind, active: bool) {
    // Not hooks — plain context lookup, safe in any order.
    let ctx = try_use_has_context();

    // Reactive handle — notifies the effect when `active` changes.
    let active = use_reactive(&active);

    // Stable per-instance owner id. `use_hook` runs once per component.
    let owner = use_hook(next_owner_id);

    use_side_effect(move || {
        // Reads `active` reactively — subscribes the effect.
        let is_active = *active.read();
        if let Some(HasContext(mut flags)) = ctx.clone() {
            if is_active {
                flags.set(kind, owner);
            } else {
                flags.clear(kind, owner);
            }
        }
    });
}
