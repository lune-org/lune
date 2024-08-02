local task = require("@lune/task")

-- Deferring a task should return the thread that can then be cancelled

local thread = task.defer(function() end)
assert(type(thread) == "thread", "Defer should return the thread spawned")

-- Deferred functions should run after other threads

local flag: boolean = false
task.defer(function()
	flag = true
end)
assert(not flag, "Defer should not run instantly or block")
task.wait(0.05)
assert(flag, "Defer should run")

-- Deferred functions should work with yielding

local flag2: boolean = false
task.defer(function()
	task.wait(0.05)
	flag2 = true
end)
assert(not flag2, "Defer should work with yielding (1)")
task.wait(0.1)
assert(flag2, "Defer should work with yielding (2)")

-- Deferred functions should run after other spawned threads
local flag3: number = 1
task.defer(function()
	if flag3 == 2 then
		flag3 = 3
	end
end)
task.spawn(function()
	if flag3 == 1 then
		flag3 = 2
	end
end)
task.wait()
assert(flag3 == 3, "Defer should run after spawned threads")

-- Defer should be able to be nested

local flag4: boolean = false
task.defer(function()
	local function nested3()
		task.defer(function()
			task.wait(0.05)
			flag4 = true
		end)
	end
	local function nested2()
		task.defer(function()
			task.wait(0.05)
			nested3()
		end)
	end
	local function nested1()
		task.defer(function()
			task.wait(0.05)
			nested2()
		end)
	end
	task.wait(0.05)
	nested1()
end)
task.wait(0.25)
assert(flag4, "Defer should work with nesting")

-- Varargs should get passed correctly

local fcheck = require("./fcheck")

local function f(...: any)
	fcheck(1, "string", select(1, ...))
	fcheck(2, "number", select(2, ...))
	fcheck(3, "function", select(3, ...))
end

task.defer(f, "", 1, f)
task.defer(f, "inf", math.huge, f)
task.defer(f, "NaN", 0 / 0, f)
