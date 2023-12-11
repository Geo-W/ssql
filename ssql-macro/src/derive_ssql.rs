use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, FieldsNamed, Ident};
use syn::Data::Struct;
use syn::Fields::Named;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use crate::utils::{extract_type_from_option, get_relations_and_tables_and_pk, parse_table_name};

pub struct DeriveSsql {
    table_name: String,
    struct_ident: Ident,
    fields: Punctuated<Field, Comma>,
    relations: Vec<String>,
    tables: Vec<String>,
    primary_key: Option<Field>,

    impl_fns: TokenStream,
}

impl DeriveSsql {
    pub(crate) fn new(ast: syn::DeriveInput) -> Self {
        let table_name = parse_table_name(&ast.attrs);
        let fields = match ast.data {
            Struct(DataStruct {
                fields: Named(FieldsNamed { named, .. }),
                ..
            }) => named,
            _ => unimplemented!(),
        };
        let (relations, tables, primary_key) =
            get_relations_and_tables_and_pk(&table_name, &fields);

        Self {
            table_name,
            struct_ident: ast.ident,
            fields,
            relations,
            tables,
            primary_key,

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

    pub(crate) fn impl_row_to_struct(&mut self) {
        let Self {
            fields, table_name, ..
        } = &self;
        let builder_row_to_self_func = fields.iter().map(|f| {
            let mn = f.clone().ident.unwrap();
            let field_name = match table_name.as_str() {
                "" => format!("{}", &mn),
                _ => format!("{}.{}", &table_name, &mn),
            };
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
                }
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
        self.impl_fns.extend(quote! {
            fn row_to_struct(row:&Row) -> Self {
                Self{
                    #(#builder_row_to_self_func,)*
                }
            }
        });
    }

    pub(crate) fn impl_row_to_json(&mut self) {
        let Self {
            fields, table_name, ..
        } = &self;
        let builder_row_func = fields.iter().map(|f| {
            let mn = f.clone().ident.unwrap().to_string();
            let field_name = match table_name.as_str() {
                "" => format!("{}", &mn),
                _ => format!("{}.{}", &table_name, &mn)
            };
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
                        map.insert(#mn.to_string(), row.get::<#ty, &str>(#field_name).and_then(|x| x.to_string().into()).into())
                    }
                }
                _ => {
                    quote! {
                        map.insert(#mn.to_string(), row.get::<#ty, &str>(#field_name).into())
                    }
                }
            };
        });
        self.impl_fns.extend(quote!{

            fn row_to_json(row:&Row) -> Map<String, Value> {
                let mut map = Map::new();
                #(#builder_row_func;)*
                map
            }

        })
    }

    pub(crate) fn impl_query(&mut self) {
        let Self {
            table_name,
            struct_ident,
            ..
        } = self;
        self.impl_fns.extend(quote!{

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
        self.impl_fns.extend(quote!{

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
            .map(|f| f.clone().ident.unwrap())
            .map(|f| quote! {&self.#f});
        self.impl_fns.extend(quote!{

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
        self.impl_fns.extend(quote!{

            async fn insert_ignore_pk(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields_ignore_pk, #builder_insert_params_ignore_pk);
                conn.execute(sql, &[#(#builder_insert_data_ignore_pk,)*]).await?;
                Ok(())
            }

        })
    }

    pub(crate) fn impl_delete(&mut self) {
        let table_name = &self.table_name;
        self.impl_fns.extend(quote!{
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
        self.impl_fns.extend(quote!{

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
        self.impl_fns.extend(quote!{

            fn relationship(input: &str) -> &'static str {
                match input {
                    #(#builder_fields,)*
                    _ =>  unimplemented!("relationship not found"),
                }
            }

        })
    }



    pub(crate) fn finalize(self) -> proc_macro::TokenStream {
        let struct_name = self.struct_ident;
        let fns = self.impl_fns;
        quote!{
            #[async_trait]
            impl SsqlMarker for #struct_name {
                #fns
            }
        }.into()
    }
}