<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD026 -->

# ✏️ Writing Lune Scripts

If you've already written some version of Lua (or Luau) scripts before, this walkthrough will make you feel right at home.

Once you have a script you want to run, head over to the [Running Scripts](https://github.com/filiptibell/lune/wiki/Getting-Started---3-Running-Scripts) page.

## Hello, Lune!

```lua
--[[
	EXAMPLE #1

	Using arguments given to the program
]]

if #process.args > 0 then
	print("Got arguments:")
	print(process.args)
	if #process.args > 3 then
		error("Too many arguments!")
	end
else
    print("Got no arguments ☹️")
end



--[[
	EXAMPLE #2

	Using the stdio library to prompt for terminal input
]]

local text = stdio.prompt("text", "Please write some text")

print("You wrote '" .. text .. "'!")

local confirmed = stdio.prompt("confirm", "Please confirm that you wrote some text")
if confirmed == false then
	error("You didn't confirm!")
else
	print("Confirmed!")
end



--[[
	EXAMPLE #3

	Get & set environment variables

	Checks if environment variables are empty or not,
	prints out ❌ if empty and ✅ if they have a value
]]

print("Reading current environment 🔎")

-- Environment variables can be read directly
assert(process.env.PATH ~= nil, "Missing PATH")
assert(process.env.PWD ~= nil, "Missing PWD")

-- And they can also be accessed using Luau's generalized iteration (but not pairs())
for key, value in process.env do
	local box = if value and value ~= "" then "✅" else "❌"
	print(string.format("[%s] %s", box, key))
end



--[[
	EXAMPLE #4

	Writing a module

    Modularizing and splitting up your code is Lune is very straight-forward,
    in contrast to other scripting languages and shells such as bash
]]

local module = {}

function module.sayHello()
    print("Hello, Lune! 🌙")
end

return module



--[[
	EXAMPLE #5

	Using a function from another module / script

    Lune has path-relative imports, similar to other popular languages such as JavaScript
]]

local module = require("../modules/module")
module.sayHello()



--[[
	EXAMPLE #6

	Spawning concurrent tasks

    These tasks will run at the same time as other Lua code which lets you do primitive multitasking
]]

task.spawn(function()
	print("Spawned a task that will run instantly but not block")
	task.wait(5)
end)

print("Spawning a delayed task that will run in 5 seconds")
task.delay(5, function()
	print("...")
	task.wait(1)
	print("Hello again!")
	task.wait(1)
	print("Goodbye again! 🌙")
end)



--[[
	EXAMPLE #7

	Read files in the current directory

	This prints out directory & file names with some fancy icons
]]

print("Reading current dir 🗂️")
local entries = fs.readDir(".")

-- NOTE: We have to do this outside of the sort function
-- to avoid yielding across the metamethod boundary, all
-- of the filesystem APIs are asynchronous and yielding
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



--[[
	EXAMPLE #8

	Call out to another program / executable

    You can also get creative and combine this with example #6 to spawn several programs at the same time!
]]

print("Sending 4 pings to google 🌏")
local result = process.spawn("ping", {
	"google.com",
	"-c 4",
})



--[[
	EXAMPLE #9

	Using the result of a spawned process, exiting the process

	This looks scary with lots of weird symbols, but, it's just some Lua-style pattern matching
    to parse the lines of "min/avg/max/stddev = W/X/Y/Z ms" that the ping program outputs to us
]]

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
	print("Failed to send ping to google!")
	print(result.stderr)
	process.exit(result.code)
end



--[[
	EXAMPLE #10

	Using the built-in networking library, encoding & decoding json
]]

print("Sending PATCH request to web API 📤")
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
	print("Failed to send network request!")
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



--[[
	EXAMPLE #11

	Using the stdio library to print pretty
]]

print("Printing with pretty colors and auto-formatting 🎨")

print(stdio.color("blue") .. string.rep("—", 22) .. stdio.color("reset"))

info("API response:", apiResponse)
warn({
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

print(stdio.color("blue") .. string.rep("—", 22) .. stdio.color("reset"))



--[[
	EXAMPLE #12

	Saying goodbye 😔
]]

print("Goodbye, lune! 🌙")

```

More real-world examples of how to write Lune scripts can be found in the [examples](https://github.com/filiptibell/lune/blob/main/.lune/examples/) folder.

Documentation for individual APIs and types can be found in "API Reference" in the sidebar of this wiki.

## Extras

### 🔀 Example translation from Bash

```bash
#!/bin/bash
VALID=true
COUNT=1
while [ $VALID ]
do
    echo $COUNT
    if [ $COUNT -eq 5 ];
    then
        break
    fi
    ((COUNT++))
done
```

**_With Lune & Luau:_**

```lua
local valid = true
local count = 1
while valid do
    print(count)
    if count == 5 then
        break
    end
    count += 1
end
```
