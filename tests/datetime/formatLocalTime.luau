local DateTime = require("@lune/datetime")
local process = require("@lune/process")

local expectedTimeString = os.date("%Y-%m-%dT%H:%M:%S", 1694078954)

assert(
	DateTime.fromUnixTimestamp(1694078954):formatLocalTime("%Y-%m-%dT%H:%M:%S", "en")
		== expectedTimeString,
	"invalid ISO 8601 formatting for DateTime.formatLocalTime()"
)

--[[
	The rest of this test requires 'fr_FR.UTF-8 UTF-8' to be in /etc/locale.gen to pass

	Locale should be set up by a script, or by the user,
	or in CI, test runner takes no responsibility for this

	To run tests related to locales, one must
	explicitly provide the `--test-locales` flag
]]
local runLocaleTests = false

for _, arg in process.args do
	if arg == "--test-locales" then
		runLocaleTests = true
		break
	end
end

if not runLocaleTests then
	return
end

local dateCmd = process.exec("bash", { "-c", "date +\"%A, %d %B %Y\" --date='@1693068988'" }, {
	env = {
		LC_ALL = "fr_FR.UTF-8 ",
	},
})
assert(dateCmd.ok, "Failed to execute date command")

local expectedLocalizedString = string.gsub(dateCmd.stdout, "\n", "")

assert(
	DateTime.fromUnixTimestamp(1693068988):formatLocalTime("%A, %d %B %Y", "fr")
		== expectedLocalizedString,
	`expected format specifier '%A, %d %B %Y' to return '{expectedLocalizedString}' for locale 'fr' (local)`
)

assert(
	DateTime.fromUnixTimestamp(1693068988):formatUniversalTime("%A, %d %B %Y", "fr")
		== "samedi, 26 août 2023",
	"expected format specifier '%A, %d %B %Y' to return 'samedi, 26 août 2023' for locale 'fr' (UTC)"
)
