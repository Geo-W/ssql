macro_rules! impl_get_dataframe {
    ($func_name:ident, $get_struct_func: ident, [$($T:ident, $R: ident, $R_Ty: ty),*]) => {
        #[allow(unused_parens)]
        #[doc="Getting data from query builder instance, \
        a vector containing tuples of DataFrame is returned, each represents a table struct in provided order. \
        Will panic if the given table struct is not main table or not joined."]
        #[cfg(feature = "polars")]
        pub async fn $func_name<$($T),*>(
            &mut self,
            conn: &mut tiberius::Client<Compat<TcpStream>>,
        ) -> SsqlResult<($($R_Ty),*)>
        where
            $($T: SsqlMarker + PolarsHelper),*
        {
            let ($($R),*) = self.$get_struct_func::<$($T),*>(conn).await?;
            Ok(($($T::dataframe($R)?),*))
        }
    };
}
