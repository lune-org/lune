local process = require("@lune/process")

local expected = "Hello from child process!"

-- When passing stdin to powershell on windows we must "accept" using the double newline

local result = if process.os == "windows"
	then process.exec("powershell", { "echo" }, { stdio = { stdin = expected .. "\n\n" } })
	else process.exec("xargs", { "echo" }, { stdio = { stdin = expected } })

local resultStdout = result.stdout
resultStdout = string.gsub(resultStdout, "^%s+", "") -- Trim leading whitespace
resultStdout = string.gsub(resultStdout, "%s+$", "") -- Trim trailing whitespace
assert(
	resultStdout == expected,
	"Stdin passing did not return proper output!" .. `\nExpected: {expected}` .. `\nReceived: {resultStdout}`
)
