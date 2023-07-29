use proc_macro::TokenStream;

use quote::{quote, ToTokens};
// no need to import a specific crate for TokenStream
use syn::{Expr, ExprLit, Lit};
use syn::{Meta, Path, punctuated::Punctuated, token::Comma};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;

use crate::utils::{
    parse_table_name,
    extract_type_from_option
};

mod utils;

// use proc_macro2::{Span, TokenTree};
// use proc_macro2::TokenTree::Group;
// use proc_macro2::Ident;
// use syn::{parse_macro_input, DeriveInput};
// use syn::token::Token;


#[proc_macro_derive(ORM, attributes(rusql))]
pub fn show_streams(tokens: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", tokens.to_string());
    // let t: proc_macro2::TokenStream = tokens.clone().into();
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    // let struct_name = ast.ident;
    let table_name = parse_table_name(&ast.attrs);
    let struct_name = ast.ident;

    // println!("attrs: {:?}", ast.attrs);
    // println!("data: {:?}", ast.data);

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!()
    };

    let builder_types = fields.iter().map(|f| {
        let mn = f.clone().ident.unwrap().to_string();
        let ty = &f.ty.to_token_stream().to_string();
        quote! {
            #mn => #ty
        }
    });

    let builder_fields_mapping = fields.iter().map(|f| f.clone().ident.unwrap().to_string());

    let builder_row_func = fields.iter().map(|f| {
        let mn = f.clone().ident.unwrap().to_string();
        let field_name = format!("{}.{}", &table_name, &mn);
        let ty = &f.ty;
        let ty = match extract_type_from_option(ty) {
            Some(value) => value,
            None => ty
        };
        let type_name = ty.to_token_stream().to_string();
        return match type_name.as_str() {
            "String" => {
                quote! {
                    map.insert(#mn.to_string(), row.get::<&str, &str>(#field_name).into())
                }
            }
            "NaiveDateTime" => {
                quote! {
                    map.insert(#mn.to_string(), row.get::<#ty, &str>(#field_name).unwrap().to_string().into())
                }
            }
            _ => {
                quote! {
                    map.insert(#mn.to_string(), row.get::<#ty, &str>(#field_name).into())
                }
            }
        };
    });

    // for pushing elements to TokenRow which used in bulk insert
    let builder_insert_rows = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        return quote! {
            row.push(item.#field.into_sql())
        };
    });


    // let builder_insert_fields = fields.iter()
    //     .map(|f| { f.clone().ident.unwrap().to_string() })
    //     .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    // let mut fields_count = 0;
    // let builder_insert_params = fields.iter()
    //     .map(|f| {
    //         fields_count += 1;
    //         return format!("@p{}", fields_count);
    //     })
    //     .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    // let builder_insert_data = fields.iter().map(|f|
    //     f.clone().ident.unwrap()
    // )
    //     // .filter(|x| { *x.to_string() != "id".to_string() })
    //     .map(|f| return quote! {&self.#f});


    // for getting dataframe
    let builder_new_vecs = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        // let mn = field.to_string();
        let ty = &f.ty;
        // let ty = match extract_type_from_option(ty) {
        //     Some(value) => value,
        //     None => ty
        // };
        quote!{
            let mut #field : Vec<#ty> = vec![]
        }
    });

    let builder_insert_to_df = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        quote!{
            #field.push(Phant_Name1.#field)
        }
    });

    let builder_df = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        let mn = field.to_string();
        quote!{
            #mn => #field
        }
    });

    // for getting vectors of self struct
    let builder_row_to_self_func = fields.iter().map(|f| {
        let mn = f.clone().ident.unwrap();
        let field_name = format!("{}.{}", &table_name, &mn.to_string());
        let ty = &f.ty;
        return match extract_type_from_option(ty) {
            Some(value) => {
                let type_name = value.to_token_stream().to_string();
                match type_name.as_str() {
                    "String" => {
                        quote! {
                            #mn: row.get::<&str, &str>(#field_name).map(|i| i.to_string())
                        }
                    }
                    _ => {
                        quote! {
                            #mn: row.get::<#value, &str>(#field_name)
                        }
                    }
                }
            },
            None => {
                let type_name = ty.to_token_stream().to_string();
                match type_name.as_str() {
                    "String" => {
                        quote! {
                            #mn: row.get::<&str, &str>(#field_name).unwrap().to_string()
                        }
                    }
                    _ => {
                        quote! {
                            #mn: row.get::<#ty, &str>(#field_name).unwrap()
                        }
                    }
                }
            }
        };
    });

    let mut result = quote! {
    };

    let mut relations: Vec<String> = vec![];
    let mut tables: Vec<String> = vec![];
    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap().to_string();
        for attr in field.attrs.iter() {
            if let Some(ident) = attr.path().get_ident() {
                if ident == "rusql" {
                    if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                        for meta in list.iter() {
                            if let Meta::NameValue(named_v) = meta {
                                let Path { ref segments, .. } = &named_v.path;
                                for rusql_segs in segments.iter() {
                                    if rusql_segs.ident == "foreign_key" {
                                        // let b = &named_v.value;
                                        if let Expr::Lit(ExprLit { lit, .. }) = &named_v.value {
                                            if let Lit::Str(v) = lit {
                                                relations.push(format!("{}.{} = {}", &table_name, field_name, v.value()));
                                                tables.push(v.value()[..v.value().find('.').unwrap()].to_string());
                                            }
                                        }
                                        // if let Expr::Path(p_v) = &named_v.value {
                                        //     dbg!(&p_v);
                                        //     for seg in p_v.path.segments.iter() {
                                        //         let i = &seg.ident;
                                        //         result.extend(quote! {
                                        //                 impl #struct_name {
                                        //                     fn #i() -> String{
                                        //                         "asdf".to_string()
                                        //                     }
                                        //                 }
                                        //             })
                                        //     }
                                        // }
                                        // if let Expr::Lit{, ..} = &named_v.value {
                                        //     dbg!(&p_v);
                                        //     for seg in p_v.path.segments.iter() {
                                        //         let i = &seg.ident;
                                        //         result.extend(quote! {
                                        //                 impl #struct_name {
                                        //                     fn #i() -> String{
                                        //                         "asdf".to_string()
                                        //                     }
                                        //                 }
                                        //             })
                                        //     }
                                        // }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let builder_fields = relations.iter().zip(tables.iter()).map(|(rel, tb)| {
        quote! { #tb => {
            concat!(" ", #tb, " ON ", #rel)
            // format!("JOIN {} ON {}", #tb, #rel)
        }}
    });

    result.extend(quote! {
        #[async_trait(?Send)]
        impl RusqlMarker for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn fields() -> Vec<&'static str> {
                vec![#(#builder_fields_mapping,)*]
            }

            fn row_to_json(row:&Row) -> Map<String, Value> {
                let mut map = Map::new();
                #(#builder_row_func;)*
                map
            }

            fn row_to_self(row:&Row) -> Self {
                Self{
                    #(#builder_row_to_self_func,)*
                }
            }

            async fn insert_many(iter: impl Iterator<Item = #struct_name> , mut conn: Client<Compat<TcpStream>>) -> Result<u64, RssqlError>
            // where I:  impl Iterator<Item = #struct_name>
            {
                let mut req = conn.bulk_insert(#table_name).await?;
                for item in iter{
                    let mut row = TokenRow::new();
                    #(#builder_insert_rows;)*
                    req.send(row).await?;
                }
                let res = req.finalize().await?;
                Ok(res.total())
            }

            // async fn insert_one(self, mut conn: Client<Compat<TcpStream>>) -> Result<(), RssqlError> {
            //     let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields, #builder_insert_params);
            //     db.execute(sql, &[#(#builder_insert_data,)*]).await?;
            //     Ok(())
            // }


        }
        impl #struct_name {

            fn relationship(input: &str) -> &'static str {
                match input {
                    #(#builder_fields,)*
                    _ =>  unimplemented!("relationship not found"),
                }
            }

            fn column_type(input: &str) -> &'static str{
                match input {
                    #(#builder_types,)*
                    _ =>  unimplemented!("column_type not found"),
                }
            }

            pub fn query() -> QueryBuilder {
                QueryBuilder::new(#table_name,
                    (#table_name, #struct_name::fields()),
                    #struct_name::relationship,
                    Box::new(#struct_name::row_to_json))
            }

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
    } );

    result.into()
}

