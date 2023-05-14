local task = require("@lune/task")

local function f() end

local thread1 = coroutine.create(f)
local thread2 = task.spawn(f)
local thread3 = task.defer(f)
local thread4 = task.delay(0, f)

assert(type(thread1) == "thread", "Calling type() did not return 'thread' after coroutine.create")
assert(type(thread2) == "thread", "Calling type() did not return 'thread' after task.spawn")
assert(type(thread3) == "thread", "Calling type() did not return 'thread' after task.defer")
assert(type(thread4) == "thread", "Calling type() did not return 'thread' after delay")
