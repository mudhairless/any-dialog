//! The crate's error type.

use std::fmt;
use std::path::PathBuf;
use std::process::ExitStatus;

/// Everything that can go wrong while showing a dialog.
#[derive(Debug)]
pub enum Error {
    /// Neither `kdialog` nor `zenity` was found.
    NoBackend,
    /// The helper could not be started.
    Spawn {
        program: PathBuf,
        source: std::io::Error,
    },
    /// The helper exited with a status that does not mean success or cancel.
    Exit {
        program: PathBuf,
        status: ExitStatus,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoBackend => {
                write!(f, "no dialog backend available (install kdialog or zenity)")
            }
            Error::Spawn { program, source } => {
                write!(f, "could not run {}: {source}", program.display())
            }
            Error::Exit { program, status } => {
                write!(f, "{} exited with {status}", program.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Spawn { source, .. } => Some(source),
            _ => None,
        }
    }
}
