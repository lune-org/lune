--[=[
	Prints given value(s) to stdout.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
export type print = <T...>(T...) -> ()

--[=[
	Prints given value(s) to stdout with a leading `[INFO]` tag.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
export type printinfo = <T...>(T...) -> ()

--[=[
	Prints given value(s) to stdout with a leading `[WARN]` tag.

	This will format and prettify values such as tables, numbers, booleans, and more.
]=]
export type warn = <T...>(T...) -> ()

--[=[
	Throws an error and prints a formatted version of it with a leading `[ERROR]` tag.

	@param message The error message to throw
	@param level The stack level to throw the error at, defaults to 0
]=]
export type error = <T>(message: T, level: number?) -> ()
