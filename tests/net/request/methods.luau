local util = require("./util")
local pass = util.pass

-- stylua: ignore start
pass("GET",    "https://httpbingo.org/get",    "Request method - GET")
pass("POST",   "https://httpbingo.org/post",   "Request method - POST")
pass("PATCH",  "https://httpbingo.org/patch",  "Request method - PATCH")
pass("PUT",    "https://httpbingo.org/put",    "Request method - PUT")
pass("DELETE", "https://httpbingo.org/delete", "Request method - DELETE")
