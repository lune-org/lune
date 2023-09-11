local DateTime = require("@lune/DateTime")

assert(
	math.abs(DateTime.fromUniversalTime()["unixTimestamp"] - os.time()) <= 3,
	"expected DateTime.fromLocalTime() with no args to return DateTime at the current moment"
)

assert(
	DateTime.fromUniversalTime({
		["year"] = 2023,
		["month"] = "aug",
		["day"] = 26,
		["hour"] = 16,
		["minute"] = 56,
		["second"] = 28,
		["millisecond"] = 0,
	})["unixTimestamp"] == 1693068988,
	"expected DateTime.fromUniversalTime() with DateTimeValues arg to return 1693068988s"
)

assert(
	DateTime.fromUniversalTime({
		["year"] = 2023,
		["month"] = "aug",
		["day"] = 26,
		["hour"] = 16,
		["minute"] = 56,
		["second"] = 28,
		["millisecond"] = 892,
	})["unixTimestampMillis"] == 1693068988892,
	"expected DateTime.fromUniversalTime() with DateTimeValues arg with millis to return 1693068988892ms"
)
