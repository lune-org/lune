local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")
local stdio = require("@lune/stdio")

local TEST_FILES_DIR = process.cwd .. "tests/serde/test-files"
local INPUT_FILE = TEST_FILES_DIR .. "/loremipsum.txt"
local TEMP_FILE = TEST_FILES_DIR .. "/loremipsum.temp"

local INPUT_FILE_CONTENTS = fs.readFile(INPUT_FILE)

-- Make some utility functions for viewing unexpected differences in files easier

local function stringAsHex(str: string): string
	local hex = {}
	for i = 1, #str do
		table.insert(hex, string.format("%02x", string.byte(str, i)))
	end
	return table.concat(hex)
end

local function hexDiff(a: string, b: string): string
	local diff = {}
	for i = 1, math.max(#a, #b) do
		local aByte = if #a >= i then string.byte(a, i) else nil
		local bByte = if #b >= i then string.byte(b, i) else nil
		if aByte == nil then
			table.insert(
				diff,
				string.format(
					"%s%02x%s",
					stdio.color("green"),
					assert(bByte, "unreachable"),
					stdio.color("reset")
				)
			)
		elseif bByte == nil then
			table.insert(
				diff,
				string.format(
					"%s%02x%s",
					stdio.color("red"),
					assert(aByte, "unreachable"),
					stdio.color("reset")
				)
			)
		else
			if aByte == bByte then
				table.insert(diff, string.format("%02x", aByte))
			else
				table.insert(
					diff,
					string.format(
						"%s%02x%s",
						stdio.color("yellow"),
						assert(bByte, "unreachable"),
						stdio.color("reset")
					)
				)
			end
		end
	end
	return table.concat(diff)
end

-- Make some processing functions for manipulating output of certain commands

local function processNoop(output: string): string
	return output
end

local function processGzipSetOsUnknown(output: string): string
	-- This will set the os bits to be "unknown" so that the
	-- output is deterministic and consistent with serde lib
	-- https://www.rfc-editor.org/rfc/rfc1952#section-2.3.1
	local buf = buffer.fromstring(output)
	buffer.writeu8(buf, 9, 0xFF)
	return buffer.tostring(buf)
end

local function processLz4PrependSize(output: string): string
	-- Lune supports only lz4 with the decompressed size
	-- prepended to it, but the lz4 command line tool
	-- doesn't add this automatically, so we have to
	-- TODO: Remove this in the future when no longer needed
	local buf = buffer.create(4 + #output)
	buffer.writeu32(buf, 0, #INPUT_FILE_CONTENTS)
	buffer.writestring(buf, 4, output)
	return buffer.tostring(buf)
end

-- Make sure we have all of the different compression tools installed,
-- note that on macos we do not use the system-installed compression
-- tools, instead preferring to use homebrew-installed (gnu) ones

local BIN_BROTLI = if process.os == "macos" then "/opt/homebrew/bin/brotli" else "brotli"
local BIN_GZIP = if process.os == "macos" then "/opt/homebrew/bin/gzip" else "gzip"
local BIN_LZ4 = if process.os == "macos" then "/opt/homebrew/bin/lz4" else "lz4"

local function checkInstalled(program: string, args: { string }?)
	print("Checking if", program, "is installed")
	local result = process.spawn(program, args)
	if not result.ok then
		stdio.ewrite(string.format("Program '%s' is not installed\n", program))
		process.exit(1)
	end
end

checkInstalled(BIN_BROTLI, { "--version" })
checkInstalled(BIN_GZIP, { "--version" })
checkInstalled(BIN_LZ4, { "--version" })
-- checkInstalled("zlib", { "--version" })

-- Run them to generate files

local function run(program: string, args: { string }): string
	local result = process.spawn(program, args)
	if not result.ok then
		stdio.ewrite(string.format("Command '%s' failed\n", program))
		if #result.stdout > 0 then
			stdio.ewrite("stdout:\n")
			stdio.ewrite(result.stdout)
			stdio.ewrite("\n")
		end
		if #result.stderr > 0 then
			stdio.ewrite("stderr:\n")
			stdio.ewrite(result.stderr)
			stdio.ewrite("\n")
		end
		process.exit(1)
	else
		if #result.stdout > 0 then
			stdio.ewrite("stdout:\n")
			stdio.ewrite(result.stdout)
			stdio.ewrite("\n")
		end
	end
	return result.stdout
end

local OUTPUT_FILES = {
	{
		command = BIN_BROTLI,
		format = "brotli" :: serde.CompressDecompressFormat,
		args = { "--best", "-w", "22", TEMP_FILE },
		output = TEMP_FILE .. ".br",
		process = processNoop,
		final = INPUT_FILE .. ".br",
	},
	{
		command = BIN_GZIP,
		format = "gzip" :: serde.CompressDecompressFormat,
		args = { "--best", "--no-name", "--synchronous", TEMP_FILE },
		output = TEMP_FILE .. ".gz",
		process = processGzipSetOsUnknown,
		final = INPUT_FILE .. ".gz",
	},
	{
		command = BIN_LZ4,
		format = "lz4" :: serde.CompressDecompressFormat,
		args = { "--best", TEMP_FILE, TEMP_FILE .. ".lz4" },
		output = TEMP_FILE .. ".lz4",
		process = processLz4PrependSize,
		final = INPUT_FILE .. ".lz4",
	},
	-- {
	-- 	command = "zlib",
	--	format = "zlib" :: serde.CompressDecompressFormat,
	-- 	args = { "-c", INPUT_FILE },
	-- 	output = TEMP_FILE .. ".z",
	-- 	process = processNoop,
	--	final = INPUT_FILE .. ".z",
	-- },
}

for _, spec in OUTPUT_FILES do
	-- Write the temp file for the compression tool to read and use, then
	-- remove it, some tools may remove it on their own, so we ignore errors
	fs.writeFile(TEMP_FILE, INPUT_FILE_CONTENTS)
	print("\nRunning", spec.command, "with args", table.concat(spec.args, " "))
	local output = run(spec.command, spec.args)
	if #output > 0 then
		print("Output:", output)
	end
	pcall(fs.removeFile, TEMP_FILE)

	-- Read the compressed output file that is now supposed to exist
	local compressedContents
	pcall(function()
		compressedContents = fs.readFile(spec.output)
		compressedContents = spec.process(compressedContents)
		fs.removeFile(spec.output)
	end)
	if not compressedContents then
		error(
			string.format(
				"Nothing was written to output file while running %s:\n%s",
				spec.command,
				spec.output
			)
		)
	end

	-- If the newly compressed contents do not match the existing contents,
	-- warn the user about this and ask if they want to overwrite the file
	local existingContents = fs.readFile(spec.final)
	if compressedContents ~= existingContents then
		stdio.ewrite("\nCompressed file does not match existing contents!")
		stdio.ewrite("\n\nExisting:\n")
		stdio.ewrite(stringAsHex(existingContents))
		stdio.ewrite("\n\nCompressed:\n")
		stdio.ewrite(hexDiff(existingContents, compressedContents))
		stdio.ewrite("\n\n")
		local confirm = stdio.prompt("confirm", "Do you want to continue?")
		if confirm == true then
			print("Overwriting file!")
		else
			stdio.ewrite("\n\nAborting...\n")
			process.exit(1)
			return
		end
	end

	-- Check if the compressed contents can be decompressed using serde
	local decompressSuccess, decompressedContents =
		pcall(serde.decompress, spec.format, compressedContents)
	if not decompressSuccess then
		stdio.ewrite("\nCompressed contents could not be decompressed using serde!")
		stdio.ewrite("\n\nCompressed:\n")
		stdio.ewrite(stringAsHex(compressedContents))
		stdio.ewrite("\n\nError:\n")
		stdio.ewrite(tostring(decompressedContents))
		stdio.ewrite("\n\n")
		local confirm = stdio.prompt("confirm", "Do you want to continue?")
		if confirm == true then
			print("Ignoring decompression error!")
		else
			stdio.ewrite("\n\nAborting...\n")
			process.exit(1)
			return
		end
	end
	if decompressedContents ~= INPUT_FILE_CONTENTS then
		stdio.ewrite("\nCompressed contents were not decompressable properly using serde!")
		stdio.ewrite("\n\nOriginal:\n")
		stdio.ewrite(INPUT_FILE_CONTENTS)
		stdio.ewrite("\n\nDecompressed:\n")
		stdio.ewrite(decompressedContents)
		stdio.ewrite("\n\n")
		local confirm = stdio.prompt("confirm", "Do you want to continue?")
		if confirm == true then
			print("Ignoring decompression mismatch!")
		else
			stdio.ewrite("\n\nAborting...\n")
			process.exit(1)
			return
		end
	end

	-- Check if the compressed contents match the serde compressed contents,
	-- if they don't this will 100% make the tests fail, but maybe we are doing
	-- it because we are updating the serde library and need to update test files
	local serdeContents = serde.compress(spec.format, INPUT_FILE_CONTENTS)
	if compressedContents ~= serdeContents then
		stdio.ewrite("\nTemp file does not match contents compressed with serde!")
		stdio.ewrite("\nThis will caused the new compressed file to fail tests.")
		stdio.ewrite("\n\nSerde:\n")
		stdio.ewrite(stringAsHex(serdeContents))
		stdio.ewrite("\n\nCompressed:\n")
		stdio.ewrite(hexDiff(serdeContents, compressedContents))
		stdio.ewrite("\n\n")
		local confirm = stdio.prompt("confirm", "Do you want to continue?")
		if confirm == true then
			print("Writing new file!")
		else
			stdio.ewrite("\n\nAborting...\n")
			process.exit(1)
			return
		end
	end

	-- Finally, write the new compressed file
	fs.writeFile(spec.final, compressedContents)
	print("Wrote new file successfully to", spec.final)
end
