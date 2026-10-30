# any-dialog

Simple desktop dialogs via whatever dialog tool the system already has.

`any-dialog` does not link a GUI toolkit. It drives an external helper
program instead, so it has zero dependencies and uses only the Rust
standard library.

## Backends

Probed in order:

1. `kdialog` (Linux, common on KDE installs)
2. `zenity` (Linux, common on GNOME installs)
3. `osascript` (macOS, bundled with the OS)
4. `powershell` (Windows, bundled with the OS)

On macOS and Windows the order flips: the OS's own bundled helper
(`osascript` or `powershell`) goes first there, since it is part of
the OS, with the other tools as fallbacks.

The expanded feature set in ROADMAP.md is version 1.0, the first full
release. Version 0.1.0 was published early to reserve the crate name;
until 1.0 lands, this README describes the dialogs the code has today.

If no helper is installed, calls return `Error::NoBackend`.

## What it does

The prompt dialogs return `Ok(None)` when the user cancels:

- `prompt_text`: ask for a line of text, with an optional initial value
- `prompt_password`: ask for a password or other secret, masked while typed
- `prompt_choice`: ask the user to pick one item from a list
- `confirm`: ask a yes/no question, returning a bool
- `show_info`, `show_warning`, `show_error`: show a message with a
  single dismiss button, at three levels of seriousness

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
