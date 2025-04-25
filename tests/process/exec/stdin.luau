local process = require("@lune/process")

local expected = "Hello from child process!"

-- NOTE: This requires "bash" to be installed even for Windows
-- users, testing a simple cat from stdin to stdout using pwsh
-- has unfortunately proved to be a significant challenge...
local result = process.exec("bash", { "-c", "cat" }, { stdio = { stdin = expected } })

local resultStdout = result.stdout
resultStdout = string.gsub(resultStdout, "^%s+", "") -- Trim leading whitespace
resultStdout = string.gsub(resultStdout, "%s+$", "") -- Trim trailing whitespace
assert(
	resultStdout == expected,
	"Stdin passing did not return proper output!" .. `\nExpected: {expected}` .. `\nReceived: {resultStdout}`
)
