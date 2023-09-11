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

--[=[
	@class DateTime

	Built-in library for date & time manipulation

	### Example usage

	```lua
	local DateTime = require("@lune/DateTime")

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
local dateTime = {
	unixTimestamp = 0,
	unixTimestampMillis = 0,
}

--[=[
	@within DateTime

    Returns a `DateTime` representing the current moment in time.

    @return A DateTime instance
]=]
function dateTime.now(): typeof(dateTime)
	return nil :: any
end

--[=[
	@within DateTime

	Returns a new `DateTime` object from the given Unix timestamp, or
	the number of **seconds** since January 1st, 1970 at 00:00 (UTC).

	@param unixTimestamp The number of seconds or milliseconds (or both) since the Unix epoch. The fraction part of a float denotes the milliseconds.
	@return A DateTime instance
]=]
function dateTime.fromUnixTimestamp(unixTimestamp: number?): typeof(dateTime)
	return nil :: any
end

--[=[
	@within DateTime

	Returns a new `DateTime` using the given units from a UTC time. The
    values accepted are similar to those found in the time value table
    returned by `toUniversalTime`.
    
    - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.

	@param dateTime Optional values for the dateTime instance, defaults to the current time
	@return A DateTime instance
]=]
function dateTime.fromUniversalTime(dateTime: DateTimeValues?): typeof(dateTime)
	return nil :: any
end

--[=[
	@within DateTime

	Returns a new `DateTime` using the given units from a local time. The
    values accepted are similar to those found in the time value table
    returned by `toLocalTime`.
    
    - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.

	@param dateTime Optional values for the dateTime instance, defaults to the current time
	@return A DateTime instance
]=]
function dateTime.fromLocalTime(dateTime: DateTimeValues?): typeof(dateTime)
	return nil :: any
end

--[=[
	@within DateTime

	Returns a `DateTime` from an ISO 8601 date-time string in UTC
    time, such as those returned by `toIsoDate`. If the
    string parsing fails, the function returns `nil`.
    
    An example ISO 8601 date-time string would be `2020-01-02T10:30:45Z`,
    which represents January 2nd 2020 at 10:30 AM, 45 seconds.

	@param isoDate An ISO 8601 formatted string
	@return A DateTime instance
]=]
function dateTime.fromIsoDate(iso_date: string): typeof(dateTime)?
	return nil :: any
end

--[=[
	@within DateTime

	Formats a date as a ISO 8601 date-time string, returns None if the DateTime 
	object is invalid. The value returned by this function could be passed to 
	`fromLocalTime` to produce the original `DateTime` object.

	@return ISO 8601 formatted string
]=]
function dateTime:toIsoDate(): string
	return nil :: any
end

--[=[
	@within DateTime

	Converts the value of this `DateTime` object to local time. The returned table
    contains the following keys: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`,
    `Millisecond`. For more details, see the time value table in this data type's
    description. The values within this table could be passed to `fromLocalTime`
    to produce the original `DateTime` object.

	@return A table of DateTime values
]=]
function dateTime:toLocalTime(): DateTimeValues & { month: number }
	return nil :: any
end

--[=[
	@within DateTime

	Converts the value of this `DateTime` object to universal time. The returned table
    contains the following keys: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`,
    `Millisecond`. For more details, see the time value table in this data type's
    description. The values within this table could be passed to `fromUniversalTime`
    to produce the original `DateTime` object.

	@return A table of DateTime values
]=]
function dateTime:toUniversalTime(): DateTimeValues & { month: number }
	return nil :: any
end

--[=[
	@within DateTime

	Generates a string from the `DateTime` value interpreted as the specified timezone
    and a format string. The format string should contain tokens, which will
    replace to certain date/time values described by the `DateTime` object.

	@param timezone The timezone the formatted time string should follow
	@param formatString A format string of strfttime items. See [chrono docs](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).
	@param locale The locale the time should be formatted in
	@return A table of DateTime values
]=]
function dateTime:formatTime(timezone: Timezone, formatString: string, locale: Locale): string
	return nil :: any
end

export type DateTime = typeof(dateTime)

return dateTime
