-- Cancel should cancel any deferred or delayed threads

local flag: boolean = false
local thread = task.defer(function()
	flag = true
end)
task.cancel(thread)
task.wait(0.1)
assert(not flag, "Cancel should handle deferred threads")

local flag2: boolean = false
local thread2 = task.delay(0.1, function()
	flag2 = true
end)
task.wait(0)
task.cancel(thread2)
task.wait(0.2)
assert(not flag2, "Cancel should handle delayed threads")

-- Cancellation should work with yields in spawned threads

local flag3: number = 1
local thread3 = task.spawn(function()
	print("1")
	task.wait(0.1)
	flag3 = 2
	print("2")
	task.wait(0.2)
	flag3 = 3
	print("3")
end)
task.wait(0.2)
task.cancel(thread3)
task.wait(0.2)
assert(flag3 == 2, "Cancel should properly handle yielding threads")
