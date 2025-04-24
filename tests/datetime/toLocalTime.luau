local DateTime = require("@lune/datetime")

local values = DateTime.fromRfc3339("2023-08-27T05:54:19Z"):toLocalTime()

local expectedDateTimeValues = os.date("*t", 1693115659)

assert(
	values.year == expectedDateTimeValues.year,
	`expected {values.year} == {expectedDateTimeValues.year}`
)
assert(
	values.month == expectedDateTimeValues.month,
	`expected {values.month} == {expectedDateTimeValues.month}`
)
assert(
	values.day == expectedDateTimeValues.day,
	`expected {values.day} == {expectedDateTimeValues.day}`
)
assert(
	values.hour == expectedDateTimeValues.hour,
	`expected {values.hour} == {expectedDateTimeValues.hour}`
)
assert(
	values.minute == expectedDateTimeValues.min,
	`expected {values.minute} == {expectedDateTimeValues.min}`
)
assert(
	values.second == expectedDateTimeValues.sec,
	`expected {values.second} == {expectedDateTimeValues.sec}`
)
