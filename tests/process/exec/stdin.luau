local process = require("@lune/process")

local expected = "Hello from child process!"

local result = if process.os == "windows"
	then process.exec("powershell", { "-Command", "$input" }, { stdio = { stdin = expected } })
	else process.exec("xargs", { "echo" }, { stdio = { stdin = expected } })

local resultStdout = result.stdout
resultStdout = string.gsub(resultStdout, "^%s+", "") -- Trim leading whitespace
resultStdout = string.gsub(resultStdout, "%s+$", "") -- Trim trailing whitespace
assert(
	resultStdout == expected,
	"Stdin passing did not return proper output!" .. `\nExpected: {expected}` .. `\nReceived: {resultStdout}`
)
