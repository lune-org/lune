local DateTime = require("@lune/datetime")

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

-- FIXME: Local timezone formatTime fails due to a rust side bug

local expectedTimeString = os.date("%Y-%m-%dT%H:%M:%SZ%z", 1693932753)

-- NOTE: UTC Timezone conversions work perfectly fine, issue only for local.
-- So far I've made it so that format_time always converts to_universal_time
-- Builder object. Then in to_string, we construct a ChronoDateTime from the values
-- and convert it to the requested timezone. We then format this with our formatter
-- string.

-- For debugging, try checking what to_universal_time returns, maybe that's where
-- an incorrect DateTimeBuilder is getting returned... If that's the case, that means
-- there probably isn't any issue with to_string at all.

-- Yes, an invalid DateTimeBuilder is being returned turns out. Still don't know why actually.

print(
	expectedTimeString,
	DateTime.fromUnixTimestamp(1693068988):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
)

assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatTime() (local)"
)
