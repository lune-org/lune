local VALID_COLORS = { "black", "red", "green", "yellow", "blue", "purple", "cyan", "white" }
local INVALID_COLORS = { "", "gray", "grass", "red?", "super red", " ", "none" }

for _, color in VALID_COLORS do
	if not pcall(console.setColor, color :: any) then
		error(string.format("Setting color failed for color '%s'", color))
	end
end

for _, color in INVALID_COLORS do
	if pcall(console.setColor, color :: any) then
		console.resetColor()
		error(string.format("Setting color should have failed for color '%s' but succeeded", color))
	end
end
