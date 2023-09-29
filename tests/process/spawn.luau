local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- Spawning a child process should work, with options

local thread = task.delay(1, function()
	stdio.ewrite("Spawning a process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local result = process.spawn("ls", {
	"-a",
})

task.cancel(thread)

assert(result.ok, "Failed to spawn child process")

assert(result.stderr == "", "Stderr was not empty")
assert(result.stdout ~= "", "Stdout was empty")

assert(string.find(result.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output")
assert(string.find(result.stdout, ".gitignore") ~= nil, "Missing .gitignore in output")

-- It should also work the same when spawned using a shell

local shellResult = process.spawn("ls", {
	"-a",
}, {
	shell = true,
})

assert(shellResult.ok, "Failed to spawn child process (shell)")

assert(shellResult.stderr == "", "Stderr was not empty (shell)")
assert(shellResult.stdout ~= "", "Stdout was empty (shell)")

assert(string.find(shellResult.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output (shell)")
assert(string.find(shellResult.stdout, ".gitignore") ~= nil, "Missing .gitignore in output (shell)")

-- Make sure the cwd option actually uses the directory we want
local rootPwd = process.spawn("pwd", {}, {
	cwd = "/",
}).stdout
rootPwd = string.gsub(rootPwd, "^%s+", "")
rootPwd = string.gsub(rootPwd, "%s+$", "")
if rootPwd ~= "/" then
	error(
		string.format(
			"Current working directory for child process was not set correctly!"
				.. "\nExpected '/', got '%s'",
			rootPwd
		)
	)
end

-- Setting cwd should not change the cwd of this process

local pwdBefore = process.spawn("pwd").stdout
process.spawn("ls", {}, {
	cwd = "/",
	shell = true,
})
local pwdAfter = process.spawn("pwd").stdout
assert(pwdBefore == pwdAfter, "Current working directory changed after running child process")

--[[
	Setting the cwd on a child process should properly
	replace any leading ~ with the users real home dir
]]

local homeDir1 = process.spawn("echo $HOME", nil, {
	shell = true,
}).stdout
local homeDir2 = process.spawn("pwd", nil, {
	shell = true,
	cwd = "~",
}).stdout

assert(#homeDir1 > 0, "Home dir from echo was empty")
assert(#homeDir2 > 0, "Home dir from pwd was empty")
assert(homeDir1 == homeDir2, "Home dirs did not match when performing tilde substitution")

--[[
	Spawning a process should not block any lua thread(s)

	We test this by sleeping more than once concurrently
	and then ensuring that the total time slept is more
	than a single sleep but also less than 1.5 sleeps
]]

local SLEEP_DURATION = 1 / 4
local SLEEP_SAMPLES = 2

local thread2 = task.delay(SLEEP_DURATION * 1.5, function()
	stdio.ewrite("Spawning a sleep process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local sleepStart = os.clock()
local sleepCounter = 0
for i = 1, SLEEP_SAMPLES, 1 do
	task.spawn(function()
		process.spawn("sleep", { tostring(SLEEP_DURATION) })
		sleepCounter += 1
	end)
end
while sleepCounter < SLEEP_SAMPLES do
	task.wait()
end

task.cancel(thread2)

local sleepElapsed = os.clock() - sleepStart
assert(
	sleepElapsed >= SLEEP_DURATION,
	"Spawning a process that does blocking sleep did not sleep enough"
)
assert(
	sleepElapsed < SLEEP_DURATION * 1.5,
	"Coroutine yielded the main lua thread during process yield"
)

-- Inheriting stdio & environment variables should work

local echoMessage = "Hello from child process!"
local echoResult = process.spawn("echo", {
	'"$TEST_VAR"',
}, {
	env = { TEST_VAR = echoMessage },
	shell = "bash",
	stdio = "inherit",
})
assert(
	echoResult.stdout == (echoMessage .. "\n"), -- Note that echo adds a newline
	"Inheriting stdio did not return proper output"
)

-- Passing stdin strings should work

local isWindows = process.os == "windows"

local stdinChild = process.spawn((((not isWindows) and "xargs") or "powershell"), {
	"echo",
}, {
	stdin = echoMessage,
})

assert(
	stdinChild.stdout == (echoMessage .. "\n"), -- Note that echo adds a newline
	"Stdin passing did not return proper output"
)
