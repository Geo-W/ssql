macro_rules! impl_cmp {
    ($($func_name:ident, $cmp_enum: ident),*) => {
        $(
            pub fn $func_name(
                self,
                other: &dyn ToSql
            ) -> FilterExpr
            {
                self.expr_wrapper(ConditionVar::$cmp_enum(other))
            }
        )*
    };
}

//
//
// pub fn eq(self, other: &dyn ToSql) -> FilterExpr {
//     self.expr_wrapper(other, ConditionVar::Eq)
// }