print("Hello, lune! 🌙")

-- Use a function from another module

local module = require("scripts/module")
module.hello()

-- Read and print out directories & files in
-- the current directory, with fancy icons

print("\nReading current dir...")
local entries = fs.readDir(".")

-- NOTE: We have to do this outside of the sort function
-- to avoid yielding across the metamethod boundary, any
-- calls to fs functions may yield for any reason
local dirs = {}
for _, entry in entries do
	dirs[entry] = fs.isDir(entry)
end

-- Sort directories first, then alphabetically
table.sort(entries, function(entry0, entry1)
	if dirs[entry0] ~= dirs[entry1] then
		return dirs[entry0]
	end
	return entry0 < entry1
end)

assert(table.find(entries, "Cargo.toml") ~= nil, "Missing Cargo.toml")
assert(table.find(entries, "Cargo.lock") ~= nil, "Missing Cargo.lock")

for _, entry in entries do
	if fs.isDir(entry) then
		print("📁 " .. entry)
	else
		print("📄 " .. entry)
	end
end

-- Read and print out environment variables

print("\nReading current environment...")
local vars = process.getEnvVars()
table.sort(vars)

assert(table.find(vars, "PATH") ~= nil, "Missing PATH")
assert(table.find(vars, "PWD") ~= nil, "Missing PWD")

for _, key in vars do
	local value = process.getEnvVar(key)
	local box = if value and value ~= "" then "✅" else "❌"
	print(string.format("[%s] %s", box, key))
end

-- Call out to another program

print("\nSending 4 pings to google...")
local result = process.spawn("ping", {
	"google.com",
	"-c 4",
})

if result.ok then
	print(result.stdout)
else
	print(result.stderr)
	process.exit(result.code)
end

print("\nGoodbye, lune! 🌙")
