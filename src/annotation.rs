use super::{Error, Result};
use std::{
    fmt::{self, Display},
    ops::Range,
};

/// Annotation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Info => f.write_str("info"),
            Self::Warning => f.write_str("warning"),
            Self::Error => f.write_str("error"),
        }
    }
}

/// Info about annotation. You can create these manually
/// and then pass to [`AnnotationList::add`](crate::AnnotationList::add)
/// or just use [`AnnotationList`](crate::AnnotationList)s helper methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    range: Range<usize>,
    /// `header` will be shown above error message
    pub header: Option<String>,
    /// `text` will be shown near annotated fragment.
    /// Note that fragment will be highlighted even if `text` is `None`.
    /// To disable this, pass a zero length range when creating the annotation.
    pub text: Option<String>,
    pub severity: Severity,
}

/// Something that can be converted to `Option<String>`.
/// You probably shouldn't implement this trait yourself, it's here only
/// to simplify annotation creation syntax.
pub trait AnnotationText {
    fn into_option_string(self) -> Option<String>;
}

impl AnnotationText for String {
    fn into_option_string(self) -> Option<String> {
        Some(self)
    }
}

impl AnnotationText for &'_ str {
    fn into_option_string(self) -> Option<String> {
        Some(self.into())
    }
}

impl AnnotationText for Option<String> {
    fn into_option_string(self) -> Option<String> {
        self
    }
}

impl Annotation {
    /// Create new annotation.
    /// Will return [`Error::InvalidRange`] if provided range has `start > end`.
    /// You can pass `&str`, `String` or `Option<String>` as header and text arguments.
    /// ```rust
    /// # use show_my_errors::{Annotation, Severity, Error};
    /// assert_eq!(
    ///     Annotation::new(0..5, Severity::Info, "header", "text").unwrap(),
    ///     Annotation::new(
    ///         0..5, Severity::Info, Some("header".into()), Some("text".into())
    ///     ).unwrap()
    /// );
    /// assert!(Annotation::new(0..5, Severity::Warning, None, None).is_ok());
    /// assert_eq!(
    ///     Annotation::new(5..0, Severity::Info, "h", "t"), Err(Error::InvalidRange(5, 0))
    /// );
    /// ```
    pub fn new(
        range: Range<usize>,
        severity: Severity,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<Self> {
        if range.end < range.start {
            Err(Error::InvalidRange(range.start, range.end))
        } else {
            Ok(Self {
                range,
                severity,
                header: header.into_option_string(),
                text: text.into_option_string(),
            })
        }
    }

    /// Create a new [`Severity::Info`] annotation
    pub fn info(
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<Self> {
        Self::new(range, Severity::Info, header, text)
    }

    /// Create a new [`Severity::Warning`] annotation
    pub fn warning(
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<Self> {
        Self::new(range, Severity::Warning, header, text)
    }

    /// Create a new [`Severity::Error`] annotation
    pub fn error(
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<Self> {
        Self::new(range, Severity::Error, header, text)
    }

    /// Get annotations range
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }
}
