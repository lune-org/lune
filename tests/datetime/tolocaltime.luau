local DateTime = require("@lune/datetime")

local dateTime = (DateTime.fromIsoDate("2023-08-27T05:54:19Z") :: DateTime.DateTime):toLocalTime()

local expectedDateTimeValues = table.pack(
	string.match(
		"2023-08-27T11:24:19Z",
		"(%d%d%d%d)-?(%d?%d?)-?(%d?%d?)T(%d?%d?):(%d?%d?):(%d?%d?)Z$"
	)
)

local expectedYear = tonumber(expectedDateTimeValues[1])
local expectedMonth = tonumber(expectedDateTimeValues[2])
local expectedDay = tonumber(expectedDateTimeValues[3])
local expectedHour = tonumber(expectedDateTimeValues[4])
local expectedMinute = tonumber(expectedDateTimeValues[5])
local expectedSecond = tonumber(expectedDateTimeValues[6])

assert(dateTime.year == expectedYear, `expected {dateTime.year} == {expectedYear}`)
assert(dateTime.month == expectedMonth, `expected {dateTime.month} == {expectedMonth}`)
assert(dateTime.day == expectedDay, `expected {dateTime.day} == {expectedDay}`)
assert(dateTime.hour == expectedHour, `expected {dateTime.hour} == {expectedHour}`)
assert(dateTime.minute == expectedMinute, `expected {dateTime.minute} == {expectedMinute}`)
assert(dateTime.second == expectedSecond, `expected {dateTime.second} == {expectedSecond}`)
