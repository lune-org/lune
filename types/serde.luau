export type EncodeDecodeFormat = "json" | "yaml" | "toml"

export type CompressDecompressFormat = "brotli" | "gzip" | "lz4" | "zlib"

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
local serde = {}

--[=[
	@within Serde
	@tag must_use

	Encodes the given value using the given format.

	Currently supported formats:

	| Name   | Learn More           |
	|:-------|:---------------------|
	| `json` | https://www.json.org |
	| `yaml` | https://yaml.org     |
	| `toml` | https://toml.io      |

	@param format The format to use
	@param value The value to encode
	@param pretty If the encoded string should be human-readable, including things such as newlines and spaces. Only supported for json and toml formats, and defaults to false
	@return The encoded string
]=]
function serde.encode(format: EncodeDecodeFormat, value: any, pretty: boolean?): string
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Decodes the given string using the given format into a lua value.

	Currently supported formats:

	| Name   | Learn More           |
	|:-------|:---------------------|
	| `json` | https://www.json.org |
	| `yaml` | https://yaml.org     |
	| `toml` | https://toml.io      |

	@param format The format to use
	@param encoded The string to decode
	@return The decoded lua value
]=]
function serde.decode(format: EncodeDecodeFormat, encoded: buffer | string): any
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Compresses the given string using the given format.

	Currently supported formats:

	| Name     | Learn More                        |
	|:---------|:----------------------------------|
	| `brotli` | https://github.com/google/brotli  |
	| `gzip`   | https://www.gnu.org/software/gzip |
	| `lz4`    | https://github.com/lz4/lz4        |
	| `zlib`   | https://www.zlib.net              |

	@param format The format to use
	@param s The string to compress
	@return The compressed string
]=]
function serde.compress(format: CompressDecompressFormat, s: buffer | string): string
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Decompresses the given string using the given format.

	Currently supported formats:

	| Name     | Learn More                        |
	|:---------|:----------------------------------|
	| `brotli` | https://github.com/google/brotli  |
	| `gzip`   | https://www.gnu.org/software/gzip |
	| `lz4`    | https://github.com/lz4/lz4        |
	| `zlib`   | https://www.zlib.net              |

	@param format The format to use
	@param s The string to decompress
	@return The decompressed string
]=]
function serde.decompress(format: CompressDecompressFormat, s: buffer | string): string
	return nil :: any
end

return serde
