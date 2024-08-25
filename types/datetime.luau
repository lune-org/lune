--[[
	NOTE: We export a couple different DateTimeValues types below to ensure
	that types are completely accurate, for method args milliseconds will
	always be optional, but for return values millis are always included

	If we figure out some better strategy here where we can
	export just a single type while maintaining accuracy we
	can change to that in a future breaking semver release
]]

type OptionalMillisecond = {
	millisecond: number?,
}

type Millisecond = {
	millisecond: number,
}

--[=[
	@interface Locale
	@within DateTime

	Enum type representing supported DateTime locales.

	Currently supported locales are:

	- `en` - English
	- `de` - German
	- `es` - Spanish
	- `fr` - French
	- `it` - Italian
	- `ja` - Japanese
	- `pl` - Polish
	- `pt-br` - Brazilian Portuguese
	- `pt` - Portuguese
	- `tr` - Turkish
]=]
export type Locale = "en" | "de" | "es" | "fr" | "it" | "ja" | "pl" | "pt-br" | "pt" | "tr"

--[=[
	@interface DateTimeValues
	@within DateTime

	Individual date & time values, representing the primitives that make up a `DateTime`.

	This is a dictionary that will contain the following values:

	- `year` - Year(s), in the range 1400 -> 9999
	- `month` - Month(s), in the range 1 -> 12
	- `day` - Day(s), in the range 1 -> 31
	- `hour` - Hour(s), in the range 0 -> 23
	- `minute` - Minute(s), in the range 0 -> 59
	- `second` - Second(s), in the range 0 -> 60, where 60 is a leap second

	An additional `millisecond` value may also be included,
	and should be within the range `0 -> 999`, but is optional.

	However, any method returning this type should be guaranteed
	to include milliseconds - see individual methods to verify.
]=]
export type DateTimeValues = {
	year: number,
	month: number,
	day: number,
	hour: number,
	minute: number,
	second: number,
}

--[=[
	@interface DateTimeValueArguments
	@within DateTime

	Alias for `DateTimeValues` with an optional `millisecond` value.

	Refer to the `DateTimeValues` documentation for additional information.
]=]
export type DateTimeValueArguments = DateTimeValues & OptionalMillisecond

--[=[
	@interface DateTimeValueReturns
	@within DateTime

	Alias for `DateTimeValues` with a mandatory `millisecond` value.

	Refer to the `DateTimeValues` documentation for additional information.
]=]
export type DateTimeValueReturns = DateTimeValues & Millisecond

--[=[
	@prop unixTimestamp number
	@within DateTime
	Number of seconds passed since the UNIX epoch.
]=]

--[=[
	@prop unixTimestampMillis number
	@within DateTime
	Number of milliseconds passed since the UNIX epoch.
]=]
local DateTime = {
	unixTimestamp = (nil :: any) :: number,
	unixTimestampMillis = (nil :: any) :: number,
}

--[=[
	@within DateTime
	@tag Method

	Formats this `DateTime` using the given `formatString` and `locale`, as local time.

	The given `formatString` is parsed using a `strftime`/`strptime`-inspired
	date and time formatting syntax, allowing tokens such as the following:

	| Token | Example  | Description   |
	|-------|----------|---------------|
	| `%Y`  | `1998`   | Year number   |
	| `%m`  | `04`     | Month number  |
	| `%d`  | `29`     | Day number    |
	| `%A`  | `Monday` | Weekday name  |
	| `%M`  | `59`     | Minute number |
	| `%S`  | `10`     | Second number |

	For a full reference of all available tokens, see the
	[chrono documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

	If not provided, `formatString` and `locale` will default
	to `"%Y-%m-%d %H:%M:%S"` and `"en"` (english) respectively.

	@param formatString -- A string containing formatting tokens
	@param locale -- The locale the time should be formatted in
	@return string -- The formatting string
]=]
function DateTime.formatLocalTime(self: DateTime, formatString: string?, locale: Locale?): string
	return nil :: any
end

--[=[
	@within DateTime
	@tag Method

	Formats this `DateTime` using the given `formatString` and `locale`, as UTC (universal) time.

	The given `formatString` is parsed using a `strftime`/`strptime`-inspired
	date and time formatting syntax, allowing tokens such as the following:

	| Token | Example  | Description   |
	|-------|----------|---------------|
	| `%Y`  | `1998`   | Year number   |
	| `%m`  | `04`     | Month number  |
	| `%d`  | `29`     | Day number    |
	| `%A`  | `Monday` | Weekday name  |
	| `%M`  | `59`     | Minute number |
	| `%S`  | `10`     | Second number |

	For a full reference of all available tokens, see the
	[chrono documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

	If not provided, `formatString` and `locale` will default
	to `"%Y-%m-%d %H:%M:%S"` and `"en"` (english) respectively.

	@param formatString -- A string containing formatting tokens
	@param locale -- The locale the time should be formatted in
	@return string -- The formatting string
]=]
function DateTime.formatUniversalTime(
	self: DateTime,
	formatString: string?,
	locale: Locale?
): string
	return nil :: any
end

--[=[
	@within DateTime
	@tag Method

	Formats this `DateTime` as an ISO 8601 date-time string.

	Some examples of ISO 8601 date-time strings are:

	- `2020-02-22T18:12:08Z`
	- `2000-01-31T12:34:56+05:00`
	- `1970-01-01T00:00:00.055Z`

	@return string -- The ISO 8601 formatted string
]=]
function DateTime.toIsoDate(self: DateTime): string
	return nil :: any
end

--[=[
	@within DateTime
	@tag Method

	Extracts separated local date & time values from this `DateTime`.

	The returned table contains the following values:

	| Key           | Type     | Range          |
	|---------------|----------|----------------|
	| `year`        | `number` | `1400 -> 9999` |
	| `month`       | `number` | `1 -> 12`      |
	| `day`         | `number` | `1 -> 31`      |
	| `hour`        | `number` | `0 -> 23`      |
	| `minute`      | `number` | `0 -> 59`      |
	| `second`      | `number` | `0 -> 60`      |
	| `millisecond` | `number` | `0 -> 999`     |

	@return DateTimeValueReturns -- A table of DateTime values
]=]
function DateTime.toLocalTime(self: DateTime): DateTimeValueReturns
	return nil :: any
end

--[=[
	@within DateTime
	@tag Method

	Extracts separated UTC (universal) date & time values from this `DateTime`.

	The returned table contains the following values:

	| Key           | Type     | Range          |
	|---------------|----------|----------------|
	| `year`        | `number` | `1400 -> 9999` |
	| `month`       | `number` | `1 -> 12`      |
	| `day`         | `number` | `1 -> 31`      |
	| `hour`        | `number` | `0 -> 23`      |
	| `minute`      | `number` | `0 -> 59`      |
	| `second`      | `number` | `0 -> 60`      |
	| `millisecond` | `number` | `0 -> 999`     |

	@return DateTimeValueReturns -- A table of DateTime values
]=]
function DateTime.toUniversalTime(self: DateTime): DateTimeValueReturns
	return nil :: any
end

export type DateTime = typeof(DateTime)

--[=[
	@class DateTime

	Built-in library for date & time

	### Example usage

	```lua
	local DateTime = require("@lune/datetime")

	-- Creates a DateTime for the current exact moment in time
	local now = DateTime.now()

	-- Formats the current moment in time as an ISO 8601 string
	print(now:toIsoDate())

	-- Formats the current moment in time, using the local
	-- time, the French locale, and the specified time string
	print(now:formatLocalTime("%A, %d %B %Y", "fr"))

	-- Returns a specific moment in time as a DateTime instance
	local someDayInTheFuture = DateTime.fromLocalTime({
		year = 3033,
		month = 8,
		day = 26,
		hour = 16,
		minute = 56,
		second = 28,
		millisecond = 892,
	})

	-- Extracts the current local date & time as separate values (same values as above table)
	print(now:toLocalTime())

	-- Returns a DateTime instance from a given float, where the whole
	-- denotes the seconds and the fraction denotes the milliseconds
	-- Note that the fraction for millis here is completely optional
	DateTime.fromUnixTimestamp(871978212313.321)

	-- Extracts the current universal (UTC) date & time as separate values
	print(now:toUniversalTime())
	```
]=]
local dateTime = {}

--[=[
	@within DateTime
	@tag Constructor

	Returns a `DateTime` representing the current moment in time.

	@return DateTime -- The new DateTime object
]=]
function dateTime.now(): DateTime
	return nil :: any
end

--[=[
	@within DateTime
	@tag Constructor

	Creates a new `DateTime` from the given UNIX timestamp.

	This timestamp may contain both a whole and fractional part -
	where the fractional part denotes milliseconds / nanoseconds.

	Example usage of fractions:

	- `DateTime.fromUnixTimestamp(123456789.001)` - one millisecond
	- `DateTime.fromUnixTimestamp(123456789.000000001)` - one nanosecond

	Note that the fractional part has limited precision down to exactly
	one nanosecond, any fraction that is more precise will get truncated.

	@param unixTimestamp -- Seconds passed since the UNIX epoch
	@return DateTime -- The new DateTime object
]=]
function dateTime.fromUnixTimestamp(unixTimestamp: number): DateTime
	return nil :: any
end

--[=[
	@within DateTime
	@tag Constructor

	Creates a new `DateTime` from the given date & time values table, in universal (UTC) time.

	The given table must contain the following values:

	| Key      | Type     | Range          |
	|----------|----------|----------------|
	| `year`   | `number` | `1400 -> 9999` |
	| `month`  | `number` | `1 -> 12`      |
	| `day`    | `number` | `1 -> 31`      |
	| `hour`   | `number` | `0 -> 23`      |
	| `minute` | `number` | `0 -> 59`      |
	| `second` | `number` | `0 -> 60`      |

	An additional `millisecond` value may also be included,
	and should be within the range `0 -> 999`, but is optional.

	Any non-integer values in the given table will be rounded down.

	### Errors

	This constructor is fallible and may throw an error in the following situations:

	- Date units (year, month, day) were given that produce an invalid date. For example, January 32nd or February 29th on a non-leap year.

	@param values -- Table containing date & time values
	@return DateTime -- The new DateTime object
]=]
function dateTime.fromUniversalTime(values: DateTimeValueArguments): DateTime
	return nil :: any
end

--[=[
	@within DateTime
	@tag Constructor

	Creates a new `DateTime` from the given date & time values table, in local time.

	The given table must contain the following values:

	| Key      | Type     | Range          |
	|----------|----------|----------------|
	| `year`   | `number` | `1400 -> 9999` |
	| `month`  | `number` | `1 -> 12`      |
	| `day`    | `number` | `1 -> 31`      |
	| `hour`   | `number` | `0 -> 23`      |
	| `minute` | `number` | `0 -> 59`      |
	| `second` | `number` | `0 -> 60`      |

	An additional `millisecond` value may also be included,
	and should be within the range `0 -> 999`, but is optional.

	Any non-integer values in the given table will be rounded down.

	### Errors

	This constructor is fallible and may throw an error in the following situations:

	- Date units (year, month, day) were given that produce an invalid date. For example, January 32nd or February 29th on a non-leap year.

	@param values -- Table containing date & time values
	@return DateTime -- The new DateTime object
]=]
function dateTime.fromLocalTime(values: DateTimeValueArguments): DateTime
	return nil :: any
end

--[=[
	@within DateTime
	@tag Constructor

	Creates a new `DateTime` from an ISO 8601 date-time string.

	### Errors

	This constructor is fallible and may throw an error if the given
	string does not strictly follow the ISO 8601 date-time string format.

	Some examples of valid ISO 8601 date-time strings are:

	- `2020-02-22T18:12:08Z`
	- `2000-01-31T12:34:56+05:00`
	- `1970-01-01T00:00:00.055Z`

	@param isoDate -- An ISO 8601 formatted string
	@return DateTime -- The new DateTime object
]=]
function dateTime.fromIsoDate(isoDate: string): DateTime
	return nil :: any
end

return dateTime
