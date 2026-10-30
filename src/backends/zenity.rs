//! The zenity backend: how zenity wants each dialog said.
//!
//! Everything in this file is specific to the `zenity` helper program.
//! The shared vocabulary it renders lives in [`crate::request`], and
//! what counts as an answer or a cancel lives in [`crate::outcome`].

use crate::request::{Level, Request};

/// The helper program's name, as probed for on `PATH`.
pub(crate) const PROGRAM: &str = "zenity";

/// Build the argument list for one request, excluding argv[0].
pub(crate) fn argv(request: &Request<'_>) -> Vec<String> {
    match *request {
        Request::Text {
            title,
            label,
            initial,
        } => {
            let mut args = vec![
                "--entry".to_string(),
                "--title".to_string(),
                title.to_string(),
                "--text".to_string(),
                label.to_string(),
            ];
            if let Some(initial) = initial {
                args.push("--entry-text".to_string());
                args.push(initial.to_string());
            }
            args
        }
        Request::Choice {
            title,
            label,
            choices,
            current,
        } => {
            let selected = super::selected(choices, current);
            let mut args = vec![
                "--list".to_string(),
                "--radiolist".to_string(),
                "--title".to_string(),
                title.to_string(),
                "--text".to_string(),
                label.to_string(),
                "--column".to_string(),
                "pick".to_string(),
                "--column".to_string(),
                "option".to_string(),
            ];
            for choice in choices {
                args.push(
                    if Some(*choice) == selected {
                        "TRUE"
                    } else {
                        "FALSE"
                    }
                    .to_string(),
                );
                args.push((*choice).to_string());
            }
            args
        }
        Request::Password { title, label } => vec![
            "--entry".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--text".to_string(),
            label.to_string(),
            "--hide-text".to_string(),
        ],
        Request::Choices {
            title,
            label,
            choices,
            current,
        } => {
            // The separator is a literal newline, so the answer
            // arrives one pick per line like the other backends'
            // multi answers, instead of zenity's default `|`.
            let mut args = vec![
                "--list".to_string(),
                "--checklist".to_string(),
                "--title".to_string(),
                title.to_string(),
                "--text".to_string(),
                label.to_string(),
                "--separator".to_string(),
                "\n".to_string(),
                "--column".to_string(),
                "pick".to_string(),
                "--column".to_string(),
                "option".to_string(),
            ];
            for choice in choices {
                args.push(
                    if current.contains(choice) {
                        "TRUE"
                    } else {
                        "FALSE"
                    }
                    .to_string(),
                );
                args.push((*choice).to_string());
            }
            args
        }
        Request::Color { title, initial } => {
            let mut args = vec![
                "--color-selection".to_string(),
                "--title".to_string(),
                title.to_string(),
            ];
            if let Some(initial) = initial {
                args.push("--color".to_string());
                args.push(initial.to_string());
            }
            args
        }
        Request::Message {
            title,
            message,
            level,
        } => {
            let flag = match level {
                Level::Info => "--info",
                Level::Warning => "--warning",
                Level::Error => "--error",
            };
            vec![
                "--title".to_string(),
                title.to_string(),
                flag.to_string(),
                "--text".to_string(),
                message.to_string(),
            ]
        }
        Request::Confirm { title, message } => vec![
            "--title".to_string(),
            title.to_string(),
            "--question".to_string(),
            "--text".to_string(),
            message.to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn zenity_text_argv() {
        let with_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: Some("1.00"),
        };
        assert_eq!(
            argv(&with_initial),
            args(&[
                "--entry",
                "--title",
                "Title",
                "--text",
                "Label",
                "--entry-text",
                "1.00",
            ])
        );

        let without_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        assert_eq!(
            argv(&without_initial),
            args(&["--entry", "--title", "Title", "--text", "Label"])
        );
    }

    #[test]
    fn choice_falls_back_to_the_first_entry() {
        let request = Request::Choice {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: Some("missing"),
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--list",
                "--radiolist",
                "--title",
                "Title",
                "--text",
                "Label",
                "--column",
                "pick",
                "--column",
                "option",
                "TRUE",
                "a",
                "FALSE",
                "b",
            ])
        );
    }

    #[test]
    fn password_argv() {
        let request = Request::Password {
            title: "Title",
            label: "Label",
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--entry",
                "--title",
                "Title",
                "--text",
                "Label",
                "--hide-text",
            ])
        );
    }

    #[test]
    fn message_argv_per_level() {
        for (level, flag) in [
            (Level::Info, "--info"),
            (Level::Warning, "--warning"),
            (Level::Error, "--error"),
        ] {
            let request = Request::Message {
                title: "Title",
                message: "Message",
                level,
            };
            assert_eq!(
                argv(&request),
                args(&["--title", "Title", flag, "--text", "Message"])
            );
        }
    }

    #[test]
    fn choices_argv_marks_current_and_separates_by_lines() {
        let request = Request::Choices {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: &["b"],
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--list",
                "--checklist",
                "--title",
                "Title",
                "--text",
                "Label",
                "--separator",
                "\n",
                "--column",
                "pick",
                "--column",
                "option",
                "FALSE",
                "a",
                "TRUE",
                "b",
            ])
        );
    }

    #[test]
    fn color_argv() {
        let with_initial = Request::Color {
            title: "Title",
            initial: Some("#ff0080"),
        };
        assert_eq!(
            argv(&with_initial),
            args(&[
                "--color-selection",
                "--title",
                "Title",
                "--color",
                "#ff0080"
            ])
        );

        let without_initial = Request::Color {
            title: "Title",
            initial: None,
        };
        assert_eq!(
            argv(&without_initial),
            args(&["--color-selection", "--title", "Title"])
        );
    }

    #[test]
    fn confirm_argv() {
        let request = Request::Confirm {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            argv(&request),
            args(&["--title", "Title", "--question", "--text", "Message"])
        );
    }
}
