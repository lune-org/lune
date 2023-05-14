local stdio = require("@lune/stdio")

assert(
	stdio.format("Hello", "world", "!") == "Hello world !",
	"Format should add a single space between arguments"
)

assert(
	stdio.format({ Hello = "World" }) == '{\n    Hello = "World",\n}',
	"Format should print out proper tables"
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
	"Format should print 4 levels of nested tables before cutting off"
)
