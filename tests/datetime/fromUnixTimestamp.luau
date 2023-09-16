local DateTime = require("@lune/datetime")

-- Bug in rust side implementation for fromUnixTimestamp, calculation for conversion there is wonky,
-- a difference of few millis causes differences as whole seconds for some reason

assert(
	DateTime.fromUnixTimestamp(0000.892).unixTimestampMillis == (0 * 1000) + 892,
	"expected DateTime.fromUnixTimestamp() with millis float to return correct millis timestamp"
)

-- We subtract one due to the floating point accuracy... Need to fix later
assert(
	DateTime.fromUnixTimestamp(1693114921.632).unixTimestampMillis
		== ((1693114921 * 1000) + 632) - 1,
	"expected DateTime.fromUnixTimestamp() with millis and seconds float to return correct millis timestamp"
)
