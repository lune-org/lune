local roblox = require("@lune/roblox") :: any
local Enum = roblox.Enum

-- Constructors & properties

assert(tostring(Enum) == "Enum")
assert(tostring(Enum.KeyCode) == "Enum.KeyCode")
assert(tostring(Enum.KeyCode.X) == "Enum.KeyCode.X")

-- NOTE: We use the axis enum here since it is unlikely
-- any more will be added to it and change the value
assert(Enum.Axis.X.Name == "X")
assert(Enum.Axis.X.Value == 0)

-- Methods

local foundKeyCode = false
for _, enum in Enum:GetEnums() do
	if enum == Enum.KeyCode then
		foundKeyCode = true
		break
	end
end
assert(foundKeyCode, "GetEnums did not contain Enum.KeyCode")

local foundKeyCodeX = false
for _, keyCode in Enum.KeyCode:GetEnumItems() do
	if keyCode == Enum.KeyCode.X then
		foundKeyCodeX = true
		break
	end
end
assert(foundKeyCodeX, "GetEnumItems did not contain X for Enum.KeyCode")

for _, keyCode in Enum.KeyCode:GetEnumItems() do
	assert(
		Enum.KeyCode:FromValue(keyCode.Value) == keyCode,
		"FromValue did not return the KeyCode from its value"
	)
	assert(
		Enum.KeyCode:FromName(keyCode.Name) == keyCode,
		"FromName did not return the KeyCode from its name"
	)
end

-- Enum items should keep a stable identity, so that they work as table keys and with rawequal

assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode.X),
	"Reading an enum item twice should give the same userdata"
)
assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode:FromName("X")),
	"FromName should give the same userdata"
)
assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode:FromValue(Enum.KeyCode.X.Value)),
	"FromValue should give the same userdata"
)

for _, keyCode in Enum.KeyCode:GetEnumItems() do
	assert(
		rawequal(keyCode, Enum.KeyCode[keyCode.Name]),
		"GetEnumItems should give the same userdata"
	)
end

local keyBinds = { [Enum.KeyCode.X] = "jump" }
assert(keyBinds[Enum.KeyCode.X] == "jump", "Enum items should be usable as table keys")

local part = roblox.Instance.new("Part")
part.Material = Enum.Material.Wood
assert(
	rawequal(part.Material, Enum.Material.Wood),
	"Enum items read from properties should give the same userdata"
)
part:SetAttribute("Material", Enum.Material.Wood)
assert(
	rawequal(part:GetAttribute("Material"), Enum.Material.Wood),
	"Enum items read from attributes should give the same userdata"
)
