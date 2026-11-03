//! Component-level helpers that don't belong on `Style` or `Theme`.

use crate::theme::ComponentState;

/// The interaction booleans a component knows about, bundled so that
/// resolving them to a [`ComponentState`] happens in one place.
///
/// Every interactive primitive (`Button`, `ListItem`, `MenuItem`, …) has
/// some subset of these and asks the theme for the corresponding state.
/// Instead of each one re-implementing the priority rules (and getting the
/// relative order of `pressed` vs `hovered` vs `focused` wrong), they
/// bundle the flags and call [`resolve`](Self::resolve).
///
/// # Priority
///
/// Matches `styled::StyleVariants`' priority order:
///
/// ```text
/// Disabled > Active > Checked > Selected > FocusVisible > Focus > Hover > Base
/// ```
///
/// The choice between `Active` and `Pressed` is deliberate: `Pressed` is
/// the "button held down" state (a subset of `Active`), and since
/// `ComponentState::Pressed` has `Active` in its fallback chain
/// (`theme.rs::ComponentState::fallback_chain`), either works. We emit
/// `Active` because it's the more generic option — a theme that defines
/// `active` but not `pressed` gets the right style for free.
///
/// # Example
///
/// ```
/// use freya_cn::utils::InteractionState;
/// use freya_cn::theme::ComponentState;
///
/// // Hovered button.
/// assert_eq!(
///     InteractionState { hovered: true, ..Default::default() }.resolve(),
///     ComponentState::Hover,
/// );
///
/// // Hovered + pressed → Active wins (Active has higher priority).
/// assert_eq!(
///     InteractionState { hovered: true, pressed: true, ..Default::default() }.resolve(),
///     ComponentState::Active,
/// );
///
/// // Disabled overrides everything.
/// assert_eq!(
///     InteractionState {
///         hovered: true,
///         pressed: true,
///         focused: true,
///         disabled: true,
///         ..Default::default()
///     }.resolve(),
///     ComponentState::Disabled,
/// );
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    /// Only meaningful when `focused` is `true`. If both are set and the
    /// theme defines a `focus-visible` variant, it wins over `focus`.
    pub focus_visible: bool,
    pub disabled: bool,
    pub checked: bool,
    pub selected: bool,
}

impl InteractionState {
    /// Resolve the interaction flags to the single "strongest"
    /// [`ComponentState`] that the theme should be queried for.
    ///
    /// See the type-level docs for the priority order and worked examples.
    pub fn resolve(self) -> ComponentState {
        if self.disabled {
            return ComponentState::Disabled;
        }
        if self.pressed {
            return ComponentState::Active;
        }
        if self.checked {
            return ComponentState::Checked;
        }
        if self.selected {
            return ComponentState::Selected;
        }
        if self.focused {
            return if self.focus_visible {
                ComponentState::FocusVisible
            } else {
                ComponentState::Focus
            };
        }
        if self.hovered {
            return ComponentState::Hover;
        }
        ComponentState::Base
    }
}

/// Convenience wrapper around [`InteractionState::resolve`] for callers
/// that would rather pass flags positionally than build a struct.
///
/// The argument order matches the field order on [`InteractionState`] —
/// but for anything beyond three flags, prefer the struct, since a
/// positional call site like `pick(true, false, true, false, false, false,
/// false)` is impossible to read.
#[inline]
pub fn pick_component_state(state: InteractionState) -> ComponentState {
    state.resolve()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_base() {
        assert_eq!(InteractionState::default().resolve(), ComponentState::Base);
    }

    #[test]
    fn priority_chain() {
        let cases = [
            (
                InteractionState {
                    hovered: true,
                    ..Default::default()
                },
                ComponentState::Hover,
            ),
            (
                InteractionState {
                    pressed: true,
                    hovered: true,
                    ..Default::default()
                },
                ComponentState::Active,
            ),
            (
                InteractionState {
                    checked: true,
                    hovered: true,
                    ..Default::default()
                },
                ComponentState::Checked,
            ),
            (
                InteractionState {
                    selected: true,
                    ..Default::default()
                },
                ComponentState::Selected,
            ),
            (
                InteractionState {
                    focused: true,
                    ..Default::default()
                },
                ComponentState::Focus,
            ),
            (
                InteractionState {
                    focused: true,
                    focus_visible: true,
                    ..Default::default()
                },
                ComponentState::FocusVisible,
            ),
            (
                InteractionState {
                    disabled: true,
                    hovered: true,
                    pressed: true,
                    focused: true,
                    ..Default::default()
                },
                ComponentState::Disabled,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(input.resolve(), expected, "input: {input:?}");
        }
    }
}
