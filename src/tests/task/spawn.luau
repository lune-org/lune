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

-- Varargs should get passed correctly

local function f(arg1: string, arg2: number, f2: (...any) -> ...any)
	assert(type(arg1) == "string", "Invalid arg 1 passed to function")
	assert(type(arg2) == "number", "Invalid arg 2 passed to function")
	assert(type(arg3) == "function", "Invalid arg 3 passed to function")
end

task.spawn(f, "", 1, f)
task.spawn(f, "inf", math.huge, f)
task.spawn(f, "NaN", 0 / 0, f)

task.wait(0.1)
