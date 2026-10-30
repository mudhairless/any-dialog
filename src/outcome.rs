//! Interpreting what a helper program said on its way out.
//!
//! Every current backend uses the same convention: exit 0 with the
//! answer on stdout, exit 1 with empty stdout for cancel, anything
//! else is a failure. If a future backend breaks that convention, this
//! is the unit that changes.

/// The result of interpreting a helper's exit status and stdout.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Exit 0; the entered/selected text, if any.
    Answered(Option<String>),
    /// Exit 1 with no stdout: the user cancelled.
    Cancelled,
    /// Any other exit status, or exit 1 that still printed something.
    Failed,
}

/// Interpret an exit code and stdout without touching the OS.
pub(crate) fn classify(exit_code: Option<i32>, stdout: &str) -> Outcome {
    match exit_code {
        Some(0) => {
            let text = stdout.trim_end_matches(['\r', '\n']);
            if text.trim().is_empty() {
                Outcome::Answered(None)
            } else {
                Outcome::Answered(Some(text.to_string()))
            }
        }
        Some(1) if stdout.trim().is_empty() => Outcome::Cancelled,
        _ => Outcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_reads_success_cancel_and_failure() {
        assert_eq!(
            classify(Some(0), "12.50\n"),
            Outcome::Answered(Some("12.50".to_string()))
        );
        assert_eq!(classify(Some(0), ""), Outcome::Answered(None));
        assert_eq!(classify(Some(1), ""), Outcome::Cancelled);
        assert_eq!(classify(Some(1), "still text"), Outcome::Failed);
        assert_eq!(classify(Some(2), ""), Outcome::Failed);
        assert_eq!(classify(None, ""), Outcome::Failed);
    }
}
