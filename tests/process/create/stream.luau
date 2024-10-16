local process = require("@lune/process")

-- Should be able to write and read from child process streams

local msg = "hello, world"

local catChild = process.create("cat")
catChild.stdin:write(msg)
assert(
	msg == catChild.stdout:read(#msg),
	"Failed to write to stdin or read from stdout of child process"
)

local echoChild = if process.os == "windows"
	then process.create("/c", { "echo", msg, "1>&2" }, { shell = "cmd" })
	else process.create("echo", { msg, ">>/dev/stderr" }, { shell = true })

assert(msg == echoChild.stderr:read(#msg), "Failed to read from stderr of child process")
