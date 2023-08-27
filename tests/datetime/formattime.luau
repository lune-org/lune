local DateTime = require("@lune/datetime")

-- UTC Timezone
assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%Y-%m-%dT%H:%M:%SZ", "en")
		== "2023-08-26T16:56:28Z",
	"invalid ISO 8601 formatting for DateTime.formatTime()"
)

assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%A, %d %B %Y", "fr")
		== "samedi, 26 août 2023",
	"expected format specifier '%A, %d %B %Y' to return 'samedi, 26 août 2023' for locale 'fr'"
)

-- TODO: local timezone tests
