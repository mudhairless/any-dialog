[![CI](https://github.com/mudhairless/any-dialog/actions/workflows/ci.yml/badge.svg)](https://github.com/mudhairless/any-dialog/actions/workflows/ci.yml)

# any-dialog

Simple desktop dialogs via whatever dialog tool the system already has.

`any-dialog` does not link a GUI toolkit. It drives an external helper
program instead, so it has zero dependencies and uses only the Rust
standard library.

## Quick start

Add the crate to your project:

```sh
cargo add any-dialog
```

or by hand in `Cargo.toml`:

```toml
[dependencies]
any-dialog = "1"
```

One thing to know first: the crate drives a helper program, so
one has to exist on the machine. On macOS and Windows the helper
is part of the OS. On Linux the user needs `kdialog` or `zenity`
installed, which most KDE and GNOME systems already have; if
neither is present, calls return `Error::NoBackend`.

```rust
use any_dialog::Dialogs;

fn main() -> Result<(), any_dialog::Error> {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        eprintln!("no dialog backend available on this system");
        return Ok(());
    }

    // Ok(None) means the user cancelled. It is not an error.
    if let Some(name) = dialogs.prompt_text("Example", "Your name:", None)? {
        println!("hello, {name}");
    }

    Ok(())
}
```

The full dialog list is under [What it does](#what-it-does)
below, and the `examples/` directory has runnable programs,
including one for the file dialogs and their filters.

## Backends

Probed in order:

1. `kdialog` (Linux, common on KDE installs)
2. `zenity` (Linux, common on GNOME installs)
3. `osascript` (macOS, bundled with the OS)
4. `powershell` (Windows, bundled with the OS)

On macOS and Windows the order flips: the OS's own bundled helper
(`osascript` or `powershell`) goes first there, since it is part of
the OS, with the other tools as fallbacks.

Version 1.0 is the first full release. Version 0.1.0 was published
early to reserve the crate name.

If no helper is installed, calls return `Error::NoBackend`.

API documentation lives on [docs.rs](https://docs.rs/any-dialog),
with runnable examples in this repository's `examples/` directory.
The minimum supported Rust version is 1.85.

## What it does

The prompt dialogs return `Ok(None)` when the user cancels:

- `prompt_text`: ask for a line of text, with an optional initial value
- `prompt_password`: ask for a password or other secret, masked while typed
- `prompt_choice`: ask the user to pick one item from a list
- `prompt_choices`: ask the user to pick any number of items from a list
- `prompt_color`: ask the user to pick a color, returned as `#rrggbb`
- `prompt_open_file` / `prompt_open_files`: pick one or several
  files to open, with optional named extension filters
- `prompt_save_file`: pick a file to save to, with an optional
  suggested path
- `prompt_folder`: pick a folder
- `confirm`: ask a yes/no question, returning a bool
- `show_info`, `show_warning`, `show_error`: show a message with a
  single dismiss button, at three levels of seriousness

## Usage

```rust
use any_dialog::Dialogs;

fn main() -> Result<(), any_dialog::Error> {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        eprintln!("no dialog backend available on this system");
        return Ok(());
    }

    if let Some(name) = dialogs.prompt_text("Example", "Your name:", None)? {
        println!("hello, {name}");
    }

    Ok(())
}
```

Every call blocks until the user answers. Run calls on a worker thread,
not on a UI or event-loop thread.

## License

MIT. See LICENSE.
