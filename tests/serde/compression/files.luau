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
	{
		Format = "zstd",
		Source = "tests/serde/test-files/loremipsum.txt",
		Target = "tests/serde/test-files/loremipsum.txt.zst",
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
	testOperation("Compress", serde.compress, test.Format, source, target)
	testOperation("Compress", serde.compress, test.Format, buffer.fromstring(source), target)

	-- Decompression
	testOperation("Decompress", serde.decompress, test.Format, target, source)
	testOperation("Decompress", serde.decompress, test.Format, buffer.fromstring(target), source)
end

if failed then
	process.exit(1)
end
