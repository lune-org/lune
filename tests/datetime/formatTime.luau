local DateTime = require("@lune/DateTime")
local process = require("@lune/Process")

-- UTC Timezone
assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%Y-%m-%dT%H:%M:%SZ", "en")
		== "2023-08-26T16:56:28Z",
	"invalid ISO 8601 formatting for DateTime.formatTime() (UTC)"
)

assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("utc", "%A, %d %B %Y", "fr")
		== "samedi, 26 août 2023",
	"expected format specifier '%A, %d %B %Y' to return 'samedi, 26 août 2023' for locale 'fr' (UTC)"
)

local expectedTimeString = os.date("%Y-%m-%dT%H:%M:%SZ", 1694078954)

assert(
	DateTime.fromUnixTimestamp(1694078954):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatTime() (local)"
)

-- This test requires 'fr_FR.UTF-8 UTF-8' to be in /etc/locale.gen to pass
-- Locale should be set up by a script, or by the user, or in CI, test runner 
-- takes no responsibility for this

-- Local Timezone

assert(
	DateTime.fromUnixTimestamp(1694078954):formatTime("local", "%Y-%m-%dT%H:%M:%SZ", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatTime() (local)"
)

local expectedLocalizedString

local dateCmd = process.spawn("bash", { "-c", "date +\"%A, %d %B %Y\" --date='@1693068988'" }, {
	env = {
		LC_ALL = "fr_FR.UTF-8 ",
	},
})

if dateCmd.ok then
	expectedLocalizedString = dateCmd.stdout:gsub("\n", "")
else
	error("Failed to execute date command")
end

assert(
	DateTime.fromUnixTimestamp(1693068988):formatTime("local", "%A, %d %B %Y", "fr")
		== expectedLocalizedString,
	`expected format specifier '%A, %d %B %Y' to return '{expectedLocalizedString}' for locale 'fr' (local)`
)
