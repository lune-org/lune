local serde = require("@lune/serde")

local INPUT_STRING = "important data to verify the integrity of"

-- if you read this string, you're obligated to keep it a secret! :-)
local SECRET_STRING = "don't read this we operate on the honor system"

local function test_case_hmac(algorithm: serde.HashAlgorithm, expected: string)
	assert(
		serde.hmac(algorithm, INPUT_STRING, SECRET_STRING) == expected,
		`HMAC test for algorithm '{algorithm}' was not correct with string input and string secret`
	)
	assert(
		serde.hmac(algorithm, INPUT_STRING, buffer.fromstring(SECRET_STRING)) == expected,
		`HMAC test for algorithm '{algorithm}' was not correct with string input and buffer secret`
	)
	assert(
		serde.hmac(algorithm, buffer.fromstring(INPUT_STRING), SECRET_STRING) == expected,
		`HMAC test for algorithm '{algorithm}' was not correct with buffer input and string secret`
	)
	assert(
		serde.hmac(algorithm, buffer.fromstring(INPUT_STRING), buffer.fromstring(SECRET_STRING))
			== expected,
		`HMAC test for algorithm '{algorithm}' was not correct with buffer input and buffer secret`
	)
end

test_case_hmac("blake3", "1d9c1b9405567fc565c2c3c6d6c0e170be72a2623d29911f43cb2ce42a373c01")
test_case_hmac("md5", "525379669c93ab5f59d2201024145b79")
test_case_hmac("sha1", "75227c11ed65133788feab0ce7eb8efc8c1f0517")
test_case_hmac("sha224", "47a4857d7d7e1070f47f76558323e03471a918facaf3667037519c29")
test_case_hmac("sha256", "4a4816ab8d4b780a8cf131e34a3df25e4c7bc4eba453cd86e50271aab4e95f45")
test_case_hmac(
	"sha384",
	"6b24aeae78d0f84ec8a4669b24bda1131205535233c344f4262c1f90f29af04c5537612c269bbab8aaca9d8293f4a280"
)
test_case_hmac(
	"sha512",
	"9fffa071241e2f361f8a47a97d251c1d4aae37498efbc49745bf9916d8431f1f361080d350067ed65744d3da42956da33ec57b04901a5fd63a891381a1485ef7"
)
test_case_hmac("sha3-224", "ea102dfaa74aa285555bdba29a04429dfd4e997fa40322459094929f")
test_case_hmac("sha3-256", "17bde287e4692e5b7f281e444efefe92e00696a089570bd6814fd0e03d7763d2")
test_case_hmac(
	"sha3-384",
	"24f68401653d25f36e7ee8635831215f8b46710d4e133c9d1e091e5972c69b0f1d0cb80f5507522fa174d5c4746963c1"
)
test_case_hmac(
	"sha3-512",
	"d2566d156c254ced0101159f97187dbf48d900b8361fa5ebdd7e81409856b1b6a21d93a1fb6e8f700e75620d244ab9e894454030da12d158e9362ffe090d2669"
)

local failed =
	pcall(serde.hmac, "a random string" :: any, "input that shouldn't be hashed", "not a secret")
assert(failed == false, "serde.hmac shouldn't allow invalid algorithms passed to it!")

assert(
	serde.hmac("sha256", "\0oh no invalid utf-8\127\0\255", SECRET_STRING)
		== "1f0d7f65016e9e4c340e3ba23da2483a7dc101ce8a9405f834c23f2e19232c3d",
	"serde.hmac should hash invalid UTF-8 just fine"
)
