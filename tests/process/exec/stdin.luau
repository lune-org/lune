--[[
	Unfortunately, this test is, for some reason, very flaky on Windows.

	The behavior of stdin itself works as expected, but a simple test
	that passes a string to stdin, and needs that exact string back out,
	is near impossible to write using PowerShell on Windows.

	For now, we will disable this test entirely on that platform to
	not disturb the rest of the test suite and contribution process.
]]

local DISABLE_WINDOWS_TESTS = true

local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local expected = "Hello from child process!"

-- Writing to stdin should succeed and we should be able
-- to read the exact message we wrote to it from stdout

if DISABLE_WINDOWS_TESTS and process.os == "windows" then
	process.exit(0)
end

local options = { stdio = { stdin = expected } }
local result = if process.os == "windows"
	then process.exec("powershell", {
		"-NonInteractive",
		"-NoProfile",
		"-Command",
		"[System.Console]::Write([System.Console]::In.ReadToEnd())",
	}, options)
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

-- Not passing anything as the stdin option should guarantee
-- that stdin is completely closed, and not waiting for input

local thread = task.delay(5, function()
	stdio.ewrite("Not passing stdin to process.exec should not hang\n")
	process.exit(1)
end)

if process.os == "windows" then
	process.exec("powershell", {
		-- "-NonInteractive", -- if interactive (we have stdin) we *should* hang!
		"-NoProfile",
		"-Command",
		"[System.Console]::In.ReadToEnd()",
	})
else
	process.exec("bash", { "-c", "cat" })
end

task.cancel(thread)
process.exit(0)
