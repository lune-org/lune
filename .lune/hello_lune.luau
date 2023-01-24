print("Hello, lune! 🌙")

--[==[
	EXAMPLE #1

	Using a function from another module
]==]

local module = require("./module")
module.sayHello()

--[==[
	EXAMPLE #2

	Using arguments given to the program
]==]

if #process.args > 0 then
	print("\nGot arguments while running hello_lune:")
	console.log(process.args)
	if #process.args > 3 then
		error("Too many arguments!")
	end
end

--[==[
	EXAMPLE #3

	Spawning tasks
]==]

task.spawn(function()
	print("\nSpawned a task that will run instantly but not block")
	task.wait(5)
end)

print("Spawning a delayed task that will run in 5 seconds")
task.delay(5, function()
	print("\n...")
	task.wait(1)
	print("Hello again!")
	task.wait(1)
	print("Goodbye again! 🌙")
end)

--[==[
	EXAMPLE #4

	Get & set environment variables

	Checks if environment variables are empty or not,
	prints out ❌ if empty and ✅ if they have a value
]==]

print("\nReading current environment 🔎")

-- Environment variables can be read directly
assert(process.env.PATH ~= nil, "Missing PATH")
assert(process.env.PWD ~= nil, "Missing PWD")

-- And they can also be accessed using generalized iteration (not pairs!)
for key, value in process.env do
	local box = if value and value ~= "" then "✅" else "❌"
	print(string.format("[%s] %s", box, key))
end

--[==[
	EXAMPLE #5

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

-- NOTE: We skip the ping example in GitHub Actions
-- since the ping command does not work in azure
if not process.env.GITHUB_ACTIONS then
	--[==[
		EXAMPLE #6

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
		EXAMPLE #7

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
end

--[==[
	EXAMPLE #8

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

if not apiResult.ok then
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
	EXAMPLE #9

	Using the console library to print pretty
]==]

print("\nPrinting with pretty colors and auto-formatting 🎨")

console.setColor("blue")
print(string.rep("—", 22))
console.resetColor()

console.info("API response:", apiResponse)
console.warn({
	Oh = {
		No = {
			TooMuch = {
				Nesting = {
					"Will not print",
				},
			},
		},
	},
})

console.setColor("blue")
print(string.rep("—", 22))
console.resetColor()

--[==[
	EXAMPLE #10

	Saying goodbye 😔
]==]

print("\nGoodbye, lune! 🌙")
