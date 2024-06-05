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

local loadSuccess = pcall(function()
	luau.load(luau.compile(RETURN_LUAU_CODE_BLOCK))
end)

assert(loadSuccess, "expected `luau.load` to be able to process the result of `luau.compile`")

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

local fooValue2 = newproxy(false)
local fooFn2 = luau.load(CUSTOM_SOURCE_WITH_FOO_FN, {
	environment = {
		foo = function()
			return fooValue2
		end,
	},
	enableGlobals = false,
})

local fooFn2Ret = fooFn2()
assert(
	fooFn2Ret == fooValue2,
	"expected `luau.load` with custom environment and no default globals to still return proper values"
)

local CUSTOM_SOURCE_WITH_PRINT_FN = "return print()"

-- NOTE: Testing overriding the print function
local overriddenPrintValue1 = newproxy(false)
local overriddenPrintFn1 = luau.load(CUSTOM_SOURCE_WITH_PRINT_FN, {
	environment = {
		print = function()
			return overriddenPrintValue1
		end,
	},
	enableGlobals = true,
})

local overriddenPrintFnRet1 = overriddenPrintFn1()
assert(
	overriddenPrintFnRet1 == overriddenPrintValue1,
	"expected `luau.load` with overridden environment to return proper values"
)

local overriddenPrintValue2 = newproxy(false)
local overriddenPrintFn2 = luau.load(CUSTOM_SOURCE_WITH_PRINT_FN, {
	environment = {
		print = function()
			return overriddenPrintValue2
		end,
	},
	enableGlobals = false,
})

local overriddenPrintFnRet2 = overriddenPrintFn2()
assert(
	overriddenPrintFnRet2 == overriddenPrintValue2,
	"expected `luau.load` with overridden environment and disabled default globals to return proper values"
)

-- NOTE: Testing whether injectGlobals works
local CUSTOM_SOURCE_WITH_DEFAULT_FN = "return string.lower(...)"

local lowerFn1 = luau.load(CUSTOM_SOURCE_WITH_DEFAULT_FN, {
	environment = {},
	injectGlobals = false,
})

local lowerFn1Success = pcall(lowerFn1, "LOWERCASE")

assert(
	not lowerFn1Success,
	"expected `luau.load` with injectGlobals = false and empty custom environment to not contain default globals"
)

local lowerFn2 = luau.load(CUSTOM_SOURCE_WITH_DEFAULT_FN, {
	environment = { string = string },
	injectGlobals = false,
})

local lowerFn2Success, lowerFn2Result = pcall(lowerFn2, "LOWERCASE")

assert(
	lowerFn2Success and lowerFn2Result == "lowercase",
	"expected `luau.load` with injectGlobals = false and valid custom environment to return proper values"
)

local lowerFn3 = luau.load(CUSTOM_SOURCE_WITH_DEFAULT_FN, {
	environment = {},
	injectGlobals = true,
})

local lowerFn3Success, lowerFn3Result = pcall(lowerFn3, "LOWERCASE")

assert(
	lowerFn3Success and lowerFn3Result == "lowercase",
	"expected `luau.load` with injectGlobals = true and empty custom environment to return proper values"
)
