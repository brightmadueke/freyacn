//! `cn!` — Tailwind-style class name concatenation.
//!
//! Joins class-name fragments into a single space-separated string, skipping
//! empty fragments and `None` options. Modeled after the `cn(...)` helper
//! from shadcn/ui.
//!
//! # Relationship to shadcn's `cn`
//!
//! shadcn's `cn()` does two things: `clsx(inputs)` (concatenate, skip falsy)
//! then `twMerge(result)` (resolve conflicting utilities so `p-4 p-2` →
//! `p-2`).
//!
//! `cn!` only does the first step. It returns `"p-4 p-2"` as-is.
//!
//! For **FreyaCN specifically**, this is intentional: conflict resolution
//! happens at styling time. [`crate::styling::styled_string::StyledString::parse`]
//! applies utilities in order, so later ones override earlier ones for the
//! same property. `Style::default().style(&cn!("p-4", "p-2"))` produces
//! `padding = 8px`, identical to shadcn's output.
//!
//! If you're using `cn!` for something **other** than feeding `StyledString`
//! (e.g. an HTML `class` attribute, a log message, a database field),
//! remember that the result is unmerged — conflicting utilities will both
//! appear.
//!
//! # Whitespace
//!
//! Every `str`-family fragment is **trimmed** before being appended, so:
//!
//! * leading/trailing whitespace in a fragment is discarded,
//! * interior whitespace is preserved,
//! * whitespace-only fragments are treated as empty and skipped.
//!
//! This means `cn!("a", "  ")` returns `"a"` (not `"a   "`), and
//! `cn!("  px-4  ", "  py-2  ")` returns `"px-4 py-2"`.
//!
//! # Example
//!
//! ```
//! use freyacn::cn;
//!
//! let active = true;
//! let classes = cn!(
//!     "px-4 py-2 rounded",
//!     "bg-blue-500 text-white",
//!     if active { "ring-2 ring-blue-300" } else { "" },
//!     Some("shadow-md"),
//!     None::<&str>,
//! );
//! assert_eq!(
//!     classes,
//!     "px-4 py-2 rounded bg-blue-500 text-white ring-2 ring-blue-300 shadow-md",
//! );
//! ```

use std::borrow::Cow;

// ===========================================================================
// CnArg trait
// ===========================================================================

/// A fragment that can be appended to a class-name string.
///
/// Implemented for every type the `cn!` macro accepts. Users can implement
/// it for their own types to extend `cn!` with custom fragment kinds.
///
/// # Contract
///
/// Implementors must:
///
/// * skip silently if their textual representation is empty,
/// * otherwise insert a single space before their text if `out` is
///   non-empty, then append their text to `out`.
///
/// Implementors **should** trim leading/trailing whitespace from their
/// textual representation — that's what the [`str`] impl does, and it's
/// what makes `cn!("a", "  ")` return `"a"` rather than `"a   "`. Custom
/// impls that push raw user input are expected to trim too.
///
/// This contract is what allows `cn!` to normalise spacing regardless of
/// how many fragments are empty.
///
/// # Example
///
/// ```
/// use freyacn::cn::CnArg;
///
/// struct Bullet;
///
/// impl CnArg for Bullet {
///     fn write_to(&self, out: &mut String) {
///         if !out.is_empty() {
///             out.push(' ');
///         }
///         out.push_str("•");
///     }
/// }
/// ```
pub trait CnArg {
    /// Append this fragment to `out`.
    ///
    /// See the [trait-level docs](CnArg) for the contract implementors must
    /// follow.
    fn write_to(&self, out: &mut String);
}

impl CnArg for str {
    fn write_to(&self, out: &mut String) {
        // Trim leading/trailing whitespace so `"  "` doesn't emit stray
        // spaces. Interior whitespace is preserved: `"px-4 py-2"` stays
        // as two classes.
        let trimmed = self.trim();
        if trimmed.is_empty() {
            return;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(trimmed);
    }
}

impl CnArg for String {
    fn write_to(&self, out: &mut String) {
        self.as_str().write_to(out);
    }
}

impl CnArg for Cow<'_, str> {
    fn write_to(&self, out: &mut String) {
        self.as_ref().write_to(out);
    }
}

impl<T: CnArg + ?Sized> CnArg for &T {
    fn write_to(&self, out: &mut String) {
        (**self).write_to(out);
    }
}

impl<T: CnArg + ?Sized> CnArg for &mut T {
    fn write_to(&self, out: &mut String) {
        (**self).write_to(out);
    }
}

impl<T: CnArg + ?Sized> CnArg for Box<T> {
    fn write_to(&self, out: &mut String) {
        (**self).write_to(out);
    }
}

impl<T: CnArg + ?Sized> CnArg for std::rc::Rc<T> {
    fn write_to(&self, out: &mut String) {
        (**self).write_to(out);
    }
}

impl<T: CnArg + ?Sized> CnArg for std::sync::Arc<T> {
    fn write_to(&self, out: &mut String) {
        (**self).write_to(out);
    }
}

impl<T: CnArg> CnArg for Option<T> {
    fn write_to(&self, out: &mut String) {
        if let Some(v) = self {
            v.write_to(out);
        }
    }
}

impl<T: CnArg> CnArg for Vec<T> {
    fn write_to(&self, out: &mut String) {
        for item in self {
            item.write_to(out);
        }
    }
}

impl<T: CnArg> CnArg for [T] {
    fn write_to(&self, out: &mut String) {
        for item in self {
            item.write_to(out);
        }
    }
}

impl<T: CnArg, const N: usize> CnArg for [T; N] {
    fn write_to(&self, out: &mut String) {
        for item in self {
            item.write_to(out);
        }
    }
}

// ===========================================================================
// The cn() function — for iterators
// ===========================================================================

/// Concatenate any iterator of class-name fragments into a single
/// space-separated string.
///
/// The function form of [`cn!`], useful when the fragments come from a
/// lazy iterator that can't be passed directly to the macro (e.g. a
/// `filter` + `map` chain). For a fixed list of fragments, prefer
/// [`cn!`] — it's terser and doesn't require the items to be
/// homogeneous.
///
/// # Example
///
/// ```
/// # use freyacn::cn::cn;
/// let items = vec![
///     Some("px-4"),
///     None,
///     Some("py-2"),
///     Some(""),
/// ];
/// let classes: String = cn(items.iter().filter_map(|x| *x));
/// assert_eq!(classes, "px-4 py-2");
/// ```
///
/// ```
/// # use freyacn::cn::cn;
/// let sizes = ["sm", "lg", "xl"];
/// let classes = cn(sizes.iter().map(|s| format!("text-{s}")));
/// assert_eq!(classes, "text-sm text-lg text-xl");
/// ```
///
/// Empty iterators produce an empty string:
///
/// ```
/// # use freyacn::cn::cn;
/// let empty: Vec<&str> = vec![];
/// assert_eq!(cn(empty), "");
/// ```
#[inline]
pub fn cn<I>(items: I) -> String
where
    I: IntoIterator,
    I::Item: CnArg,
{
    let mut out = String::new();
    for item in items {
        item.write_to(&mut out);
    }
    out
}

// ===========================================================================
// The cn! macro
// ===========================================================================

/// Concatenate class-name fragments into a single space-separated string.
///
/// Modeled after the `cn(...)` helper from shadcn/ui. Each argument is
/// appended in order; empty fragments and `None` options are skipped; every
/// non-empty fragment is separated by a single space.
///
/// **Note:** unlike shadcn's `cn`, this does **not** resolve conflicting
/// utilities. `cn!("p-4", "p-2")` returns `"p-4 p-2"`. When consumed by
/// [`crate::styling::styled::StyledExt::style`] (which routes through
/// `StyledString::parse`), later utilities override earlier ones — the
/// same result as shadcn's `twMerge` step. See the [module docs](self)
/// for details.
///
/// # Accepted inputs
///
/// Anything that implements [`CnArg`]. In practice:
///
/// * `&str`, `String`, `Cow<'_, str>`,
/// * references to any of the above,
/// * `Option<T>` where `T` implements `CnArg`,
/// * `Vec<T>`, `[T]`, `[T; N]` where `T` implements `CnArg`,
/// * `Box<T>`, `Rc<T>`, `Arc<T>` where `T: CnArg + ?Sized`
///   (useful for `Box<dyn CnArg>`),
/// * a previous `cn!(...)` result.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use freyacn::cn;
/// let classes = cn!("px-4", "py-2", "rounded");
/// assert_eq!(classes, "px-4 py-2 rounded");
/// ```
///
/// Conditional classes:
///
/// ```
/// # use freyacn::cn;
/// let active = true;
/// let classes = cn!("btn", if active { "btn-active" } else { "" });
/// assert_eq!(classes, "btn btn-active");
/// ```
///
/// Optional classes:
///
/// ```
/// # use freyacn::cn;
/// let disabled = false;
/// let classes = cn!(
///     "input",
///     Some("input-lg"),
///     if disabled { Some("opacity-50") } else { None },
/// );
/// assert_eq!(classes, "input input-lg");
/// ```
///
/// Empty input:
///
/// ```
/// # use freyacn::cn;
/// assert_eq!(cn!(), "");
/// assert_eq!(cn!(""), "");
/// assert_eq!(cn!("", "", ""), "");
/// ```
///
/// Whitespace-only fragments are skipped:
///
/// ```
/// # use freyacn::cn;
/// assert_eq!(cn!("a", "  "), "a");
/// assert_eq!(cn!("  a  ", "  b  "), "a b");
/// ```
///
/// Nesting:
///
/// ```
/// # use freyacn::cn;
/// let inner = cn!("text-sm", "font-bold");
/// let classes = cn!("card", inner, "p-4");
/// assert_eq!(classes, "card text-sm font-bold p-4");
/// ```
///
/// Trailing comma:
///
/// ```
/// # use freyacn::cn;
/// let classes = cn!(
///     "a",
///     "b",
/// );
/// assert_eq!(classes, "a b");
/// ```
///
/// Vectors and slices:
///
/// ```
/// # use freyacn::cn;
/// let v = vec!["a", "b"];
/// assert_eq!(cn!(v), "a b");
/// assert_eq!(cn!(["c", "d"]), "c d");
/// ```
///
/// `Box<dyn CnArg>` for heterogeneous fragments:
///
/// ```
/// # use freyacn::cn;
/// # use freyacn::cn::CnArg;
/// let a: Box<dyn CnArg> = Box::new("a");
/// let b: Box<dyn CnArg> = Box::new(String::from("b"));
/// assert_eq!(cn!(a, b), "a b");
/// ```
#[macro_export]
macro_rules! cn {
    () => {
        ::std::string::String::new()
    };
    ($($part:expr),+ $(,)?) => {{
        let mut __cn_out = ::std::string::String::new();
        $(
            $crate::cn::CnArg::write_to(&$part, &mut __cn_out);
        )+
        __cn_out
    }};
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs() {
        assert_eq!(cn!(), "");
        assert_eq!(cn!(""), "");
        assert_eq!(cn!("", "", ""), "");
        assert_eq!(cn!("   "), "");
        assert_eq!(cn!("\t\n"), "");
    }

    #[test]
    fn basic_concat() {
        assert_eq!(cn!("a", "b", "c"), "a b c");
        assert_eq!(cn!("a"), "a");
        assert_eq!(cn!("a",), "a");
    }

    #[test]
    fn skips_none() {
        assert_eq!(cn!("a", None::<&str>, "b"), "a b");
        assert_eq!(cn!(None::<&str>, None::<&str>), "");
    }

    #[test]
    fn trims_fragments() {
        // Was `"a   "` before the trim fix.
        assert_eq!(cn!("a", "  "), "a");
        // Was `"a   b  "` before the trim fix.
        assert_eq!(cn!("a", "  b  "), "a b");
        // Was `"  a     b  "` before the trim fix.
        assert_eq!(cn!("  a  ", "  b  "), "a b");
        // Whitespace-only between non-empty fragments is skipped.
        assert_eq!(cn!("a", "   ", "b"), "a b");
    }

    #[test]
    fn preserves_interior_whitespace() {
        // Internal spaces separate classes and are preserved verbatim.
        assert_eq!(cn!("px-4 py-2"), "px-4 py-2");
        assert_eq!(cn!("px-4 py-2", "rounded"), "px-4 py-2 rounded");
        // Interior runs are preserved (not collapsed) — `split_whitespace`
        // in `StyledString::parse` normalises them downstream.
        assert_eq!(cn!("a  b"), "a  b");
    }

    #[test]
    fn string_types() {
        assert_eq!(cn!(String::from("a"), String::from("b")), "a b");
        assert_eq!(
            cn!(Cow::Borrowed("a"), Cow::Owned::<str>("b".into())),
            "a b"
        );
    }

    #[test]
    fn references() {
        let a = "a";
        let b = String::from("b");
        assert_eq!(cn!(&a, &b), "a b");
        // `&&str` works too.
        assert_eq!(cn!(&&a), "a");
    }

    #[test]
    fn options() {
        assert_eq!(cn!(Some("a"), Some("b")), "a b");
        assert_eq!(cn!(Some(String::from("a"))), "a");
        // Nested options.
        assert_eq!(cn!(Some(Some("a")), Some(None::<&str>)), "a");
    }

    #[test]
    fn collections() {
        assert_eq!(cn!(vec!["a", "b"]), "a b");
        assert_eq!(cn!(["a", "b"]), "a b");
        let slice: &[&str] = &["a", "b"];
        assert_eq!(cn!(slice), "a b");
        // Nested collections.
        assert_eq!(cn!(vec![vec!["a"], vec!["b"]]), "a b");
    }

    #[test]
    fn boxed_trait_objects() {
        let a: Box<dyn CnArg> = Box::new("a");
        let b: Box<dyn CnArg> = Box::new(String::from("b"));
        assert_eq!(cn!(a, b), "a b");
    }

    #[test]
    fn nested_cn() {
        let inner = cn!("a", "b");
        assert_eq!(cn!("pre", inner, "post"), "pre a b post");
    }

    #[test]
    fn cn_function_basic() {
        let items = vec!["a", "b", "c"];
        assert_eq!(cn(items), "a b c");
    }

    #[test]
    fn cn_function_with_iter() {
        let items = vec!["a", "b", "c"];
        assert_eq!(cn(items.iter()), "a b c");
    }

    #[test]
    fn cn_function_with_filter_map() {
        let items = vec![Some("a"), None, Some("b"), Some("")];
        let classes: String = cn(items.iter().filter_map(|x| *x));
        assert_eq!(classes, "a b");
    }

    #[test]
    fn cn_function_empty() {
        let empty: Vec<&str> = vec![];
        assert_eq!(cn(empty), "");
    }

    #[test]
    fn cn_function_with_options() {
        let items: Vec<Option<&str>> = vec![Some("a"), None, Some("b")];
        assert_eq!(cn(items), "a b");
    }
}
