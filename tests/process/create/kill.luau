local process = require("@lune/process")

-- Killing a child process should work as expected

local message = "Hello, world!"
local child = process.create("cat")

child.stdin:write(message)
child.kill()

assert(child.status().code == 9, "Child process should have an exit code of 9 (SIGKILL)")

assert(
	child.stdout:readToEnd() == message,
	"Reading from stdout of child process should work even after kill"
)

local stdinWriteOk = pcall(function()
	child.stdin:write(message)
end)
assert(not stdinWriteOk, "Writing to stdin of child process should not work after kill")
