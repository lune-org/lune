-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Enum = _G.Enum
local Font = _G.Font

-- Constructors

Font.new("")
Font.new("", Enum.FontWeight.Bold)
Font.new("", Enum.FontWeight.Bold, Enum.FontStyle.Italic)

assert(not pcall(function()
	Font.new("", Enum.FontStyle.Italic, Enum.FontWeight.Bold)
end))
assert(not pcall(function()
	return Font.new()
end))
assert(not pcall(function()
	return Font.new(false)
end))
assert(not pcall(function()
	return Font.new("", "")
end))
assert(not pcall(function()
	return Font.new(newproxy(true))
end))

Font.fromEnum(Enum.Font.Gotham)
Font.fromEnum(Enum.Font.GothamMedium)
Font.fromEnum(Enum.Font.GothamBold)

Font.fromName("file-name")
Font.fromName("file-name", Enum.FontWeight.Bold)
Font.fromName("file-name", Enum.FontWeight.Bold, Enum.FontStyle.Italic)

Font.fromId(1234567890)
Font.fromId(1234567890, Enum.FontWeight.Bold)
Font.fromId(1234567890, Enum.FontWeight.Bold, Enum.FontStyle.Italic)

-- Properties

local arial = "rbxasset://fonts/families/Arial.json"
assert(Font.new(arial).Family == arial)
assert(Font.fromName("Arial").Family == arial)
assert(Font.fromEnum(Enum.Font.Arial).Family == arial)
assert(Font.fromId(1234567890).Family == "rbxassetid://1234567890")

for _, weight in Enum.FontWeight:GetEnumItems() do
	assert(Font.new(arial, weight).Weight == weight)
end

for _, style in Enum.FontStyle:GetEnumItems() do
	assert(Font.new(arial, Enum.FontWeight.Regular, style).Style == style)
end
