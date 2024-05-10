local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")
local stdio = require("@lune/stdio")

-- Make sure we have all of the different compression tools installed

local function checkInstalled(program: string, args: { string }?)
	print("Checking if", program, "is installed")
	local result = process.spawn(program, args)
	if not result.ok then
		stdio.ewrite(string.format("Program '%s' is not installed\n", program))
		process.exit(1)
	end
end

checkInstalled("brotli", { "--version" })
checkInstalled("gzip", { "--version" })
checkInstalled("lz4", { "--version" })
-- checkInstalled("zlib", { "--version" })

-- Run them to generate files

local function run(program: string, args: { string }): string
	local result = process.spawn(program, args)
	if not result.ok then
		stdio.ewrite(string.format("Command '%s' failed\n", program))
		if result.stdout then
			stdio.ewrite("stdout: ")
			stdio.ewrite(result.stdout)
			stdio.ewrite("\n")
		end
		if result.stderr then
			stdio.ewrite("stderr: ")
			stdio.ewrite(result.stderr)
			stdio.ewrite("\n")
		end
		process.exit(1)
	end
	return result.stdout
end

local TEST_FILES_DIR = process.cwd .. "tests/serde/test-files"
local INPUT_FILE = TEST_FILES_DIR .. "/loremipsum.txt"
local TEMP_FILE = TEST_FILES_DIR .. "/loremipsum.temp"

local INPUT_FILE_CONTENTS = fs.readFile(INPUT_FILE)

local OUTPUT_FILES = {
	{
		command = "brotli",
		format = "brotli" :: serde.CompressDecompressFormat,
		args = { "--best", "-w", "22", TEMP_FILE },
		output = TEMP_FILE .. ".br",
		final = INPUT_FILE .. ".br",
	},
	{
		command = "gzip",
		format = "gzip" :: serde.CompressDecompressFormat,
		args = { "--best", "--no-name", TEMP_FILE },
		output = TEMP_FILE .. ".gz",
		final = INPUT_FILE .. ".gz",
	},
	{
		command = "lz4",
		format = "lz4" :: serde.CompressDecompressFormat,
		args = { "--best", TEMP_FILE },
		output = TEMP_FILE .. ".lz4",
		final = INPUT_FILE .. ".lz4",
	},
	-- {
	-- 	command = "zlib",
	--	format = "zlib" :: serde.CompressDecompressFormat,
	-- 	args = { "-c", INPUT_FILE },
	-- 	output = TEST_FILES_DIR .. "/loremipsum.txt.z",
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
		stdio.ewrite("Output file does not match expected contents\n")
		stdio.ewrite("\nCompressed:\n")
		stdio.ewrite(compressedContents)
		stdio.ewrite("\n")
		stdio.ewrite("\nExisting:\n")
		stdio.ewrite(existingContents)
		stdio.ewrite("\n\n")
		local confirm = stdio.prompt("confirm", "Do you want to overwrite the file?")
		if confirm == true then
			print("Overwriting file!")
			fs.writeFile(spec.output, compressedContents)
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
		stdio.ewrite("Temp file does not match contents compressed with serde!\n")
		stdio.ewrite("This will caused the new compressed file to fail tests.\n")
		stdio.ewrite("\nCompressed:\n")
		stdio.ewrite(compressedContents)
		stdio.ewrite("\n")
		stdio.ewrite("\nSerde:\n")
		stdio.ewrite(serdeContents)
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
