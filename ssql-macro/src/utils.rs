use syn::{punctuated::Punctuated, token::Comma, Meta, Path};
use syn::{Expr, ExprLit, Field, Lit};

pub(crate) fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, PathArguments, PathSegment};

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

pub(crate) fn parse_table_name(attrs: &Vec<syn::Attribute>) -> String {
    let mut table: (String, String) = ("".to_string(), "".to_string());
    for attr in attrs.iter() {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "ssql" {
                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list.iter() {
                        if let Meta::NameValue(named_v) = meta {
                            let Path { ref segments, .. } = &named_v.path;
                            for ssql_segs in segments.iter() {
                                if ssql_segs.ident == "table" {
                                    // let b = &named_v.value;
                                    if let Expr::Path(p_v) = &named_v.value {
                                        for seg in p_v.path.segments.iter() {
                                            let i = &seg.ident;
                                            table.0 = i.to_string();
                                            // return i.to_string();
                                        }
                                    }
                                } else if ssql_segs.ident == "schema" {
                                    if let Expr::Path(p_v) = &named_v.value {
                                        for seg in p_v.path.segments.iter() {
                                            let i = &seg.ident;
                                            table.1 = i.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // match table_name, schema
    match (table.0.as_str(), table.1.as_str()) {
        // ("", _) => unimplemented!(),
        (_, "") => table.0,
        _ => format!("{}.{}", table.1, table.0),
    }
}

pub(crate) fn get_relations_and_tables_and_pk(
    table_name: &String,
    fields: &Punctuated<Field, Comma>,
) -> (Vec<String>, Vec<String>, Option<Field>) {
    let mut relations: Vec<String> = vec![];
    let mut tables: Vec<String> = vec![];
    let mut primary_key = None;
    for field in fields.iter() {
        for attr in field.attrs.iter() {
            if let Some(ident) = attr.path().get_ident() {
                if ident == "ssql" {
                    if let Ok(list) =
                        attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        for meta in list.iter() {
                            if let Meta::Path(path) = meta {
                                let Path { ref segments, .. } = path;
                                for ssql_tags in segments.iter() {
                                    if ssql_tags.ident == "primary_key" {
                                        primary_key = Some(field.clone());
                                    }
                                }
                            }

                            if let Meta::NameValue(named_v) = meta {
                                let Path { ref segments, .. } = &named_v.path;
                                for ssql_tags in segments.iter() {
                                    if ssql_tags.ident == "foreign_key" {
                                        if let Expr::Lit(ExprLit { lit, .. }) = &named_v.value {
                                            if let Lit::Str(v) = lit {
                                                let field_name =
                                                    field.ident.as_ref().unwrap().to_string();
                                                relations.push(format!(
                                                    "{}.{} = {}",
                                                    &table_name,
                                                    field_name,
                                                    v.value()
                                                ));
                                                tables.push(
                                                    v.value()[..v.value().rfind('.').unwrap()]
                                                        .to_string(),
                                                );
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
    (relations, tables, primary_key)
}
