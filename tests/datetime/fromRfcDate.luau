local DateTime = require("@lune/datetime")

assert(
	DateTime.fromRfcDate("Fri, 21 Nov 1997 09:55:06 -0600") ~= nil,
	"expected DateTime.fromRfcDate() to return DateTime, got nil"
)

assert(
	DateTime.fromRfcDate("Tue, 1 Jul 2003 10:52:37 +0200") ~= nil,
	"expected DateTime.fromRfcDate() to return DateTime, got nil"
)