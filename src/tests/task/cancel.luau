-- Cancel should cancel any deferred or delayed threads

local flag: boolean = false
local thread = task.defer(function()
	flag = true
end)
local thread2 = task.delay(0, function()
	flag = true
end)
task.cancel(thread)
task.cancel(thread2)
task.wait(0.1)
assert(not flag, "Cancel should handle non-immediate threads")

-- Cancellation should be as immediate as possible

local flag2: number = 1
task.spawn(function()
	flag2 = 2
	task.wait()
	flag2 = 3
end)
assert(flag2 == 2, "Cancel should properly handle yielding threads")
