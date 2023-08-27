local DateTime = require("@lune/datetime")

assert(
	string.match(
		DateTime.now():toIsoDate(),
		"(%d%d%d%d)-?(%d?%d?)-?(%d?%d?)T(%d?%d?):(%d?%d?):(%d?%d?)Z$"
	),
	"invalid ISO 8601 date returned by dateTime.toIsoDate()"
)
