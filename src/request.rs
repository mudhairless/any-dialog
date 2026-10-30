//! The backend-independent description of a dialog to show.
//!
//! A `Request` is everything a backend needs to render one dialog and
//! nothing about how any particular helper program wants it said. New
//! dialog kinds land here first, then gain a rendering per backend.

/// One request, independent of which backend will render it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request<'a> {
    Text {
        title: &'a str,
        label: &'a str,
        initial: Option<&'a str>,
    },
    Choice {
        title: &'a str,
        label: &'a str,
        choices: &'a [&'a str],
        current: Option<&'a str>,
    },
    Error {
        title: &'a str,
        message: &'a str,
    },
}
