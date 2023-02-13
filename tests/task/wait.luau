-- Wait should be accurate down to at least 10ms

local EPSILON = 1 / 100

local function test(expected: number)
	local start = os.clock()
	local returned = task.wait(expected)
	if typeof(returned) ~= "number" then
		error(
			string.format(
				"Expected task.wait to return a number, got %s %s",
				typeof(returned),
				stdio.format(returned)
			),
			2
		)
	end
	local elapsed = (os.clock() - start)
	local difference = math.abs(elapsed - expected)
	if difference > EPSILON then
		error(
			string.format(
				"Elapsed time diverged too much from argument!"
					.. "\nGot argument of %.3fms and elapsed time of %.3fms"
					.. "\nGot maximum difference of %.3fms and real difference of %.3fms",
				expected * 1_000,
				elapsed * 1_000,
				EPSILON * 1_000,
				difference * 1_000
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
assert(not flag, "Wait failed while inside task-spawned thread (1)")
task.wait(0.2)
assert(flag, "Wait failed while inside task-spawned thread (2)")

local flag2: boolean = false
coroutine.resume(coroutine.create(function()
	task.wait(0.1)
	flag2 = true
end))
assert(not flag2, "Wait failed while inside coroutine (1)")
task.wait(0.2)
assert(flag2, "Wait failed while inside coroutine (2)")
