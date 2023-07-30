#![allow(dead_code)]

use std::fmt;
use std::fmt::Formatter;

#[cfg(feature = "polars")]
use polars::error::PolarsError;
use serde;

pub type RssqlResult<T> = Result<T, RssqlError>;

#[derive(Debug)]
pub enum RssqlError {
    SqlServerError(tiberius::error::Error),
    #[cfg(feature = "polars")]
    PolarsError(PolarsError),
    RsRunningError(&'static str),
    CustomError(CustomError),
    SentError(SentError),
}

impl serde::Serialize for RssqlError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl fmt::Display for RssqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = match self {
            RssqlError::SqlServerError(inner) => inner.to_string(),
            RssqlError::CustomError(_) => "Internal Server Error".to_owned(),
            RssqlError::SentError(inner) => inner.to_string(),
            #[cfg(feature = "polars")]
            RssqlError::PolarsError(inner) => {inner.to_string()}
            RssqlError::RsRunningError(inner) => {inner.to_string()}
        };
        write!(f, "{}", a)
    }
}

#[cfg(feature = "polars")]
impl From<PolarsError> for RssqlError {
    fn from(value: PolarsError) -> Self {
        RssqlError::PolarsError(value)
    }
}

impl From<tiberius::error::Error> for RssqlError {
    fn from(value: tiberius::error::Error) -> Self {
        RssqlError::SqlServerError(value)
    }
}

impl From<&'static str> for RssqlError {
    fn from(value: &'static str) -> Self{
        RssqlError::RsRunningError(value)
    }
}

// this would work only with nightly build #![feature(with_negative_coherence)]
// impl From<&str> for AppError{
//     fn from(value: &str) -> Self {
//         AppError::new(value)
//     }
// }

impl RssqlError {
    pub fn new(msg: &str) -> Self {
        let msg = msg.to_string();
        RssqlError::CustomError(CustomError { msg })
    }
    pub fn send(msg: &str) -> Self {
        let msg = msg.to_owned();
        RssqlError::SentError(SentError { msg })
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
