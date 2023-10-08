use tiberius::ToSql;

/// Column Expression
pub struct ColExpr {
    pub(crate) table: &'static str,
    pub(crate) field: &'static str,
}

impl ColExpr {
    /// generate filter expression checking whether this column equals to a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("id")?.eq(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id = 5`
    pub fn eq(self, other: &dyn ToSql) -> FilterExpr {
        self.expr_wrapper(ConditionVar::Eq(other))
    }

    /// generate filter expression checking whether this column not equals to a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("id")?.neq(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id <> 5`
    pub fn neq(self, other: &dyn ToSql) -> FilterExpr {
        self.expr_wrapper(ConditionVar::Neq(other))
    }

    /// generate filter expression checking whether this column is less than a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.lt(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id < 5`
    pub fn lt(self, other: &dyn ToSql) -> FilterExpr { self.expr_wrapper(ConditionVar::Lt(other)) }

    /// generate filter expression checking whether this column is less or equal than a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.lt_eq(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id <= 5`
    pub fn lt_eq(self, other: &dyn ToSql) -> FilterExpr { self.expr_wrapper(ConditionVar::LtEq(other)) }

    /// generate filter expression checking whether this column is greater than a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.gt(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id > 5`
    pub fn gt(self, other: &dyn ToSql) -> FilterExpr { self.expr_wrapper(ConditionVar::Gt(other)) }

    /// generate filter expression checking whether this column is greater or equal than a value.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.gt_eq(&5)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id >= 5`
    pub fn gt_eq(self, other: &dyn ToSql) -> FilterExpr { self.expr_wrapper(ConditionVar::GtEq(other)) }

    /// generate filter expression checking whether this column is null.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.is_null()
    /// )?;
    /// ```
    /// SQL: `**... WHERE person.email IS NULL**`
    pub fn is_null<'b>(self) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::IsNull)
    }

    /// generate filter expression checking whether this column is not null.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.is_not_null()
    /// )?;
    /// ```
    /// SQL: `... WHERE person.email IS NOT NULL`
    pub fn is_not_null<'b>(self) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::IsNotNull)
    }

    /// generate filter expression checking whether a char column contains a given str.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.contains("gmail")
    /// )?;
    /// ```
    /// SQL: `... WHERE person.email LIKE '%gmail%' `
    pub fn contains<'b>(self, other: &'b str) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::Contains(other))
    }

    /// generate filter expression checking whether a char column starts with a given str.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.startswith("john")
    /// )?;
    /// ```
    /// SQL: `... WHERE person.email LIKE 'john%' `
    pub fn startswith<'b>(self, other: &'b str) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::StarsWith(other))
    }

    /// generate filter expression checking whether a char column ends with a given str.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.endswith("gmail.com")
    /// )?;
    /// ```
    /// SQL: `... WHERE person.email LIKE '%gmail.com' `
    pub fn endswith<'b>(self, other: &'b str) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::EndsWith(other))
    }

    /// generate filter expression checking whether a value in a given list.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("id")?.is_in(&[3,4,5,6,7])
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id IN (3,4,5,6,7) `
    pub fn is_in(self, ls: &[impl ToSql]) -> FilterExpr {
        let v = ls.iter()
            .map(|x| x as &dyn ToSql)
            .collect();
        self.expr_wrapper(ConditionVar::IsIn(v))
    }

    /// generate filter expression checking whether a value in a given list.
    /// This method allows for providing different types of args.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.is_in_ref(&[&3, &"4", &5])
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id IN (3,'4',5) `
    pub fn is_in_ref<'b>(self, ls: &[&'b dyn ToSql]) -> FilterExpr<'b> {
        let v = ls.to_vec();
        self.expr_wrapper(ConditionVar::IsIn(v))
    }

    /// generate filter expression checking whether a value between a range.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// let query = Person::query().filter(
    ///     Person::col("email")?.between(&3, &7)
    /// )?;
    /// ```
    /// SQL: `... WHERE person.id BETWEEN 3 AND 7 `
    pub fn between<'b>(self, start: &'b dyn ToSql, end: &'b dyn ToSql) -> FilterExpr<'b> {
        self.expr_wrapper(ConditionVar::Between((start, end)))
    }

    fn expr_wrapper(self, con: ConditionVar) -> FilterExpr {
        FilterExpr {
            col: self,
            con,
            or_cons: vec![],
        }
    }

    pub(crate) fn full_column_name(&self) -> String {
        format!("{}.{}", self.table, self.field)
    }
}

pub struct FilterExpr<'b> {
    pub(crate) col: ColExpr,
    // pub conditions: &'b dyn ToSql,
    con: ConditionVar<'b>,
    or_cons: Vec<FilterExpr<'b>>,
}

impl<'b> FilterExpr<'b> {
    pub(crate) fn to_sql(&self, idx: &mut i32, query_params: &mut Vec<&'b dyn ToSql>) -> String {
        match self.or_cons.is_empty() {
            true => {
                self.to_sql_wrapper(idx, query_params)
            }
            false => {
                let tmp = self.or_cons.iter()
                    .chain([self])
                    .map(|x| x.to_sql_wrapper(idx, query_params))
                    .reduce(|cur, nxt| format!("{cur} OR {nxt}")).unwrap();
                format!("( {} )", tmp)
            }
        }
    }
    pub(crate) fn to_sql_wrapper(&self, idx: &mut i32, query_params: &mut Vec<&'b dyn ToSql>) -> String {
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
            ConditionVar::Between((v1, v2)) => {
                *idx += 2;
                query_params.push(*v1);
                query_params.push(*v2);
                format!("{} BETWEEN @p{} AND @p{}", self.col.full_column_name(), *idx - 1, idx)
            }
            ConditionVar::StarsWith(v) => {
                format!("{} LIKE '{}%' ", self.col.full_column_name(), v)
            }
            ConditionVar::EndsWith(v) => {
                format!("{} LIKE '%{}' ", self.col.full_column_name(), v)
            }
        }
    }

    pub fn or(mut self, rhs: FilterExpr<'b>) -> Self {
        self.or_cons.push(rhs);
        self
    }
}

pub(crate) enum ConditionVar<'a> {
    Eq(&'a dyn ToSql),
    Neq(&'a dyn ToSql),
    Gt(&'a dyn ToSql),
    GtEq(&'a dyn ToSql),
    Lt(&'a dyn ToSql),
    LtEq(&'a dyn ToSql),
    IsNull,
    IsNotNull,
    IsIn(Vec<&'a dyn ToSql>),
    Contains(&'a str),
    StarsWith(&'a str),
    EndsWith(&'a str),
    Between((&'a dyn ToSql, &'a dyn ToSql)),
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
            ConditionVar::Contains(_) => "",
            ConditionVar::IsIn(_) => "",
            ConditionVar::Between(_) => "",
            ConditionVar::StarsWith(_) => "",
            ConditionVar::EndsWith(_) => ""
        }
    }
}