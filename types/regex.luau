--[=[
	@class RegexMatch

	A match from a regular expression.

	Contains the following values:

	- `start` -- The start index of the match in the original string.
	- `finish` -- The end index of the match in the original string.
	- `text` -- The text that was matched.
	- `len` -- The length of the text that was matched.
]=]
local RegexMatch = {
	start = 0,
	finish = 0,
	text = "",
	len = 0,
}

type RegexMatch = typeof(RegexMatch)

local RegexCaptures = {}

function RegexCaptures.get(self: RegexCaptures, index: number): RegexMatch?
	return nil :: any
end

function RegexCaptures.group(self: RegexCaptures, group: string): RegexMatch?
	return nil :: any
end

function RegexCaptures.format(self: RegexCaptures, format: string): string
	return nil :: any
end

--[=[
	@class RegexCaptures

	Captures from a regular expression.
]=]
export type RegexCaptures = typeof(setmetatable(
	{} :: {
		--[=[
			@within RegexCaptures
			@tag Method
			@method get

			Returns the match at the given index, if one exists.

			@param index -- The index of the match to get
			@return RegexMatch -- The match, if one exists
		]=]

		get: (self: RegexCaptures, index: number) -> RegexMatch?,

		--[=[
			@within RegexCaptures
			@tag Method
			@method group

			Returns the match for the given named match group, if one exists.

			@param group -- The name of the group to get
			@return RegexMatch -- The match, if one exists
		]=]
		group: (self: RegexCaptures, group: string) -> RegexMatch?,

		--[=[
			@within RegexCaptures
			@tag Method
			@method format

			Formats the captures using the given format string.

			### Example usage

			```lua
			local regex = require("@lune/regex")

			local re = regex.new("(?<day>[0-9]{2})-(?<month>[0-9]{2})-(?<year>[0-9]{4})")

			local caps = re:captures("On 14-03-2010, I became a Tenneessee lamb.");
			assert(caps ~= nil, "Example pattern should match example text")

			local formatted = caps:format("year=$year, month=$month, day=$day")
			print(formatted) -- "year=2010, month=03, day=14"
			```

			@param format -- The format string to use
			@return string -- The formatted string
		]=]
		format: (self: RegexCaptures, format: string) -> string,
	},
	{} :: {
		__len: (self: RegexCaptures) -> number,
	}
))

local Regex = {}

--[=[
	@within Regex
	@tag Method

	Check if the given text matches the regular expression.

	This method may be slightly more efficient than calling `find`
	if you only need to know if the text matches the pattern.

	@param text -- The text to search
	@return boolean -- Whether the text matches the pattern
]=]
function Regex.isMatch(self: Regex, text: string): boolean
	return nil :: any
end

--[=[
	@within Regex
	@tag Method

	Finds the first match in the given text.

	Returns `nil` if no match was found.

	@param text -- The text to search
	@return RegexMatch? -- The match object
]=]
function Regex.find(self: Regex, text: string): RegexMatch?
	return nil :: any
end

--[=[
	@within Regex
	@tag Method

	Finds all matches in the given text as a `RegexCaptures` object.

	Returns `nil` if no matches are found.

	@param text -- The text to search
	@return RegexCaptures? -- The captures object
]=]
function Regex.captures(self: Regex, text: string): RegexCaptures?
	return nil :: any
end

--[=[
	@within Regex
	@tag Method

	Splits the given text using the regular expression.

	@param text -- The text to split
	@return { string } -- The split text
]=]
function Regex.split(self: Regex, text: string): { string }
	return nil :: any
end

--[=[
	@within Regex
	@tag Method

	Replaces the first match in the given text with the given replacer string.

	@param haystack -- The text to search
	@param replacer -- The string to replace matches with
	@return string -- The text with the first match replaced
]=]
function Regex.replace(self: Regex, haystack: string, replacer: string): string
	return nil :: any
end

--[=[
	@within Regex
	@tag Method

	Replaces all matches in the given text with the given replacer string.

	@param haystack -- The text to search
	@param replacer -- The string to replace matches with
	@return string -- The text with all matches replaced
]=]
function Regex.replaceAll(self: Regex, haystack: string, replacer: string): string
	return nil :: any
end

export type Regex = typeof(Regex)

--[=[
	@class Regex

	Built-in library for regular expressions

	### Example usage

	```lua
	local Regex = require("@lune/regex")

	local re = Regex.new("hello")

	if re:isMatch("hello, world!") then
		print("Matched!")
	end

	local caps = re:captures("hello, world! hello, again!")

	print(#caps) -- 2
	print(caps:get(1)) -- "hello"
	print(caps:get(2)) -- "hello"
	print(caps:get(3)) -- nil
	```
]=]
local regex = {}

--[=[
	@within Regex
	@tag Constructor

	Creates a new `Regex` from a given string pattern.

	### Errors

	This constructor throws an error if the given pattern is invalid.

	@param pattern -- The string pattern to use
	@return Regex -- The new Regex object
]=]
function regex.new(pattern: string): Regex
	return nil :: any
end

return regex
