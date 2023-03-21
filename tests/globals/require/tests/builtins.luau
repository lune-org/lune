local fs = require("@lune/fs") :: typeof(fs)
local net = require("@lune/net") :: typeof(net)
local process = require("@lune/process") :: typeof(process)
local stdio = require("@lune/stdio") :: typeof(stdio)
local task = require("@lune/task") :: typeof(task)

assert(type(fs.move) == "function")
assert(type(net.request) == "function")
assert(type(process.cwd) == "string")
assert(type(stdio.format("")) == "string")
assert(type(task.spawn(function() end)) == "thread")

assert(not pcall(function()
	return require("@") :: any
end))

assert(not pcall(function()
	return require("@lune") :: any
end))

assert(not pcall(function()
	return require("@lune/") :: any
end))

assert(not pcall(function()
	return require("@src") :: any
end))
