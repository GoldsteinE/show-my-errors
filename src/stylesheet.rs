use super::Severity;
use termcolor::{Color, ColorSpec};

/// Set of styles to colorize the output
#[derive(Clone, Debug, Default)]
pub struct Stylesheet {
    /// Color of [`Severity::Info`] annotations
    pub info: ColorSpec,
    /// Color of [`Severity::Warning`] annotations
    pub warning: ColorSpec,
    /// Color of [`Severity::Error`] annotations
    pub error: ColorSpec,
    /// Color of line numbers column
    pub linenr: ColorSpec,
    /// Color of filename
    pub filename: ColorSpec,
    /// Color of annotated line content
    pub content: ColorSpec,
}

impl Stylesheet {
    /// Get a monochrome stylesheet without any colors set.
    /// This is also available via [`Default`](std::default::Default).
    pub fn monochrome() -> Self {
        Self::default()
    }

    /// Get a default rustc-like colored stylesheet
    pub fn colored() -> Self {
        let mut info = ColorSpec::new();
        let mut warning = ColorSpec::new();
        let mut error = ColorSpec::new();
        let mut linenr = ColorSpec::new();
        let mut filename = ColorSpec::new();
        let content = ColorSpec::new();
        info.set_bold(true);
        warning.set_bold(true).set_fg(Some(Color::Yellow));
        error.set_bold(true).set_fg(Some(Color::Red));
        linenr.set_bold(true).set_fg(Some(Color::Blue));
        filename.set_bold(true);
        Self {
            info,
            warning,
            error,
            linenr,
            filename,
            content,
        }
    }

    /// Get color of message by its [`Severity`]
    pub fn by_severity(&self, severity: &Severity) -> &ColorSpec {
        match severity {
            Severity::Info => &self.info,
            Severity::Warning => &self.warning,
            Severity::Error => &self.error,
        }
    }
}
