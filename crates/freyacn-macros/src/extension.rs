//! Bundles of code generated for each `#[extensions(...)]` name.
//!
//! The crate that the generated code refers to is configurable via a
//! leading `crate = <path>` argument. It defaults to `::freyacn` (the facade).
//!
//! ```ignore
//! #[extensions(children, key, styled)]
//! pub struct MyStruct { /* ... */ }
//!
//! #[extensions(crate = freyacn_core, children, key, styled)]
//! pub struct MyStruct { /* ... */ }
//! ```

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Fields, Ident, ItemStruct, LitStr, Path, Token};

// ---------------------------------------------------------------------------
// Supported extensions
// ---------------------------------------------------------------------------

const SUPPORTED: &[&str] = &[
    "children",
    "key",
    "styled",
    "event_handlers",
    "accessibility",
    "semantic_events",
];

fn dependencies_of(name: &str) -> &'static [&'static str] {
    match name {
        _ => &[],
    }
}

// ---------------------------------------------------------------------------
// Attribute argument parsing
// ---------------------------------------------------------------------------

pub struct ExtensionNames {
    pub names: Vec<String>,
    pub krate: Option<Path>,
}

impl Parse for ExtensionNames {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut names = Vec::new();
        let mut krate: Option<Path> = None;

        while !input.is_empty() {
            // `Ident::parse_any` is required to peek/consume the `crate`
            // keyword; `peek(Token![crate])` and `Ident::parse` both filter
            // it out and would produce "expected identifier, found keyword".
            let is_crate = {
                let fork = input.fork();
                fork.call(Ident::parse_any)
                    .map(|id| id == "crate")
                    .unwrap_or(false)
            };

            if is_crate {
                let _ = input.call(Ident::parse_any)?;
                input.parse::<Token![=]>()?;
                if krate.is_some() {
                    return Err(input.error("duplicate `crate = ...` argument"));
                }
                krate = Some(input.parse()?);
            } else {
                let ident: Ident = input.parse()?;
                names.push(ident.to_string());
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self { names, krate })
    }
}

pub fn parse_args(attr: TokenStream2) -> syn::Result<ExtensionNames> {
    syn::parse2(attr)
}

// ---------------------------------------------------------------------------
// Field-name lookup
// ---------------------------------------------------------------------------

pub fn field_names_for(name: &str) -> &'static [&'static str] {
    match name {
        "children" => &["children"],
        "key" => &["key"],
        "styled" => &["style"],
        "event_handlers" => &["event_handlers"],
        "accessibility" => &["accessibility"],
        "semantic_events" => &["semantic_events"],
        _ => &[],
    }
}

pub fn collect_field_names(names: &[String]) -> Vec<String> {
    names
        .iter()
        .flat_map(|n| field_names_for(n).iter().map(|s| s.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Extension bundle
// ---------------------------------------------------------------------------

struct Extension {
    fields: Vec<TokenStream2>,
    inits: Vec<TokenStream2>,
    locals: Vec<TokenStream2>,
    impls: Vec<TokenStream2>,
}

fn extension_for(
    krate: &Path,
    name: &str,
    struct_name: &Ident,
    variant_field_names: &[Ident],
    base_style: Option<&LitStr>,
) -> syn::Result<Extension> {
    let ext = match name {
        "children" => Extension {
            fields: vec![quote! {
                /// Child elements attached to this component.
                pub children: ::std::vec::Vec<#krate::prelude::Element>
            }],
            inits: vec![quote! { children: ::std::vec::Vec::new() }],
            locals: vec![quote! {
                let children: &::std::vec::Vec<#krate::prelude::Element> = &self.children;
            }],
            impls: vec![quote! {
                impl #krate::prelude::ChildrenExt for #struct_name {
                    fn children_mut(&mut self) -> &mut ::std::vec::Vec<#krate::prelude::Element> {
                        &mut self.children
                    }
                }
            }],
        },

        "key" => Extension {
            fields: vec![quote! {
                /// The element's diff key.
                pub key: #krate::prelude::DiffKey
            }],
            inits: vec![quote! { key: #krate::prelude::DiffKey::None }],
            locals: vec![quote! {
                let key: &#krate::prelude::DiffKey = &self.key;
            }],
            impls: vec![quote! {
                impl #krate::prelude::KeyExt for #struct_name {
                    fn key_mut(&mut self) -> &mut #krate::prelude::DiffKey {
                        &mut self.key
                    }
                }
            }],
        },

        // -------------------------------------------------------------------
        // styled → Style / StyledExt
        // -------------------------------------------------------------------
        //
        // `Style::merge_style` and `StyledExt::style` are both chainable:
        // they consume `self` and return a new `Style`. The generated fold
        // therefore threads `__freyacn_base` by value and reassigns:
        //
        //     __freyacn_base = StyledExt::style(__freyacn_base, base);
        //     __freyacn_base = __freyacn_base.merge_style(&variant);
        //     __freyacn_base = __freyacn_base.merge_style(&user);
        //
        // `merge_style` takes `&Style`, hence the `&` on each argument.
        //
        // `self.style` is initialised with `Style::bare()`, not
        // `Style::default()`. That keeps it a *sparse patch*: only fields
        // the user explicitly set via `.style("…")` / `.bg(..)` /
        // `.text_color(..)` / etc. are non-bare, so `merge_style` copies
        // exactly those fields and leaves theme defaults intact.
        //
        // Starting from `Style::default()` would seed `text_color =
        // theme.foreground`, `background = theme.background`, etc., and
        // merging that back would silently overwrite the variant's
        // `text-primary-foreground` (or any other class-string colour
        // the variant set).
        "styled" => {
            let has_base = base_style.is_some();
            let has_variants = !variant_field_names.is_empty();

            let locals = if !has_base && !has_variants {
                // Fast path: nothing to merge.
                vec![quote! {
                    let style: &#krate::styling::Style = &self.style;
                }]
            } else {
                let base_assign = match base_style {
                    Some(lit) => quote! {
                        __freyacn_base =
                            #krate::styling::StyledExt::style(__freyacn_base, #lit);
                    },
                    None => quote! {},
                };

                let merges = variant_field_names.iter().map(|field| {
                    quote! {
                        __freyacn_base = __freyacn_base.merge_style(
                            &#krate::prelude::ComponentVariant::resolve(&self.#field)
                        );
                    }
                });

                vec![quote! {
                    let __freyacn_merged_style: #krate::styling::Style = {
                        // `Style::bare()` — start transparent. `Style::default()`
                        // would seed `background = theme.background` and
                        // `text_color = theme.foreground`, making every
                        // `#[component]` opaque and overriding inherited text
                        // colour from ancestors. Matches CSS: a bare `<div>` is
                        // transparent, not opaque.
                        let mut __freyacn_base = #krate::styling::Style::bare();
                        #base_assign
                        #(#merges)*
                        __freyacn_base.merge_style(&self.style)
                    };
                    let style: &#krate::styling::Style = &__freyacn_merged_style;
                }]
            };

            Extension {
                fields: vec![quote! {
                    /// Accumulated user overrides.
                    pub style: #krate::styling::Style
                }],
                inits: vec![quote! {
                    style: #krate::styling::Style::bare()
                }],
                locals,
                impls: vec![quote! {
                    impl #krate::styling::StyledExt for #struct_name {
                        fn get_style(&mut self) -> &mut #krate::styling::Style {
                            &mut self.style
                        }
                    }
                }],
            }
        }

        "event_handlers" => Extension {
            fields: vec![quote! {
                /// Event-handler registry.
                pub event_handlers:
                    #krate::prelude::FxHashMap<#krate::prelude::EventName, #krate::prelude::EventHandlerType>
            }],
            inits: vec![quote! {
                event_handlers:
                    <#krate::prelude::FxHashMap<_, _> as ::core::default::Default>::default()
            }],
            locals: vec![quote! {
                let event_handlers: &#krate::prelude::FxHashMap<
                    #krate::prelude::EventName,
                    #krate::prelude::EventHandlerType,
                > = &self.event_handlers;
            }],
            impls: vec![quote! {
                impl #krate::prelude::EventHandlersExt for #struct_name {
                    fn event_handlers_mut(
                        &mut self,
                    ) -> &mut #krate::prelude::FxHashMap<
                        #krate::prelude::EventName,
                        #krate::prelude::EventHandlerType,
                    > {
                        &mut self.event_handlers
                    }
                }
            }],
        },

        "accessibility" => Extension {
            fields: vec![quote! {
                /// Accessibility data for this element.
                pub accessibility: #krate::prelude::AccessibilityData
            }],
            inits: vec![quote! {
                accessibility: <#krate::prelude::AccessibilityData as ::core::default::Default>::default()
            }],
            locals: vec![quote! {
                let accessibility: &#krate::prelude::AccessibilityData = &self.accessibility;
            }],
            impls: vec![quote! {
                impl #krate::prelude::AccessibilityExt for #struct_name {
                    fn accessibility_data_mut(
                        &mut self,
                    ) -> &mut #krate::prelude::AccessibilityData {
                        &mut self.accessibility
                    }
                }
            }],
        },

        "semantic_events" => Extension {
            fields: vec![quote! {
                /// Semantic events (hover / press / focus callbacks) attached
                /// to this component.
                pub semantic_events: #krate::prelude::SemanticEvents
            }],
            inits: vec![quote! {
                semantic_events: <#krate::prelude::SemanticEvents as ::core::default::Default>::default()
            }],
            locals: vec![quote! {
                let semantic_events: &#krate::prelude::SemanticEvents = &self.semantic_events;
            }],
            impls: vec![quote! {
                impl #krate::prelude::HasSemanticEvents for #struct_name {
                    fn semantic_events_mut(&mut self) -> &mut #krate::prelude::SemanticEvents {
                        &mut self.semantic_events
                    }
                }
            }],
        },

        other => {
            let valid = SUPPORTED
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Error::new(
                Span::call_site(),
                format!("Unsupported extension `{other}`. Valid names: {valid}."),
            ));
        }
    };
    Ok(ext)
}

// ---------------------------------------------------------------------------
// Collector
// ---------------------------------------------------------------------------

pub struct ExtensionsCode {
    pub field_defs: Vec<TokenStream2>,
    pub field_inits: Vec<TokenStream2>,
    pub local_bindings: Vec<TokenStream2>,
    pub trait_impls: Vec<TokenStream2>,
}

pub fn collect(
    krate: &Path,
    struct_name: &Ident,
    names: &[String],
    variant_field_names: &[Ident],
    base_style: Option<&LitStr>,
) -> syn::Result<ExtensionsCode> {
    let mut out = ExtensionsCode {
        field_defs: Vec::new(),
        field_inits: Vec::new(),
        local_bindings: Vec::new(),
        trait_impls: Vec::new(),
    };

    for name in names {
        for dep in dependencies_of(name) {
            if !names.iter().any(|n| n == dep) {
                return Err(Error::new(
                    Span::call_site(),
                    format!(
                        "`{name}` requires `{dep}` to also be listed in \
                         `ext(...)`. Add it, e.g. `ext({dep}, {name})`.",
                    ),
                ));
            }
        }

        let ext = extension_for(krate, name, struct_name, variant_field_names, base_style)?;
        out.field_defs.extend(ext.fields);
        out.field_inits.extend(ext.inits);
        out.local_bindings.extend(ext.locals);
        out.trait_impls.extend(ext.impls);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Standalone `#[extensions(...)]`
// ---------------------------------------------------------------------------

pub fn parse_extensions(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
    let item_struct: ItemStruct = syn::parse2(item)?;
    let struct_name = item_struct.ident.clone();

    let args = parse_args(attr)?;
    let krate: Path = args.krate.unwrap_or_else(|| syn::parse_quote!(::freyacn));
    let names = args.names;

    let code = collect(&krate, &struct_name, &names, &[], None)?;

    let Fields::Named(named) = &item_struct.fields else {
        return Err(Error::new_spanned(
            &struct_name,
            "`#[extensions]` can only be used on structs with named fields",
        ));
    };

    let existing_fields = named.named.iter();
    let extra_fields = &code.field_defs;
    let impls = &code.trait_impls;

    let attrs = &item_struct.attrs;
    let vis = &item_struct.vis;
    let generics = &item_struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ext_list = names
        .iter()
        .map(|n| format!("* `{n}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let generated_note =
        format!("## Extensions\n\nThis struct has been extended with:\n\n{ext_list}");

    Ok(quote! {
        #(#attrs)*
        #[doc = ""]
        #[doc = #generated_note]
        #vis struct #struct_name #generics #where_clause {
            #(#existing_fields,)*
            #(#extra_fields,)*
        }

        #(#impls)*

        #[allow(unused)]
        const _: fn() = || {
            fn _assert_generics #impl_generics () #where_clause {}
            let _ = _assert_generics::<#ty_generics>;
        };
    })
}
