-- Spawning a child process should work with options

local result = process.spawn("ls", {
	"-a",
})

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

local before = process.spawn("pwd").stdout
process.spawn("ls", {}, {
	cwd = "/",
	shell = true,
})
local after = process.spawn("pwd").stdout
assert(before == after, "Current working directory changed after running child process")

-- Inheriting stdio & environment variables should work

task.delay(2, function()
	local message = "Hello from child process!"
	local result = process.spawn("echo", {
		'"$TEST_VAR"',
	}, {
		env = { TEST_VAR = message },
		shell = "bash",
		stdio = "inherit",
	})
	assert(
		result.stdout == (message .. "\n"), -- Note that echo adds a newline
		"Inheriting stdio did not return proper output"
	)
end)
