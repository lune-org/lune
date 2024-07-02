--[=[
	@within Serde
	@interface EncodeDecodeFormat

	A serialization/deserialization format supported by the Serde library.

	Currently supported formats:

	| Name   | Learn More           |
	|:-------|:---------------------|
	| `json` | https://www.json.org |
	| `yaml` | https://yaml.org     |
	| `toml` | https://toml.io      |
]=]
export type EncodeDecodeFormat = "json" | "yaml" | "toml"

--[=[
	@within Serde
	@interface CompressDecompressFormat

	A compression/decompression format supported by the Serde library.

	Currently supported formats:

	| Name     | Learn More                        |
	|:---------|:----------------------------------|
	| `brotli` | https://github.com/google/brotli  |
	| `gzip`   | https://www.gnu.org/software/gzip |
	| `lz4`    | https://github.com/lz4/lz4        |
	| `zlib`   | https://www.zlib.net              |
]=]
export type CompressDecompressFormat = "brotli" | "gzip" | "lz4" | "zlib"

--[=[
	@within Serde
	@interface HashAlgorithm

	A hash algorithm supported by the Serde library.

	Currently supported algorithms:

	| Name       | Learn More                           |
	|:-----------|:-------------------------------------|
	| `md5`      | https://en.wikipedia.org/wiki/MD5    |
	| `sha1`     | https://en.wikipedia.org/wiki/SHA-1  |
	| `sha224`   | https://en.wikipedia.org/wiki/SHA-2  |
	| `sha256`   | https://en.wikipedia.org/wiki/SHA-2  |
	| `sha384`   | https://en.wikipedia.org/wiki/SHA-2  |
	| `sha512`   | https://en.wikipedia.org/wiki/SHA-2  |
	| `sha3-224` | https://en.wikipedia.org/wiki/SHA-3  |
	| `sha3-256` | https://en.wikipedia.org/wiki/SHA-3  |
	| `sha3-384` | https://en.wikipedia.org/wiki/SHA-3  |
	| `sha3-512` | https://en.wikipedia.org/wiki/SHA-3  |
	| `blake3`   | https://en.wikipedia.org/wiki/BLAKE3 |
]=]
export type HashAlgorithm =
	"md5"
	| "sha1"
	| "sha224"
	| "sha256"
	| "sha384"
	| "sha512"
	| "sha3-224"
	| "sha3-256"
	| "sha3-384"
	| "sha3-512"
	| "blake3"

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

	See [`EncodeDecodeFormat`] for a list of supported formats.

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

	See [`EncodeDecodeFormat`] for a list of supported formats.

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

	See [`CompressDecompressFormat`] for a list of supported formats.

	@param format The format to use
	@param s The string to compress
	@param level The compression level to use, clamped to the format's limits. The best compression level is used by default
	@return The compressed string
]=]
function serde.compress(format: CompressDecompressFormat, s: buffer | string, level: number?): string
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Decompresses the given string using the given format.

	See [`CompressDecompressFormat`] for a list of supported formats.

	@param format The format to use
	@param s The string to decompress
	@return The decompressed string
]=]
function serde.decompress(format: CompressDecompressFormat, s: buffer | string): string
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Hashes the given message using the given algorithm
	and returns the hash as a hex string.

	See [`HashAlgorithm`] for a list of supported algorithms.

	@param algorithm The algorithm to use
	@param message The message to hash
	@return The hash as a hex string
]=]
function serde.hash(algorithm: HashAlgorithm, message: string | buffer): string
	return nil :: any
end

--[=[
	@within Serde
	@tag must_use

	Hashes the given message using HMAC with the given secret
	and algorithm, returning the hash as a base64 string.

	See [`HashAlgorithm`] for a list of supported algorithms.

	@param algorithm The algorithm to use
	@param message The message to hash
	@return The hash as a base64 string
]=]
function serde.hmac(
	algorithm: HashAlgorithm,
	message: string | buffer,
	secret: string | buffer
): string
	return nil :: any
end

return serde
