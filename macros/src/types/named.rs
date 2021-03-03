use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, FieldsNamed, Result};

use crate::attr::{FieldAttr, Inflection};
use crate::utils::convert_lifetime_to_static;
use crate::DerivedTS;

pub(crate) fn named(
    name: &str,
    rename_all: &Option<Inflection>,
    fields: &FieldsNamed,
    generics: TokenStream,
) -> Result<DerivedTS> {
    let mut formatted_fields = vec![];
    let mut dependencies = vec![];
    for field in &fields.named {
        format_field(&mut formatted_fields, &mut dependencies, field, &rename_all)?;
    }

    let fields = quote!(vec![#(#formatted_fields),*].join("\n"));

    Ok(DerivedTS {
        inline: quote! {
            format!(
                "{{\n{}\n{}}}",
                #fields,
                " ".repeat(indent * 4)
            )
        },
        decl: quote!(format!("export interface {} {}", #name, Self::inline(0))),
        inline_flattened: Some(fields),
        name: name.to_owned(),
        dependencies: quote! {
            let mut dependencies = vec![];
            #( #dependencies )*
            dependencies
        },
        generics,
    })
}

// build an expresion which expands to a string, representing a single field of a struct.
fn format_field(
    formatted_fields: &mut Vec<TokenStream>,
    dependencies: &mut Vec<TokenStream>,
    field: &Field,
    rename_all: &Option<Inflection>,
) -> Result<()> {
    let FieldAttr {
        type_override,
        rename,
        inline,
        skip,
        flatten,
    } = FieldAttr::from_attrs(&field.attrs)?;

    if skip {
        return Ok(());
    }

    let ty = convert_lifetime_to_static(&field.ty);

    if flatten {
        match (&type_override, &rename, inline) {
            (Some(_), _, _) => syn_err!("`type` is not compatible with `flatten`"),
            (_, Some(_), _) => syn_err!("`rename` is not compatible with `flatten`"),
            (_, _, true) => syn_err!("`inline` is not compatible with `flatten`"),
            _ => {}
        }

        formatted_fields.push(quote!(<#ty as ts_rs::TS>::inline_flattened(indent)));
        dependencies.push(quote!(dependencies.append(&mut <#ty as ts_rs::TS>::dependencies());));
        return Ok(());
    }

    if type_override.is_none() {
        dependencies.push(match inline {
            true => quote! { dependencies.append(&mut <#ty as ts_rs::TS>::dependencies()); },
            false => quote! {
                if <#ty as ts_rs::TS>::transparent() {
                    dependencies.append(&mut <#ty as ts_rs::TS>::dependencies());
                } else {
                    dependencies.push((std::any::TypeId::of::<#ty>(), <#ty as ts_rs::TS>::name()));
                }
            },
        });
    }

    let formatted_ty = type_override
        .map(|t| quote!(#t))
        .unwrap_or_else(|| match inline {
            true => quote!(<#ty as ts_rs::TS>::inline(indent + 1)),
            false => quote!(<#ty as ts_rs::TS>::name()),
        });
    let name = match (rename, rename_all) {
        (Some(rn), _) => rn,
        (None, Some(rn)) => rn.apply(&field.ident.as_ref().unwrap().to_string()),
        (None, None) => field.ident.as_ref().unwrap().to_string(),
    };

    formatted_fields.push(quote! {
        format!("{}{}: {},", " ".repeat((indent + 1) * 4), #name, #formatted_ty)
    });

    Ok(())
}
