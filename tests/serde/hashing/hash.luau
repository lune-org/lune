local serde = require("@lune/serde")

local TEST_INPUT =
	"Luau is a fast, small, safe, gradually typed embeddable scripting language derived from Lua."

local function test_case_hash(algorithm: serde.HashAlgorithm, expected: string)
	assert(
		serde.hash(algorithm, TEST_INPUT) == expected,
		`hashing algorithm '{algorithm}' did not hash test string correctly`
	)
	assert(
		serde.hash(algorithm, buffer.fromstring(TEST_INPUT)) == expected,
		`hashing algorithm '{algorithm}' did not hash test buffer correctly`
	)
end

test_case_hash("blake3", "eccfe3a6696b2a1861c64cc78663cff51301058e5dc22bb6249e7e1e0173d7fe")
test_case_hash("md5", "2aed9e020b49d219dc383884c5bd7acd")
test_case_hash("sha1", "9dce74190857f36e6d3f5e8eb7fe704a74060726")
test_case_hash("sha224", "f7ccd8a5f2697df8470b66f03824e073075292a1fab40d3a2ddc2e83")
test_case_hash("sha256", "f1d149bfd1ea38833ae6abf2a6fece1531532283820d719272e9cf3d9344efea")
test_case_hash(
	"sha384",
	"f6da4b47846c6016a9b32f01b861e45195cf1fa6fc5c9dd2257f7dc1c14092f11001839ec1223c30ab7adb7370812863"
)
test_case_hash(
	"sha512",
	"49fd834fdf3d4eaf4d4aff289acfc24d649f81cee7a5a7940e5c86854e04816f0a97c53f2ca4908969a512ec5ad1dc466422e3928f5ce3da9913959315df807c"
)
test_case_hash("sha3-224", "56a4dd1ff1bd9baff7f8bbe380dbf2c75b073161693f94ebf91aeee5")
test_case_hash("sha3-256", "ee01be10e0dc133cd702999e854b396f40b039d5ba6ddec9d04bf8623ba04dd7")
test_case_hash(
	"sha3-384",
	"e992f31e638b47802f33a4327c0a951823e32491ddcef5af9ce18cff84475c98ced23928d47ef51a8a4299dfe2ece361"
)
test_case_hash(
	"sha3-512",
	"08bd02aca3052b7740de80b8e8b9969dc9059a4bfae197095430e0aa204fbd3afb11731b127559b90c2f7e295835ea844ddbb29baf2fdb1d823046052c120fc9"
)

local failed = pcall(serde.hash, "a random string" :: any, "input that shouldn't be hashed")
assert(failed == false, "serde.hash shouldn't allow invalid algorithms passed to it!")

assert(
	serde.hash("sha256", "\0oh no invalid utf-8\127\0\255")
		== "c18ed3188f9e93f9ecd3582d7398c45120b0b30a0e26243809206228ab711b78",
	"serde.hash should hash invalid UTF-8 just fine"
)
