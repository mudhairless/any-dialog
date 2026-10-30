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
mod powershell;
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
    Powershell,
}

/// Probe for a backend, in preference order. Each OS's bundled
/// helper goes first on its own OS (osascript on macOS,
/// powershell.exe on Windows); anywhere else the bundled helpers
/// are last resorts that normally are not installed at all.
pub(crate) fn probe() -> Option<(Backend, PathBuf)> {
    let order: [(Backend, &str); 4] = if cfg!(target_os = "windows") {
        [
            (Backend::Powershell, powershell::PROGRAM),
            (Backend::Kdialog, kdialog::PROGRAM),
            (Backend::Zenity, zenity::PROGRAM),
            (Backend::Osascript, osascript::PROGRAM),
        ]
    } else if cfg!(target_os = "macos") {
        [
            (Backend::Osascript, osascript::PROGRAM),
            (Backend::Kdialog, kdialog::PROGRAM),
            (Backend::Zenity, zenity::PROGRAM),
            (Backend::Powershell, powershell::PROGRAM),
        ]
    } else {
        [
            (Backend::Kdialog, kdialog::PROGRAM),
            (Backend::Zenity, zenity::PROGRAM),
            (Backend::Osascript, osascript::PROGRAM),
            (Backend::Powershell, powershell::PROGRAM),
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
        Backend::Powershell => powershell::argv(request),
    }
}

/// Environment variables to set for a run. Only the PowerShell
/// backend takes its values this way (see its module docs); the
/// other backends take everything as arguments.
pub(crate) fn env(backend: Backend, request: &Request<'_>) -> Vec<(String, String)> {
    match backend {
        Backend::Powershell => powershell::env(request),
        _ => Vec::new(),
    }
}

/// Interpret what a helper said on its way out. Backends using the
/// shared convention go through [`crate::outcome`]; osascript and
/// PowerShell exit 1 for genuine errors too, so they classify for
/// themselves.
pub(crate) fn classify(
    backend: Backend,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> Outcome {
    match backend {
        Backend::Osascript => osascript::classify(exit_code, stdout, stderr),
        Backend::Powershell => powershell::classify(exit_code, stdout, stderr),
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
