use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::Fields::Named;
use syn::{DataStruct, DeriveInput, Field, FieldsNamed, Ident, parse_quote, Type};

use crate::utils::{extract_type_from_option, get_relations_and_tables_and_pk, parse_table_name};

pub struct DeriveSsql<'a> {
    table_name: String,
    struct_ident: &'a Ident,
    fields: &'a Punctuated<Field, Comma>,
    relations: Vec<String>,
    tables: Vec<String>,
    primary_key: Option<Field>,

    fields_type: Vec<FieldType<'a>>,

    impl_fns: TokenStream,
}

struct FieldType<'a> {
    ident: &'a Ident,
    // used for query
    query_name: String,
    //type for query in row.get::<?,?>()
    query_type: Type,
    #[cfg(feature = "polars")]
    origin_type: Type,
    unwrap_quote: TokenStream,
    to_string_quote: TokenStream,
}

impl<'a> DeriveSsql<'a> {
    pub(crate) fn new(ast: &'a DeriveInput) -> Self {
        let table_name = parse_table_name(&ast.attrs);
        let fields = match &ast.data {
            Struct(DataStruct {
                       fields: Named(FieldsNamed { ref named, .. }),
                       ..
                   }) => named,
            _ => unimplemented!(),
        };
        let (relations, tables, primary_key) =
            get_relations_and_tables_and_pk(&table_name, &fields);

        // let str: Type = parse_quote!(String);

        let fields_type = fields.iter().map(|x| {
            let mut ty = &x.ty;
            let unwrap_quote = match extract_type_from_option(&x.ty) {
                None => {
                    quote! {
                        .unwrap()
                    }
                }
                Some(v) => {
                    ty = v;
                    quote! {}
                }
            };
            let (new_ty, to_string_quote) = if ty.to_token_stream().to_string().as_str() == "String" {
                (parse_quote!(&str), quote! {.map(|i| i.to_string())})
            } else {
                (ty.clone(), quote! {})
            };

            let ident = x.ident.as_ref().unwrap();
            let query_name = match table_name.is_empty() {
                true => {
                    format!("{}", &ident)
                }
                false => {
                    format!("{}.{}", &table_name, &ident)
                }
            };

            FieldType {
                ident,
                query_name,
                query_type: new_ty,
                #[cfg(feature = "polars")]
                origin_type: ty.clone(),
                unwrap_quote: unwrap_quote,
                to_string_quote: to_string_quote,
            }
        }).collect();
        Self {
            table_name,
            struct_ident: &ast.ident,
            fields,
            relations,
            tables,
            primary_key,

            fields_type: fields_type,
            impl_fns: Default::default(),
        }
    }

    pub(crate) fn impl_table_name(&mut self) {
        let table_name = &self.table_name;
        self.impl_fns.extend(quote! {

            fn table_name() -> &'static str {
                #table_name
            }

        });
    }

    pub(crate) fn impl_fields(&mut self) {
        let builder_fields_mapping = self
            .fields
            .iter()
            .map(|f| f.clone().ident.unwrap().to_string());
        self.impl_fns.extend(quote! {

            fn fields() -> Vec<&'static str> {
                vec![#(#builder_fields_mapping,)*]
            }

        });
    }

    pub(crate) fn impl_query(&mut self) {
        let Self {
            table_name,
            struct_ident,
            ..
        } = self;
        self.impl_fns.extend(quote! {

            fn query<'a>() -> ssql::QueryBuilderI<'a, Self> {
                QueryBuilderI::new(
                    (#table_name, #struct_ident::fields()),
                    #struct_ident::relationship)
            }

        })
    }

    pub(crate) fn impl_insert_many(&mut self) {
        let Self {
            fields, table_name, ..
        } = self;
        let builder_insert_rows = fields.iter().map(|f| {
            let field = f.clone().ident.unwrap();
            return quote! {
                row.push(item.#field.into_sql())
            };
        });
        self.impl_fns.extend(quote! {

            async fn insert_many<I: IntoIterator<Item=Self> + Send>(iter: I, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<u64>
                where I::IntoIter: Send
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

        })
    }

    pub(crate) fn impl_insert(&mut self) {
        let Self {
            fields, table_name, ..
        } = self;
        let builder_insert_fields = fields
            .iter()
            .map(|f| f.clone().ident.unwrap().to_string())
            .reduce(|cur: String, next: String| format!("{},{}", cur, &next))
            .unwrap();
        let mut fields_count = 0;
        let builder_insert_params = fields
            .iter()
            .map(|_| {
                fields_count += 1;
                return format!("@p{}", fields_count);
            })
            .reduce(|cur: String, next: String| format!("{},{}", cur, &next))
            .unwrap();
        let builder_insert_data = fields
            .iter()
            .map(|f| f.ident.as_ref().unwrap())
            .map(|f| quote! {&self.#f});
        self.impl_fns.extend(quote! {

             async fn insert(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields, #builder_insert_params);
                conn.execute(sql, &[#(#builder_insert_data,)*]).await?;
                Ok(())
            }

        })
    }

    pub(crate) fn impl_insert_ignore_pk(&mut self) {
        let Self {
            table_name,
            fields,
            primary_key,
            ..
        } = self;
        let builder_insert_fields_ignore_pk = fields
            .iter()
            .filter(|f| Some(*f) != primary_key.as_ref())
            .map(|f| f.clone().ident.unwrap().to_string())
            .reduce(|cur: String, next: String| format!("{},{}", cur, &next))
            .unwrap();
        let mut fields_count = 0;
        let builder_insert_params_ignore_pk = fields
            .iter()
            .filter(|f| Some(*f) != primary_key.as_ref())
            .map(|_| {
                fields_count += 1;
                return format!("@p{}", fields_count);
            })
            .reduce(|cur: String, next: String| format!("{},{}", cur, &next))
            .unwrap();
        let builder_insert_data_ignore_pk = fields
            .iter()
            .filter(|f| Some(*f) != primary_key.as_ref())
            .map(|f| f.clone().ident.unwrap())
            .map(|f| quote! {&self.#f});
        self.impl_fns.extend(quote! {

            async fn insert_ignore_pk(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields_ignore_pk, #builder_insert_params_ignore_pk);
                conn.execute(sql, &[#(#builder_insert_data_ignore_pk,)*]).await?;
                Ok(())
            }

        })
    }

    pub(crate) fn impl_delete(&mut self) {
        let table_name = &self.table_name;
        self.impl_fns.extend(quote! {
            async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let (pk, dt) = self.primary_key();
                conn.execute(
                    format!("DELETE FROM {} WHERE {} = @p1", #table_name, pk),
                    &[dt],
                )
                .await?;
                Ok(())
            }
        })
    }

    pub(crate) fn impl_update(&mut self) {
        let Self {
            fields,
            primary_key,
            table_name,
            ..
        } = self;
        let mut fields_count = 0;
        let builder_update_fields = fields
            .iter()
            .filter(|f| Some(*f) != primary_key.as_ref())
            .map(|f| {
                fields_count += 1;
                return format!(
                    " {} = @p{}",
                    f.clone().ident.unwrap().to_string(),
                    fields_count
                );
            })
            .reduce(|cur: String, next: String| format!("{},{}", cur, &next))
            .unwrap();
        let builder_insert_data_ignore_pk = fields
            .iter()
            .filter(|f| Some(*f) != primary_key.as_ref())
            .map(|f| f.clone().ident.unwrap())
            .map(|f| quote! {&self.#f});
        let builder_update_data = builder_insert_data_ignore_pk.clone();
        self.impl_fns.extend(quote! {

            async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let (pk, dt) = self.primary_key();
                let sql = format!("UPDATE {} SET {} WHERE {} =@p{}", #table_name, #builder_update_fields, pk, #fields_count + 1);
                conn.execute(sql, &[#(#builder_update_data,)* dt]).await?;
                Ok(())
            }

        })
    }

    pub(crate) fn impl_primary_key(&mut self) {
        let primary_key = &self.primary_key;
        let pk = if let Some(f) = &primary_key {
            let field_name = f.ident.as_ref().unwrap().to_string();
            let mn = f.ident.as_ref().unwrap();
            quote! {
                fn primary_key(&self) -> (&'static str, &dyn ToSql) {
                    (#field_name, &self.#mn)
                }
            }
        } else {
            quote! {
                fn primary_key(&self) -> (&'static str, &dyn ToSql) {
                    unimplemented!("Primary key not set");
                }
            }
        };
        self.impl_fns.extend(pk);
    }

    pub(crate) fn impl_relationship(&mut self) {
        let builder_fields = self
            .relations
            .iter()
            .zip(self.tables.iter())
            .map(|(rel, tb)| {
                quote! { #tb => {
                    concat!(" ", #tb, " ON ", #rel)
                }}
            });
        self.impl_fns.extend(quote! {

            fn relationship(input: &str) -> &'static str {
                match input {
                    #(#builder_fields,)*
                    _ =>  unimplemented!("relationship not found"),
                }
            }

        })
    }

    pub(crate) fn impl_row_to_struct(&mut self) {
        let fields_type = &self.fields_type;
        let builder_row_to_self_func = fields_type.iter().map(|f| {
            let ident = &f.ident;
            let ty = &f.query_type;
            let unwrap = &f.unwrap_quote;
            let to_string = &f.to_string_quote;
            let query_name = &f.query_name;


            return quote! {
                #ident: row.get::<#ty, &str>(#query_name)#to_string #unwrap
            };
        });
        self.impl_fns.extend(quote! {
            fn row_to_struct(row:&Row) -> Self {
                Self{
                    #(#builder_row_to_self_func,)*
                }
            }
        });
    }

    #[cfg(feature = "serde")]
    pub(crate) fn impl_row_to_json(&mut self) {
        let fields_type = &self.fields_type;
        let builder_row_func = fields_type.iter().map(|f| {
            let ident_str = f.ident.to_string();
            let query_name = &f.query_name;
            let ty = &f.query_type;
            return quote! {
                map.insert(#ident_str.to_string(), row.get::<#ty, &str>(#query_name).serialize(Serializer).unwrap())
            };
        });
        self.impl_fns.extend(quote! {

            fn row_to_json(row:&Row) -> Map<String, Value> {
                let mut map = Map::new();
                #(#builder_row_func;)*
                map
            }

        })
    }

    #[cfg(feature = "polars")]
    pub(crate) fn impl_dataframe(&mut self) {
        let fields_type = &self.fields_type;
        let builder_new_vecs = fields_type.iter().map(|f| {
            let ident = f.ident;
            let ty = &f.origin_type;
            quote! {
                let mut #ident : Vec<Option<#ty>> = vec![]
            }
        });

        let builder_insert_to_df = fields_type.iter().map(|f| {
            let field = f.ident;
            let ty = &f.query_type;
            let query_name = &f.query_name;
            let to_string = &f.to_string_quote;
            quote! {
                #field.push(row.get::<#ty, &str>(#query_name)#to_string)
            }
        });

        let builder_df = fields_type.iter().map(|f| {
            let field = f.ident;
            let mn = field.to_string();
            quote! {
                #mn => #field
            }
        });

        self.impl_fns.extend(quote! {

            async fn dataframe<'a>(stream: QueryStream<'a>) -> SsqlResult<DataFrame> {
                #(#builder_new_vecs;)*
                #[allow(non_snake_case)]
                let mut stream = stream.into_row_stream();
                while let Some(row) = stream.try_next().await? {
                    #(#builder_insert_to_df;)*
                }
                Ok(
                    df!(
                    #(#builder_df,)*
                )?
                )
            }

        });
    }

    pub(crate) fn finalize(self) -> proc_macro::TokenStream {
        let struct_name = self.struct_ident;
        let fns = self.impl_fns;
        quote! {
            impl SsqlMarker for #struct_name {
                #fns
            }
        }
        .into()
    }
}
