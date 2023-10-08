macro_rules! impl_get_data {
    ($func_name:ident, $process_row: ident, [$($T:ident, $R: ident, $R_Ty: ty),*]) => {
        #[allow(unused_parens)]
        pub async fn $func_name<$($T),*>(
            &mut self,
            conn: &mut tiberius::Client<Compat<TcpStream>>,
        ) -> SsqlResult<($(Vec<$R_Ty>),*)>
        where
            $($T: SsqlMarker),*
        {
            let mut stream = self.execute(conn).await?;
            $(let mut $R: Vec<$R_Ty> = vec![];)*

            while let Some(item) = stream.try_next().await.unwrap() {
                match item {
                    QueryItem::Row(row) => {
                        $($R.push($T::$process_row(&row).into());)*
                    }
                    QueryItem::Metadata(_) => {}
                }
            }

            Ok(($($R),*))
        }
    };
}