use tiberius::ToSql;

// trait FilterHelper {
//     fn eq<'a>(&'static self, rhs: &'a dyn ToSql) -> Filter<'a>;
// }
//
//
// impl FilterHelper for &'static str {
//     fn eq<'a>(&'static self, rhs: &'a dyn ToSql) -> Filter<'a> {
//         Filter{
//             fields: vec![self.as_ref()],
//             conditions: vec![rhs],
//         }
//     }
// }


pub struct Column {
    pub table: &'static str,
    pub field: &'static str,
}

impl Column {
    pub fn eq(self, other: &dyn ToSql) -> FilterExpr {
        self.expr_wrapper(other, ConditionVar::Eq)
    }

    pub fn neq(self, other: &dyn ToSql) -> FilterExpr {
        self.expr_wrapper(other, ConditionVar::Neq)
    }

    fn expr_wrapper(self, other: &dyn ToSql, con: ConditionVar) -> FilterExpr {
        FilterExpr {
            col: self,
            conditions: other,
            con,
        }
    }

    pub fn full_column_name(&self) -> String {
        format!("{}.{}", self.table, self.field)
    }
}

pub struct FilterExpr<'b> {
    pub col: Column,
    pub conditions: &'b dyn ToSql,
    pub con: ConditionVar,
}

impl<'b> FilterExpr<'b> {
    pub(crate) fn to_sql(&self, idx: &mut i32) -> String {
        match self.con {
            ConditionVar::Eq => {
                self.generic_to_sql(idx)
            }
            ConditionVar::Neq => {
                self.generic_to_sql(idx)
            }
        }
    }

    fn generic_to_sql(&self, idx: &mut i32) -> String {
        *idx += 1;
        format!(" {} {} @p{}", self.col.full_column_name(), self.con.to_sql_symbol(), idx)
    }
}

pub enum ConditionVar {
    Eq,
    Neq,
}

impl ConditionVar {
    fn to_sql_symbol(&self) -> &'static str {
        match self {
            ConditionVar::Eq => "=",
            ConditionVar::Neq => "<>"
        }
    }
}