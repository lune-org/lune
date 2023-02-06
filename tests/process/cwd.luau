assert(process.cwd ~= nil, "Process cwd is missing")

assert(type(process.cwd) == "string", "Process cwd is not a string")

assert(#process.cwd > 0, "Process cwd is an empty string")

assert(string.sub(process.cwd, -1) == "/", "Process cwd does not end with '/'")
