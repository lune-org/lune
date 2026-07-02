local process = require("@lune/process")
local regex = require("@lune/regex")
local roblox = require("@lune/roblox")
local stdio = require("@lune/stdio")

local function assertFormatting(errorMessage: string, formatted: string, expected: string)
	if formatted ~= expected then
		stdio.ewrite(string.format("%s\nExpected: %s\nGot: %s", errorMessage, expected, formatted))
		process.exit(1)
	end
end

local function assertContains(errorMessage: string, haystack: string, needle: string)
	if string.find(haystack, needle) == nil then
		stdio.ewrite(string.format("%s\nHaystack: %s\nNeedle: %s", errorMessage, needle, haystack))
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
	Bar = regex.new("TEST"),
	Baz = (roblox :: any).Vector3.new(1, 2, 3),
}

assertFormatting(
	"Should format userdatas as generic 'userdata' if unknown",
	stdio.format(userdatas.Foo),
	"<userdata>"
)

assertContains(
	"Should format userdatas with their type if they have a __type metafield",
	stdio.format(userdatas.Bar),
	"Regex"
)

assertContains(
	"Should format userdatas with their type even if they have a __tostring metamethod",
	stdio.format(userdatas.Baz),
	"Vector3"
)

assertContains(
	"Should format userdatas with their tostringed value if they have a __tostring metamethod",
	stdio.format(userdatas.Baz),
	"1, 2, 3"
)

assertFormatting(
	"Should format userdatas properly in tables",
	stdio.format(userdatas),
	"{\n    Bar = <Regex(TEST)>,\n    Baz = <Vector3(1, 2, 3)>,\n    Foo = <userdata>,\n}"
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

assertContains(
	"Should print 4 levels of nested tables before cutting off",
	stdio.format(nested),
	"Nesting = { ... }"
)

local _, errorMessage = pcall(function()
	local function innerInnerFn()
		process.exec("PROGRAM_THAT_DOES_NOT_EXIST")
	end
	local function innerFn()
		innerInnerFn()
	end
	innerFn()
end)

stdio.ewrite(typeof(errorMessage))

assertContains("Should format errors similarly to userdata", stdio.format(errorMessage), "<LuaErr")
assertContains("Should format errors with stack begins", stdio.format(errorMessage), "Stack Begin")
assertContains("Should format errors with stack ends", stdio.format(errorMessage), "Stack End")

-- Check that calling stdio.format in a __tostring metamethod by print doesn't cause a deadlock

local inner = {}
setmetatable(inner, {
	__tostring = function()
		return stdio.format(5)
	end,
})

print(inner)

local outer = {}
setmetatable(outer, {
	__tostring = function()
		return stdio.format(inner)
	end,
})

print(outer)
