use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;

use crate::utils::{
    extract_type_from_option,
    get_relations_and_tables_and_pk,
    parse_table_name,
};

mod utils;


#[proc_macro_derive(ORM, attributes(ssql))]
pub fn ssql(tokens: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    let table_name = parse_table_name(&ast.attrs);
    let struct_name = ast.ident;


    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!()
    };

    let (relations, tables, primary_key) = get_relations_and_tables_and_pk(&table_name, fields);

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

    // for insert one
    let builder_insert_fields = fields.iter()
        .map(|f| { f.clone().ident.unwrap().to_string() })
        .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    let mut fields_count = 0;
    let builder_insert_params = fields.iter()
        .map(|_| {
            fields_count += 1;
            return format!("@p{}", fields_count);
        })
        .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    let builder_insert_data = fields.iter().map(|f|
        f.clone().ident.unwrap()
    )
        // .filter(|x| { *x.to_string() != "id".to_string() })
        .map(|f| quote! {&self.#f});

    // for insert one without primary key
    let builder_insert_fields_ignore_pk = fields.iter()
        .filter(|f| Some(*f) != primary_key.as_ref())
        .map(|f| { f.clone().ident.unwrap().to_string() })
        .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    let mut fields_count = 0;
    let builder_insert_params_ignore_pk = fields.iter()
        .filter(|f| Some(*f) != primary_key.as_ref())
        .map(|_| {
            fields_count += 1;
            return format!("@p{}", fields_count);
        })
        .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    let builder_insert_data_ignore_pk = fields.iter()
        .filter(|f| Some(*f) != primary_key.as_ref())
        .map(|f|f.clone().ident.unwrap())
        .map(|f| quote! {&self.#f});

    // for update one
    fields_count = 0;
    let builder_update_fields = fields.iter()
        .filter(|f| Some(*f) != primary_key.as_ref())
        .map(|f| {
            fields_count += 1;
            return format!(" {} = @p{}", f.clone().ident.unwrap().to_string(), fields_count);
        })
        .reduce(|cur: String, next: String| format!("{},{}", cur, &next)).unwrap();
    let builder_update_data = builder_insert_data_ignore_pk.clone();


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

    // for getting vectors of self struct
    let builder_row_to_self_func = fields.iter().map(|f| {
        let mn = f.clone().ident.unwrap();
        let field_name = match table_name.as_str() {
            "" => format!("{}", &mn),
            _ => format!("{}.{}", &table_name, &mn)
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

    let mut result = quote! {
    };


    let builder_fields = relations.iter().zip(tables.iter()).map(|(rel, tb)| {
        quote! { #tb => {
            concat!(" ", #tb, " ON ", #rel)
            // format!("JOIN {} ON {}", #tb, #rel)
        }}
    });

    let pk = if let Some(f) = &primary_key {
        let field_name = f.ident.as_ref().unwrap().to_string();
        let mn = f.ident.as_ref().unwrap();
        quote! {
            impl #struct_name {
                fn primary_key(&self) -> (&'static str, &dyn ToSql) {
                    (#field_name, &self.#mn)
                }
            }
        }
    } else {
        quote! {
            impl #struct_name {
                fn primary_key(&self) -> (&'static str, &dyn ToSql) {
                    unimplemented!("Primary key not set");
                }
            }
        }
    };
    result.extend(pk);

    result.extend(quote! {
        #[async_trait]
        impl SsqlMarker for #struct_name {
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

            fn row_to_struct(row:&Row) -> Self {
                Self{
                    #(#builder_row_to_self_func,)*
                }
            }

            fn query<'a>() -> QueryBuilder<'a, #struct_name> {
                QueryBuilder::<#struct_name>::new(
                    (#table_name, #struct_name::fields()),
                    #struct_name::relationship)
            }

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

            async fn insert(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields, #builder_insert_params);
                conn.execute(sql, &[#(#builder_insert_data,)*]).await?;
                Ok(())
            }

            async fn insert_ignore_pk(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let sql = format!("INSERT INTO {} ({}) values({})", #table_name, #builder_insert_fields_ignore_pk, #builder_insert_params_ignore_pk);
                conn.execute(sql, &[#(#builder_insert_data_ignore_pk,)*]).await?;
                Ok(())
            }

            async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let (pk, dt) = self.primary_key();
                conn.execute(format!("DELETE FROM {} WHERE {} = @p1", #table_name, pk), &[dt]).await?;
                Ok(())
            }

            async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()> {
                let (pk, dt) = self.primary_key();
                let sql = format!("UPDATE {} SET {} WHERE {} =@p{}", #table_name, #builder_update_fields, pk, #fields_count + 1);
                conn.execute(sql, &[#(#builder_update_data,)* dt]).await?;
                Ok(())
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
                    _ =>  unimplemented!("column_type not found"),
                }
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
    });

    result.into()
}

