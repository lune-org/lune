export type OS = "linux" | "macos" | "windows"
export type Arch = "x86_64" | "aarch64"

export type SpawnOptionsStdioKind = "default" | "inherit" | "forward" | "none"
export type SpawnOptionsStdio = {
	stdout: SpawnOptionsStdioKind?,
	stderr: SpawnOptionsStdioKind?,
	stdin: string?,
}

--[=[
	@interface SpawnOptions
	@within Process

	A dictionary of options for `process.spawn`, with the following available values:

	* `cwd` - The current working directory for the process
	* `env` - Extra environment variables to give to the process
	* `shell` - Whether to run in a shell or not - set to `true` to run using the default shell, or a string to run using a specific shell
	* `stdio` - How to treat output and error streams from the child process - see `SpawnOptionsStdioKind` and `SpawnOptionsStdio` for more info
	* `stdin` - Optional standard input to pass to spawned child process
]=]
export type SpawnOptions = {
	cwd: string?,
	env: { [string]: string }?,
	shell: (boolean | string)?,
	stdio: (SpawnOptionsStdioKind | SpawnOptionsStdio)?,
	stdin: string?, -- TODO: Remove this since it is now available in stdio above, breaking change
}

--[=[
	@interface SpawnResult
	@within Process

	Result type for child processes in `process.spawn`.

	This is a dictionary containing the following values:

	* `ok` - If the child process exited successfully or not, meaning the exit code was zero or not set
	* `code` - The exit code set by the child process, or 0 if one was not set
	* `stdout` - The full contents written to stdout by the child process, or an empty string if nothing was written
	* `stderr` - The full contents written to stderr by the child process, or an empty string if nothing was written
]=]
export type SpawnResult = {
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

	-- Spawning a child process
	local result = process.spawn("program", {
		"cli argument",
		"other cli argument"
	})
	if result.ok then
		print(result.stdout)
	else
		print(result.stderr)
	end
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

	Spawns a child process that will run the program `program`, and returns a dictionary that describes the final status and ouput of the child process.

	The second argument, `params`, can be passed as a list of string parameters to give to the program.

	The third argument, `options`, can be passed as a dictionary of options to give to the child process.
	Refer to the documentation for `SpawnOptions` for specific option keys and their values.

	@param program The program to spawn as a child process
	@param params Additional parameters to pass to the program
	@param options A dictionary of options for the child process
	@return A dictionary representing the result of the child process
]=]
function process.spawn(program: string, params: { string }?, options: SpawnOptions?): SpawnResult
	return nil :: any
end

return process
