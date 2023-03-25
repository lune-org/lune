local roblox = require("@lune/roblox") :: any
local BrickColor = roblox.BrickColor
local Color3 = roblox.Color3
local ColorSequence = roblox.ColorSequence
local ColorSequenceKeypoint = roblox.ColorSequenceKeypoint
local NumberRange = roblox.NumberRange
local NumberSequence = roblox.NumberSequence
local NumberSequenceKeypoint = roblox.NumberSequenceKeypoint
local Rect = roblox.Rect
local UDim = roblox.UDim
local UDim2 = roblox.UDim2
local Vector2 = roblox.Vector2
local Vector3 = roblox.Vector3
local CFrame = roblox.CFrame

local model = roblox.readModelFile("tests/roblox/rbx-test-files/models/attributes/binary.rbxm")[1]

model:SetAttribute("Foo", "Bar")
model:SetAttribute("CFrame", CFrame.identity)

local ATTRS_ACTUAL = model:GetAttributes()
local ATTRS_EXPECTED: { [string]: any } = {
	-- From the file
	Boolean = true,
	BrickColor = BrickColor.new("Really red"),
	Color3 = Color3.fromRGB(162, 0, 255),
	ColorSequence = ColorSequence.new({
		ColorSequenceKeypoint.new(0, Color3.new(1, 0, 0)),
		ColorSequenceKeypoint.new(0.5, Color3.new(0, 1, 0)),
		ColorSequenceKeypoint.new(1, Color3.new(0, 0, 1)),
	}),
	Number = 12345,
	NumberRange = NumberRange.new(5, 10),
	NumberSequence = NumberSequence.new({
		NumberSequenceKeypoint.new(0, 1),
		NumberSequenceKeypoint.new(0.5, 0),
		NumberSequenceKeypoint.new(1, 1),
	}),
	Rect = Rect.new(1, 2, 3, 4),
	String = "Hello, world!",
	UDim = UDim.new(0.5, 100),
	UDim2 = UDim2.new(0.5, 10, 0.7, 30),
	Vector2 = Vector2.new(10, 50),
	Vector3 = Vector3.new(1, 2, 3),
	Infinity = math.huge,
	NaN = 0 / 0,
	-- Extras we set
	Foo = "Bar",
	CFrame = CFrame.identity,
}

for name, value in ATTRS_EXPECTED do
	local actual = ATTRS_ACTUAL[name]
	if actual ~= value then
		if value ~= value and actual ~= actual then
			continue -- NaN
		end
		error(
			string.format(
				"Expected attribute '%s' to have value '%s', got value '%s'",
				name,
				tostring(value),
				tostring(actual)
			)
		)
	end
end
