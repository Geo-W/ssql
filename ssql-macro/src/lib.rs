use proc_macro::TokenStream;

use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Field, Ident};

use derive_ssql::DeriveSsql;

mod derive_ssql;
mod utils;

#[proc_macro_derive(ORM, attributes(ssql))]
pub fn ssql(tokens: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    let mut impls = DeriveSsql::new(ast);
    impls.impl_table_name();
    impls.impl_fields();

    impls.impl_query();
    impls.impl_primary_key();
    impls.impl_insert();
    impls.impl_insert_ignore_pk();
    impls.impl_insert_many();
    impls.impl_update();
    impls.impl_delete();
    impls.impl_relationship();

    impls.impl_row_to_struct();
    impls.impl_row_to_json();

    impls.finalize()

    // #[cfg(feature = "polars")]
    // build_polars(struct_name, fields, &mut result);

    // result.into()
}

fn build_polars(
    struct_name: Ident,
    fields: &Punctuated<Field, Comma>,
    result: &mut proc_macro2::TokenStream,
) {
    #[cfg(feature = "polars")]
    let builder_new_vecs = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        let ty = &f.ty;
        quote! {
            let mut #field : Vec<#ty> = vec![]
        }
    });

    #[cfg(feature = "polars")]
    let builder_insert_to_df = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        quote! {
            #field.push(Phant_Name1.#field)
        }
    });

    #[cfg(feature = "polars")]
    let builder_df = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        let mn = field.to_string();
        quote! {
            #mn => #field
        }
    });

    #[cfg(feature = "polars")]
    result.extend(quote! {
        impl PolarsHelper for #struct_name {
            fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame> {
                #(#builder_new_vecs;)*
                #[allow(non_snake_case)]
                for Phant_Name1 in vec {
                    #(#builder_insert_to_df;)*
                }
                df!(
                    #(#builder_df,)*
                )
            }
        }
    });
}
