print("Hello, lune! 🌙")

--[==[
	EXAMPLE #1

	Using a function from another module
]==]

local module = require(".lune/module")
module.sayHello()

--[==[
	EXAMPLE #2

	Using arguments given to the program
]==]

local arg: string? = ...
if arg then
	print("\nGot an argument while running hello_lune:")
	print(arg)
end

--[==[
	EXAMPLE #3

	Get & set environment variables

	Checks if environment variables are empty or not,
	prints out ❌ if empty and ✅ if they have a value
]==]

print("\nReading current environment 🔎")
local vars = process.getEnvVars()
table.sort(vars)

assert(table.find(vars, "PATH") ~= nil, "Missing PATH")
assert(table.find(vars, "PWD") ~= nil, "Missing PWD")

for _, key in vars do
	local value = process.getEnvVar(key)
	local box = if value and value ~= "" then "✅" else "❌"
	print(string.format("[%s] %s", box, key))
end

--[==[
	EXAMPLE #4

	Read files in the current directory

	This prints out directory & file names with some fancy icons
]==]

print("\nReading current dir 🗂️")
local entries = fs.readDir(".")

-- NOTE: We have to do this outside of the sort function
-- to avoid yielding across the metamethod boundary, any
-- calls to fs functions may yield for any reason
local entryIsDir = {}
for _, entry in entries do
	entryIsDir[entry] = fs.isDir(entry)
end

-- Sort prioritizing directories first, then alphabetically
table.sort(entries, function(entry0, entry1)
	if entryIsDir[entry0] ~= entryIsDir[entry1] then
		return entryIsDir[entry0]
	end
	return entry0 < entry1
end)

-- Make sure we got some known files that should always exist
assert(table.find(entries, "Cargo.toml") ~= nil, "Missing Cargo.toml")
assert(table.find(entries, "Cargo.lock") ~= nil, "Missing Cargo.lock")

-- Print the pretty stuff
for _, entry in entries do
	if fs.isDir(entry) then
		print("📁 " .. entry)
	else
		print("📄 " .. entry)
	end
end

-- NOTE: We skip the last example in GitHub Actions
-- since the ping command does not work in azure
if process.getEnvVar("GITHUB_ACTIONS") then
	print("\nGoodbye, lune! 🌙")
	process.exit(0)
end

--[==[
	EXAMPLE #5

	Call out to another program / executable

	Here we send some pings to google to demonstrate that programs
	that yield or perform any network requests work correctly
]==]

print("\nSending 4 pings to google 🌏")
local result = process.spawn("ping", {
	"google.com",
	"-c 4",
})

--[==[
	EXAMPLE #6

	Using the result of a spawned process, exiting the process

	We use the result from the above ping command and parse
	it to show the results it gave us in a nicer format, here we
	also exit with an error (exit code 1) if spawning the process failed
]==]

if result.ok then
	assert(#result.stdout > 0, "Result output was empty")
	local min, avg, max, stddev = string.match(
		result.stdout,
		"min/avg/max/stddev = ([%d%.]+)/([%d%.]+)/([%d%.]+)/([%d%.]+) ms"
	)
	print(string.format("Minimum ping time: %.3fms", assert(tonumber(min))))
	print(string.format("Maximum ping time: %.3fms", assert(tonumber(max))))
	print(string.format("Average ping time: %.3fms", assert(tonumber(avg))))
	print(string.format("Standard deviation: %.3fms", assert(tonumber(stddev))))
else
	print("\nFailed to send ping to google!")
	print(result.stderr)
	process.exit(result.code)
end

--[==[
	EXAMPLE #7

	Using the built-in networking library
]==]
print("\nSending PATCH request to web API 📤")
local apiResult = net.request({
	url = "https://jsonplaceholder.typicode.com/posts/1",
	method = "PATCH",
	headers = {
		["Content-Type"] = "application/json",
	},
	body = net.jsonEncode({
		title = "foo",
		body = "bar",
	}),
})

if not result.ok then
	print("\nFailed to send network request!")
	print(string.format("%d (%s)", apiResult.statusCode, apiResult.statusMessage))
	print(apiResult.body)
	process.exit(1)
end

type ApiResponse = {
	id: number,
	title: string,
	body: string,
	userId: number,
}

local apiResponse: ApiResponse = net.jsonDecode(apiResult.body)
assert(apiResponse.title == "foo", "Invalid json response")
assert(apiResponse.body == "bar", "Invalid json response")
print("Got valid JSON response with changes applied")

--[==[
	Example #8

	Saying goodbye 😔
]==]

print("\nGoodbye, lune! 🌙")
