local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- Spawning a child process should work, with options

local thread = task.delay(1, function()
	stdio.ewrite("Spawning a process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local isWindows = process.os == "windows";

-- To run windows command, needs to launch cmd.exe
local result = process.spawn(
	if isWindows then "cmd" else "ls",
	if isWindows then {
		"/c", "dir"
	} else {
		"-a"
	}
)

task.cancel(thread)

assert(result.ok, "Failed to spawn child process")

assert(result.stderr == "", "Stderr was not empty")
assert(result.stdout ~= "", "Stdout was empty")

assert(string.find(result.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output")
assert(string.find(result.stdout, ".gitignore") ~= nil, "Missing .gitignore in output")

-- It should also work the same when spawned using a shell

local shellResult = process.spawn("ls", {
	if isWindows then "-Force" else "-a"
}, {
	shell = true,
})

assert(shellResult.ok, "Failed to spawn child process (shell)")

assert(shellResult.stderr == "", "Stderr was not empty (shell)")
assert(shellResult.stdout ~= "", "Stdout was empty (shell)")

assert(string.find(shellResult.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output (shell)")
assert(string.find(shellResult.stdout, ".gitignore") ~= nil, "Missing .gitignore in output (shell)")

local pwdCommand = if isWindows then "cmd" else "pwd"
local pwdArgs = if isWindows then { "/c", "cd" } else {}
-- Make sure the cwd option actually uses the directory we want
local rootPwd = process.spawn(pwdCommand, pwdArgs, {
	cwd = "/",
}).stdout
rootPwd = string.gsub(rootPwd, "^%s+", "")
rootPwd = string.gsub(rootPwd, "%s+$", "")
-- Windows: C:\, Unix: /
local expectedRootPwd = if isWindows then string.sub(rootPwd, 1, 1) .. ":\\" else "/"
if rootPwd ~= expectedRootPwd then
	error(
		string.format(
			"Current working directory for child process was not set correctly!"
				.. "\nExpected '%s', got '%s'",
			expectedRootPwd, rootPwd
		)
	)
end

-- Setting cwd should not change the cwd of this process

local pwdBefore = process.spawn(pwdCommand, pwdArgs).stdout
process.spawn("ls", {}, {
	cwd = "/",
	shell = true,
})
local pwdAfter = process.spawn(pwdCommand, pwdArgs).stdout
assert(pwdBefore == pwdAfter, "Current working directory changed after running child process")

--[[
	Setting the cwd on a child process should properly
	replace any leading ~ with the users real home dir
]]

local homeDir1 = process.spawn("echo $HOME", nil, {
	shell = true,
}).stdout

-- Powershell for windows uses `$pwd.Path` instead of `pwd` as pwd would return a PathInfo object,
-- using $pwd.Path gets the Path property of the PathInfo object.
local homeDir2 = process.spawn(if isWindows then "$pwd.Path" else "pwd", nil, {
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

-- We need to account for the time it takes to spawn a process, because of Windows
local commandExecutionTolarance = 0 do
	local commandExecutionStart = os.clock()
	process.spawn("sleep", {"0"}, if isWindows then { shell = true } else nil)
	commandExecutionTolarance = os.clock() - commandExecutionStart
end

local SLEEP_DURATION = 1 / 4
local SLEEP_SAMPLES = 2

local thread2 = task.delay(commandExecutionTolarance + (SLEEP_DURATION * 1.5), function()
	stdio.ewrite("Spawning a sleep process should take a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local sleepStart = os.clock()
local sleepCounter = 0
for i = 1, SLEEP_SAMPLES, 1 do
	task.spawn(function()
		-- Windows does not have a sleep command, so we use shell instead.
		local args = {
			tostring(SLEEP_DURATION * (isWindows and 1000 or 1))
		};
		if isWindows then
			-- Sleep command on Windows in Seconds has some weird behavior with decimals.
			table.insert(args, 1, "-Milliseconds");
		end
		process.spawn("sleep", args, if isWindows then { shell = true } else nil)
		sleepCounter += 1
	end)
end
while sleepCounter < SLEEP_SAMPLES do
	task.wait()
end

task.cancel(thread2)

local sleepElapsed = (os.clock() - sleepStart) - commandExecutionTolarance
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
	if isWindows then '"$Env:TEST_VAR"' else '"$TEST_VAR"',
}, {
	env = { TEST_VAR = echoMessage },
	shell = if isWindows then true else "bash", -- Windows needs a default shell, bash does not exist on Windows
	stdio = "inherit",
})

local trailingAddition = if isWindows then "\r\n" else "\n"
assert(
	echoResult.stdout == (echoMessage .. trailingAddition), -- Note that echo adds a newline
	"Inheriting stdio did not return proper output"
)
