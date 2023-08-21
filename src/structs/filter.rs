use tiberius::ToSql;

trait FilterHelper {
    fn eq<'a>(&'static self, rhs: &'a dyn ToSql) -> Filter<'a>;
}


impl FilterHelper for &'static str {
    fn eq<'a>(&'static self, rhs: &'a dyn ToSql) -> Filter<'a> {
        Filter{
            fields: vec![self.as_ref()],
            conditions: vec![rhs],
        }
    }
}


struct Filter<'b> {
    fields: Vec<&'static str>,
    conditions: Vec<&'b dyn ToSql>
}