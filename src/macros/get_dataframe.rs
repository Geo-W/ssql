macro_rules! impl_get_dataframe {
    ($func_name:ident, $get_struct_func: ident, [$($T:ident, $R: ident, $R_Ty: ty),*]) => {
        #[allow(unused_parens)]
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
