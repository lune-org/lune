-- Lune v0.1.3

--[=[
	@class console

	Logging & formatting
]=]
declare console: {
	--[=[
		@within console

		Resets the current persistent output color.
	]=]
	resetColor: () -> (),
	--[=[
		@within console

		Sets the current persistent output color.
	]=]
	setColor: (color: "black" | "red" | "green" | "yellow" | "blue" | "purple" | "cyan" | "white") -> (),
	--[=[
		@within console

		Resets the current persistent output style.
	]=]
	resetStyle: () -> (),
	--[=[
		@within console

		Sets the current persistent output style.
	]=]
	setStyle: (style: "bold" | "dim") -> (),
	--[=[
		@within console

		Formats arguments into a human-readable string with syntax highlighting for tables.
	]=]
	format: (...any) -> (string),
	--[=[
		@within console

		Prints arguments as a human-readable string with syntax highlighting for tables to stdout.
	]=]
	log: (...any) -> (),
	--[=[
		@within console

		Prints arguments as a human-readable string with syntax highlighting for tables to stdout.

		This will also prepend an [INFO] tag at the beginning of the message.
	]=]
	info: (...any) -> (),
	--[=[
		@within console

		Prints arguments as a human-readable string with syntax highlighting for tables to stdout.

		This will also prepend an [INFO] tag at the beginning of the message.
	]=]
	warn: (...any) -> (),
	--[=[
		@within console

		Prints arguments as a human-readable string with syntax highlighting for tables to stderr.

		This will also prepend an [ERROR] tag at the beginning of the message.

		Using this function will automatically set the exit code of the process
		to 1, unless it gets manually specified afterwards using `process.exit`.
	]=]
	error: (...any) -> (),
}

--[=[
	@class fs
	
	Filesystem
]=]
declare fs: {
	--[=[
		@within fs

		Reads a file at `path`.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to read the file.
		* The contents of the file cannot be read as a UTF-8 string.
		* Some other I/O error occurred.
	]=]
	readFile: (path: string) -> string,
	--[=[
		@within fs

		Reads entries in a directory at `path`.
		
		An error will be thrown in the following situations:

		* `path` does not point to an existing directory.
		* The current process lacks permissions to read the contents of the directory.
		* Some other I/O error occurred.
	]=]
	readDir: (path: string) -> { string },
	--[=[
		@within fs

		Writes to a file at `path`.

		An error will be thrown in the following situations:

		* The file's parent directory does not exist.
		* The current process lacks permissions to write to the file.
		* Some other I/O error occurred.
	]=]
	writeFile: (path: string, contents: string) -> (),
	--[=[
		@within fs

		Creates a directory and its parent directories if they are missing.

		An error will be thrown in the following situations:

		* `path` already points to an existing file or directory.
		* The current process lacks permissions to create the directory or its missing parents.
		* Some other I/O error occurred.
	]=]
	writeDir: (path: string) -> (),
	--[=[
		@within fs

		Removes a file.

		An error will be thrown in the following situations:

		* `path` does not point to an existing file.
		* The current process lacks permissions to remove the file.
		* Some other I/O error occurred.
	]=]
	removeFile: (path: string) -> (),
	--[=[
		@within fs

		Removes a directory and all of its contents.

		An error will be thrown in the following situations:

		* `path` is not an existing and empty directory.
		* The current process lacks permissions to remove the directory.
		* Some other I/O error occurred.
	]=]
	removeDir: (path: string) -> (),
	--[=[
		@within fs

		Checks if a given path is a file.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.
	]=]
	isFile: (path: string) -> boolean,
	--[=[
		@within fs

		Checks if a given path is a directory.

		An error will be thrown in the following situations:

		* The current process lacks permissions to read at `path`.
		* Some other I/O error occurred.
	]=]
	isDir: (path: string) -> boolean,
}

--[=[
	@class net

	Networking
]=]
declare net: {
	--[=[
		@within net

		Sends an HTTP request using the given url and / or parameters, and returns a dictionary that describes the response received.
		
		Only throws an error if a miscellaneous network or I/O error occurs, never for unsuccessful status codes.
	]=]
	request: (config: string | {
		url: string,
		method: ("GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH")?,
		headers: { [string]: string }?,
		body: string?,
	}) -> {
		ok: boolean,
		statusCode: number,
		statusMessage: string,
		headers: { [string]: string },
		body: string,
	},
	--[=[
		@within net

		Encodes the given value as JSON.
	]=]
	jsonEncode: (value: any, pretty: boolean?) -> string,
	--[=[
		@within net

		Decodes the given JSON string into a lua value.
	]=]
	jsonDecode: (encoded: string) -> any,
}

--[=[
	@class process

	Current process & child processes
]=]
declare process: {
	--[=[
		@within process

		The arguments given when running the Lune script.
	]=]
	args: { string },
	--[=[
		@within process

		Current environment variables for this process.

		Setting a value on this table will set the corresponding environment variable.
	]=]
	env: { [string]: string? },
	--[=[
		@within process

		Exits the currently running script as soon as possible with the given exit code.

		Exit code 0 is treated as a successful exit, any other value is treated as an error.

		Setting the exit code using this function will override any otherwise automatic exit code.
	]=]
	exit: (code: number?) -> (),
	--[=[
		@within process
		
		Spawns a child process that will run the program `program` with the given `params` as arguments, and returns a dictionary that describes the final status and ouput of the child process.
	]=]
	spawn: (program: string, params: { string }?) -> {
		ok: boolean,
		code: number,
		stdout: string,
		stderr: string,
	},
}

--[=[
	@class task

	Task scheduler & thread spawning
]=]
declare task: {
	--[=[
		@within task

		Stops a currently scheduled thread from resuming.

		@param thread The thread to cancel
	]=]
	cancel: (thread: thread) -> (),
	--[=[
		@within task

		Defers a thread or function to run at the end of the current task queue.

		@param functionOrThread The function or thread to defer
		@return The thread that will be deferred
	]=]
	defer: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within task

		Delays a thread or function to run after `duration` seconds.

		@param functionOrThread The function or thread to delay
		@return The thread that will be delayed
	]=]
	delay: <T...>(duration: number?, functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within task

		Instantly runs a thread or function.
		
		If the spawned task yields, the thread that spawned the task
		will resume, letting the spawned task run in the background.

		@param functionOrThread The function or thread to spawn
		@return The thread that was spawned
	]=]
	spawn: <T...>(functionOrThread: thread | (T...) -> (...any), T...) -> thread,
	--[=[
		@within task

		Waits for the given duration, with a minimum wait time of 10 milliseconds.

		@param duration The amount of time to wait
		@return The exact amount of time waited
	]=]
	wait: (duration: number?) -> (number),
}
