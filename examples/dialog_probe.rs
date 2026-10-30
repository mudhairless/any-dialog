//! A probe harness for validating the osascript backend on a real
//! Mac. Not part of the crate's API and not shipped anywhere; it
//! exists so the macOS assumptions in `src/backends/osascript.rs`
//! can be checked on hardware neither maintainer machine is.
//!
//! Modes:
//!
//! - `--scripted` (the default): what CI runs unattended. Raw
//!   osascript probes use `giving up after` so nothing can block:
//!   they confirm the result shapes the crate's scripts rely on.
//!   The list dialog is skipped here, because `choose from list`
//!   has no timeout parameter and would hang an unattended job.
//! - `--case text|choice|error`: run exactly one dialog through
//!   the crate and print the mapped result. CI uses this with a
//!   synthetic Escape keypress; humans can use it too.
//! - `--interactive`: the full human checklist through the crate:
//!   OK, cancel, selection, and hostile text (quotes, commas,
//!   non-ASCII), printing the expected and actual result per case.
//!
//! Every case prints one structured line: the case name, then the
//! exit code, stdout, and stderr (raw probes) or the crate's mapped
//! `Result` (crate cases), so a run log is the validation report.

use std::io::Write;
use std::process::Command;

use any_dialog::Dialogs;

const RAW_TEXT_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) default answer (item 3 of argv) with title (item 1 of argv) giving up after 3",
    "return text returned of answer",
    "end run",
];

const RAW_ALERT_SCRIPT: &[&str] = &[
    "on run argv",
    "display alert (item 1 of argv) message (item 2 of argv) giving up after 3",
    "return \"\"",
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
    println!("CRATE backend available: {}", dialogs.available());

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
    println!(
        "RAW CASE choice: SKIPPED in scripted mode (choose from list has no giving up after; covered by --case and --interactive)"
    );
}

/// Run one raw script the same way the crate runs its scripts (one
/// `-e` per line, values as run-handler arguments) and print what
/// came back, verbatim.
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
        "error" => {
            let result =
                dialogs.show_error("any-dialog probe", "Crate error probe: dismiss this alert");
            println!("CASE error => {result:?}");
        }
        other => println!("CASE {other}: unknown case (want text, choice, or error)"),
    }
    let _ = std::io::stdout().flush();
}

fn interactive() {
    let dialogs = Dialogs::new();
    println!("INTERACTIVE PROBE: work through the dialogs as they appear.");
    println!("backend available: {}", dialogs.available());

    step("1/6 text, OK path: type anything, then click OK");
    let result = dialogs.prompt_text(
        "any-dialog probe",
        "Step 1: type anything and click OK",
        Some("prefilled"),
    );
    println!("CASE text-ok => {result:?} (want Ok(Some(your text)))");

    step("2/6 text, cancel path: click Cancel");
    let result = dialogs.prompt_text("any-dialog probe", "Step 2: click Cancel", None);
    println!("CASE text-cancel => {result:?} (want Ok(None))");

    step("3/6 hostile text: leave the text as is, click OK");
    let result = dialogs.prompt_text(
        "Probe \"quotes\", commas",
        "Step 3: quotes \"here\", commas, and ünicode in label and text",
        Some("a\"b,c—d"),
    );
    println!("CASE text-hostile => {result:?} (want Ok(Some(\"a\\\"b,c—d\")))");

    step("4/6 choice, pick gamma");
    let result = dialogs.prompt_choice(
        "any-dialog probe",
        "Step 4: pick gamma and click OK",
        &["alpha", "beta", "gamma"],
        Some("beta"),
    );
    println!("CASE choice-pick => {result:?} (want Ok(Some(\"gamma\")))");

    step("5/6 choice, cancel path: click Cancel");
    let result = dialogs.prompt_choice(
        "any-dialog probe",
        "Step 5: click Cancel",
        &["alpha", "beta", "gamma"],
        None,
    );
    println!("CASE choice-cancel => {result:?} (want Ok(None))");

    step("6/6 error alert: dismiss it");
    let result = dialogs.show_error("any-dialog probe", "Step 6: dismiss this alert");
    println!("CASE error => {result:?} (want Ok(()))");

    println!("INTERACTIVE PROBE done. Save this output with your macOS version.");
}

fn step(text: &str) {
    println!("STEP {text}");
    let _ = std::io::stdout().flush();
}
