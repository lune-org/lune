local DateTime = require("@lune/datetime")

local now = DateTime.now()
local nowRfc = now:toRfc3339()

-- Make sure we have separator characters, T to separate date & time, + or Z to separate timezone

local dateTimeSplitIdx = string.find(nowRfc, "T")
local timezoneSplitIdx = string.find(nowRfc, "+")
local timezoneZeroedIdx = string.find(nowRfc, "Z")

assert(dateTimeSplitIdx ~= nil, "Missing date & time separator 'T' in RFC 3339 string")
assert(timezoneSplitIdx ~= nil or timezoneZeroedIdx ~= nil, "Missing timezone separator '+' or 'Z' in RFC 3339 string")

-- Split date (before T) by dashes, split time (after T, before + or Z)
-- by colons, we should then get 3 substrings for each of date & time

local dateParts = string.split(string.sub(nowRfc, 1, dateTimeSplitIdx - 1), "-")
local timeParts =
	string.split(string.sub(nowRfc, dateTimeSplitIdx + 1, ((timezoneSplitIdx or timezoneZeroedIdx) :: number) - 1), ":")

assert(#dateParts == 3, "Date partial of RFC 3339 should consist of 3 substrings, separated by '-'")
assert(#timeParts == 3, "Time partial of RFC 3339 should consist of 3 substrings, separated by ':'")

-- date should be in format YYYY:MM::DD
-- time should be in format HH:MM:SS with optional fraction for seconds

assert(string.match(dateParts[1], "^%d%d%d%d$"), "Date partial should have 4 digits for year")
assert(string.match(dateParts[2], "^%d%d$"), "Date partial should have 2 digits for month")
assert(string.match(dateParts[3], "^%d%d$"), "Date partial should have 2 digits for day")

assert(string.match(timeParts[1], "^%d%d$"), "Time partial should have 2 digits for hour")
assert(string.match(timeParts[2], "^%d%d$"), "Time partial should have 2 digits for minute")
assert(
	string.match(timeParts[3], "^%d%d%.?%d*$") and tonumber(timeParts[3]) ~= nil,
	"Time partial should have minimum 2 digits with optional fraction for seconds"
)

-- Timezone specifier is either 'Z' for zeroed out timezone (no offset),
-- in which case we don't need to check anything other than it being the
-- last character, or it can be a timezone offset in the format HH::MM

if timezoneZeroedIdx ~= nil then
	-- No timezone offset
	assert(timezoneZeroedIdx == #nowRfc, "Timezone specifier 'Z' must be at the last character in RFC 3339 string")
elseif timezoneSplitIdx ~= nil then
	-- Timezone offset
	local timezoneParts = string.split(string.sub(nowRfc, timezoneSplitIdx + 1), ":")
	assert(#timezoneParts == 2, "Timezone partial should consist of 2 substings, separated by ':'")
	assert(string.match(timezoneParts[1], "^%d%d$"), "Timezone partial should have 2 digits for hour")
	assert(string.match(timezoneParts[2], "^%d%d$"), "Timezone partial should have 2 digits for minute")
else
	error("unreachable")
end
