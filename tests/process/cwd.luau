local process = require("@lune/process")

assert(process.cwd ~= nil, "Process cwd is missing")

assert(type(process.cwd) == "string", "Process cwd is not a string")

assert(#process.cwd > 0, "Process cwd is an empty string")

if process.os == "windows" then
	assert(string.sub(process.cwd, -1) == "\\", "Process cwd does not end with '\\'")
else
	assert(string.sub(process.cwd, -1) == "/", "Process cwd does not end with '/'")
end
