local process = require("@lune/process")

-- Spawning a child process should not block the thread

local childThread = coroutine.create(process.create)

local ok, err = coroutine.resume(childThread, "echo", { "hello, world" })
assert(ok, err)

assert(
	coroutine.status(childThread) == "dead",
	"Child process should not block the thread it is running on"
)
