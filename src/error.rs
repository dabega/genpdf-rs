// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Error types for `genpdf`.

use std::error;
use std::fmt;
use std::io;

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
            ErrorKind::IoError(err) => Some(err),
            ErrorKind::PrintpdfError(err) => Some(err),
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
    /// An IO error.
    IoError(io::Error),
    /// An error caused by `printpdf`.
    PrintpdfError(printpdf::Error),
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
        ErrorKind::PrintpdfError(error)
    }
}

impl From<rusttype::Error> for ErrorKind {
    fn from(error: rusttype::Error) -> ErrorKind {
        ErrorKind::RusttypeError(error)
    }
}
