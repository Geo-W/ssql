macro_rules! impl_queryable {
    ($this_model:ident, $nxt_model: ident, [$($Tables: ident),*],
        [$($field: ident),*]) => {
        #[async_trait]
        impl<'a, $($Tables),*> QueryAble<'a> for $this_model<'a, $($Tables),*>
        where
            $($Tables: SsqlMarker + Send + Sync + 'static, )*
        {
            type NxtModel<NxtType: SsqlMarker> = $nxt_model<'a, $($Tables),*, NxtType>;
            type Ret = ($($Tables),*);

            fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
            where
                NxtType: SsqlMarker,
            {
                $nxt_model {
                    core: self.core.join::<NxtType>(join_args),
                    $($field: Default::default(),)*
                }
            }

        }
    };
}
