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

local CUSTOM_SOURCE_WITH_FOO_FN = "return foo()"

-- NOTE: We use newproxy here to make a userdata to ensure
-- we get the *exact* same value sent back, not some copy
local fooValue = newproxy(false)
local fooFn = luau.load(CUSTOM_SOURCE_WITH_FOO_FN, {
	environment = {
		foo = function()
			return fooValue
		end,
	},
})

local fooFnRet = fooFn()
assert(fooFnRet == fooValue, "expected `luau.load` with custom environment to return proper values")

local CUSTOM_SOURCE_WITH_PRINT_FN = "return print()"

-- NOTE: Same as what we did above, new userdata to guarantee unique-ness
local overriddenValue = newproxy(false)
local overriddenFn = luau.load(CUSTOM_SOURCE_WITH_PRINT_FN, {
	environment = {
		print = function()
			return overriddenValue
		end,
	},
})

local overriddenFnRet = overriddenFn()
assert(
	overriddenFnRet == overriddenValue,
	"expected `luau.load` with overridden environment to return proper values"
)

local CUSTOM_SOURCE_WITH_DEFAULT_FN = "return string.lower(...)"

local overriddenFn2 = luau.load(CUSTOM_SOURCE_WITH_DEFAULT_FN, {
	environment = {
		hello = "world",
	},
})

local overriddenFn2Ret = overriddenFn2("LOWERCASE")
assert(
	overriddenFn2Ret == "lowercase",
	"expected `luau.load` with overridden environment to contain default globals"
)
