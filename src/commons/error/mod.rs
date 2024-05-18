use std::error::Error;
use std::fmt::{Debug, Formatter};
use base64::DecodeError;
use std::string::FromUtf8Error;
use image::ImageError;
use redis::RedisError;
use std::time::SystemTimeError;
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use std::num::ParseIntError;
use hex::FromHexError;
use log::error;
use tokio::task::JoinError;
use std::fmt::Result as FormatResult;
use std::hash;
use ndarray::ShapeError;

fn fmt_err(err: &impl Error, formatter: &mut Formatter<'_>) -> FormatResult {
    writeln!(formatter, "{}\n", err)?;
    let mut current = err.source();
    while let Some(cause) = current {
        writeln!(formatter, "caused by:\n {}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub type DortCapResult<T> = Result<T, DortCapError>;

#[macro_export] macro_rules! conv_option {
    ($x:expr) => {
        match ($x) {
            Some(t) => {
                Ok(t)
            },
            None => {
                Err(DetailedInternalErr("UNWRAP_FAILED"))
            }
        }
    };
}

#[derive(thiserror::Error)]
pub enum DortCapError {
    #[error("SOMETHING_WENT_HORRIBLY_FUCKING_WRONG_ERROR")]
    InternalErr,
    #[error("DETAILED_ERROR ({0})")]
    DetailedInternalErr(&'static str),
    #[error("ERROR_0x0{0} ({1})")]
    CodeErr(u128, &'static str),
    #[error("INTERNAL_ERROR ({0})")]
    InternalErrString(String),
    #[error("JSON_SERDE_ERROR")]
    JSONErr(#[from] serde_json::Error),
    #[error("BASE64_DECODE_ERROR")]
    DecodeErr(#[from] DecodeError),
    #[error("UTF8_STRING_PARSE_ERROR")]
    StringParseErr(#[from] FromUtf8Error),
    #[error("REQUEST_FAILED")]
    RequestErr(#[from] reqwest::Error),
    #[error("IMAGE_LOAD_ERROR")]
    ImageErr(#[from] ImageError),
    #[error("DATABASE_ERROR")]
    RedisErr(#[from] RedisError),
    #[error("CLOCK_TICKED_BACKWARDS_ERROR")]
    TimeErr(#[from] SystemTimeError),
    #[error("HEADER_VALUE_ERROR")]
    BadHeaderValErr(#[from] InvalidHeaderValue),
    #[error("HEADER_NAME_ERROR")]
    BadHeaderNameErr(#[from] InvalidHeaderName),
    #[error("INTEGER_PARSE_ERROR")]
    ParseIntErr(#[from] ParseIntError),
    #[error("HEX_PARSE_ERROR")]
    FromHexErr(#[from] FromHexError),
    #[error("URL_PARSE_ERROR")]
    URLErr(#[from] url::ParseError),
    #[error("HEADER_STRING_ERROR")]
    ToStrErr(#[from] ToStrError),
    #[error("REGULAR_EXPRESSION_ERROR")]
    RegexErr(#[from] regex::Error),
    #[error("THREAD_JOIN_ERROR")]
    ThreadJoinErr(#[from] JoinError),
    #[error("SHAPE_ERROR_NDARRAY")]
    ShapeErr(#[from] ShapeError),
    #[error("IO_ERROR")]
    IOErr(#[from] std::io::Error),
    #[error("ENCODE_SERDE_ERROR")]
    EncodeErr(#[from] serde_urlencoded::ser::Error),
    #[error("ANYHOW_ERROR")]
    AnyhowErr(#[from] anyhow::Error),
    #[error("HASHCASH_ERROR")]
    HashCashErr(#[from] hashcash::HcError),
}
impl From<String> for DortCapError {
    fn from(value: String) -> Self {
        DortCapError::InternalErrString(value)
    }
}

impl Debug for DortCapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        fmt_err(self, f)
    }
}
