use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;

pub(crate) enum GoldenError {
    Integer {
        field: String,
        source: ParseIntError,
    },
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Utf8 {
        path: PathBuf,
        source: FromUtf8Error,
    },
    Violation(String),
}

impl GoldenError {
    pub(super) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub(super) fn violation(message: impl Into<String>) -> Self {
        Self::Violation(message.into())
    }
}

impl fmt::Debug for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for GoldenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("golden corpus check failed: ")?;
        match self {
            Self::Integer { field, .. } => {
                write!(formatter, "cannot parse canonical {field}")
            }
            Self::Io { action, path, .. } => {
                write!(formatter, "cannot {action} `{}`", path.display())
            }
            Self::Utf8 { path, .. } => {
                write!(formatter, "{}: protocol is not UTF-8", path.display())
            }
            Self::Violation(message) => formatter.write_str(message),
        }
    }
}

impl Error for GoldenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Integer { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Utf8 { source, .. } => Some(source),
            Self::Violation(_) => None,
        }
    }
}
