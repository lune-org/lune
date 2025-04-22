local DateTime = require("@lune/datetime")

assert(
	DateTime.fromRfc3339("2023-08-26T16:56:28Z") ~= nil,
	"expected DateTime.fromIsoDate() to return DateTime, got nil"
)

assert(
	DateTime.fromRfc3339("1929-12-05T23:18:23Z") ~= nil,
	"expected DateTime.fromIsoDate() to return DateTime, got nil"
)
