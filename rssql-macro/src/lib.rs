use proc_macro::TokenStream;
use std::iter::{Map, Zip};
use std::slice::Iter;
// no need to import a specific crate for TokenStream
use syn::{Attribute, Expr, ExprLit, Lit, parse};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::{Field, punctuated::Punctuated, token::Comma, Meta, Path};
use syn::FieldsNamed;
use proc_macro2::{Span, TokenTree};
use proc_macro2::TokenTree::Group;
use proc_macro2::Ident;
use syn::token::Token;


#[proc_macro_derive(ORM, attributes(rusql))]
pub fn show_streams(tokens: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", tokens.to_string());
    let t: proc_macro2::TokenStream = tokens.clone().into();
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    // let struct_name = ast.ident;
    let table_name = parse_table_name(&ast.attrs);
    let struct_name = ast.ident;
    let struct_name_str = struct_name.to_string();
    // dbg!(table_name);

    println!("attrs: {:?}", ast.attrs);
    println!("data: {:?}", ast.data);

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

    let builder_insert_rows = fields.iter().map(|f| {
        let field = f.clone().ident.unwrap();
        return quote! {
            row.push(item.#field.into_sql())
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
                                dbg!(&named_v);
                                let Path { ref segments, .. } = &named_v.path;
                                dbg!(&segments);
                                for rusql_segs in segments.iter() {
                                    if rusql_segs.ident == "foreign_key" {
                                        let b = &named_v.value;
                                        if let Expr::Lit(ExprLit { lit, .. }) = &named_v.value {
                                            if let Lit::Str(v) = lit {
                                                relations.push(format!("{}.{} = {}", &table_name, field_name, v.value()));
                                                tables.push(v.value()[..v.value().find('.').unwrap()].to_string());
                                                dbg!(&v.value());
                                            }
                                        }
                                        dbg!(&named_v.value);
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

            fn from_row(row:&Row) -> Map<String, Value> {
                let mut map = Map::new();
                #(#builder_row_func;)*
                map
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
                    _ =>  unimplemented!("relationship not found"),
                }
            }

            pub fn query() -> QueryBuilder {
                QueryBuilder::new(#table_name,
                    (#table_name, #struct_name::fields()),
                    #struct_name::relationship,
                    Box::new(#struct_name::from_row))
            }


        }
    });


    // let e = d.attrs[0].meta.clone();
    // println!("{:?}", e);


    // let mut b=None;
    // d.attrs[0].tokens.clone().into_iter().for_each(|item| {
    //     println!("{:?}", item);
    //     b = match item {
    //         Group(group) => Some(group),
    //         _ => unimplemented!()
    //     }
    // });
    //
    // let z = b.unwrap().stream().into_iter().for_each(|f| {
    //     match f {
    //         Ident{ident, span} =>
    //     }
    // });
    //
    // println!("d{:?}", b.unwrap());

    // println!("b: {:?}", b.iter);

    // item
    result.into()
}

fn parse_table_name(attrs: &Vec<Attribute>) -> String {
    for attr in attrs.iter() {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "rusql" {
                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                    for meta in list.iter() {
                        if let Meta::NameValue(named_v) = meta {
                            dbg!(&named_v);
                            let Path { ref segments, .. } = &named_v.path;
                            dbg!(&segments);
                            for rusql_segs in segments.iter() {
                                if rusql_segs.ident == "table" {
                                    let b = &named_v.value;
                                    if let Expr::Path(p_v) = &named_v.value {
                                        dbg!(&p_v);
                                        for seg in p_v.path.segments.iter() {
                                            let i = &seg.ident;
                                            return i.to_string();
                                        }
                                    }
                                    dbg!(&named_v.value);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    unimplemented!("no table name found")
}

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}