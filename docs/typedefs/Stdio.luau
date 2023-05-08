export type Color =
	"reset"
	| "black"
	| "red"
	| "green"
	| "yellow"
	| "blue"
	| "purple"
	| "cyan"
	| "white"
export type Style = "reset" | "bold" | "dim"

--[=[
	@class Stdio

	Built-in standard input / output & utility functions

	### Example usage

	```lua
	local stdio = require("@lune/stdio")

	-- Prompting the user for basic input
	local text: string = stdio.prompt("text", "Please write some text")
	local confirmed: boolean = stdio.prompt("confirm", "Please confirm this action")

	-- Writing directly to stdout or stderr, without the auto-formatting of print/warn/error
	stdio.write("Hello, ")
	stdio.write("World! ")
	stdio.write("All on the same line")
	stdio.ewrite("\nAnd some error text, too")
	```
]=]
export type Stdio = {
	--[=[
		@within Stdio
		@must_use

		Return an ANSI string that can be used to modify the persistent output color.

		Pass `"reset"` to get a string that can reset the persistent output color.

		### Example usage

		```lua
		stdio.write(stdio.color("red"))
		print("This text will be red")
		stdio.write(stdio.color("reset"))
		print("This text will be normal")
		```

		@param color The color to use
		@return A printable ANSI string
	]=]
	color: (color: Color) -> string,
	--[=[
		@within Stdio
		@must_use

		Return an ANSI string that can be used to modify the persistent output style.

		Pass `"reset"` to get a string that can reset the persistent output style.

		### Example usage

		```lua
		stdio.write(stdio.style("bold"))
		print("This text will be bold")
		stdio.write(stdio.style("reset"))
		print("This text will be normal")
		```

		@param style The style to use
		@return A printable ANSI string
	]=]
	style: (style: Style) -> string,
	--[=[
		@within Stdio
		@must_use

		Formats arguments into a human-readable string with syntax highlighting for tables.

		@param ... The values to format
		@return The formatted string
	]=]
	format: (...any) -> string,
	--[=[
		@within Stdio

		Writes a string directly to stdout, without any newline.

		@param s The string to write to stdout
	]=]
	write: (s: string) -> (),
	--[=[
		@within Stdio

		Writes a string directly to stderr, without any newline.

		@param s The string to write to stderr
	]=]
	ewrite: (s: string) -> (),
	--[=[
		@within Stdio
		@must_use

		Prompts for user input using the wanted kind of prompt:

		* `"text"` - Prompts for a plain text string from the user
		* `"confirm"` - Prompts the user to confirm with y / n (yes / no)
		* `"select"` - Prompts the user to select *one* value from a list
		* `"multiselect"` - Prompts the user to select *one or more* values from a list
		* `nil` - Equivalent to `"text"` with no extra arguments

		@param kind The kind of prompt to use
		@param message The message to show the user
		@param defaultOrOptions The default value for the prompt, or options to choose from for selection prompts
	]=]
	prompt: (
		(() -> string)
		& ((kind: "text", message: string?, defaultOrOptions: string?) -> string)
		& ((kind: "confirm", message: string, defaultOrOptions: boolean?) -> boolean)
		& ((kind: "select", message: string?, defaultOrOptions: { string }) -> number?)
		& ((kind: "multiselect", message: string?, defaultOrOptions: { string }) -> { number }?)
	),
}
