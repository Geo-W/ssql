macro_rules! impl_queryable {
    ($this_model:ident, $nxt_model: ident, [$($Table: ident), *], [$($Ret: ty), *], [$new_field: ident, $($field: ident),*], [$($func: ident), *]) => {
        #[async_trait]
        impl<'a, FN, Ret, $($Table),*> QueryAble<'a> for $this_model<'a, FN, Ret, $($Table),*>
        where
            $($Table: SsqlMarker + Send + Sync, )*
            FN: Fn(&Row) -> Ret + 'static + Send + Sync,
            Ret: Send + Sync,
        {
            type NxtModel<NxtType: SsqlMarker> = $nxt_model<'a, FN, Ret, $($Table),*, NxtType>;
            type NewFnModel<NewFN, NewRet> = $this_model<'a, NewFN, NewRet, $($Table),*>
            where
                NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;
            type Ret = SsqlResult<Vec<($($Ret),*)>>;

            fn join<NxtType>(self) -> Self::NxtModel<NxtType>
            where
                NxtType: SsqlMarker,
            {
                Self::NxtModel {
                    a: self.a.left_join::<NxtType>(),
                    $($field: PhantomData,)*
                    $new_field: PhantomData,
                    func: self.func,
                }
            }

            async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
                self.a
                    .exec(conn, |x| ($((&self.$func)(x)),*))
                    .await
            }

            fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
            where
                Self: Sized,
            {
                self.a = self.a.filter(filter_expr)?;
                Ok(self)
            }

            fn replace_fn<NewFN, NewRet>(self, new_fn: NewFN) -> Self::NewFnModel<NewFN, NewRet>
            where
                NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync,
            {
                Self::NewFnModel{
                    a: self.a,
                    $($field: self.$field,)*
                    func: new_fn
                }
            }
        }
    };
}