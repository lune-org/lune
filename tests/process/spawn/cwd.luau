local process = require("@lune/process")

local IS_WINDOWS = process.os == "windows"

local pwdCommand = if IS_WINDOWS then "cmd" else "pwd"
local pwdArgs = if IS_WINDOWS then { "/c", "cd" } else {}

-- Make sure the cwd option actually uses the directory we want
local rootPwd = process.spawn(pwdCommand, pwdArgs, {
	cwd = "/",
}).stdout
rootPwd = string.gsub(rootPwd, "^%s+", "")
rootPwd = string.gsub(rootPwd, "%s+$", "")

-- Windows: <Drive Letter>:\, Unix: /
local expectedRootPwd = if IS_WINDOWS then string.sub(rootPwd, 1, 1) .. ":\\" else "/"
if rootPwd ~= expectedRootPwd then
	error(
		string.format(
			"Current working directory for child process was not set correctly!"
				.. "\nExpected '%s', got '%s'",
			expectedRootPwd,
			rootPwd
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

-- Setting the cwd on a child process should properly
-- replace any leading ~ with the users real home dir

local homeDir1 = process.spawn("echo $HOME", nil, {
	shell = true,
}).stdout

-- NOTE: Powershell for windows uses `$pwd.Path` instead of `pwd` as pwd would return
-- a PathInfo object, using $pwd.Path gets the Path property of the PathInfo object
local homeDir2 = process.spawn(if IS_WINDOWS then "$pwd.Path" else "pwd", nil, {
	shell = true,
	cwd = "~",
}).stdout

assert(#homeDir1 > 0, "Home dir from echo was empty")
assert(#homeDir2 > 0, "Home dir from pwd was empty")
assert(homeDir1 == homeDir2, "Home dirs did not match when performing tilde substitution")
