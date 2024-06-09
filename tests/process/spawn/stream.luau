local process = require("@lune/process")

-- Should be able to write and read from child process streams

local msg = "hello, world"

local catChild = process.spawn("cat")
catChild.stdin:write(msg)
assert(
	msg == buffer.tostring(catChild.stdout:read(#msg)),
	"Failed to write to stdin or read from stdout of child process"
)

local echoChild = if process.os == "windows"
	then process.spawn("Write-Error", { msg }, { shell = "powershell" })
	else process.spawn("echo", { msg, ">>/dev/stderr" }, { shell = true })

assert(
	msg == buffer.tostring(echoChild.stderr:read(#msg)),
	"Failed to read from stderr of child process"
)
