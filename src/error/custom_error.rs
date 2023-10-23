#![allow(dead_code)]

use std::fmt;
use std::fmt::Formatter;

#[cfg(feature = "polars")]
use polars::error::PolarsError;
use serde;

/// Alias for Result<T, [`SsqlError`]>
///
/// [`SsqlError`]: enum.SsqlError
pub type SsqlResult<T> = Result<T, SsqlError>;

/// Error enum representing different errors during execution.
#[derive(Debug)]
pub enum SsqlError {
    /// An Error occurs when executing sql.
    SqlServerError(tiberius::error::Error),
    /// An Error occurs when transforming result to polars dataframe.
    #[cfg(feature = "polars")]
    PolarsError(PolarsError),
    /// An Error occurs when constructing query.
    RsRunningError(String),
}

impl std::error::Error for SsqlError {}

impl serde::Serialize for SsqlError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl fmt::Display for SsqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = match self {
            SsqlError::SqlServerError(inner) => {
                format!("Error occur when executing sql: {}", inner)
            }
            #[cfg(feature = "polars")]
            SsqlError::PolarsError(inner) => {
                format!("Error occur when transforming to polars: {}", inner)
            }
            SsqlError::RsRunningError(inner) => inner.to_string(),
        };
        write!(f, "{}", a)
    }
}

#[cfg(feature = "polars")]
impl From<PolarsError> for SsqlError {
    fn from(value: PolarsError) -> Self {
        SsqlError::PolarsError(value)
    }
}

impl From<tiberius::error::Error> for SsqlError {
    fn from(value: tiberius::error::Error) -> Self {
        SsqlError::SqlServerError(value)
    }
}

impl From<&'static str> for SsqlError {
    fn from(value: &'static str) -> Self {
        SsqlError::RsRunningError(value.to_string())
    }
}

impl From<String> for SsqlError {
    fn from(value: String) -> Self {
        SsqlError::RsRunningError(value)
    }
}

// this would work only with nightly build #![feature(with_negative_coherence)]
// impl From<&str> for AppError{
//     fn from(value: &str) -> Self {
//         AppError::new(value)
//     }
// }

#[derive(Debug)]
pub struct CustomError {
    msg: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.msg)
    }
}

#[derive(Debug)]
pub struct SentError {
    msg: String,
}

impl fmt::Display for SentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.msg)
    }
}
