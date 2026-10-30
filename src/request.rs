//! The backend-independent description of a dialog to show.
//!
//! A `Request` is everything a backend needs to render one dialog and
//! nothing about how any particular helper program wants it said. New
//! dialog kinds land here first, then gain a rendering per backend.

/// How serious a [`Request::Message`] is. Each backend maps this to
/// its own message flags or icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    Info,
    Warning,
    Error,
}

/// One request, independent of which backend will render it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request<'a> {
    Text {
        title: &'a str,
        label: &'a str,
        initial: Option<&'a str>,
    },
    Password {
        title: &'a str,
        label: &'a str,
    },
    Choice {
        title: &'a str,
        label: &'a str,
        choices: &'a [&'a str],
        current: Option<&'a str>,
    },
    Choices {
        title: &'a str,
        label: &'a str,
        choices: &'a [&'a str],
        current: &'a [&'a str],
    },
    /// A color picker. `initial` is a `#rrggbb` string when the
    /// caller has one; backends that cannot take an initial color
    /// ignore it. Whatever form a backend answers in, the public
    /// API normalizes it to `#rrggbb`.
    Color {
        title: &'a str,
        initial: Option<&'a str>,
    },
    Message {
        title: &'a str,
        message: &'a str,
        level: Level,
    },
    /// A yes/no question. Renderings that can print an answer print
    /// the token `yes` or `no`; renderings that answer by exit code
    /// alone report yes as success with no output and no as a
    /// cancel. [`crate::Dialogs::confirm`] reads both forms.
    Confirm {
        title: &'a str,
        message: &'a str,
    },
}
