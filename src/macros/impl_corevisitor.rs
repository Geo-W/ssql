macro_rules! impl_corevisitor {
    ($this_model:ident, [$($Tables: ident),*]) => {
        #[async_trait]
        impl<'a, $($Tables),*> CoreVisitor<'a> for $this_model<'a, $($Tables),*>
        where
            $($Tables: SsqlMarker, )*
        {
            fn core_mut(&mut self) -> &mut QueryCore<'a> {
                &mut self.core
            }

            fn core_ref(&self) -> &QueryCore<'a> {
                &self.core
            }
        }
    };
}

