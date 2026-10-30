//! The backends: one file per helper program.
//!
//! Everything specific to a particular helper lives in that helper's
//! file: its program name and how it wants each dialog said. Shared
//! things stay shared and live elsewhere: what a dialog is
//! ([`crate::request`]), what counts as an answer or a cancel
//! ([`crate::outcome`]), and the error type ([`crate::error`]). A
//! backend whose answer convention differs from the shared one
//! brings its own classifier in its own file: osascript is the
//! standing example.
//!
//! [`Backend`] and the probing that finds a helper also live here,
//! since the set of backends and the probe order are facts about the
//! backends as a group.

mod kdialog;
mod osascript;
mod zenity;

use std::path::{Path, PathBuf};

use crate::outcome::Outcome;
use crate::request::Request;

/// Which helper program a [`crate::Dialogs`] handle drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Kdialog,
    Zenity,
    Osascript,
}

/// Probe for a backend, in preference order. osascript is bundled
/// with macOS, so there it goes first; anywhere else it is a last
/// resort that normally is not installed at all.
pub(crate) fn probe() -> Option<(Backend, PathBuf)> {
    let order: [(Backend, &str); 3] = if cfg!(target_os = "macos") {
        [
            (Backend::Osascript, osascript::PROGRAM),
            (Backend::Kdialog, kdialog::PROGRAM),
            (Backend::Zenity, zenity::PROGRAM),
        ]
    } else {
        [
            (Backend::Kdialog, kdialog::PROGRAM),
            (Backend::Zenity, zenity::PROGRAM),
            (Backend::Osascript, osascript::PROGRAM),
        ]
    };
    for (backend, program) in order {
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
        Backend::Osascript => osascript::argv(request),
    }
}

/// Interpret what a helper said on its way out. Backends using the
/// shared convention go through [`crate::outcome`]; osascript's exit
/// codes cannot tell a cancel from an error without stderr, so it
/// classifies for itself.
pub(crate) fn classify(
    backend: Backend,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> Outcome {
    match backend {
        Backend::Osascript => osascript::classify(exit_code, stdout, stderr),
        _ => crate::outcome::classify(exit_code, stdout),
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
