local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local IS_WINDOWS = process.os == "windows"

-- Spawning a process should not block any lua thread(s)

local SLEEP_DURATION = 1 / 4
local SLEEP_SAMPLES = 2

local thread2 = task.delay(30, function()
	stdio.ewrite("Spawning a sleep process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local sleepStart = os.clock()
local sleepCounter = 0
for i = 1, SLEEP_SAMPLES, 1 do
	task.spawn(function()
		local args = {
			-- Sleep command on Windows in Seconds has some weird behavior with decimals ...
			tostring(SLEEP_DURATION * (IS_WINDOWS and 1000 or 1)),
		}
		if IS_WINDOWS then
			-- ... so we use milliseconds instead.
			table.insert(args, 1, "-Milliseconds")
		end
		-- Windows does not have `sleep` as a process, so we use powershell instead.
		process.spawn("sleep", args, if IS_WINDOWS then { shell = true } else nil)
		sleepCounter += 1
	end)
end
while sleepCounter < SLEEP_SAMPLES do
	task.wait()
end

task.cancel(thread2)

assert(
	(os.clock() - sleepStart) >= SLEEP_DURATION,
	"Spawning a process that does blocking sleep did not sleep enough"
)
