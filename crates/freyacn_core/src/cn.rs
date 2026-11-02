//! `cn!` — Tailwind-style class name concatenation.
//!
//! Joins class-name fragments into a single space-separated string, skipping
//! empty fragments and `None` options. Modeled after the `cn(...)` helper
//! from shadcn/ui.
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
/// * append their textual representation to `out`,
/// * insert a single space before their text if `out` is non-empty,
/// * do nothing if their textual representation is empty.
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
    fn write_to(&self, out: &mut String);
}

impl CnArg for str {
    fn write_to(&self, out: &mut String) {
        if self.is_empty() {
            return;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(self);
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
// The cn! macro
// ===========================================================================

/// Concatenate class-name fragments into a single space-separated string.
///
/// Modeled after the `cn(...)` helper from shadcn/ui. Each argument is
/// appended in order; empty fragments and `None` options are skipped; every
/// non-empty fragment is separated by a single space.
///
/// # Accepted inputs
///
/// Anything that implements [`CnArg`]. In practice:
///
/// * `&str`, `String`, `Cow<'_, str>`,
/// * references to any of the above,
/// * `Option<T>` where `T` implements `CnArg`,
/// * `Vec<T>`, `[T]`, `[T; N]` where `T` implements `CnArg`,
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