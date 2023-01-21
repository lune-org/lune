assert(
	console.format("Hello", "world", "!") == "Hello world !",
	"Format should add a single space between arguments"
)

assert(
	console.format({ Hello = "World" }) == '{\n    Hello = "World",\n}',
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
	string.find(console.format(nested), "Nesting = { ... }", 1, true) ~= nil,
	"Format should print 4 levels of nested tables before cutting off"
)
