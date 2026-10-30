//! The PowerShell backend: how Windows' bundled `powershell.exe`
//! wants each dialog said.
//!
//! The helper runs one of the fixed WinForms scripts in the
//! `powershell` directory next to this file (embedded with
//! `include_str!`, so the crate still ships as a single binary
//! artefact with zero dependencies, and CI can lint the scripts as
//! ordinary .ps1 files). The scripts build real Win32 dialogs
//! through System.Windows.Forms, so every answer comes back as a
//! DialogResult: cancel is always distinguishable from an empty
//! answer, which is what the Visual Basic InputBox route could not
//! do and why this backend exists at all.
//!
//! User text is never interpolated into a script, and it does not
//! travel as command line arguments either: with a `-Command`
//! string, PowerShell treats anything after the string as more
//! command text, which would hand user input to the parser. Values
//! travel as `ANYDIALOG_*` environment variables instead
//! ([`env`]), which are exact and Unicode safe.
//!
//! The scripts set their own exit contract: 0 with the answer on
//! stdout, 1 with nothing printed anywhere for cancel, 2 with the
//! message on stderr for an error. PowerShell itself also exits 1
//! when a script dies outside a try/catch, so this backend
//! classifies for itself like osascript does: an exit 1 counts as
//! a cancel only when both output streams are empty.

use crate::outcome::Outcome;
use crate::request::{Level, Request};

/// The helper program's name, as probed for on `PATH`.
pub(crate) const PROGRAM: &str = "powershell.exe";

const TEXT_SCRIPT: &str = include_str!("powershell/text.ps1");
const PASSWORD_SCRIPT: &str = include_str!("powershell/password.ps1");
const CHOICE_SCRIPT: &str = include_str!("powershell/choice.ps1");
const MESSAGE_SCRIPT: &str = include_str!("powershell/message.ps1");
const CONFIRM_SCRIPT: &str = include_str!("powershell/confirm.ps1");

/// Build the argument list for one request, excluding argv[0]: the
/// fixed invocation flags, then the request's script as the
/// `-Command` string.
pub(crate) fn argv(request: &Request<'_>) -> Vec<String> {
    let script = match request {
        Request::Text { .. } => TEXT_SCRIPT,
        Request::Password { .. } => PASSWORD_SCRIPT,
        Request::Choice { .. } => CHOICE_SCRIPT,
        Request::Message { .. } => MESSAGE_SCRIPT,
        Request::Confirm { .. } => CONFIRM_SCRIPT,
    };
    [
        "-NoProfile",
        "-STA",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]
    .iter()
    .map(|part| (*part).to_string())
    .collect()
}

/// The environment variables carrying one request's values to its
/// script. Everything else about the invocation is fixed flags.
pub(crate) fn env(request: &Request<'_>) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut push = |name: &str, value: &str| {
        pairs.push((name.to_string(), value.to_string()));
    };
    match *request {
        Request::Text {
            title,
            label,
            initial,
        } => {
            push("ANYDIALOG_TITLE", title);
            push("ANYDIALOG_LABEL", label);
            push("ANYDIALOG_INITIAL", initial.unwrap_or(""));
        }
        Request::Choice {
            title,
            label,
            choices,
            current,
        } => {
            push("ANYDIALOG_TITLE", title);
            push("ANYDIALOG_LABEL", label);
            push(
                "ANYDIALOG_CURRENT",
                super::selected(choices, current).unwrap_or(""),
            );
            push("ANYDIALOG_CHOICE_COUNT", &choices.len().to_string());
            for (index, choice) in choices.iter().enumerate() {
                push(&format!("ANYDIALOG_CHOICE_{index}"), choice);
            }
        }
        Request::Password { title, label } => {
            push("ANYDIALOG_TITLE", title);
            push("ANYDIALOG_LABEL", label);
        }
        Request::Message {
            title,
            message,
            level,
        } => {
            push("ANYDIALOG_TITLE", title);
            push("ANYDIALOG_MESSAGE", message);
            push(
                "ANYDIALOG_ICON",
                match level {
                    Level::Info => "Information",
                    Level::Warning => "Warning",
                    Level::Error => "Error",
                },
            );
        }
        Request::Confirm { title, message } => {
            push("ANYDIALOG_TITLE", title);
            push("ANYDIALOG_MESSAGE", message);
        }
    }
    pairs
}

/// Interpret a PowerShell run. Success reuses the shared reading of
/// stdout; an exit 1 is a cancel only when nothing was printed on
/// either stream, since the scripts print their errors to stderr
/// and exit 2, while an unscripted PowerShell failure exits 1 with
/// noise.
pub(crate) fn classify(exit_code: Option<i32>, stdout: &str, stderr: &str) -> Outcome {
    match exit_code {
        Some(0) => crate::outcome::classify(Some(0), stdout),
        Some(1) if stdout.trim().is_empty() && stderr.trim().is_empty() => Outcome::Cancelled,
        _ => Outcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    fn pair(name: &str, value: &str) -> (String, String) {
        (name.to_string(), value.to_string())
    }

    #[test]
    fn powershell_argv_is_flags_then_the_right_script() {
        let text = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        let text_argv = argv(&text);
        assert_eq!(
            text_argv[..5],
            owned(&[
                "-NoProfile",
                "-STA",
                "-ExecutionPolicy",
                "Bypass",
                "-Command"
            ])
        );
        assert_eq!(text_argv[5], TEXT_SCRIPT);
        assert!(text_argv[5].contains("TextBox"));

        let choice = Request::Choice {
            title: "Title",
            label: "Label",
            choices: &["a"],
            current: None,
        };
        assert_eq!(argv(&choice)[5], CHOICE_SCRIPT);

        let password = Request::Password {
            title: "Title",
            label: "Label",
        };
        assert_eq!(argv(&password)[5], PASSWORD_SCRIPT);
        assert!(argv(&password)[5].contains("UseSystemPasswordChar"));

        let message = Request::Message {
            title: "Title",
            message: "Message",
            level: Level::Info,
        };
        assert_eq!(argv(&message)[5], MESSAGE_SCRIPT);

        let confirm = Request::Confirm {
            title: "Title",
            message: "Message",
        };
        assert_eq!(argv(&confirm)[5], CONFIRM_SCRIPT);
        assert!(argv(&confirm)[5].contains("YesNo"));
    }

    #[test]
    fn powershell_env_carries_the_values() {
        let text = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        assert_eq!(
            env(&text),
            vec![
                pair("ANYDIALOG_TITLE", "Title"),
                pair("ANYDIALOG_LABEL", "Label"),
                pair("ANYDIALOG_INITIAL", ""),
            ]
        );

        let choice = Request::Choice {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: Some("b"),
        };
        assert_eq!(
            env(&choice),
            vec![
                pair("ANYDIALOG_TITLE", "Title"),
                pair("ANYDIALOG_LABEL", "Label"),
                pair("ANYDIALOG_CURRENT", "b"),
                pair("ANYDIALOG_CHOICE_COUNT", "2"),
                pair("ANYDIALOG_CHOICE_0", "a"),
                pair("ANYDIALOG_CHOICE_1", "b"),
            ]
        );

        let password = Request::Password {
            title: "Title",
            label: "Label",
        };
        assert_eq!(
            env(&password),
            vec![
                pair("ANYDIALOG_TITLE", "Title"),
                pair("ANYDIALOG_LABEL", "Label"),
            ]
        );

        let message = Request::Message {
            title: "Title",
            message: "Message",
            level: Level::Warning,
        };
        assert_eq!(
            env(&message),
            vec![
                pair("ANYDIALOG_TITLE", "Title"),
                pair("ANYDIALOG_MESSAGE", "Message"),
                pair("ANYDIALOG_ICON", "Warning"),
            ]
        );

        let confirm = Request::Confirm {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            env(&confirm),
            vec![
                pair("ANYDIALOG_TITLE", "Title"),
                pair("ANYDIALOG_MESSAGE", "Message"),
            ]
        );
    }

    #[test]
    fn powershell_classifier_wants_a_silent_exit_1_for_cancel() {
        assert_eq!(
            classify(Some(0), "typed\n", ""),
            Outcome::Answered(Some("typed".to_string()))
        );
        assert_eq!(classify(Some(1), "", ""), Outcome::Cancelled);
        // A script error that escaped the catch block also exits 1,
        // but it leaves noise on stderr, so it is not a cancel.
        assert_eq!(classify(Some(1), "", "parser error"), Outcome::Failed);
        assert_eq!(classify(Some(2), "", "boom"), Outcome::Failed);
        assert_eq!(classify(None, "", ""), Outcome::Failed);
    }
}
