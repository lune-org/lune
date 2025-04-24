export type OS = "linux" | "macos" | "windows"
export type Arch = "x86_64" | "aarch64"
export type Endianness = "big" | "little"

export type StdioKind = "default" | "inherit" | "forward" | "none"
export type StdioOptions = {
	stdin: StdioKind?,
	stdout: StdioKind?,
	stderr: StdioKind?,
}

--[=[
	@interface CreateOptions
	@within Process

	A dictionary of options for `process.create`, with the following available values:

	* `cwd` - The current working directory for the process
	* `env` - Extra environment variables to give to the process
	* `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
]=]
export type CreateOptions = {
	cwd: string?,
	env: { [string]: string }?,
	shell: (boolean | string)?,
}

--[=[
	@interface ExecOptions
	@within Process

	A dictionary of options for `process.exec`, with the following available values:

	* `cwd` - The current working directory for the process
	* `env` - Extra environment variables to give to the process
	* `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
	* `stdio` - How to treat output and error streams from the child process - see `StdioKind` and `StdioOptions` for more info
]=]
export type ExecOptions = {
	cwd: string?,
	env: { [string]: string }?,
	shell: (boolean | string)?,
	stdio: (StdioKind | StdioOptions)?,
}

--[=[
	@class ChildProcessReader
	@within Process

	A reader class to read data from a child process' streams in realtime.
]=]
local ChildProcessReader = {}

--[=[
	@within ChildProcessReader

	Reads a chunk of data up to the specified length, or a default of 1KB at a time.

	Returns nil if there is no more data to read.

	This function may yield until there is new data to read from reader, if all data
	till present has already been read, and the process has not exited.

	@return The string containing the data read from the reader
]=]
function ChildProcessReader:read(chunkSize: number?): string?
	return nil :: any
end

--[=[
	@within ChildProcessReader

	Reads all the data currently present in the reader as a string.
	This function will yield until the process exits.

	@return The string containing the data read from the reader
]=]
function ChildProcessReader:readToEnd(): string
	return nil :: any
end

--[=[
	@class ChildProcessWriter
	@within Process

	A writer class to write data to a child process' streams in realtime.
]=]
local ChildProcessWriter = {}

--[=[
	@within ChildProcessWriter

	Writes a buffer or string of data to the writer.

	@param data The data to write to the writer
]=]
function ChildProcessWriter:write(data: buffer | string): ()
	return nil :: any
end

--[=[
	@within ChildProcessWriter

	Closes the underlying I/O stream for the writer.
]=]
function ChildProcessWriter:close(): ()
	return nil :: any
end

--[=[
	@interface ChildProcess
	@within Process

	Result type for child processes in `process.create`.

	This is a dictionary containing the following values:

	* `stdin` - A writer to write to the child process' stdin - see `ChildProcessWriter` for more info
	* `stdout` - A reader to read from the child process' stdout - see `ChildProcessReader` for more info
	* `stderr` - A reader to read from the child process' stderr - see `ChildProcessReader` for more info
	* `kill` - A method that kills the child process
	* `status` - A method that yields and returns the exit status of the child process
]=]
export type ChildProcess = {
	stdin: typeof(ChildProcessWriter),
	stdout: typeof(ChildProcessReader),
	stderr: typeof(ChildProcessReader),
	kill: (self: ChildProcess) -> (),
	status: (self: ChildProcess) -> {
		ok: boolean,
		code: number,
	},
}

--[=[
	@interface ExecResult
	@within Process

	Result type for child processes in `process.exec`.

	This is a dictionary containing the following values:

	* `ok` - If the child process exited successfully or not, meaning the exit code was zero or not set
	* `code` - The exit code set by the child process, or 0 if one was not set
	* `stdout` - The full contents written to stdout by the child process, or an empty string if nothing was written
	* `stderr` - The full contents written to stderr by the child process, or an empty string if nothing was written
]=]
export type ExecResult = {
	ok: boolean,
	code: number,
	stdout: string,
	stderr: string,
}

--[=[
	@class Process

	Built-in functions for the current process & child processes

	### Example usage

	```lua
	local process = require("@lune/process")

	-- Getting the arguments passed to the Lune script
	for index, arg in process.args do
		print("Process argument #" .. tostring(index) .. ": " .. arg)
	end

	-- Getting the currently available environment variables
	local PORT: string? = process.env.PORT
	local HOME: string? = process.env.HOME
	for name, value in process.env do
		print("Environment variable " .. name .. " is set to " .. value)
	end

	-- Getting the current os and processor architecture
	print("Running " .. process.os .. " on " .. process.arch .. "!")

	-- Executing a command
	local result = process.exec("program", {
		"cli argument",
		"other cli argument"
	})
	if result.ok then
		print(result.stdout)
	else
		print(result.stderr)
	end

	-- Spawning a child process
	local child = process.create("program", {
		"cli argument",
		"other cli argument"
	})

	-- Writing to the child process' stdin
	child.stdin:write("Hello from Lune!")

	-- Reading from the child process' stdout
	local data = child.stdout:read()
	print(data)
	```
]=]
local process = {}

--[=[
	@within Process
	@prop os OS
	@tag read_only

	The current operating system being used.

	Possible values:

	* `"linux"`
	* `"macos"`
	* `"windows"`
]=]
process.os = (nil :: any) :: OS

--[=[
	@within Process
	@prop arch Arch
	@tag read_only

	The architecture of the processor currently being used.

	Possible values:

	* `"x86_64"`
	* `"aarch64"`
]=]
process.arch = (nil :: any) :: Arch

--[=[
	@within Process
	@prop endianness Endianness
	@tag read_only

	The endianness of the processor currently being used.

	Possible values:

	* `"big"`
	* `"little"`
]=]
process.endianness = (nil :: any) :: Endianness

--[=[
	@within Process
	@prop args { string }
	@tag read_only

	The arguments given when running the Lune script.
]=]
process.args = (nil :: any) :: { string }

--[=[
	@within Process
	@prop cwd string
	@tag read_only

	The current working directory in which the Lune script is running.
]=]
process.cwd = (nil :: any) :: string

--[=[
	@within Process
	@prop env { [string]: string? }
	@tag read_write

	Current environment variables for this process.

	Setting a value on this table will set the corresponding environment variable.
]=]
process.env = (nil :: any) :: { [string]: string? }

--[=[
	@within Process

	Exits the currently running script as soon as possible with the given exit code.

	Exit code 0 is treated as a successful exit, any other value is treated as an error.

	Setting the exit code using this function will override any otherwise automatic exit code.

	@param code The exit code to set
]=]
function process.exit(code: number?): never
	return nil :: any
end

--[=[
	@within Process

	Spawns a child process in the background that runs the program `program`,
	and immediately returns readers and writers to communicate with it.

	In order to execute a command and wait for its output, see `process.exec`.

	The second argument, `params`, can be passed as a list of string parameters to give to the program.

	The third argument, `options`, can be passed as a dictionary of options to give to the child process.
	Refer to the documentation for `SpawnOptions` for specific option keys and their values.

	@param program The program to Execute as a child process
	@param params Additional parameters to pass to the program
	@param options A dictionary of options for the child process
	@return A dictionary with the readers and writers to communicate with the child process
]=]
function process.create(program: string, params: { string }?, options: CreateOptions?): ChildProcess
	return nil :: any
end

--[=[
	@within Process

	Executes a child process that will execute the command `program`, waiting for it to exit.
	Upon exit, it returns a dictionary that describes the final status and ouput of the child process.

	In order to spawn a child process in the background, see `process.create`.

	The second argument, `params`, can be passed as a list of string parameters to give to the program.

	The third argument, `options`, can be passed as a dictionary of options to give to the child process.
	Refer to the documentation for `ExecOptions` for specific option keys and their values.

	@param program The program to Execute as a child process
	@param params Additional parameters to pass to the program
	@param options A dictionary of options for the child process
	@return A dictionary representing the result of the child process
]=]
function process.exec(program: string, params: { string }?, options: ExecOptions?): ExecResult
	return nil :: any
end

return process
