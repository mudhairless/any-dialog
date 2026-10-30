//! The backends: one file per helper program.
//!
//! Everything specific to a particular helper lives in that helper's
//! file: its program name and how it wants each dialog said. Shared
//! things stay shared and live elsewhere: what a dialog is
//! ([`crate::request`]), what counts as an answer or a cancel
//! ([`crate::outcome`]), and the error type ([`crate::error`]). A
//! backend whose answer convention ever differs from the shared one
//! brings its own classifier in its own file.
//!
//! [`Backend`] and the probing that finds a helper also live here,
//! since the set of backends and the probe order are facts about the
//! backends as a group.

mod kdialog;
mod zenity;

use std::path::{Path, PathBuf};

use crate::request::Request;

/// Which helper program a [`crate::Dialogs`] handle drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Kdialog,
    Zenity,
}

/// Probe for a backend, in preference order.
pub(crate) fn probe() -> Option<(Backend, PathBuf)> {
    for (backend, program) in [
        (Backend::Kdialog, kdialog::PROGRAM),
        (Backend::Zenity, zenity::PROGRAM),
    ] {
        if let Some(path) = find_on_path(program) {
            return Some((backend, path));
        }
    }
    None
}

/// Build the argument list for `backend`, excluding argv[0].
pub(crate) fn argv(backend: Backend, request: &Request<'_>) -> Vec<String> {
    match backend {
        Backend::Kdialog => kdialog::argv(request),
        Backend::Zenity => zenity::argv(request),
    }
}

/// The choice a list dialog should start on: the requested current
/// choice when it is among the choices, else the first choice. Shared
/// by the backends whose list dialogs take a preselection.
pub(crate) fn selected<'a>(choices: &[&'a str], current: Option<&'a str>) -> Option<&'a str> {
    current
        .filter(|current| choices.contains(current))
        .or_else(|| choices.first().copied())
}

/// Find an executable on `PATH`.
pub(crate) fn find_on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}
