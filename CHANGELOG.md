# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added full test suite for all Lune globals to ensure correct behavior
- Added library version of Lune that can be used from other Rust projects

### Changed

- Improved general formatting of errors to make them more readable & glanceable
- Improved output formatting of non-primitive types
- Improved output formatting of empty tables

### Fixed

- Fixed double stack trace for certain kinds of errors

## `0.0.4` - January 21st, 2023

### Added

- Added `process.args` for inspecting values given to Lune when running (read only)
- Added `process.env` which is a plain table where you can get & set environment variables

### Changed

- Improved error formatting & added proper file name to stack traces

### Removed

- Removed `...` for process arguments, use `process.args` instead
- Removed individual functions for getting & setting environment variables, use `process.env` instead

## `0.0.3` - January 20th, 2023

### Added

- Added networking functions under `net`

  Example usage:

  ```lua
  local apiResult = net.request({
  	url = "https://jsonplaceholder.typicode.com/posts/1",
  	method = "PATCH",
  	headers = {
  		["Content-Type"] = "application/json",
  	},
  	body = net.jsonEncode({
  		title = "foo",
  		body = "bar",
  	}),
  })

  local apiResponse = net.jsonDecode(apiResult.body)
  assert(apiResponse.title == "foo", "Invalid json response")
  assert(apiResponse.body == "bar", "Invalid json response")
  ```

- Added console logging & coloring functions under `console`

  This piece of code:

  ```lua
  local tab = { Integer = 1234, Hello = { "World" } }
  console.log(tab)
  ```

  Will print the following formatted text to the console, **_with syntax highlighting_**:

  ```lua
  {
      Integer = 1234,
      Hello = {
          "World",
      }
  }
  ```

  Additional utility functions exist with the same behavior but that also print out a colored
  tag together with any data given to them: `console.info`, `console.warn`, `console.error` -
  These print out prefix tags `[INFO]`, `[WARN]`, `[ERROR]` in blue, orange, and red, respectively.

### Changed

- The `json` api is now part of `net`
  - `json.encode` becomes `net.jsonEncode`
  - `json.decode` become `net.jsonDecode`

### Fixed

- Fixed JSON decode not working properly

## `0.0.2` - January 19th, 2023

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

## `0.0.1` - January 18th, 2023

Initial Release
