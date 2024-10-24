local ffi = require("@lune/ffi")
local process = require("@lune/process")

local function getLibSuffix(): string
	if process.os == "linux" then
		return "so"
	elseif process.os == "windows" then
		return "dll"
	elseif process.os == "macos" then
		return "dylib"
	end
	error("Unknown OS")
end

local function compile(file: string): ffi.LibData
	local out = file:gsub("%.c$", "." .. getLibSuffix())
	local gcc = process.exec("gcc", { "-shared", "-o", out, "-fPIC", file })
	if not gcc.ok then
		error("Failed to execute gcc command\n" .. gcc.stdout .. gcc.stderr)
	end

	return ffi.open(out)
end

return compile
