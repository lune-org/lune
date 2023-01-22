-- Delaying a task should return the thread that can then be cancelled

local thread = task.delay(0, function() end)
assert(type(thread) == "thread", "Delay should return the thread spawned")

-- Delayed functions should never run right away

local flag: boolean = false
task.delay(0, function()
	flag = true
end)
assert(not flag, "Delay should not run instantly or block")
task.wait(1 / 60)
assert(flag, "Delay should run after the wanted duration")

-- Delayed functions should work with yielding

local flag2: boolean = false
task.delay(0.2, function()
	flag2 = true
	task.wait(0.4)
	flag2 = false
end)
task.wait(0.4)
assert(flag, "Delay should work with yielding (1)")
task.wait(0.4)
assert(not flag2, "Delay should work with yielding (2)")

-- Varargs should get passed correctly

local function f(arg1: string, arg2: number, f2: (...any) -> ...any)
	assert(type(arg1) == "string", "Invalid arg 1 passed to function")
	assert(type(arg2) == "number", "Invalid arg 2 passed to function")
	assert(type(arg3) == "function", "Invalid arg 3 passed to function")
end

task.delay(0, f, "", 1, f)
task.delay(0, f, "inf", math.huge, f)
task.delay(0, f, "NaN", 0 / 0, f)

task.wait(0.1)
