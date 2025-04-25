local process = require("@lune/process")

local expected = "Hello from child process!"

local options = { stdio = { stdin = expected } }
local result = if process.os == "windows"
	then process.exec(
		"powershell",
		{ "-Command", "[System.Console]::Write([System.Console]::In.ReadToEnd())" },
		options
	)
	else process.exec("bash", { "-c", "cat" }, options)

local resultStdout = result.stdout
resultStdout = string.gsub(resultStdout, "^%s+", "") -- Trim leading whitespace
resultStdout = string.gsub(resultStdout, "%s+$", "") -- Trim trailing whitespace
assert(
	resultStdout == expected,
	"Passing a string to stdin did not return the expected output!"
		.. `\nExpected: {expected}`
		.. `\nReceived: {resultStdout}`
)
