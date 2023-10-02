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
        self.expr_wrapper(ConditionVar::Eq(other))
    }

    pub fn neq(self, other: &dyn ToSql) -> FilterExpr {
        self.expr_wrapper(ConditionVar::Neq(other))
    }

    crate::impl_cmp! {lt, Lt, lt_eq, LtEq, gt, Gt, gt_eq, GtEq}

    pub fn is_null<'b>(self) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::IsNull)
    }

    pub fn is_not_null<'b>(self) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::IsNotNull)
    }

    pub fn contains<'b>(self, other: impl ToString) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::Contains(other.to_string()))
    }

    pub fn is_in(self, ls: &[impl ToSql]) -> FilterExpr {
        let v = ls.iter()
            .map(|x| x as &dyn ToSql)
            .collect();
        self.expr_wrapper(ConditionVar::IsIn(v))
    }

    pub fn is_in_ref<'b>(self, ls: &[&'b dyn ToSql]) -> FilterExpr<'b> {
        let v = ls.to_vec();
        self.expr_wrapper(ConditionVar::IsIn(v))
    }

    fn expr_wrapper(self, con: ConditionVar) -> FilterExpr {
        FilterExpr {
            col: self,
            con,
        }
    }

    pub fn full_column_name(&self) -> String {
        format!("{}.{}", self.table, self.field)
    }
}

pub struct FilterExpr<'b> {
    pub col: Column,
    // pub conditions: &'b dyn ToSql,
    pub con: ConditionVar<'b>,
}

impl<'b> FilterExpr<'b> {
    pub(crate) fn to_sql(&self, idx: &mut i32, query_params: &mut Vec<&'b dyn ToSql>) -> String {
        match &self.con {
            ConditionVar::Eq(v) | ConditionVar::Neq(v) | ConditionVar::Gt(v) | ConditionVar::GtEq(v) | ConditionVar::Lt(v) | ConditionVar::LtEq(v) => {
                query_params.push(*v);
                *idx += 1;
                format!(" {} {} @p{}", self.col.full_column_name(), self.con.to_sql_symbol(), idx)
            }
            ConditionVar::IsNull | ConditionVar::IsNotNull => {
                format!("{} {}", self.col.full_column_name(), self.con.to_sql_symbol())
            }
            ConditionVar::Contains(v) => {
                format!("{} LIKE '%{}%' ", self.col.full_column_name(), v)
            }
            ConditionVar::IsIn(v) => {
                let mut i = *idx;
                *idx += v.len() as i32;
                let cond_params = v.iter()
                    .map(|_| {
                    i += 1;
                    format!("@p{}", i)
                })
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt))
                    .unwrap();
                query_params.extend(v);
                format!("{} IN ({})", self.col.full_column_name(), cond_params)
            }
        }
    }
}

pub enum ConditionVar<'a> {
    Eq(&'a dyn ToSql),
    Neq(&'a dyn ToSql),
    Gt(&'a dyn ToSql),
    GtEq(&'a dyn ToSql),
    Lt(&'a dyn ToSql),
    LtEq(&'a dyn ToSql),
    IsNull,
    IsNotNull,
    IsIn(Vec<&'a dyn ToSql>),
    Contains(String),
}

impl<'a> ConditionVar<'a> {
    fn to_sql_symbol(&self) -> &'static str {
        match self {
            ConditionVar::Eq(_) => "=",
            ConditionVar::Neq(_) => "<>",
            ConditionVar::Gt(_) => ">",
            ConditionVar::GtEq(_) => ">=",
            ConditionVar::Lt(_) => "<",
            ConditionVar::LtEq(_) => "<=",
            ConditionVar::IsNull => "is null",
            ConditionVar::IsNotNull => "is not null",
            // ConditionVar::IsIn => "",
            ConditionVar::Contains(_) => "",
            ConditionVar::IsIn(_) => ""
        }
    }
}