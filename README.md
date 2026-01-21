# Dart-unused

This is a tool to scan the Dart project and determine if there are unused files, assets, dependencies etc.

## How does it work

Based on the entry file, it will scan the imports and determine which files and dependencies are referenced and recursively search through all the linked files.

NOTE: This method is not perfect as there are other ways to import packages. To combat this, the tool also checks if the file contains the name of the package. While not perfect it does find some edge cases.

## Features

 - Find unused files
 - Find unused dependencies
 - Find unused assets
 - Find GetIt entries registered but never used
 - Find unused ARB file entries used for localisation

## Areas that can be improved

 - Determine if there are unused imports in the files
    - by analysing the code
    - by using `dart/flutter analyze` and ensure that the unused imports config is enabled
 - Improve check for unused assets
    - Assets path can be assigned to a variable and hence imported but actually never used anywhere in the code
 - Test against Dart/Flutter packages
 - Automatically remove unused files/assets
    - dependencies might just need to move to dev-dependencies, so this might require user input
 - Output as warning instead of erroring
 - Save output to a file
 - Available as a Github Action
 - Specify config
 - Handle flavours
 - Able to specify entry file

## Usage

dart-unused [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>
          Path to the Dart project

  -a, --assets
          Check for unreferenced assets

  -d, --deps
          Check for unreferenced dependencies

  -l, --labels
          Check for unused arb file(s) entries

      --loc
          List items registered in locator but not used

  -h, --help
          Print help (see a summary with '-h')

## Why Rust

Rust has great libraries for creating custom parsers and is really performant. While developing this and testing it against a Flutter project with over 6100 files, it managed to complete it in less than 1 second with all the flags enabled.

It would be possible to do the same in Dart using the `Analyzer` package, but the performance would be not great.
