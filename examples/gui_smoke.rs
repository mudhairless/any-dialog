//! A GUI-subsystem smoke test, for Windows: build it with
//! `cargo build --example gui_smoke`, then double-click
//! `target/debug/examples/gui_smoke.exe` in Explorer. Three
//! dialogs appear in sequence. Because the exe has no console
//! of its own, it exercises the case a console test cannot
//! see: whether spawning the (console) powershell.exe helper
//! pops up a console window next to the dialogs. It should
//! not; the crate spawns its helpers with CREATE_NO_WINDOW.
//!
//! On other platforms this builds as an ordinary example and
//! simply shows the same three dialogs from a terminal.
#![cfg_attr(windows, windows_subsystem = "windows")]

use any_dialog::Dialogs;

fn main() {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        return;
    }
    let _ = dialogs.show_info(
        "any-dialog GUI smoke test",
        "Watch for a black console window while these dialogs are up: there should be none.",
    );
    let _ = dialogs.prompt_text("any-dialog GUI smoke test", "Type anything:", None);
    let _ = dialogs.confirm(
        "any-dialog GUI smoke test",
        "Did every dialog appear without a console window?",
    );
}
