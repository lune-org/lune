local process = require("@lune/process")

local expected = "hello, world"

-- Stdout test

local catChild = process.create("cat")
catChild.stdin:write(expected)
catChild.stdin:close()
local catOutput = catChild.stdout:read(#expected)

assert(
	expected == catOutput,
	"Failed to write to stdin or read from stdout of child process!"
		.. `\nExpected: "{expected}"`
		.. `\nReceived: "{catOutput}"`
)

-- Stderr test, needs to run in shell because there is no
-- other good cross-platform way to simply write to stdout

local echoChild = if process.os == "windows"
	then process.create("/c", { "echo", expected, "1>&2" }, { shell = "cmd" })
	else process.create("echo", { expected, ">>/dev/stderr" }, { shell = true })
local echoOutput = echoChild.stderr:read(#expected)

assert(
	expected == echoOutput,
	"Failed to write to stdin or read from stderr of child process!"
		.. `\nExpected: "{expected}"`
		.. `\nReceived: "{echoOutput}"`
)
