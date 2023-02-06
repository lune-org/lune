local COLORS_VALID =
	{ "reset", "black", "red", "green", "yellow", "blue", "purple", "cyan", "white" }
local COLORS_INVALID = { "", "gray", "grass", "red?", "super red", " ", "none" }

for _, color in COLORS_VALID do
	stdio.color(color :: any)
	stdio.color("reset")
end

for _, color in COLORS_INVALID do
	if pcall(stdio.color, color :: any) then
		stdio.color("reset")
		error(string.format("Setting color should have failed for color '%s' but succeeded", color))
	end
end
