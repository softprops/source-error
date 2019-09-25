//! A magical type for displaying source file errors
//!
//! # example
//!
//! ```rust
//! use source_error::{from_file, Position};
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     println!(
//!       "{}",
//!       from_file("whoopsie!", "tests/source.json", Position::new(3, 4))?
//!     );
//!     Ok(())
//! }
//! ```
use colored::Colorize;
use std::{
    error::Error as StdError,
    fmt, io,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

/// Line and column coordinates
#[derive(Debug)]
pub struct Position {
    line: usize,
    col: usize,
}

impl Position {
    /// Return's a new `Position` given a line and column number
    /// These should be 1-based
    pub fn new(
        line: usize,
        col: usize,
    ) -> Position {
        Position { line, col }
    }
}

/// An `Error` type targetting errors tied to source file contents
///
/// Most of the utility of this type is in its implementation of `Display` which
/// renders the error next along with the relative line of code
pub struct Error<S> {
    message: String,
    path: PathBuf,
    lines: S,
    position: Position,
}

impl<S> fmt::Debug for Error<S> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("Error")
            .field("message", &self.message)
            .field("path", &self.path.display())
            .field("position", &self.position)
            .finish()
    }
}

/// Creates an `Error` with a message and path to source file a
/// along with the relative position of the error
///
/// Positions should be  1-based to align with what people see in their
/// source file editors
pub fn from_file<M, P>(
    message: M,
    path: P,
    position: Position,
) -> io::Result<Error<String>>
where
    P: AsRef<Path>,
    M: AsRef<str>,
{
    Ok(from_lines(
        message,
        path.as_ref(),
        std::fs::read_to_string(path.as_ref())?,
        position,
    ))
}

/// Creates an `Error` with a message, path to source file a
/// and source lines along with the relative position of the error
///
/// Positions are 1 based to align with what people see in their
/// source file editors
pub fn from_lines<M, P, S>(
    message: M,
    path: P,
    lines: S,
    position: Position,
) -> Error<S>
where
    M: AsRef<str>,
    P: AsRef<Path>,
    S: AsRef<str>,
{
    Error {
        message: message.as_ref().into(),
        path: path.as_ref().into(),
        lines,
        position,
    }
}

fn line_range(line: usize) -> RangeInclusive<usize> {
    line.checked_sub(2).unwrap_or_default()..=line.checked_add(2).unwrap_or(std::usize::MAX)
}

/// Creates a colorized display of error information
///
/// You can disable color by exporting the `NO_COLOR` environment variable
/// to anything but "0"
impl<S> fmt::Display for Error<S>
where
    S: AsRef<str>,
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let Self {
            message,
            path,
            lines,
            position,
        } = self;
        let Position { line, col } = position;
        let line_range = line_range(*line);
        writeln!(f, "⚠️ {}\n", format!("error: {}", message).red())?;
        writeln!(
            f,
            "   {}\n",
            format!("at {}:{}:{}", path.display(), line, col).dimmed()
        )?;
        let lines = lines
            .as_ref()
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let line_idx = idx + 1;
                if line_range.contains(&line_idx) {
                    Some((line_idx, line))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let max_line = lines
            .last()
            .map(|(idx, _)| idx.to_string().len())
            .unwrap_or_default();
        for (idx, matched) in lines {
            if idx == *line {
                write!(f, "{} ", ">".red())?;
            } else {
                f.write_str("  ")?;
            }
            writeln!(
                f,
                " {}{}",
                format!("{}{} |", " ".repeat(max_line - idx.to_string().len()), idx).dimmed(),
                matched
            )?;
        }
        Ok(())
    }
}

impl<S> StdError for Error<S> where S: AsRef<str> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::env;

    #[test]
    fn impl_error() {
        fn is<E>(_: E)
        where
            E: StdError,
        {
        }
        is(from_lines("..", "...", "", Position::new(1, 1)))
    }

    #[test]
    fn impl_custom_debug() {
        assert_eq!(
            format!(
                "{:?}",
                from_lines(
                    "error occurs here",
                    "path/to/file.txt",
                    ":)",
                    Position::new(1, 1)
                )
            ),
            "Error { message: \"error occurs here\", path: \"path/to/file.txt\", position: Position { line: 1, col: 1 } }"
        )
    }

    #[test]
    fn it_works() {
        env::set_var("NO_COLOR", "");
        let expected = include_str!("../tests/expect.txt");
        let err = from_lines(
            "something is definitely wrong here",
            "../tests/source.json",
            include_str!("../tests/source.json"),
            Position::new(3, 4),
        );
        assert_eq!(format!("{}", err), expected)
    }

    #[test]
    fn line_range_is_expected() {
        for (given, expect) in &[
            (1, (0, 3)),
            (2, (0, 4)),
            (3, (1, 5)),
            (std::usize::MAX, (std::usize::MAX - 2, std::usize::MAX)),
        ] {
            let (start, end) = expect;
            let range = line_range(*given);
            assert_eq!(start, range.start());
            assert_eq!(end, range.end());
        }
    }
}
