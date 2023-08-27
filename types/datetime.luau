-- TODO: Add docs

export type Locale = "en" | "de" | "es" | "fr" | "it" | "ja" | "pl" | "pt-br" | "pt" | "tr"

export type Timezone = "utc" | "local"

export type ShortMonth =
	"jan"
	| "feb"
	| "mar"
	| "apr"
	| "may"
	| "jun"
	| "jul"
	| "aug"
	| "sep"
	| "oct"
	| "nov"
	| "dec"

export type Month =
	"january"
	| "february"
	| "march"
	| "april"
	| "may"
	| "june"
	| "july"
	| "august"
	| "september"
	| "october"
	| "november"
	| "december"

export type DateTimeValues = {
	year: number,
	month: number | ShortMonth | Month,
	day: number,
	hour: number,
	minute: number,
	second: number,
	millisecond: number,
}

export type DateTime = {
	unixTimestamp: number,
	unixTimestampMillis: number,

	toIsoDate: (self: DateTime) -> string,
	toLocalTime: (self: DateTime) -> DateTimeValues,
	toUniversalTime: (self: DateTime) -> DateTimeValues,
	formatTime: (
		self: DateTime,
		timezone: Timezone,
		formatString: string,
		locale: Locale
	) -> string,
}

--[=[
	@class DateTime

	Built-in library for date & time manipulation

	### Example usage

	```lua
	local DateTime = require("@lune/datetime")
	```
]=]
local dateTime = {}

--[=[
	@within DateTime

    Returns a `DateTime` representing the current moment in time.

    @return A DateTime instance
]=]
function dateTime.now(): DateTime
	return nil :: any
end

--[=[
	@within DateTime

	Returns a new `DateTime` object from the given Unix timestamp, or
	the number of **seconds** since January 1st, 1970 at 00:00 (UTC).

	@param unixTimestamp The number of seconds or milliseconds (or both) since the Unix epoch. The fraction part of a float denotes the milliseconds.
	@return A DateTime instance
]=]
function dateTime.fromUnixTimestamp(unixTimestamp: number?): DateTime
	return nil :: any
end

function dateTime.fromUniversalTime(dateTime: DateTimeValues?): DateTime
	return nil :: any
end

function dateTime.fromLocalTime(dateTime: DateTimeValues?): DateTime
	return nil :: any
end

function dateTime.fromIsoDate(iso_date: string): DateTime?
	return nil :: any
end

return dateTime
