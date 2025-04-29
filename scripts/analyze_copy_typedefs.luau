local fs = require("@lune/fs")

fs.writeDir("./types")

for _, dir in fs.readDir("./crates") do
	local std = string.match(dir, "^lune%-std%-(%w+)$")
	if std ~= nil then
		local from = `./crates/{dir}/types.d.luau`
		if fs.isFile(from) then
			local to = `./types/{std}.luau`
			fs.copy(from, to, true)
		end
	end
end
