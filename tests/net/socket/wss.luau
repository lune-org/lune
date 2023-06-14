local net = require("@lune/net")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- net.socket() will inherently spit out an error if init connection fails, but for
-- the sake of this test, prot. call checks are unnecessary. Also, we're going to use
-- Discord's WebSocket gateway server for soley testing wss, as we don't need to auth
-- or anything for that; we just want to make sure it functions..
print("Connecting to Discord wss gateway..")
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")
print("Connected!") -- Would've errored at net.socket() call before this if init connection err

while not socket.closeCode do
    local nextMessage = socket.next()

    if nextMessage then
        print(`{stdio.style("bold")}Discord:{stdio.style("reset")}`, nextMessage)
        
        print("Will (deliberately) close the socket in 2 seconds..")
        task.wait(2)
        socket.close(1000)
    end
end

print(`Connection to socket closed with closeCode {socket.closeCode}`)
