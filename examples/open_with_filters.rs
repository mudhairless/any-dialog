//! The file pickers with extension filters: build two named
//! filter groups with `FileFilter`, open one file through
//! them, then open several at once through the same filters.
//!
//! Run it with `cargo run --example open_with_filters` on a
//! machine with a desktop session.
//!
//! One platform note, also documented on `FileFilter`:
//! osascript merges all filter groups into a single flat list
//! of extensions, so on macOS the picker offers the union of
//! both groups under no names; the other backends show the
//! groups as named, selectable filters.

use any_dialog::{Dialogs, FileFilter};

fn main() -> Result<(), any_dialog::Error> {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        eprintln!("no dialog backend on this system (on Linux, install kdialog or zenity)");
        return Ok(());
    }

    // Extensions are bare, without a dot or a glob: "rs", not
    // "*.rs". Each filter is one named group in the picker.
    let filters = [
        FileFilter {
            name: "Rust source",
            extensions: &["rs", "toml"],
        },
        FileFilter {
            name: "Text and markdown",
            extensions: &["txt", "md"],
        },
    ];

    match dialogs.prompt_open_file("Pick one file", None, &filters)? {
        Some(path) => println!("one file: {}", path.display()),
        None => {
            println!("cancelled at the single picker");
            return Ok(());
        }
    }

    match dialogs.prompt_open_files("Pick any number of files", None, &filters)? {
        Some(paths) => {
            println!("{} file(s) picked:", paths.len());
            for path in &paths {
                println!("  {}", path.display());
            }
        }
        None => println!("cancelled at the multiple picker"),
    }
    Ok(())
}
