local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")
local stdio = require("@lune/stdio")

local FORMATS: { serde.CompressDecompressFormat } = { "brotli", "gzip", "lz4", "zlib" }
local FILES: { string } = {
	"tests/serde/test-files/loremipsum.txt",
	"tests/serde/test-files/uncompressed.csv",
	"tests/serde/test-files/uncompressed.json",
	"tests/serde/test-files/uncompressed.yaml",
}

local failed = false
for _, filePath in FILES do
	local source = fs.readFile(filePath)
	for _, format: serde.CompressDecompressFormat in FORMATS do
		local compressed = serde.compress(format, source)
		local decompressed = serde.decompress(format, compressed)

		-- Compressing something should return something else
		if #compressed <= 0 then
			stdio.ewrite(
				string.format(
					"Compressing source using '%s' returned an empty string!\n",
					tostring(format)
				)
			)
			stdio.ewrite(string.format("Source (%d chars long):\n%s\n", #source, tostring(source)))
			failed = true
			continue
		end
		if compressed == source then
			stdio.ewrite(
				string.format(
					"Compressing source using '%s' format did not change contents!\n",
					tostring(format)
				)
			)
			stdio.ewrite(
				string.format(
					"Source (%d chars long):\n%s\nCompressed (%d chars long):\n%s\n",
					#source,
					tostring(source),
					#compressed,
					tostring(compressed)
				)
			)
			failed = true
			continue
		end

		-- Decompressing that something else should return the original source
		if decompressed ~= source then
			stdio.ewrite(
				string.format(
					"Decompressing using '%s' format did not return the source!\n",
					tostring(format)
				)
			)
			stdio.ewrite(
				string.format(
					"Source (%d chars long):\n%s\nCompressed (%d chars long):\n%s\nDecompressed (%d chars long):\n%s\n",
					#source,
					tostring(source),
					#compressed,
					tostring(compressed),
					#decompressed,
					tostring(decompressed)
				)
			)
			failed = true
			continue
		end
	end
end

if failed then
	process.exit(1)
end
