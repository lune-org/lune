-- TODO: Add more docs

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

	-- Returns the current moment in time as a ISO 8601 string
	DateTime.now():toIsoDate()

	-- Returns the current moment in time as per the format template in French
	DateTime.now():formatTime("utc", "%A, %d %B %Y", "fr")

	-- Returns a specific moment in time as a DateTime instance
	DateTime.fromLocalTime({
		year = 2023,
		month = "aug",
		day = 26,
		hour = 16,
		minute = 56,
		second = 28,
		millisecond = 892,
	})

	-- Returns the current local time as a DateTime instance
	DateTime.fromLocalTime()

	-- Returns a DateTime instance from a given float, where the whole denotes the 
	-- seconds and the fraction denotes the milliseconds
	DateTime.fromUnixTimestamp(871978212313.321)

	-- Returns the current time in terms of UTC
	DateTime.now():toUniversalTime() 
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

--[=[
	@within DateTime

	Returns a new `DateTime` using the given units from a UTC time. The
    values accepted are similar to those found in the time value table
    returned by `to_universal_time`.
    
    - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.

	@param dateTime Optional values for the dateTime instance, defaults to the current time
	@return A DateTime instance
]=]
function dateTime.fromUniversalTime(dateTime: DateTimeValues?): DateTime
	return nil :: any
end

--[=[
	@within DateTime

	Returns a new `DateTime` using the given units from a local time. The
    values accepted are similar to those found in the time value table
    returned by `to_local_time`.
    
    - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.

	@param dateTime Optional values for the dateTime instance, defaults to the current time
	@return A DateTime instance
]=]
function dateTime.fromLocalTime(dateTime: DateTimeValues?): DateTime
	return nil :: any
end

--[=[
	@within DateTime

	Returns a `DateTime` from an ISO 8601 date-time string in UTC
    time, such as those returned by `to_iso_date`. If the
    string parsing fails, the function returns `nil`.
    
    An example ISO 8601 date-time string would be `2020-01-02T10:30:45Z`,
    which represents January 2nd 2020 at 10:30 AM, 45 seconds.

	@param isoDate An ISO 8601 formatted string
	@return A DateTime instance
]=]
function dateTime.fromIsoDate(iso_date: string): DateTime?
	return nil :: any
end

return dateTime
