//! A probe harness for validating the backends on real
//! hardware. Not part of the crate's API and not shipped
//! anywhere; it exists so the platform assumptions in the
//! backends can be checked where they actually run: the
//! osascript assumptions need a real Mac, and the PowerShell
//! backend gets the same human check on a real Windows box
//! (the maintainer's machine dual-boots). The `--case` and
//! `--interactive` modes drive the crate itself, so they work
//! on any OS and always exercise whichever backend the crate
//! probes there.
//!
//! Modes:
//!
//! - `--scripted` (the default, macOS only): what CI runs unattended. Raw
//!   osascript probes use `giving up after` so nothing can block:
//!   they confirm the result shapes the crate's scripts rely on.
//!   The list, color, and file dialogs are skipped here, because
//!   `choose from list`, `choose color`, and the `choose file`
//!   family have no timeout parameter and would hang an
//!   unattended job.
//! - `--case text|password|choice|multi|color|open|open-multi|save|folder|error|info|warning|confirm`:
//!   run exactly one dialog through the crate and print the mapped
//!   result. CI uses this with a synthetic Escape keypress; humans
//!   can use it too.
//! - `--interactive`: the full human checklist through the crate:
//!   OK, cancel, selection, and hostile text (quotes, commas,
//!   non-ASCII), printing the expected and actual result per case.
//!
//! Every case prints one structured line: the case name, then the
//! exit code, stdout, and stderr (raw probes) or the crate's mapped
//! `Result` (crate cases), so a run log is the validation report.

use std::io::Write;
#[cfg(target_os = "macos")]
use std::process::Command;

use any_dialog::Dialogs;

#[cfg(target_os = "macos")]
const RAW_TEXT_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) default answer (item 3 of argv) with title (item 1 of argv) giving up after 3",
    "return text returned of answer",
    "end run",
];

#[cfg(target_os = "macos")]
const RAW_ALERT_SCRIPT: &[&str] = &[
    "on run argv",
    "display alert (item 1 of argv) message (item 2 of argv) giving up after 3",
    "return \"\"",
    "end run",
];

#[cfg(target_os = "macos")]
const RAW_MESSAGE_SCRIPT: &[&str] = &[
    "on run argv",
    "display dialog (item 2 of argv) buttons {\"OK\"} default button \"OK\" with title (item 1 of argv) with icon (item 3 of argv as integer) giving up after 3",
    "return \"\"",
    "end run",
];

// When this dialog gives up, `button returned` comes back empty,
// so the script prints "no". That is a property of the timeout,
// not of the crate's script, which has no giving-up clause and
// can only return through a real click or a cancel.
#[cfg(target_os = "macos")]
const RAW_CONFIRM_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) buttons {\"No\", \"Yes\"} default button \"Yes\" with title (item 1 of argv) giving up after 3",
    "if button returned of answer is \"Yes\" then return \"yes\"",
    "return \"no\"",
    "end run",
];

fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref() {
        Some("--interactive") => interactive(),
        Some("--case") => {
            let case = std::env::args().nth(2).unwrap_or_default();
            crate_case(&case);
        }
        _ => scripted(),
    }
}

fn scripted() {
    let dialogs = Dialogs::new();
    println!("PROBE scripted mode");
    println!(
        "CRATE backend available: {} ({:?})",
        dialogs.available(),
        dialogs.backend()
    );

    #[cfg(not(target_os = "macos"))]
    {
        println!(
            "RAW CASES: macOS only (the raw probes drive osascript directly); use --case or --interactive on this OS"
        );
    }

    #[cfg(target_os = "macos")]
    {
        raw_case(
            "text (giving up after 3)",
            RAW_TEXT_SCRIPT,
            &[
                "any-dialog probe",
                "Scripted text probe: this dialog dismisses itself",
                "prefilled",
            ],
        );
        raw_case(
            "alert (giving up after 3)",
            RAW_ALERT_SCRIPT,
            &[
                "any-dialog probe",
                "Scripted alert probe: this alert dismisses itself",
            ],
        );
        // Message and confirm run after text and alert on purpose: the
        // CI workflow screenshots at fixed times that frame the first
        // two dialogs, and adding cases must not move them.
        raw_case(
            "message (giving up after 3)",
            RAW_MESSAGE_SCRIPT,
            &[
                "any-dialog probe",
                "Scripted message probe: this dialog dismisses itself",
                "1",
            ],
        );
        raw_case(
            "confirm (giving up after 3)",
            RAW_CONFIRM_SCRIPT,
            &[
                "any-dialog probe",
                "Scripted confirm probe: this dialog dismisses itself",
            ],
        );
        println!(
            "RAW CASE choice, multi, color, open, save, folder: SKIPPED in scripted mode (the choose commands have no giving up after; covered by --case and --interactive)"
        );
    }
}

/// Run one raw script the same way the crate runs its scripts (one
/// `-e` per line, values as run-handler arguments) and print what
/// came back, verbatim.
#[cfg(target_os = "macos")]
fn raw_case(name: &str, script: &[&str], data: &[&str]) {
    let mut args: Vec<String> = Vec::new();
    for line in script {
        args.push("-e".to_string());
        args.push((*line).to_string());
    }
    args.extend(data.iter().map(|value| (*value).to_string()));

    match Command::new("osascript").args(&args).output() {
        Ok(output) => println!(
            "RAW CASE {name}: exit={:?} stdout={:?} stderr={:?}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        Err(error) => println!("RAW CASE {name}: could not run osascript: {error}"),
    }
    let _ = std::io::stdout().flush();
}

fn crate_case(case: &str) {
    let dialogs = Dialogs::new();
    println!("CASE backend: {:?}", dialogs.backend());
    match case {
        "text" => {
            let result = dialogs.prompt_text(
                "any-dialog probe",
                "Crate text probe: press Escape or click Cancel",
                Some("prefilled"),
            );
            println!("CASE text => {result:?}");
        }
        "choice" => {
            let result = dialogs.prompt_choice(
                "any-dialog probe",
                "Crate choice probe: pick one, or cancel",
                &["alpha", "beta", "gamma"],
                Some("beta"),
            );
            println!("CASE choice => {result:?}");
        }
        "password" => {
            let result = dialogs.prompt_password(
                "any-dialog probe",
                "Crate password probe: type something, or cancel",
            );
            println!("CASE password => {result:?}");
        }
        "multi" => {
            let result = dialogs.prompt_choices(
                "any-dialog probe",
                "Crate multi probe: check any, or cancel",
                &["alpha", "beta", "gamma"],
                &["beta"],
            );
            println!("CASE multi => {result:?}");
        }
        "color" => {
            let result = dialogs.prompt_color("any-dialog probe", Some("#3366cc"));
            println!("CASE color => {result:?}");
        }
        "open" => {
            let result = dialogs.prompt_open_file("any-dialog probe", None, &[]);
            println!("CASE open => {result:?}");
        }
        "open-multi" => {
            let result = dialogs.prompt_open_files("any-dialog probe", None, &[]);
            println!("CASE open-multi => {result:?}");
        }
        "save" => {
            let result = dialogs.prompt_save_file("any-dialog probe", None, &[]);
            println!("CASE save => {result:?}");
        }
        "folder" => {
            let result = dialogs.prompt_folder("any-dialog probe", None);
            println!("CASE folder => {result:?}");
        }
        "error" => {
            let result =
                dialogs.show_error("any-dialog probe", "Crate error probe: dismiss this alert");
            println!("CASE error => {result:?}");
        }
        "info" => {
            let result =
                dialogs.show_info("any-dialog probe", "Crate info probe: dismiss this message");
            println!("CASE info => {result:?}");
        }
        "warning" => {
            let result = dialogs.show_warning(
                "any-dialog probe",
                "Crate warning probe: dismiss this message",
            );
            println!("CASE warning => {result:?}");
        }
        "confirm" => {
            let result = dialogs.confirm(
                "any-dialog probe",
                "Crate confirm probe: answer, or dismiss",
            );
            println!("CASE confirm => {result:?}");
        }
        other => println!(
            "CASE {other}: unknown case (want text, password, choice, multi, color, open, open-multi, save, folder, error, info, warning, or confirm)"
        ),
    }
    let _ = std::io::stdout().flush();
}

fn interactive() {
    let dialogs = Dialogs::new();
    println!("INTERACTIVE PROBE: work through the dialogs as they appear.");
    println!(
        "backend available: {} ({:?})",
        dialogs.available(),
        dialogs.backend()
    );

    step("1/24 text, OK path: type anything, then click OK");
    let result = dialogs.prompt_text(
        "any-dialog probe",
        "Step 1: type anything and click OK",
        Some("prefilled"),
    );
    println!("CASE text-ok => {result:?} (want Ok(Some(your text)))");

    step("2/24 text, cancel path: click Cancel");
    let result = dialogs.prompt_text("any-dialog probe", "Step 2: click Cancel", None);
    println!("CASE text-cancel => {result:?} (want Ok(None))");

    step("3/24 hostile text: leave the text as is, click OK");
    let result = dialogs.prompt_text(
        "Probe \"quotes\", commas",
        "Step 3: quotes \"here\", commas, and ünicode in label and text",
        Some("a\"b,c—d"),
    );
    println!("CASE text-hostile => {result:?} (want Ok(Some(\"a\\\"b,c—d\")))");

    step("4/24 choice, pick gamma");
    let result = dialogs.prompt_choice(
        "any-dialog probe",
        "Step 4: pick gamma and click OK",
        &["alpha", "beta", "gamma"],
        Some("beta"),
    );
    println!("CASE choice-pick => {result:?} (want Ok(Some(\"gamma\")))");

    step("5/24 choice, cancel path: click Cancel");
    let result = dialogs.prompt_choice(
        "any-dialog probe",
        "Step 5: click Cancel",
        &["alpha", "beta", "gamma"],
        None,
    );
    println!("CASE choice-cancel => {result:?} (want Ok(None))");

    step("6/24 error alert: dismiss it");
    let result = dialogs.show_error("any-dialog probe", "Step 6: dismiss this alert");
    println!("CASE error => {result:?} (want Ok(()))");

    step("7/24 info message: dismiss it");
    let result = dialogs.show_info("any-dialog probe", "Step 7: dismiss this message");
    println!("CASE info => {result:?} (want Ok(()))");

    step("8/24 warning message: dismiss it");
    let result = dialogs.show_warning("any-dialog probe", "Step 8: dismiss this message");
    println!("CASE warning => {result:?} (want Ok(()))");

    step("9/24 confirm, yes path: click Yes");
    let result = dialogs.confirm("any-dialog probe", "Step 9: click Yes");
    println!("CASE confirm-yes => {result:?} (want Ok(true))");

    step("10/24 confirm, no path: click No");
    let result = dialogs.confirm("any-dialog probe", "Step 10: click No");
    println!("CASE confirm-no => {result:?} (want Ok(false))");

    step("11/24 password, OK path: type a throwaway password, click OK");
    let result = dialogs.prompt_password(
        "any-dialog probe",
        "Step 11: the typed text should be masked; type anything",
    );
    println!("CASE password-ok => {result:?} (want Ok(Some(your text)))");

    step("12/24 password, cancel path: click Cancel");
    let result = dialogs.prompt_password("any-dialog probe", "Step 12: click Cancel");
    println!("CASE password-cancel => {result:?} (want Ok(None))");

    step("13/24 multi, pick path: check alpha and gamma, uncheck beta, click OK");
    let result = dialogs.prompt_choices(
        "any-dialog probe",
        "Step 13: check alpha and gamma only",
        &["alpha", "beta", "gamma"],
        &["beta"],
    );
    println!("CASE multi-pick => {result:?} (want Ok(Some([\"alpha\", \"gamma\"])))");

    step("14/24 multi, cancel path: click Cancel");
    let result = dialogs.prompt_choices(
        "any-dialog probe",
        "Step 14: click Cancel",
        &["alpha", "beta", "gamma"],
        &[],
    );
    println!("CASE multi-cancel => {result:?} (want Ok(None))");

    step("15/24 color, OK path: pick any color, click OK");
    let result = dialogs.prompt_color("any-dialog probe", Some("#3366cc"));
    println!("CASE color-ok => {result:?} (want Ok(Some(\"#......\")))");

    step("16/24 color, cancel path: click Cancel");
    let result = dialogs.prompt_color("any-dialog probe", None);
    println!("CASE color-cancel => {result:?} (want Ok(None))");

    step("17/24 open file, OK path: pick any file");
    let result = dialogs.prompt_open_file("any-dialog probe", None, &[]);
    println!("CASE open-ok => {result:?} (want Ok(Some(path)))");

    step("18/24 open file, cancel path: click Cancel");
    let result = dialogs.prompt_open_file("any-dialog probe", None, &[]);
    println!("CASE open-cancel => {result:?} (want Ok(None))");

    step("19/24 open files, OK path: pick two or more files");
    let result = dialogs.prompt_open_files("any-dialog probe", None, &[]);
    println!("CASE open-multi-ok => {result:?} (want Ok(Some([paths])))");

    step("20/24 open files, cancel path: click Cancel");
    let result = dialogs.prompt_open_files("any-dialog probe", None, &[]);
    println!("CASE open-multi-cancel => {result:?} (want Ok(None))");

    step("21/24 save file, OK path: name the file and save");
    let result = dialogs.prompt_save_file(
        "any-dialog probe",
        Some(std::path::Path::new("probe-output.txt")),
        &[],
    );
    println!("CASE save-ok => {result:?} (want Ok(Some(path ending probe-output.txt)))");

    step("22/24 save file, cancel path: click Cancel");
    let result = dialogs.prompt_save_file("any-dialog probe", None, &[]);
    println!("CASE save-cancel => {result:?} (want Ok(None))");

    step("23/24 folder, OK path: pick any folder");
    let result = dialogs.prompt_folder("any-dialog probe", None);
    println!("CASE folder-ok => {result:?} (want Ok(Some(path)))");

    step("24/24 folder, cancel path: click Cancel");
    let result = dialogs.prompt_folder("any-dialog probe", None);
    println!("CASE folder-cancel => {result:?} (want Ok(None))");

    println!("INTERACTIVE PROBE done. Save this output with your OS and version.");
}

fn step(text: &str) {
    println!("STEP {text}");
    let _ = std::io::stdout().flush();
}
