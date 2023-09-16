local DateTime = require("@lune/datetime")

assert(
	DateTime.fromUnixTimestamp(1693068988):formatUniversalTime("%Y-%m-%dT%H:%M:%SZ", "en")
		== "2023-08-26T16:56:28Z",
	"invalid ISO 8601 formatting for DateTime.formatTime() (UTC)"
)

local expectedTimeString = os.date("%Y-%m-%dT%H:%M:%SZ", 1694078954)

assert(
	DateTime.fromUnixTimestamp(1694078954):formatLocalTime("%Y-%m-%dT%H:%M:%SZ", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatTime()"
)
