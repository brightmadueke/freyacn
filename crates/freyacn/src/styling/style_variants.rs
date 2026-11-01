use crate::styling::style::Style;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StyleVariant {
    Hover,
    Focus,
    FocusVisible,
    FocusWithin,
    Selected,
    Checked,
    Active,
    Disabled,
}

impl StyleVariant {
    /// Lower numbers are applied first; higher numbers override.
    ///
    /// This is the *canonical* precedence used by [`StyleVariants::apply_all`],
    /// regardless of the order the caller listed them in.
    pub fn priority(self) -> u8 {
        match self {
            StyleVariant::Hover => 10,
            StyleVariant::Focus => 20,
            StyleVariant::FocusVisible => 30,
            StyleVariant::FocusWithin => 40,
            StyleVariant::Selected => 50,
            StyleVariant::Checked => 60,
            StyleVariant::Active => 70,
            StyleVariant::Disabled => 100,
        }
    }
}

/// A variant callback: takes a `Style` and returns a patched `Style`.
///
/// `Send + Sync` so `Style` (and therefore signals holding it) stay
/// `Send + Sync` — required by Freya's async signal/effect machinery.
pub type VariantFn = Arc<dyn Fn(Style) -> Style + Send + Sync>;

#[derive(Clone, Default)]
pub struct StyleVariants {
    entries: Vec<(StyleVariant, VariantFn)>,
}

impl StyleVariants {
    pub fn push(&mut self, kind: StyleVariant, f: VariantFn) {
        self.entries.push((kind, f));
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Apply every registered callback for `kind`, in registration order.
    pub fn apply(&self, kind: StyleVariant, style: Style) -> Style {
        self.entries
            .iter()
            .filter(|(k, _)| *k == kind)
            .fold(style, |s, (_, f)| f(s))
    }

    /// Apply every kind in `kinds` in **canonical priority order**
    /// (see [`StyleVariant::priority`]), so a later, higher-priority
    /// variant reliably overrides an earlier one on shared fields.
    pub fn apply_all(&self, kinds: &[StyleVariant], style: Style) -> Style {
        let mut sorted: Vec<StyleVariant> = kinds.iter().copied().collect();
        sorted.sort_by_key(|k| k.priority());
        sorted.dedup();
        sorted.into_iter().fold(style, |s, k| self.apply(k, s))
    }
}

impl fmt::Debug for StyleVariants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.entries.iter().map(|(k, _)| k))
            .finish()
    }
}

/// Compares **by kind only**, not by closure identity.
///
/// This is deliberate: variant closures are freshly allocated every
/// render, so `ptr_eq`/`Arc::eq`-based comparison would make `Style`
/// never equal across renders → spurious re-renders and effects.
///
/// Caveat: two styles with the *same* kinds but *different* callbacks
/// compare equal. That is fine for the "rebuild from props every render"
/// model — see `StyledExt` docs — but if you ever *store* applied styles
/// in signals and rely on `PartialEq` to detect closure changes, this
/// will silently miss them. In that case, drop `PartialEq` from `Style`
/// entirely and rebuild from props instead of comparing styles.
impl PartialEq for StyleVariants {
    fn eq(&self, other: &Self) -> bool {
        self.entries.len() == other.entries.len()
            && self
                .entries
                .iter()
                .zip(other.entries.iter())
                .all(|((k1, _), (k2, _))| k1 == k2)
    }
}
