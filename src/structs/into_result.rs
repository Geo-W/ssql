use crate::SsqlMarker;
use tiberius::Row;

pub trait IntoResult {
    fn processing(r: &Row) -> Self
    where
        Self: Sized + 'static;
}

impl<Ta> IntoResult for Ta
where
    Ta: SsqlMarker,
{
    fn processing(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        Ta::row_to_struct(r)
    }
}

impl<Ta, Tb> IntoResult for (Ta, Tb)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    fn processing(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (Ta::row_to_struct(r), Tb::row_to_struct(r))
    }
}
