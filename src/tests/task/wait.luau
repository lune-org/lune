-- Wait should be accurate down to at least 10ms

local EPSILON = 1 / 100

local function test(expected: number)
	local start = os.clock()
	local returned = task.wait(expected)
	local elapsed = (os.clock() - start)
	local difference = math.abs(elapsed - returned)
	if difference > EPSILON then
		error(
			string.format(
				"Elapsed time diverged too much from argument!"
					.. "\nGot argument of %.3fs and elapsed time of %.3fs"
					.. "\nGot maximum difference of %.3fs and real difference of %.3fs",
				expected,
				elapsed,
				EPSILON,
				difference
			)
		)
	end
end

local function measure(duration: number)
	for _ = 1, 5 do
		test(duration)
	end
end

measure(1 / 100)
measure(1 / 60)
measure(1 / 30)
measure(1 / 20)
measure(1 / 10)

-- Wait should work in other threads, too

local flag: boolean = false
task.spawn(function()
	task.wait(0.1)
	flag = true
end)
assert(not flag, "Wait failed for a task-spawned thread (1)")
task.wait(0.2)
assert(flag, "Wait failed for a task-spawned thread (2)")

local flag2: boolean = false
coroutine.wrap(function()
	task.wait(0.1)
	flag2 = true
end)()
assert(not flag2, "Wait failed for a coroutine (1)")
task.wait(0.2)
assert(flag2, "Wait failed for a coroutine (2)")
