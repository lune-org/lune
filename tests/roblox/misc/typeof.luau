local roblox = require("@lune/roblox") :: any

local TYPES_AND_VALUES = {
	Axes = roblox.Axes.new(),
	BrickColor = roblox.BrickColor.new("Really red"),
	CFrame = roblox.CFrame.new(),
	Color3 = roblox.Color3.new(0, 0, 0),
	ColorSequence = roblox.ColorSequence.new(roblox.Color3.new(0, 0, 0)),
	ColorSequenceKeypoint = roblox.ColorSequenceKeypoint.new(0, roblox.Color3.new(0, 0, 0)),
	Enums = roblox.Enum,
	Enum = roblox.Enum.KeyCode,
	EnumItem = roblox.Enum.KeyCode.Unknown,
	Faces = roblox.Faces.new(),
	Font = roblox.Font.new("Gotham"),
	NumberRange = roblox.NumberRange.new(0, 1),
	NumberSequence = roblox.NumberSequence.new(0, 1),
	NumberSequenceKeypoint = roblox.NumberSequenceKeypoint.new(0, 1),
	PhysicalProperties = roblox.PhysicalProperties.new(1, 1, 1),
	Ray = roblox.Ray.new(roblox.Vector3.zero, roblox.Vector3.one),
	Rect = roblox.Rect.new(0, 0, 0, 0),
	Region3 = roblox.Region3.new(roblox.Vector3.zero, roblox.Vector3.one),
	Region3int16 = roblox.Region3int16.new(
		roblox.Vector3int16.new(0, 0, 0),
		roblox.Vector3int16.new(1, 1, 1)
	),
	UDim = roblox.UDim.new(0, 0),
	UDim2 = roblox.UDim2.new(0, 0, 0, 0),
	Vector2 = roblox.Vector2.new(0, 0),
	Vector2int16 = roblox.Vector2int16.new(0, 0),
	Vector3 = roblox.Vector3.new(0, 0),
	Vector3int16 = roblox.Vector3int16.new(0, 0),
}

for name, value in TYPES_AND_VALUES :: { [string]: any } do
	if typeof(value) ~= name then
		error(
			string.format(
				"typeof() did not return correct value!\nExpected: %s\nActual: %s",
				name,
				typeof(value)
			)
		)
	end
end
