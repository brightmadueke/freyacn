//! The `#[component(...)]` attribute macro.

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Expr, FnArg, Ident, ItemFn, LitStr, Pat, PatType, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::extension;

// ---------------------------------------------------------------------------
// Prop declaration parsing
// ---------------------------------------------------------------------------
//
// Grammar (shared by `props(...)`, `variants(...)`, and
// `#[struct_fields(...)]`):
//
//     prop := ident ":" type [ "=" expr ]
//     list := prop { "," prop } [ "," ]

struct PropDecl {
    name: Ident,
    ty: Type,
    default: Option<Expr>,
}

impl Parse for PropDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        if name == "required" {
            return Err(syn::Error::new_spanned(
                &name,
                "`required` is not a keyword. Function parameters are the only \
                 way to declare required props: `fn Card(name: String)`. \
                 Attribute props are optional (or defaulted with `= expr`).",
            ));
        }

        if !input.peek(Token![:]) {
            return Err(syn::Error::new_spanned(
                &name,
                "missing type — every field must declare its type, e.g. \
                 `name: String`. (Required props are declared via the function \
                 signature instead.)",
            ));
        }
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;

        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { name, ty, default })
    }
}

// ---------------------------------------------------------------------------
// `#[component(props(…), ext(…), variants(…))]`
// ---------------------------------------------------------------------------

struct ComponentArgs {
    props: Vec<PropDecl>,
    ext: Vec<String>,
    variants: Vec<PropDecl>,
    /// Optional leading string literal inside `variants(...)` — e.g.
    /// `variants("p-4 w-3", size: ButtonSize = ButtonSize::Default)`.
    /// Applied at the bottom of the style merge chain, so variants and
    /// user overrides still win.
    base_style: Option<LitStr>,
}

impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut props = Vec::new();
        let mut ext = Vec::new();
        let mut variants = Vec::new();
        let mut base_style: Option<LitStr> = None;

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            let content;
            syn::parenthesized!(content in input);

            match name.to_string().as_str() {
                "props" => {
                    let items = Punctuated::<PropDecl, Token![,]>::parse_terminated(&content)?;
                    props.extend(items);
                }
                "ext" => {
                    let items = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
                    ext.extend(items.into_iter().map(|i| i.to_string()));
                }
                "variants" => {
                    // Optional leading base-style string literal.
                    if content.peek(LitStr) {
                        let lit: LitStr = content.parse()?;
                        if base_style.is_some() {
                            return Err(syn::Error::new_spanned(
                                &lit,
                                "duplicate base-style string in `variants(...)`; \
                                 only one is allowed",
                            ));
                        }
                        base_style = Some(lit);
                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    if !content.is_empty() {
                        let items = Punctuated::<PropDecl, Token![,]>::parse_terminated(&content)?;
                        variants.extend(items);
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &name,
                        format!(
                            "unknown group `{other}`. \
                             Expected `props(...)`, `ext(...)`, or `variants(...)`."
                        ),
                    ));
                }
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            props,
            ext,
            variants,
            base_style,
        })
    }
}

// ---------------------------------------------------------------------------
// Resolved fields
// ---------------------------------------------------------------------------

enum PropKind {
    /// From a function parameter. Stored as `Option<T>`, initialised to
    /// `None`, body sees `&T` (panics if unset).
    Required,
    /// No default. Stored as `Option<T>`, initialised to `None`, body sees
    /// `&Option<T>`.
    Optional,
    /// Has `= expr`. Stored as `T`, initialised with `expr`, body sees `&T`.
    WithDefault(Expr),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PropSource {
    FnParam,
    AttrProp,
    StructField,
    Variant,
}

fn source_label(source: PropSource) -> &'static str {
    match source {
        PropSource::FnParam => "a function parameter",
        PropSource::AttrProp => "an attribute prop (`props(...)`)",
        PropSource::StructField => "a struct field (`#[struct_fields(...)]`)",
        PropSource::Variant => "a variant prop (`variants(...)`)",
    }
}

struct Prop {
    name: Ident,
    ty: Type,
    kind: PropKind,
    source: PropSource,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn parse_component(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
    let mut func: ItemFn = syn::parse2(item)?;

    // =====================================================================
    // 0. Extract attributes.
    // =====================================================================

    // ---- 0a. `#[struct_fields(...)]` — pulled out first. ----
    let mut struct_field_decls: Vec<PropDecl> = Vec::new();
    let mut kept_attrs: Vec<Attribute> = Vec::new();
    for a in std::mem::take(&mut func.attrs) {
        if a.path().is_ident("struct_fields") {
            let decls = a.parse_args_with(Punctuated::<PropDecl, Token![,]>::parse_terminated)?;
            struct_field_decls.extend(decls);
        } else {
            kept_attrs.push(a);
        }
    }
    func.attrs = kept_attrs;

    // ---- 0b. Reject leftover `#[extensions(...)]`. ----
    if let Some(pos) = func
        .attrs
        .iter()
        .position(|a| a.path().is_ident("extensions"))
    {
        return Err(syn::Error::new_spanned(
            &func.attrs[pos],
            "`#[extensions(...)]` is no longer an attribute on `#[component]`. \
             Move the extension names into the `ext(...)` group, e.g. \
             `#[component(ext(children, style))]`.",
        ));
    }

    // ---- 0c. Preserve user doc comments. ----
    let user_docs: Vec<Attribute> = func
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .cloned()
        .collect();

    // ---- 0d. Everything remaining is forwarded to the struct. ----
    let forwarded_attrs: Vec<Attribute> = func
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("doc"))
        .cloned()
        .collect();

    // =====================================================================
    // 1. Names.
    // =====================================================================

    let fn_name_str = func.sig.ident.to_string();
    let fn_ident = format_ident!("{}", to_pascal_case(&fn_name_str));
    let struct_ident = format_ident!("{}Component", fn_ident);
    let vis = func.vis.clone();

    // =====================================================================
    // 2. Parse `#[component(...)]` groups.
    // =====================================================================

    let args: ComponentArgs = syn::parse2(attr)?;
    let base_style = args.base_style;
    let extension_names = args.ext;
    let attr_props = args.props;
    let variant_props = args.variants;

    // Every variant prop must declare a default.
    for v in &variant_props {
        if v.default.is_none() {
            return Err(syn::Error::new_spanned(
                &v.name,
                format!(
                    "variant prop `{}` requires a default, e.g. \
                     `{}: MyEnum = MyEnum::Default`",
                    v.name, v.name,
                ),
            ));
        }
    }

    // Variants require `ext(style)` — the merged style needs somewhere to
    // live in the render body.
    if !variant_props.is_empty() && !extension_names.iter().any(|n| n == "style") {
        return Err(syn::Error::new_spanned(
            &variant_props[0].name,
            "`variants(...)` requires `style` to also be listed in `ext(...)`, \
             so the resolved variant stylesheet have somewhere to be merged. \
             Add `style`, e.g. `ext(style, children)`.",
        ));
    }

    // Base style lives in the same merge chain and therefore needs
    // `ext(style)` too.
    if base_style.is_some() && !extension_names.iter().any(|n| n == "style") {
        return Err(syn::Error::new_spanned(
            base_style.as_ref().unwrap(),
            "`variants(\"...\")` base style requires `style` to also be listed \
             in `ext(...)`, so the base can be folded into the render body's \
             style. Add `style`, e.g. `ext(style, children)`.",
        ));
    }

    let variant_field_names: Vec<Ident> = variant_props.iter().map(|p| p.name.clone()).collect();

    // =====================================================================
    // 3. Extension bundle.
    // =====================================================================

    let extensions_code = extension::collect(
        &struct_ident,
        &extension_names,
        &variant_field_names,
        base_style.as_ref(),
    )?;
    let ext_field_defs = &extensions_code.field_defs;
    let ext_field_inits = &extensions_code.field_inits;
    let ext_local_bindings = &extensions_code.local_bindings;
    let ext_trait_impls = &extensions_code.trait_impls;
    let ext_field_names = extension::collect_field_names(&extension_names);

    // =====================================================================
    // 4. Function parameters (all required).
    // =====================================================================

    let fn_params: Vec<(Ident, Type)> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => match &**pat {
                Pat::Ident(pi) => Some((pi.ident.clone(), (**ty).clone())),
                _ => None,
            },
            _ => None,
        })
        .collect();

    // =====================================================================
    // 5. Build `props` in declaration order:
    //       props(...) → struct_fields(...) → variants(...) → fn params
    // =====================================================================

    let mut props: Vec<Prop> = Vec::new();

    for decl in attr_props {
        let kind = match decl.default {
            Some(expr) => PropKind::WithDefault(expr),
            None => PropKind::Optional,
        };
        props.push(Prop {
            name: decl.name,
            ty: decl.ty,
            kind,
            source: PropSource::AttrProp,
        });
    }

    for decl in struct_field_decls {
        let kind = match decl.default {
            Some(expr) => PropKind::WithDefault(expr),
            None => PropKind::Optional,
        };
        props.push(Prop {
            name: decl.name,
            ty: decl.ty,
            kind,
            source: PropSource::StructField,
        });
    }

    for decl in variant_props {
        let expr = decl.default.expect("validated above");
        props.push(Prop {
            name: decl.name,
            ty: decl.ty,
            kind: PropKind::WithDefault(expr),
            source: PropSource::Variant,
        });
    }

    for (name, ty) in &fn_params {
        props.push(Prop {
            name: name.clone(),
            ty: ty.clone(),
            kind: PropKind::Required,
            source: PropSource::FnParam,
        });
    }

    // =====================================================================
    // 6. Collision checks.
    // =====================================================================

    let mut seen: std::collections::HashMap<String, PropSource> = std::collections::HashMap::new();
    for prop in &props {
        let key = prop.name.to_string();
        if let Some(previous) = seen.get(&key) {
            return Err(syn::Error::new_spanned(
                &prop.name,
                format!(
                    "`{key}` is declared more than once ({} and {}). \
                     Each field name may be used by exactly one declaration.",
                    source_label(*previous),
                    source_label(prop.source),
                ),
            ));
        }
        seen.insert(key, prop.source);
    }

    for prop in &props {
        if ext_field_names.iter().any(|n| n == &prop.name.to_string()) {
            return Err(syn::Error::new_spanned(
                &prop.name,
                format!(
                    "`{}` collides with a field contributed by `ext(...)`. \
                     Extension fields are reserved by freyacn and cannot be \
                     shadowed by {}.",
                    prop.name,
                    source_label(prop.source),
                ),
            ));
        }
    }

    // =====================================================================
    // 7. Struct fields.
    // =====================================================================

    let prop_fields = props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        let ty_str = quote!(#ty).to_string().replace(' ', "");

        let (storage_ty, doc) = match (&p.kind, p.source) {
            (PropKind::Required, _) => (
                quote! { ::core::option::Option<#ty> },
                format!(
                    "The `{name}` prop (`{ty_str}`).\n\n\
                     **Required.** Declared as a function parameter; stored as \
                     `Option<{ty_str}>` and initialised to `None`. The render \
                     body unwraps it and panics if it was never set.\n\n\
                     Set via [`Self::{name}`]."
                ),
            ),
            (PropKind::Optional, PropSource::StructField) => (
                quote! { ::core::option::Option<#ty> },
                format!(
                    "Internal storage field `{name}` (`{ty_str}`).\n\n\
                     **Struct field.** Declared via `#[struct_fields(...)]`; \
                     initialised to `None`. No setter is generated.\n\n\
                     The render body sees `&Option<{ty_str}>`."
                ),
            ),
            (PropKind::Optional, _) => (
                quote! { ::core::option::Option<#ty> },
                format!(
                    "The `{name}` prop (`{ty_str}`).\n\n\
                     **Optional.** Declared in `props(...)` without a default; \
                     stored as `Option<{ty_str}>` and initialised to `None`. \
                     The render body sees `&Option<{ty_str}>`.\n\n\
                     Set via [`Self::{name}`]."
                ),
            ),
            (PropKind::WithDefault(_), PropSource::StructField) => (
                quote! { #ty },
                format!(
                    "Internal storage field `{name}` (`{ty_str}`).\n\n\
                     **Struct field.** Declared via `#[struct_fields(...)]` \
                     with a default expression; initialised with it in \
                     [`Self::new`]. No setter is generated.\n\n\
                     The render body sees `&{ty_str}`."
                ),
            ),
            (PropKind::WithDefault(_), PropSource::Variant) => (
                quote! { #ty },
                format!(
                    "The `{name}` variant prop (`{ty_str}`).\n\n\
                     **Variant.** Declared in `variants(...)` with a default. \
                     Stored as `{ty_str}` and initialised with that default. \
                     Its style is merged into the render body's `style` local \
                     before the user overrides.\n\n\
                     Set via [`Self::{name}`]; accepts any `impl Into<{ty_str}>` \
                     — including `&str` and `String` when the enum implements \
                     `From`."
                ),
            ),
            (PropKind::WithDefault(_), _) => (
                quote! { #ty },
                format!(
                    "The `{name}` prop (`{ty_str}`).\n\n\
                     **Defaulted.** Declared in `props(...)` with a default \
                     expression; stored as `{ty_str}` and initialised with it \
                     in [`Self::new`]. The render body sees `&{ty_str}`.\n\n\
                     Set via [`Self::{name}`]."
                ),
            ),
        };

        quote! {
            #[doc = #doc]
            #vis #name: #storage_ty
        }
    });

    // =====================================================================
    // 8. `new()` initialisers.
    // =====================================================================

    let prop_inits = props.iter().map(|p| {
        let name = &p.name;
        match &p.kind {
            PropKind::Required | PropKind::Optional => quote! {
                #name: ::core::option::Option::None
            },
            PropKind::WithDefault(expr) => quote! {
                #name: #expr
            },
        }
    });

    // =====================================================================
    // 9. Setters.
    // =====================================================================

    let prop_setters = props
        .iter()
        .filter(|p| p.source != PropSource::StructField)
        .map(|p| {
            let name = &p.name;
            let ty = &p.ty;
            let ty_str = quote!(#ty).to_string().replace(' ', "");

            let (accepts, doc, assign) = match &p.kind {
                PropKind::Required => (
                    quote! { impl ::core::convert::Into<#ty> },
                    format!(
                        "Set the [`{name}`](Self::{name}) prop.\n\n\
                         **Required.** Accepts any type convertible into \
                         `{ty_str}`. Passing `None` or `Some(…)` is a type \
                         error, which is intentional."
                    ),
                    quote! { ::core::option::Option::Some(::core::convert::Into::into(value)) },
                ),
                PropKind::Optional => (
                    quote! { impl ::core::convert::Into<::core::option::Option<#ty>> },
                    format!(
                        "Set the [`{name}`](Self::{name}) prop.\n\n\
                         Optional prop. Accepts an `Option<{ty_str}>` directly."
                    ),
                    quote! { ::core::convert::Into::into(value) },
                ),
                PropKind::WithDefault(_) => {
                    let kind_label = if p.source == PropSource::Variant {
                        "Variant prop"
                    } else {
                        "Defaulted prop"
                    };
                    (
                        quote! { impl ::core::convert::Into<#ty> },
                        format!(
                            "Set the [`{name}`](Self::{name}) prop.\n\n\
                             {kind_label} — accepts any type convertible into \
                             `{ty_str}`."
                        ),
                        quote! { ::core::convert::Into::into(value) },
                    )
                }
            };

            quote! {
                #[doc = #doc]
                #[allow(dead_code)]
                #vis fn #name(
                    mut self,
                    value: #accepts,
                ) -> Self {
                    self.#name = #assign;
                    self
                }
            }
        });

    // =====================================================================
    // 10. Render-body bindings.
    // =====================================================================

    let prop_bindings = props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        match &p.kind {
            PropKind::Required => quote! {
                let #name: &#ty = self
                    .#name
                    .as_ref()
                    .expect(::core::concat!(
                        "required prop `",
                        ::core::stringify!(#name),
                        "` was not set before render"
                    ));
            },
            PropKind::Optional => quote! {
                let #name: &::core::option::Option<#ty> = &self.#name;
            },
            PropKind::WithDefault(_) => quote! {
                let #name: &#ty = &self.#name;
            },
        }
    });

    let body = &func.block;

    // =====================================================================
    // 11. Generated documentation.
    // =====================================================================

    let prop_summary: Vec<String> = props
        .iter()
        .filter(|p| p.source != PropSource::StructField)
        .map(|p| {
            let ty = &p.ty;
            let ty_str = quote!(#ty).to_string().replace(' ', "");
            let kind = match (&p.kind, p.source) {
                (_, PropSource::Variant) => "variant",
                (PropKind::Required, _) => "required (fn param)",
                (PropKind::Optional, _) => "optional",
                (PropKind::WithDefault(_), _) => "with default",
            };
            format!("* `{}`: `{}` — {}", p.name, ty_str, kind)
        })
        .collect();

    let props_section = if prop_summary.is_empty() {
        "*No props.*".to_string()
    } else {
        prop_summary.join("\n")
    };

    let struct_field_summary: Vec<String> = props
        .iter()
        .filter(|p| p.source == PropSource::StructField)
        .map(|p| {
            let ty = &p.ty;
            let ty_str = quote!(#ty).to_string().replace(' ', "");
            let kind = match &p.kind {
                PropKind::Optional => "`None`",
                PropKind::WithDefault(_) => "with default",
                PropKind::Required => unreachable!(),
            };
            format!("* `{}`: `{}` — {}", p.name, ty_str, kind)
        })
        .collect();

    let struct_fields_section = if struct_field_summary.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n## Struct fields\n\n\
             Internal storage. No setters are generated; manage these fields \
             via your own trait impls.\n\n{}",
            struct_field_summary.join("\n"),
        )
    };

    let base_style_section = match &base_style {
        Some(lit) => format!(
            "\n\n## Base style\n\n\
             Applied at the bottom of the style merge chain — variants and \
             user overrides win over it:\n\n```text\n{}\n```",
            lit.value(),
        ),
        None => String::new(),
    };

    let ext_section = if extension_names.is_empty() {
        "*No extensions.*".to_string()
    } else {
        extension_names
            .iter()
            .map(|n| format!("* `{n}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let struct_doc = format!(
        "Component generated from the function `{fn_name}`.\n\n\
         ## Props\n\n{props_section}\
         {struct_fields_section}\
         {base_style_section}\n\n\
         ## Extensions\n\n{ext_section}",
        fn_name = fn_name_str,
    );

    let new_doc = format!(
        "Create a new [`{struct_ident}`] with every prop in its initial state.\n\n\
         Chain setters to configure props:\n\n\
         ```\n\
         {struct_ident}::new().prop1(value1).prop2(value2)\n\
         ```"
    );

    let ctor_doc = format!(
        "Construct a new [`{struct_ident}`].\n\n\
         Equivalent to [`{struct_ident}::new`]. Chain setters to configure props:\n\n\
         ```\n\
         {fn_ident}().prop1(value1).prop2(value2)\n\
         ```"
    );

    let render_doc = format!(
        "Renders this [`{struct_ident}`].\n\n\
         Props are bound by reference as locals: required, defaulted, and \
         variant props as `&T`, optional props and struct fields as \
         `&Option<T>`. Extension fields are bound the same way. When the \
         component declares `variants(...)` (with or without a base style), \
         the `style` local is the merge of the base style, every variant's \
         resolved style, and the user's overrides."
    );

    let macro_doc = format!(
        "Sugar macro for constructing [`{struct_ident}`].\n\n\
         ```\n\
         {fn_ident}!(prop1 = value1, prop2 = value2)\n\
         ```"
    );

    // =====================================================================
    // 12. Component impl.
    // =====================================================================

    let component_trait_impl = quote! {
        impl ::freyacn::prelude::Component for #struct_ident {
            #[doc = #render_doc]
            fn render(&self) -> impl ::freyacn::prelude::IntoElement {
                #(#ext_local_bindings)*
                #(#prop_bindings)*
                #body
            }
        }
    };

    // =====================================================================
    // 13. Companion macro.
    // =====================================================================

    let companion_macro = quote! {
        #[doc = #macro_doc]
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #fn_ident {
            ($($key:ident = $val:expr),* $(,)?) => {
                #struct_ident::new()$(.$key($val))*
            };
        }
    };

    // =====================================================================
    // 14. Final expansion.
    // =====================================================================

    let expanded = quote! {
        #(#user_docs)*
        #[doc = ""]
        #[doc = #struct_doc]
        #(#forwarded_attrs)*
        #vis struct #struct_ident {
            #(#prop_fields,)*
            #(#ext_field_defs,)*
        }

        impl #struct_ident {
            #[doc = #new_doc]
            #[allow(dead_code)]
            #vis fn new() -> Self {
                Self {
                    #(#prop_inits,)*
                    #(#ext_field_inits,)*
                }
            }

            #(#prop_setters)*
        }

        #[doc = "Default constructor — delegates to [`Self::new`]."]
        impl ::core::default::Default for #struct_ident {
            fn default() -> Self {
                Self::new()
            }
        }

        #[doc = #ctor_doc]
        #[allow(non_snake_case)]
        #vis fn #fn_ident() -> #struct_ident {
            #struct_ident::new()
        }

        #(#ext_trait_impls)*

        #component_trait_impl

        #companion_macro
    };

    Ok(expanded)
}

/// Convert `snake_case` to `PascalCase`.
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
