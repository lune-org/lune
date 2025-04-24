--[[
	The "compress" tests are currently disabled, and may not ever be enabled again.

	Unit testing that our compression implementation produces the exact result saved
	in test files is very fragile and prone to breakage whenever one of the crates
	we depend on for compression algorithms updates, be it to improve compression
	ratio, speed, memory usage, or any other metric.

	Instead, we should focus on the correctness of the implementation, ensuring that
	any files we know are good from other sources can be decompressed without issues.

	In the future, it is probably a good idea to rewrite these compression tests so
	that they do not rely on any exact output, but instead call some external program
	to verify that others can decompress without issues. This is a challenge in and of
	itself since finding CLI programs for testing compression is near-impossible, but alas.
]]
local TEST_COMPRESS = false
local TEST_DECOMPRESS = true

local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")
local stdio = require("@lune/stdio")

type Test = {
	Format: serde.CompressDecompressFormat,
	Source: string,
	Target: string,
}

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
local function testOperation(
	operationName: "Compress" | "Decompress",
	operation: (
		format: serde.CompressDecompressFormat,
		s: buffer | string
	) -> string,
	format: serde.CompressDecompressFormat,
	source: string | buffer,
	target: string
)
	local success, res = pcall(operation, format, source)
	if not success then
		stdio.ewrite(
			string.format(
				"%sing source using '%s' format threw an error!\n%s",
				operationName,
				tostring(format),
				tostring(res)
			)
		)
		failed = true
	elseif res ~= target then
		stdio.ewrite(
			string.format(
				"%sing source using '%s' format did not produce target!\n",
				operationName,
				tostring(format)
			)
		)
		stdio.ewrite(
			string.format(
				"%sed (%d chars long):\n%s\nTarget (%d chars long):\n%s\n\n",
				operationName,
				#res,
				tostring(res),
				#target,
				tostring(target)
			)
		)
		failed = true
	end
end

for _, test in TESTS do
	local source = fs.readFile(test.Source)
	local target = fs.readFile(test.Target)

	-- Compression
	if TEST_COMPRESS then
		testOperation("Compress", serde.compress, test.Format, source, target)
		testOperation("Compress", serde.compress, test.Format, buffer.fromstring(source), target)
	end

	-- Decompression
	if TEST_DECOMPRESS then
		testOperation("Decompress", serde.decompress, test.Format, target, source)
		testOperation(
			"Decompress",
			serde.decompress,
			test.Format,
			buffer.fromstring(target),
			source
		)
	end
end

if failed then
	process.exit(1)
end
