//! Simple desktop dialogs via whatever dialog helper the system
//! already has: `kdialog` or `zenity` on Linux, `osascript` on
//! macOS, or PowerShell on Windows. The crate links no GUI
//! toolkit, has zero dependencies, and uses only the standard
//! library.
//!
//! [`Dialogs::new`] probes `PATH` for a helper, preferring the
//! operating system's own bundled tool on macOS and Windows, and
//! every method on the handle renders through the helper it
//! found. If no helper exists, calls fail with
//! [`Error::NoBackend`]; check [`Dialogs::available`] first if
//! that matters.
//!
//! Prompt methods return `Ok(None)` when the user cancels, so a
//! cancel is a value to handle, not an error:
//!
//! ```no_run
//! use any_dialog::Dialogs;
//!
//! fn main() -> Result<(), any_dialog::Error> {
//!     let dialogs = Dialogs::new();
//!     if let Some(name) = dialogs.prompt_text("Example", "Your name:", None)? {
//!         println!("hello, {name}");
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Every call blocks until the user answers, so callers should
//! run them on a worker thread rather than on a UI or event-loop
//! thread.

mod backends;
mod error;
mod outcome;
mod request;

pub use backends::Backend;
pub use error::Error;
pub use request::{FileFilter, FileFilterBuilder};

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use backends::{argv, classify, decode_choices, env};
use outcome::Outcome;
use request::{Level, Request};

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
    /// order kdialog, zenity, osascript, powershell.
    pub fn with_paths(
        kdialog: Option<PathBuf>,
        zenity: Option<PathBuf>,
        osascript: Option<PathBuf>,
        powershell: Option<PathBuf>,
    ) -> Dialogs {
        let backend = kdialog
            .map(|path| (Backend::Kdialog, path))
            .or_else(|| zenity.map(|path| (Backend::Zenity, path)))
            .or_else(|| osascript.map(|path| (Backend::Osascript, path)))
            .or_else(|| powershell.map(|path| (Backend::Powershell, path)));
        Dialogs { backend }
    }

    /// Is there a backend to drive?
    pub fn available(&self) -> bool {
        self.backend.is_some()
    }

    /// Which backend this handle drives, if any. Mostly useful
    /// for logging and diagnostics: the dialogs themselves do
    /// not depend on which helper renders them.
    pub fn backend(&self) -> Option<Backend> {
        self.backend.as_ref().map(|(backend, _)| *backend)
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

    /// Ask for a password or other secret. The text is hidden while
    /// typed. `Ok(None)` means the user cancelled.
    pub fn prompt_password(&self, title: &str, label: &str) -> Result<Option<String>, Error> {
        self.run(Request::Password { title, label })
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

    /// Ask the user to pick any number of `choices`, starting with
    /// `current` checked. `Ok(None)` means they cancelled;
    /// `Ok(Some(vec![]))` means they confirmed with nothing picked.
    pub fn prompt_choices(
        &self,
        title: &str,
        label: &str,
        choices: &[&str],
        current: &[&str],
    ) -> Result<Option<Vec<String>>, Error> {
        let backend = self.backend.as_ref().map(|(backend, _)| *backend);
        let request = Request::Choices {
            title,
            label,
            choices,
            current,
        };
        match self.run_completed(request)? {
            // A run that answered implies a backend was there to
            // answer, so the decode always has its backend.
            Completed::Answered(Some(text)) => Ok(Some(decode_choices(
                backend.expect("an answered run has a backend"),
                &text,
                choices,
            ))),
            Completed::Answered(None) => Ok(Some(Vec::new())),
            Completed::Cancelled => Ok(None),
        }
    }

    /// Ask the user to pick a color, starting from `initial` (a
    /// `#rrggbb` string) where the backend supports one. The
    /// answer is normalized to lowercase `#rrggbb`. `Ok(None)`
    /// means the user cancelled.
    pub fn prompt_color(
        &self,
        title: &str,
        initial: Option<&str>,
    ) -> Result<Option<String>, Error> {
        match self.run(Request::Color { title, initial })? {
            Some(raw) => Ok(Some(normalize_color(&raw))),
            None => Ok(None),
        }
    }

    /// Ask the user to pick a file to open. `start` is where the
    /// dialog opens: a directory, or a file whose folder it opens
    /// on. `Ok(None)` means they cancelled.
    pub fn prompt_open_file(
        &self,
        title: &str,
        start: Option<&Path>,
        filters: &[FileFilter],
    ) -> Result<Option<PathBuf>, Error> {
        self.run(Request::OpenFile {
            title,
            start,
            filters,
            multiple: false,
        })
        .map(|answer| answer.map(PathBuf::from))
    }

    /// Ask the user to pick any number of files to open, starting
    /// at `start`. `Ok(None)` means they cancelled.
    pub fn prompt_open_files(
        &self,
        title: &str,
        start: Option<&Path>,
        filters: &[FileFilter],
    ) -> Result<Option<Vec<PathBuf>>, Error> {
        let request = Request::OpenFile {
            title,
            start,
            filters,
            multiple: true,
        };
        match self.run_completed(request)? {
            Completed::Answered(Some(text)) => Ok(Some(text.lines().map(PathBuf::from).collect())),
            Completed::Answered(None) => Ok(Some(Vec::new())),
            Completed::Cancelled => Ok(None),
        }
    }

    /// Ask the user to pick a file to save to. `start` may name a
    /// suggested file, whose name pre-fills the dialog where the
    /// backend supports it. Native overwrite confirmation is left
    /// to the dialog itself. `Ok(None)` means they cancelled.
    pub fn prompt_save_file(
        &self,
        title: &str,
        start: Option<&Path>,
        filters: &[FileFilter],
    ) -> Result<Option<PathBuf>, Error> {
        self.run(Request::SaveFile {
            title,
            start,
            filters,
        })
        .map(|answer| answer.map(PathBuf::from))
    }

    /// Ask the user to pick a folder, starting at `start`.
    /// `Ok(None)` means they cancelled.
    pub fn prompt_folder(
        &self,
        title: &str,
        start: Option<&Path>,
    ) -> Result<Option<PathBuf>, Error> {
        self.run(Request::Folder { title, start })
            .map(|answer| answer.map(PathBuf::from))
    }

    /// Show an informational message with a single dismiss button.
    pub fn show_info(&self, title: &str, message: &str) -> Result<(), Error> {
        self.run(Request::Message {
            title,
            message,
            level: Level::Info,
        })
        .map(|_| ())
    }

    /// Show a warning message with a single dismiss button.
    pub fn show_warning(&self, title: &str, message: &str) -> Result<(), Error> {
        self.run(Request::Message {
            title,
            message,
            level: Level::Warning,
        })
        .map(|_| ())
    }

    /// Show an error message with a single dismiss button.
    pub fn show_error(&self, title: &str, message: &str) -> Result<(), Error> {
        self.run(Request::Message {
            title,
            message,
            level: Level::Error,
        })
        .map(|_| ())
    }

    /// Ask a yes/no question. `Ok(true)` is yes; `Ok(false)` is no,
    /// and also covers the dialog being dismissed without answering,
    /// which most backends cannot tell apart from no.
    pub fn confirm(&self, title: &str, message: &str) -> Result<bool, Error> {
        match self.run_completed(Request::Confirm { title, message })? {
            Completed::Answered(Some(text)) => Ok(text == "yes"),
            Completed::Answered(None) => Ok(true),
            Completed::Cancelled => Ok(false),
        }
    }

    fn run(&self, request: Request<'_>) -> Result<Option<String>, Error> {
        match self.run_completed(request)? {
            Completed::Answered(text) => Ok(text),
            Completed::Cancelled => Ok(None),
        }
    }

    fn run_completed(&self, request: Request<'_>) -> Result<Completed, Error> {
        let Some((backend, program)) = &self.backend else {
            return Err(Error::NoBackend);
        };

        let mut command = Command::new(program);
        command.args(argv(*backend, &request)).stdin(Stdio::null());
        for (name, value) in env(*backend, &request) {
            command.env(name, value);
        }

        let output = command.output().map_err(|source| Error::Spawn {
            program: program.clone(),
            source,
        })?;

        match classify(
            *backend,
            output.status.code(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        ) {
            Outcome::Answered(text) => Ok(Completed::Answered(text)),
            Outcome::Cancelled => Ok(Completed::Cancelled),
            Outcome::Failed => Err(Error::Exit {
                program: program.clone(),
                status: output.status,
            }),
        }
    }
}

/// A dialog run that finished: either it was answered (with whatever
/// text the backend produced, if any) or it was cancelled. Failures
/// travel as `Err` instead, so this never carries [`Outcome::Failed`].
enum Completed {
    Answered(Option<String>),
    Cancelled,
}

/// Normalize a helper's color answer to lowercase `#rrggbb`.
/// kdialog, osascript, and PowerShell already answer in hex;
/// zenity answers in CSS `rgb()` / `rgba()` form. Anything
/// unrecognized passes through trimmed and lowercased, so a
/// helper changing its answer format degrades visibly instead of
/// silently corrupting the value.
fn normalize_color(raw: &str) -> String {
    let raw = raw.trim().to_lowercase();
    let hex = raw.strip_prefix('#');
    if hex.is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())) {
        return raw;
    }
    let inner = raw
        .strip_prefix("rgba(")
        .or_else(|| raw.strip_prefix("rgb("))
        .and_then(|rest| rest.strip_suffix(')'));
    if let Some(inner) = inner {
        let parts: Vec<&str> = inner.split(',').collect();
        let channel = |index: usize| {
            parts
                .get(index)
                .and_then(|part| part.trim().parse::<u32>().ok())
                .map(|value| value.min(255))
        };
        if let (Some(r), Some(g), Some(b)) = (channel(0), channel(1), channel(2)) {
            return format!("#{r:02x}{g:02x}{b:02x}");
        }
    }
    raw
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
    fn color_answers_normalize_to_lowercase_hex() {
        assert_eq!(normalize_color("#FF0080\n"), "#ff0080");
        assert_eq!(normalize_color("rgb(255, 0, 128)"), "#ff0080");
        assert_eq!(normalize_color("rgba(0, 128, 255, 0.5)"), "#0080ff");
        // Unrecognized formats pass through, visibly degraded
        // rather than silently rewritten.
        assert_eq!(normalize_color("surprise"), "surprise");
    }

    #[test]
    fn explicit_paths_choose_the_backend() {
        assert!(!Dialogs::with_paths(None, None, None, None).available());
        assert!(
            Dialogs::with_paths(Some(PathBuf::from("/bin/kdialog")), None, None, None).available()
        );
        assert!(
            Dialogs::with_paths(None, Some(PathBuf::from("/bin/zenity")), None, None).available()
        );
        assert!(
            Dialogs::with_paths(None, None, Some(PathBuf::from("/usr/bin/osascript")), None)
                .available()
        );
        assert!(
            Dialogs::with_paths(
                None,
                None,
                None,
                Some(PathBuf::from("/windows/system32/powershell.exe"))
            )
            .available()
        );
        assert_eq!(
            Dialogs::with_paths(None, Some(PathBuf::from("/bin/zenity")), None, None).backend(),
            Some(Backend::Zenity)
        );
        assert_eq!(Dialogs::with_paths(None, None, None, None).backend(), None);
    }
}
