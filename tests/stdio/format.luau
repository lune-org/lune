local process = require("@lune/process")
local roblox = require("@lune/roblox")
local stdio = require("@lune/stdio")

local function assertFormatting(errorMessage: string, formatted: string, expected: string)
	if formatted ~= expected then
		stdio.ewrite(string.format("%s\nExpected: %s\nGot: %s", errorMessage, expected, formatted))
		process.exit(1)
	end
end

assertFormatting(
	"Should add a single space between arguments",
	stdio.format("Hello", "world", "!"),
	"Hello world !"
)

assertFormatting(
	"Should format tables in a sorted manner",
	stdio.format({ A = "A", B = "B", C = "C" }),
	'{\n    A = "A",\n    B = "B",\n    C = "C",\n}'
)

assertFormatting(
	"Should format tables properly with single values",
	stdio.format({ Hello = "World" }),
	'{\n    Hello = "World",\n}'
)

assertFormatting(
	"Should format tables properly with multiple values",
	stdio.format({ Hello = "World", Hello2 = "Value" }),
	'{\n    Hello = "World",\n    Hello2 = "Value",\n}'
)

assertFormatting(
	"Should simplify array-like tables and not format keys",
	stdio.format({ "Hello", "World" }),
	'{\n    "Hello",\n    "World",\n}'
)

assertFormatting(
	"Should still format numeric keys for mixed tables",
	stdio.format({ "Hello", "World", Hello = "World" }),
	'{\n    [1] = "Hello",\n    [2] = "World",\n    Hello = "World",\n}'
)

local userdatas = {
	Foo = newproxy(false),
	Bar = (roblox :: any).Vector3.new(),
}

assertFormatting(
	"Should format userdatas as their type (unknown userdata)",
	stdio.format(userdatas.Foo),
	"<userdata>"
)

assertFormatting(
	"Should format userdatas as their type (known userdata)",
	stdio.format(userdatas.Bar),
	"<Vector3>"
)

assertFormatting(
	"Should format userdatas as their type in tables",
	stdio.format(userdatas),
	"{\n    Foo = <userdata>,\n    Bar = <Vector3>,\n}"
)

local nested = {
	Oh = {
		No = {
			TooMuch = {
				Nesting = {
					"Will not print",
				},
			},
		},
	},
}

assert(
	string.find(stdio.format(nested), "Nesting = { ... }", 1, true) ~= nil,
	"Should print 4 levels of nested tables before cutting off"
)
