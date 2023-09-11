local DateTime = require("@lune/DateTime")

local dateTime = (DateTime.fromIsoDate("2023-08-27T05:54:19Z") :: DateTime.DateTime):toLocalTime()

local expectedDateTimeValues = os.date("*t", 1693115659)

assert(
	dateTime.year == expectedDateTimeValues.year,
	`expected {dateTime.year} == {expectedDateTimeValues.year}`
)
assert(
	dateTime.month == expectedDateTimeValues.month,
	`expected {dateTime.month} == {expectedDateTimeValues.month}`
)
assert(
	dateTime.day == expectedDateTimeValues.day,
	`expected {dateTime.day} == {expectedDateTimeValues.day}`
)
assert(
	dateTime.hour == expectedDateTimeValues.hour,
	`expected {dateTime.hour} == {expectedDateTimeValues.hour}`
)
assert(
	dateTime.minute == expectedDateTimeValues.min,
	`expected {dateTime.minute} == {expectedDateTimeValues.min}`
)
assert(
	dateTime.second == expectedDateTimeValues.sec,
	`expected {dateTime.second} == {expectedDateTimeValues.sec}`
)
