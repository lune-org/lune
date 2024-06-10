--!nolint

local regex = require("@lune/regex")

local re = regex.new("[0-9]+")
assert(tostring(re) == "[0-9]+")
assert(typeof(re) == "Regex")

local mtch = re:find("1337 wow")
assert(tostring(mtch) == "1337")
assert(typeof(mtch) == "RegexMatch")

local re2 = regex.new("([0-9]+) ([0-9]+) wow! ([0-9]+) ([0-9]+)")
local captures = re2:captures("1337 125600 wow! 1984 0")
assert(tostring(captures) == "4")
assert(typeof(captures) == "RegexCaptures")
