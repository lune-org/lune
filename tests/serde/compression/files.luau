local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")
local stdio = require("@lune/stdio")

type Test = {
	Format: serde.CompressDecompressFormat,
	Source: string,
	Target: string,
}

local isWindows = process.os == "windows";

local TESTS: { Test } = {
	{
		Format = "brotli",
		Source = "tests/serde/test-files/loremipsum.txt",
		Target = "tests/serde/test-files/loremipsum.txt.br",
	},
	{
		Format = "gzip",
		Source = "tests/serde/test-files/loremipsum.txt",
		Target = "tests/serde/test-files/loremipsum.txt.gz",
	},
	{
		Format = "lz4",
		Source = "tests/serde/test-files/loremipsum.txt",
		Target = "tests/serde/test-files/loremipsum.txt.lz4",
	},
	{
		Format = "zlib",
		Source = "tests/serde/test-files/loremipsum.txt",
		Target = "tests/serde/test-files/loremipsum.txt.z",
	},
}

local failed = false
for _, test in TESTS do
	local source = fs.readFile(test.Source)
	local target = fs.readFile(test.Target)

	if isWindows then
		-- Windows: CRLF -> LF
		source = string.gsub(source, "\r\n", "\n", #source - 2);
	end

	local success, compressed = pcall(serde.compress, test.Format, source)
	if not success then
		stdio.ewrite(
			string.format(
				"Compressing source using '%s' format threw an error!\n%s",
				tostring(test.Format),
				tostring(compressed)
			)
		)
		failed = true
		continue
	elseif compressed ~= target then
		stdio.ewrite(
			string.format(
				"Compressing source using '%s' format did not produce target!\n",
				tostring(test.Format)
			)
		)
		stdio.ewrite(
			string.format(
				"Compressed (%d chars long):\n%s\nTarget (%d chars long):\n%s\n\n",
				#compressed,
				tostring(compressed),
				#target,
				tostring(target)
			)
		)
		failed = true
		continue
	end

	local success2, decompressed = pcall(serde.decompress, test.Format, target)
	if not success2 then
		stdio.ewrite(
			string.format(
				"Decompressing source using '%s' format threw an error!\n%s",
				tostring(test.Format),
				tostring(decompressed)
			)
		)
		failed = true
		continue
	elseif decompressed ~= source then
		stdio.ewrite(
			string.format(
				"Decompressing target using '%s' format did not produce source!\n",
				tostring(test.Format)
			)
		)
		stdio.ewrite(
			string.format(
				"Decompressed (%d chars long):\n%s\n\n",
				#decompressed,
				tostring(decompressed)
			)
		)
		failed = true
		continue
	end
end

if failed then
	process.exit(1)
end
