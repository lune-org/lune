--[=[
	@interface CompileOptions
	@within Luau

	The options passed to the luau compiler while compiling bytecode.

	This is a dictionary that may contain one or more of the following values:

	* `optimizationLevel` - Sets the compiler option "optimizationLevel". Defaults to `1`.
	* `coverageLevel` - Sets the compiler option "coverageLevel". Defaults to `0`.
	* `debugLevel` - Sets the compiler option "debugLevel". Defaults to `1`.

	Documentation regarding what these values represent can be found [here](https://github.com/Roblox/luau/blob/bd229816c0a82a8590395416c81c333087f541fd/Compiler/include/luacode.h#L13-L39).
]=]
export type CompileOptions = {
	optimizationLevel: number?,
	coverageLevel: number?,
	debugLevel: number?,
}

--[=[
	@interface LoadOptions
	@within Luau

	The options passed while loading a luau chunk from an arbitrary string, or bytecode.

	This is a dictionary that may contain one or more of the following values:

	* `debugName` - The debug name of the closure. Defaults to `luau.load(...)`.
	* `environment` - A custom environment to load the chunk in. Setting a custom environment will deoptimize the chunk and forcefully disable codegen. Defaults to the global environment.
	* `injectGlobals` - Whether or not to inject globals in the custom environment. Has no effect if no custom environment is provided. Defaults to `true`.
	* `codegenEnabled` - Whether or not to enable codegen. Defaults to `false`.
]=]
export type LoadOptions = {
	debugName: string?,
	environment: { [string]: any }?,
	injectGlobals: boolean?,
	codegenEnabled: boolean?,
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

	Since luau bytecode is highly compressible, it may also make sense to compress it using the `serde` library
	while transmitting large amounts of it.
]=]
local luau = {}

--[=[
	@within Luau

	Compiles sourcecode into Luau bytecode

	An error will be thrown if the sourcecode given isn't valid Luau code.

	### Example usage

	```lua
	local luau = require("@lune/luau")

	-- Compile the source to some highly optimized bytecode
	local bytecode = luau.compile("print('Hello, World!')", {
		optimizationLevel = 2,
		coverageLevel = 0,
		debugLevel = 1,
	})
	```

	@param source The string that will be compiled into bytecode
	@param compileOptions The options passed to the luau compiler that will output the bytecode

	@return luau bytecode
]=]
function luau.compile(source: string, compileOptions: CompileOptions?): string
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

	@param source Either luau bytecode or string source code
	@param loadOptions The options passed to luau for loading the chunk

	@return luau chunk
]=]
function luau.load(source: string, loadOptions: LoadOptions?): (...any) -> ...any
	return nil :: any
end

return luau
