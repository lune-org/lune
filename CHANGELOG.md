# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added support for command-line parameters to scripts

  These can be accessed as a vararg in the root of a script:

  ```lua
  local firstArg: string, secondArg: string = ...
  print(firstArg, secondArg)
  ```

- Added CLI parameters for downloading type definitions:

  - `lune --download-selene-types` to download Selene types to the current directory
  - `lune --download-luau-types` to download Luau types to the current directory

  These files will be downloaded as `lune.yml` and `luneTypes.d.luau`
  respectively and are also available in each release on GitHub.

## `0.0.1` - January 19th, 2023

Initial Release
