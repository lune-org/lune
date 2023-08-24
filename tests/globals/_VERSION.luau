-- _VERSION global should follow the following format:
--[[
	_VERSION global must have the following format:

	Lune LUNE_MAJOR.LUNE_MINOR.LUNE_PATCH+LUAU_VERSION

	Examples:

	Lune 0.0.0+0
	Lune 1.0.0+500
	Lune 0.11.22+9999
]]

assert(_VERSION ~= nil, "_VERSION global is missing")
assert(type(_VERSION) == "string", "_VERSION global must be a string")

assert(string.sub(_VERSION, 1, 5) == "Lune ", "_VERSION global must start with 'Lune '")

local withoutPrefix = string.sub(_VERSION, 6)
local versions = string.split(withoutPrefix, "+")
assert(versions[1] ~= nil, "_VERSION global does not contain lune version")
assert(versions[2] ~= nil, "_VERSION global does not contain luau version")

local luneVersion = string.split(versions[1], ".")
assert(luneVersion[1] ~= nil, "_VERSION global is missing lune major version")
assert(luneVersion[2] ~= nil, "_VERSION global is missing lune minor version")
assert(luneVersion[3] ~= nil, "_VERSION global is missing lune patch version")

assert(
	string.find(versions[2], ".", 1, true) == nil,
	"_VERSION global contains more than one semver partial for luau version"
)

assert(tonumber(luneVersion[1]) ~= nil, "_VERSION global lune major version is not a number")
assert(tonumber(luneVersion[2]) ~= nil, "_VERSION global lune minor version is not a number")
assert(tonumber(luneVersion[3]) ~= nil, "_VERSION global lune patch version is not a number")
assert(tonumber(versions[2]) ~= nil, "_VERSION global luau version is not a number")

local rebuilt = string.format(
	"Lune %d.%d.%d+%d",
	tonumber(luneVersion[1]) :: number,
	tonumber(luneVersion[2]) :: number,
	tonumber(luneVersion[3]) :: number,
	tonumber(versions[2]) :: number
)

print("_VERSION:", _VERSION)
print("Rebuilt:", rebuilt)

assert(rebuilt == _VERSION)
