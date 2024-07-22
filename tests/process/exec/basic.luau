local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- Executing a command should work, with options

local thread = task.delay(1, function()
	stdio.ewrite("Spawning a process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local IS_WINDOWS = process.os == "windows"

local result = process.exec(
	if IS_WINDOWS then "cmd" else "ls",
	if IS_WINDOWS then { "/c", "dir" } else { "-a" }
)

task.cancel(thread)

assert(result.ok, "Failed to spawn child process")

assert(result.stderr == "", "Stderr was not empty")
assert(result.stdout ~= "", "Stdout was empty")

assert(string.find(result.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output")
assert(string.find(result.stdout, ".gitignore") ~= nil, "Missing .gitignore in output")
