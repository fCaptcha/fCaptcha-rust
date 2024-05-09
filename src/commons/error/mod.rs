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

#[derive(thiserror::Error)]
pub enum DortCapError {
    #[error("ERROR")]
    InternalErr,
    #[error("Internal error. Details: {0}")]
    DetailedInternalErr(&'static str),
    #[error("0x0{0} ({1})")]
    CodeErr(u128, &'static str),
    #[error("Internal error. Details: {0}")]
    InternalErrString(String),
    #[error("JSON deserialization/serialization failed")]
    JSONErr(#[from] serde_json::Error),
    #[error("Decode error.")]
    DecodeErr(#[from] DecodeError),
    #[error("String parsing failed.")]
    StringParseErr(#[from] FromUtf8Error),
    #[error("Request failed to complete.")]
    RequestErr(#[from] reqwest::Error),
    #[error("Failed to load image.")]
    ImageErr(#[from] ImageError),
    #[error("Failed to load database entry.")]
    RedisErr(#[from] RedisError),
    #[error("Time went backwards.")]
    TimeErr(#[from] SystemTimeError),
    #[error("Invalid header value.")]
    BadHeaderValErr(#[from] InvalidHeaderValue),
    #[error("Invalid header name.")]
    BadHeaderNameErr(#[from] InvalidHeaderName),
    #[error("Integer parse failed.")]
    ParseIntErr(#[from] ParseIntError),
    #[error("Failed to parse hex string.")]
    FromHexErr(#[from] FromHexError),
    #[error("Failed to parse URL.")]
    URLErr(#[from] url::ParseError),
    #[error("String conversion failed.")]
    ToStrErr(#[from] ToStrError),
    #[error("Regex match failed.")]
    RegexErr(#[from] regex::Error),
    #[error("Thread join Error.")]
    ThreadJoinErr(#[from] JoinError),
    #[error("Shape Error.")]
    ShapeErr(#[from] ShapeError),
    // #[error("Request Error.")]
    // CurlErr(#[from] ratcurl::Error),
    #[error("I/O Error")]
    IOErr(#[from] std::io::Error),
    #[error("Encode Error")]
    EncodeErr(#[from] serde_urlencoded::ser::Error),
    #[error("{0}")]
    AnyhowErr(#[from] anyhow::Error),
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
