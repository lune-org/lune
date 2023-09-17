local roblox = require("@lune/roblox") :: any
local CFrame = roblox.CFrame
local Vector3 = roblox.Vector3
local Instance = roblox.Instance

local COMPONENT_NAMES =
	{ "X", "Y", "Z", "R00", "R01", "R02", "R10", "R11", "R12", "R20", "R21", "R22" }

local function formatCFrame(cf)
	local rot = Vector3.new(cf:ToOrientation())
	return string.format(
		"%.2f, %.2f, %.2f | %.2f, %.2f, %.2f",
		cf.Position.X,
		cf.Position.Y,
		cf.Position.Z,
		math.deg(rot.X),
		math.deg(rot.Y),
		math.deg(rot.Z)
	)
end

local function assertEq(actual, expected)
	local actComps: { number } = { actual:GetComponents() }
	local expComps: { number } = { expected:GetComponents() }
	for index, actComp in actComps do
		local expComp = expComps[index]
		if math.abs(expComp - actComp) >= (1 / 512) then
			error(
				string.format(
					"Expected component '%s' to be %.4f, got %.4f"
						.. "\nActual:   %s"
						.. "\nExpected: %s",
					COMPONENT_NAMES[index],
					expComp,
					actComp,
					formatCFrame(actual),
					formatCFrame(expected)
				)
			)
		end
	end
end

-- Constructors & properties

CFrame.new()
CFrame.new(0, 0, 0)
CFrame.new(0 / 0, 0 / 0, 0 / 0)
CFrame.new(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1)

assert(not pcall(function()
	return CFrame.new(false)
end))
assert(not pcall(function()
	return CFrame.new("", "")
end))
assert(not pcall(function()
	return CFrame.new(newproxy(true))
end))

assert(CFrame.new(1, 2, 3).X == 1)
assert(CFrame.new(1, 2, 3).Y == 2)
assert(CFrame.new(1, 2, 3).Z == 3)

assertEq(
	CFrame.fromMatrix(
		Vector3.new(1, 2, 3),
		Vector3.new(1, 0, 0),
		Vector3.new(0, 1, 0),
		Vector3.new(0, 0, 1)
	),
	CFrame.new(1, 2, 3)
)

assertEq(CFrame.new(1, 2, 3, 1, 0, 0, 0, 1, 0, 0, 0, 1), CFrame.new(1, 2, 3))

-- Constants

assertEq(CFrame.identity, CFrame.new())
assertEq(CFrame.identity, CFrame.new(0, 0, 0))
assertEq(CFrame.identity, CFrame.Angles(0, 0, 0))
assertEq(CFrame.identity, CFrame.fromOrientation(0, 0, 0))

-- Ops

assertEq(CFrame.new(2, 4, 8) + Vector3.new(1, 1, 2), CFrame.new(3, 5, 10))
assertEq(CFrame.new(2, 4, 8) - Vector3.new(1, 1, 2), CFrame.new(1, 3, 6))
assertEq(CFrame.new(2, 4, 8) * CFrame.new(1, 1, 2), CFrame.new(3, 5, 10))
assert(CFrame.new(2, 4, 8) * Vector3.new(1, 1, 2) == Vector3.new(3, 5, 10))

-- Mult ops with rotated CFrames

assertEq(
	CFrame.fromOrientation(0, math.rad(90), 0) * CFrame.fromOrientation(math.rad(5), 0, 0),
	CFrame.fromOrientation(math.rad(5), math.rad(90), 0)
)
assertEq(
	CFrame.fromOrientation(0, math.rad(90), 0) * CFrame.new(0, 0, -5),
	CFrame.new(-5, 0, 0) * CFrame.fromOrientation(0, math.rad(90), 0)
)

-- World & object space conversions

local offset = CFrame.new(0, 0, -5)
assert(offset:ToWorldSpace(offset).Z == offset.Z * 2)
assert(offset:ToObjectSpace(offset).Z == 0)

assert(select("#", offset:ToWorldSpace(offset, offset, offset)) == 3)
assert(select("#", offset:ToObjectSpace(offset, offset, offset)) == 3)

local world = CFrame.fromOrientation(0, math.rad(90), 0) * CFrame.new(0, 0, -5)
local world2 = CFrame.fromOrientation(0, -math.rad(90), 0) * CFrame.new(0, 0, -5)
assertEq(CFrame.identity:ToObjectSpace(world), world)
assertEq(
	world:ToObjectSpace(world2),
	CFrame.fromOrientation(0, math.rad(180), 0) * CFrame.new(0, 0, -10)
)

-- Look

assertEq(CFrame.fromOrientation(0, math.rad(90), 0), CFrame.lookAt(Vector3.zero, -Vector3.xAxis))
assertEq(CFrame.fromOrientation(0, -math.rad(90), 0), CFrame.lookAt(Vector3.zero, Vector3.xAxis))
assertEq(
	CFrame.new(0, 0, -5) * CFrame.fromOrientation(0, math.rad(90), 0),
	CFrame.lookAt(Vector3.new(0, 0, -5), Vector3.new(0, 0, -5) - Vector3.xAxis)
)

-- TODO: More methods

-- CFrames on instances

local part0 = Instance.new("Part")
local part1 = Instance.new("MeshPart")

part0.CFrame = CFrame.fromOrientation(-math.rad(45), math.rad(180), 0)
part1.CFrame = CFrame.new(0, 0, -5) * CFrame.fromOrientation(0, math.rad(180), 0)

local weld = Instance.new("Weld")
weld.C0 = part0.CFrame:ToObjectSpace(part1.CFrame)
weld.Part0 = part0
weld.Part1 = part1
weld.Parent = part1

assertEq(weld.C0, CFrame.new(0, -3.5355, 3.5355) * CFrame.fromOrientation(math.rad(45), 0, 0))
