local DateTime = require("@lune/datetime")

local now = DateTime.now()
local nowRfc = now:toRfcDate()

assert(type(nowRfc) == "string", "toRfcDate should return a string")
assert(
	string.match(nowRfc, "^%a%a%a, %d%d? %a%a%a %d%d%d%d %d%d:%d%d:%d%d [+-]%d%d%d%d$"),
	"RFC 2822 date string does not match expected format"
)

-- Extract components of the RFC 2822 string
local day, date, month, year, time, timezone =
	nowRfc:match("^(%a%a%a), (%d%d?) (%a%a%a) (%d%d%d%d) (%d%d:%d%d:%d%d) ([+-]%d%d%d%d)$")

if not day or not date or not month or not year or not time or not timezone then
	error("Failed to extract components from RFC 2822 date string")
end

-- Validate month
local validMonths = {
	Jan = true, Feb = true, Mar = true, Apr = true, May = true, Jun = true,
	Jul = true, Aug = true, Sep = true, Oct = true, Nov = true, Dec = true,
}
assert(validMonths[month], "Month must be a valid RFC 2822 month abbreviation")

-- Validate year
assert(string.match(year, "^%d%d%d%d$"), "Year must be a 4-digit number")

-- Validate date
local dayNum = tonumber(date)
assert(dayNum >= 1 and dayNum <= 31, "Date must be between 1 and 31")

-- Validate time
local hour, minute, second = time:match("^(%d%d):(%d%d):(%d%d)$")
if not hour or not minute or not second then
	error("Failed to extract time components from RFC 2822 date string")
end

assert(hour and tonumber(hour) >= 0 and tonumber(hour) < 24, "Hour must be between 0 and 23")
assert(minute and tonumber(minute) >= 0 and tonumber(minute) < 60, "Minute must be between 0 and 59")
assert(second and tonumber(second) >= 0 and tonumber(second) < 60, "Second must be between 0 and 59")

-- Validate timezone
local tzHour, tzMinute = timezone:match("^([+-]%d%d)(%d%d)$")
if not tzHour or not tzMinute then
	error("Failed to extract timezone components from RFC 2822 date string")
end

assert(tzHour and tonumber(tzHour) and tonumber(tzHour) <= 14, "Timezone hour offset must be between -14 and +14")
assert(tzMinute and tonumber(tzMinute) >= 0 and tonumber(tzMinute) < 60, "Timezone minute offset must be between 0 and 59")