export type EncodeDecodeFormat = "json" | "yaml" | "toml"

--[=[
	@class Serde

	Built-in serialization/deserialization & encoding/decoding functions

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
export type Serde = {
	--[=[
		@within Serde
		@must_use

		Encodes the given value using the given format.

		@param format The format to use
		@param value The value to encode
		@param pretty If the encoded string should be human-readable, including things such as newlines and spaces. Only supported for json and toml formats, and defaults to false
		@return The encoded string
	]=]
	encode: (format: EncodeDecodeFormat, value: any, pretty: boolean?) -> string,
	--[=[
		@within Serde
		@must_use

		Decodes the given string using the given format into a lua value.

		@param format The format to use
		@param encoded The string to decode
		@return The decoded lua value
	]=]
	decode: (format: EncodeDecodeFormat, encoded: string) -> any,
}
