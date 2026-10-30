//! Which helper programs exist, and finding them.
//!
//! Probing lives here: adding a backend means adding a variant, a
//! program name to probe for, and a rendering in the argv unit.

use std::path::{Path, PathBuf};

/// Which helper program a [`crate::Dialogs`] handle drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Kdialog,
    Zenity,
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
