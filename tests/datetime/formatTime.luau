local DateTime = require("@lune/DateTime")

-- UTC Timezone
assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%Y-%m-%dT%H:%M:%SZ", "en")
		== "2023-08-26T16:56:28Z",
	"invalid ISO 8601 formatting for DateTime.formatTime() (UTC)"
)

assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%A, %d %B %Y", "fr")
		== "samedi, 26 août 2023",
	"expected format specifier '%A, %d %B %Y' to return 'samedi, 26 août 2023' for locale 'fr'"
)

local expectedTimeString = os.date("%Y-%m-%dT%H:%M:%SZ", 1694078954)

print(
	expectedTimeString,
	DateTime.fromUnixTimestamp(1694078954):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
)

assert(
	DateTime.fromUnixTimestamp(1694078954):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatTime() (local)"
)

-- TODO: Locale tests, by using os.date()
