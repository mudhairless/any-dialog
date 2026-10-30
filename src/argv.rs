//! Turning a [`Request`] into a helper program's argument list.
//!
//! This unit is pure: it builds `Vec<String>` and never touches the OS,
//! so every rendering is unit-testable without a dialog tool installed.
//! When this file outgrows one screen per backend, the natural next
//! split is one module per backend under a `backends/` directory, with
//! this file's function as the dispatch between them.
//!
//! [`Request`]: crate::request::Request

use crate::backend::Backend;
use crate::request::Request;

/// Build the argument list for `backend`, excluding argv[0].
pub(crate) fn argv(backend: Backend, request: &Request<'_>) -> Vec<String> {
    match *request {
        Request::Text {
            title,
            label,
            initial,
        } => match backend {
            Backend::Kdialog => {
                let mut args = vec![
                    "--title".to_string(),
                    title.to_string(),
                    "--inputbox".to_string(),
                    label.to_string(),
                ];
                if let Some(initial) = initial {
                    args.push(initial.to_string());
                }
                args
            }
            Backend::Zenity => {
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
        },
        Request::Choice {
            title,
            label,
            choices,
            current,
        } => {
            let selected = current
                .filter(|current| choices.contains(current))
                .or_else(|| choices.first().copied());

            match backend {
                Backend::Kdialog => {
                    let mut args = vec![
                        "--title".to_string(),
                        title.to_string(),
                        "--radiolist".to_string(),
                        label.to_string(),
                    ];
                    for choice in choices {
                        args.push((*choice).to_string());
                        args.push((*choice).to_string());
                        args.push(
                            if Some(*choice) == selected {
                                "on"
                            } else {
                                "off"
                            }
                            .to_string(),
                        );
                    }
                    args
                }
                Backend::Zenity => {
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
            }
        }
        Request::Error { title, message } => match backend {
            Backend::Kdialog => vec![
                "--title".to_string(),
                title.to_string(),
                "--error".to_string(),
                message.to_string(),
            ],
            Backend::Zenity => vec![
                "--title".to_string(),
                title.to_string(),
                "--error".to_string(),
                "--text".to_string(),
                message.to_string(),
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn kdialog_text_argv() {
        let with_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: Some("1.00"),
        };
        assert_eq!(
            argv(Backend::Kdialog, &with_initial),
            args(&["--title", "Title", "--inputbox", "Label", "1.00"])
        );

        let without_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        assert_eq!(
            argv(Backend::Kdialog, &without_initial),
            args(&["--title", "Title", "--inputbox", "Label"])
        );
    }

    #[test]
    fn zenity_text_argv() {
        let with_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: Some("1.00"),
        };
        assert_eq!(
            argv(Backend::Zenity, &with_initial),
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
            argv(Backend::Zenity, &without_initial),
            args(&["--entry", "--title", "Title", "--text", "Label"])
        );
    }

    #[test]
    fn kdialog_choice_argv_marks_the_current_choice() {
        let request = Request::Choice {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: Some("b"),
        };
        assert_eq!(
            argv(Backend::Kdialog, &request),
            args(&[
                "--title",
                "Title",
                "--radiolist",
                "Label",
                "a",
                "a",
                "off",
                "b",
                "b",
                "on",
            ])
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
            argv(Backend::Zenity, &request),
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
    fn error_argv() {
        let request = Request::Error {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            argv(Backend::Kdialog, &request),
            args(&["--title", "Title", "--error", "Message"])
        );
        assert_eq!(
            argv(Backend::Zenity, &request),
            args(&["--title", "Title", "--error", "--text", "Message"])
        );
    }
}
