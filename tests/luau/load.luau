local luau = require("@lune/luau")

local RETURN_VALUE = 1

local EMPTY_LUAU_CODE_BLOCK = "do end"
local RETURN_LUAU_CODE_BLOCK = "return " .. tostring(RETURN_VALUE)

local CUSTOM_SOURCE_BLOCK_NAME = "test"

assert(type(luau.load) == "function", "expected `luau.compile` to be a function")

assert(
	type(luau.load(EMPTY_LUAU_CODE_BLOCK)) == "function",
	"expected 'luau.load' to return a function"
)
assert(
	luau.load(RETURN_LUAU_CODE_BLOCK)() == RETURN_VALUE,
	"expected 'luau.load' to return a value"
)

local sourceFunction = luau.load(EMPTY_LUAU_CODE_BLOCK, { debugName = CUSTOM_SOURCE_BLOCK_NAME })
local sourceFunctionDebugName = debug.info(sourceFunction, "s")

assert(
	string.find(sourceFunctionDebugName, CUSTOM_SOURCE_BLOCK_NAME),
	"expected source block name for 'luau.load' to return a custom debug name"
)

local success = pcall(function()
	luau.load(luau.compile(RETURN_LUAU_CODE_BLOCK))
end)

assert(success, "expected `luau.load` to be able to process the result of `luau.compile`")
