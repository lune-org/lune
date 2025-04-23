local task = require("@lune/task")

local started = os.clock()

local amount = 400000
local batches = 5
local per_batch = amount / batches

for current = 1, batches do
	local thread = coroutine.running()

	print(`Batch {current} / {batches}`)

	for i = 1, per_batch do
		--print("Spawning thread #" .. i)
		task.spawn(function()
			print("[BEFORE] Thread number", i)
			task.wait(0.1)
			--_TEST_ASYNC_WORK(0.1)
			print("[AFTER] Thread number", i)
			if i == per_batch then
				print("Last thread in batch #" .. current)
				assert(
					coroutine.status(thread) == "suspended",
					`Thread {i} has status {coroutine.status(thread)}`
				)
				task.spawn(thread)
			end
		end)
	end

	coroutine.yield()
end
local took = os.clock() - started
print(`Spawned {amount} sleeping threads in {took}s`)
