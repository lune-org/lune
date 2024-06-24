local process = require("@lune/process")

-- Spawning a child process should not block the main thread

local SAMPLES = 400

for _ = 1, SAMPLES do
	local start = os.time()
	local child = process.create("echo", { "hello, world" })

	assert(child ~= nil, "Failed to spawn child process")

	local delta = os.time() - start
	assert(
		delta <= 1,
		`Spawning a child process should not block the main thread, process.spawn took {delta}s to return when it should return immediately`
	)
end
