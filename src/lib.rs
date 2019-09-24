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
    ops::RangeInclusive,
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
    ///
    /// Positions are 1 based to align with what people see in their
    /// source file editors
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

fn line_range(line: usize) -> RangeInclusive<usize> {
    line.checked_sub(2).unwrap_or_default()..=line.checked_add(2).unwrap_or_else(|| std::usize::MAX)
}

/// Creates a colorized display of error information
///
/// You can disable color by exporting the `NO_COLOR` environment variable
/// to anything but "0"
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
        let line_range = line_range(*line);
        writeln!(f, "⚠️ {}\n", format!("error: {}", message).red())?;
        writeln!(
            f,
            "{}",
            format!("at {}:{}:{}", path.display(), line, col).dimmed()
        )?;
        let lines = src
            .clone()
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
