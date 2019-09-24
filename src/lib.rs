//! a magical type for displaying source file errors
//!
//! # example
//!
//! ```rust
//! //todo
//! ```
use colored::Colorize;
use std::{
    error::Error as StdError,
    fmt,
    path::{Path, PathBuf},
    str::Lines,
};

/// An `Error` type targetting errors tied to source file contents
///
/// Most of the utility of this type is in its implementation of `Display` which
/// renders the error next along with the relative line of code
#[derive(Debug)]
pub struct Error<'a> {
    message: String,
    path: PathBuf,
    src: Lines<'a>,
    position: (usize, usize),
}

impl<'a> Error<'a> {
    /// Creates an `Error` with a message, path to file a
    /// nd src lines along with the relative position of the error
    pub fn new<M, P>(
        message: M,
        path: P,
        src: Lines<'a>,
        position: (usize, usize),
    ) -> Self
    where
        M: AsRef<str>,
        P: AsRef<Path>,
    {
        Error {
            message: message.as_ref().into(),
            path: path.as_ref().into(),
            src,
            position,
        }
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let Self {
            message,
            path,
            src,
            position,
        } = self;
        let (line, col) = position;
        let line_range = line - 3..=*line + 1;
        writeln!(f, "⚠️ {}\n", format!("error: {}", message).red())?;
        writeln!(
            f,
            "{}",
            format!("at {}:{}:{}", path.display(), line, col).dimmed()
        )?;
        let lines = src
            .clone()
            .enumerate()
            .filter(|(idx, _)| line_range.contains(idx))
            .collect::<Vec<_>>();
        let max_line = lines
            .last()
            .map(|(idx, _)| idx.to_string().len())
            .unwrap_or_default();
        for (idx, matched) in lines {
            if idx == line - 1 {
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

impl<'a> StdError for Error<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::env;

    #[test]
    fn it_impl_error() {
        fn is<E>(_: E)
        where
            E: StdError,
        {
        }
        is(Error::new("..", "...", "".lines(), (0, 0)))
    }

    #[test]
    fn it_works() {
        env::set_var("NO_COLOR", "");
        let expected = include_str!("../tests/expect.txt");
        let err = Error::new(
            "something is definitely wrong here",
            "../tests/source.json",
            include_str!("../tests/source.json").lines(),
            (3, 4),
        );
        println!("{}", err);
        assert_eq!(format!("{}", err), expected)
    }
}
