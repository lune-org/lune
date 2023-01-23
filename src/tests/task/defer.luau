-- Deferring a task should return the thread that can then be cancelled

local thread = task.defer(function() end)
assert(type(thread) == "thread", "Defer should return the thread spawned")

-- Deferred functions should run after other threads

local flag: boolean = false
task.defer(function()
	flag = true
end)
assert(not flag, "Defer should not run instantly or block")
task.wait(0.1)
assert(flag, "Defer should run")

-- Deferred functions should work with yielding

local flag2: boolean = false
task.defer(function()
	task.wait(0.1)
	flag2 = true
end)
assert(not flag2, "Defer should work with yielding (1)")
task.wait(0.2)
assert(flag2, "Defer should work with yielding (2)")

-- Deferred functions should run after other spawned threads
local flag3: boolean = false
task.defer(function()
	if flag3 == true then
		flag3 = false
	end
end)
task.spawn(function()
	if flag3 == false then
		flag3 = true
	end
end)
assert(not flag2, "Defer should run after spawned threads")

-- Varargs should get passed correctly

local function f(arg1: string, arg2: number, f2: (...any) -> ...any)
	assert(type(arg1) == "string", "Invalid arg 1 passed to function")
	assert(type(arg2) == "number", "Invalid arg 2 passed to function")
	assert(type(arg3) == "function", "Invalid arg 3 passed to function")
end

task.defer(f, "", 1, f)
task.defer(f, "inf", math.huge, f)
task.defer(f, "NaN", 0 / 0, f)

task.wait(0.1)
