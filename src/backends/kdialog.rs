//! The kdialog backend: how kdialog wants each dialog said.
//!
//! Everything in this file is specific to the `kdialog` helper program.
//! The shared vocabulary it renders lives in [`crate::request`], and
//! what counts as an answer or a cancel lives in [`crate::outcome`].

use crate::request::{FileFilter, Level, Request};

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
        Request::OpenFile {
            title,
            start,
            filters,
            multiple,
        } => {
            let mut args = vec!["--title".to_string(), title.to_string()];
            args.push("--getopenfilename".to_string());
            if multiple {
                args.push("--multiple".to_string());
                args.push("--separate-output".to_string());
            }
            push_start_and_filter(&mut args, start, filters);
            args
        }
        Request::SaveFile {
            title,
            start,
            filters,
        } => {
            let mut args = vec![
                "--title".to_string(),
                title.to_string(),
                "--getsavefilename".to_string(),
            ];
            push_start_and_filter(&mut args, start, filters);
            args
        }
        Request::Folder { title, start } => {
            let mut args = vec![
                "--title".to_string(),
                title.to_string(),
                "--getexistingdirectory".to_string(),
            ];
            if let Some(start) = start {
                args.push(start_text(start));
            }
            args
        }
    }
}

/// The start path as kdialog must hear it: absolute. kdialog
/// reads this argument as a URL, so a bare relative name goes
/// through KDE's URI filtering and can come back as a web URL:
/// the probe's save step was offered back as "http:
/// probe-output.txt" and then refused on Save for its scheme.
/// An absolute path is unambiguous. Falls back to the raw text
/// if absolutizing fails.
fn start_text(path: &std::path::Path) -> String {
    std::path::absolute(path)
        .map(|absolute| super::path_text(&absolute))
        .unwrap_or_else(|_| super::path_text(path))
}

/// The file dialogs take their starting path and filter as
/// positionals, the filter after the path, so a filter without a
/// start still needs a path in front of it: the current directory
/// stands in. The filter itself is KDE's format, one
/// `patterns|name` entry per filter, newline-separated.
fn push_start_and_filter(
    args: &mut Vec<String>,
    start: Option<&std::path::Path>,
    filters: &[FileFilter<'_>],
) {
    if start.is_none() && filters.is_empty() {
        return;
    }
    args.push(start.map(start_text).unwrap_or_else(|| ".".to_string()));
    if !filters.is_empty() {
        let entries: Vec<String> = filters
            .iter()
            .map(|filter| {
                let patterns: Vec<String> = filter
                    .extensions
                    .iter()
                    .map(|extension| format!("*.{extension}"))
                    .collect();
                format!("{}|{}", patterns.join(" "), filter.name)
            })
            .collect();
        args.push(entries.join("\n"));
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
                "--title",
                "Title",
                "--getopenfilename",
                "/home/user",
                "*.png *.jpg|Images",
            ])
        );
    }

    #[test]
    fn open_files_argv_is_multiple_with_separate_output() {
        let request = Request::OpenFile {
            title: "Title",
            start: None,
            filters: &[],
            multiple: true,
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--title",
                "Title",
                "--getopenfilename",
                "--multiple",
                "--separate-output",
            ])
        );
    }

    #[test]
    fn save_file_argv_uses_a_placeholder_start_for_a_filter() {
        let filters = [FileFilter {
            name: "Text",
            extensions: &["txt"],
        }];
        let request = Request::SaveFile {
            title: "Title",
            start: None,
            filters: &filters,
        };
        assert_eq!(
            argv(&request),
            args(&["--title", "Title", "--getsavefilename", ".", "*.txt|Text"])
        );
    }

    #[test]
    fn save_file_argv_absolutizes_a_relative_start() {
        // Regression test for the probe's step 21: kdialog reads
        // the start argument as a URL, so "probe-output.txt" came
        // back as an http URL and the save was refused. The argv
        // must carry the absolutized path instead.
        let request = Request::SaveFile {
            title: "Title",
            start: Some(std::path::Path::new("probe-output.txt")),
            filters: &[],
        };
        let save_argv = argv(&request);
        let start = save_argv.last().expect("a start argument");
        assert!(
            start.starts_with('/'),
            "start should be absolute, got {start}"
        );
        assert!(start.ends_with("probe-output.txt"));
        assert_ne!(start, "probe-output.txt");
    }

    #[test]
    fn folder_argv() {
        let request = Request::Folder {
            title: "Title",
            start: Some(std::path::Path::new("/home/user/docs")),
        };
        assert_eq!(
            argv(&request),
            args(&[
                "--title",
                "Title",
                "--getexistingdirectory",
                "/home/user/docs",
            ])
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
            args(&["--title", "Title", "--yesno", "Message"])
        );
    }
}
