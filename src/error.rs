// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Error types for `genpdf`.

use std::error;
use std::fmt;
use std::io;

/// Helper trait for creating [`Error`][] instances.
///
/// This trait is inspired by [`anyhow::Context`][].
///
/// [`Error`]: struct.Error.html
/// [`anyhow::Context`]: https://docs.rs/anyhow/latest/anyhow/trait.Context.html
pub trait Context<T> {
    /// Maps the error to an [`Error`][] instance with the given message.
    ///
    /// [`Error`]: struct.Error.html
    fn context(self, msg: impl Into<String>) -> Result<T, Error>;

    /// Maps the error to an [`Error`][] instance message produced by the given callback.
    ///
    /// [`Error`]: struct.Error.html
    fn with_context<F, S>(self, cb: F) -> Result<T, Error>
    where
        F: Fn() -> S,
        S: Into<String>;
}

impl<T, E: Into<ErrorKind>> Context<T> for Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T, Error> {
        self.map_err(|err| Error::new(msg, err))
    }

    fn with_context<F, S>(self, cb: F) -> Result<T, Error>
    where
        F: Fn() -> S,
        S: Into<String>,
    {
        self.map_err(move |err| Error::new(cb(), err))
    }
}

/// An error that occured in a `genpdf` function.
///
/// The error consists of an error message (provided by the `Display` implementation) and an error
/// kind, see [`kind`](#method.kind).
#[derive(Debug)]
pub struct Error {
    msg: String,
    kind: ErrorKind,
}

impl Error {
    /// Creates a new error.
    pub fn new(msg: impl Into<String>, kind: impl Into<ErrorKind>) -> Error {
        Error {
            msg: msg.into(),
            kind: kind.into(),
        }
    }

    /// Returns the error kind for this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Internal => None,
            ErrorKind::InvalidData => None,
            ErrorKind::InvalidFont => None,
            ErrorKind::PageSizeExceeded => None,
            ErrorKind::UnsupportedEncoding => None,
            ErrorKind::IoError(err) => Some(err),
            ErrorKind::PdfError(err) => Some(err),
            ErrorKind::PdfIndexError(err) => Some(err),
            ErrorKind::RusttypeError(err) => Some(err),
        }
    }
}

/// The kind of an [`Error`](struct.Error.html).
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An internal error.
    Internal,
    /// An error caused by invalid data.
    InvalidData,
    /// An error caused by an invalid font.
    InvalidFont,
    /// An element exceeds the page size and could not be printed.
    PageSizeExceeded,
    /// A string with unsupported characters was used with a built-in font.
    UnsupportedEncoding,
    /// An IO error.
    IoError(io::Error),
    /// An error caused by invalid data in `printpdf`.
    PdfError(printpdf::PdfError),
    /// An error caused by an invalid index in `printpdf`.
    PdfIndexError(printpdf::IndexError),
    /// An error caused by `rusttype`.
    RusttypeError(rusttype::Error),
}

impl From<io::Error> for ErrorKind {
    fn from(error: io::Error) -> ErrorKind {
        ErrorKind::IoError(error)
    }
}

impl From<printpdf::Error> for ErrorKind {
    fn from(error: printpdf::Error) -> ErrorKind {
        match error {
            printpdf::Error::Io(err) => err.into(),
            printpdf::Error::Rusttype(err) => err.into(),
            printpdf::Error::Pdf(err) => err.into(),
            printpdf::Error::Index(err) => err.into(),
        }
    }
}

impl From<printpdf::IndexError> for ErrorKind {
    fn from(error: printpdf::IndexError) -> ErrorKind {
        ErrorKind::PdfIndexError(error)
    }
}

impl From<printpdf::PdfError> for ErrorKind {
    fn from(error: printpdf::PdfError) -> ErrorKind {
        ErrorKind::PdfError(error)
    }
}

impl From<rusttype::Error> for ErrorKind {
    fn from(error: rusttype::Error) -> ErrorKind {
        ErrorKind::RusttypeError(error)
    }
}
