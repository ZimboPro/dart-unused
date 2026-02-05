/// This module contains the parser for Dart files.
/// It provides functions to parse different types of Dart file statements, such as imports, packages, and parts.
/// The main entry point is the `dart_file` function, which takes a string input and returns a parsed `DartFile` enum.
/// The `DartFile` enum represents different types of Dart file statements, including imports, packages, and parts.
/// The module also includes conversion implementations for `TryFrom<&str>` and `TryFrom<&DartFile>`, which allow converting between `&str` and `DartFile` types.
/// Additionally, there are several helper functions used by the parser, such as `quote`, `no_colons_in_input`, and `take_until_quote`.
/// The module also includes unit tests for the parser functions.
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::multispace1,
    combinator::map_res,
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
        match dart_file(value.as_bytes()) {
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
fn quote(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((tag("'"), tag("\""))).parse(input)
}

/// Checks if the input string contains a colon.
fn no_colons_in_input(input: &[u8]) -> IResult<&[u8], &[u8]> {
    if input.contains(&b':') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NoneOf,
        )));
    }
    Ok((b"", input))
}

/// Parses an import statement and returns a `DartFile::Import` variant.
fn import_parser(input: &[u8]) -> IResult<&[u8], DartFile> {
    let (remaining, (_, _, _, path)) =
        (tag("import"), multispace1, quote, take_until_quote).parse(input)?;
    no_colons_in_input(path)?;

    Ok((
        remaining,
        DartFile::Import(String::from_utf8(path.to_vec()).unwrap()),
    ))
}

/// Parses an import statement using the `import_parser` function and converts the result to `DartFile` using `TryFrom`.
fn import(input: &[u8]) -> IResult<&[u8], DartFile> {
    let mut parser = map_res(import_parser, DartFile::try_from);

    parser.parse(input)
}

/// Parses an export statement and returns a `DartFile::Export` variant.
fn export_parser(input: &[u8]) -> IResult<&[u8], DartFile> {
    let (remaining, (_, _, _, path)) =
        (tag("export"), multispace1, quote, take_until_quote).parse(input)?;
    no_colons_in_input(path)?;

    Ok((
        remaining,
        DartFile::Export(String::from_utf8(path.to_vec()).unwrap()),
    ))
}

/// Parses an export statement using the `import_parser` function and converts the result to `DartFile` using `TryFrom`.
fn export(input: &[u8]) -> IResult<&[u8], DartFile> {
    export_parser(input)
}

/// Parses a package statement and returns a `DartFile::Package` variant.
fn package(input: &[u8]) -> IResult<&[u8], DartFile> {
    let (remaining, (_, _, _, _, name, path)) = (
        tag("import"),
        multispace1,
        quote,
        tag("package:"),
        take_until1("/"),
        take_until_quote,
    )
        .parse(input)?;
    Ok((
        remaining,
        DartFile::Package(
            String::from_utf8(name.to_vec()).unwrap(),
            String::from_utf8(path.to_vec()).unwrap(),
        ),
    ))
}

/// Parses a part statement and returns a `DartFile::Part` variant.
fn part(input: &[u8]) -> IResult<&[u8], DartFile> {
    let (remaining, (_, _, _, value)) =
        (tag("part"), multispace1, quote, take_until_quote).parse(input)?;

    Ok((
        remaining,
        DartFile::Part(String::from_utf8(value.to_vec()).unwrap()),
    ))
}

/// Parses a Dart file statement and returns a `DartFile` variant.
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = b"import 'flutter/material.dart';";
/// let expected = DartFile::Import("flutter/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = b"import 'package:flutter/material.dart';";
/// let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = b"part 'material.g.dart';";
/// let expected = DartFile::Part("material.g.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = b"import 'dart:io';";
/// let result = dart_file(input);
/// assert!(result.is_err());
/// ```
///
/// ```rust
/// use dart_unused::parser::{DartFile, dart_file};
///
/// let input = b"import 'flutter/material.dart';";
/// let expected = DartFile::Import("flutter/material.dart".to_string());
/// let result = dart_file(input);
/// assert_eq!(result, Ok(("';", expected)));
/// ```
pub fn dart_file(input: &[u8]) -> IResult<&[u8], DartFile> {
    alt((package, import, part, export)).parse(input)
}

/// Parses a string until a quote is encountered (either single or double quotes).
fn take_until_quote(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((take_until1("'"), take_until1("\""))).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import() {
        let input = b"import 'flutter/material.dart';";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_import_path() {
        let input = b"import './flutter/material.dart';";
        let expected = DartFile::Import("./flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_import_relative() {
        let input = b"import '../flutter/material.dart';";
        let expected = DartFile::Import("../flutter/material.dart".to_string());
        let result = import(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_import_failure() {
        let input = b"import 'dart:io';";
        let result = import(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_package() {
        let input = b"import 'package:flutter/material.dart';";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = package(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_part() {
        let input = b"part 'material.g.dart';";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = part(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_dart_file_import() {
        let input = b"import 'flutter/material.dart';";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_dart_file_package() {
        let input = b"import 'package:flutter/material.dart';";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_dart_file_part() {
        let input = b"part 'material.g.dart';";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("';".as_bytes(), expected)));
    }

    #[test]
    fn test_dart_file_import_error() {
        let input = b"import 'dart:io';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_commented_import() {
        let input = b"// import 'flutter/material.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_with_comment() {
        let input = b"import 'flutter/material.dart'; // comment";
        let expected = DartFile::Import("flutter/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment".as_bytes(), expected)));
    }

    #[test]
    fn test_commented_part() {
        let input = b"// part 'material.g.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_with_comment() {
        let input = b"part 'material.g.dart'; // comment";
        let expected = DartFile::Part("material.g.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment".as_bytes(), expected)));
    }

    #[test]
    fn test_commented_package() {
        let input = b"// import 'package:flutter/material.dart';";
        let result = dart_file(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_package_with_comment() {
        let input = b"import 'package:flutter/material.dart'; // comment";
        let expected = DartFile::Package("flutter".to_string(), "/material.dart".to_string());
        let result = dart_file(input);
        assert_eq!(result, Ok(("'; // comment".as_bytes(), expected)));
    }
}
