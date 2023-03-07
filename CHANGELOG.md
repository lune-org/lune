<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.5.4` - March 7th, 2023

### Added

-   Added support for reading scripts from stdin by passing `"-"` as the script name
-   Added support for close codes in the `net` WebSocket APIs:
    -   A close code can be sent by passing it to `socket.close`
    -   A received close code can be checked with the `socket.closeCode` value, which is populated after a socket has been closed - note that using `socket.close` will not set the close code value, it is only set when received and is guaranteed to exist after closure

### Changed

-   Update to Luau version 0.566

### Fixed

-   Fixed scripts having to be valid utf8, they may now use any kind of encoding that base Luau supports
-   The `net` WebSocket APIs will no longer return `nil` for partial messages being received in `socket.next`, and will instead wait for the full message to arrive

## `0.5.3` - February 26th, 2023

### Fixed

-   Fixed `lune --generate-selene-types` generating an invalid Selene definitions file
-   Fixed type definition parsing issues on Windows

## `0.5.2` - February 26th, 2023

### Fixed

-   Fixed crash when using `stdio.color()` or `stdio.style()` in a CI environment or non-interactive terminal

## `0.5.1` - February 25th, 2023

### Added

-   Added `net.encode` and `net.decode` which are equivalent to `net.jsonEncode` and `net.jsonDecode`, but with support for more formats.

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

-   Fixed indentation of closing curly bracket when printing tables

## `0.5.0` - February 23rd, 2023

### Added

-   Added auto-generated API reference pages and documentation using GitHub wiki pages
-   Added support for `query` in `net.request` parameters, which enables usage of query parameters in URLs without having to manually URL encode values.
-   Added a new function `fs.move` to move / rename a file or directory from one path to another.
-   Implemented a new task scheduler which resolves several long-standing issues:

    -   Issues with yielding across the C-call/metamethod boundary no longer occur when calling certain async APIs that Lune provides.
    -   Ordering of interleaved calls to `task.spawn/task.defer` is now completely deterministic, deferring is now guaranteed to run last even in these cases.
    -   The minimum wait time possible when using `task.wait` and minimum delay time using `task.delay` are now much smaller, and only limited by the underlying OS implementation. For most systems this means `task.wait` and `task.delay` are now accurate down to about 5 milliseconds or less.

### Changed

-   Type definitions are now bundled as part of the Lune executable, meaning they no longer need to be downloaded.
    -   `lune --generate-selene-types` will generate the Selene type definitions file, replacing `lune --download-selene-types`
    -   `lune --generate-luau-types` will generate the Luau type definitions file, replacing `lune --download-luau-types`
-   Improved accuracy of Selene type definitions, strongly typed arrays are now used where possible
-   Improved error handling and messages for `net.serve`
-   Improved error handling and messages for `stdio.prompt`
-   File path representations on Windows now use legacy paths instead of UNC paths wherever possible, preventing some confusing cases where file paths don't work as expected

### Fixed

-   Fixed `process.cwd` not having the correct ending path separator on Windows
-   Fixed remaining edge cases where the `task` and `coroutine` libraries weren't interoperable
-   Fixed `task.delay` keeping the script running even if it was cancelled using `task.cancel`
-   Fixed `stdio.prompt` blocking all other lua threads while prompting for input

## `0.4.0` - February 11th, 2023

### Added

-   ### Web Sockets

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

-   `net.serve` now returns a `NetServeHandle` which can be used to stop serving requests safely.

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

-   The third and optional argument of `process.spawn` is now a global type `ProcessSpawnOptions`.
-   Setting `cwd` in the options for `process.spawn` to a path starting with a tilde (`~`) will now use a path relative to the platform-specific home / user directory.
-   `NetRequest` query parameters value has been changed to be a table of key-value pairs similar to `process.env`.
    If any query parameter is specified more than once in the request url, the value chosen will be the last one that was specified.
-   The internal http client for `net.request` now reuses headers and connections for more efficient requests.
-   Refactored the Lune rust crate to be much more user-friendly and documented all of the public functions.

### Fixed

-   Fixed `process.spawn` blocking all lua threads if the spawned child process yields.

## `0.3.0` - February 6th, 2023

### Added

-   Added a new global `stdio` which replaces `console`
-   Added `stdio.write` which writes a string directly to stdout, without any newlines
-   Added `stdio.ewrite` which writes a string directly to stderr, without any newlines
-   Added `stdio.prompt` which will prompt the user for different kinds of input

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

-   Migrated `console.setColor/resetColor` and `console.setStyle/resetStyle` to `stdio.color` and `stdio.style` to allow for more flexibility in custom printing using ANSI color codes. Check the documentation for new usage and behavior.
-   Migrated the pretty-printing and formatting behavior of `console.log/info/warn/error` to the standard Luau printing functions.

### Removed

-   Removed printing functions `console.log/info/warn/error` in favor of regular global functions for printing.

### Fixed

-   Fixed scripts hanging indefinitely on error

## `0.2.2` - February 5th, 2023

### Added

-   Added global types for networking & child process APIs
    -   `net.request` gets `NetFetchParams` and `NetFetchResponse` for its argument and return value
    -   `net.serve` gets `NetRequest` and `NetResponse` for the handler function argument and return value
    -   `process.spawn` gets `ProcessSpawnOptions` for its third and optional parameter

### Changed

-   Reorganize repository structure to take advantage of cargo workspaces, improves compile times

## `0.2.1` - February 3rd, 2023

### Added

-   Added support for string interpolation syntax (update to Luau 0.561)
-   Added network server functionality using `net.serve`

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

-   Improved type definitions file for Selene, now including constants like `process.env` + tags such as `readonly` and `mustuse` wherever applicable

### Fixed

-   Fixed type definitions file for Selene not including all API members and parameters
-   Fixed `process.exit` exiting at the first yield instead of exiting instantly as it should

## `0.2.0` - January 28th, 2023

### Added

-   Added full documentation for all global APIs provided by Lune! This includes over 200 lines of pure documentation about behavior & error cases for all of the current 35 constants & functions. Check the [README](/README.md) to find out how to enable documentation in your editor.

-   Added a third argument `options` for `process.spawn`:

    -   `cwd` - The current working directory for the process
    -   `env` - Extra environment variables to give to the process
    -   `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
    -   `stdio` - How to treat output and error streams from the child process - set to `"inherit"` to pass output and error streams to the current process

-   Added `process.cwd`, the path to the current working directory in which the Lune script is running

## `0.1.3` - January 25th, 2023

### Added

-   Added a `--list` subcommand to list scripts found in the `lune` or `.lune` directory.

## `0.1.2` - January 24th, 2023

### Added

-   Added automatic publishing of the Lune library to [crates.io](https://crates.io/crates/lune)

### Fixed

-   Fixed scripts that terminate instantly sometimes hanging

## `0.1.1` - January 24th, 2023

### Fixed

-   Fixed errors containing `./` and / or `../` in the middle of file paths
-   Potential fix for spawned processes that yield erroring with "attempt to yield across metamethod/c-call boundary"

## `0.1.0` - January 24th, 2023

### Added

-   `task` now supports passing arguments in `task.spawn` / `task.delay` / `task.defer`
-   `require` now uses paths relative to the file instead of being relative to the current directory, which is consistent with almost all other languages but not original Lua / Luau - this is a breaking change but will allow for proper packaging of third-party modules and more in the future.
    -   **_NOTE:_** _If you still want to use the default Lua behavior instead of relative paths, set the environment variable `LUAU_PWD_REQUIRE` to `true`_

### Changed

-   Improved error message when an invalid file path is passed to `require`
-   Much improved error formatting and stack traces

### Fixed

-   Fixed downloading of type definitions making json files instead of the proper format
-   Process termination will now always make sure all lua state is cleaned up before exiting, in all cases

## `0.0.6` - January 23rd, 2023

### Added

-   Initial implementation of [Roblox's task library](https://create.roblox.com/docs/reference/engine/libraries/task), with some caveats:

    -   Minimum wait / delay time is currently set to 10ms, subject to change
    -   It is not yet possible to pass arguments to tasks created using `task.spawn` / `task.delay` / `task.defer`
    -   Timings for `task.defer` are flaky and deferred tasks are not (yet) guaranteed to run after spawned tasks

    With all that said, everything else should be stable!

    -   Mixing and matching the `coroutine` library with `task` works in all cases
    -   `process.exit()` will stop all spawned / delayed / deferred threads and exit the process
    -   Lune is guaranteed to keep running until there are no longer any waiting threads

    If any of the abovementioned things do not work as expected, it is a bug, please file an issue!

### Fixed

-   Potential fix for spawned processes that yield erroring with "attempt to yield across metamethod/c-call boundary"

## `0.0.5` - January 22nd, 2023

### Added

-   Added full test suites for all Lune globals to ensure correct behavior
-   Added library version of Lune that can be used from other Rust projects

### Changed

-   Large internal changes to allow for implementing the `task` library.
-   Improved general formatting of errors to make them more readable & glanceable
-   Improved output formatting of non-primitive types
-   Improved output formatting of empty tables

### Fixed

-   Fixed double stack trace for certain kinds of errors

## `0.0.4` - January 21st, 2023

### Added

-   Added `process.args` for inspecting values given to Lune when running (read only)
-   Added `process.env` which is a plain table where you can get & set environment variables

### Changed

-   Improved error formatting & added proper file name to stack traces

### Removed

-   Removed `...` for process arguments, use `process.args` instead
-   Removed individual functions for getting & setting environment variables, use `process.env` instead

## `0.0.3` - January 20th, 2023

### Added

-   Added networking functions under `net`

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

-   Added console logging & coloring functions under `console`

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

-   The `json` api is now part of `net`
    -   `json.encode` becomes `net.jsonEncode`
    -   `json.decode` become `net.jsonDecode`

### Fixed

-   Fixed JSON decode not working properly

## `0.0.2` - January 19th, 2023

### Added

-   Added support for command-line parameters to scripts

    These can be accessed as a vararg in the root of a script:

    ```lua
    local firstArg: string, secondArg: string = ...
    print(firstArg, secondArg)
    ```

-   Added CLI parameters for downloading type definitions:

    -   `lune --download-selene-types` to download Selene types to the current directory
    -   `lune --download-luau-types` to download Luau types to the current directory

    These files will be downloaded as `lune.yml` and `luneTypes.d.luau`
    respectively and are also available in each release on GitHub.

## `0.0.1` - January 18th, 2023

Initial Release
