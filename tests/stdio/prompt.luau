local task = require("@lune/task")
local stdio = require("@lune/stdio")
local process = require("@lune/process")

-- NOTE: This test is intentionally not included in the
-- automated tests suite since it requires user input

local passed = false
task.delay(0.2, function()
	if passed then
		task.spawn(error, "Prompt must not block other lua threads")
		process.exit(1)
	else
		-- stdio.ewrite("Hello from concurrent task!")
	end
end)

-- Text prompt

local text = stdio.prompt("text", "Type some text")
assert(#text > 0, "Did not get any text")
print(`Got text '{text}'\n`)

passed = true

-- Confirmation prompt

local confirmed = stdio.prompt("confirm", "Please confirm", true)
assert(type(confirmed) == "boolean", "Did not get a boolean as result")
print(if confirmed then "Confirmed\n" else "Did not confirm\n")

-- Selection prompt

local option = stdio.prompt(
	"select",
	"Please select the first option from the list",
	{ "one", "two", "three", "four" }
)
assert(option == 1, "Did not get the first option as result")
print(`Got option #{option}\n`)

-- Multi-selection prompt

local options = stdio.prompt(
	"multiselect",
	"Please select options two and four",
	{ "one", "two", "three", "four", "five" }
)
assert(
	options ~= nil and table.find(options, 2) and table.find(options, 4),
	"Did not get options 2 and 4 as result"
)
print(`Got option(s) {stdio.format(options)}\n`)
