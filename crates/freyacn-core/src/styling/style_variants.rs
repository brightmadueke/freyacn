use crate::styling::style::Style;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StyleVariant {
    /// `dark:` — mode-conditional patch. Lowest priority so it acts as a
    /// base override that any interaction state can win against.
    Dark,
    Hover,
    Focus,
    FocusVisible,
    FocusWithin,
    Selected,
    /// `aria-selected:` — a11y-selected state for elements that expose
    /// `aria-selected` instead of a native state (tabs, listboxes).
    AriaSelected,
    Checked,
    /// `aria-checked:` — a11y-checked state (checkbox, radio, switch).
    AriaChecked,
    Active,
    /// `aria-expanded:` — open/closed state (accordion, dropdown, combobox).
    AriaExpanded,
    /// `aria-pressed:` — toggle state (toggle button, toolbar button).
    AriaPressed,
    /// `aria-invalid:` — validation state. Fires when the control is
    /// marked invalid; typically styles the error border/ring.
    AriaInvalid,
    /// `aria-required:` — required-field state.
    AriaRequired,
    /// `aria-readonly:` — read-only state.
    AriaReadonly,
    /// `aria-hidden:` — hidden state.
    AriaHidden,
    /// `aria-disabled:` — disabled state exposed via aria instead of the
    /// native `disabled` attribute (non-form controls).
    AriaDisabled,
    Disabled,
}

impl StyleVariant {
    pub fn priority(self) -> u8 {
        match self {
            StyleVariant::Dark => 5,
            StyleVariant::Hover => 10,
            StyleVariant::Focus => 20,
            StyleVariant::FocusVisible => 30,
            StyleVariant::FocusWithin => 40,
            StyleVariant::Selected => 50,
            StyleVariant::AriaSelected => 55,
            StyleVariant::Checked => 60,
            StyleVariant::AriaChecked => 65,
            StyleVariant::Active => 70,
            StyleVariant::AriaExpanded => 72,
            StyleVariant::AriaPressed => 74,
            StyleVariant::AriaInvalid => 80,
            StyleVariant::AriaRequired => 82,
            StyleVariant::AriaReadonly => 84,
            StyleVariant::AriaHidden => 86,
            StyleVariant::AriaDisabled => 99,
            StyleVariant::Disabled => 100,
        }
    }

    /// `true` for `aria-*` variants. Useful for diagnostics / debugging.
    pub fn is_aria(self) -> bool {
        matches!(
            self,
            StyleVariant::AriaSelected
                | StyleVariant::AriaChecked
                | StyleVariant::AriaExpanded
                | StyleVariant::AriaPressed
                | StyleVariant::AriaInvalid
                | StyleVariant::AriaRequired
                | StyleVariant::AriaReadonly
                | StyleVariant::AriaHidden
                | StyleVariant::AriaDisabled
        )
    }
}

pub type VariantFn = Arc<dyn Fn(Style) -> Style + Send + Sync>;

#[derive(Clone, Default)]
pub struct StyleVariants {
    entries: Vec<(Vec<StyleVariant>, VariantFn)>,
}

impl StyleVariants {
    pub fn push(&mut self, kind: StyleVariant, f: VariantFn) {
        self.entries.push((vec![kind], f));
    }

    pub fn push_compound(&mut self, kinds: Vec<StyleVariant>, f: VariantFn) {
        assert!(
            !kinds.is_empty(),
            "StyleVariants::push_compound requires at least one condition"
        );
        self.entries.push((kinds, f));
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn apply(&self, kind: StyleVariant, style: Style) -> Style {
        self.apply_all(&[kind], style)
    }

    pub fn apply_all(&self, active: &[StyleVariant], style: Style) -> Style {
        if active.is_empty() || self.entries.is_empty() {
            return style;
        }

        let active_set: HashSet<StyleVariant> = active.iter().copied().collect();

        let mut applicable: Vec<&(Vec<StyleVariant>, VariantFn)> = self
            .entries
            .iter()
            .filter(|(conds, _)| conds.iter().all(|c| active_set.contains(c)))
            .collect();

        applicable.sort_by_key(|(conds, _)| {
            let max_priority = conds.iter().map(|c| c.priority()).max().unwrap_or(0);
            (max_priority, conds.len())
        });

        applicable.into_iter().fold(style, |s, (_, f)| f(s))
    }

    pub fn has(&self, kind: StyleVariant) -> bool {
        self.entries.iter().any(|(conds, _)| conds.contains(&kind))
    }

    pub fn has_any_aria(&self) -> bool {
        self.entries
            .iter()
            .any(|(conds, _)| conds.iter().any(|c| c.is_aria()))
    }
}

impl fmt::Debug for StyleVariants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.entries.iter().map(|(conds, _)| conds))
            .finish()
    }
}

impl PartialEq for StyleVariants {
    fn eq(&self, other: &Self) -> bool {
        self.entries.len() == other.entries.len()
            && self
                .entries
                .iter()
                .zip(other.entries.iter())
                .all(|((c1, _), (c2, _))| c1 == c2)
    }
}
