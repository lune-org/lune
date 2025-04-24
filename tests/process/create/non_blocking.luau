local process = require("@lune/process")

local childThread = coroutine.create(process.create)

local ok, err = coroutine.resume(childThread, "echo", { "hello, world" })
assert(ok, err)

assert(coroutine.status(childThread) == "dead", "Child process should not yield the thread it is created on")
