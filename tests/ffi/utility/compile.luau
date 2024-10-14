local process = require("@lune/process")
local function compile(file, out)
	local gcc = process.spawn("gcc", { "-shared", "-o", out, "-fPIC", file })
	if not gcc.ok then
		error("Failed to execute gcc command\n" .. gcc.stdout .. gcc.stderr)
	end
end

return compile
