local luau = require("@lune/luau")

assert(type(luau.compile) == "function", "expected `luau.compile` to be a function")

assert(
	type(luau.compile("do end")) == "string",
	"expected `luau.compile` to return bytecode string"
)
