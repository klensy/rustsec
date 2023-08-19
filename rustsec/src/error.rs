//! Error types used by this crate

use std::{
    fmt::{self, Display},
    io,
    str::Utf8Error,
};
use thiserror::Error;

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! format_err {
    ($kind:path, $msg:expr) => {
        crate::error::Error::new(
            $kind,
            &$msg.to_string()
        )
    };
    ($kind:path, $fmt:expr, $($arg:tt)+) => {
        format_err!($kind, &format!($fmt, $($arg)+))
    };
}

/// Create and return an error with a formatted message
macro_rules! fail {
    ($kind:path, $msg:expr) => {
        return Err(format_err!($kind, $msg).into())
    };
    ($kind:path, $fmt:expr, $($arg:tt)+) => {
        fail!($kind, &format!($fmt, $($arg)+))
    };
}

/// Result alias with the `rustsec` crate's `Error` type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type
#[derive(Debug)]
pub struct Error {
    /// Kind of error
    kind: ErrorKind,

    /// Message providing additional information
    msg: String,
}

impl Error {
    /// Create a new error with the given description
    pub fn new<S: ToString>(kind: ErrorKind, description: &S) -> Self {
        Self {
            kind,
            msg: description.to_string(),
        }
    }

    /// Obtain the inner `ErrorKind` for this error
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", &self.kind, &self.msg)
    }
}

impl std::error::Error for Error {}

/// Custom error type for this library
#[derive(Copy, Clone, Debug, Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Invalid argument or parameter
    #[error("bad parameter")]
    BadParam,

    /// Error performing an automatic fix
    #[cfg(feature = "fix")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fix")))]
    #[error("fix failed")]
    Fix,

    /// An error occurred performing an I/O operation (e.g. network, file)
    #[error("I/O operation failed")]
    Io,

    /// Not found
    #[error("not found")]
    NotFound,

    /// Unable to acquire filesystem lock
    #[error("unable to acquire filesystem lock")]
    LockTimeout,

    /// Couldn't parse response data
    #[error("parse error")]
    Parse,

    /// Registry-related error
    #[error("registry")]
    Registry,

    /// Git operation failed
    #[error("git operation failed")]
    Repo,

    /// Errors related to versions
    #[error("bad version")]
    Version,
}

impl From<Utf8Error> for Error {
    fn from(other: Utf8Error) -> Self {
        format_err!(ErrorKind::Parse, &other)
    }
}

#[cfg(feature = "fix")]
#[cfg_attr(docsrs, doc(cfg(feature = "fix")))]
impl From<cargo_edit::Error> for Error {
    fn from(other: cargo_edit::Error) -> Self {
        format_err!(ErrorKind::Fix, &other)
    }
}

impl From<cargo_lock::Error> for Error {
    fn from(other: cargo_lock::Error) -> Self {
        format_err!(ErrorKind::Io, &other)
    }
}

impl From<fmt::Error> for Error {
    fn from(other: fmt::Error) -> Self {
        format_err!(ErrorKind::Io, &other)
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        format_err!(ErrorKind::Io, &other)
    }
}

#[cfg(feature = "git")]
#[cfg_attr(docsrs, doc(cfg(feature = "git")))]
impl From<tame_index::Error> for Error {
    fn from(err: tame_index::Error) -> Self {
        // Separate lock timeouts into their own LockTimeout variant.
        //
        // This is implemented with repetitive `match` rather than `if let`
        // because `if let` causes errors around partial moves :(
        match err {
            tame_index::Error::Git(git_err) => match git_err {
                tame_index::error::GitError::Lock(lock_err) => lock_err.into(),
                other => format_err!(ErrorKind::Registry, "{}", other),
            },
            other => format_err!(ErrorKind::Registry, "{}", other),
        }
    }
}

#[cfg(feature = "git")]
#[cfg_attr(docsrs, doc(cfg(feature = "git")))]
impl From<gix::lock::acquire::Error> for Error {
    fn from(other: gix::lock::acquire::Error) -> Self {
        match other {
            gix::lock::acquire::Error::Io(e) => {
                format_err!(ErrorKind::Repo, "failed to aquire directory lock: {}", e)
            }
            gix::lock::acquire::Error::PermanentlyLocked {
                // rustc doesn't recognize inline printing as uses of variables,
                // so we have to explicitly discard them here even though they are used
                resource_path: _,
                mode: _,
                attempts: _,
            } => format_err!(
                ErrorKind::LockTimeout,
                "directory \"{resource_path:?}\" still locked after {attempts} attempts"
            ),
        }
    }
}

impl From<semver::Error> for Error {
    fn from(other: semver::Error) -> Self {
        format_err!(ErrorKind::Version, &other)
    }
}

impl From<toml::de::Error> for Error {
    fn from(other: toml::de::Error) -> Self {
        format_err!(ErrorKind::Parse, &other)
    }
}
