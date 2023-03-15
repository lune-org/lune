-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Enum = _G.Enum

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
