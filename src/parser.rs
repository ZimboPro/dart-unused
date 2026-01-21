/// This module contains the parser for Dart files.
/// It provides functions to parse different types of Dart file statements, such as imports, packages, and parts.
/// The main entry point is the `dart_file` function, which takes a string input and returns a parsed `DartFile` enum.
/// The `DartFile` enum represents different types of Dart file statements, including imports, packages, and parts.
/// The module also includes conversion implementations for `TryFrom<&str>` and `TryFrom<&DartFile>`, which allow converting between `&str` and `DartFile` types.
/// Additionally, there are several helper functions used by the parser, such as `quote`, `no_colons_in_input`, and `take_until_quote`.
/// The module also includes unit tests for the parser functions.
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::multispace1,
    combinator::map_res,
    sequence::tuple,
};

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum DartFile {
    Import(String),
    Package(String, String),
    Part(String),
    Export(String),
}

impl TryFrom<&str> for DartFile {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match dart_file(value) {
            Ok((_, dart)) => {
                if let DartFile::Import(path) = &dart
                    && path.contains(":")
                {
                    return Err("Package imports are not supported");
                }
                Ok(dart)
            }
            Err(_) => Err("Failed to parse dart file"),
        }
    }
}

impl TryFrom<&DartFile> for DartFile {
    type Error = &'static str;

    fn try_from(value: &DartFile) -> Result<Self, Self::Error> {
        log::info!("Parsing: {:?}", value);
        if let DartFile::Import(path) = &value
            && path.contains(":")
        {
            return Err("Package imports are not supported");
        }
        Ok(value.clone())
    }
}

/// Parses a single or multiple quotes (either single or double quotes).
fn quote(input: &str) -> IResult<&str, &str> {
    alt((tag("'"), tag("\"")))(input)
}

/// Checks if the input string contains a colon.
fn no_colons_in_input(input: &str) -> IResult<&str, &str> {
    if input.contains(":") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NoneOf,
        )));
    }
    Ok(("", input))
}

/// Parses an import statement and returns a `DartFile::Import` variant.
fn import_parser(input: &str) -> IResult<&str, DartFile> {
    let (remaining, (_, _, _, path)) =
        tuple((tag("import"), multispace1, quote, take_until_quote))(input)?;
    no_colons_in_input(path)?;

    Ok((remaining, DartFile::Import(path.to_string())))
}

/// Parses an import statement using the `import_parser` function and converts the result to `DartFile` using `TryFrom`.
fn import(input: &str) -> IResult<&str, DartFile> {
    let mut parser = map_res(import_parser, DartFile::try_from);

    parser(input)
}

/// Parses an export statement and returns a `DartFile::Export` variant.
fn export_parser(input: &str) -> IResult<&str, DartFile> {
    let (remaining, (_, _, _, path)) =
        tuple((tag("export"), multispace1, quote, take_until_quote))(input)?;
    no_colons_in_input(path)?;

    Ok((remaining, DartFile::Export(path.to_string())))
}

/// Parses an export statement using the `import_parser` function and converts the result to `DartFile` using `TryFrom`.
fn export(input: &str) -> IResult<&str, DartFile> {
    export_parser(input)
}

/// Parses a package statement and returns a `DartFile::Package` variant.
fn package(input: &str) -> IResult<&str, DartFile> {
    let (remaining, (_, _, _, _, name, path)) = tuple((
        tag("import"),
        multispace1,
        quote,
        tag("package:"),
        take_until1("/"),
        take_until_quote,
    ))(input)?;
    Ok((
        remaining,
        DartFile::Package(name.to_string(), path.to_string()),
    ))
}

/// Parses a part statement and returns a `DartFile::Part` variant.
fn part(input: &str) -> IResult<&str, DartFile> {
    let (remaining, (_, _, _, value)) =
        tuple((tag("part"), multispace1, quote, take_until_quote))(input)?;

    Ok((remaining, DartFile::Part(value.to_string())))
}

/// Parses a Dart file statement and returns a `DartFile` variant.
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = "import 'flutter/material.dart';";
/// let expected = DartFile::Import("flutter/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = "import 'package:flutter/material.dart';";
/// let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = "part 'material.g.dart';";
/// let expected = DartFile::Part("material.g.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = "import 'dart:io';";
/// let result = dart_file(input);
/// assert!(result.is_err());
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = "import 'flutter/material.dart';";
/// let expected = DartFile::Import("flutter/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
pub fn dart_file(input: &str) -> IResult<&str, DartFile> {
    alt((package, import, part, export))(input)
}

/// Parses a string until a quote is encountered (either single or double quotes).
fn take_until_quote(input: &str) -> IResult<&str, &str> {
    alt((take_until1("'"), take_until1("\"")))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import() {
        let input = "import 'flutter/material.dart';";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_import_path() {
        let input = "import './flutter/material.dart';";
        let expected = DartFile::Import("./flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_import_relative() {
        let input = "import '../flutter/material.dart';";
        let expected = DartFile::Import("../flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_import_failure() {
        let input = "import 'dart:io';";
        let result = import(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_package() {
        let input = "import 'package:flutter/material.dart';";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = package(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_part() {
        let input = "part 'material.g.dart';";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = part(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_dart_file_import() {
        let input = "import 'flutter/material.dart';";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_dart_file_package() {
        let input = "import 'package:flutter/material.dart';";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_dart_file_part() {
        let input = "part 'material.g.dart';";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';", expected)));
    }

    #[test]
    fn test_dart_file_import_error() {
        let input = "import 'dart:io';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_commented_import() {
        let input = "// import 'flutter/material.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_with_comment() {
        let input = "import 'flutter/material.dart'; // comment";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment", expected)));
    }

    #[test]
    fn test_commented_part() {
        let input = "// part 'material.g.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_with_comment() {
        let input = "part 'material.g.dart'; // comment";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment", expected)));
    }

    #[test]
    fn test_commented_package() {
        let input = "// import 'package:flutter/material.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_package_with_comment() {
        let input = "import 'package:flutter/material.dart'; // comment";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment", expected)));
    }
}
