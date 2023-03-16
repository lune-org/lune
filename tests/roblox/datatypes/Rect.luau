-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Vector2 = _G.Vector2
local Rect = _G.Rect

-- Constructors & properties

Rect.new()
Rect.new(0)
Rect.new(0, 0)
Rect.new(0, 0, 0)
Rect.new(0, 0, 0, 0)
Rect.new(0 / 0, 0, 0 / 0, 0)

Rect.new(Vector2.zero)
Rect.new(Vector2.zero, Vector2.zero)

assert(not pcall(function()
	return Rect.new(false)
end))
assert(not pcall(function()
	return Rect.new("", "")
end))
assert(not pcall(function()
	return Rect.new(newproxy(true))
end))

assert(Rect.new(1, 0, 2, 4).Min == Vector2.new(1, 0))
assert(Rect.new(1, 0, 2, 4).Max == Vector2.new(2, 4))
assert(Rect.new(0, 0, 1, 2).Width == 1)
assert(Rect.new(0, 0, 1, 2).Height == 2)

assert(Rect.new(Vector2.new(1, 0), Vector2.new(2, 4)).Min == Vector2.new(1, 0))
assert(Rect.new(Vector2.new(1, 0), Vector2.new(2, 4)).Max == Vector2.new(2, 4))
assert(Rect.new(Vector2.new(1, 0), Vector2.new(2, 4)).Width == 1)
assert(Rect.new(Vector2.new(1, 0), Vector2.new(2, 4)).Height == 4)

-- Ops

assert(Rect.new(2, 4, 6, 8) + Rect.new(1, 1, 1, 1) == Rect.new(3, 5, 7, 9))
assert(Rect.new(2, 4, 6, 8) - -Rect.new(1, 1, 1, 1) == Rect.new(3, 5, 7, 9))
assert(Rect.new(2, 4, 6, 8) - Rect.new(1, 1, 1, 1) == Rect.new(1, 3, 5, 7))
