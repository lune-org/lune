local roblox = require("@lune/roblox") :: any
local Color3 = roblox.Color3

-- Color3:ToHex() should match Roblox exactly: channels are rounded to the
-- nearest integer, and the resulting hex string is lowercase.

-- Boundaries

assert(Color3.new(0, 0, 0):ToHex() == "000000")
assert(Color3.new(1, 1, 1):ToHex() == "ffffff")

-- Rounding
--
-- 0.001 * 255 = 0.255, which rounds down to 0
-- 0.5 * 255 = 127.5, which rounds up to 128 (0x80) rather than truncating to 127 (0x7f)
-- 0.9999 * 255 = 254.97, which rounds up to 255 (0xff) rather than truncating to 254 (0xfe)

assert(Color3.new(0.001, 0, 0):ToHex() == "000000")
assert(Color3.new(0.5, 0.5, 0.5):ToHex() == "808080")
assert(Color3.new(0.9999, 0, 0):ToHex() == "ff0000")

-- Channels are converted independently

assert(Color3.new(1, 0.5, 0):ToHex() == "ff8000")
assert(Color3.new(0, 0.9999, 0.001):ToHex() == "00ff00")

-- Casing
--
-- Every hex digit above 9 must be lowercase

assert(Color3.new(170 / 255, 187 / 255, 204 / 255):ToHex() == "aabbcc")
assert(Color3.new(221 / 255, 238 / 255, 255 / 255):ToHex() == "ddeeff")

-- Values are clamped to the 0-255 range

assert(Color3.new(-1, -1, -1):ToHex() == "000000")
assert(Color3.new(2, 2, 2):ToHex() == "ffffff")

-- Round-tripping through fromHex is stable at 8-bit precision

local function assertRoundTrips(hex: string)
	assert(Color3.fromHex(hex):ToHex() == hex)
end

assertRoundTrips("000000")
assertRoundTrips("ffffff")
assertRoundTrips("808080")
assertRoundTrips("aabbcc")
assertRoundTrips("ff8000")

-- fromHex accepts either casing, and always produces the lowercase form

assert(Color3.fromHex("AABBCC"):ToHex() == "aabbcc")
assert(Color3.fromHex("aabbcc"):ToHex() == "aabbcc")
assert(Color3.fromHex("AABBCC") == Color3.fromHex("aabbcc"))
