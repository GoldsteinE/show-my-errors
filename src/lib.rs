//! This is a library to display simple errors in colorful, rustc-like way.
//! It can't show multi-line errors or draw arrows between parts of code, but its interface
//! is simple and easy to use. If you want something more complex, you probably should use
//! [annotate-snippets](https://docs.rs/annotate-snippets), which is used by rustc itself.
//!
//! ## Basic usage
//! Entry point of this library is [`AnnotationList`]. You should create it, add some errors
//! and then use [`.show_stderr()`](AnnotationList::show_stderr) or
//! [`.show_stdout()`](AnnotationList::show_stdout)
//! with some [`Stylesheet`] to display the message.
//! ```rust
//! # use std::error::Error;
//! # use show_my_errors::AnnotationList;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let mut list = AnnotationList::new("hello.txt", "Hello world!");
//! list
//!     .warning(4..7, "punctuation problem", "you probably forgot a comma")?
//!     .info(0..0, "consider adding some translations", None)?;
//! assert_eq!(list.to_string()?, r#"warning: punctuation problem
//!   --> hello.txt:1:5
//!    |
//!  1 | Hello world!
//!    |     ^^^ you probably forgot a comma
//!
//! info: consider adding some translations
//!   --> hello.txt:1:1
//!    |
//!  1 | Hello world!
//!    |
//! "#);
//! # Ok(())
//! # }
//! ```

use std::{
    io::{self, Write},
    iter,
    ops::Range,
};
use termcolor::{BufferWriter, ColorChoice, WriteColor};
use thiserror::Error;

mod annotation;
pub use annotation::{Annotation, AnnotationText, Severity};

mod stylesheet;
pub use stylesheet::Stylesheet;

#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
/// Errors that can occure while constructing [`AnnotationList`]. Fields of each variant are the
/// start and the end of range, respectively.
pub enum Error {
    /// Provided annotation range crosses line boundary
    #[error("range {0} .. {1} crosses line boundary")]
    MultilineRange(usize, usize),
    /// Range `end` is greater than its `start`
    #[error("range {0} .. {1} is invalid: {1} < {0}")]
    InvalidRange(usize, usize),
    /// Range starts after last line end
    #[error("range {0} .. {1} starts after last line end")]
    AfterStringEnd(usize, usize),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
#[doc(hidden)]
pub struct AnnotatedLine<'a> {
    start: usize,
    content: &'a str,
    annotations: Vec<Annotation>,
}

impl AnnotatedLine<'_> {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    pub fn content(&self) -> &str {
        self.content
    }

    pub fn add(&mut self, annotation: Annotation) -> Result<&mut Self> {
        let range = annotation.range();
        if range.end - range.start > self.content.len() {
            Err(Error::MultilineRange(range.start, range.end))
        } else {
            self.annotations.push(annotation);
            Ok(self)
        }
    }
}

/// List of annotations applied to some input string.
/// Doesn't owns string, so has a limited lifetime.
#[derive(Debug, PartialEq, Eq)]
pub struct AnnotationList<'a> {
    lines: Vec<AnnotatedLine<'a>>,
    filename: String,
}

impl<'a> AnnotationList<'a> {
    /// Create an annotation list from string. `filename` is used only to format messages, so
    /// corresponding file doesn't need to exist.
    pub fn new(filename: impl AsRef<str>, string: &'a str) -> Self {
        let linebreaks: Vec<_> = iter::once(0)
            .chain(
                string
                    .chars()
                    .enumerate()
                    .filter(|(_idx, c)| *c == '\n')
                    .flat_map(|(idx, _c)| iter::once(idx).chain(iter::once(idx + 1))),
            )
            .chain(iter::once(string.len()))
            .collect();
        let lines = linebreaks
            .chunks(2)
            .map(|bounds| AnnotatedLine {
                start: bounds[0],
                content: &string[bounds[0]..bounds[1]],
                annotations: vec![],
            })
            .collect();
        Self {
            filename: filename.as_ref().into(),
            lines,
        }
    }

    #[doc(hidden)]
    pub fn annotated_lines(&self) -> &[AnnotatedLine] {
        &self.lines
    }

    /// Add an [`Annotation`] to list. You may also use [`.info()`](AnnotationList::info),
    /// [`.warning()`](AnnotationList::warning) and [`.error()`](AnnotationList::error) methods.
    pub fn add(&mut self, annotation: Annotation) -> Result<&mut Self> {
        let range = annotation.range();
        let line_idx = match self
            .lines
            .binary_search_by(|line| line.start.cmp(&range.start))
        {
            Ok(idx) => idx,
            Err(idx) if idx > 0 => idx - 1,
            _ => unreachable!("lines in AnnotationList not starting at 0"),
        };
        let line = &mut self.lines[line_idx];
        if range.start >= line.start() + line.content.len() {
            Err(Error::AfterStringEnd(range.start, range.end))
        } else {
            self.lines[line_idx].add(annotation)?;
            Ok(self)
        }
    }

    /// Add an [`Severity::Info`] annotation to list. See [`Annotation::new`] docs for details
    pub fn info(
        &mut self,
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<&mut Self> {
        self.add(Annotation::info(range, header, text)?)
    }

    /// Add an [`Severity::Warning`] annotation to list. See [`Annotation::new`] docs for details
    pub fn warning(
        &mut self,
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<&mut Self> {
        self.add(Annotation::warning(range, header, text)?)
    }

    /// Add an [`Severity::Error`] annotation to list. See [`Annotation::new`] docs for details
    pub fn error(
        &mut self,
        range: Range<usize>,
        header: impl AnnotationText,
        text: impl AnnotationText,
    ) -> Result<&mut Self> {
        self.add(Annotation::error(range, header, text)?)
    }

    /// Print an error message to stream using given stylesheet. If your stream implements
    /// [`Write`](std::io::Write), but not [`WriteColor`](termcolor::WriteColor), consider wrapping
    /// it into [`termcolor::Ansi`] or [`termcolor::NoColor`].
    ///
    /// This method uses no buffering, so you probably want to pass [`termcolor::Buffer`] to it
    /// rather than raw stream.
    ///
    /// If you want to just print message to stdout/stderr, consider using
    /// [`.print_stdout()`](AnnotationList::show_stdout) or
    /// [`.print_stderr()`](AnnotationList::show_stderr) instead.
    pub fn show<W: Write + WriteColor>(
        &self,
        mut stream: W,
        stylesheet: &Stylesheet,
    ) -> io::Result<()> {
        let mut first_output = true;
        for (idx, line) in self.lines.iter().enumerate() {
            for annotation in line.annotations() {
                let range = annotation.range();

                // Padding
                if first_output {
                    first_output = false;
                } else {
                    stream.write(b"\n")?;
                }

                // Severity and header
                let severity_color = stylesheet.by_severity(&annotation.severity);
                stream.set_color(severity_color)?;
                write!(stream, "{}:", annotation.severity)?;
                if let Some(header) = &annotation.header {
                    write!(stream, " {}\n", header)?;
                } else {
                    stream.write(b"\n")?;
                }

                // Line numbers column & filename
                stream.set_color(&stylesheet.linenr)?;
                let linenr = (idx + 1).to_string();
                let nrcol_width = linenr.len() + 2;
                print_n(&mut stream, b" ", linenr.len() + 1)?;
                write!(stream, "--> ")?;
                stream.set_color(&stylesheet.filename)?;
                write!(
                    stream,
                    "{}:{}:{}\n",
                    self.filename,
                    idx + 1,
                    range.start - line.start() + 1
                )?;
                stream.set_color(&stylesheet.linenr)?;
                print_n(&mut stream, b" ", nrcol_width)?;
                write!(stream, "|\n {} | ", idx + 1)?;

                // Line content
                stream.set_color(&stylesheet.content)?;
                write!(stream, "{}\n", line.content)?;

                // Line numbers column
                stream.set_color(&stylesheet.linenr)?;
                print_n(&mut stream, b" ", nrcol_width)?;
                stream.write(b"|")?;

                // Annotation
                if range.end - range.start != 0 {
                    stream.set_color(severity_color)?;
                    print_n(&mut stream, b" ", range.start - line.start + 1)?;
                    print_n(&mut stream, b"^", range.end - range.start)?;
                    if let Some(text) = &annotation.text {
                        write!(stream, " {}", text)?;
                    }
                }
                stream.write(b"\n")?;
                stream.reset()?;
            }
        }
        Ok(())
    }

    fn show_bufwriter(&self, stream: BufferWriter, stylesheet: &Stylesheet) -> io::Result<()> {
        let mut buf = stream.buffer();
        self.show(&mut buf, stylesheet)?;
        stream.print(&buf)
    }

    /// Print error message to stdout. Output will be colorized if stdout is a TTY
    pub fn show_stdout(&self, stylesheet: &Stylesheet) -> io::Result<()> {
        let color_choice = if atty::is(atty::Stream::Stdout) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        self.show_bufwriter(termcolor::BufferWriter::stdout(color_choice), stylesheet)
    }

    /// Print error message to stderr. Output will be colorized if stderr is a TTY
    pub fn show_stderr(&self, stylesheet: &Stylesheet) -> io::Result<()> {
        let color_choice = if atty::is(atty::Stream::Stderr) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        self.show_bufwriter(termcolor::BufferWriter::stderr(color_choice), stylesheet)
    }

    /// "Print" monochrome message to `Vec<u8>`
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = termcolor::Buffer::no_color();
        self.show(&mut buf, &Stylesheet::monochrome())?;
        Ok(buf.into_inner())
    }

    /// "Print" message to `Vec<u8>`, colorizing it using ANSI escape codes
    pub fn to_ansi_bytes(&self, stylesheet: &Stylesheet) -> io::Result<Vec<u8>> {
        let mut buf = termcolor::Buffer::ansi();
        self.show(&mut buf, stylesheet)?;
        Ok(buf.into_inner())
    }

    /// "Print" monochrome message to [`String`]
    /// # Panics
    /// Panics if message cannot be converted to UTF-8
    pub fn to_string(&self) -> io::Result<String> {
        Ok(String::from_utf8(self.to_bytes()?).expect("invalid utf-8 in AnnotationList"))
    }

    /// "Print" message to [`String`], colorizing it using ANSI escape codes
    /// # Panics
    /// Panics if message cannot be converted to UTF-8
    pub fn to_ansi_string(&self, stylesheet: &Stylesheet) -> io::Result<String> {
        Ok(String::from_utf8(self.to_ansi_bytes(stylesheet)?)
            .expect("invalid utf-8 in AnnotationList"))
    }
}

fn print_n(mut stream: impl io::Write, buf: &[u8], count: usize) -> io::Result<()> {
    for _ in 0..count {
        stream.write(buf)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_start_content<'a>(line: &AnnotatedLine<'a>, start: usize, content: &'a str) {
        assert_eq!(line.start(), start);
        assert_eq!(line.content(), content);
    }

    fn create_list() -> AnnotationList<'static> {
        AnnotationList::new("test.txt", "\nstring\nwith\nmany\n\nnewlines\n\n")
    }

    #[test]
    fn test_new_annotation_list() {
        let annotation_list = create_list();
        let mut lines = annotation_list.annotated_lines().iter();
        assert_start_content(lines.next().unwrap(), 0, "");
        assert_start_content(lines.next().unwrap(), 1, "string");
        assert_start_content(lines.next().unwrap(), 8, "with");
        assert_start_content(lines.next().unwrap(), 13, "many");
        assert_start_content(lines.next().unwrap(), 18, "");
        assert_start_content(lines.next().unwrap(), 19, "newlines");
        assert_start_content(lines.next().unwrap(), 28, "");
        assert_start_content(lines.next().unwrap(), 29, "");
        assert!(lines.next().is_none());
    }

    #[test]
    fn test_add() -> Result<()> {
        let ann1 = Annotation::info(1..3, "test1", "ann1")?;
        let ann2 = Annotation::warning(13..17, "test2", "ann2")?;
        let ann3 = Annotation::error(19..20, "test3", None)?;
        let ann4 = Annotation::error(14..16, "test4", "ann4")?;

        let mut list = create_list();
        list.add(ann1.clone())?
            .add(ann2.clone())?
            .add(ann3.clone())?
            .add(ann4.clone())?;

        let mut other_option = create_list();
        other_option
            .info(1..3, "test1", "ann1")?
            .warning(13..17, "test2", "ann2")?
            .error(19..20, "test3", None)?
            .error(14..16, "test4", "ann4")?;
        assert_eq!(list, other_option);

        for (idx, line) in list.annotated_lines().iter().enumerate() {
            match idx {
                1 => assert_eq!(line.annotations(), &[ann1.clone()]),
                3 => assert_eq!(line.annotations(), &[ann2.clone(), ann4.clone()]),
                5 => assert_eq!(line.annotations(), &[ann3.clone()]),
                _ => assert_eq!(line.annotations(), &[]),
            }
        }
        Ok(())
    }

    #[test]
    fn test_invalid_adds() -> Result<()> {
        let mut list = create_list();
        assert_eq!(
            list.add(Annotation::info(1..10, "test", "ann")?)
                .unwrap_err(),
            Error::MultilineRange(1, 10)
        );
        assert_eq!(
            list.add(Annotation::info(1000..1001, "test", "ann")?)
                .unwrap_err(),
            Error::AfterStringEnd(1000, 1001)
        );
        assert_eq!(
            Annotation::info(10..9, "test", "ann").unwrap_err(),
            Error::InvalidRange(10, 9)
        );
        Ok(())
    }

    #[test]
    fn test_to_string() -> Result<()> {
        let mut list = create_list();
        list.info(1..3, "test1", "ann1")?
            .warning(13..17, "test2", "ann2")?
            .error(19..20, "test3", None)?
            .error(14..16, "test4", "ann4")?
            .error(14..16, None, "ann5")?;
        let result = r#"info: test1
  --> test.txt:2:1
   |
 2 | string
   | ^^ ann1

warning: test2
  --> test.txt:4:1
   |
 4 | many
   | ^^^^ ann2

error: test4
  --> test.txt:4:2
   |
 4 | many
   |  ^^ ann4

error:
  --> test.txt:4:2
   |
 4 | many
   |  ^^ ann5

error: test3
  --> test.txt:6:1
   |
 6 | newlines
   | ^
"#;
        assert_eq!(list.to_string().unwrap(), result);
        Ok(())
    }
}
