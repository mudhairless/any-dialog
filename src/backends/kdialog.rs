//! The kdialog backend: how kdialog wants each dialog said.
//!
//! Everything in this file is specific to the `kdialog` helper program.
//! The shared vocabulary it renders lives in [`crate::request`], and
//! what counts as an answer or a cancel lives in [`crate::outcome`].

use crate::request::{Level, Request};

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
        Request::Password { title, label } => vec![
            "--title".to_string(),
            title.to_string(),
            "--password".to_string(),
            label.to_string(),
        ],
        Request::Choices {
            title,
            label,
            choices,
            current,
        } => {
            // The tags are the choice indices, not the choice text
            // (the single-choice list uses the text itself). A
            // checklist answer prints every picked tag quoted and
            // space-separated, so text tags would need the answer
            // parsed for quoting and escapes; index tags decode by
            // lookup instead. See `decode_choices`.
            let mut args = vec![
                "--title".to_string(),
                title.to_string(),
                "--checklist".to_string(),
                label.to_string(),
            ];
            for (index, choice) in choices.iter().enumerate() {
                args.push(index.to_string());
                args.push((*choice).to_string());
                args.push(
                    if current.contains(choice) {
                        "on"
                    } else {
                        "off"
                    }
                    .to_string(),
                );
            }
            args
        }
        // kdialog's color dialog takes no initial color, so
        // `initial` is ignored by this backend.
        Request::Color { title, initial: _ } => vec![
            "--title".to_string(),
            title.to_string(),
            "--getcolor".to_string(),
        ],
        Request::Message {
            title,
            message,
            level,
        } => {
            let flag = match level {
                Level::Info => "--msgbox",
                Level::Warning => "--sorry",
                Level::Error => "--error",
            };
            vec![
                "--title".to_string(),
                title.to_string(),
                flag.to_string(),
                message.to_string(),
            ]
        }
        Request::Confirm { title, message } => vec![
            "--title".to_string(),
            title.to_string(),
            "--yesno".to_string(),
            message.to_string(),
        ],
    }
}

/// Decode a checklist answer: the picked index tags, printed
/// quoted and space-separated (`"0" "2"`), mapped back to the
/// choice text they stood for in [`argv`].
pub(crate) fn decode_choices(text: &str, choices: &[&str]) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| token.trim_matches('"').parse::<usize>().ok())
        .filter_map(|index| choices.get(index).copied())
        .map(|choice| choice.to_string())
        .collect()
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
    fn password_argv() {
        let request = Request::Password {
            title: "Title",
            label: "Label",
        };
        assert_eq!(
            argv(&request),
            args(&["--title", "Title", "--password", "Label"])
        );
    }

    #[test]
    fn message_argv_per_level() {
        for (level, flag) in [
            (Level::Info, "--msgbox"),
            (Level::Warning, "--sorry"),
            (Level::Error, "--error"),
        ] {
            let request = Request::Message {
                title: "Title",
                message: "Message",
                level,
            };
            assert_eq!(argv(&request), args(&["--title", "Title", flag, "Message"]));
        }
    }

    #[test]
    fn choices_argv_uses_index_tags_and_marks_current() {
        let request = Request::Choices {
            title: "Title",
            label: "Label",
            choices: &["a", "b"],
            current: &["b"],
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--title",
                "Title",
                "--checklist",
                "Label",
                "0",
                "a",
                "off",
                "1",
                "b",
                "on",
            ])
        );
    }

    #[test]
    fn choices_decode_maps_index_tags_back_to_text() {
        let choices = ["a", "b", "c"];
        assert_eq!(decode_choices("\"0\" \"2\"\n", &choices), args(&["a", "c"]));
        assert_eq!(decode_choices("", &choices), Vec::<String>::new());
    }

    #[test]
    fn color_argv() {
        let request = Request::Color {
            title: "Title",
            initial: Some("#ff0080"),
        };
        assert_eq!(argv(&request), args(&["--title", "Title", "--getcolor"]));
    }

    #[test]
    fn confirm_argv() {
        let request = Request::Confirm {
            title: "Title",
            message: "Message",
        };
        assert_eq!(
            argv(&request),
            args(&["--title", "Title", "--yesno", "Message"])
        );
    }
}
