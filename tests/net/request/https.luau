local task = require("@lune/task")

local util = require("./util")
local pass = util.pass

-- These are some public APIs that have, or most likely have, different
-- certificate authorities (CAs), plus are both free to use and stable.
-- This should be enough to ensure that rustls is configured correctly.
local servers = {
	"https://www.googleapis.com/discovery/v1/apis",
	"https://api.cloudflare.com/client/v4/ips",
	"https://azure.microsoft.com/en-us/updates/feed/",
	"https://acme-v02.api.letsencrypt.org/directory",
	"https://ip-ranges.amazonaws.com/ip-ranges.json",
	"https://api.github.com/zen",
	"https://en.wikipedia.org/w/api.php",
	"https://status.godaddy.com/api/v2/summary.json",
}

for _, server in servers do
	task.spawn(function()
		pass("GET", server, server)
	end)
end
