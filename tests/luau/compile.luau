local luau = require("@lune/luau")

local EMPTY_LUAU_CODE_BLOCK = "do end"
local BROKEN_LUAU_CODE_BLOCK = "do"

assert(type(luau.compile) == "function", "expected `luau.compile` to be a function")

assert(
	type(luau.compile(EMPTY_LUAU_CODE_BLOCK)) == "string",
	"expected `luau.compile` to return bytecode string"
)

local success = pcall(function()
	luau.compile(BROKEN_LUAU_CODE_BLOCK)
end)

assert(success == false, "expected 'BROKEN_LUAU_CODE_BLOCK' to fail to compile into bytecode.")
