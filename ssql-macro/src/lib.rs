use proc_macro::TokenStream;

use derive_ssql::DeriveSsql;

mod derive_ssql;
mod utils;

#[proc_macro_derive(ORM, attributes(ssql))]
pub fn ssql(tokens: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();

    let mut impls = DeriveSsql::new(&ast);
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

    #[cfg(feature = "polars")]
    impls.impl_dataframe();

    impls.finalize()
}

