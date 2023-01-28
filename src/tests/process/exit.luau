local function assert(condition, err)
	if not condition then
		console.error(err)
		process.exit(0)
	end
end

local path = process.cwd .. "asdfghjkl"

assert(fs.isDir(path), "Process should exit")
assert(fs.isDir(path), "Process should exit")
assert(fs.isDir(path), "Process should exit")

error("Process should have exited...")
