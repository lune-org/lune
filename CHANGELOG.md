<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added support for running directories with an `init.luau` or `init.lua` file in them in the CLI

### Changed

- Update to Luau version `0.583`

### Fixed

- Fixed publishing of Lune to crates.io by migrating away from a monorepo
- Fixed crashes when writing a very deeply nested `Instance` to a file ([#62])
- Fixed not being able to read & write to WebSocket objects at the same time ([#68])
- Fixed tab character at the start of a script causing it not to parse correctly ([#72])

[#62]: https://github.com/filiptibell/lune/pull/62
[#68]: https://github.com/filiptibell/lune/pull/66
[#72]: https://github.com/filiptibell/lune/pull/72

## `0.7.4` - July 7th, 2023

### Added

- Added support for `CFrame` and `Font` types in attributes when using the `roblox` builtin.

### Fixed

- Fixed `roblox.serializeModel` still keeping some unique ids.

## `0.7.3` - July 5th, 2023

### Changed

- When using `roblox.serializeModel`, Lune will no longer keep internal unique ids. <br/>
  This is consistent with what Roblox does and prevents Lune from always generating a new and unique file. <br/>
  This previously caused unnecessary diffs when using git or other kinds of source control. ([Relevant issue](https://github.com/filiptibell/lune/issues/61))

## `0.7.2` - June 28th, 2023

### Added

- Added support for `init` files in directories, similar to Rojo, or `index.js` / `mod.rs` in JavaScript / Rust. <br/>
  This means that placing a file named `init.luau` or `init.lua` in a directory will now let you `require` that directory.

### Changed

- The `lune --setup` command is now much more user-friendly
- Update to Luau version `0.581`

## `0.7.1` - June 17th, 2023

### Added

- Added support for TLS in websockets, enabling usage of `wss://`-prefixed URLs. ([#57])

### Fixed

- Fixed `closeCode` erroring when being accessed on websockets. ([#57])
- Fixed issues with `UniqueId` when using the `roblox` builtin by downgrading `rbx-dom`.

[#57]: https://github.com/filiptibell/lune/pull/57

## `0.7.0` - June 12th, 2023

### Breaking Changes

- Globals for the `fs`, `net`, `process`, `stdio`, and `task` builtins have been removed, and the `require("@lune/...")` syntax is now the only way to access builtin libraries. If you have previously been using a global such as `fs` directly, you will now need to put `local fs = require("@lune/fs")` at the top of the file instead.

- Migrated several functions in the `roblox` builtin to new, more flexible APIs:

  - `readPlaceFile -> deserializePlace`
  - `readModelFile -> deserializeModel`
  - `writePlaceFile -> serializePlace`
  - `writeModelFile -> serializeModel`

  These new APIs **_no longer use file paths_**, meaning to use them with files you must first read them using the `fs` builtin.

- Removed `CollectionService` and its methods from the `roblox` builtin library - new instance methods have been added as replacements.
- Removed [`Instance:FindFirstDescendant`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstDescendant) which was a method that was never enabled in the official Roblox API and will soon be removed. <br/>
  Use the second argument of the already existing find methods instead to find descendants.
- Removed the global `printinfo` function - it was generally not used, and did not work as intended. Use the `stdio` builtin for formatting and logging instead.
- Removed support for Windows on ARM - it's more trouble than its worth right now, we may revisit it later.

### Added

- Added `serde.compress` and `serde.decompress` for compressing and decompressing strings using one of several compression formats: `brotli`, `gzip`, `lz4`, or `zlib`.

  Example usage:

  ```lua
  local INPUT = string.rep("Input string to compress", 16) -- Repeated string 16 times for the purposes of this example

  local serde = require("@lune/serde")

  local compressed = serde.compress("gzip", INPUT)
  local decompressed = serde.decompress("gzip", compressed)

  assert(decompressed == INPUT)
  ```

- Added automatic decompression for compressed responses when using `net.request`.
  This behavior can be disabled by passing `options = { decompress = false }` in request params.

- Added support for finding scripts in the current home directory.
  This means that if you have a script called `script-name.luau`, you can place it in the following location:

  - `C:\Users\YourName\.lune\script-name.luau` (Windows)
  - `/Users/YourName/.lune/script-name.luau` (macOS)
  - `/home/YourName/.lune/script-name.luau` (Linux)

  And then run it using `lune script-name` from any directory you are currently in.

- Added several new instance methods in the `roblox` builtin library:
  - [`Instance:AddTag`](https://create.roblox.com/docs/reference/engine/classes/Instance#AddTag)
  - [`Instance:GetTags`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetTags)
  - [`Instance:HasTag`](https://create.roblox.com/docs/reference/engine/classes/Instance#HasTag)
  - [`Instance:RemoveTag`](https://create.roblox.com/docs/reference/engine/classes/Instance#RemoveTag)
- Implemented the second argument of the `FindFirstChild` / `FindFirstChildOfClass` / `FindFirstChildWhichIsA` instance methods.

### Changed

- Update to Luau version `0.579`
- Both `stdio.write` and `stdio.ewrite` now support writing arbitrary bytes, instead of only valid UTF-8.

### Fixed

- Fixed `stdio.write` and `stdio.ewrite` not being flushed and causing output to be interleaved. ([#47])
- Fixed `typeof` returning `userdata` for roblox types such as `Instance`, `Vector3`, ...

[#47]: https://github.com/filiptibell/lune/pull/47

## `0.6.7` - May 14th, 2023

### Added

- Replaced all of the separate typedef & documentation generation commands with a unified `lune --setup` command.

  This command will generate type definition files for all of the builtins and will work with the new `require("@lune/...")` syntax. Note that this also means that there is no longer any way to generate type definitions for globals - this is because they will be removed in the next major release in favor of the beforementioned syntax.

- New releases now include prebuilt binaries for arm64 / aarch64! <br />
  These new binaries will have names with the following format:
  - `lune-windows-0.6.7-aarch64.exe`
  - `lune-linux-0.6.7-aarch64`
  - `lune-macos-0.6.7-aarch64`
- Added global types to documentation site

## `0.6.6` - April 30th, 2023

### Added

- Added tracing / logging for rare and hard to diagnose error cases, which can be configured using the env var `RUST_LOG`.

### Changed

- The `_VERSION` global now follows a consistent format `Lune x.y.z+luau` to allow libraries to check against it for version requirements.

  Examples:

  - `Lune 0.0.0+0`
  - `Lune 1.0.0+500`
  - `Lune 0.11.22+9999`

- Updated to Luau version `0.573`
- Updated `rbx-dom` to support reading and writing `Font` datatypes

### Fixed

- Fixed `_G` not being a readable & writable table
- Fixed `_G` containing normal globals such as `print`, `math`, ...
- Fixed using instances as keys in tables

## `0.6.5` - March 27th, 2023

### Changed

- Functions such as `print`, `warn`, ... now respect `__tostring` metamethods.

### Fixed

- Fixed access of roblox instance properties such as `Workspace.Terrain`, `game.Workspace` that are actually links to child instances. <br />
  These properties are always guaranteed to exist, and they are not always properly set, meaning they must be found through an internal lookup.
- Fixed issues with the `CFrame.lookAt` and `CFrame.new(Vector3, Vector3)` constructors.
- Fixed issues with CFrame math operations returning rotation angles in the wrong order.

## `0.6.4` - March 26th, 2023

### Fixed

- Fixed instances with attributes not saving if they contain integer attributes.
- Fixed attributes not being set properly if the instance has an empty attributes property.
- Fixed error messages for reading & writing roblox files not containing the full error message.
- Fixed crash when trying to access an instance reference property that points to a destroyed instance.
- Fixed crash when trying to save instances that contain unsupported attribute types.

## `0.6.3` - March 26th, 2023

### Added

- Added support for instance tags & `CollectionService` in the `roblox` built-in. <br />
  Currently implemented methods are listed on the [docs site](https://lune.gitbook.io/lune/roblox/api-status).

### Fixed

- Fixed accessing a destroyed instance printing an error message even if placed inside a pcall.
- Fixed cloned instances not having correct instance reference properties set (`ObjectValue.Value`, `Motor6D.Part0`, ...)
- Fixed `Instance::GetDescendants` returning the same thing as `Instance::GetChildren`.

## `0.6.2` - March 25th, 2023

This release adds some new features and fixes for the `roblox` built-in.

### Added

- Added `GetAttribute`, `GetAttributes` and `SetAttribute` methods for instances.
- Added support for getting & setting properties that are instance references.

### Changed

- Improved handling of optional property types such as optional cframes & default physical properties.

### Fixed

- Fixed handling of instance properties that are serialized as binary strings.

## `0.6.1` - March 22nd, 2023

### Fixed

- Fixed `writePlaceFile` and `writeModelFile` in the new `roblox` built-in making mysterious "ROOT" instances.

## `0.6.0` - March 22nd, 2023

### Added

- Added a `roblox` built-in

  If you're familiar with [Remodel](https://github.com/rojo-rbx/remodel), this new built-in contains more or less the same APIs, integrated into Lune. <br />
  There are just too many new APIs to list in this changelog, so head over to the [docs sit](https://lune.gitbook.io/lune/roblox/intro) to learn more!

- Added a `serde` built-in

  This built-in contains previously available functions `encode` and `decode` from the `net` global. <br />
  The plan is for this built-in to contain more serialization and encoding functionality in the future.

- `require` has been reimplemented and overhauled in several ways:

  - New built-ins such as `roblox` and `serde` can **_only_** be imported using `require("@lune/roblox")`, `require("@lune/serde")`, ...
  - Previous globals such as `fs`, `net` and others can now _also_ be imported using `require("@lune/fs")`, `require("@lune/net")`, ...
  - Requiring a script is now completely asynchronous and will not block lua threads other than the caller.
  - Requiring a script will no longer error when using async APIs in the main body of the required script.

  All new built-ins will be added using this syntax and new built-ins will no longer be available in the global scope, and current globals will stay available as globals until proper editor and LSP support is available to ensure Lune users have a good development experience. This is the first step towards moving away from adding each library as a global, and allowing Lune to have more built-in libraries in general.

  Behavior otherwise stays the same, and requires are still relative to file unless the special `@` prefix is used.

- Added `net.urlEncode` and `net.urlDecode` for URL-encoding and decoding strings

### Changed

- Renamed the global `info` function to `printinfo` to make it less ambiguous

### Removed

- Removed experimental `net.encode` and `net.decode` functions, since they are now available using `require("@lune/serde")`
- Removed option to preserve default Luau require behavior

## `0.5.6` - March 11th, 2023

### Added

- Added support for shebangs at the top of a script, meaning scripts such as this one will now run without throwing a syntax error:

  ```lua
  #!/usr/bin/env lune

  print("Hello, world!")
  ```

### Fixed

- Fixed `fs.writeFile` and `fs.readFile` not working with strings / files that are invalid utf-8

## `0.5.5` - March 8th, 2023

### Added

- Added support for running scripts by passing absolute file paths in the CLI
  - This does not have the restriction of scripts having to use the `.luau` or `.lua` extension, since it is presumed that if you pass an absolute path you know exactly what you are doing

### Changed

- Improved error messages for passing invalid file names / file paths substantially - they now include helpful formatting to make file names distinct from file extensions, and give suggestions on how to solve the problem
- Improved general formatting of error messages, both in the CLI and for Luau scripts being run

### Fixed

- Fixed the CLI being a bit too picky about file names when trying to run files in `lune` or `.lune` directories
- Fixed documentation misses from large changes made in version `0.5.0`

## `0.5.4` - March 7th, 2023

### Added

- Added support for reading scripts from stdin by passing `"-"` as the script name
- Added support for close codes in the `net` WebSocket APIs:
  - A close code can be sent by passing it to `socket.close`
  - A received close code can be checked with the `socket.closeCode` value, which is populated after a socket has been closed - note that using `socket.close` will not set the close code value, it is only set when received and is guaranteed to exist after closure

### Changed

- Update to Luau version 0.566

### Fixed

- Fixed scripts having to be valid utf8, they may now use any kind of encoding that base Luau supports
- The `net` WebSocket APIs will no longer return `nil` for partial messages being received in `socket.next`, and will instead wait for the full message to arrive

## `0.5.3` - February 26th, 2023

### Fixed

- Fixed `lune --generate-selene-types` generating an invalid Selene definitions file
- Fixed type definition parsing issues on Windows

## `0.5.2` - February 26th, 2023

### Fixed

- Fixed crash when using `stdio.color()` or `stdio.style()` in a CI environment or non-interactive terminal

## `0.5.1` - February 25th, 2023

### Added

- Added `net.encode` and `net.decode` which are equivalent to `net.jsonEncode` and `net.jsonDecode`, but with support for more formats.

  **_WARNING: Unstable API_**

  _This API is unstable and may change or be removed in the next major version of Lune. The purpose of making a new release with these functions is to gather feedback from the community, and potentially replace the JSON-specific encoding and decoding utilities._

  Example usage:

  ```lua
  local toml = net.decode("toml", [[
  [package]
  name = "my-cool-toml-package"
  version = "0.1.0"

  [values]
  epic = true
  ]])

  assert(toml.package.name == "my-cool-toml-package")
  assert(toml.package.version == "0.1.0")
  assert(toml.values.epic == true)
  ```

### Fixed

- Fixed indentation of closing curly bracket when printing tables

## `0.5.0` - February 23rd, 2023

### Added

- Added auto-generated API reference pages and documentation using GitHub wiki pages
- Added support for `query` in `net.request` parameters, which enables usage of query parameters in URLs without having to manually URL encode values.
- Added a new function `fs.move` to move / rename a file or directory from one path to another.
- Implemented a new task scheduler which resolves several long-standing issues:

  - Issues with yielding across the C-call/metamethod boundary no longer occur when calling certain async APIs that Lune provides.
  - Ordering of interleaved calls to `task.spawn/task.defer` is now completely deterministic, deferring is now guaranteed to run last even in these cases.
  - The minimum wait time possible when using `task.wait` and minimum delay time using `task.delay` are now much smaller, and only limited by the underlying OS implementation. For most systems this means `task.wait` and `task.delay` are now accurate down to about 5 milliseconds or less.

### Changed

- Type definitions are now bundled as part of the Lune executable, meaning they no longer need to be downloaded.
  - `lune --generate-selene-types` will generate the Selene type definitions file, replacing `lune --download-selene-types`
  - `lune --generate-luau-types` will generate the Luau type definitions file, replacing `lune --download-luau-types`
- Improved accuracy of Selene type definitions, strongly typed arrays are now used where possible
- Improved error handling and messages for `net.serve`
- Improved error handling and messages for `stdio.prompt`
- File path representations on Windows now use legacy paths instead of UNC paths wherever possible, preventing some confusing cases where file paths don't work as expected

### Fixed

- Fixed `process.cwd` not having the correct ending path separator on Windows
- Fixed remaining edge cases where the `task` and `coroutine` libraries weren't interoperable
- Fixed `task.delay` keeping the script running even if it was cancelled using `task.cancel`
- Fixed `stdio.prompt` blocking all other lua threads while prompting for input

## `0.4.0` - February 11th, 2023

### Added

- ### Web Sockets

  `net` now supports web sockets for both clients and servers! <br />
  Note that the web socket object is identical on both client and
  server, but how you retrieve a web socket object is different.

  #### Server API

  The server web socket API is an extension of the existing `net.serve` function. <br />
  This allows for serving both normal HTTP requests and web socket requests on the same port.

  Example usage:

  ```lua
  net.serve(8080, {
      handleRequest = function(request)
          return "Hello, world!"
      end,
      handleWebSocket = function(socket)
          task.delay(10, function()
              socket.send("Timed out!")
              socket.close()
          end)
          -- The message will be nil when the socket has closed
          repeat
              local messageFromClient = socket.next()
              if messageFromClient == "Ping" then
                  socket.send("Pong")
              end
          until messageFromClient == nil
      end,
  })
  ```

  #### Client API

  Example usage:

  ```lua
  local socket = net.socket("ws://localhost:8080")

  socket.send("Ping")

  task.delay(5, function()
      socket.close()
  end)

  -- The message will be nil when the socket has closed
  repeat
      local messageFromServer = socket.next()
      if messageFromServer == "Ping" then
          socket.send("Pong")
      end
  until messageFromServer == nil
  ```

### Changed

- `net.serve` now returns a `NetServeHandle` which can be used to stop serving requests safely.

  Example usage:

  ```lua
  local handle = net.serve(8080, function()
      return "Hello, world!"
  end)

  print("Shutting down after 1 second...")
  task.wait(1)
  handle.stop()
  print("Shut down succesfully")
  ```

- The third and optional argument of `process.spawn` is now a global type `ProcessSpawnOptions`.
- Setting `cwd` in the options for `process.spawn` to a path starting with a tilde (`~`) will now use a path relative to the platform-specific home / user directory.
- `NetRequest` query parameters value has been changed to be a table of key-value pairs similar to `process.env`.
  If any query parameter is specified more than once in the request url, the value chosen will be the last one that was specified.
- The internal http client for `net.request` now reuses headers and connections for more efficient requests.
- Refactored the Lune rust crate to be much more user-friendly and documented all of the public functions.

### Fixed

- Fixed `process.spawn` blocking all lua threads if the spawned child process yields.

## `0.3.0` - February 6th, 2023

### Added

- Added a new global `stdio` which replaces `console`
- Added `stdio.write` which writes a string directly to stdout, without any newlines
- Added `stdio.ewrite` which writes a string directly to stderr, without any newlines
- Added `stdio.prompt` which will prompt the user for different kinds of input

  Example usage:

  ```lua
  local text = stdio.prompt()

  local text2 = stdio.prompt("text", "Please write some text")

  local didConfirm = stdio.prompt("confirm", "Please confirm this action")

  local optionIndex = stdio.prompt("select", "Please select an option", { "one", "two", "three" })

  local optionIndices = stdio.prompt(
      "multiselect",
      "Please select one or more options",
      { "one", "two", "three", "four", "five" }
  )
  ```

### Changed

- Migrated `console.setColor/resetColor` and `console.setStyle/resetStyle` to `stdio.color` and `stdio.style` to allow for more flexibility in custom printing using ANSI color codes. Check the documentation for new usage and behavior.
- Migrated the pretty-printing and formatting behavior of `console.log/info/warn/error` to the standard Luau printing functions.

### Removed

- Removed printing functions `console.log/info/warn/error` in favor of regular global functions for printing.

### Fixed

- Fixed scripts hanging indefinitely on error

## `0.2.2` - February 5th, 2023

### Added

- Added global types for networking & child process APIs
  - `net.request` gets `NetFetchParams` and `NetFetchResponse` for its argument and return value
  - `net.serve` gets `NetRequest` and `NetResponse` for the handler function argument and return value
  - `process.spawn` gets `ProcessSpawnOptions` for its third and optional parameter

### Changed

- Reorganize repository structure to take advantage of cargo workspaces, improves compile times

## `0.2.1` - February 3rd, 2023

### Added

- Added support for string interpolation syntax (update to Luau 0.561)
- Added network server functionality using `net.serve`

  Example usage:

  ```lua
  net.serve(8080, function(request)
      print(`Got a {request.method} request at {request.path}!`)

      local data = net.jsonDecode(request.body)

      -- For simple text responses with a 200 status
      return "OK"

      -- For anything else
      return {
          status = 203,
          headers = { ["Content-Type"] = "application/json" },
          body = net.jsonEncode({
              message = "echo",
              data = data,
          })
      }
  end)
  ```

### Changed

- Improved type definitions file for Selene, now including constants like `process.env` + tags such as `readonly` and `mustuse` wherever applicable

### Fixed

- Fixed type definitions file for Selene not including all API members and parameters
- Fixed `process.exit` exiting at the first yield instead of exiting instantly as it should

## `0.2.0` - January 28th, 2023

### Added

- Added full documentation for all global APIs provided by Lune! This includes over 200 lines of pure documentation about behavior & error cases for all of the current 35 constants & functions. Check the [README](/README.md) to find out how to enable documentation in your editor.

- Added a third argument `options` for `process.spawn`:

  - `cwd` - The current working directory for the process
  - `env` - Extra environment variables to give to the process
  - `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
  - `stdio` - How to treat output and error streams from the child process - set to `"inherit"` to pass output and error streams to the current process

- Added `process.cwd`, the path to the current working directory in which the Lune script is running

## `0.1.3` - January 25th, 2023

### Added

- Added a `--list` subcommand to list scripts found in the `lune` or `.lune` directory.

## `0.1.2` - January 24th, 2023

### Added

- Added automatic publishing of the Lune library to [crates.io](https://crates.io/crates/lune)

### Fixed

- Fixed scripts that terminate instantly sometimes hanging

## `0.1.1` - January 24th, 2023

### Fixed

- Fixed errors containing `./` and / or `../` in the middle of file paths
- Potential fix for spawned processes that yield erroring with "attempt to yield across metamethod/c-call boundary"

## `0.1.0` - January 24th, 2023

### Added

- `task` now supports passing arguments in `task.spawn` / `task.delay` / `task.defer`
- `require` now uses paths relative to the file instead of being relative to the current directory, which is consistent with almost all other languages but not original Lua / Luau - this is a breaking change but will allow for proper packaging of third-party modules and more in the future.
  - **_NOTE:_** _If you still want to use the default Lua behavior instead of relative paths, set the environment variable `LUAU_PWD_REQUIRE` to `true`_

### Changed

- Improved error message when an invalid file path is passed to `require`
- Much improved error formatting and stack traces

### Fixed

- Fixed downloading of type definitions making json files instead of the proper format
- Process termination will now always make sure all lua state is cleaned up before exiting, in all cases

## `0.0.6` - January 23rd, 2023

### Added

- Initial implementation of [Roblox's task library](https://create.roblox.com/docs/reference/engine/libraries/task), with some caveats:

  - Minimum wait / delay time is currently set to 10ms, subject to change
  - It is not yet possible to pass arguments to tasks created using `task.spawn` / `task.delay` / `task.defer`
  - Timings for `task.defer` are flaky and deferred tasks are not (yet) guaranteed to run after spawned tasks

  With all that said, everything else should be stable!

  - Mixing and matching the `coroutine` library with `task` works in all cases
  - `process.exit()` will stop all spawned / delayed / deferred threads and exit the process
  - Lune is guaranteed to keep running until there are no longer any waiting threads

  If any of the abovementioned things do not work as expected, it is a bug, please file an issue!

### Fixed

- Potential fix for spawned processes that yield erroring with "attempt to yield across metamethod/c-call boundary"

## `0.0.5` - January 22nd, 2023

### Added

- Added full test suites for all Lune globals to ensure correct behavior
- Added library version of Lune that can be used from other Rust projects

### Changed

- Large internal changes to allow for implementing the `task` library.
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
