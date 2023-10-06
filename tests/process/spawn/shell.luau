local process = require("@lune/process")

local IS_WINDOWS = process.os == "windows"

-- Default shell should be /bin/sh on unix and powershell on Windows,
-- note that powershell needs slightly different command flags for ls

local shellResult = process.spawn("ls", {
	if IS_WINDOWS then "-Force" else "-a",
}, {
	shell = true,
})

assert(shellResult.ok, "Failed to spawn child process (shell)")

assert(shellResult.stderr == "", "Stderr was not empty (shell)")
assert(shellResult.stdout ~= "", "Stdout was empty (shell)")

assert(string.find(shellResult.stdout, "Cargo.toml") ~= nil, "Missing Cargo.toml in output (shell)")
assert(string.find(shellResult.stdout, ".gitignore") ~= nil, "Missing .gitignore in output (shell)")
