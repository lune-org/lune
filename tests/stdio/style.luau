local stdio = require("@lune/stdio")

local STYLES_VALID = { "reset", "bold", "dim" }
local STYLES_INVALID = { "", "*bold*", "dimm", "megabright", "cheerful", "sad", " " }

for _, style in STYLES_VALID do
	stdio.style(style :: any)
	stdio.style("reset")
end

for _, style in STYLES_INVALID do
	if pcall(stdio.style, style :: any) then
		stdio.style("reset")
		error(string.format("Setting style should have failed for style '%s' but succeeded", style))
	end
end
