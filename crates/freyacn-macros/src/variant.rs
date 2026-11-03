//! The `#[variant]` attribute macro for enums.
//!
//! Generates `From<&str>`, `From<String>`, `.style_str()`, `.resolve()`,
//! and an `impl freyacn_core::ComponentVariant` from per-variant
//! `#[variant("…")]` attributes.
//!
//! The crate that the generated code refers to is configurable at the
//! enum level:
//!
//! ```ignore
//! // default — resolves against the `freyacn` facade
//! #[variant]
//! pub enum MyVariant { #[variant("…")] A }
//!
//! // explicit — use an internal crate instead of the facade
//! #[variant(freyacn_core)]
//! pub enum ButtonVariant { #[variant("…")] Default }
//! ```

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Error, Ident, ItemEnum, LitStr, Path};

pub fn parse_variant(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
    // ---- Resolve the crate path the expansion should reference. ----
    let krate: Path = if attr.is_empty() {
        syn::parse_quote!(::freyacn)
    } else {
        syn::parse2(attr)?
    };

    let mut item_enum: ItemEnum = syn::parse2(item)?;
    let enum_ident = item_enum.ident.clone();

    // ---- Extract per-variant style strings, stripping the attribute. ----
    let mut variant_styles: Vec<(Ident, String)> = Vec::new();
    for variant in &mut item_enum.variants {
        let mut style: Option<String> = None;
        let mut kept = Vec::new();
        for attr in std::mem::take(&mut variant.attrs) {
            if attr.path().is_ident("variant") {
                let lit: LitStr = attr.parse_args()?;
                if style.is_some() {
                    return Err(Error::new_spanned(
                        &attr,
                        "duplicate `#[variant(...)]` attribute on this variant",
                    ));
                }
                style = Some(lit.value());
            } else {
                kept.push(attr);
            }
        }
        variant.attrs = kept;

        let style = style.ok_or_else(|| {
            Error::new_spanned(
                &variant.ident,
                format!(
                    "variant `{}` is missing its `#[variant(\"...\")]` attribute",
                    variant.ident,
                ),
            )
        })?;
        variant_styles.push((variant.ident.clone(), style));
    }

    if variant_styles.is_empty() {
        return Err(Error::new_spanned(
            &enum_ident,
            "`#[variant]` requires at least one variant",
        ));
    }

    let variant_idents: Vec<&Ident> = variant_styles.iter().map(|(id, _)| id).collect();
    let style_lits: Vec<LitStr> = variant_styles
        .iter()
        .map(|(_, s)| LitStr::new(s, Span::call_site()))
        .collect();
    let kebab_lits: Vec<LitStr> = variant_styles
        .iter()
        .map(|(id, _)| LitStr::new(&to_kebab_case(&id.to_string()), Span::call_site()))
        .collect();

    let fallback = variant_idents[0];

    Ok(quote! {
        #item_enum

        impl #enum_ident {
            /// The raw style string attached to the current variant.
            pub fn style_str(&self) -> &'static str {
                match self {
                    #( Self::#variant_idents => #style_lits, )*
                }
            }

            /// Convenience alias for `ComponentVariant::resolve`.
            ///
            /// Lets callers write `v.resolve()` without importing the
            /// `ComponentVariant` trait. Must be called inside a Freya render
            /// context — reads the active theme.
            pub fn resolve(&self) -> #krate::styling::Style {
                <Self as #krate::prelude::ComponentVariant>::resolve(self)
            }
        }

        impl #krate::prelude::ComponentVariant for #enum_ident {
            fn resolve(&self) -> #krate::styling::Style {
                // Start from `Style::bare()`, NOT `Style::default()`.
                //
                // `Style::default()` seeds every field with a theme value
                // (`text_color = foreground`, `background = background`,
                // `font_size`, …). Because `merge_style` treats any field
                // that differs from `Style::bare()` as "explicitly set",
                // those seeded values would be copied back when this
                // `Style` is merged into the component's fold chain,
                // overwriting whatever the variant's class string actually
                // set.
                //
                // Concretely: `ButtonVariant::Default` sets
                // `text-primary-foreground`. If `resolve()` returned
                // `Style::default()`, its `text_color` would be
                // `theme.foreground` — and merging `ButtonSize::Default`'s
                // `Style::default()` afterward would copy that back,
                // overwriting the variant's `text-primary-foreground`.
                //
                // Starting from `bare()` means only fields the class
                // string touches are non-bare, so `merge_style` copies
                // exactly those fields and leaves theme defaults alone.
                #krate::styling::StyledExt::style(
                    #krate::styling::Style::bare(),
                    self.style_str(),
                )
            }
        }

        impl ::core::convert::From<&str> for #enum_ident {
            /// Parse a kebab-case variant name. Unknown names fall back to
            /// the first declared variant.
            fn from(s: &str) -> Self {
                match s {
                    #( #kebab_lits => Self::#variant_idents, )*
                    _ => Self::#fallback,
                }
            }
        }

        impl ::core::convert::From<::std::string::String> for #enum_ident {
            fn from(s: ::std::string::String) -> Self {
                Self::from(s.as_str())
            }
        }
    })
}

/// Convert `PascalCase` to `kebab-case`.
///
/// `SecondaryDestructive` → `secondary-destructive`
/// `Md` → `md`
/// `XS` → `xs`
fn to_kebab_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            if i > 0 && (prev_lower || next_lower) {
                out.push('-');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(*c);
        }
    }
    out
}
