local STYLES_VALID = { "bold", "dim" }
local STYLES_INVALID = { "", "*bold*", "dimm", "megabright", "cheerful", "sad", " " }

local COLORS_VALID = { "black", "red", "green", "yellow", "blue", "purple", "cyan", "white" }
local COLORS_INVALID = { "", "gray", "grass", "red?", "super red", " ", "none" }

-- Test colors

for _, color in COLORS_VALID do
	if not pcall(console.setStyle, color :: any, nil) then
		error(string.format("Setting color failed for color '%s'", color))
	end
end

for _, color in COLORS_INVALID do
	if pcall(console.setStyle, color :: any, nil) then
		console.resetStyle()
		error(string.format("Setting color should have failed for color '%s' but succeeded", color))
	end
end

-- Test styles

for _, style in STYLES_VALID do
	if not pcall(console.setStyle, nil, style :: any) then
		error(string.format("Setting style failed for style '%s'", style))
	end
end

for _, style in STYLES_INVALID do
	if pcall(console.setStyle, nil, style :: any) then
		console.resetStyle()
		error(string.format("Setting style should have failed for style '%s' but succeeded", style))
	end
end
