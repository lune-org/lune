local luau = require("@lune/luau")

local EMPTY_LUAU_CODE_BLOCK = "do end"
local BROKEN_LUAU_CODE_BLOCK = "do"

assert(type(luau.compile) == "function", "expected `luau.compile` to be a function")

assert(
	type(luau.compile(EMPTY_LUAU_CODE_BLOCK)) == "string",
	"expected `luau.compile` to return bytecode string"
)

assert(
	string.byte(luau.compile(BROKEN_LUAU_CODE_BLOCK), 1, 1) == 0,
	"expected errors to return with \0 byte, error message attached afterwards."
)
