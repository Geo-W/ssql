#![allow(dead_code)]

use std::fmt;
use std::fmt::Formatter;

#[cfg(feature = "polars")]
use polars::error::PolarsError;
use serde;

pub type SsqlResult<T> = Result<T, SsqlError>;

#[derive(Debug)]
pub enum SsqlError {
    SqlServerError(tiberius::error::Error),
    #[cfg(feature = "polars")]
    PolarsError(PolarsError),
    RsRunningError(&'static str),
    CustomError(CustomError),
    SentError(SentError),
}


impl std::error::Error for SsqlError {

}

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
            SsqlError::SqlServerError(inner) => inner.to_string(),
            SsqlError::CustomError(_) => "Internal Server Error".to_owned(),
            SsqlError::SentError(inner) => inner.to_string(),
            #[cfg(feature = "polars")]
            SsqlError::PolarsError(inner) => {inner.to_string()}
            SsqlError::RsRunningError(inner) => {inner.to_string()}
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
    fn from(value: &'static str) -> Self{
        SsqlError::RsRunningError(value)
    }
}

// this would work only with nightly build #![feature(with_negative_coherence)]
// impl From<&str> for AppError{
//     fn from(value: &str) -> Self {
//         AppError::new(value)
//     }
// }

impl SsqlError {
    pub fn new(msg: &str) -> Self {
        let msg = msg.to_string();
        SsqlError::CustomError(CustomError { msg })
    }
    pub fn send(msg: &str) -> Self {
        let msg = msg.to_owned();
        SsqlError::SentError(SentError { msg })
    }
}

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
