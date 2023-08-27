local DateTime = require("@lune/datetime")

assert(
	DateTime.fromLocalTime()["unixTimestamp"] == os.time(),
	"expected DateTime.fromLocalTime() with no args to return DateTime at current moment"
)

assert(
	DateTime.fromLocalTime({
		["year"] = 2023,
		["month"] = "aug",
		["day"] = 26,
		["hour"] = 16,
		["minute"] = 56,
		["second"] = 28,
		["millisecond"] = 0,
	})["unixTimestamp"] == 1693049188,
	"expected DateTime.fromLocalTime() to return DateTime with timestamp to return 1693049188s"
)

assert(
	DateTime.fromLocalTime({
		["year"] = 2023,
		["month"] = "aug",
		["day"] = 26,
		["hour"] = 16,
		["minute"] = 56,
		["second"] = 28,
		["millisecond"] = 892,
	})["unixTimestampMillis"] == 1693049188000,
	"expected DateTime.fromLocalTime() with float millis arg timestamp to return 1693049188000ms"
)
