//! The zenity backend: how zenity wants each dialog said.
//!
//! Everything in this file is specific to the `zenity` helper program.
//! The shared vocabulary it renders lives in [`crate::request`], and
//! what counts as an answer or a cancel lives in [`crate::outcome`].

use crate::request::{FileFilter, Level, Request};

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
        Request::OpenFile {
            title,
            start,
            filters,
            multiple,
        } => {
            let mut args = file_selection_head(title);
            if multiple {
                args.push("--multiple".to_string());
                args.push("--separator".to_string());
                args.push("\n".to_string());
            }
            push_filename_and_filters(&mut args, start, filters);
            args
        }
        Request::SaveFile {
            title,
            start,
            filters,
        } => {
            let mut args = file_selection_head(title);
            args.push("--save".to_string());
            args.push("--confirm-overwrite".to_string());
            push_filename_and_filters(&mut args, start, filters);
            args
        }
        Request::Folder { title, start } => {
            let mut args = file_selection_head(title);
            args.push("--directory".to_string());
            if let Some(start) = start {
                args.push("--filename".to_string());
                args.push(super::path_text(start));
            }
            args
        }
    }
}

fn file_selection_head(title: &str) -> Vec<String> {
    vec![
        "--file-selection".to_string(),
        "--title".to_string(),
        title.to_string(),
    ]
}

/// The starting path travels as `--filename`, and each filter as
/// its own `--file-filter` in zenity's `name | patterns` form.
/// Zenity adds an all-files entry itself.
fn push_filename_and_filters(
    args: &mut Vec<String>,
    start: Option<&std::path::Path>,
    filters: &[FileFilter<'_>],
) {
    if let Some(start) = start {
        args.push("--filename".to_string());
        args.push(super::path_text(start));
    }
    for filter in filters {
        let patterns: Vec<String> = filter
            .extensions
            .iter()
            .map(|extension| format!("*.{extension}"))
            .collect();
        args.push("--file-filter".to_string());
        args.push(format!("{} | {}", filter.name, patterns.join(" ")));
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
    fn open_file_argv_with_start_and_filter() {
        let filters = [FileFilter {
            name: "Images",
            extensions: &["png", "jpg"],
        }];
        let request = Request::OpenFile {
            title: "Title",
            start: Some(std::path::Path::new("/home/user")),
            filters: &filters,
            multiple: false,
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--file-selection",
                "--title",
                "Title",
                "--filename",
                "/home/user",
                "--file-filter",
                "Images | *.png *.jpg",
            ])
        );
    }

    #[test]
    fn open_files_argv_is_multiple_with_newline_separator() {
        let request = Request::OpenFile {
            title: "Title",
            start: None,
            filters: &[],
            multiple: true,
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--file-selection",
                "--title",
                "Title",
                "--multiple",
                "--separator",
                "\n",
            ])
        );
    }

    #[test]
    fn save_file_argv_confirms_overwrite() {
        let request = Request::SaveFile {
            title: "Title",
            start: Some(std::path::Path::new("/home/user/out.txt")),
            filters: &[],
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--file-selection",
                "--title",
                "Title",
                "--save",
                "--confirm-overwrite",
                "--filename",
                "/home/user/out.txt",
            ])
        );
    }

    #[test]
    fn folder_argv() {
        let request = Request::Folder {
            title: "Title",
            start: None,
        };
        assert_eq!(
            argv(&request),
            args(&["--file-selection", "--title", "Title", "--directory"])
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
