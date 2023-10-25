local roblox = require("@lune/roblox")

local part = roblox.Instance.new("Part")

local id = part:GetDebugId()
assert(type(id) == "string", "GetDebugId should return a string")
assert(#id == 32, "GetDebugId should return a string with length 32")
assert(
	string.match(id, "^[0-9A-Fa-f]+$"),
	"GetDebugId should return a string with only hexadecimal characters"
)
