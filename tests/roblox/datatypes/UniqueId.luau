local roblox = require("@lune/roblox") :: any
local UniqueId = roblox.UniqueId

UniqueId.new()
UniqueId.fromString("1234567890123456")
UniqueId.fromString(buffer.fromstring("1234567890123456"))

assert(not pcall(function()
	return UniqueId.fromString(false)
end))
assert(not pcall(function()
	return UniqueId.fromString(1, 2)
end))

assert(typeof(UniqueId.null) :: any == "UniqueId")

assert(UniqueId.null == UniqueId.null)
assert(UniqueId.fromString("1234567890123456") == UniqueId.fromString("1234567890123456"))
assert(
	UniqueId.fromString("1234567890123456")
		== UniqueId.fromString(buffer.fromstring("1234567890123456"))
)

assert(tostring(UniqueId.null) == "00000000000000000000000000000000")
assert(
	tostring(
		UniqueId.fromString("\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16")
	) == "01020304050607080910111213141516"
)

-- Check that it reads it correctly from files
local fs = require("@lune/fs")
local dom = roblox.deserializePlace(
	fs.readFile("tests/roblox/rbx-test-files/places/baseplate-566/binary.rbxl")
)
local workspace = assert(dom:FindFirstChild("Workspace")) :: any
local expected = "44b188dace632b4702e9c68d004815fc"
assert(tostring(workspace.UniqueId) == expected)
