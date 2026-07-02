local process = require("@lune/process")

local expected = "Hello, world!"

local catChild = process.create("cat")
catChild.stdin:write(expected)
catChild:kill()
local catStatus = catChild:status()
local catStdout = catChild.stdout:readToEnd()

assert(catStatus.code == 9, "Child process should have an exit code of 9 (SIGKILL)")
assert(catStdout == expected, "Reading from stdout of child process should work even after kill")

local stdinWriteOk = pcall(function()
	catChild.stdin:write(expected)
end)
assert(not stdinWriteOk, "Writing to stdin of child process should not work after kill")
