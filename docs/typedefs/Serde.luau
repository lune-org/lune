export type EncodeDecodeFormat = "json" | "yaml" | "toml"

export type CompressDecompressFormat = "brotli" | "gzip" | "zlib"

--[=[
	@class Serde

	Built-in library for:
	- serialization & deserialization
	- encoding & decoding
	- compression

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local serde = require("@lune/serde")

	-- Parse different file formats into lua tables
	local someJson = serde.decode("json", fs.readFile("myFile.json"))
	local someToml = serde.decode("toml", fs.readFile("myFile.toml"))
	local someYaml = serde.decode("yaml", fs.readFile("myFile.yaml"))

	-- Write lua tables to files in different formats
	fs.writeFile("myFile.json", serde.encode("json", someJson))
	fs.writeFile("myFile.toml", serde.encode("toml", someToml))
	fs.writeFile("myFile.yaml", serde.encode("yaml", someYaml))
	```
]=]
return {
	--[=[
		@within Serde
		@must_use

		Encodes the given value using the given format.

		@param format The format to use
		@param value The value to encode
		@param pretty If the encoded string should be human-readable, including things such as newlines and spaces. Only supported for json and toml formats, and defaults to false
		@return The encoded string
	]=]
	encode = function(format: EncodeDecodeFormat, value: any, pretty: boolean?): string
		return nil :: any
	end,
	--[=[
		@within Serde
		@must_use

		Decodes the given string using the given format into a lua value.

		@param format The format to use
		@param encoded The string to decode
		@return The decoded lua value
	]=]
	decode = function(format: EncodeDecodeFormat, encoded: string): any
		return nil :: any
	end,
	--[=[
		@within Serde
		@must_use

		Compresses the given string using the given format.

		@param format The format to use
		@param encoded The string to compress
		@return The compressed string
	]=]
	compress = function(format: CompressDecompressFormat, s: string): string
		return nil :: any
	end,
	--[=[
		@within Serde
		@must_use

		Decompresses the given string using the given format.

		@param format The format to use
		@param encoded The string to decompress
		@return The decompressed string
	]=]
	decompress = function(format: CompressDecompressFormat, s: string): string
		return nil :: any
	end,
}
