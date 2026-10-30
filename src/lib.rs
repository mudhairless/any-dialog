//! any-dialog: Minimal desktop dialogs over `kdialog`, `zenity`, or
//! `osascript`.
//!
//! This module is deliberately self-contained: it uses only the standard
//! library, knows nothing about the rest of the application, and can be
//! lifted into another project unchanged.
//!
//! Every call blocks until the user answers, so callers should run them on a
//! worker thread rather than on a UI or event-loop thread.

mod backends;
mod error;
mod outcome;
mod request;

pub use backends::Backend;
pub use error::Error;

use std::path::PathBuf;
use std::process::{Command, Stdio};

use backends::{argv, classify};
use outcome::Outcome;
use request::Request;

/// A handle to the best dialog backend found on this machine.
#[derive(Clone, Debug)]
pub struct Dialogs {
    backend: Option<(Backend, PathBuf)>,
}

impl Dialogs {
    /// Probe `PATH` for a backend, in the backends module's probe
    /// order.
    pub fn new() -> Dialogs {
        Dialogs {
            backend: backends::probe(),
        }
    }

    /// Build a handle from explicit executable paths, bypassing `PATH`.
    /// Primarily useful for tests. The first path present wins, in the
    /// order kdialog, zenity, osascript.
    pub fn with_paths(
        kdialog: Option<PathBuf>,
        zenity: Option<PathBuf>,
        osascript: Option<PathBuf>,
    ) -> Dialogs {
        let backend = kdialog
            .map(|path| (Backend::Kdialog, path))
            .or_else(|| zenity.map(|path| (Backend::Zenity, path)))
            .or_else(|| osascript.map(|path| (Backend::Osascript, path)));
        Dialogs { backend }
    }

    /// Is there a backend to drive?
    pub fn available(&self) -> bool {
        self.backend.is_some()
    }

    /// Ask for a line of text. `Ok(None)` means the user cancelled.
    pub fn prompt_text(
        &self,
        title: &str,
        label: &str,
        initial: Option<&str>,
    ) -> Result<Option<String>, Error> {
        self.run(Request::Text {
            title,
            label,
            initial,
        })
    }

    /// Ask the user to pick one of `choices`. `Ok(None)` means they cancelled.
    #[allow(dead_code)] // part of the liftable adapter API; the tray has no picker yet
    pub fn prompt_choice(
        &self,
        title: &str,
        label: &str,
        choices: &[&str],
        current: Option<&str>,
    ) -> Result<Option<String>, Error> {
        self.run(Request::Choice {
            title,
            label,
            choices,
            current,
        })
    }

    /// Show a message with a single dismiss button.
    pub fn show_error(&self, title: &str, message: &str) -> Result<(), Error> {
        self.run(Request::Error { title, message }).map(|_| ())
    }

    fn run(&self, request: Request<'_>) -> Result<Option<String>, Error> {
        let Some((backend, program)) = &self.backend else {
            return Err(Error::NoBackend);
        };

        let output = Command::new(program)
            .args(argv(*backend, &request))
            .stdin(Stdio::null())
            .output()
            .map_err(|source| Error::Spawn {
                program: program.clone(),
                source,
            })?;

        match classify(
            *backend,
            output.status.code(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        ) {
            Outcome::Answered(text) => Ok(text),
            Outcome::Cancelled => Ok(None),
            Outcome::Failed => Err(Error::Exit {
                program: program.clone(),
                status: output.status,
            }),
        }
    }
}

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_paths_choose_the_backend() {
        assert!(!Dialogs::with_paths(None, None, None).available());
        assert!(Dialogs::with_paths(Some(PathBuf::from("/bin/kdialog")), None, None).available());
        assert!(Dialogs::with_paths(None, Some(PathBuf::from("/bin/zenity")), None).available());
        assert!(
            Dialogs::with_paths(None, None, Some(PathBuf::from("/usr/bin/osascript"))).available()
        );
    }
}
