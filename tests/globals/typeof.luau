local function f() end

local thread1 = coroutine.create(f)
local thread2 = task.spawn(f)
local thread3 = task.defer(f)
local thread4 = task.delay(0, f)

assert(
	typeof(thread1) == "thread",
	"Calling typeof() did not return 'thread' after coroutine.create"
)
assert(typeof(thread2) == "thread", "Calling typeof() did not return 'thread' after task.spawn")
assert(typeof(thread3) == "thread", "Calling typeof() did not return 'thread' after task.defer")
assert(typeof(thread4) == "thread", "Calling typeof() did not return 'thread' after delay")
