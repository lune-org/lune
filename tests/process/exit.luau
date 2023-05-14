local fs = require("@lune/fs")
local task = require("@lune/task")
local process = require("@lune/process")

local function assert(condition, err)
	if not condition then
		task.spawn(error, err)
		process.exit(0)
	end
end

local path = process.cwd .. "asdfghjkl"

assert(fs.isDir(path), "Process should exit with success")
assert(fs.isDir(path), "Process should exit with success")
assert(fs.isDir(path), "Process should exit with success")

error("Process should have exited with success...")
