local process = require("@lune/process")

-- The exit code of an child process should be correct

local randomExitCode = math.random(0, 255)
local isOk = randomExitCode == 0
local child = process.spawn("exit", { tostring(randomExitCode) }, { shell = true })
local status = child.status()

assert(
	status.code == randomExitCode,
	`Child process exited with wrong exit code, expected {randomExitCode}`
)

assert(status.ok == isOk, `Child status should be {if status.ok then "ok" else "not ok"}`)
