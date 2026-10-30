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

/// A named set of file extensions for a file dialog, such as
/// "Images" over `png` and `jpg`. Extensions are bare names,
/// without dots or wildcards; each backend decorates them into
/// its own filter syntax. How much of the grouping survives is
/// backend-dependent: macOS's chooser takes one flat type list,
/// so there the groups merge into a single extension set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileFilter<'a> {
    pub name: &'a str,
    pub extensions: &'a [&'a str],
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
    /// Pick a file to open. `start` is where the dialog opens: a
    /// directory, or a file whose folder it opens on. `multiple`
    /// allows picking several files at once; the answer then
    /// arrives one path per line.
    OpenFile {
        title: &'a str,
        start: Option<&'a std::path::Path>,
        filters: &'a [FileFilter<'a>],
        multiple: bool,
    },
    /// Pick a file to save to. `start` may name a suggested file;
    /// its name pre-fills the dialog's name field where the
    /// backend supports that.
    SaveFile {
        title: &'a str,
        start: Option<&'a std::path::Path>,
        filters: &'a [FileFilter<'a>],
    },
    /// Pick a folder. `start` is the folder the dialog opens on.
    Folder {
        title: &'a str,
        start: Option<&'a std::path::Path>,
    },
}
