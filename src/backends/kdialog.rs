//! The kdialog backend: how kdialog wants each dialog said.
//!
//! Everything in this file is specific to the `kdialog` helper program.
//! The shared vocabulary it renders lives in [`crate::request`], and
//! what counts as an answer or a cancel lives in [`crate::outcome`].

use crate::request::Request;

/// The helper program's name, as probed for on `PATH`.
pub(crate) const PROGRAM: &str = "kdialog";

/// Build the argument list for one request, excluding argv[0].
pub(crate) fn argv(request: &Request<'_>) -> Vec<String> {
    match *request {
        Request::Text {
            title,
            label,
            initial,
        } => {
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
        Request::Choice {
            title,
            label,
            choices,
            current,
        } => {
            let selected = super::selected(choices, current);
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
        Request::Error { title, message } => vec![
            "--title".to_string(),
            title.to_string(),
            "--error".to_string(),
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
    fn kdialog_text_argv() {
        let with_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: Some("1.00"),
        };
        assert_eq!(
            argv(&with_initial),
            args(&["--title", "Title", "--inputbox", "Label", "1.00"])
        );

        let without_initial = Request::Text {
            title: "Title",
            label: "Label",
            initial: None,
        };
        assert_eq!(
            argv(&without_initial),
            args(&["--title", "Title", "--inputbox", "Label"])
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
            argv(&request),
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
    fn error_argv() {
        let request = Request::Error {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            argv(&request),
            args(&["--title", "Title", "--error", "Message"])
        );
    }
}
