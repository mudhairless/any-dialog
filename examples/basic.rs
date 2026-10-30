//! The crate's normal usage in one small program: check that a
//! backend exists, ask for text, and ask a yes/no question,
//! handling a cancel at each step as the `None` it is rather
//! than as an error.
//!
//! Run it with `cargo run --example basic` on a machine with a
//! desktop session. For the file dialogs, including filters,
//! see the `open_with_filters` example.

use any_dialog::Dialogs;

fn main() -> Result<(), any_dialog::Error> {
    let dialogs = Dialogs::new();
    if !dialogs.available() {
        eprintln!("no dialog backend on this system (on Linux, install kdialog or zenity)");
        return Ok(());
    }

    let Some(name) = dialogs.prompt_text("any-dialog basic example", "Your name:", None)? else {
        println!("cancelled at the name prompt");
        return Ok(());
    };
    println!("hello, {name}");

    if dialogs.confirm("any-dialog basic example", "Show an info message?")? {
        dialogs.show_info("any-dialog basic example", "That was a confirm.")?;
    } else {
        println!("no message wanted; done");
    }
    Ok(())
}
