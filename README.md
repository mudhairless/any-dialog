# any-dialog

Simple desktop dialogs via whatever dialog tool the system already has.

`any-dialog` does not link a GUI toolkit. It drives an external helper
program instead, so it has zero dependencies and uses only the Rust
standard library.

## Backends

Probed in order:

1. `kdialog` (Linux, common on KDE installs)
2. `zenity` (Linux, common on GNOME installs)

A macOS backend via `osascript` is planned.
Windows is not supported yet. Support may be added later if a suitable
approach turns up, which is why the crate name does not name an OS or a
mechanism.

The expanded feature set in ROADMAP.md is version 1.0, the first full
release. Version 0.1.0 was published early to reserve the crate name;
until 1.0 lands, this README describes the dialogs the code has today.

If neither helper is installed, calls return `Error::NoBackend`.

## What it does

Three dialogs, each returning `Ok(None)` when the user cancels:

- `prompt_text`: ask for a line of text, with an optional initial value
- `prompt_choice`: ask the user to pick one item from a list
- `show_error`: show a message with a single dismiss button

## Usage

```rust
use any_dialog::Dialogs;

fn main() -> Result<(), any_dialog::Error> {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        eprintln!("install kdialog or zenity to enable dialogs");
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
