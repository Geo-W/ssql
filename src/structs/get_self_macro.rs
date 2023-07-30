#[macro_export]
macro_rules! impl_get_self {
    ($func_name:ident, [$($T:ident, $R: ident),*]) => {
        #[allow(unused_parens)]
        pub async fn $func_name<$($T),*>(
            &mut self,
            conn: &mut tiberius::Client<Compat<TcpStream>>,
        ) -> RssqlResult<($(Vec<$T>),*)>
        where
            $($T: RusqlMarker),*
        {
            let mut stream = self.execute(conn).await?;
            $(let mut $R: Vec<$T> = vec![];)*

            while let Some(item) = stream.try_next().await.unwrap() {
                match item {
                    QueryItem::Row(row) => {
                        $($R.push($T::row_to_self(&row));)*
                    }
                    QueryItem::Metadata(_) => {}
                }
            }

            Ok(($($R),*))
        }
    };
}


    // pub async fn get_self<A>(&mut self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> Result<(Vec<A>), RssqlError>
    // where A: RusqlMarker
    // {
    //     let mut stream = self.execute(conn).await?;
    //     let mut ret1: Vec<A> = vec![];
    //     while let Some(item) = stream.try_next().await.unwrap() {
    //         match item {
    //             QueryItem::Row(row) => {
    //                 ret1.push(
    //                     A::row_to_self(&row)
    //                 );
    //             }
    //             QueryItem::Metadata(_) => {}
    //         }
    //     }
    //     Ok((ret1))
    // }


    // pub async fn get_self_2<A, B>(&mut self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> Result<(Vec<A>, Vec<B>), RssqlError>
    //     where A: RusqlMarker,
    //           B: RusqlMarker
    // {
    //     let mut stream = self.execute(conn).await?;
    //     let mut ret1: Vec<A> = vec![];
    //     let mut ret2: Vec<B> = vec![];
    //     while let Some(item) = stream.try_next().await.unwrap() {
    //         match item {
    //             QueryItem::Row(row) => {
    //                 ret1.push(
    //                     A::row_to_self(&row)
    //                 );
    //                 ret2.push(
    //                     B::row_to_self(&row)
    //                 )
    //             }
    //             QueryItem::Metadata(_) => {}
    //         }
    //     }
    //     Ok((ret1, ret2))
    // }
