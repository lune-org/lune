local DateTime = require("@lune/datetime")
assert(
	DateTime.fromLocalTime()["unixTimestamp"] == os.time(),
	"expected DateTime.fromLocalTime() with no args to return DateTime at current moment"
)

local timeValues1 = os.date("*t", 1693049188)

assert(
	DateTime.fromLocalTime({
		["year"] = timeValues1.year,
		["month"] = timeValues1.month,
		["day"] = timeValues1.day,
		["hour"] = timeValues1.hour,
		["minute"] = timeValues1.min,
		["second"] = timeValues1.sec,
		["millisecond"] = 0,
	})["unixTimestamp"] == 1693049188,
	"expected DateTime.fromLocalTime() with DateTimeValues arg to return 1693049188s"
)

print(DateTime.fromLocalTime({
	["year"] = 2023,
	["month"] = "aug",
	["day"] = 26,
	["hour"] = 16,
	["minute"] = 56,
	["second"] = 28,
	["millisecond"] = 892,
})["unixTimestamp"])

local timeValues2 = os.date("*t", 1693049188.892)

assert(
	DateTime.fromLocalTime({
		["year"] = timeValues2.year,
		["month"] = timeValues2.month,
		["day"] = timeValues2.day,
		["hour"] = timeValues2.hour,
		["minute"] = timeValues2.min,
		["second"] = timeValues2.sec,
		["millisecond"] = 892,
	})["unixTimestampMillis"] == 1693049188892,
	"expected DateTime.fromLocalTime() with DateTimeValues arg with millis to return 1693049188892ms"
)
