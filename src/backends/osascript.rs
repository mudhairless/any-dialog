//! The osascript backend: how macOS's bundled script runner wants
//! each dialog said.
//!
//! Unlike the other helpers, osascript runs a script rather than
//! taking flags, so the scripts here are fixed text. User text is
//! never interpolated into a script: it travels as arguments to the
//! script's `run` handler instead (the arguments after the `-e`
//! lines), so no AppleScript escaping is needed anywhere. Each script
//! returns only the extracted value, never the raw result record, so
//! stdout stays a plain answer.
//!
//! Cancellation needs care. `display dialog` raises the "User
//! canceled" error (-128) itself, which osascript reports on stderr
//! with a nonzero exit. `choose from list` instead returns `false`,
//! so its script re-raises -128, giving every dialog the same cancel
//! signature. That signature is why this backend classifies results
//! itself instead of using [`crate::outcome`]: osascript also exits 1
//! for genuine script errors, so an exit code alone cannot tell a
//! cancel from a failure; the -128 marker on stderr can.

use crate::outcome::Outcome;
use crate::request::{Level, Request};

/// The helper program's name, as probed for on `PATH`.
pub(crate) const PROGRAM: &str = "osascript";

const TEXT_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) default answer (item 3 of argv) with title (item 1 of argv)",
    "return text returned of answer",
    "end run",
];

const CHOICE_SCRIPT: &[&str] = &[
    "on run argv",
    "set choices to {}",
    "if (count of argv) > 3 then set choices to items 4 thru -1 of argv",
    "set picks to choose from list choices with title (item 1 of argv) with prompt (item 2 of argv) default items {(item 3 of argv)}",
    "if picks is false then error number -128",
    "return item 1 of picks",
    "end run",
];

const ERROR_SCRIPT: &[&str] = &[
    "on run argv",
    "display alert (item 1 of argv) message (item 2 of argv)",
    "return \"\"",
    "end run",
];

const PASSWORD_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) default answer \"\" with title (item 1 of argv) hidden answer",
    "return text returned of answer",
    "end run",
];

/// Info and warning messages. Errors keep the sharper `display
/// alert` form above; these use `display dialog` because it is the
/// form that takes an icon: 1 is the application (note) icon, 2 is
/// caution. The icon number travels as a script argument like any
/// other value and is coerced to an integer inside the script.
const MESSAGE_SCRIPT: &[&str] = &[
    "on run argv",
    "display dialog (item 2 of argv) buttons {\"OK\"} default button \"OK\" with title (item 1 of argv) with icon (item 3 of argv as integer)",
    "return \"\"",
    "end run",
];

/// The buttons are named No and Yes rather than Cancel and OK on
/// purpose: a button named Cancel would raise -128 like a cancel,
/// making "no" indistinguishable from dismissing the dialog. As
/// written, no answers normally and only dismissal cancels.
const CONFIRM_SCRIPT: &[&str] = &[
    "on run argv",
    "set answer to display dialog (item 2 of argv) buttons {\"No\", \"Yes\"} default button \"Yes\" with title (item 1 of argv)",
    "if button returned of answer is \"Yes\" then return \"yes\"",
    "return \"no\"",
    "end run",
];

/// Build the argument list for one request, excluding argv[0]: one
/// `-e` per script line, then the values the script reads from argv.
pub(crate) fn argv(request: &Request<'_>) -> Vec<String> {
    let (script, mut data): (&[&str], Vec<String>) = match *request {
        Request::Text {
            title,
            label,
            initial,
        } => (
            TEXT_SCRIPT,
            vec![
                title.to_string(),
                label.to_string(),
                initial.unwrap_or("").to_string(),
            ],
        ),
        Request::Choice {
            title,
            label,
            choices,
            current,
        } => {
            let mut data = vec![
                title.to_string(),
                label.to_string(),
                super::selected(choices, current).unwrap_or("").to_string(),
            ];
            data.extend(choices.iter().map(|choice| (*choice).to_string()));
            (CHOICE_SCRIPT, data)
        }
        Request::Password { title, label } => {
            (PASSWORD_SCRIPT, vec![title.to_string(), label.to_string()])
        }
        Request::Message {
            title,
            message,
            level,
        } => match level {
            Level::Error => (ERROR_SCRIPT, vec![title.to_string(), message.to_string()]),
            Level::Info => (
                MESSAGE_SCRIPT,
                vec![title.to_string(), message.to_string(), "1".to_string()],
            ),
            Level::Warning => (
                MESSAGE_SCRIPT,
                vec![title.to_string(), message.to_string(), "2".to_string()],
            ),
        },
        Request::Confirm { title, message } => {
            (CONFIRM_SCRIPT, vec![title.to_string(), message.to_string()])
        }
    };

    let mut args = Vec::new();
    for line in script {
        args.push("-e".to_string());
        args.push((*line).to_string());
    }
    args.append(&mut data);
    args
}

/// Interpret an osascript run. Success reuses the shared reading of
/// stdout; a failure counts as a cancel only when stderr carries the
/// -128 "User canceled" marker.
pub(crate) fn classify(exit_code: Option<i32>, stdout: &str, stderr: &str) -> Outcome {
    match exit_code {
        Some(0) => crate::outcome::classify(Some(0), stdout),
        Some(1) if stderr.contains("(-128)") => Outcome::Cancelled,
        _ => Outcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn osascript_text_argv_passes_values_as_script_arguments() {
        let request = Request::Text {
            title: "Title",
            label: "Label",
            initial: Some("1.00"),
        };
        assert_eq!(
            argv(&request),
            args(&[
                "-e",
                "on run argv",
                "-e",
                "set answer to display dialog (item 2 of argv) default answer (item 3 of argv) with title (item 1 of argv)",
                "-e",
                "return text returned of answer",
                "-e",
                "end run",
                "Title",
                "Label",
                "1.00",
            ])
        );

        let without_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        assert_eq!(argv(&without_initial).last().map(String::as_str), Some(""));
    }

    #[test]
    fn osascript_choice_argv_appends_choices_after_the_selection() {
        let request = Request::Choice {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: Some("b"),
        };
        let argv = argv(&request);
        assert_eq!(
            &argv[argv.len() - 5..],
            args(&["Title", "Label", "b", "a", "b"]).as_slice()
        );
    }

    #[test]
    fn osascript_password_argv_passes_values_as_script_arguments() {
        let request = Request::Password {
            title: "Title",
            label: "Label",
        };
        assert_eq!(
            argv(&request),
            args(&[
                "-e",
                "on run argv",
                "-e",
                "set answer to display dialog (item 2 of argv) default answer \"\" with title (item 1 of argv) hidden answer",
                "-e",
                "return text returned of answer",
                "-e",
                "end run",
                "Title",
                "Label",
            ])
        );
    }

    #[test]
    fn osascript_error_message_argv_uses_the_alert_script() {
        let request = Request::Message {
            title: "Title",
            message: "Message",
            level: Level::Error,
        };
        let argv = argv(&request);
        assert_eq!(
            &argv[argv.len() - 2..],
            args(&["Title", "Message"]).as_slice()
        );
        assert!(
            argv.iter().any(|arg| arg.contains("display alert")),
            "error messages render with display alert"
        );
    }

    #[test]
    fn osascript_info_and_warning_argv_carry_their_icon_numbers() {
        for (level, icon) in [(Level::Info, "1"), (Level::Warning, "2")] {
            let request = Request::Message {
                title: "Title",
                message: "Message",
                level,
            };
            let argv = argv(&request);
            assert_eq!(
                &argv[argv.len() - 3..],
                args(&["Title", "Message", icon]).as_slice()
            );
        }
    }

    #[test]
    fn osascript_confirm_argv_passes_values_as_script_arguments() {
        let request = Request::Confirm {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            argv(&request),
            args(&[
                "-e",
                "on run argv",
                "-e",
                "set answer to display dialog (item 2 of argv) buttons {\"No\", \"Yes\"} default button \"Yes\" with title (item 1 of argv)",
                "-e",
                "if button returned of answer is \"Yes\" then return \"yes\"",
                "-e",
                "return \"no\"",
                "-e",
                "end run",
                "Title",
                "Message",
            ])
        );
    }

    #[test]
    fn osascript_classifier_needs_the_cancel_marker() {
        assert_eq!(
            classify(Some(0), "hello\n", ""),
            Outcome::Answered(Some("hello".to_string()))
        );
        assert_eq!(
            classify(Some(1), "", "36:40: execution error: User canceled. (-128)"),
            Outcome::Cancelled
        );
        // Same exit code and empty stdout as a cancel, but no marker:
        // a real error, where the shared classifier would say cancel.
        assert_eq!(classify(Some(1), "", ""), Outcome::Failed);
        assert_eq!(
            classify(Some(1), "", "0:10: syntax error: Expected end of line"),
            Outcome::Failed
        );
        assert_eq!(classify(None, "", ""), Outcome::Failed);
    }
}
