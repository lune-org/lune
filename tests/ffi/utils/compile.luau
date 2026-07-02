local ffi = require("@lune/ffi")
local fs = require("@lune/fs")
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

-- A small stable hash so each calling test compiles to its own temp file.
-- The test suite runs in parallel and several tests may share a single lib.c;
-- without a unique intermediate, concurrent `gcc` runs would truncate each
-- other's output ("file too short"). We compile to a per-caller temp file and
-- then atomically rename it onto the shared output.
local function hashString(str: string): string
	local hash = 5381
	for i = 1, #str do
		hash = bit32.bxor(hash * 33 % 0x100000000, string.byte(str, i))
	end
	return string.format("%08x", hash)
end

local function compile(file: string): ffi.LibData
	local base = file:gsub("%.c$", "")
	local suffix = getLibSuffix()
	local out = `{base}.{suffix}`

	-- Identify the calling test so concurrent compiles never collide. The temp
	-- keeps the library suffix so it is covered by the tests/ffi *.so gitignore.
	local caller = debug.info(2, "s") or file
	local temp = `{base}.{hashString(caller)}.tmp.{suffix}`

	local gcc = process.exec("gcc", { "-shared", "-o", temp, "-fPIC", file })
	if not gcc.ok then
		error("Failed to execute gcc command\n" .. gcc.stdout .. gcc.stderr)
	end

	-- Atomic replace: readers see either the old or the new complete file,
	-- never a half-written one
	fs.move(temp, out, { overwrite = true })

	return ffi.open(out)
end

return compile
