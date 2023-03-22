--[=[
	@type FsWriteOptions
	@within FS

	Options for filesystem APIs what write to files and/or directories.

	This is a dictionary that may contain one or more of the following values:

	* `overwrite` - If the target path should be overwritten or not, in the case that it already exists
]=]
export type FsWriteOptions = {
	overwrite: boolean?,
}

--[=[
	@class FS

	Filesystem
]=]
declare fs: {
	--[=[
		@within FS
		@must_use

		Reads a file at `path`.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to read the file.
		* The contents of the file cannot be read as a UTF-8 string.
		* Some other I/O error occurred.

		@param path The path to the file to read
		@return The contents of the file
	]=]
	readFile: (path: string) -> string,
	--[=[
		@within FS
		@must_use

		Reads entries in a directory at `path`.

		An error will be thrown in the following situations:

		* `path` does not point to an existing directory.
		* The current process lacks permissions to read the contents of the directory.
		* Some other I/O error occurred.

		@param path The directory path to search in
		@return A list of files & directories found
	]=]
	readDir: (path: string) -> { string },
	--[=[
		@within FS

		Writes to a file at `path`.

		An error will be thrown in the following situations:

		* The file's parent directory does not exist.
		* The current process lacks permissions to write to the file.
		* Some other I/O error occurred.

		@param path The path of the file
		@param contents The contents of the file
	]=]
	writeFile: (path: string, contents: string) -> (),
	--[=[
		@within FS

		Creates a directory and its parent directories if they are missing.

		An error will be thrown in the following situations:

		* `path` already points to an existing file or directory.
		* The current process lacks permissions to create the directory or its missing parents.
		* Some other I/O error occurred.

		@param path The directory to create
	]=]
	writeDir: (path: string) -> (),
	--[=[
		@within FS

		Removes a file.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to remove the file.
		* Some other I/O error occurred.

		@param path The file to remove
	]=]
	removeFile: (path: string) -> (),
	--[=[
		@within FS

		Removes a directory and all of its contents.

		An error will be thrown in the following situations:

		* `path` is not an existing and empty directory.
		* The current process lacks permissions to remove the directory.
		* Some other I/O error occurred.

		@param path The directory to remove
	]=]
	removeDir: (path: string) -> (),
	--[=[
		@within FS
		@must_use

		Checks if a given path is a file.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.

		@param path The file path to check
		@return If the path is a file or not
	]=]
	isFile: (path: string) -> boolean,
	--[=[
		@within FS
		@must_use

		Checks if a given path is a directory.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.

		@param path The directory path to check
		@return If the path is a directory or not
	]=]
	isDir: (path: string) -> boolean,
	--[=[
		@within FS

		Moves a file or directory to a new path.

		Throws an error if a file or directory already exists at the target path.
		This can be bypassed by passing `true` as the third argument, or a dictionary of options.
		Refer to the documentation for `FsWriteOptions` for specific option keys and their values.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `from` or write at `to`.
		* The new path exists on a different mount point.
		* Some other I/O error occurred.

		@param from The path to move from
		@param to The path to move to
		@param overwriteOrOptions Options for the target path, such as if should be overwritten if it already exists
	]=]
	move: (from: string, to: string, overwriteOrOptions: (boolean | FsWriteOptions)?) -> (),
}

type NetMethod = "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH"

--[=[
	@type NetFetchParams
	@within Net

	Parameters for sending network requests with `net.request`.

	This is a dictionary that may contain one or more of the following values:

	* `url` - The URL to send a request to. This is always required
	* `method` - The HTTP method verb, such as `"GET"`, `"POST"`, `"PATCH"`, `"PUT"`, or `"DELETE"`. Defaults to `"GET"`
	* `query` - A table of key-value pairs representing query parameters in the request path
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body
]=]
export type NetFetchParams = {
	url: string,
	method: NetMethod?,
	query: { [string]: string }?,
	headers: { [string]: string }?,
	body: string?,
}

--[=[
	@type NetFetchResponse
	@within new

	Response type for sending network requests with `net.request`.

	This is a dictionary containing the following values:

	* `ok` - If the status code is a canonical success status code, meaning within the range 200 -> 299
	* `statusCode` - The status code returned for the request
	* `statusMessage` - The canonical status message for the returned status code, such as `"Not Found"` for status code 404
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body, or an empty string if one was not given
]=]
export type NetFetchResponse = {
	ok: boolean,
	statusCode: number,
	statusMessage: string,
	headers: { [string]: string },
	body: string,
}

--[=[
	@type NetRequest
	@within Net

	Data type for requests in `net.serve`.

	This is a dictionary containing the following values:

	* `path` - The path being requested, relative to the root. Will be `/` if not specified
	* `query` - A table of key-value pairs representing query parameters in the request path
	* `method` - The HTTP method verb, such as `"GET"`, `"POST"`, `"PATCH"`, `"PUT"`, or `"DELETE"`. Will always be uppercase
	* `headers` - A table of key-value pairs representing headers
	* `body` - The request body, or an empty string if one was not given
]=]
export type NetRequest = {
	path: string,
	query: { [string]: string? },
	method: NetMethod,
	headers: { [string]: string },
	body: string,
}

--[=[
	@type NetRequest
	@within Net

	Response type for requests in `net.serve`.

	This is a dictionary that may contain one or more of the following values:

	* `status` - The status code for the request, in the range `100` -> `599`
	* `headers` - A table of key-value pairs representing headers
	* `body` - The response body
]=]
export type NetResponse = {
	status: number?,
	headers: { [string]: string }?,
	body: string?,
}

type NetServeHttpHandler = (request: NetRequest) -> string | NetResponse
type NetServeWebSocketHandler = (socket: NetWebSocket) -> ()

--[=[
	@type NetServeConfig
	@within Net

	Configuration for `net.serve`.

	This may contain one of, or both of the following callbacks:

	* `handleRequest` for handling normal http requests, equivalent to just passing a function to `net.serve`
	* `handleWebSocket` for handling web socket requests, which will receive a `NetWebSocket` object as its first and only parameter
]=]
export type NetServeConfig = {
	handleRequest: NetServeHttpHandler?,
	handleWebSocket: NetServeWebSocketHandler?,
}

--[=[
	@type NetServeHandle
	@within Net

	A handle to a currently running web server, containing a single `stop` function to gracefully shut down the web server.
]=]
export type NetServeHandle = {
	stop: () -> (),
}

--[=[
	@type NetWebSocket
	@within Net

	A reference to a web socket connection.

	The web socket may be in either an "open" or a "closed" state, changing its current behavior.

	When open:

	* Any function on the socket such as `send`, `next` or `close` can be called without erroring
	* `next` can be called to yield until the next message is received or the socket becomes closed

	When closed:

	* `next` will no longer return any message(s) and instead instantly return nil
	* `send` will throw an error stating that the socket has been closed

	Once the websocket has been closed, `closeCode` will no longer be nil, and will be populated with a close
	code according to the [WebSocket specification](https://www.iana.org/assignments/websocket/websocket.xhtml).
	This will be an integer between 1000 and 4999, where 1000 is the canonical code for normal, error-free closure.
]=]
export type NetWebSocket = {
	closeCode: number?,
	close: (code: number?) -> (),
	send: (message: string, asBinaryMessage: boolean?) -> (),
	next: () -> string?,
}

--[=[
	@class Net

	Networking
]=]
declare net: {
	--[=[
		@within Net

		Sends an HTTP request using the given url and / or parameters, and returns a dictionary that describes the response received.

		Only throws an error if a miscellaneous network or I/O error occurs, never for unsuccessful status codes.

		@param config The URL or request config to use
		@return A dictionary representing the response for the request
	]=]
	request: (config: string | NetFetchParams) -> NetFetchResponse,
	--[=[
		@within Net
		@must_use

		Connects to a web socket at the given URL.

		Throws an error if the server at the given URL does not support
		web sockets, or if a miscellaneous network or I/O error occurs.

		@param url The URL to connect to
		@return A web socket handle
	]=]
	socket: (url: string) -> NetWebSocket,
	--[=[
		@within Net

		Creates an HTTP server that listens on the given `port`.

		This will ***not*** block and will keep listening for requests on the given `port`
		until the `stop` function on the returned `NetServeHandle` has been called.

		@param port The port to use for the server
		@param handlerOrConfig The handler function or config to use for the server
	]=]
	serve: (port: number, handlerOrConfig: NetServeHttpHandler | NetServeConfig) -> NetServeHandle,
	--[=[
		@within Net
		@must_use

		Encodes the given value as JSON.

		@param value The value to encode as JSON
		@param pretty If the encoded JSON string should include newlines and spaces. Defaults to false
		@return The encoded JSON string
	]=]
	jsonEncode: (value: any, pretty: boolean?) -> string,
	--[=[
		@within Net
		@must_use

		Decodes the given JSON string into a lua value.

		@param encoded The JSON string to decode
		@return The decoded lua value
	]=]
	jsonDecode: (encoded: string) -> any,
	--[=[
		@within Net
		@must_use

		Encodes the given string using URL encoding.

		@param s The string to encode
		@param binary If the string should be treated as binary data and/or is not valid utf-8. Defaults to false
		@return The encoded string
	]=]
	urlEncode: (s: string, binary: boolean?) -> string,
	--[=[
		@within Net
		@must_use

		Decodes the given string using URL decoding.

		@param s The string to decode
		@param binary If the string should be treated as binary data and/or is not valid utf-8. Defaults to false
		@return The decoded string
	]=]
	urlDecode: (s: string, binary: boolean?) -> string,
}

-- TODO: Somehow link this up to require("@lune/serde")

type SerdeEncodeDecodeFormat = "json" | "yaml" | "toml"

type Serde = {
	--[=[
		@within Serde
		@must_use

		Encodes the given value using the given format.

		@param format The format to use
		@param value The value to encode
		@param pretty If the encoded string should be human-readable, including things such as newlines and spaces. Only supported for json and toml formats, and defaults to false
		@return The encoded string
	]=]
	encode: (format: SerdeEncodeDecodeFormat, value: any, pretty: boolean?) -> string,
	--[=[
		@within Serde
		@must_use

		Decodes the given string using the given format into a lua value.

		@param format The format to use
		@param encoded The string to decode
		@return The decoded lua value
	]=]
	decode: (format: SerdeEncodeDecodeFormat, encoded: string) -> any,
}

type ProcessSpawnOptionsStdio = "inherit" | "default"

--[=[
	@type ProcessSpawnOptions
	@within Process

	A dictionary of options for `process.spawn`, with the following available values:

	* `cwd` - The current working directory for the process
	* `env` - Extra environment variables to give to the process
	* `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
	* `stdio` - How to treat output and error streams from the child process - set to "inherit" to pass output and error streams to the current process
]=]
export type ProcessSpawnOptions = {
	cwd: string?,
	env: { [string]: string }?,
	shell: (boolean | string)?,
	stdio: ProcessSpawnOptionsStdio?,
}

--[=[
	@type ProcessSpawnResult
	@within Process

	Result type for child processes in `process.spawn`.

	This is a dictionary containing the following values:

	* `ok` - If the child process exited successfully or not, meaning the exit code was zero or not set
	* `code` - The exit code set by the child process, or 0 if one was not set
	* `stdout` - The full contents written to stdout by the child process, or an empty string if nothing was written
	* `stderr` - The full contents written to stderr by the child process, or an empty string if nothing was written
]=]
export type ProcessSpawnResult = {
	ok: boolean,
	code: number,
	stdout: string,
	stderr: string,
}

--[=[
	@class Process

	Current process & child processes
]=]
declare process: {
	--[=[
		@within Process
		@read_only

		The current operating system being used.

		Possible values:

		* `"linux"`
		* `"macos"`
		* `"windows"`
	]=]
	os: "linux" | "macos" | "windows",
	--[=[
		@within Process
		@read_only

		The architecture of the processor currently being used.

		Possible values:

		* `"x86_64"`
		* `"aarch64"`
	]=]
	arch: "x86_64" | "aarch64",
	--[=[
		@within Process
		@read_only

		The arguments given when running the Lune script.
	]=]
	args: { string },
	--[=[
		@within Process
		@read_only

		The current working directory in which the Lune script is running.
	]=]
	cwd: string,
	--[=[
		@within Process
		@read_write

		Current environment variables for this process.

		Setting a value on this table will set the corresponding environment variable.
	]=]
	env: { [string]: string? },
	--[=[
		@within Process

		Exits the currently running script as soon as possible with the given exit code.

		Exit code 0 is treated as a successful exit, any other value is treated as an error.

		Setting the exit code using this function will override any otherwise automatic exit code.

		@param code The exit code to set
	]=]
	exit: (code: number?) -> (),
	--[=[
		@within Process

		Spawns a child process that will run the program `program`, and returns a dictionary that describes the final status and ouput of the child process.

		The second argument, `params`, can be passed as a list of string parameters to give to the program.

		The third argument, `options`, can be passed as a dictionary of options to give to the child process.
		Refer to the documentation for `ProcessSpawnOptions` for specific option keys and their values.

		@param program The program to spawn as a child process
		@param params Additional parameters to pass to the program
		@param options A dictionary of options for the child process
		@return A dictionary representing the result of the child process
	]=]
	spawn: (
		program: string,
		params: { string }?,
		options: ProcessSpawnOptions?
	) -> ProcessSpawnResult,
}

--[=[
	@class Stdio

	Standard input / output & utility functions
]=]
declare stdio: {
	--[=[
		@within Stdio
		@must_use

		Return an ANSI string that can be used to modify the persistent output color.

		Pass `"reset"` to get a string that can reset the persistent output color.

		### Example usage

		```lua
		stdio.write(stdio.color("red"))
		print("This text will be red")
		stdio.write(stdio.color("reset"))
		print("This text will be normal")
		```

		@param color The color to use
		@return A printable ANSI string
	]=]
	color: (color: "reset" | "black" | "red" | "green" | "yellow" | "blue" | "purple" | "cyan" | "white") -> string,
	--[=[
		@within Stdio
		@must_use

		Return an ANSI string that can be used to modify the persistent output style.

		Pass `"reset"` to get a string that can reset the persistent output style.

		### Example usage

		```lua
		stdio.write(stdio.style("bold"))
		print("This text will be bold")
		stdio.write(stdio.style("reset"))
		print("This text will be normal")
		```

		@param style The style to use
		@return A printable ANSI string
	]=]
	style: (style: "reset" | "bold" | "dim") -> string,
	--[=[
		@within Stdio
		@must_use

		Formats arguments into a human-readable string with syntax highlighting for tables.

		@param ... The values to format
		@return The formatted string
	]=]
	format: (...any) -> string,
	--[=[
		@within Stdio

		Writes a string directly to stdout, without any newline.

		@param s The string to write to stdout
	]=]
	write: (s: string) -> (),
	--[=[
		@within Stdio

		Writes a string directly to stderr, without any newline.

		@param s The string to write to stderr
	]=]
	ewrite: (s: string) -> (),
	--[=[
		@within Stdio
		@must_use

		Prompts for user input using the wanted kind of prompt:

		* `"text"` - Prompts for a plain text string from the user
		* `"confirm"` - Prompts the user to confirm with y / n (yes / no)
		* `"select"` - Prompts the user to select *one* value from a list
		* `"multiselect"` - Prompts the user to select *one or more* values from a list
		* `nil` - Equivalent to `"text"` with no extra arguments

		@param kind The kind of prompt to use
		@param message The message to show the user
		@param defaultOrOptions The default value for the prompt, or options to choose from for selection prompts
	]=]
	prompt: (
		(() -> string)
		& ((kind: "text", message: string?, defaultOrOptions: string?) -> string)
		& ((kind: "confirm", message: string, defaultOrOptions: boolean?) -> boolean)
		& ((kind: "select", message: string?, defaultOrOptions: { string }) -> number?)
		& ((kind: "multiselect", message: string?, defaultOrOptions: { string }) -> { number }?)
	),
}

--[=[
	@class Task

	Task scheduler & thread spawning
]=]
declare task: {
	--[=[
		@within Task

		Stops a currently scheduled thread from resuming.

		@param thread The thread to cancel
	]=]
	cancel: (thread: thread) -> (),
	--[=[
		@within Task

		Defers a thread or function to run at the end of the current task queue.

		@param functionOrThread The function or thread to defer
		@return The thread that will be deferred
	]=]
	defer: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within Task

		Delays a thread or function to run after `duration` seconds.

		@param functionOrThread The function or thread to delay
		@return The thread that will be delayed
	]=]
	delay: <T...>(duration: number?, functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within Task

		Instantly runs a thread or function.

		If the spawned task yields, the thread that spawned the task
		will resume, letting the spawned task run in the background.

		@param functionOrThread The function or thread to spawn
		@return The thread that was spawned
	]=]
	spawn: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within Task

		Waits for *at least* the given amount of time.

		The minimum wait time possible when using `task.wait` is limited by the underlying OS sleep implementation.
		For most systems this means `task.wait` is accurate down to about 5 milliseconds or less.

		@param duration The amount of time to wait
		@return The exact amount of time waited
	]=]
	wait: (duration: number?) -> number,
}

--[=[
	Prints given value(s) to stdout.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
declare print: <T...>(T...) -> ()

--[=[
	Prints given value(s) to stdout with a leading `[INFO]` tag.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
declare printinfo: <T...>(T...) -> ()

--[=[
	Prints given value(s) to stdout with a leading `[WARN]` tag.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
declare warn: <T...>(T...) -> ()

--[=[
	Throws an error and prints a formatted version of it with a leading `[ERROR]` tag.

	@param message The error message to throw
	@param level The stack level to throw the error at, defaults to 0
]=]
declare error: <T>(message: T, level: number?) -> ()
