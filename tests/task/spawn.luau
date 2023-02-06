-- Spawning a task should return the thread that can then be cancelled

local thread = task.spawn(function() end)
assert(type(thread) == "thread", "Spawn should return the thread spawned")

-- Spawned functions should run right away

local flag: boolean = false
task.spawn(function()
	flag = true
end)
assert(flag, "Spawn should run instantly")

-- Spawned functions should work with yielding

local flag2: boolean = false
task.spawn(function()
	task.wait(0.1)
	flag2 = true
end)
assert(not flag2, "Spawn should work with yielding (1)")
task.wait(0.2)
assert(flag2, "Spawn should work with yielding (2)")

-- Spawned functions should be able to run threads created with the coroutine global

local flag3: boolean = false
local thread2 = coroutine.create(function()
	flag3 = true
end)
task.spawn(thread2)
assert(flag3, "Spawn should run threads made from coroutine.create")

-- Varargs should get passed correctly

local fcheck = require("./fcheck")

local function f(...: any)
	fcheck(1, "string", select(1, ...))
	fcheck(2, "number", select(2, ...))
	fcheck(3, "function", select(3, ...))
end

task.spawn(f, "", 1, f)
task.spawn(f, "inf", math.huge, f)
task.spawn(f, "NaN", 0 / 0, f)
