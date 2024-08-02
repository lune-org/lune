local task = require("@lune/task")

-- Delaying a task should return the thread that can then be cancelled

local thread = task.delay(0, function() end)
assert(type(thread) == "thread", "Delay should return the thread spawned")

-- Delayed functions should never run right away

local flag: boolean = false
task.delay(0, function()
	flag = true
end)
assert(not flag, "Delay should not run instantly or block")
task.wait(0.05)
assert(flag, "Delay should run after the wanted duration")

-- Delayed functions should work with yielding

local flag2: boolean = false
task.delay(0.05, function()
	flag2 = true
	task.wait(0.1)
	flag2 = false
end)
task.wait(0.1)
assert(flag, "Delay should work with yielding (1)")
task.wait(0.1)
assert(not flag2, "Delay should work with yielding (2)")

-- Delay should be able to be nested

local flag4: boolean = false
task.delay(0.05, function()
	local function nested3()
		task.delay(0.05, function()
			flag4 = true
		end)
	end
	local function nested2()
		task.delay(0.05, nested3)
	end
	local function nested1()
		task.delay(0.05, nested2)
	end
	nested1()
end)
task.wait(0.25)
assert(flag4, "Delay should work with nesting")

-- Varargs should get passed correctly

local fcheck = require("./fcheck")

local function f(...: any)
	fcheck(1, "string", select(1, ...))
	fcheck(2, "number", select(2, ...))
	fcheck(3, "function", select(3, ...))
end

task.delay(0, f, "", 1, f)
task.delay(0, f, "inf", math.huge, f)
task.delay(0, f, "NaN", 0 / 0, f)
