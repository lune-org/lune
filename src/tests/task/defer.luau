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

local function fcheck(index: number, type: string, value: any)
	if typeof(value) ~= type then
		console.error(
			string.format(
				"Expected argument #%d to be of type %s, got %s",
				index,
				type,
				console.format(value)
			)
		)
		process.exit(1)
	end
end

local function f(...: any)
	fcheck(1, "string", select(1, ...))
	fcheck(2, "number", select(2, ...))
	fcheck(3, "function", select(3, ...))
end

task.defer(f, "", 1, f)
task.defer(f, "inf", math.huge, f)
task.defer(f, "NaN", 0 / 0, f)
