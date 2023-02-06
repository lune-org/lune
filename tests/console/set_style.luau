local VALID_STYLES = { "bold", "dim" }
local INVALID_STYLES = { "", "*bold*", "dimm", "megabright", "cheerful", "sad", " " }

for _, style in VALID_STYLES do
	if not pcall(console.setStyle, style :: any) then
		error(string.format("Setting style failed for style '%s'", style))
	end
end

for _, style in INVALID_STYLES do
	if pcall(console.setStyle, style :: any) then
		console.resetStyle()
		error(string.format("Setting style should have failed for style '%s' but succeeded", style))
	end
end
