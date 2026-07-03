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

-- A small stable hash so each calling test compiles to its own file.
-- The test suite runs in parallel and several tests may share a single
-- lib.c; Windows cannot replace a library that another test has loaded
local function hashString(str: string): string
	local hash = 5381
	for i = 1, #str do
		hash = bit32.bxor(hash * 33 % 0x100000000, string.byte(str, i))
	end
	return string.format("%08x", hash)
end

local function compile(file: string): (ffi.LibData, string)
	local base = file:gsub("%.c$", "")
	local caller = debug.info(2, "s") or file
	local out = `{base}.{hashString(caller)}.{getLibSuffix()}`

	-- clang, unlike gcc, can also target arm64 windows
	local args = if process.os == "windows"
		then { "-shared", "-o", out, file }
		else { "-shared", "-o", out, "-fPIC", file }

	local result = process.exec("clang", args)
	if not result.ok then
		error("Failed to execute clang command\n" .. result.stdout .. result.stderr)
	end

	return ffi.open(out), out
end

return compile
