local net = require("@lune/net")
local serde = require("@lune/serde")
local task = require("@lune/task")

-- We're going to use Discord's WebSocket gateway server
-- for testing wss - it does not require auth, and this test
-- only exists to ensure wss (WebSockets with TLS) works correctly
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

while not socket.closeCode do
	local response = socket.next()

	if response then
		local decodeSuccess, decodeMessage = pcall(serde.decode, "json" :: "json", response)
		if not decodeSuccess then
			error(
				string.format(
					"Discord API did not respond with valid JSON!\n%s",
					tostring(decodeMessage)
				)
			)
		end

		-- Close the connection after a second with the success close code
		task.wait(1)
		socket.close(1000)
	end
end
