--[=[
	@interface CompileOptions
	@within Luau

	The Luau compiler options used in generating luau bytecode

	This is a dictionary that may contain one or more of the following values:

	* `optimizationLevel` - Sets the compiler option "optimizationLevel". Defaults to `1`
	* `coverageLevel` - Sets the compiler option "coverageLevel". Defaults to `0`
	* `debugLevel` - Sets the compiler option "debugLevel". Defaults to `1`

	Documentation regarding what these values represent can be found here;
	* https://github.com/Roblox/luau/blob/bd229816c0a82a8590395416c81c333087f541fd/Compiler/include/luacode.h#L13
]=]
export type CompileOptions = {
	optimizationLevel: number,
	coverageLevel: number,
	debugLevel: number,
}

--[=[
	@interface LoadOptions
	@within Luau

	The Luau load options are used for generating a lua function from either bytecode or sourcecode

	This is a dictionary that may contain one or more of the following values:

	* `debugName` - The debug name of the closure. Defaults to `string ["..."]`
]=]
export type LoadOptions = {
	debugName: string,
}

--[=[
	@class Luau

	Built-in library for generating luau bytecode & functions.

	### Example usage

	```lua
	local luau = require("@lune/luau")

	local bytecode = luau.compile("print('Hello, World!')")
	local callableFn = luau.load(bytecode)

	-- Additionally, we can skip the bytecode generation and load a callable function directly from the code itself.
	-- local callableFn = luau.load("print('Hello, World!')")

	callableFn()
	```
]=]
local luau = {}

--[=[
	@within Luau

	Compiles sourcecode into Luau bytecode

	An error will be thrown if the sourcecode given isn't valid Luau code.

	### Example usage

	```lua
	local luau = require("@lune/luau")

	local bytecode = luau.compile("print('Hello, World!')", {
		optimizationLevel: 1,
		coverageLevel: 0,
		debugLevel: 1,
	})
	```

	@param source The string that'll be compiled into bytecode
	@param CompileOptions The luau compiler options used when compiling the source string

	@return luau bytecode
]=]
function luau.compile(source: string, CompileOptions: CompileOptions?): string
	return nil :: any
end

--[=[
	@within Luau

	Generates a function from either bytecode or sourcecode

	An error will be thrown if the sourcecode given isn't valid luau code.

	### Example usage

	```lua
	local luau = require("@lune/luau")

	local bytecode = luau.compile("print('Hello, World!')")
	local callableFn = luau.load(bytecode, {
		debugName = "'Hello, World'"
	})

	callableFn()
	```

	@param source Either bytecode or sourcecode
	@param loadOptions The load options used when creating a callbable function

	@return luau function
]=]
function luau.load(source: string, loadOptions: LoadOptions?): (...any) -> ...any
	return nil :: any
end

return luau
